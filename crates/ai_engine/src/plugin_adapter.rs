use crate::AiEngine;
use core_contracts::{
    AiPipelineConfig, AiTask, CompanionError, Plugin, PluginContext, PluginManifest, PluginResult,
    Result,
};
use image::{GrayImage, Rgba, RgbaImage};
use std::sync::Arc;
use tracing::info;

/// Generic Zero-Core AI Plugin runtime executing any CV neural network via declarative manifest
pub struct GenericAiPlugin {
    manifest: PluginManifest,
    engine: Arc<AiEngine>,
}

impl GenericAiPlugin {
    pub fn new(manifest: PluginManifest, engine: Arc<AiEngine>) -> Self {
        Self { manifest, engine }
    }
}

#[async_trait::async_trait]
impl Plugin for GenericAiPlugin {
    fn id(&self) -> &str {
        &self.manifest.id
    }

    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    async fn execute(&self, ctx: &PluginContext) -> Result<PluginResult> {
        info!(
            "Executing Generic AI plugin '{}' [{}]",
            self.manifest.name, self.manifest.id
        );

        let ai_cfg = self.manifest.ai.clone().unwrap_or(AiPipelineConfig {
            task: AiTask::ImageToImage,
            scale: None,
            tile_size: None,
            tile_pad: None,
            normalize: None,
            input_size: None,
            output_type: None,
        });

        // 1. Locate model
        let model_req = self.manifest.models.first().ok_or_else(|| {
            CompanionError::Plugin(format!(
                "No models specified in manifest for '{}'",
                self.manifest.name
            ))
        })?;

        let model_path = self
            .engine
            .resolve_model_path(&model_req.name, self.manifest.plugin_dir.as_deref())
            .ok_or_else(|| {
                CompanionError::Plugin(format!(
                    "ONNX model '{}' not found. Please download it or install plugin package.",
                    model_req.name
                ))
            })?;

        // 2. Fetch input according to task and selection
        let (input_img, inpaint_mask) = if ai_cfg.task == AiTask::Inpainting {
            ctx.client.copy_active_layer_and_selection_mask().await?
        } else {
            let img = ctx.client.copy_active_layer_image().await?;
            (img, None)
        };

        let source_name = ctx
            .document
            .active_layer_name
            .clone()
            .unwrap_or_else(|| "Layer".to_string());

        let lang = core_contracts::Language::Auto.resolve(None);
        let target_name = match ai_cfg.task {
            AiTask::SegmentationMask => {
                if ai_cfg.output_type.as_ref()
                    == Some(&core_contracts::AiOutputType::GrayscaleLayer)
                {
                    lang.format_depth_name(&source_name)
                } else {
                    lang.format_cutout_name(&source_name)
                }
            }
            AiTask::SuperResolution => lang.format_upscale_name(&source_name),
            AiTask::Inpainting => lang.format_inpaint_name(&source_name),
            AiTask::ImageToImage => format!("{source_name} ({})", self.manifest.name),
        };

        // 3. Execute inference in blocking thread
        let engine_clone = self.engine.clone();
        let ai_cfg_clone = ai_cfg.clone();
        let model_path_clone = model_path.clone();

        let processed_img = tokio::task::spawn_blocking(move || -> Result<RgbaImage> {
            match ai_cfg_clone.task {
                AiTask::ImageToImage => engine_clone.execute_image_to_image(
                    &model_path_clone,
                    &input_img,
                    &ai_cfg_clone,
                ),
                AiTask::SuperResolution => engine_clone.execute_super_resolution(
                    &model_path_clone,
                    &input_img,
                    &ai_cfg_clone,
                ),
                AiTask::SegmentationMask => {
                    let mask = engine_clone.execute_segmentation(
                        &model_path_clone,
                        &input_img,
                        &ai_cfg_clone,
                    )?;
                    if ai_cfg_clone.output_type.as_ref()
                        == Some(&core_contracts::AiOutputType::GrayscaleLayer)
                    {
                        let (w, h) = (mask.width(), mask.height());
                        let mut gray_layer = RgbaImage::new(w, h);
                        for y in 0..h {
                            for x in 0..w {
                                let d = mask.get_pixel(x, y)[0];
                                gray_layer.put_pixel(x, y, Rgba([d, d, d, 255]));
                            }
                        }
                        Ok(gray_layer)
                    } else {
                        let mut cutout = input_img.clone();
                        for y in 0..cutout.height() {
                            for x in 0..cutout.width() {
                                let m_val = mask.get_pixel(x, y)[0] as f32 / 255.0;
                                let p = input_img.get_pixel(x, y);
                                let new_alpha = (p[3] as f32 * m_val).round() as u8;
                                cutout.put_pixel(x, y, Rgba([p[0], p[1], p[2], new_alpha]));
                            }
                        }
                        Ok(cutout)
                    }
                }
                AiTask::Inpainting => {
                    let mask = if let Some(m) = inpaint_mask {
                        m
                    } else {
                        let (w, h) = (input_img.width(), input_img.height());
                        let mut m = GrayImage::new(w, h);
                        for y in 0..h {
                            for x in 0..w {
                                let a = input_img.get_pixel(x, y)[3];
                                if a < 128 {
                                    m.put_pixel(x, y, image::Luma([255]));
                                }
                            }
                        }
                        m
                    };
                    engine_clone.execute_inpainting(
                        &model_path_clone,
                        &input_img,
                        &mask,
                        &ai_cfg_clone,
                    )
                }
            }
        })
        .await
        .map_err(|e| CompanionError::Plugin(format!("AI worker join error: {e}")))?
        .map_err(|e| CompanionError::Plugin(format!("AI inference error: {e}")))?;

        let source_layer_id = ctx.document.active_layer;

        // 4. Output Integration according to output_mode
        let paste_target = if ai_cfg.task != AiTask::Inpainting {
            bake_layer_offset_onto_canvas(
                &processed_img,
                ctx.document.active_layer_bounds,
                ctx.document.width,
                ctx.document.height,
            )
        } else {
            processed_img.clone()
        };

        let paste_res = ctx.client.paste_image_in_place(&paste_target).await?;
        let result_layer_id = paste_res
            .get("layer")
            .or_else(|| paste_res.get("layer_id"))
            .or_else(|| paste_res.get("id"))
            .and_then(|l| l.as_u64());

        if let Some(new_layer_id) = result_layer_id {
            let _ = ctx.client.rename_layer(new_layer_id, &target_name).await;
        }

        let mut res = PluginResult::ok(format!(
            "Plugin '{}' processed '{}' successfully",
            self.manifest.name, target_name
        ));
        res.data = Some(serde_json::json!({
            "source_layer_id": source_layer_id,
            "result_layer_id": result_layer_id,
        }));
        Ok(res)
    }
}

