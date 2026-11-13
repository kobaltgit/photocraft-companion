use crate::dynamic_tool::state::{DynamicValue, DynamicWindowState};
use crate::theme::ThemePalette;
use core_contracts::{Language, WidgetNode};
use eframe::egui::{self, Vec2};

pub fn render_text_input(
    ui: &mut egui::Ui,
    node: &WidgetNode,
    state: &mut DynamicWindowState,
    palette: &ThemePalette,
    lang: Language,
) -> bool {
    let mut changed = false;

    let (id, label_text, placeholder, is_password, multiline, action_btn_id, action_btn_label) =
        match node {
            WidgetNode::TextInput {
                id,
                is_password,
                multiline,
                action_button_id,
                ..
            } => (
                id,
                node.label_for_lang(lang),
                node.placeholder_for_lang(lang),
                *is_password,
                *multiline,
                action_button_id.as_deref(),
                node.action_button_label_for_lang(lang),
            ),
            _ => return false,
        };

    let mut current_text = match state.values.get(id) {
        Some(DynamicValue::String(s)) => s.clone(),
        _ => String::new(),
    };

    let is_busy = state
        .values
        .get("is_busy")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let is_visible_key = format!("{}_visible", id);
    let mut is_visible = state
        .values
        .get(&is_visible_key)
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if multiline {
        if !label_text.is_empty() {
            ui.label(egui::RichText::new(label_text).strong().size(12.0));
        }
        let text_edit = egui::TextEdit::multiline(&mut current_text)
            .hint_text(placeholder)
            .desired_rows(3)
            .desired_width(f32::INFINITY);

        if ui.add_enabled(!is_busy, text_edit).changed() {
            state
                .values
                .insert(id.clone(), DynamicValue::String(current_text));
            changed = true;
        }
    } else {
        if !label_text.is_empty() {
            ui.label(egui::RichText::new(label_text).strong().size(12.0));
        }

        ui.horizontal(|ui| {
            let show_as_password = is_password && !is_visible;
            let mut text_edit = egui::TextEdit::singleline(&mut current_text)
                .hint_text(placeholder)
                .password(show_as_password);

            let btn_width = if let Some(btn_label) = action_btn_label {
                let text_w = ui
                    .painter()
                    .layout_no_wrap(
                        btn_label.to_string(),
                        egui::FontId::proportional(12.0),
                        palette.text_primary,
                    )
                    .size()
                    .x;
                (text_w + 16.0).max(64.0)
            } else {
                0.0
            };
            let extra_w = (if is_password { 28.0 } else { 0.0 })
                + (if action_btn_id.is_some() {
                    btn_width + 6.0
                } else {
                    0.0
                });
            let text_edit_w = (ui.available_width() - extra_w).max(80.0);
            text_edit = text_edit.desired_width(text_edit_w);

            if ui.add_enabled(!is_busy, text_edit).changed() {
                state
                    .values
                    .insert(id.clone(), DynamicValue::String(current_text));
                changed = true;
            }

            // Eye toggle button for passwords (Vector-First: SVG via SvgIconCache, no emojis/Unicode glyphs!)
            if is_password {
                let eye_tex =
                    state
                        .icon_cache
                        .get_or_load_eye_icon(ui.ctx(), is_visible, palette.is_dark);

                let btn = if let Some(tex) = eye_tex {
                    let img = egui::Image::new(&tex).fit_to_exact_size(Vec2::new(16.0, 16.0));
                    egui::Button::image(img).min_size(Vec2::new(24.0, 24.0))
                } else {
                    egui::Button::new(if is_visible { "O" } else { "X" })
                        .min_size(Vec2::new(24.0, 24.0))
                };

                if ui.add_enabled(!is_busy, btn).clicked() {
                    is_visible = !is_visible;
                    state
                        .values
                        .insert(is_visible_key, DynamicValue::Bool(is_visible));
                    changed = true;
                }
            }

            // Optional action button (e.g. "Check" or "Save")
            if let (Some(btn_id), Some(btn_label)) = (action_btn_id, action_btn_label) {
                let btn = egui::Button::new(egui::RichText::new(btn_label).size(12.0).strong())
                    .min_size(Vec2::new(btn_width, 22.0));
                if ui.add_enabled(!is_busy, btn).clicked() {
                    state
                        .values
                        .insert(format!("action_{}", btn_id), DynamicValue::Bool(true));
                    changed = true;
                }
            }
        });
    }

    changed
}

pub fn render_progress_bar(
    ui: &mut egui::Ui,
    node: &WidgetNode,
    state: &DynamicWindowState,
    palette: &ThemePalette,
    lang: Language,
) {
    let (id, label_text, show_percentage) = match node {
        WidgetNode::ProgressBar {
            id,
            show_percentage,
            ..
        } => (id, node.label_for_lang(lang), *show_percentage),
        _ => return,
    };

    let progress_val = state.values.get(id).and_then(|v| v.as_f64()).unwrap_or(0.0);

    let fraction = if progress_val > 1.0 {
        (progress_val / 100.0).clamp(0.0, 1.0) as f32
    } else {
        progress_val.clamp(0.0, 1.0) as f32
    };

    let status_text = state
        .values
        .get(&format!("{}_text", id))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    ui.vertical(|ui| {
        if !label_text.is_empty() || !status_text.is_empty() {
            ui.horizontal(|ui| {
                if !label_text.is_empty() {
                    ui.label(egui::RichText::new(label_text).size(11.0).strong());
                }
                if !status_text.is_empty() {
                    ui.label(
                        egui::RichText::new(status_text)
                            .size(11.0)
                            .color(palette.text_muted),
                    );
                }
            });
        }

        let mut bar = egui::ProgressBar::new(fraction)
            .fill(palette.accent_blue)
            .desired_height(14.0)
            .desired_width(ui.available_width());

        if show_percentage {
            bar = bar.text(format!("{:.0}%", fraction * 100.0));
        }

        ui.add(bar);
    });
}
