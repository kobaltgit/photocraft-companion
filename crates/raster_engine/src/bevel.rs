use image::{Rgba, RgbaImage};

/// Apply an inner bevel / highlight contour to the front face layer
pub fn apply_inner_bevel(
    src: &RgbaImage,
    radius: f32,
    highlight: f32,
    light_angle_deg: f32,
) -> RgbaImage {
    if radius <= 0.0 || highlight <= 0.001 {
        return src.clone();
    }

    let (width, height) = src.dimensions();
    let mut result = src.clone();
    let rad = light_angle_deg.to_radians();
    let lx = rad.cos();
    let ly = rad.sin();

    for y in 0..height {
        for x in 0..width {
            let center_pix = src.get_pixel(x, y);
            if center_pix[3] == 0 {
                continue;
            }

            // Sample neighbor along light direction to detect inward edge
            let sample_x =
                (x as i32 - (lx * radius).round() as i32).clamp(0, width as i32 - 1) as u32;
            let sample_y =
                (y as i32 - (ly * radius).round() as i32).clamp(0, height as i32 - 1) as u32;
            let neighbor_alpha = src.get_pixel(sample_x, sample_y)[3] as f32;

            // If neighbor has less alpha, this is an exposed edge facing the light
            let edge_factor = ((center_pix[3] as f32 - neighbor_alpha) / 255.0).clamp(0.0, 1.0);
            if edge_factor > 0.05 {
                let boost = edge_factor * highlight * 255.0;
                let r = ((center_pix[0] as f32) + boost).min(255.0) as u8;
                let g = ((center_pix[1] as f32) + boost).min(255.0) as u8;
                let b = ((center_pix[2] as f32) + boost).min(255.0) as u8;
                result.put_pixel(x, y, Rgba([r, g, b, center_pix[3]]));
            }
        }
    }

    result
}
