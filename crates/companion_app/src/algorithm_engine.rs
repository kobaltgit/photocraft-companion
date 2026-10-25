use async_trait::async_trait;
use core_contracts::{
    AlgorithmConfig, CompanionError, Plugin, PluginContext, PluginManifest, PluginResult, Result,
};
use image::RgbaImage;
use resvg::tiny_skia;
use resvg::usvg;
use tracing::info;

/// Generic Native Algorithm plugin for CPU-based algorithmic workflows (e.g. VTracer vectorization)
pub struct GenericAlgorithmPlugin {
    manifest: PluginManifest,
}

impl GenericAlgorithmPlugin {
    pub fn new(manifest: PluginManifest) -> Self {
        Self { manifest }
    }
}

#[async_trait]
impl Plugin for GenericAlgorithmPlugin {
    fn id(&self) -> &str {
        &self.manifest.id
    }

    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    async fn execute(&self, ctx: &PluginContext) -> Result<PluginResult> {
        info!(
            "Executing Generic Algorithm plugin '{}' [{}]",
            self.manifest.name, self.manifest.id
        );

        let algo_cfg = self.manifest.algorithm.clone().unwrap_or(AlgorithmConfig {
            task: "vector_trace".to_string(),
            mode: Some("spline".to_string()),
            color_mode: Some("color".to_string()),
            filter_speckle: Some(4),
            corner_threshold: Some(60),
            copy_clipboard: true,
            export_file: true,
        });

        match algo_cfg.task.as_str() {
            "vector_trace" | "vectorize" => self.execute_vector_trace(ctx, &algo_cfg).await,
            unknown => Err(CompanionError::Plugin(format!(
                "Unsupported algorithm task: '{unknown}'"
            ))),
        }
    }
}

