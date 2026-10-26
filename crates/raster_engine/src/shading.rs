use image::{Rgba, RgbaImage};

/// Calculate pseudo-normals from alpha channel gradient (Sobel operator)
pub fn compute_alpha_normals(src: &RgbaImage) -> Vec<(f32, f32)> {
    let (width, height) = src.dimensions();
    let mut normals = vec![(0.0f32, 0.0f32); (width * height) as usize];
    if width < 3 || height < 3 {
        return normals;
    }

    for y in 1..(height - 1) {
        for x in 1..(width - 1) {
            let p = |dx: i32, dy: i32| -> f32 {
                src.get_pixel((x as i32 + dx) as u32, (y as i32 + dy) as u32)[3] as f32
            };

            // Sobel kernels
            let gx = (p(1, -1) + 2.0 * p(1, 0) + p(1, 1)) - (p(-1, -1) + 2.0 * p(-1, 0) + p(-1, 1));
            let gy = (p(-1, 1) + 2.0 * p(0, 1) + p(1, 1)) - (p(-1, -1) + 2.0 * p(0, -1) + p(1, -1));

            let len = (gx * gx + gy * gy).sqrt();
            if len > 0.001 {
                normals[(y * width + x) as usize] = (gx / len, gy / len);
            }
        }
    }

    normals
}

/// Shade an RGBA color based on 2D normal and light direction
pub fn apply_directional_shading(
    base: Rgba<u8>,
    normal: (f32, f32),
    light_angle_deg: f32,
    intensity: f32,
) -> Rgba<u8> {
    if intensity <= 0.001 || (normal.0 == 0.0 && normal.1 == 0.0) {
        return base;
    }

    let rad = light_angle_deg.to_radians();
    let lx = rad.cos();
    let ly = rad.sin();

    // Dot product between surface normal and light direction
    let dot = normal.0 * lx + normal.1 * ly;

    // Modulate RGB brightness
    let factor = 1.0 + dot * intensity;
    let r = ((base[0] as f32) * factor).clamp(0.0, 255.0) as u8;
    let g = ((base[1] as f32) * factor).clamp(0.0, 255.0) as u8;
    let b = ((base[2] as f32) * factor).clamp(0.0, 255.0) as u8;

    Rgba([r, g, b, base[3]])
}
