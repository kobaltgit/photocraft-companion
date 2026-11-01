use image::{GrayImage, Rgba, RgbaImage};

/// Parameters controlling depth-of-field defocus bokeh and atmospheric fog
#[derive(Debug, Clone, PartialEq)]
pub struct LensBlurParams {
    /// Whether atmospheric fog effect is enabled
    pub fog_enabled: bool,
    /// RGBA fog color tint
    pub fog_color: [u8; 4],
    /// Fog density percentage (0.0 .. 100.0)
    pub fog_density: f32,
    /// Distance threshold where fog begins (0.0 .. 100.0%)
    pub fog_start: f32,
    /// Distance falloff power curve (0.5 .. 3.0)
    pub fog_falloff: f32,
    /// Fog blend mode (0 = Normal, 1 = Screen, 2 = Soft Light)
    pub fog_blend_mode: u8,

    /// Whether depth-of-field bokeh defocus blur is enabled
    pub bokeh_enabled: bool,
    /// Focus distance plane on 8-bit depth map scale (0.0 .. 255.0)
    pub focus_distance: f32,
    /// Depth of field sharpness zone width (1.0 .. 100.0)
    pub dof_width: f32,
    /// Maximum aperture blur radius in pixels (0.0 .. 50.0)
    pub blur_radius: f32,
    /// Specular highlight bloom percentage (0.0 .. 100.0)
    pub specular_bloom: f32,
}

impl Default for LensBlurParams {
    fn default() -> Self {
        Self {
            fog_enabled: false,
            fog_color: [200, 214, 229, 255],
            fog_density: 65.0,
            fog_start: 15.0,
            fog_falloff: 1.5,
            fog_blend_mode: 0,
            bokeh_enabled: true,
            focus_distance: 128.0,
            dof_width: 30.0,
            blur_radius: 14.0,
            specular_bloom: 25.0,
        }
    }
}

impl LensBlurParams {
    /// Proportionally scale all pixel-distance parameters (blur_radius)
    /// to maintain visual parity between preview thumbnail and full canvas render.
    pub fn scaled(&self, factor: f32) -> Self {
        let mut scaled = self.clone();
        scaled.blur_radius = (self.blur_radius * factor).max(0.0);
        scaled
    }
}

/// Accumulator pixel for weighted separable blur passes with specular bloom
#[derive(Clone, Copy, Default)]
struct AccumPixel {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
    w: f32,
}

impl AccumPixel {
    #[inline(always)]
    fn add(&mut self, other: &AccumPixel) {
        self.r += other.r;
        self.g += other.g;
        self.b += other.b;
        self.a += other.a;
        self.w += other.w;
    }

    #[inline(always)]
    fn sub(&mut self, other: &AccumPixel) {
        self.r -= other.r;
        self.g -= other.g;
        self.b -= other.b;
        self.a -= other.a;
        self.w -= other.w;
    }

    #[inline(always)]
    fn scale(&self, factor: f32) -> AccumPixel {
        AccumPixel {
            r: self.r * factor,
            g: self.g * factor,
            b: self.b * factor,
            a: self.a * factor,
            w: self.w * factor,
        }
    }

    #[inline(always)]
    fn to_rgba(&self) -> Rgba<u8> {
        if self.w > 0.0001 {
            let inv_w = 1.0 / self.w;
            let r = (self.r * inv_w).round().clamp(0.0, 255.0) as u8;
            let g = (self.g * inv_w).round().clamp(0.0, 255.0) as u8;
            let b = (self.b * inv_w).round().clamp(0.0, 255.0) as u8;
            let a = (self.a * inv_w).round().clamp(0.0, 255.0) as u8;
            Rgba([r, g, b, a])
        } else {
            Rgba([0, 0, 0, 0])
        }
    }
}

