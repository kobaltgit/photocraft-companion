use image::{Rgba, RgbaImage};

/// Fast horizontal and vertical blur on an alpha channel, producing a soft drop shadow
pub fn render_drop_shadow(
    src: &RgbaImage,
    offset_x: f32,
    offset_y: f32,
    blur_radius: f32,
    opacity: f32,
) -> Option<RgbaImage> {
    if opacity <= 0.001 || (blur_radius <= 0.0 && offset_x.abs() < 0.5 && offset_y.abs() < 0.5) {
        return None;
    }

    let (width, height) = src.dimensions();
    if width == 0 || height == 0 {
        return None;
    }

    let radius = (blur_radius.round() as usize).clamp(0, 100);
    let mut shadow = RgbaImage::new(width, height);

    let ox = offset_x.round() as i32;
    let oy = offset_y.round() as i32;

    // 1. Shift alpha according to offset
    let mut alpha_buf = vec![0u8; (width * height) as usize];
    for y in 0..height as i32 {
        for x in 0..width as i32 {
            let src_x = x - ox;
            let src_y = y - oy;
            if src_x >= 0 && src_x < width as i32 && src_y >= 0 && src_y < height as i32 {
                let pixel = src.get_pixel(src_x as u32, src_y as u32);
                alpha_buf[(y as u32 * width + x as u32) as usize] = pixel[3];
            }
        }
    }

    // 2. Perform two-pass separable box blur (approximating gaussian blur)
    if radius > 0 {
        alpha_buf = separable_box_blur(&alpha_buf, width as usize, height as usize, radius);
        if radius > 2 {
            alpha_buf = separable_box_blur(&alpha_buf, width as usize, height as usize, radius / 2);
        }
    }

    // 3. Bake into black RGBA shadow with target opacity
    for y in 0..height {
        for x in 0..width {
            let a = alpha_buf[(y * width + x) as usize];
            if a > 0 {
                let final_a = ((a as f32) * opacity.clamp(0.0, 1.0)).round() as u8;
                if final_a > 0 {
                    shadow.put_pixel(x, y, Rgba([0, 0, 0, final_a]));
                }
            }
        }
    }

    Some(shadow)
}

fn separable_box_blur(src: &[u8], width: usize, height: usize, radius: usize) -> Vec<u8> {
    if radius == 0 || width == 0 || height == 0 {
        return src.to_vec();
    }

    let mut temp = vec![0u8; width * height];
    let mut dest = vec![0u8; width * height];
    let window_size = (radius * 2 + 1) as u32;

    // Horizontal pass
    for y in 0..height {
        let row_offset = y * width;
        let mut sum: u32 = 0;

        // At x = 0, the window covers [-radius .. radius].
        // Pixels outside [0..width) are transparent (0).
        for i in 0..=radius {
            if i < width {
                sum += src[row_offset + i] as u32;
            }
        }

        for x in 0..width {
            temp[row_offset + x] = (sum / window_size).min(255) as u8;

            let right_in = if x + radius + 1 < width {
                src[row_offset + x + radius + 1] as u32
            } else {
                0
            };
            let left_out = if x >= radius {
                src[row_offset + x - radius] as u32
            } else {
                0
            };
            sum = sum.saturating_add(right_in).saturating_sub(left_out);
        }
    }

    // Vertical pass
    for x in 0..width {
        let mut sum: u32 = 0;

        for i in 0..=radius {
            if i < height {
                sum += temp[i * width + x] as u32;
            }
        }

        for y in 0..height {
            dest[y * width + x] = (sum / window_size).min(255) as u8;

            let bottom_in = if y + radius + 1 < height {
                temp[(y + radius + 1) * width + x] as u32
            } else {
                0
            };
            let top_out = if y >= radius {
                temp[(y - radius) * width + x] as u32
            } else {
                0
            };
            sum = sum.saturating_add(bottom_in).saturating_sub(top_out);
        }
    }

    dest
}
