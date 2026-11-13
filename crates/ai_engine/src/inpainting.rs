use crate::AiEngine;
use core_contracts::{CompanionError, NormalizationMode, Result};
use image::{imageops::FilterType, DynamicImage, GrayImage, Rgba, RgbaImage};
use ort::{inputs, value::Tensor};
use std::path::Path;
use tracing::{info, warn};

impl AiEngine {
    /// Computes bounding box [min_x, min_y, max_x, max_y] of nonzero mask pixels
    pub(crate) fn compute_mask_bbox(mask: &GrayImage) -> Option<(u32, u32, u32, u32)> {
        let (w, h) = (mask.width(), mask.height());
        let mut min_x = w;
        let mut min_y = h;
        let mut max_x = 0;
        let mut max_y = 0;
        let mut has_pixels = false;

        for y in 0..h {
            for x in 0..w {
                if mask.get_pixel(x, y)[0] > 0 {
                    has_pixels = true;
                    if x < min_x {
                        min_x = x;
                    }
                    if x > max_x {
                        max_x = x;
                    }
                    if y < min_y {
                        min_y = y;
                    }
                    if y > max_y {
                        max_y = y;
                    }
                }
            }
        }

        if has_pixels {
            Some((min_x, min_y, max_x, max_y))
        } else {
            None
        }
    }

    /// Paradigm 3 Engine: Adaptive Context-Aware Inpainting (High-Fidelity)
    pub(crate) fn run_adaptive_inpaint(
        &self,
        model_path: &Path,
        img: &RgbaImage,
        mask: &GrayImage,
        tile_size: u32,
        _tile_pad: u32,
        norm_mode: &NormalizationMode,
    ) -> Result<RgbaImage> {
        let (w, h) = (img.width(), img.height());

        // 1. Locate bounding box of inpaint mask
        let bbox = match Self::compute_mask_bbox(mask) {
            Some(b) => b,
            None => {
                info!("Inpaint mask is empty, returning original image");
                return Ok(img.clone());
            }
        };

        let (min_x, min_y, max_x, max_y) = bbox;
        let bbox_w = max_x.saturating_sub(min_x) + 1;
        let bbox_h = max_y.saturating_sub(min_y) + 1;

        info!(
            "Inpaint mask bounding box: [{}, {}] to [{}, {}] (size: {}x{}), canvas: {}x{}, model tile_size: {}",
            min_x, min_y, max_x, max_y, bbox_w, bbox_h, w, h, tile_size
        );

        // 2. Compute Context-Aware Region around the mask
        // Inpainting requires surrounding context (sky, snow, background) to plausibly reconstruct holes.
        // If the mask fits comfortably within tile_size, we take an exact 1:1 window of size tile_size x tile_size
        // without ANY scaling, preserving 100% micro-sharpness.
        // If the mask is larger than tile_size, we expand the bbox by a 25% padding margin and process that local crop.
        let (crop_x, crop_y, crop_w, crop_h) = if bbox_w <= tile_size && bbox_h <= tile_size {
            let cx = (min_x + max_x) / 2;
            let cy = (min_y + max_y) / 2;
            let half = tile_size / 2;

            let px = if w <= tile_size {
                0
            } else {
                cx.saturating_sub(half).min(w - tile_size)
            };
            let py = if h <= tile_size {
                0
            } else {
                cy.saturating_sub(half).min(h - tile_size)
            };
            let pw = tile_size.min(w - px);
            let ph = tile_size.min(h - py);
            (px, py, pw, ph)
        } else {
            // Expand bbox by 25% on each side for surrounding texture context
            let pad_x = (bbox_w as f32 * 0.25).max(32.0) as u32;
            let pad_y = (bbox_h as f32 * 0.25).max(32.0) as u32;

            let x0 = min_x.saturating_sub(pad_x);
            let y0 = min_y.saturating_sub(pad_y);
            let x1 = (max_x + pad_x).min(w - 1);
            let y1 = (max_y + pad_y).min(h - 1);

            let cw = x1.saturating_sub(x0) + 1;
            let ch = y1.saturating_sub(y0) + 1;
            (x0, y0, cw, ch)
        };

        info!(
            "Inpaint local context window: at ({}, {}) size {}x{} (original canvas {}x{})",
            crop_x, crop_y, crop_w, crop_h, w, h
        );

        // Extract native RGB and Mask for this localized context window
        let context_img = image::imageops::crop_imm(img, crop_x, crop_y, crop_w, crop_h).to_image();
        let context_mask =
            image::imageops::crop_imm(mask, crop_x, crop_y, crop_w, crop_h).to_image();

        // Run neural inpainting on the context window (scaled to model tile_size and restored to crop_w x crop_h)
        let infilled_crop = self.run_single_inpaint_patch(
            model_path,
            &context_img,
            &context_mask,
            tile_size,
            norm_mode,
        )?;

        // If the context window was scaled (because it was larger than tile_size),
        // we recover high-frequency sharpness using Catmull-Rom interpolation + adaptive unsharp filter
        let restored_crop = if infilled_crop.width() == crop_w && infilled_crop.height() == crop_h {
            infilled_crop
        } else {
            let mut enhanced = DynamicImage::ImageRgba8(infilled_crop)
                .resize_exact(crop_w, crop_h, FilterType::CatmullRom)
                .to_rgba8();

            // Apply unsharp mask enhancement to eliminate remaining soft blur and restore crisp edges
            let dyn_img = DynamicImage::ImageRgba8(enhanced.clone());
            let blurred = dyn_img.blur(1.5f32).to_rgba8();
            for y in 0..crop_h {
                for x in 0..crop_w {
                    let orig = enhanced.get_pixel(x, y);
                    let blur = blurred.get_pixel(x, y);
                    // Unsharp high-pass contrast boost (amount = 1.6)
                    let r = (orig[0] as f32 + 1.6 * (orig[0] as f32 - blur[0] as f32))
                        .clamp(0.0, 255.0) as u8;
                    let g = (orig[1] as f32 + 1.6 * (orig[1] as f32 - blur[1] as f32))
                        .clamp(0.0, 255.0) as u8;
                    let b = (orig[2] as f32 + 1.6 * (orig[2] as f32 - blur[2] as f32))
                        .clamp(0.0, 255.0) as u8;
                    enhanced.put_pixel(x, y, Rgba([r, g, b, orig[3]]));
                }
            }

            enhanced
        };

        // Blend generated pixels back into canvas STRICTLY inside the original mask
        // Pixels outside the mask retain 100% untouched native original quality!
        let mut result = img.clone();
        for cy in 0..crop_h {
            let gy = crop_y + cy;
            for cx in 0..crop_w {
                let gx = crop_x + cx;
                let m = mask.get_pixel(gx, gy)[0];
                if m > 0 {
                    let blend_factor = m as f32 / 255.0;
                    let gen_p = restored_crop.get_pixel(cx, cy);
                    let orig_p = img.get_pixel(gx, gy);

                    let r = ((orig_p[0] as f32 * (1.0 - blend_factor))
                        + (gen_p[0] as f32 * blend_factor))
                        .round() as u8;
                    let g = ((orig_p[1] as f32 * (1.0 - blend_factor))
                        + (gen_p[1] as f32 * blend_factor))
                        .round() as u8;
                    let b = ((orig_p[2] as f32 * (1.0 - blend_factor))
                        + (gen_p[2] as f32 * blend_factor))
                        .round() as u8;
                    result.put_pixel(gx, gy, Rgba([r, g, b, orig_p[3]]));
                }
            }
        }

        Ok(result)
    }

