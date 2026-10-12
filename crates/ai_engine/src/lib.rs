use core_contracts::{
    AiPipelineConfig, AiTask, CompanionError, NormalizationMode, Plugin, PluginContext,
    PluginManifest, PluginResult, Result,
};
use image::{imageops::FilterType, DynamicImage, GrayImage, Rgba, RgbaImage};
use ort::{inputs, session::Session, value::Tensor};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

/// Thread-safe runtime for ONNX sessions with adaptive DirectML/CPU fallback
pub struct AiEngine {
    pub model_dir: PathBuf,
    cpu_fallback_models: Mutex<Vec<PathBuf>>,
}

impl AiEngine {
    pub fn new(model_dir: impl AsRef<Path>) -> Self {
        Self {
            model_dir: model_dir.as_ref().to_path_buf(),
            cpu_fallback_models: Mutex::new(Vec::new()),
        }
    }

    /// Resolve absolute or relative path to a model file across plugin dirs, app cache, and global dirs
    pub fn resolve_model_path(
        &self,
        model_name: &str,
        plugin_dir: Option<&Path>,
    ) -> Option<PathBuf> {
        // 1. Check inside plugin directory: plugin_dir/models/<name> or plugin_dir/<name>
        if let Some(p_dir) = plugin_dir {
            let direct = p_dir.join("models").join(model_name);
            if direct.exists() {
                return Some(direct);
            }
            let flat = p_dir.join(model_name);
            if flat.exists() {
                return Some(flat);
            }
        }

        // 2. Direct path inside configured model_dir / PhotoCraftData
        let in_models = self.model_dir.join("models").join(model_name);
        if in_models.exists() {
            return Some(in_models);
        }
        let direct = self.model_dir.join(model_name);
        if direct.exists() {
            return Some(direct);
        }

        // 3. Check relative to plugins/ directory in CWD or exe location
        let in_plugins = PathBuf::from("plugins").join("models").join(model_name);
        if in_plugins.exists() {
            return Some(in_plugins);
        }

        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let candidate = parent.join("models").join(model_name);
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }

        // 4. Fallback to PhotoCraft portable models directory
        let portable = PathBuf::from(
            r"D:\Portable Soft\ArtCraft\photocraft-0.6.0-windows-x64-portable\PhotoCraftData\models",
        )
        .join(model_name);
        if portable.exists() {
            return Some(portable);
        }

