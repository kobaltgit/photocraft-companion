use crate::dynamic_tool::state::{DynamicValue, DynamicWindowState};
use crate::dynamic_tool::widgets::hue_wheel::parse_hex_color;
use crate::theme::ThemePalette;
use core_contracts::{Language, WidgetNode};
use eframe::egui::{self, Color32, Sense, Stroke, Vec2};

pub fn render_color_slot(
    ui: &mut egui::Ui,
    node: &WidgetNode,
    state: &mut DynamicWindowState,
    palette: &ThemePalette,
    lang: Language,
) -> bool {
    let mut changed = false;

    if let WidgetNode::ColorSlot {
        id,
        read_only,
        selectable,
        target_binding,
        label_binding,
        ..
    } = node
    {
        let mut current_hex = match state.values.get(id) {
            Some(DynamicValue::String(s)) => s.clone(),
            _ => "#000000".to_string(),
        };

        let parsed_color = parse_hex_color(&current_hex).unwrap_or(Color32::BLACK);

        let is_selected = target_binding
            .as_ref()
            .and_then(|tb| state.values.get(tb))
            .and_then(|v| v.as_str())
            .map(|sel_id| sel_id == id)
            .unwrap_or(false);

        ui.horizontal(|ui| {
            if *read_only {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(20.0, 20.0), Sense::hover());
                ui.painter().rect_filled(rect, 4.0, parsed_color);
                ui.painter().rect_stroke(
                    rect,
                    4.0,
                    Stroke::new(1.0, palette.border_stroke),
                    egui::StrokeKind::Inside,
                );
            } else {
                let mut srgba = parsed_color;
                if egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut srgba,
                    egui::color_picker::Alpha::Opaque,
                )
                .changed()
                {
                    current_hex = format!("#{:02x}{:02x}{:02x}", srgba.r(), srgba.g(), srgba.b());
                    state
                        .values
                        .insert(id.clone(), DynamicValue::String(current_hex.clone()));
                    changed = true;
                }
            }

            let bound_label = label_binding
                .as_ref()
                .and_then(|lb| state.values.get(lb))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty());
            let label_text = bound_label.unwrap_or_else(|| node.label_for_lang(lang));
            let label_color = if is_selected {
                palette.accent_blue
            } else {
                palette.text_primary
            };
            let label_resp = ui.add(
                egui::Label::new(
                    egui::RichText::new(label_text)
                        .size(12.0)
                        .color(label_color),
                )
                .sense(Sense::click()),
            );
            if *selectable && label_resp.clicked() {
                if let Some(tb) = target_binding {
                    state
                        .values
                        .insert(tb.clone(), DynamicValue::String(id.clone()));
                    changed = true;
                }
            }
        });
    }

    changed
}
