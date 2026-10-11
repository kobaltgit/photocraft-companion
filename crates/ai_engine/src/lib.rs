use core_contracts::{CompanionError, Result};
use image::{imageops::FilterType, DynamicImage, GrayImage, RgbaImage};
use ort::{inputs, session::Session, value::Tensor};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{debug, info};

pub struct AiEngine {
    pub model_dir: PathBuf,
    session: Mutex<Option<Session>>,
}

impl AiEngine {
    pub fn new(model_dir: impl AsRef<Path>) -> Self {
        Self {
            model_dir: model_dir.as_ref().to_path_buf(),
            session: Mutex::new(None),
        }
    }

    /// Locate RMBG-1.4 onnx model path with priority on self-contained plugin folder
    pub fn get_model_path(&self) -> PathBuf {
        // 1. Direct path inside configured model_dir
        let in_models = self.model_dir.join("models").join("rmbg-1.4.onnx");
        if in_models.exists() {
            return in_models;
        }
        let direct = self.model_dir.join("rmbg-1.4.onnx");
        if direct.exists() {
            return direct;
        }

        // 2. Self-contained plugin folder: plugins/remove_bg/models/rmbg-1.4.onnx
        let plugin_model = PathBuf::from("plugins").join("remove_bg").join("models").join("rmbg-1.4.onnx");
        if plugin_model.exists() {
            return plugin_model;
        }

        // 3. Check relative to executable directory
        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let candidate = parent.join("plugins").join("remove_bg").join("models").join("rmbg-1.4.onnx");
                if candidate.exists() {
                    return candidate;
                }
                if let Some(grandparent) = parent.parent() {
                    let candidate = grandparent.join("plugins").join("remove_bg").join("models").join("rmbg-1.4.onnx");
                    if candidate.exists() {
                        return candidate;
                    }
                }
            }
        }

        // 4. Fallback to PhotoCraftData portable cache
        let portable = PathBuf::from(r"D:\Portable Soft\ArtCraft\photocraft-0.6.0-windows-x64-portable\PhotoCraftData\models\rmbg-1.4.onnx");
        if portable.exists() {
            return portable;
        }
        in_models
    }

    /// Ensure session is loaded
    fn get_or_load_session(&self) -> Result<()> {
        let mut guard = self.session.lock().unwrap();
        if guard.is_none() {
            let model_path = self.get_model_path();
            if !model_path.exists() {
                return Err(CompanionError::AiEngine(format!(
                    "RMBG-1.4 ONNX model not found at {}",
                    model_path.display()
                )));
            }
            info!("Loading RMBG-1.4 model from {}...", model_path.display());
            let session = Session::builder()
                .map_err(|e| CompanionError::AiEngine(format!("Failed to init Session builder: {e}")))?
                .with_execution_providers([
                    ort::ep::DirectML::default().build(),
                    ort::ep::CPU::default().build(),
                ])
                .map_err(|e| CompanionError::AiEngine(format!("Failed to configure execution providers: {e}")))?
                .commit_from_file(&model_path)
                .map_err(|e| CompanionError::AiEngine(format!("Failed to load ONNX model: {e}")))?;
            *guard = Some(session);
        }
        Ok(())
    }

    /// Process RGBA image and return a new RGBA image with background removed
    pub fn remove_background(&self, image: &RgbaImage) -> Result<RgbaImage> {
        self.get_or_load_session()?;

        let orig_w = image.width();
        let orig_h = image.height();

        debug!("Preprocessing image ({}x{}) for RMBG-1.4...", orig_w, orig_h);

        // 1. Resize to 1024x1024 for RMBG-1.4
        let dynamic_img = DynamicImage::ImageRgba8(image.clone());
        let resized = dynamic_img.resize_exact(1024, 1024, FilterType::Triangle).to_rgb8();

        // 2. Build NCHW tensor [1, 3, 1024, 1024] normalized by ([0.5, 0.5, 0.5], [1.0, 1.0, 1.0])
        let channel_stride = 1024 * 1024;
        let mut input_data = vec![0.0f32; 3 * channel_stride];
        for y in 0..1024 {
            for x in 0..1024 {
                let idx = y * 1024 + x;
                let pixel = resized.get_pixel(x as u32, y as u32);
                input_data[idx] = (pixel[0] as f32 / 255.0 - 0.5) / 1.0;
                input_data[channel_stride + idx] = (pixel[1] as f32 / 255.0 - 0.5) / 1.0;
                input_data[2 * channel_stride + idx] = (pixel[2] as f32 / 255.0 - 0.5) / 1.0;
            }
        }

        let input_tensor = Tensor::from_array(([1usize, 3, 1024, 1024], input_data))
            .map_err(|e| CompanionError::AiEngine(format!("Failed to build input tensor: {e}")))?;

        // 3. Inference
        debug!("Running ONNX inference...");
        let mut guard = self.session.lock().unwrap();
        let session = guard.as_mut().ok_or_else(|| CompanionError::AiEngine("Session not loaded".into()))?;

        let outputs = session
            .run(inputs![input_tensor])
            .map_err(|e| CompanionError::AiEngine(format!("Inference failed: {e}")))?;

        // Output tensor shape is [1, 1, 1024, 1024]
        let (_shape, mask_slice) = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| CompanionError::AiEngine(format!("Failed to extract output array: {e}")))?;

        debug!("Postprocessing mask...");
        // 4. Construct 1024x1024 alpha mask
        let mask_raw: Vec<u8> = mask_slice
            .iter()
            .map(|&val| (val.clamp(0.0, 1.0) * 255.0).round() as u8)
            .collect();

        let mask_img = GrayImage::from_raw(1024, 1024, mask_raw)
            .ok_or_else(|| CompanionError::AiEngine("Failed to create mask GrayImage".into()))?;

        // 5. Upscale mask back to original dimensions
        let dynamic_mask = DynamicImage::ImageLuma8(mask_img);
        let resized_mask = dynamic_mask.resize_exact(orig_w, orig_h, FilterType::CatmullRom).to_luma8();

        // 6. Apply mask to original image RGBA
        let mut result_img = image.clone();
        for y in 0..orig_h {
            for x in 0..orig_w {
                let mask_val = resized_mask.get_pixel(x, y)[0] as f32 / 255.0;
                let orig_pixel = image.get_pixel(x, y);
                let new_alpha = (orig_pixel[3] as f32 * mask_val).round() as u8;
                result_img.put_pixel(
                    x,
                    y,
                    image::Rgba([orig_pixel[0], orig_pixel[1], orig_pixel[2], new_alpha]),
                );
            }
        }

        info!("Background removal completed successfully!");
        Ok(result_img)
    }
}

