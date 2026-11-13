use crate::AiEngine;
use core_contracts::{CompanionError, NormalizationMode, Result};
use image::{Rgba, RgbaImage};
use ort::{inputs, session::Session, value::Tensor};
use std::path::Path;
use tracing::{info, warn};

impl AiEngine {
    /// Universal tiled inference for RGB processing (ImageToImage & SuperResolution)
    pub(crate) fn run_tiled_rgb_inference(
        &self,
        model_path: &Path,
        img: &RgbaImage,
        scale: u32,
        tile_size: u32,
        tile_pad: u32,
        normalize: &NormalizationMode,
    ) -> Result<RgbaImage> {
        let w = img.width();
        let h = img.height();
        let out_w = w * scale;
        let out_h = h * scale;

        let is_cpu = self
            .cpu_fallback_models
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains(&model_path.to_path_buf());

        let session_arc = if !is_cpu {
            match self.get_or_create_session(model_path, false) {
                Ok(s) => s,
                Err(e) => {
                    warn!("DirectML session setup failed ({e}), falling back to CPU");
                    self.cpu_fallback_models
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .push(model_path.to_path_buf());
                    self.get_or_create_session(model_path, true)?
                }
            }
        } else {
            self.get_or_create_session(model_path, true)?
        };
        let mut session = session_arc.lock().unwrap_or_else(|e| e.into_inner());

        // If image fits within a single tile, pad to exact tile_size x tile_size, run, and crop back
        if w <= tile_size && h <= tile_size {
            let mut padded = RgbaImage::new(tile_size, tile_size);
            for py in 0..tile_size {
                let src_y = if py < h {
                    py
                } else {
                    let d = py - h + 1;
                    h.saturating_sub(d).min(h - 1)
                };
                for px in 0..tile_size {
                    let src_x = if px < w {
                        px
                    } else {
                        let d = px - w + 1;
                        w.saturating_sub(d).min(w - 1)
                    };
                    padded.put_pixel(px, py, *img.get_pixel(src_x, src_y));
                }
            }

            let full_out =
                self.run_single_rgb_patch_session(&mut session, &padded, scale, normalize)?;
            let cropped = image::imageops::crop_imm(&full_out, 0, 0, out_w, out_h).to_image();
            return Ok(cropped);
        }

        info!(
            "Running tiled inference ({}x{} -> {}x{}, scale={}x, tile={}, pad={})",
            w, h, out_w, out_h, scale, tile_size, tile_pad
        );

        let mut out_accum = vec![0.0f32; (out_w * out_h * 3) as usize];
        let mut weight_accum = vec![0.0f32; (out_w * out_h) as usize];

        let step = tile_size.saturating_sub(tile_pad * 2).max(1);

        let mut y = 0;
        loop {
            let tile_y = if y + tile_size > h {
                h.saturating_sub(tile_size)
            } else {
                y
            };

            let mut x = 0;
            loop {
                let tile_x = if x + tile_size > w {
                    w.saturating_sub(tile_size)
                } else {
                    x
                };

                // Extract exact tile_size x tile_size patch (with boundary clamping if needed)
                let mut patch = RgbaImage::new(tile_size, tile_size);
                for py in 0..tile_size {
                    let sy = (tile_y + py).min(h - 1);
                    for px in 0..tile_size {
                        let sx = (tile_x + px).min(w - 1);
                        patch.put_pixel(px, py, *img.get_pixel(sx, sy));
                    }
                }

                let patch_out =
                    self.run_single_rgb_patch_session(&mut session, &patch, scale, normalize)?;

                let out_patch_w = patch_out.width();
                let out_patch_h = patch_out.height();
                let out_x_base = tile_x * scale;
                let out_y_base = tile_y * scale;
                let pad_scaled = (tile_pad * scale) as f32;

                // Blend patch into accumulator with linear edge feathering
                for py in 0..out_patch_h {
                    let global_y = out_y_base + py;
                    if global_y >= out_h {
                        continue;
                    }

                    let wy = if (py as f32) < pad_scaled {
                        (py as f32 + 1.0) / (pad_scaled + 1.0)
                    } else if py >= out_patch_h.saturating_sub(tile_pad * scale) {
                        ((out_patch_h - py) as f32) / (pad_scaled + 1.0)
                    } else {
                        1.0f32
                    };

                    for px in 0..out_patch_w {
                        let global_x = out_x_base + px;
                        if global_x >= out_w {
                            continue;
                        }

                        let wx = if (px as f32) < pad_scaled {
                            (px as f32 + 1.0) / (pad_scaled + 1.0)
                        } else if px >= out_patch_w.saturating_sub(tile_pad * scale) {
                            ((out_patch_w - px) as f32) / (pad_scaled + 1.0)
                        } else {
                            1.0f32
                        };

                        let weight = wx * wy;
                        let g_idx = (global_y * out_w + global_x) as usize;
                        let p_pixel = patch_out.get_pixel(px, py);

                        out_accum[g_idx * 3] += p_pixel[0] as f32 * weight;
                        out_accum[g_idx * 3 + 1] += p_pixel[1] as f32 * weight;
                        out_accum[g_idx * 3 + 2] += p_pixel[2] as f32 * weight;
                        weight_accum[g_idx] += weight;
                    }
                }

                if tile_x + tile_size >= w {
                    break;
                }
                x += step;
            }

            if tile_y + tile_size >= h {
                break;
            }
            y += step;
        }

        // Construct final RGBA image
        let mut result = RgbaImage::new(out_w, out_h);
        for gy in 0..out_h {
            for gx in 0..out_w {
                let g_idx = (gy * out_w + gx) as usize;
                let w = weight_accum[g_idx].max(1e-5);
                let r = (out_accum[g_idx * 3] / w).clamp(0.0, 255.0).round() as u8;
                let g = (out_accum[g_idx * 3 + 1] / w).clamp(0.0, 255.0).round() as u8;
                let b = (out_accum[g_idx * 3 + 2] / w).clamp(0.0, 255.0).round() as u8;

                // Scale alpha if original had alpha
                let orig_x = (gx / scale).min(img.width() - 1);
                let orig_y = (gy / scale).min(img.height() - 1);
                let orig_a = img.get_pixel(orig_x, orig_y)[3];

                result.put_pixel(gx, gy, Rgba([r, g, b, orig_a]));
            }
        }

        Ok(result)
    }