        None
    }

    /// Load ONNX session with DirectML or CPU execution providers
    pub fn create_session(&self, model_path: &Path, force_cpu: bool) -> Result<Session> {
        let is_cpu_only = force_cpu
            || self
                .cpu_fallback_models
                .lock()
                .unwrap()
                .contains(&model_path.to_path_buf());

        let builder = Session::builder().map_err(|e| {
            CompanionError::AiEngine(format!("Failed to init Session builder: {e}"))
        })?;

        #[cfg(windows)]
        let mut builder = if !is_cpu_only {
            match builder.with_execution_providers([
                ort::ep::DirectML::default().build(),
                ort::ep::CPU::default().build(),
            ]) {
                Ok(b) => {
                    debug!("Configured DirectML + CPU execution provider");
                    b
                }
                Err(e) => {
                    warn!("DirectML provider setup failed ({e}), falling back to CPU");
                    Session::builder()
                        .map_err(|e| {
                            CompanionError::AiEngine(format!("Failed to re-init builder: {e}"))
                        })?
                        .with_execution_providers([ort::ep::CPU::default().build()])
                        .map_err(|e| CompanionError::AiEngine(format!("CPU provider error: {e}")))?
                }
            }
        } else {
            builder
                .with_execution_providers([ort::ep::CPU::default().build()])
                .map_err(|e| CompanionError::AiEngine(format!("CPU provider error: {e}")))?
        };

        #[cfg(not(windows))]
        let mut builder = builder
            .with_execution_providers([ort::ep::CPU::default().build()])
            .map_err(|e| CompanionError::AiEngine(format!("CPU provider error: {e}")))?;

        builder.commit_from_file(model_path).map_err(|e| {
            CompanionError::AiEngine(format!(
                "Failed to load ONNX model from {}: {e}",
                model_path.display()
            ))
        })
    }

    /// Paradigm 1: Image-to-Image (Denoise, Deblur, Colorization, 1:1 Filters)
    pub fn execute_image_to_image(
        &self,
        model_path: &Path,
        img: &RgbaImage,
        config: &AiPipelineConfig,
    ) -> Result<RgbaImage> {
        let tile_size = config.tile_size.unwrap_or(512);
        let tile_pad = config.tile_pad.unwrap_or(16);

        self.run_tiled_rgb_inference(
            model_path,
            img,
            1,
            tile_size,
            tile_pad,
            config
                .normalize
                .as_ref()
                .unwrap_or(&NormalizationMode::ZeroToOne),
        )
    }

    /// Paradigm 2: Super-Resolution (2x, 4x Upscaling via Real-ESRGAN etc.)
    pub fn execute_super_resolution(
        &self,
        model_path: &Path,
        img: &RgbaImage,
        config: &AiPipelineConfig,
    ) -> Result<RgbaImage> {
        let scale = config.scale.unwrap_or(2);
        let tile_size = config.tile_size.unwrap_or(512);
        let tile_pad = config.tile_pad.unwrap_or(16);

        self.run_tiled_rgb_inference(
            model_path,
            img,
            scale,
            tile_size,
            tile_pad,
            config
                .normalize
                .as_ref()
                .unwrap_or(&NormalizationMode::ZeroToOne),
        )
    }

    /// Paradigm 3: Inpainting (LaMa Generative Fill / Object Eraser)
    pub fn execute_inpainting(
        &self,
        model_path: &Path,
        img: &RgbaImage,
        mask: &GrayImage,
        config: &AiPipelineConfig,
    ) -> Result<RgbaImage> {
        let norm_mode = config
            .normalize
            .as_ref()
            .unwrap_or(&NormalizationMode::ZeroToOne);
        let tile_size = config.tile_size.unwrap_or(512);
        let tile_pad = config.tile_pad.unwrap_or(32);

        self.run_adaptive_inpaint(model_path, img, mask, tile_size, tile_pad, norm_mode)
    }

    /// Paradigm 4: Segmentation Mask (RMBG Background Removal)
    pub fn execute_segmentation(
        &self,
        model_path: &Path,
        img: &RgbaImage,
        config: &AiPipelineConfig,
    ) -> Result<GrayImage> {
        let norm_mode = config
            .normalize
            .as_ref()
            .unwrap_or(&NormalizationMode::Centered);
        self.run_segmentation_inference(model_path, img, norm_mode)
    }

    /// Universal tiled inference for RGB processing (ImageToImage & SuperResolution)
    fn run_tiled_rgb_inference(
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

        // If image fits comfortably in a single tile, run directly
        if w <= tile_size && h <= tile_size {
            return self.run_single_rgb_patch(model_path, img, scale, normalize);
        }

        info!(
            "Running tiled inference ({}x{} -> {}x{}, scale={}x, tile={}, pad={})",
            w, h, out_w, out_h, scale, tile_size, tile_pad
        );

        let mut out_accum = vec![0.0f32; (out_w * out_h * 3) as usize];
        let mut weight_accum = vec![0.0f32; (out_w * out_h) as usize];

        let step = tile_size.saturating_sub(tile_pad * 2).max(1);

        let mut y = 0;
        while y < h {
            let mut x = 0;
            let tile_h = tile_size.min(h - y);
            while x < w {
                let tile_w = tile_size.min(w - x);

                // Extract patch
                let patch = image::imageops::crop_imm(img, x, y, tile_w, tile_h).to_image();
                let patch_out = self.run_single_rgb_patch(model_path, &patch, scale, normalize)?;

                let out_patch_w = patch_out.width();
                let out_patch_h = patch_out.height();
                let out_x_base = x * scale;
                let out_y_base = y * scale;
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

                if x + tile_w >= w {
                    break;
                }
                x += step;
            }

            if y + tile_h >= h {
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
    fn run_single_inference_vector(
        &self,
        model_path: &Path,
        input_data: Vec<f32>,
        shape: [usize; 4],
    ) -> Result<Vec<f32>> {
        let is_cpu = self
            .cpu_fallback_models
            .lock()
            .unwrap()
            .contains(&model_path.to_path_buf());

        if !is_cpu {
            if let Ok(mut session) = self.create_session(model_path, false) {
                let tensor = Tensor::from_array((shape, input_data.clone())).map_err(|e| {
                    CompanionError::AiEngine(format!("Failed to build input tensor: {e}"))
                })?;
                if let Ok(outputs) = session.run(inputs![tensor]) {
                    if let Ok((_s, slice)) = outputs[0].try_extract_tensor::<f32>() {
                        return Ok(slice.to_vec());
                    }
                }
                warn!(
                    "Default session run failed, remembering model for CPU fallback: {}",
                    model_path.display()
                );
                self.cpu_fallback_models
                    .lock()
                    .unwrap()
                    .push(model_path.to_path_buf());
            }
        }

        // CPU execution
        let mut cpu_session = self.create_session(model_path, true)?;
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

    /// Single patch inference for models accepting [1, 3, H, W] RGB tensor
    fn run_single_rgb_patch(
        &self,
        model_path: &Path,
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
                }
            }
        }

        let out_w = (w as u32) * scale;
        let out_h = (h as u32) * scale;
        let out_stride = (out_w * out_h) as usize;

        let out_slice =
            self.run_single_inference_vector(model_path, input_data, [1usize, 3, h, w])?;

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

    /// Computes bounding box [min_x, min_y, max_x, max_y] of nonzero mask pixels
    fn compute_mask_bbox(mask: &GrayImage) -> Option<(u32, u32, u32, u32)> {
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
    fn run_adaptive_inpaint(
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
        // we recover high-frequency sharpness using Real-ESRGAN super-resolution + adaptive unsharp filter
        let restored_crop = if infilled_crop.width() == crop_w && infilled_crop.height() == crop_h {
            infilled_crop
        } else {
            // Check if Real-ESRGAN model is available to restore neural micro-textures
            let upscaler_path = self.resolve_model_path("realesrgan-x2.onnx", None);
            let mut enhanced = if let Some(ref sr_path) = upscaler_path {
                info!("Restoring inpaint micro-sharpness via Real-ESRGAN super-resolution...");
                match self.execute_super_resolution(
                    sr_path,
                    &infilled_crop,
                    &AiPipelineConfig {
                        task: AiTask::SuperResolution,
                        scale: Some(2),
                        tile_size: Some(512),
                        tile_pad: Some(16),
                        normalize: Some(NormalizationMode::ZeroToOne),
                    },
                ) {
                    Ok(sr_img) => DynamicImage::ImageRgba8(sr_img)
                        .resize_exact(crop_w, crop_h, FilterType::CatmullRom)
                        .to_rgba8(),
                    Err(e) => {
                        warn!(
                            "Real-ESRGAN detail recovery failed ({e}), using Catmull-Rom fallback"
                        );
                        DynamicImage::ImageRgba8(infilled_crop)
                            .resize_exact(crop_w, crop_h, FilterType::CatmullRom)
                            .to_rgba8()
                    }
                }
            } else {
                DynamicImage::ImageRgba8(infilled_crop)
                    .resize_exact(crop_w, crop_h, FilterType::CatmullRom)
                    .to_rgba8()
            };

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
    fn run_single_inpaint_patch(
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
            .unwrap()
            .contains(&model_path.to_path_buf());

        let ts = tile_size as usize;

        let out_slice = if !is_cpu {
            let res = (|| -> Result<Vec<f32>> {
                let mut session = self.create_session(model_path, false)?;
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
                    warn!("Inpaint default execution failed ({e}), falling back to CPU...");
                    self.cpu_fallback_models
                        .lock()
                        .unwrap()
                        .push(model_path.to_path_buf());
                    let mut cpu_session = self.create_session(model_path, true)?;
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
            let mut cpu_session = self.create_session(model_path, true)?;
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

    /// Paradigm 4 Engine: Segmentation Mask (RMBG-1.4)
    fn run_segmentation_inference(
        &self,
        model_path: &Path,
        img: &RgbaImage,
        norm_mode: &NormalizationMode,
    ) -> Result<GrayImage> {
        let (orig_w, orig_h) = (img.width(), img.height());

        // RMBG-1.4 expects [1, 3, 1024, 1024]
        let dynamic_img = DynamicImage::ImageRgba8(img.clone());
        let resized = dynamic_img
            .resize_exact(1024, 1024, FilterType::Triangle)
            .to_rgb8();

        let channel_stride = 1024 * 1024;
        let mut input_data = vec![0.0f32; 3 * channel_stride];
        for y in 0..1024 {
            for x in 0..1024 {
                let idx = y * 1024 + x;
                let pixel = resized.get_pixel(x as u32, y as u32);
                match norm_mode {
                    NormalizationMode::Centered => {
                        input_data[idx] = (pixel[0] as f32 / 255.0 - 0.5) / 1.0;
                        input_data[channel_stride + idx] = (pixel[1] as f32 / 255.0 - 0.5) / 1.0;
                        input_data[2 * channel_stride + idx] =
                            (pixel[2] as f32 / 255.0 - 0.5) / 1.0;
                    }
                    _ => {
                        input_data[idx] = pixel[0] as f32 / 255.0;
                        input_data[channel_stride + idx] = pixel[1] as f32 / 255.0;
                        input_data[2 * channel_stride + idx] = pixel[2] as f32 / 255.0;
                    }
                }
            }
        }

        let mask_slice =
            self.run_single_inference_vector(model_path, input_data, [1usize, 3, 1024, 1024])?;

        let mask_raw: Vec<u8> = mask_slice
            .iter()
            .map(|&val| (val.clamp(0.0, 1.0) * 255.0).round() as u8)
            .collect();

        let mask_1024 = GrayImage::from_raw(1024, 1024, mask_raw)
            .ok_or_else(|| CompanionError::AiEngine("Failed to create mask GrayImage".into()))?;

        let resized_mask = DynamicImage::ImageLuma8(mask_1024)
            .resize_exact(orig_w, orig_h, FilterType::CatmullRom)
            .to_luma8();

        Ok(resized_mask)
    }
}

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
        });

        // 1. Locate model path
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

        // 2. Fetch input according to manifest.input
        // 2. Fetch input according to task and selection
        let (input_img, inpaint_mask) =
            if ai_cfg.task == AiTask::Inpainting && ctx.document.has_selection {
                // Inpainting requires both the full background context and the exact cutout alpha mask
                let cutout = ctx.client.copy_active_layer_image().await?;
                let _ = ctx
                    .client
                    .execute_command("select.deselect", serde_json::json!({}))
                    .await;
                let full_img = ctx.client.copy_active_layer_image().await?;
                let _ = ctx
                    .client
                    .execute_command("select.reselect", serde_json::json!({}))
                    .await;

                let (w, h) = (full_img.width(), full_img.height());
                let mut mask = GrayImage::new(w, h);
                let (bx0, by0) = ctx
                    .document
                    .selection_bounds
                    .map(|b| (b[0].max(0) as u32, b[1].max(0) as u32))
                    .unwrap_or((0, 0));

                for cy in 0..cutout.height() {
                    for cx in 0..cutout.width() {
                        if cutout.get_pixel(cx, cy)[3] > 0 {
                            let tx = bx0 + cx;
                            let ty = by0 + cy;
                            if tx < w && ty < h {
                                mask.put_pixel(tx, ty, image::Luma([255]));
                            }
                        }
                    }
                }

                (full_img, Some(mask))
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
            AiTask::SegmentationMask => lang.format_cutout_name(&source_name),
            AiTask::SuperResolution => lang.format_upscale_name(&source_name),
            AiTask::Inpainting => lang.format_inpaint_name(&source_name),
            AiTask::ImageToImage => format!("{source_name} (AI)"),
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

        // 4. Output Integration according to output_mode
        let paste_res = ctx.client.paste_image_in_place(&processed_img).await?;
        if let Some(new_layer_id) = paste_res.get("layer").and_then(|l| l.as_u64()) {
            let _ = ctx.client.rename_layer(new_layer_id, &target_name).await;
        }

        // 5. Execute post_commands sequentially
        for (idx, step) in self.manifest.post_commands.iter().enumerate() {
            debug!(
                "Executing post-command [{}/{}]: {}",
                idx + 1,
                self.manifest.post_commands.len(),
                step.command
            );
            if let Err(e) = ctx
                .client
                .execute_command(&step.command, step.params.clone())
                .await
            {
                warn!("Post-command '{}' warning: {}", step.command, e);
            }
        }

        Ok(PluginResult::ok(format!(
            "Plugin '{}' processed '{}' successfully",
            self.manifest.name, target_name
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_contracts::{BridgeOperations, DocumentInfo, ModelRequirement, PluginType};
    use serde_json::json;

    #[test]
    fn test_generic_ai_plugin_manifest_binding() {
        let engine = Arc::new(AiEngine::new("test_models"));
        let manifest = PluginManifest {
            id: "ai.photocraft.plugins.upscale".to_string(),
            name: "AI Super-Resolution (2x)".to_string(),
            version: "1.0.0".to_string(),
            description: "2x Upscale".to_string(),
            author: "kobaltGIT".to_string(),
            plugin_type: PluginType::NativeAi,
            models: vec![ModelRequirement {
                name: "realesrgan-x2.onnx".to_string(),
                required: true,
                size_mb: Some(68),
                url: None,
                sha256: None,
            }],
            ai: Some(AiPipelineConfig {
                task: AiTask::SuperResolution,
                scale: Some(2),
                tile_size: Some(512),
                tile_pad: Some(16),
                normalize: Some(NormalizationMode::ZeroToOne),
            }),
            ..Default::default()
        };

        let plugin = GenericAiPlugin::new(manifest.clone(), engine);
        assert_eq!(plugin.id(), "ai.photocraft.plugins.upscale");
        assert_eq!(plugin.manifest().name, "AI Super-Resolution (2x)");
        assert_eq!(plugin.manifest().models[0].name, "realesrgan-x2.onnx");
    }

    #[test]
    fn test_ai_engine_model_path_resolution() {
        let temp_dir = std::env::temp_dir().join(format!(
            "ai_resolve_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let models_dir = temp_dir.join("models");
        std::fs::create_dir_all(&models_dir).unwrap();
        let target_file = models_dir.join("test_model.onnx");
        std::fs::write(&target_file, b"test onnx").unwrap();

        let engine = AiEngine::new(&temp_dir);
        let resolved = engine.resolve_model_path("test_model.onnx", None);
        assert_eq!(resolved, Some(target_file));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    struct MockBridgeGeneric;

    #[async_trait::async_trait]
    impl BridgeOperations for MockBridgeGeneric {
        async fn execute_command(
            &self,
            _command: &str,
            _params: serde_json::Value,
        ) -> Result<serde_json::Value> {
            Ok(json!({}))
        }

        async fn copy_active_layer_image(&self) -> Result<RgbaImage> {
            Ok(RgbaImage::from_pixel(16, 16, Rgba([255, 0, 0, 255])))
        }

        async fn paste_image_in_place(&self, _img: &RgbaImage) -> Result<serde_json::Value> {
            Ok(json!({ "layer": 42 }))
        }

        async fn rename_layer(&self, _layer_id: u64, _name: &str) -> Result<serde_json::Value> {
            Ok(json!({ "ok": true }))
        }

        async fn get_document_info(&self) -> Result<DocumentInfo> {
            Ok(DocumentInfo {
                name: "Test.psd".to_string(),
                width: 16,
                height: 16,
                active_layer: Some(10),
                active_layer_name: Some("Layer 1".to_string()),
                has_selection: false,
                selection_bounds: None,
            })
        }
    }

    #[tokio::test]
    async fn test_generic_ai_missing_model_graceful_error() {
        let engine = Arc::new(AiEngine::new("non_existent_dir"));
        let manifest = PluginManifest {
            id: "ai.photocraft.plugins.unknown".to_string(),
            name: "Unknown AI".to_string(),
            version: "1.0.0".to_string(),
            description: "Desc".to_string(),
            author: "Tester".to_string(),
            plugin_type: PluginType::NativeAi,
            models: vec![ModelRequirement {
                name: "missing_model_12345.onnx".to_string(),
                required: true,
                size_mb: None,
                url: None,
                sha256: None,
            }],
            ..Default::default()
        };

        let plugin = GenericAiPlugin::new(manifest, engine);
        let ctx = PluginContext {
            document: DocumentInfo::default(),
            models_dir: PathBuf::from("models"),
            plugin_dir: PathBuf::from("plugins/unknown"),
            client: Arc::new(MockBridgeGeneric),
        };

        let res = plugin.execute(&ctx).await;
        assert!(res.is_err());
        let err_str = res.unwrap_err().to_string();
        assert!(err_str.contains("missing_model_12345.onnx"));
    }

    #[test]
    fn test_compute_mask_bbox() {
        let mut empty_mask = GrayImage::new(100, 100);
        assert_eq!(AiEngine::compute_mask_bbox(&empty_mask), None);

        // Put pixels in [20..40, 30..60]
        empty_mask.put_pixel(20, 30, image::Luma([255]));
        empty_mask.put_pixel(40, 60, image::Luma([255]));
        assert_eq!(
            AiEngine::compute_mask_bbox(&empty_mask),
            Some((20, 30, 40, 60))
        );
    }
}