/// Safely sample depth value with automatic aspect/dimension scaling
#[inline(always)]
fn sample_depth(depth_map: &GrayImage, x: u32, y: u32, base_w: u32, base_h: u32) -> u8 {
    let (dw, dh) = depth_map.dimensions();
    if dw == 0 || dh == 0 {
        return 128;
    }
    if dw == base_w && dh == base_h {
        depth_map.get_pixel(x, y)[0]
    } else {
        let sx = ((x as f32 / base_w as f32) * dw as f32).min((dw - 1) as f32) as u32;
        let sy = ((y as f32 / base_h as f32) * dh as f32).min((dh - 1) as f32) as u32;
        depth_map.get_pixel(sx, sy)[0]
    }
}

/// Calculate circle of confusion radius in pixels for given depth value
#[inline(always)]
fn compute_coc_radius(depth_val: f32, params: &LensBlurParams) -> f32 {
    let half_dof = (params.dof_width * 0.5).max(0.5);
    let diff = (depth_val - params.focus_distance).abs();
    if diff <= half_dof {
        0.0
    } else {
        let excess = diff - half_dof;
        let max_diff = params.focus_distance.max(255.0 - params.focus_distance);
        let max_excess = (max_diff - half_dof).max(1.0);
        let norm_coc = (excess / max_excess).clamp(0.0, 1.0);
        norm_coc * params.blur_radius
    }
}

/// Perform a single 2D separable box pass on accumulator pixels with edge clamping
fn separable_box_pass(src: &[AccumPixel], w: usize, h: usize, radius: usize) -> Vec<AccumPixel> {
    if radius == 0 || w == 0 || h == 0 {
        return src.to_vec();
    }

    let mut temp = vec![AccumPixel::default(); w * h];
    let inv_window = 1.0 / (2 * radius + 1) as f32;

    // Horizontal pass
    for y in 0..h {
        let row = y * w;
        let mut sum = AccumPixel::default();

        for k in 0..=radius {
            let p0 = src[row];
            sum.add(&p0);
            if k > 0 {
                let px = src[row + k.min(w - 1)];
                sum.add(&px);
            }
        }

        for x in 0..w {
            temp[row + x] = sum.scale(inv_window);

            let right_idx = (x + radius + 1).min(w - 1);
            let left_idx = if x >= radius { x - radius } else { 0 };

            sum.add(&src[row + right_idx]);
            sum.sub(&src[row + left_idx]);
        }
    }

    // Vertical pass
    let mut dest = vec![AccumPixel::default(); w * h];
    for x in 0..w {
        let mut sum = AccumPixel::default();

        for k in 0..=radius {
            let p0 = temp[x];
            sum.add(&p0);
            if k > 0 {
                let py = temp[k.min(h - 1) * w + x];
                sum.add(&py);
            }
        }

        for y in 0..h {
            dest[y * w + x] = sum.scale(inv_window);

            let down_idx = (y + radius + 1).min(h - 1);
            let up_idx = if y >= radius { y - radius } else { 0 };

            sum.add(&temp[down_idx * w + x]);
            sum.sub(&temp[up_idx * w + x]);
        }
    }

    dest
}

/// Blur precomputed accumulator buffer to produce a discrete blur level
fn blur_level_to_rgba(accum_init: &[AccumPixel], w: usize, h: usize, radius: f32) -> RgbaImage {
    let r_int = radius.round() as usize;
    if r_int == 0 {
        let mut img = RgbaImage::new(w as u32, h as u32);
        for y in 0..h {
            for x in 0..w {
                img.put_pixel(x as u32, y as u32, accum_init[y * w + x].to_rgba());
            }
        }
        return img;
    }

    let accum_blurred = if radius >= 2.0 {
        let r1 = ((radius * 0.6).round() as usize).max(1);
        let r2 = ((radius * 0.4).round() as usize).max(1);
        let pass1 = separable_box_pass(accum_init, w, h, r1);
        separable_box_pass(&pass1, w, h, r2)
    } else {
        separable_box_pass(accum_init, w, h, 1)
    };

    let mut img = RgbaImage::new(w as u32, h as u32);
    for y in 0..h {
        for x in 0..w {
            img.put_pixel(x as u32, y as u32, accum_blurred[y * w + x].to_rgba());
        }
    }
    img
}

