use crate::bevel::apply_inner_bevel;
use crate::shading::{apply_directional_shading, compute_alpha_normals};
use crate::shadow::render_drop_shadow;
use crate::types::ExtrudeParams;
use image::{Rgba, RgbaImage};

/// Calculate scale factor for fitting original layer dimensions into a preview max dimension
pub fn calculate_preview_scale(orig_w: u32, orig_h: u32, max_preview_dim: u32) -> f32 {
    let max_dim = orig_w.max(orig_h);
    if max_dim == 0 || max_dim <= max_preview_dim {
        1.0
    } else {
        max_preview_dim as f32 / max_dim as f32
    }
}

/// Calculate bounding box of non-transparent pixels: (min_x, min_y, max_x, max_y)
pub fn calculate_content_bounds(img: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let (w, h) = img.dimensions();
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut has_content = false;

    for y in 0..h {
        for x in 0..w {
            if img.get_pixel(x, y)[3] > 0 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                has_content = true;
            }
        }
    }

    if has_content {
        Some((min_x, min_y, max_x, max_y))
    } else {
        None
    }
}

/// Render 2.5D extrusion for interactive preview with automatic content cropping and directional expansion
pub fn render_extrude_preview(src: &RgbaImage, params: &ExtrudeParams) -> RgbaImage {
    let bounds = match calculate_content_bounds(src) {
        Some(b) => b,
        None => return RgbaImage::new(1, 1),
    };

    let (min_x, min_y, max_x, max_y) = bounds;
    let cw = max_x - min_x + 1;
    let ch = max_y - min_y + 1;

    let cropped = image::imageops::crop_imm(src, min_x, min_y, cw, ch).to_image();

    let rad = params.angle_deg.to_radians();
    let total_dist = params.distance + params.shadow_distance;
    let sx = rad.cos() * total_dist;
    let sy = rad.sin() * total_dist;

    let blur_pad = (params.shadow_blur * 2.0 + params.bevel_radius + 12.0).ceil() as u32;
    let pad_left = blur_pad + (if sx < 0.0 { sx.abs().ceil() as u32 } else { 0 });
    let pad_right = blur_pad + (if sx > 0.0 { sx.ceil() as u32 } else { 0 });
    let pad_top = blur_pad + (if sy < 0.0 { sy.abs().ceil() as u32 } else { 0 });
    let pad_bottom = blur_pad + (if sy > 0.0 { sy.ceil() as u32 } else { 0 });

    let total_w = cw + pad_left + pad_right;
    let total_h = ch + pad_top + pad_bottom;

    let mut padded = RgbaImage::new(total_w, total_h);
    image::imageops::overlay(&mut padded, &cropped, pad_left as i64, pad_top as i64);

    render_extrude_2_5d(&padded, params)
}