    /// Helper executing inference on a session and extracting output tensor slice as Vec<f32>
    pub(crate) fn run_single_inference_vector(
        &self,
        model_path: &Path,
        input_data: Vec<f32>,
        shape: [usize; 4],
    ) -> Result<Vec<f32>> {
        let is_cpu = self
            .cpu_fallback_models
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains(&model_path.to_path_buf());

        if !is_cpu {
            if let Ok(session_arc) = self.get_or_create_session(model_path, false) {
                let mut session = session_arc.lock().unwrap_or_else(|e| e.into_inner());
                let tensor = Tensor::from_array((shape, input_data.clone())).map_err(|e| {
                    CompanionError::AiEngine(format!("Failed to build input tensor: {e}"))
                })?;
                if let Ok(outputs) = session.run(inputs![tensor]) {
                    if let Ok((_s, slice)) = outputs[0].try_extract_tensor::<f32>() {
                        return Ok(slice.to_vec());
                    }
                }
                warn!(
                    "Default session run failed, invalidating and remembering model for CPU fallback: {}",
                    model_path.display()
                );
                drop(session);
                self.invalidate_session(model_path, false);
                self.cpu_fallback_models
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(model_path.to_path_buf());
            }
        }

        // CPU execution
        let cpu_session_arc = self.get_or_create_session(model_path, true)?;
        let mut cpu_session = cpu_session_arc.lock().unwrap_or_else(|e| e.into_inner());
        let tensor = Tensor::from_array((shape, input_data)).map_err(|e| {
            CompanionError::AiEngine(format!("Failed to build CPU input tensor: {e}"))
        })?;
        let outputs = cpu_session
            .run(inputs![tensor])
            .map_err(|e| CompanionError::AiEngine(format!("CPU inference failed: {e}")))?;
        let (_s, slice) = outputs[0].try_extract_tensor::<f32>().map_err(|e| {
            CompanionError::AiEngine(format!("Failed to extract tensor from CPU output: {e}"))
        })?;
        Ok(slice.to_vec())
    }