/// Linearly interpolate between two RGBA pixels
#[inline(always)]
fn lerp_rgba(p0: &Rgba<u8>, p1: &Rgba<u8>, t: f32) -> Rgba<u8> {
    let inv_t = 1.0 - t;
    let r = (p0[0] as f32 * inv_t + p1[0] as f32 * t).round() as u8;
    let g = (p0[1] as f32 * inv_t + p1[1] as f32 * t).round() as u8;
    let b = (p0[2] as f32 * inv_t + p1[2] as f32 * t).round() as u8;
    let a = (p0[3] as f32 * inv_t + p1[3] as f32 * t).round() as u8;
    Rgba([r, g, b, a])
}

/// Standard Soft Light blend formula (W3C / Photoshop standard)
#[inline(always)]
fn blend_soft_light(base: f32, blend: f32) -> f32 {
    if blend <= 0.5 {
        base - (1.0 - 2.0 * blend) * base * (1.0 - base)
    } else {
        let d = if base <= 0.25 {
            ((16.0 * base - 12.0) * base + 4.0) * base
        } else {
            base.sqrt()
        };
        base + (2.0 * blend - 1.0) * (d - base)
    }
}

/// Composite fog layer onto base layer according to blend mode
fn composite_fog(base: &RgbaImage, fog_layer: &RgbaImage, blend_mode: u8) -> RgbaImage {
    let (width, height) = base.dimensions();
    let mut out = base.clone();

    for y in 0..height {
        for x in 0..width {
            let fog_px = fog_layer.get_pixel(x, y);
            let fog_a = fog_px[3] as f32 / 255.0;
            if fog_a <= 0.001 {
                continue;
            }

            let base_px = base.get_pixel(x, y);
            let base_r = base_px[0] as f32 / 255.0;
            let base_g = base_px[1] as f32 / 255.0;
            let base_b = base_px[2] as f32 / 255.0;
            let base_a = base_px[3] as f32 / 255.0;

            let fog_r = fog_px[0] as f32 / 255.0;
            let fog_g = fog_px[1] as f32 / 255.0;
            let fog_b = fog_px[2] as f32 / 255.0;

            let (blended_r, blended_g, blended_b) = match blend_mode {
                1 => {
                    // Screen
                    (
                        1.0 - (1.0 - base_r) * (1.0 - fog_r),
                        1.0 - (1.0 - base_g) * (1.0 - fog_g),
                        1.0 - (1.0 - base_b) * (1.0 - fog_b),
                    )
                }
                2 => {
                    // Soft Light
                    (
                        blend_soft_light(base_r, fog_r),
                        blend_soft_light(base_g, fog_g),
                        blend_soft_light(base_b, fog_b),
                    )
                }
                _ => {
                    // Normal
                    (fog_r, fog_g, fog_b)
                }
            };

            let r = (base_r * (1.0 - fog_a) + blended_r * fog_a).clamp(0.0, 1.0) * 255.0;
            let g = (base_g * (1.0 - fog_a) + blended_g * fog_a).clamp(0.0, 1.0) * 255.0;
            let b = (base_b * (1.0 - fog_a) + blended_b * fog_a).clamp(0.0, 1.0) * 255.0;
            let a = (base_a + fog_a * (1.0 - base_a)).clamp(0.0, 1.0) * 255.0;

            out.put_pixel(
                x,
                y,
                Rgba([
                    r.round() as u8,
                    g.round() as u8,
                    b.round() as u8,
                    a.round() as u8,
                ]),
            );
        }
    }

    out
}