impl GenericAlgorithmPlugin {
    async fn execute_vector_trace(
        &self,
        ctx: &PluginContext,
        config: &AlgorithmConfig,
    ) -> Result<PluginResult> {
        // 1. Fetch source active layer image via bridge
        let input_img = ctx.client.copy_active_layer_image().await?;
        let width = input_img.width();
        let height = input_img.height();

        if width == 0 || height == 0 {
            return Err(CompanionError::Plugin(
                "Source image has zero dimensions".into(),
            ));
        }

        // 2. Run vectorization & SVG rasterization in blocking thread pool
        let config_clone = config.clone();
        let (tx, rx) = tokio::sync::oneshot::channel();
        std::thread::Builder::new()
            .name("vtracer_worker".to_string())
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                let res = (|| -> Result<(String, RgbaImage)> {
                    let vtracer_img = vtracer::ColorImage {
                        pixels: input_img.into_raw(),
                        width: width as usize,
                        height: height as usize,
                    };

                    let mut vt_config = vtracer::Config::default();
                    match config_clone.color_mode.as_deref() {
                        Some("binary") => vt_config.color_mode = vtracer::ColorMode::Binary,
                        _ => vt_config.color_mode = vtracer::ColorMode::Color,
                    }
                    if let Some(speckle) = config_clone.filter_speckle {
                        vt_config.filter_speckle = speckle;
                    }
                    if let Some(corner) = config_clone.corner_threshold {
                        vt_config.corner_threshold = corner;
                    }

                    let svg_file = vtracer::convert(vtracer_img, vt_config).map_err(|e| {
                        CompanionError::Plugin(format!("VTracer vectorization failed: {e}"))
                    })?;
                    let svg_string = svg_file.to_string();

                    let opt = usvg::Options::default();
                    let tree = usvg::Tree::from_str(&svg_string, &opt)
                        .map_err(|e| CompanionError::Plugin(format!("SVG parsing failed: {e}")))?;

                    let mut pixmap = tiny_skia::Pixmap::new(width, height).ok_or_else(|| {
                        CompanionError::Plugin("Failed to allocate render pixmap".into())
                    })?;

                    let sx = width as f32 / tree.size().width();
                    let sy = height as f32 / tree.size().height();
                    let transform = tiny_skia::Transform::from_scale(sx, sy);

                    resvg::render(&tree, transform, &mut pixmap.as_mut());

                    let rendered =
                        RgbaImage::from_raw(width, height, pixmap.take()).ok_or_else(|| {
                            CompanionError::Plugin(
                                "Failed to build RgbaImage from rendered SVG".into(),
                            )
                        })?;

                    Ok((svg_string, rendered))
                })();
                let _ = tx.send(res);
            })
            .map_err(|e| {
                CompanionError::Plugin(format!("Failed to spawn vectorizer thread: {e}"))
            })?;

        let (svg_string, rendered_img) = rx
            .await
            .map_err(|e| CompanionError::Plugin(format!("Vectorization task panicked: {e}")))?
            .map_err(|e| CompanionError::Plugin(format!("Vectorization error: {e}")))?;
        // 3. Copy SVG string to system clipboard if requested
        if config.copy_clipboard {
            let svg_to_clip = svg_string.clone();
            tokio::task::spawn_blocking(move || {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(svg_to_clip);
                }
            })
            .await
            .map_err(|e| CompanionError::Plugin(format!("Clipboard worker error: {e}")))?;
        }

        // 4. Export SVG file to disk if requested
        let mut saved_file_path = None;
        if config.export_file {
            let exports_dir = std::path::PathBuf::from("exports");
            let _ = std::fs::create_dir_all(&exports_dir);

            let clean_doc_name = ctx
                .document
                .name
                .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
            let clean_layer_name = ctx
                .document
                .active_layer_name
                .as_deref()
                .unwrap_or("layer")
                .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");

            let file_name = format!("{clean_doc_name}_{clean_layer_name}_vector.svg");
            let target_path = exports_dir.join(&file_name);
            if std::fs::write(&target_path, &svg_string).is_ok() {
                saved_file_path = Some(target_path.to_string_lossy().to_string());
            }
        }

        // 5. Paste rendered vector layer into PhotoCraft document
        let paste_res = ctx.client.paste_image_in_place(&rendered_img).await?;
        let lang = core_contracts::Language::Auto.resolve(None);
        let source_name = ctx.document.active_layer_name.as_deref().unwrap_or("Layer");
        let target_name = lang.format_vectorized_name(source_name);

        if let Some(new_layer_id) = paste_res.get("layer").and_then(|l| l.as_u64()) {
            let _ = ctx.client.rename_layer(new_layer_id, &target_name).await;
        }

        // 6. Execute post commands declared in manifest
        for step in &self.manifest.post_commands {
            let _ = ctx
                .client
                .execute_command(&step.command, step.params.clone())
                .await;
        }

        let mut msg = format!("Vectorized '{source_name}' successfully");
        if let Some(ref path) = saved_file_path {
            msg.push_str(&format!(" (saved to {path})"));
        }
        if config.copy_clipboard {
            msg.push_str(" [SVG copied to clipboard]");
        }

        Ok(PluginResult::ok_with_data(
            msg,
            serde_json::json!({
                "svg_len": svg_string.len(),
                "file": saved_file_path,
                "layer": target_name,
            }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_contracts::{BridgeOperations, DocumentInfo, PluginType};
    use image::{Rgba, RgbaImage};
    use serde_json::Value;
    use std::sync::{Arc, Mutex};

    struct MockBridge {
        copied_img: RgbaImage,
        pasted_img: Mutex<Option<RgbaImage>>,
        renamed_layer: Mutex<Option<String>>,
    }

    impl MockBridge {
        fn new(img: RgbaImage) -> Self {
            Self {
                copied_img: img,
                pasted_img: Mutex::new(None),
                renamed_layer: Mutex::new(None),
            }
        }
    }

    #[async_trait]
    impl BridgeOperations for MockBridge {
        async fn execute_command(&self, _cmd: &str, _params: Value) -> Result<Value> {
            Ok(serde_json::json!({ "ok": true }))
        }

        async fn copy_active_layer_image(&self) -> Result<RgbaImage> {
            Ok(self.copied_img.clone())
        }

        async fn paste_image_in_place(&self, image: &RgbaImage) -> Result<Value> {
            *self.pasted_img.lock().unwrap() = Some(image.clone());
            Ok(serde_json::json!({ "layer": 42 }))
        }

        async fn rename_layer(&self, _layer_id: u64, new_name: &str) -> Result<Value> {
            *self.renamed_layer.lock().unwrap() = Some(new_name.to_string());
            Ok(serde_json::json!({ "ok": true }))
        }

        async fn get_document_info(&self) -> Result<DocumentInfo> {
            Ok(DocumentInfo {
                name: "TestDoc".to_string(),
                width: self.copied_img.width(),
                height: self.copied_img.height(),
                active_layer: Some(1),
                active_layer_name: Some("Logo".to_string()),
                has_selection: false,
                selection_bounds: None,
            })
        }
    }

    #[tokio::test]
    async fn test_generic_algorithm_vector_trace_execution() {
        // Create 16x16 test image with high-contrast shapes
        let mut img = RgbaImage::new(16, 16);
        for y in 0..16 {
            for x in 0..16 {
                if (4..12).contains(&x) && (4..12).contains(&y) {
                    img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
                } else {
                    img.put_pixel(x, y, Rgba([0, 0, 0, 255]));
                }
            }
        }

        let bridge = Arc::new(MockBridge::new(img));
        let ctx = PluginContext {
            document: bridge.get_document_info().await.unwrap(),
            models_dir: std::path::PathBuf::from("models"),
            plugin_dir: std::path::PathBuf::from("plugins/vectorize"),
            client: bridge.clone(),
            sub_action_id: None,
        };

        let manifest = PluginManifest {
            id: "algorithm.photocraft.plugins.vectorize".to_string(),
            name: "Smart Vectorizer".to_string(),
            version: "1.0.0".to_string(),
            description: "Test vectorizer".to_string(),
            plugin_type: PluginType::NativeAlgorithm,
            algorithm: Some(AlgorithmConfig {
                task: "vector_trace".to_string(),
                mode: Some("spline".to_string()),
                color_mode: Some("color".to_string()),
                filter_speckle: Some(1),
                corner_threshold: Some(60),
                copy_clipboard: false, // Don't touch real clipboard in headless tests
                export_file: false,
            }),
            ..Default::default()
        };

        let plugin = GenericAlgorithmPlugin::new(manifest);
        let res = plugin.execute(&ctx).await.unwrap();
        assert!(res.success);
        assert!(res.message.contains("Vectorized"));

        // Verify pasted image has original dimensions
        let pasted = bridge.pasted_img.lock().unwrap().clone().unwrap();
        assert_eq!(pasted.width(), 16);
        assert_eq!(pasted.height(), 16);

        // Verify layer was renamed
        let renamed = bridge.renamed_layer.lock().unwrap().clone().unwrap();
        assert!(renamed.contains("Logo"));
        let detected_lang = core_contracts::Language::Auto.resolve(None);
        let expected_suffix = detected_lang.vectorized_suffix();
        assert!(renamed.contains(expected_suffix) || renamed.contains("Vectorized"));
    }
}