    /// Single patch inference using an already instantiated/warmed Session (fast path for tiled inference)
    pub(crate) fn run_single_rgb_patch_session(
        &self,
        session: &mut Session,
        img: &RgbaImage,
        scale: u32,
        normalize: &NormalizationMode,
    ) -> Result<RgbaImage> {
        let w = img.width() as usize;
        let h = img.height() as usize;
        let channel_stride = w * h;

        let mut input_data = vec![0.0f32; 3 * channel_stride];
        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                let p = img.get_pixel(x as u32, y as u32);
                match normalize {
                    NormalizationMode::ZeroToOne => {
                        input_data[idx] = p[0] as f32 / 255.0;
                        input_data[channel_stride + idx] = p[1] as f32 / 255.0;
                        input_data[2 * channel_stride + idx] = p[2] as f32 / 255.0;
                    }
                    NormalizationMode::Centered => {
                        input_data[idx] = (p[0] as f32 / 255.0 - 0.5) / 1.0;
                        input_data[channel_stride + idx] = (p[1] as f32 / 255.0 - 0.5) / 1.0;
                        input_data[2 * channel_stride + idx] = (p[2] as f32 / 255.0 - 0.5) / 1.0;
                    }
                    NormalizationMode::Raw255 => {
                        input_data[idx] = p[0] as f32;
                        input_data[channel_stride + idx] = p[1] as f32;
                        input_data[2 * channel_stride + idx] = p[2] as f32;
                    }
                    NormalizationMode::ImageNet => {
                        input_data[idx] = (p[0] as f32 / 255.0 - 0.485) / 0.229;
                        input_data[channel_stride + idx] = (p[1] as f32 / 255.0 - 0.456) / 0.224;
                        input_data[2 * channel_stride + idx] =
                            (p[2] as f32 / 255.0 - 0.406) / 0.225;
                    }
                }
            }
        }

        let out_w = (w as u32) * scale;
        let out_h = (h as u32) * scale;
        let out_stride = (out_w * out_h) as usize;

        let tensor = Tensor::from_array(([1usize, 3, h, w], input_data))
            .map_err(|e| CompanionError::AiEngine(format!("Failed to build input tensor: {e}")))?;

        let outputs = session
            .run(inputs![tensor])
            .map_err(|e| CompanionError::AiEngine(format!("Inference run failed: {e}")))?;

        let (_s, out_slice) = outputs[0].try_extract_tensor::<f32>().map_err(|e| {
            CompanionError::AiEngine(format!("Failed to extract tensor from output: {e}"))
        })?;

        let mut result = RgbaImage::new(out_w, out_h);
        for y in 0..out_h {
            for x in 0..out_w {
                let idx = (y * out_w + x) as usize;
                let r_val = out_slice[idx];
                let g_val = out_slice[out_stride + idx];
                let b_val = out_slice[2 * out_stride + idx];

                let (r, g, b) = match normalize {
                    NormalizationMode::Raw255 => (
                        r_val.clamp(0.0, 255.0).round() as u8,
                        g_val.clamp(0.0, 255.0).round() as u8,
                        b_val.clamp(0.0, 255.0).round() as u8,
                    ),
                    _ => (
                        (r_val.clamp(0.0, 1.0) * 255.0).round() as u8,
                        (g_val.clamp(0.0, 1.0) * 255.0).round() as u8,
                        (b_val.clamp(0.0, 1.0) * 255.0).round() as u8,
                    ),
                };

                let orig_x = (x / scale).min(img.width() - 1);
                let orig_y = (y / scale).min(img.height() - 1);
                let a = img.get_pixel(orig_x, orig_y)[3];

                result.put_pixel(x, y, Rgba([r, g, b, a]));
            }
        }

        Ok(result)
    }

    /// Single patch inference for models accepting [1, 3, H, W] RGB tensor
    #[allow(dead_code)]
    pub(crate) fn run_single_rgb_patch(
        &self,
        model_path: &Path,
        img: &RgbaImage,
        scale: u32,
        normalize: &NormalizationMode,
    ) -> Result<RgbaImage> {
        let is_cpu = self
            .cpu_fallback_models
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains(&model_path.to_path_buf());

        let session_arc = if !is_cpu {
            match self.get_or_create_session(model_path, false) {
                Ok(s) => s,
                Err(e) => {
                    warn!("DirectML session setup failed ({e}), falling back to CPU");
                    self.cpu_fallback_models
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .push(model_path.to_path_buf());
                    self.get_or_create_session(model_path, true)?
                }
            }
        } else {
            self.get_or_create_session(model_path, true)?
        };
        let mut session = session_arc.lock().unwrap_or_else(|e| e.into_inner());

        self.run_single_rgb_patch_session(&mut session, img, scale, normalize)
    }
}
