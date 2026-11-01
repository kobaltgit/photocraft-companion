pub use crate::dynamic_tool::named_colors::{NamedColorEntry, NAMED_COLORS};
use crate::dynamic_tool::state::DynamicWindowState;
use crate::theme::ThemePalette;
use core_contracts::{Language, WidgetNode};
use eframe::egui::{self, Color32, Sense, Stroke, Vec2};

/// Helper: calculates hue in degrees (0.0..360.0) from Color32
pub fn rgb_to_hue_degrees(color: Color32) -> f32 {
    let r = color.r() as f32 / 255.0;
    let g = color.g() as f32 / 255.0;
    let b = color.b() as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    if delta < 1e-5 {
        return 0.0;
    }

    let hue = if (max - r).abs() < 1e-5 {
        ((g - b) / delta) % 6.0
    } else if (max - g).abs() < 1e-5 {
        ((b - r) / delta) + 2.0
    } else {
        ((r - g) / delta) + 4.0
    };

    let mut deg = hue * 60.0;
    if deg < 0.0 {
        deg += 360.0;
    }
    deg
}

/// Helper: calculates Color32 from HSV
pub fn hsv_to_rgb(h_deg: f32, s: f32, v: f32) -> Color32 {
    let h = (h_deg % 360.0) / 60.0;
    let c = v * s;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = if (0.0..1.0).contains(&h) {
        (c, x, 0.0)
    } else if (1.0..2.0).contains(&h) {
        (x, c, 0.0)
    } else if (2.0..3.0).contains(&h) {
        (0.0, c, x)
    } else if (3.0..4.0).contains(&h) {
        (0.0, x, c)
    } else if (4.0..5.0).contains(&h) {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Color32::from_rgb(
        ((r1 + m) * 255.0).round() as u8,
        ((g1 + m) * 255.0).round() as u8,
        ((b1 + m) * 255.0).round() as u8,
    )
}

/// Helper: parses #RRGGBB hex string
pub fn parse_hex_color(hex: &str) -> Option<Color32> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        Some(Color32::from_rgb(r, g, b))
    } else if clean.len() == 3 {
        let r = u8::from_str_radix(&clean[0..1], 16).ok()?;
        let g = u8::from_str_radix(&clean[1..2], 16).ok()?;
        let b = u8::from_str_radix(&clean[2..3], 16).ok()?;
        Some(Color32::from_rgb(r * 17, g * 17, b * 17))
    } else {
        None
    }
}

/// Finds nearest named shade by weighted perceptual distance:
/// delta_c = sqrt(2*dr^2 + 4*dg^2 + 3*db^2)
pub fn find_nearest_named_color(r: i64, g: i64, b: i64, lang: Language) -> String {
    let r_clamped = r.clamp(0, 255);
    let g_clamped = g.clamp(0, 255);
    let b_clamped = b.clamp(0, 255);

    let mut best_entry = &NAMED_COLORS[0];
    let mut min_dist = i64::MAX;

    for entry in NAMED_COLORS {
        let dr = r_clamped - entry.r as i64;
        let dg = g_clamped - entry.g as i64;
        let db = b_clamped - entry.b as i64;
        let dist = 2 * dr * dr + 4 * dg * dg + 3 * db * db;
        if dist < min_dist {
            min_dist = dist;
            best_entry = entry;
        }
    }

    if matches!(lang, Language::Ru) {
        best_entry.ru.to_string()
    } else {
        best_entry.en.to_string()
    }
}

/// Simulates color vision deficiency (CVD) perception using the Brettel-Vienot model
pub fn simulate_cvd_color(r: f64, g: f64, b: f64, profile_index: i64) -> (u8, u8, u8) {
    let r_clamped = r.clamp(0.0, 255.0);
    let g_clamped = g.clamp(0.0, 255.0);
    let b_clamped = b.clamp(0.0, 255.0);

    let (sim_r, sim_g, sim_b) = match profile_index {
        1 => {
            // Deuteranopia (green-blind)
            let nr = 0.625 * r_clamped + 0.375 * g_clamped;
            let ng = 0.70 * r_clamped + 0.30 * g_clamped;
            let nb = 0.30 * g_clamped + 0.70 * b_clamped;
            (nr, ng, nb)
        }
        2 => {
            // Protanopia (red-blind)
            let nr = 0.56667 * r_clamped + 0.43333 * g_clamped;
            let ng = 0.55833 * r_clamped + 0.44167 * g_clamped;
            let nb = 0.24167 * g_clamped + 0.75833 * b_clamped;
            (nr, ng, nb)
        }
        3 => {
            // Tritanopia (blue-blind)
            let nr = 0.95 * r_clamped + 0.05 * g_clamped;
            let ng = 0.43333 * g_clamped + 0.56667 * b_clamped;
            let nb = 0.475 * g_clamped + 0.525 * b_clamped;
            (nr, ng, nb)
        }
        _ => (r_clamped, g_clamped, b_clamped),
    };

    (
        sim_r.round().clamp(0.0, 255.0) as u8,
        sim_g.round().clamp(0.0, 255.0) as u8,
        sim_b.round().clamp(0.0, 255.0) as u8,
    )
}