pub struct RemoveBgPlugin {
    manifest: core_contracts::PluginManifest,
    engine: Arc<AiEngine>,
}

impl RemoveBgPlugin {
    pub fn new(engine: Arc<AiEngine>) -> Self {
        let manifest = core_contracts::PluginManifest {
            id: "ai.photocraft.plugins.remove_bg".to_string(),
            name: "AI Remove Background (RMBG-1.4)".to_string(),
            version: "1.0.0".to_string(),
            description: "High-fidelity neural background removal for PhotoCraft using Bria AI RMBG-1.4 model via local ONNX Runtime.".to_string(),
            author: "kobaltGIT".to_string(),
            plugin_type: core_contracts::PluginType::NativeAi,
            default_hotkey: Some("Ctrl+Shift+B".to_string()),
            icon: Some("scissors".to_string()),
            commands: vec![],
            models: vec![core_contracts::ModelRequirement {
                name: "rmbg-1.4.onnx".to_string(),
                required: true,
                size_mb: Some(168),
                url: Some("https://huggingface.co/briaai/RMBG-1.4/resolve/main/onnx/model.onnx".to_string()),
                sha256: None,
            }],
            action_id: Some("plugin:remove_bg".to_string()),
            entrypoint: None,
            enabled: true,
            plugin_dir: Some(std::path::PathBuf::from("plugins/remove_bg")),
        };
        Self { manifest, engine }
    }
}

#[async_trait::async_trait]
impl core_contracts::Plugin for RemoveBgPlugin {
    fn id(&self) -> &str {
        &self.manifest.id
    }

    fn manifest(&self) -> &core_contracts::PluginManifest {
        &self.manifest
    }

