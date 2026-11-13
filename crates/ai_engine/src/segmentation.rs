use crate::AiEngine;
use core_contracts::{CompanionError, NormalizationMode, Result};
use image::{imageops::FilterType, DynamicImage, GrayImage, RgbaImage};
use std::path::Path;

impl AiEngine {
    /// Paradigm 4 Engine: Segmentation Mask & Monocular Depth Map (RMBG-1.4, Depth Anything V2)
    pub(crate) fn run_segmentation_inference(
        &self,
        model_path: &Path,
        img: &RgbaImage,
        norm_mode: &NormalizationMode,
        input_size: u32,
        is_grayscale_layer: bool,
    ) -> Result<GrayImage> {
        let (orig_w, orig_h) = (img.width(), img.height());

        let dynamic_img = DynamicImage::ImageRgba8(img.clone());
        let resized = dynamic_img
            .resize_exact(input_size, input_size, FilterType::Triangle)
            .to_rgb8();

        let s = input_size as usize;
        let channel_stride = s * s;
        let mut input_data = vec![0.0f32; 3 * channel_stride];
        for y in 0..s {
            for x in 0..s {
                let idx = y * s + x;
                let pixel = resized.get_pixel(x as u32, y as u32);
                match norm_mode {
                    NormalizationMode::Centered => {
                        input_data[idx] = (pixel[0] as f32 / 255.0 - 0.5) / 1.0;
                        input_data[channel_stride + idx] = (pixel[1] as f32 / 255.0 - 0.5) / 1.0;
                        input_data[2 * channel_stride + idx] =
                            (pixel[2] as f32 / 255.0 - 0.5) / 1.0;
                    }
                    NormalizationMode::ImageNet => {
                        input_data[idx] = (pixel[0] as f32 / 255.0 - 0.485) / 0.229;
                        input_data[channel_stride + idx] =
                            (pixel[1] as f32 / 255.0 - 0.456) / 0.224;
                        input_data[2 * channel_stride + idx] =
                            (pixel[2] as f32 / 255.0 - 0.406) / 0.225;
                    }
                    NormalizationMode::Raw255 => {
                        input_data[idx] = pixel[0] as f32;
                        input_data[channel_stride + idx] = pixel[1] as f32;
                        input_data[2 * channel_stride + idx] = pixel[2] as f32;
                    }
                    NormalizationMode::ZeroToOne => {
                        input_data[idx] = pixel[0] as f32 / 255.0;
                        input_data[channel_stride + idx] = pixel[1] as f32 / 255.0;
                        input_data[2 * channel_stride + idx] = pixel[2] as f32 / 255.0;
                    }
                }
            }
        }

        let mask_slice =
            self.run_single_inference_vector(model_path, input_data, [1usize, 3, s, s])?;

        let mask_raw: Vec<u8> = if is_grayscale_layer {
            // For depth maps, model outputs unconstrained disparity/depth values (e.g. 0..30+).
            // Perform robust min-max normalization to map full depth range into 0..255.
            let mut min_val = f32::INFINITY;
            let mut max_val = f32::NEG_INFINITY;
            for &val in &mask_slice {
                if val.is_finite() {
                    if val < min_val {
                        min_val = val;
                    }
                    if val > max_val {
                        max_val = val;
                    }
                }
            }
            let range = (max_val - min_val).max(1e-6);
            mask_slice
                .iter()
                .map(|&val| {
                    let norm = (val - min_val) / range;
                    (norm.clamp(0.0, 1.0) * 255.0).round() as u8
                })
                .collect()
        } else {
            // Standard segmentation mask (0.0..1.0 probability)
            mask_slice
                .iter()
                .map(|&val| (val.clamp(0.0, 1.0) * 255.0).round() as u8)
                .collect()
        };

        let mask_sq = GrayImage::from_raw(input_size, input_size, mask_raw)
            .ok_or_else(|| CompanionError::AiEngine("Failed to create mask GrayImage".into()))?;

        let resized_mask = DynamicImage::ImageLuma8(mask_sq)
            .resize_exact(orig_w, orig_h, FilterType::CatmullRom)
            .to_luma8();

        Ok(resized_mask)
    }
}
