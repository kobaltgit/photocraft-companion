use crate::dynamic_tool::state::{DynamicValue, DynamicWindowState};
use crate::theme::ThemePalette;
use core_contracts::{Language, WidgetNode};
use eframe::egui::{self, Color32, Sense, Stroke, Vec2};

pub fn render_vector_pad(
    ui: &mut egui::Ui,
    node: &WidgetNode,
    state: &mut DynamicWindowState,
    palette: &ThemePalette,
    lang: Language,
) -> bool {
    let mut changed = false;

    if let WidgetNode::VectorPad {
        id,
        max_distance,
        default_angle_deg,
        default_distance,
        ..
    } = node
    {
        let angle_key = format!("{}_angle", id);
        let distance_key = format!("{}_distance", id);

        let mut current_angle = state
            .values
            .get(&angle_key)
            .and_then(|v| v.as_f64())
            .unwrap_or(*default_angle_deg);
        let mut current_distance = state
            .values
            .get(&distance_key)
            .and_then(|v| v.as_f64())
            .unwrap_or(*default_distance);

        let label_text = node.label_for_lang(lang);
        if !label_text.is_empty() {
            ui.label(label_text);
        }

        ui.vertical_centered(|ui| {
            let pad_diameter = 130.0;
            let pad_size = Vec2::splat(pad_diameter);
            let (rect, response) = ui.allocate_exact_size(pad_size, Sense::click_and_drag());
            let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

            let center = rect.center();
            let radius = (pad_diameter * 0.5) - 4.0;

            if response.dragged() || response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    let delta = pos - center;
                    let dist_px = (delta.x * delta.x + delta.y * delta.y).sqrt();
                    let norm = (dist_px / radius).clamp(0.0, 1.0);
                    let new_dist = ((norm as f64 * *max_distance) * 10.0).round() / 10.0;

                    let mut rad = delta.y.atan2(delta.x);
                    if rad < 0.0 {
                        rad += std::f32::consts::TAU;
                    }
                    let mut deg = (rad.to_degrees() as f64).round();
                    if deg >= 360.0 {
                        deg -= 360.0;
                    }

                    if (deg - current_angle).abs() > 0.01
                        || (new_dist - current_distance).abs() > 0.01
                    {
                        current_angle = deg;
                        current_distance = new_dist;
                        state
                            .values
                            .insert(angle_key.clone(), DynamicValue::Number(current_angle));
                        state
                            .values
                            .insert(distance_key.clone(), DynamicValue::Number(current_distance));
                        changed = true;
                    }
                }
            }

            let painter = ui.painter();

            // 1. Circular viewfinder background and border
            painter.circle_filled(center, radius, palette.panel_fill);
            painter.circle_stroke(center, radius, Stroke::new(1.0, palette.border_stroke));
            painter.circle_stroke(
                center,
                radius * 0.33,
                Stroke::new(0.5, palette.border_stroke.gamma_multiply(0.4)),
            );
            painter.circle_stroke(
                center,
                radius * 0.66,
                Stroke::new(0.5, palette.border_stroke.gamma_multiply(0.4)),
            );

            // 2. Crosshair guide lines
            let cross_stroke = Stroke::new(1.0, palette.border_stroke.gamma_multiply(0.6));
            painter.line_segment(
                [
                    egui::pos2(center.x - radius, center.y),
                    egui::pos2(center.x + radius, center.y),
                ],
                cross_stroke,
            );
            painter.line_segment(
                [
                    egui::pos2(center.x, center.y - radius),
                    egui::pos2(center.x, center.y + radius),
                ],
                cross_stroke,
            );

            // Center pivot point
            painter.circle_filled(center, 2.5, palette.text_muted);

            // 3. Thrust vector line and handle marker with accent color
            let rad = (current_angle as f32).to_radians();
            let dist_ratio = if *max_distance > 0.0 {
                (current_distance / *max_distance).clamp(0.0, 1.0) as f32
            } else {
                0.0
            };
            let handle_pos = center + Vec2::new(rad.cos(), rad.sin()) * (radius * dist_ratio);

            painter.line_segment([center, handle_pos], Stroke::new(2.0, palette.accent_blue));
            painter.circle_filled(handle_pos, 5.0, palette.accent_blue);
            painter.circle_stroke(handle_pos, 5.0, Stroke::new(1.5, Color32::WHITE));

            // 4. Numeric readout of angle and distance below the viewfinder
            let readout = format!("{:.0} deg, {:.0} px", current_angle, current_distance);
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(readout)
                    .size(11.0)
                    .monospace()
                    .color(palette.text_muted),
            );
        });
    }

    changed
}