    /// Single ONNX patch inference for inpainting on exact tile_size x tile_size
    pub(crate) fn run_single_inpaint_patch(
        &self,
        model_path: &Path,
        img: &RgbaImage,
        mask: &GrayImage,
        tile_size: u32,
        norm_mode: &NormalizationMode,
    ) -> Result<RgbaImage> {
        let (patch_w, patch_h) = (img.width(), img.height());

        // Ensure input is exactly tile_size x tile_size
        let (dyn_img, dyn_mask) = if patch_w == tile_size && patch_h == tile_size {
            (
                DynamicImage::ImageRgba8(img.clone()),
                DynamicImage::ImageLuma8(mask.clone()),
            )
        } else {
            (
                DynamicImage::ImageRgba8(img.clone()).resize_exact(
                    tile_size,
                    tile_size,
                    FilterType::Triangle,
                ),
                DynamicImage::ImageLuma8(mask.clone()).resize_exact(
                    tile_size,
                    tile_size,
                    FilterType::Nearest,
                ),
            )
        };

        let rgb = dyn_img.to_rgb8();
        let m_img = dyn_mask.to_luma8();

        let channel_stride = (tile_size * tile_size) as usize;
        let mut img_data = vec![0.0f32; 3 * channel_stride];
        let mut mask_data = vec![0.0f32; channel_stride];

        for y in 0..tile_size {
            for x in 0..tile_size {
                let idx = (y * tile_size + x) as usize;
                let p = rgb.get_pixel(x, y);
                match norm_mode {
                    NormalizationMode::Raw255 => {
                        img_data[idx] = p[0] as f32;
                        img_data[channel_stride + idx] = p[1] as f32;
                        img_data[2 * channel_stride + idx] = p[2] as f32;
                    }
                    _ => {
                        img_data[idx] = p[0] as f32 / 255.0;
                        img_data[channel_stride + idx] = p[1] as f32 / 255.0;
                        img_data[2 * channel_stride + idx] = p[2] as f32 / 255.0;
                    }
                }
                let m = m_img.get_pixel(x, y)[0];
                mask_data[idx] = if m > 127 { 1.0f32 } else { 0.0f32 };
            }
        }

        let is_cpu = self
            .cpu_fallback_models
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains(&model_path.to_path_buf());

        let ts = tile_size as usize;

        let out_slice = if !is_cpu {
            let res = (|| -> Result<Vec<f32>> {
                let session_arc = self.get_or_create_session(model_path, false)?;
                let mut session = session_arc.lock().unwrap_or_else(|e| e.into_inner());
                let img_tensor = Tensor::from_array(([1usize, 3, ts, ts], img_data.clone()))
                    .map_err(|e| {
                        CompanionError::AiEngine(format!("Failed to create image tensor: {e}"))
                    })?;
                let mask_tensor = Tensor::from_array(([1usize, 1, ts, ts], mask_data.clone()))
                    .map_err(|e| {
                        CompanionError::AiEngine(format!("Failed to create mask tensor: {e}"))
                    })?;
                let outputs = session
                    .run(inputs!["image" => img_tensor, "mask" => mask_tensor])
                    .map_err(|e| CompanionError::AiEngine(format!("Inference failed: {e}")))?;
                let (_s, slice) = outputs[0].try_extract_tensor::<f32>().map_err(|e| {
                    CompanionError::AiEngine(format!("Failed to extract tensor: {e}"))
                })?;
                Ok(slice.to_vec())
            })();

            match res {
                Ok(v) => v,
                Err(e) => {
                    warn!("Inpaint default execution failed ({e}), invalidating and falling back to CPU...");
                    self.invalidate_session(model_path, false);
                    self.cpu_fallback_models
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .push(model_path.to_path_buf());
                    let cpu_session_arc = self.get_or_create_session(model_path, true)?;
                    let mut cpu_session = cpu_session_arc.lock().unwrap_or_else(|e| e.into_inner());
                    let img_tensor =
                        Tensor::from_array(([1usize, 3, ts, ts], img_data)).map_err(|e| {
                            CompanionError::AiEngine(format!("Failed to create image tensor: {e}"))
                        })?;
                    let mask_tensor = Tensor::from_array(([1usize, 1, ts, ts], mask_data))
                        .map_err(|e| {
                            CompanionError::AiEngine(format!("Failed to create mask tensor: {e}"))
                        })?;
                    let outputs = cpu_session
                        .run(inputs!["image" => img_tensor, "mask" => mask_tensor])
                        .map_err(|e| {
                            CompanionError::AiEngine(format!("CPU inference failed: {e}"))
                        })?;
                    let (_s, slice) = outputs[0].try_extract_tensor::<f32>().map_err(|e| {
                        CompanionError::AiEngine(format!("Failed to extract tensor: {e}"))
                    })?;
                    slice.to_vec()
                }
            }
        } else {
            let cpu_session_arc = self.get_or_create_session(model_path, true)?;
            let mut cpu_session = cpu_session_arc.lock().unwrap_or_else(|e| e.into_inner());
            let img_tensor = Tensor::from_array(([1usize, 3, ts, ts], img_data)).map_err(|e| {
                CompanionError::AiEngine(format!("Failed to create image tensor: {e}"))
            })?;
            let mask_tensor =
                Tensor::from_array(([1usize, 1, ts, ts], mask_data)).map_err(|e| {
                    CompanionError::AiEngine(format!("Failed to create mask tensor: {e}"))
                })?;
            let outputs = cpu_session
                .run(inputs!["image" => img_tensor, "mask" => mask_tensor])
                .map_err(|e| CompanionError::AiEngine(format!("CPU inference failed: {e}")))?;
            let (_s, slice) = outputs[0]
                .try_extract_tensor::<f32>()
                .map_err(|e| CompanionError::AiEngine(format!("Failed to extract tensor: {e}")))?;
            slice.to_vec()
        };

        // Reconstruct tile_size x tile_size RGB
        let mut out_patch = RgbaImage::new(tile_size, tile_size);
        for y in 0..tile_size {
            for x in 0..tile_size {
                let idx = (y * tile_size + x) as usize;
                let r_val = out_slice[idx];
                let g_val = out_slice[channel_stride + idx];
                let b_val = out_slice[2 * channel_stride + idx];

                let r = r_val.clamp(0.0, 255.0).round() as u8;
                let g = g_val.clamp(0.0, 255.0).round() as u8;
                let b = b_val.clamp(0.0, 255.0).round() as u8;
                out_patch.put_pixel(x, y, Rgba([r, g, b, 255]));
            }
        }

        Ok(out_patch)
    }
}