/// Internal pipeline supporting stepped multi-level defocus and optional fog generation
fn render_lens_blur_internal(
    base: &RgbaImage,
    depth_map: &GrayImage,
    params: &LensBlurParams,
    num_steps: usize,
) -> (RgbaImage, Option<RgbaImage>) {
    let (width, height) = base.dimensions();
    if width == 0 || height == 0 {
        return (RgbaImage::new(0, 0), None);
    }

    let w = width as usize;
    let h = height as usize;

    // 1. Defocus Bokeh Pass
    let defocused = if !params.bokeh_enabled || params.blur_radius < 0.5 {
        base.clone()
    } else {
        let bloom_factor = (params.specular_bloom / 100.0).clamp(0.0, 1.0);
        let mut accum_init = vec![AccumPixel::default(); w * h];

        for y in 0..h {
            for x in 0..w {
                let px = base.get_pixel(x as u32, y as u32);
                let a_norm = px[3] as f32 / 255.0;
                let lum =
                    (0.299 * px[0] as f32 + 0.587 * px[1] as f32 + 0.114 * px[2] as f32) / 255.0;
                let highlight = (lum - 0.5).max(0.0) * 2.0;
                let weight = (1.0 + highlight * highlight * bloom_factor * 4.0) * a_norm;

                accum_init[y * w + x] = AccumPixel {
                    r: px[0] as f32 * weight,
                    g: px[1] as f32 * weight,
                    b: px[2] as f32 * weight,
                    a: px[3] as f32 * weight,
                    w: weight,
                };
            }
        }

        let num_levels = num_steps.max(2);
        let mut levels: Vec<RgbaImage> = Vec::with_capacity(num_levels);
        levels.push(base.clone());

        for i in 1..num_levels {
            let r = (params.blur_radius * (i as f32) / (num_levels - 1) as f32).max(0.5);
            levels.push(blur_level_to_rgba(&accum_init, w, h, r));
        }

        let mut res = RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let d = sample_depth(depth_map, x, y, width, height) as f32;
                let coc = compute_coc_radius(d, params);
                if coc <= 0.05 {
                    res.put_pixel(x, y, *levels[0].get_pixel(x, y));
                } else {
                    let step_val =
                        (coc / params.blur_radius).clamp(0.0, 1.0) * (num_levels - 1) as f32;
                    let low_idx = step_val.floor() as usize;
                    let frac = step_val - low_idx as f32;
                    if low_idx >= num_levels - 1 {
                        res.put_pixel(x, y, *levels[num_levels - 1].get_pixel(x, y));
                    } else {
                        let p0 = levels[low_idx].get_pixel(x, y);
                        let p1 = levels[low_idx + 1].get_pixel(x, y);
                        res.put_pixel(x, y, lerp_rgba(p0, p1, frac));
                    }
                }
            }
        }
        res
    };

    // 2. Atmospheric Fog Pass
    let fog_layer = if params.fog_enabled {
        let mut fog_img = RgbaImage::new(width, height);
        let start_norm = (params.fog_start / 100.0).clamp(0.0, 1.0);
        let density_norm = (params.fog_density / 100.0).clamp(0.0, 1.0);
        let falloff = params.fog_falloff.clamp(0.1, 10.0);
        let range = (1.0 - start_norm).max(0.001);

        for y in 0..height {
            for x in 0..width {
                let d = sample_depth(depth_map, x, y, width, height) as f32;
                // 255 = near foreground, 0 = far background
                let dist = 1.0 - (d / 255.0);
                if dist > start_norm {
                    let t = ((dist - start_norm) / range).clamp(0.0, 1.0);
                    let t_curve = t.powf(falloff);
                    let alpha = (t_curve * density_norm * (params.fog_color[3] as f32 / 255.0))
                        .clamp(0.0, 1.0);
                    let a_byte = (alpha * 255.0).round() as u8;
                    if a_byte > 0 {
                        fog_img.put_pixel(
                            x,
                            y,
                            Rgba([
                                params.fog_color[0],
                                params.fog_color[1],
                                params.fog_color[2],
                                a_byte,
                            ]),
                        );
                    }
                }
            }
        }
        Some(fog_img)
    } else {
        None
    };

    (defocused, fog_layer)
}

/// Fast stepped defocus blur and atmospheric fog for interactive 60 FPS preview on 512px proxy
pub fn render_lens_blur_preview(
    base: &RgbaImage,
    depth_map: &GrayImage,
    params: &LensBlurParams,
) -> RgbaImage {
    let (defocused, fog_layer_opt) = render_lens_blur_internal(base, depth_map, params, 5);
    if let Some(fog_layer) = fog_layer_opt {
        composite_fog(&defocused, &fog_layer, params.fog_blend_mode)
    } else {
        defocused
    }
}