/// Render 2.5D extrusion in split layers: (Front, Body, Shadow)
pub fn render_extrude_split(
    src: &RgbaImage,
    params: &ExtrudeParams,
) -> (RgbaImage, RgbaImage, Option<RgbaImage>) {
    let (width, height) = src.dimensions();
    if width == 0 || height == 0 {
        return (RgbaImage::new(0, 0), RgbaImage::new(0, 0), None);
    }

    // 1. Prepare front face
    let mut front = RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let px = src.get_pixel(x, y);
            if px[3] > 0 {
                let color = if let Some(fc) = params.front_color {
                    let a = (px[3] as f32 * (fc[3] as f32 / 255.0))
                        .round()
                        .clamp(0.0, 255.0) as u8;
                    Rgba([fc[0], fc[1], fc[2], a])
                } else {
                    *px
                };
                front.put_pixel(x, y, color);
            }
        }
    }

    // Apply inner bevel to front face
    if params.bevel_radius > 0.0 && params.bevel_highlight > 0.0 {
        front = apply_inner_bevel(
            &front,
            params.bevel_radius,
            params.bevel_highlight,
            params.light_angle_deg,
        );
    }

    // 2. Prepare extrusion body
    let mut body = RgbaImage::new(width, height);
    let dist = params.distance;

    if dist >= 0.5 {
        let rad = params.angle_deg.to_radians();
        let vx = rad.cos() * dist;
        let vy = rad.sin() * dist;

        let steps = ((dist * 2.0).ceil() as usize).clamp(2, 600);
        let normals = compute_alpha_normals(src);

        // Precompute gradient color for each slice step
        let start_c = params.body_gradient_start;
        let end_c = params.body_gradient_end;

        // Content centroid for perspective tapering towards vanishing point
        let (cx, cy) = if let Some((bx0, by0, bx1, by1)) = calculate_content_bounds(src) {
            ((bx0 + bx1) as f32 * 0.5, (by0 + by1) as f32 * 0.5)
        } else {
            (width as f32 * 0.5, height as f32 * 0.5)
        };

        // Render slices back-to-front
        for step in (1..=steps).rev() {
            let t = step as f32 / steps as f32;
            let ox = vx * t;
            let oy = vy * t;

            // Perspective scale factor: recedes towards vanishing point
            let scale = if params.perspective.abs() > 0.001 {
                (1.0 - t * params.perspective).clamp(0.05, 2.0)
            } else {
                1.0
            };

            // Interpolate body color along extrusion depth
            let r = ((1.0 - t) * (start_c[0] as f32) + t * (end_c[0] as f32)).round() as u8;
            let g = ((1.0 - t) * (start_c[1] as f32) + t * (end_c[1] as f32)).round() as u8;
            let b = ((1.0 - t) * (start_c[2] as f32) + t * (end_c[2] as f32)).round() as u8;
            let a = ((1.0 - t) * (start_c[3] as f32) + t * (end_c[3] as f32)).round() as u8;
            let slice_color = Rgba([r, g, b, a]);

            for y in 0..height as i32 {
                for x in 0..width as i32 {
                    let src_px = src.get_pixel(x as u32, y as u32);
                    if src_px[3] == 0 {
                        continue;
                    }

                    let dest_x = if scale != 1.0 {
                        (cx + (x as f32 - cx) * scale + ox).round() as i32
                    } else {
                        x + ox.round() as i32
                    };
                    let dest_y = if scale != 1.0 {
                        (cy + (y as f32 - cy) * scale + oy).round() as i32
                    } else {
                        y + oy.round() as i32
                    };

                    if dest_x >= 0 && dest_x < width as i32 && dest_y >= 0 && dest_y < height as i32
                    {
                        let normal = normals[(y as u32 * width + x as u32) as usize];
                        let shaded = apply_directional_shading(
                            slice_color,
                            normal,
                            params.light_angle_deg,
                            params.light_intensity,
                        );

                        // Alpha composite slice onto body
                        let dest_px = body.get_pixel_mut(dest_x as u32, dest_y as u32);
                        alpha_blend_pixel(dest_px, shaded, src_px[3]);
                    }
                }
            }
        }
    }

    // 3. Prepare drop shadow
    let shadow = if params.shadow_opacity > 0.0
        && (params.shadow_blur > 0.0 || params.shadow_distance > 0.0)
    {
        // Union of front and body for realistic silhouette
        let mut silhouette = RgbaImage::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let fa = front.get_pixel(x, y)[3];
                let ba = body.get_pixel(x, y)[3];
                let max_a = fa.max(ba);
                if max_a > 0 {
                    silhouette.put_pixel(x, y, Rgba([0, 0, 0, max_a]));
                }
            }
        }

        let s_rad = params.angle_deg.to_radians();
        let total_offset = params.distance + params.shadow_distance;
        let sox = s_rad.cos() * total_offset;
        let soy = s_rad.sin() * total_offset;

        render_drop_shadow(
            &silhouette,
            sox,
            soy,
            params.shadow_blur,
            params.shadow_opacity,
        )
    } else {
        None
    };

    (front, body, shadow)
}

/// Render full 2.5D extrusion flat merged in bottom-to-top Z-order (Shadow -> Body -> Front)
pub fn render_extrude_2_5d(src: &RgbaImage, params: &ExtrudeParams) -> RgbaImage {
    let (front, body, shadow) = render_extrude_split(src, params);
    let (width, height) = src.dimensions();
    let mut merged = RgbaImage::new(width, height);

    // 1. Bottom layer: Drop shadow
    if let Some(sh) = shadow {
        composite_layer_onto(&mut merged, &sh);
    }

    // 2. Middle layer: Extrusion body
    composite_layer_onto(&mut merged, &body);

    // 3. Top layer: Front face
    composite_layer_onto(&mut merged, &front);

    merged
}

#[inline]
fn alpha_blend_pixel(dest: &mut Rgba<u8>, color: Rgba<u8>, source_mask_alpha: u8) {
    let sa = (color[3] as f32 / 255.0) * (source_mask_alpha as f32 / 255.0);
    if sa <= 0.001 {
        return;
    }

    let da = dest[3] as f32 / 255.0;
    let out_a = sa + da * (1.0 - sa);

    if out_a <= 0.001 {
        *dest = Rgba([0, 0, 0, 0]);
        return;
    }

    let r = ((color[0] as f32 * sa + dest[0] as f32 * da * (1.0 - sa)) / out_a).round() as u8;
    let g = ((color[1] as f32 * sa + dest[1] as f32 * da * (1.0 - sa)) / out_a).round() as u8;
    let b = ((color[2] as f32 * sa + dest[2] as f32 * da * (1.0 - sa)) / out_a).round() as u8;
    let a = (out_a * 255.0).round().min(255.0) as u8;

    *dest = Rgba([r, g, b, a]);
}

fn composite_layer_onto(dest: &mut RgbaImage, src: &RgbaImage) {
    let (w, h) = dest.dimensions();
    for y in 0..h {
        for x in 0..w {
            let spx = src.get_pixel(x, y);
            if spx[3] > 0 {
                let dpx = dest.get_pixel_mut(x, y);
                alpha_blend_pixel(dpx, *spx, 255);
            }
        }
    }
}