/// Simulates color vision deficiency and outputs standard #rrggbb hex string
pub fn simulate_cvd_hex(r: f64, g: f64, b: f64, profile_index: i64) -> String {
    let (sr, sg, sb) = simulate_cvd_color(r, g, b, profile_index);
    format!("#{sr:02x}{sg:02x}{sb:02x}")
}

/// Calculates standard WCAG 2.1 relative luminance and contrast ratio between two sRGB colors
pub fn calculate_wcag_contrast(r1: f64, g1: f64, b1: f64, r2: f64, g2: f64, b2: f64) -> f64 {
    fn srgb_to_linear(c: f64) -> f64 {
        let v = (c / 255.0).clamp(0.0, 1.0);
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }
    let l1 =
        0.2126 * srgb_to_linear(r1) + 0.7152 * srgb_to_linear(g1) + 0.0722 * srgb_to_linear(b1);
    let l2 =
        0.2126 * srgb_to_linear(r2) + 0.7152 * srgb_to_linear(g2) + 0.0722 * srgb_to_linear(b2);
    let max = l1.max(l2);
    let min = l1.min(l2);
    (max + 0.05) / (min + 0.05)
}

pub fn render_hue_wheel(
    ui: &mut egui::Ui,
    node: &WidgetNode,
    state: &DynamicWindowState,
    palette: &ThemePalette,
) {
    if let WidgetNode::HueWheel {
        source_color_id,
        show_pointer,
        markers_binding,
        connect_mode_binding,
        ..
    } = node
    {
        let source_hex = {
            let raw_val = state.values.get(source_color_id).and_then(|v| v.as_str());
            match raw_val {
                Some(s) if s.starts_with('#') => s.to_string(),
                Some(target_id) => state
                    .values
                    .get(target_id)
                    .and_then(|v| v.as_str())
                    .unwrap_or("#ff0000")
                    .to_string(),
                None => "#ff0000".to_string(),
            }
        };
        let parsed = parse_hex_color(&source_hex).unwrap_or(Color32::from_rgb(255, 0, 0));
        let hue_deg = rgb_to_hue_degrees(parsed);

        let markers: Vec<String> = markers_binding
            .as_ref()
            .and_then(|mb| state.values.get(mb))
            .and_then(|v| v.as_string_list())
            .map(|slice| slice.to_vec())
            .unwrap_or_default();

        let connect_mode = connect_mode_binding
            .as_ref()
            .and_then(|cmb| state.values.get(cmb))
            .and_then(|v| v.as_str())
            .unwrap_or("none");

        ui.vertical_centered(|ui| {
            draw_vector_hue_wheel(ui, hue_deg, *show_pointer, &markers, connect_mode, palette);
        });
    }
}