/// Full-quality depth-of-field bokeh defocus and atmospheric fog rendering
pub fn render_lens_blur(
    base: &RgbaImage,
    depth_map: &GrayImage,
    params: &LensBlurParams,
) -> RgbaImage {
    let (defocused, fog_layer_opt) = render_lens_blur_split(base, depth_map, params);
    if let Some(fog_layer) = fog_layer_opt {
        composite_fog(&defocused, &fog_layer, params.fog_blend_mode)
    } else {
        defocused
    }
}

/// Split-layer rendering for non-destructive multi-layer canvas export:
/// Returns (defocused_base_layer, optional_fog_color_layer_with_alpha)
pub fn render_lens_blur_split(
    base: &RgbaImage,
    depth_map: &GrayImage,
    params: &LensBlurParams,
) -> (RgbaImage, Option<RgbaImage>) {
    render_lens_blur_internal(base, depth_map, params, 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_lens_blur_params() {
        let p = LensBlurParams::default();
        assert!(!p.fog_enabled);
        assert!(p.bokeh_enabled);
        assert_eq!(p.focus_distance, 128.0);
        assert_eq!(p.dof_width, 30.0);
        assert_eq!(p.blur_radius, 14.0);
        assert_eq!(p.specular_bloom, 25.0);
    }

    #[test]
    fn test_params_scaled() {
        let p = LensBlurParams::default();
        let scaled = p.scaled(0.5);
        assert_eq!(scaled.blur_radius, 7.0);
        assert_eq!(scaled.focus_distance, p.focus_distance);
        assert_eq!(scaled.dof_width, p.dof_width);
    }

    #[test]
    fn test_zero_blur_is_noop() {
        let base = RgbaImage::from_pixel(16, 16, Rgba([255, 0, 0, 255]));
        let depth = GrayImage::from_pixel(16, 16, image::Luma([0]));
        let mut params = LensBlurParams::default();
        params.blur_radius = 0.0;
        params.fog_enabled = false;

        let out = render_lens_blur(&base, &depth, &params);
        assert_eq!(out, base);
    }

    #[test]
    fn test_bokeh_defocus_in_focus_remains_sharp() {
        let mut base = RgbaImage::new(32, 32);
        for y in 0..32 {
            for x in 0..32 {
                let color = if (x + y) % 2 == 0 { 255 } else { 0 };
                base.put_pixel(x, y, Rgba([color, color, color, 255]));
            }
        }

        // Left half is in focus (128), right half is far out of focus (0)
        let mut depth = GrayImage::new(32, 32);
        for y in 0..32 {
            for x in 0..16 {
                depth.put_pixel(x, y, image::Luma([128]));
            }
            for x in 16..32 {
                depth.put_pixel(x, y, image::Luma([0]));
            }
        }

        let mut params = LensBlurParams::default();
        params.focus_distance = 128.0;
        params.dof_width = 20.0;
        params.blur_radius = 8.0;
        params.fog_enabled = false;

        let out = render_lens_blur(&base, &depth, &params);

        // Pixel inside the in-focus region must remain sharp
        assert_eq!(out.get_pixel(4, 4), base.get_pixel(4, 4));

        // Pixel in out-of-focus region should have averaged / blurred values
        let blurred_val = out.get_pixel(24, 24)[0];
        assert!(blurred_val > 50 && blurred_val < 205);
    }

    #[test]
    fn test_specular_bloom_boosts_highlights() {
        let mut base = RgbaImage::from_pixel(17, 17, Rgba([10, 10, 10, 255]));
        base.put_pixel(8, 8, Rgba([255, 255, 255, 255]));

        let depth = GrayImage::from_pixel(17, 17, image::Luma([0])); // out of focus

        let mut params_no_bloom = LensBlurParams::default();
        params_no_bloom.focus_distance = 255.0;
        params_no_bloom.dof_width = 10.0;
        params_no_bloom.blur_radius = 6.0;
        params_no_bloom.specular_bloom = 0.0;
        params_no_bloom.fog_enabled = false;

        let mut params_bloom = params_no_bloom.clone();
        params_bloom.specular_bloom = 100.0;

        let out_no_bloom = render_lens_blur(&base, &depth, &params_no_bloom);
        let out_bloom = render_lens_blur(&base, &depth, &params_bloom);

        // With specular bloom, the neighborhood around the highlight should be noticeably brighter
        assert!(out_bloom.get_pixel(8, 8)[0] >= out_no_bloom.get_pixel(8, 8)[0]);
        assert!(out_bloom.get_pixel(7, 8)[0] >= out_no_bloom.get_pixel(7, 8)[0]);
    }

    #[test]
    fn test_atmospheric_fog_gradient() {
        let base = RgbaImage::from_pixel(32, 1, Rgba([30, 30, 30, 255]));
        let mut depth = GrayImage::new(32, 1);
        for x in 0..32 {
            // 255 (near) -> 0 (far)
            let d = (255.0 * (1.0 - x as f32 / 31.0)).round() as u8;
            depth.put_pixel(x, 0, image::Luma([d]));
        }

        let mut params = LensBlurParams::default();
        params.bokeh_enabled = false;
        params.fog_enabled = true;
        params.fog_color = [200, 220, 250, 255];
        params.fog_density = 80.0;
        params.fog_start = 10.0;
        params.fog_falloff = 1.0;
        params.fog_blend_mode = 0; // Normal

        let out = render_lens_blur(&base, &depth, &params);

        // Near pixel (x = 0) has no fog
        assert_eq!(out.get_pixel(0, 0), &Rgba([30, 30, 30, 255]));

        // Far pixel (x = 31) has dense fog
        let far_px = out.get_pixel(31, 0);
        assert!(far_px[0] > 100);
        assert!(far_px[1] > 120);
        assert!(far_px[2] > 150);
    }

    #[test]
    fn test_split_layer_export() {
        let base = RgbaImage::from_pixel(16, 16, Rgba([50, 50, 50, 255]));
        let depth = GrayImage::from_pixel(16, 16, image::Luma([0])); // Far

        let mut params = LensBlurParams::default();
        params.fog_enabled = true;
        params.fog_color = [180, 200, 220, 255];
        params.fog_density = 100.0;
        params.fog_start = 0.0;

        let (defocused, fog_layer_opt) = render_lens_blur_split(&base, &depth, &params);
        assert_eq!(defocused.dimensions(), (16, 16));
        assert!(fog_layer_opt.is_some());

        let fog = fog_layer_opt.unwrap();
        assert_eq!(fog.dimensions(), (16, 16));
        let fog_px = fog.get_pixel(8, 8);
        assert_eq!(fog_px[0], 180);
        assert_eq!(fog_px[1], 200);
        assert_eq!(fog_px[2], 220);
        assert!(fog_px[3] > 200);
    }

    #[test]
    fn test_preview_runs_fast_and_matches_dimensions() {
        let base = RgbaImage::from_pixel(64, 64, Rgba([120, 80, 40, 255]));
        let depth = GrayImage::from_pixel(64, 64, image::Luma([64]));
        let params = LensBlurParams::default();

        let preview = render_lens_blur_preview(&base, &depth, &params);
        assert_eq!(preview.dimensions(), (64, 64));
    }

    #[test]
    fn test_empty_images() {
        let base = RgbaImage::new(0, 0);
        let depth = GrayImage::new(0, 0);
        let params = LensBlurParams::default();

        let preview = render_lens_blur_preview(&base, &depth, &params);
        assert_eq!(preview.dimensions(), (0, 0));

        let (split_base, split_fog) = render_lens_blur_split(&base, &depth, &params);
        assert_eq!(split_base.dimensions(), (0, 0));
        assert!(split_fog.is_none());
    }
}