    async fn execute(&self, ctx: &core_contracts::PluginContext) -> Result<core_contracts::PluginResult> {
        let source_name = ctx.document.active_layer_name.clone().unwrap_or_else(|| "Layer".to_string());
        let lang = core_contracts::Language::Auto.resolve(None);
        let target_name = lang.format_cutout_name(&source_name);

        let img = ctx.client.copy_active_layer_image().await?;
        let engine_clone = self.engine.clone();
        let img_clone = img.clone();

        let cutout = tokio::task::spawn_blocking(move || {
            engine_clone.remove_background(&img_clone)
        })
        .await
        .map_err(|e| CompanionError::Plugin(format!("Background task join error: {e}")))?
        .map_err(|e| CompanionError::Plugin(format!("Inference error: {e}")))?;

        let paste_res = ctx.client.paste_image_in_place(&cutout).await?;
        if let Some(new_layer_id) = paste_res.get("layer").and_then(|l| l.as_u64()) {
            let _ = ctx.client.rename_layer(new_layer_id, &target_name).await;
        }

        Ok(core_contracts::PluginResult::ok(format!(
            "Layer '{}' successfully extracted to '{}'",
            source_name, target_name
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_contracts::{BridgeOperations, DocumentInfo, Plugin, PluginContext};
    use serde_json::json;
    use std::sync::Mutex;

    #[test]
    fn test_remove_bg_plugin_manifest_metadata() {
        let engine = Arc::new(AiEngine::new("non_existent"));
        let plugin = RemoveBgPlugin::new(engine);

        assert_eq!(plugin.id(), "ai.photocraft.plugins.remove_bg");
        let manifest = plugin.manifest();
        assert_eq!(manifest.name, "AI Remove Background (RMBG-1.4)");
        assert_eq!(manifest.author, "kobaltGIT");
        assert_eq!(manifest.plugin_type, core_contracts::PluginType::NativeAi);
        assert_eq!(manifest.default_hotkey, Some("Ctrl+Shift+B".to_string()));
        assert_eq!(manifest.action_id, Some("plugin:remove_bg".to_string()));
        assert!(manifest.enabled);
        assert_eq!(manifest.models.len(), 1);
        assert_eq!(manifest.models[0].name, "rmbg-1.4.onnx");
    }

    #[test]
    fn test_ai_engine_model_path_resolution() {
        let temp_dir = std::env::temp_dir().join(format!("ai_path_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let models_sub = temp_dir.join("models");
        std::fs::create_dir_all(&models_sub).unwrap();
        let target_model = models_sub.join("rmbg-1.4.onnx");
        std::fs::write(&target_model, b"dummy onnx").unwrap();

        let engine = AiEngine::new(&temp_dir);
        assert_eq!(engine.get_model_path(), target_model);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_ai_engine_missing_model_error() {
        let temp_dir = std::env::temp_dir().join(format!("ai_missing_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let engine = AiEngine::new(&temp_dir);
        // Force session load with a path that definitely doesn't exist
        let dummy_img = RgbaImage::new(10, 10);
        let res = engine.remove_background(&dummy_img);

        // If the machine happens to have global portable model, get_model_path might find it;
        // but if it points strictly to missing without fallback, it errors.
        if !engine.get_model_path().exists() {
            assert!(res.is_err());
            let err = res.unwrap_err().to_string();
            assert!(err.contains("not found"));
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    struct MockBridgeAi {
        renamed: Mutex<Option<(u64, String)>>,
    }

    #[async_trait::async_trait]
    impl BridgeOperations for MockBridgeAi {
        async fn execute_command(&self, _command: &str, _params: serde_json::Value) -> Result<serde_json::Value> {
            Ok(json!({}))
        }

        async fn copy_active_layer_image(&self) -> Result<RgbaImage> {
            Ok(RgbaImage::from_pixel(16, 16, image::Rgba([200, 100, 50, 255])))
        }

        async fn paste_image_in_place(&self, _image: &RgbaImage) -> Result<serde_json::Value> {
            Ok(json!({ "layer": 105 }))
        }

        async fn rename_layer(&self, layer_id: u64, new_name: &str) -> Result<serde_json::Value> {
            *self.renamed.lock().unwrap() = Some((layer_id, new_name.to_string()));
            Ok(json!({ "ok": true }))
        }

        async fn get_document_info(&self) -> Result<DocumentInfo> {
            Ok(DocumentInfo {
                name: "Portrait.psd".to_string(),
                width: 16,
                height: 16,
                active_layer: Some(101),
                active_layer_name: Some("Hero".to_string()),
            })
        }
    }

    #[tokio::test]
    async fn test_remove_bg_plugin_execution_with_model() {
        // Test real or simulated plugin execution
        let plugin_model = std::path::PathBuf::from("plugins").join("remove_bg").join("models").join("rmbg-1.4.onnx");
        if !plugin_model.exists() {
            // Skip model inference if weights are not fetched
            return;
        }

        let engine = Arc::new(AiEngine::new("plugins/remove_bg"));
        let plugin = RemoveBgPlugin::new(engine);

        let mock_bridge = Arc::new(MockBridgeAi {
            renamed: Mutex::new(None),
        });

        let ctx = PluginContext {
            document: DocumentInfo {
                name: "Photo.psd".to_string(),
                width: 16,
                height: 16,
                active_layer: Some(101),
                active_layer_name: Some("Model Layer".to_string()),
            },
            models_dir: PathBuf::from("plugins/remove_bg/models"),
            plugin_dir: PathBuf::from("plugins/remove_bg"),
            client: mock_bridge.clone(),
        };

        let res = plugin.execute(&ctx).await;
        assert!(res.is_ok(), "Plugin execute failed: {:?}", res.err());
        let outcome = res.unwrap();
        assert!(outcome.success);

        let renamed = mock_bridge.renamed.lock().unwrap().clone();
        assert!(renamed.is_some());
        let (layer_id, layer_name) = renamed.unwrap();
        assert_eq!(layer_id, 105);
        assert!(layer_name.contains("Model Layer"));
    }
}