/// If `img` matches the tight bounding box of the source layer and that layer has a
/// non-zero offset on the canvas, expand `img` back onto a full canvas at that offset.
///
/// `paste_image_in_place` relies on the active selection and does not carry the layer
/// position — with no selection it pastes at `(0, 0)`. For Native AI plugins that produce
/// a new layer, `copy_active_layer_image` returns the tight bbox of the source content
/// (e.g. `896×1152` inside a `2480×3508` canvas), so without this expansion the result
/// would be shifted by `active_layer_bounds.xy` toward the top-left corner.
///
/// Returns `img.clone()` untouched if no expansion is needed (missing bounds, zero
/// offset, canvas size unknown, or size mismatch such as super-resolution output).
pub(crate) fn bake_layer_offset_onto_canvas(
    img: &RgbaImage,
    layer_bounds: Option<[i32; 4]>,
    canvas_w: u32,
    canvas_h: u32,
) -> RgbaImage {
    let Some([x, y, bw, bh]) = layer_bounds else {
        return img.clone();
    };
    if canvas_w == 0 || canvas_h == 0 || bw <= 0 || bh <= 0 {
        return img.clone();
    }
    if img.width() != bw as u32 || img.height() != bh as u32 {
        return img.clone();
    }
    if x == 0 && y == 0 {
        return img.clone();
    }
    let mut full = RgbaImage::new(canvas_w, canvas_h);
    image::imageops::overlay(&mut full, img, x as i64, y as i64);
    full
}
