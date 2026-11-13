pub mod color_slot;
pub mod hue_wheel;
pub mod inputs;
pub mod modifier_stack;
pub mod preview;
pub mod preview_params;
pub mod tabs;
pub mod vector_pad;

use crate::dynamic_tool::state::{DynamicValue, DynamicWindowState};
use crate::theme::ThemePalette;
use core_contracts::{BannerStyle, Language, WidgetNode};
use eframe::egui::{self, Color32, Sense, Stroke, Vec2};
use std::collections::HashMap;

#[allow(clippy::too_many_arguments)]
pub fn render_widget_node(
    ui: &mut egui::Ui,
    plugin_id: &str,
    node: &WidgetNode,
    state: &mut DynamicWindowState,
    palette: &ThemePalette,
    lang: Language,
    preview_thumbnails: &mut HashMap<String, (image::RgbaImage, f32)>,
    preview_textures: &mut HashMap<String, (egui::TextureHandle, u64)>,
    config_path: Option<&std::path::Path>,
) -> bool {
    let mut changed = false;

    match node {
        WidgetNode::ColorSlot { .. } => {
            if color_slot::render_color_slot(ui, node, state, palette, lang) {
                changed = true;
            }
        }

        WidgetNode::Button { id, icon, url, .. } => {
            let action_key = format!("action_{}", id);
            let is_busy = state
                .values
                .get("is_busy")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let label_text = if is_busy {
                "..."
            } else {
                node.label_for_lang(lang)
            };
            let is_swap = !is_busy && icon.as_deref() == Some("swap");

            let resp = ui
                .scope(|ui| {
                    ui.spacing_mut().button_padding = Vec2::new(12.0, 5.0);
                    if is_swap {
                        ui.spacing_mut().button_padding = Vec2::new(18.0, 5.0);
                    }
                    let btn_w = ui.available_width().max(0.0);
                    let mut btn = egui::Button::new(egui::RichText::new(label_text).size(12.0))
                        .min_size(Vec2::new(btn_w, 28.0));
                    if is_busy {
                        btn = btn.sense(Sense::hover());
                    }
                    ui.add(btn)
                })
                .inner;

            if is_busy {
                ui.painter().rect_stroke(
                    resp.rect.expand(1.0),
                    4.0,
                    Stroke::new(1.5, palette.accent_blue),
                    egui::StrokeKind::Outside,
                );
            }

            if is_swap {
                let icon_c = egui::pos2(resp.rect.min.x + 9.5, resp.rect.center().y);
                let stroke_color = if resp.hovered() {
                    palette.accent_hover
                } else {
                    palette.text_primary
                };
                let stroke = Stroke::new(1.2, stroke_color);

                // Top arrow pointing right
                let y_top = icon_c.y - 2.5;
                ui.painter().line_segment(
                    [
                        egui::pos2(icon_c.x - 4.5, y_top),
                        egui::pos2(icon_c.x + 4.5, y_top),
                    ],
                    stroke,
                );
                ui.painter().line_segment(
                    [
                        egui::pos2(icon_c.x + 4.5, y_top),
                        egui::pos2(icon_c.x + 2.0, y_top - 2.0),
                    ],
                    stroke,
                );
                ui.painter().line_segment(
                    [
                        egui::pos2(icon_c.x + 4.5, y_top),
                        egui::pos2(icon_c.x + 2.0, y_top + 2.0),
                    ],
                    stroke,
                );

                // Bottom arrow pointing left
                let y_bot = icon_c.y + 2.5;
                ui.painter().line_segment(
                    [
                        egui::pos2(icon_c.x - 4.5, y_bot),
                        egui::pos2(icon_c.x + 4.5, y_bot),
                    ],
                    stroke,
                );
                ui.painter().line_segment(
                    [
                        egui::pos2(icon_c.x - 4.5, y_bot),
                        egui::pos2(icon_c.x - 2.0, y_bot - 2.0),
                    ],
                    stroke,
                );
                ui.painter().line_segment(
                    [
                        egui::pos2(icon_c.x - 4.5, y_bot),
                        egui::pos2(icon_c.x - 2.0, y_bot + 2.0),
                    ],
                    stroke,
                );
            }

            if resp.clicked() && !is_busy {
                if let Some(target_url) = url {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(target_url));
                }
                state.values.insert(action_key, DynamicValue::Bool(true));
                changed = true;
            }
        }

        WidgetNode::Slider {
            id,
            min,
            max,
            step,
            enabled_binding,
            ..
        } => {
            let is_enabled = enabled_binding
                .as_ref()
                .and_then(|eb| state.values.get(eb))
                .and_then(|v| v.as_bool())
                .or_else(|| {
                    state
                        .values
                        .get(&format!("{}_enabled", id))
                        .and_then(|v| v.as_bool())
                })
                .unwrap_or(true);

            let mut current_val = match state.values.get(id) {
                Some(DynamicValue::Number(n)) => *n,
                _ => *min,
            };

            ui.add_enabled_ui(is_enabled, |ui| {
                let row_resp = ui.horizontal(|ui| {
                    ui.label(node.label_for_lang(lang));
                    let step_val = step.unwrap_or(1.0);
                    let is_integer = step_val >= 1.0;
                    let mut slider = egui::Slider::new(&mut current_val, *min..=*max)
                        .step_by(step_val)
                        .show_value(true);

                    if is_integer {
                        slider = slider
                            .custom_formatter(|v, _| format!("{:.0}", v))
                            .custom_parser(|s| s.trim().parse::<f64>().ok());
                    } else {
                        slider = slider
                            .custom_formatter(|v, _| format!("{:.1}", v))
                            .custom_parser(|s| s.trim().parse::<f64>().ok());
                    }

                    let resp = ui.add(slider);
                    let slider_changed = resp.changed();

                    let mut wheel_ticks: f32 = 0.0;
                    ui.input(|i| {
                        for event in &i.events {
                            if let egui::Event::MouseWheel { delta, .. } = event {
                                if delta.y > 0.0 {
                                    wheel_ticks += 1.0;
                                } else if delta.y < 0.0 {
                                    wheel_ticks -= 1.0;
                                }
                            }
                        }
                    });

                    (resp, slider_changed, wheel_ticks, is_integer)
                });

                let (resp, mut slider_changed, wheel_ticks, is_integer) = row_resp.inner;
                let is_hovered = resp.hovered() || row_resp.response.hovered();

                if is_enabled && wheel_ticks != 0.0 && is_hovered {
                    let wheel_step = step.unwrap_or(if is_integer { 1.0 } else { 0.1 });
                    let new_val =
                        (current_val + (wheel_ticks as f64) * wheel_step).clamp(*min, *max);
                    let new_val = if is_integer {
                        new_val.round()
                    } else {
                        (new_val * 10.0).round() / 10.0
                    };
                    if (new_val - current_val).abs() > f64::EPSILON {
                        current_val = new_val;
                        slider_changed = true;
                    }
                }

                if slider_changed {
                    state
                        .values
                        .insert(id.clone(), DynamicValue::Number(current_val));
                    changed = true;
                }
            });
        }

        WidgetNode::Segmented { id, .. } => {
            let mut current_idx = match state.values.get(id) {
                Some(DynamicValue::Index(i)) => *i,
                _ => 0,
            };

            let options = node.options_for_lang(lang);
            let lbl = node.label_for_lang(lang);

            ui.horizontal(|ui| {
                if !lbl.is_empty() {
                    ui.label(lbl);
                }
                for (idx, opt) in options.iter().enumerate() {
                    let is_selected = idx == current_idx;
                    let btn = egui::Button::new(opt.as_str());
                    let resp = if is_selected {
                        ui.add(btn.stroke(Stroke::new(1.5, palette.accent_blue)))
                    } else {
                        ui.add(btn)
                    };
                    if resp.clicked() {
                        current_idx = idx;
                        state.values.insert(id.clone(), DynamicValue::Index(idx));
                        changed = true;
                    }
                }
            });
        }

        WidgetNode::Dropdown { id, .. } => {
            let mut current_idx = match state.values.get(id) {
                Some(DynamicValue::Index(i)) => *i,
                _ => 0,
            };

            let options = node.options_for_lang(lang);

            ui.horizontal(|ui| {
                ui.label(node.label_for_lang(lang));
                let selected_text = options.get(current_idx).cloned().unwrap_or_default();
                egui::ComboBox::from_id_salt(id)
                    .selected_text(selected_text)
                    .width(ui.available_width() - 8.0)
                    .show_ui(ui, |ui| {
                        for (idx, opt) in options.iter().enumerate() {
                            if ui.selectable_value(&mut current_idx, idx, opt).changed() {
                                state
                                    .values
                                    .insert(id.clone(), DynamicValue::Index(current_idx));
                                changed = true;
                            }
                        }
                    });
            });
        }

        WidgetNode::Toggle { id, .. } => {
            let mut current_bool = match state.values.get(id) {
                Some(DynamicValue::Bool(b)) => *b,
                _ => false,
            };

            if ui
                .checkbox(&mut current_bool, node.label_for_lang(lang))
                .changed()
            {
                state
                    .values
                    .insert(id.clone(), DynamicValue::Bool(current_bool));
                changed = true;
            }
        }

        WidgetNode::HueWheel { .. } => {
            hue_wheel::render_hue_wheel(ui, node, state, palette);
        }

        WidgetNode::Banner {
            message_binding,
            style_binding,
            ..
        } => {
            let msg = match state.values.get(message_binding) {
                Some(DynamicValue::String(s)) => s.clone(),
                _ => String::new(),
            };

            if !msg.is_empty() {
                let style = style_binding
                    .as_ref()
                    .and_then(|sb| state.values.get(sb))
                    .and_then(|v| v.as_banner_style())
                    .unwrap_or(BannerStyle::Info);

                draw_vector_banner(ui, &msg, style, palette);
            }
        }

        WidgetNode::Label { text_binding } => {
            let text = match state.values.get(text_binding) {
                Some(DynamicValue::String(s)) => s.clone(),
                Some(DynamicValue::Number(n)) => {
                    if n.fract() == 0.0 {
                        format!("{:.0}", n)
                    } else {
                        format!("{:.2}", n)
                    }
                }
                Some(DynamicValue::Bool(b)) => b.to_string(),
                _ => String::new(),
            };
            if !text.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new(text).strong());
                });
            }
        }

        WidgetNode::VectorPad { .. } => {
            if vector_pad::render_vector_pad(ui, node, state, palette, lang) {
                changed = true;
            }
        }

        WidgetNode::ImagePreview { .. } => {
            if preview::render_image_preview(
                ui,
                plugin_id,
                node,
                state,
                palette,
                lang,
                preview_thumbnails,
                preview_textures,
            ) {
                changed = true;
            }
        }

        WidgetNode::Tabs { .. } => {
            if tabs::render_tabs(
                ui,
                plugin_id,
                node,
                state,
                palette,
                lang,
                preview_thumbnails,
                preview_textures,
                config_path,
                |ui, pid, n, st, pal, l, pth, ptx, cp| {
                    render_widget_node(ui, pid, n, st, pal, l, pth, ptx, cp)
                },
            ) {
                changed = true;
            }
        }

        WidgetNode::Separator => {
            ui.separator();
        }

        WidgetNode::Row { items } => {
            if items.len() > 1 {
                ui.columns(items.len(), |columns| {
                    for (i, item) in items.iter().enumerate() {
                        if render_widget_node(
                            &mut columns[i],
                            plugin_id,
                            item,
                            state,
                            palette,
                            lang,
                            preview_thumbnails,
                            preview_textures,
                            config_path,
                        ) {
                            changed = true;
                        }
                    }
                });
            } else {
                ui.horizontal(|ui| {
                    for item in items {
                        if render_widget_node(
                            ui,
                            plugin_id,
                            item,
                            state,
                            palette,
                            lang,
                            preview_thumbnails,
                            preview_textures,
                            config_path,
                        ) {
                            changed = true;
                        }
                    }
                });
            }
        }

        WidgetNode::TextInput { .. } => {
            if inputs::render_text_input(ui, node, state, palette, lang) {
                changed = true;
            }
        }

        WidgetNode::ProgressBar { .. } => {
            inputs::render_progress_bar(ui, node, state, palette, lang);
        }

        WidgetNode::ModifierStack { .. } => {
            if modifier_stack::render_modifier_stack(
                ui,
                plugin_id,
                node,
                state,
                palette,
                lang,
                config_path,
            ) {
                changed = true;
            }
        }
    }

    changed
}