/// Renders a pure vector HSV hue wheel without bitmaps or tofu glyphs
pub fn draw_vector_hue_wheel(
    ui: &mut egui::Ui,
    active_hue_deg: f32,
    show_pointer: bool,
    markers: &[String],
    connect_mode: &str,
    palette: &ThemePalette,
) {
    let size = Vec2::new(140.0, 140.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let center = rect.center();
    let outer_radius = 60.0;
    let inner_radius = 42.0;

    let painter = ui.painter();
    let num_segments = 36;
    let step_rad = std::f32::consts::TAU / num_segments as f32;

    for i in 0..num_segments {
        let angle1 = i as f32 * step_rad;
        let angle2 = (i + 1) as f32 * step_rad;
        let mid_angle_deg = ((i as f32 + 0.5) / num_segments as f32) * 360.0;
        let seg_color = hsv_to_rgb(mid_angle_deg, 0.95, 0.95);

        let p1 = center + Vec2::new(angle1.cos(), angle1.sin()) * inner_radius;
        let p2 = center + Vec2::new(angle1.cos(), angle1.sin()) * outer_radius;
        let p3 = center + Vec2::new(angle2.cos(), angle2.sin()) * outer_radius;
        let p4 = center + Vec2::new(angle2.cos(), angle2.sin()) * inner_radius;

        painter.add(egui::Shape::convex_polygon(
            vec![p1, p2, p3, p4],
            seg_color,
            Stroke::NONE,
        ));
    }

    // Inner circle background
    painter.circle_filled(center, inner_radius - 1.0, palette.panel_fill);
    painter.circle_stroke(
        center,
        outer_radius,
        Stroke::new(1.0, palette.border_stroke),
    );
    painter.circle_stroke(
        center,
        inner_radius,
        Stroke::new(1.0, palette.border_stroke),
    );

    struct MarkerVertex {
        hue: f32,
        pt: egui::Pos2,
        color: Color32,
    }

    let ring_radius = (inner_radius + outer_radius) * 0.5;
    let mut vertices: Vec<MarkerVertex> = Vec::new();
    for hex in markers {
        if let Some(c) = parse_hex_color(hex) {
            let hue = rgb_to_hue_degrees(c);
            let rad = hue.to_radians();
            let dir = Vec2::new(rad.cos(), rad.sin());
            let pt = center + dir * ring_radius;
            vertices.push(MarkerVertex { hue, pt, color: c });
        }
    }

    let line_stroke = Stroke::new(1.2, Color32::from_rgba_unmultiplied(220, 220, 220, 160));

    // Connect markers according to connect_mode
    match connect_mode.to_ascii_lowercase().as_str() {
        "polygon" if vertices.len() >= 2 => {
            let mut sorted_indices: Vec<usize> = (0..vertices.len()).collect();
            sorted_indices.sort_by(|&a, &b| {
                vertices[a]
                    .hue
                    .partial_cmp(&vertices[b].hue)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            let sorted_pts: Vec<egui::Pos2> =
                sorted_indices.iter().map(|&i| vertices[i].pt).collect();

            if sorted_pts.len() >= 3 {
                painter.add(egui::Shape::closed_line(sorted_pts, line_stroke));
            } else if sorted_pts.len() == 2 {
                painter.line_segment([sorted_pts[0], sorted_pts[1]], line_stroke);
            }
        }
        "diameter" if !vertices.is_empty() => {
            if vertices.len() >= 2 {
                painter.line_segment([vertices[0].pt, vertices[1].pt], line_stroke);
            } else {
                painter.line_segment([center, vertices[0].pt], line_stroke);
            }
            painter.circle_filled(center, 2.0, line_stroke.color);
        }
        "spokes" if !vertices.is_empty() => {
            for v in &vertices {
                painter.line_segment([center, v.pt], line_stroke);
            }
            painter.circle_filled(center, 2.0, line_stroke.color);
        }
        _ => {}
    }

    // Draw markers
    if !vertices.is_empty() {
        let mut closest_idx = None;
        let mut min_diff = f32::MAX;
        for (idx, v) in vertices.iter().enumerate() {
            let diff = (v.hue - active_hue_deg).abs();
            let wrapped_diff = diff.min((diff - 360.0).abs());
            if wrapped_diff < min_diff {
                min_diff = wrapped_diff;
                closest_idx = Some(idx);
            }
        }

        for (idx, v) in vertices.iter().enumerate() {
            let is_base = closest_idx == Some(idx) && min_diff < 5.0;

            if is_base {
                painter.circle_filled(v.pt, 5.5, v.color);
                painter.circle_stroke(v.pt, 5.5, Stroke::new(1.5, Color32::WHITE));
                painter.circle_stroke(v.pt, 7.0, Stroke::new(1.2, Color32::from_rgb(20, 20, 20)));
            } else {
                painter.circle_filled(v.pt, 4.0, v.color);
                painter.circle_stroke(v.pt, 4.0, Stroke::new(1.2, Color32::from_rgb(20, 20, 20)));
            }
        }
    } else if show_pointer {
        let rad = active_hue_deg.to_radians();
        let dir = Vec2::new(rad.cos(), rad.sin());
        let pointer_center = center + dir * ring_radius;

        painter.circle_filled(pointer_center, 4.5, Color32::WHITE);
        painter.circle_stroke(
            pointer_center,
            4.5,
            Stroke::new(1.5, Color32::from_rgb(20, 20, 20)),
        );
    }
}