/// Renders a vector status banner strictly complying with Vector-First UI (no emojis)
pub fn draw_vector_banner(
    ui: &mut egui::Ui,
    msg: &str,
    style: BannerStyle,
    palette: &ThemePalette,
) {
    let (bg_color, stroke_color) = match style {
        BannerStyle::Info => (
            Color32::from_rgba_unmultiplied(26, 54, 93, 200),
            palette.accent_blue,
        ),
        BannerStyle::Warning => (
            Color32::from_rgba_unmultiplied(120, 53, 15, 200),
            Color32::from_rgb(245, 158, 11),
        ),
        BannerStyle::Error => (
            Color32::from_rgba_unmultiplied(127, 29, 29, 200),
            Color32::from_rgb(239, 68, 68),
        ),
        BannerStyle::Success => (
            Color32::from_rgba_unmultiplied(20, 83, 45, 200),
            Color32::from_rgb(34, 197, 94),
        ),
    };

    egui::Frame::new()
        .fill(bg_color)
        .stroke(Stroke::new(1.0, stroke_color))
        .corner_radius(4.0)
        .inner_margin(egui::Margin::symmetric(8, 6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Vector icon (circle with accent point)
                let (icon_rect, _) = ui.allocate_exact_size(Vec2::new(14.0, 14.0), Sense::hover());
                ui.painter()
                    .circle_stroke(icon_rect.center(), 5.5, Stroke::new(1.5, stroke_color));
                ui.painter()
                    .circle_filled(icon_rect.center(), 2.0, stroke_color);

                ui.add(
                    egui::Label::new(
                        egui::RichText::new(msg)
                            .size(11.0)
                            .color(palette.text_primary),
                    )
                    .wrap_mode(egui::TextWrapMode::Wrap),
                );
            });
        });
}
