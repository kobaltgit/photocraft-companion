use crate::dynamic_tool::state::{DynamicValue, DynamicWindowState};
use crate::theme::ThemePalette;
use core_contracts::{Language, WidgetNode};
use eframe::egui::{self, Stroke, Vec2};
use tracing::warn;

pub fn render_modifier_stack(
    ui: &mut egui::Ui,
    plugin_id: &str,
    node: &WidgetNode,
    state: &mut DynamicWindowState,
    palette: &ThemePalette,
    lang: Language,
    config_path: Option<&std::path::Path>,
) -> bool {
    let mut changed = false;

    if let WidgetNode::ModifierStack {
        id,
        items,
        allow_reordering,
        ..
    } = node
    {
        ui.set_width(ui.available_width());
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label(
                egui::RichText::new(node.label_for_lang(lang))
                    .strong()
                    .size(13.0)
                    .color(palette.text_primary),
            );
            ui.add_space(2.0);

            let order_key = format!("{}_order", id);
            let mut current_order: Vec<String> = match state.values.get(&order_key) {
                Some(DynamicValue::StringList(list)) => list.clone(),
                _ => items.iter().map(|it| it.id.clone()).collect(),
            };

            let mut swap_pair = None;

            let font_id = egui::FontId::proportional(12.0);
            let max_label_w = items
                .iter()
                .map(|it| {
                    ui.painter()
                        .layout_no_wrap(
                            it.label_for_lang(lang).to_string(),
                            font_id.clone(),
                            palette.text_primary,
                        )
                        .size()
                        .x
                })
                .fold(80.0_f32, |a, b| a.max(b))
                + 12.0;

            for (idx, item_id) in current_order.iter().enumerate() {
                if let Some(item_cfg) = items.iter().find(|it| &it.id == item_id) {
                    ui.horizontal(|ui| {
                        if *allow_reordering {
                            // Move Up Vector Button
                            let up_enabled = idx > 0;
                            let up_resp = ui.add_enabled(
                                up_enabled,
                                egui::Button::new("")
                                    .min_size(Vec2::new(18.0, 18.0))
                                    .corner_radius(2.0),
                            );
                            let c = up_resp.rect.center();
                            let stroke_color = if up_enabled {
                                if up_resp.hovered() {
                                    palette.accent_hover
                                } else {
                                    palette.text_primary
                                }
                            } else {
                                palette.text_muted
                            };
                            let stroke = Stroke::new(1.2, stroke_color);
                            ui.painter().line_segment(
                                [egui::pos2(c.x - 3.5, c.y + 1.5), egui::pos2(c.x, c.y - 2.0)],
                                stroke,
                            );
                            ui.painter().line_segment(
                                [egui::pos2(c.x, c.y - 2.0), egui::pos2(c.x + 3.5, c.y + 1.5)],
                                stroke,
                            );
                            if up_resp.clicked() && idx > 0 {
                                swap_pair = Some((idx, idx - 1));
                            }

                            // Move Down Vector Button
                            let down_enabled = idx + 1 < current_order.len();
                            let down_resp = ui.add_enabled(
                                down_enabled,
                                egui::Button::new("")
                                    .min_size(Vec2::new(18.0, 18.0))
                                    .corner_radius(2.0),
                            );
                            let c = down_resp.rect.center();
                            let stroke_color = if down_enabled {
                                if down_resp.hovered() {
                                    palette.accent_hover
                                } else {
                                    palette.text_primary
                                }
                            } else {
                                palette.text_muted
                            };
                            let stroke = Stroke::new(1.2, stroke_color);
                            ui.painter().line_segment(
                                [egui::pos2(c.x - 3.5, c.y - 1.5), egui::pos2(c.x, c.y + 2.0)],
                                stroke,
                            );
                            ui.painter().line_segment(
                                [egui::pos2(c.x, c.y + 2.0), egui::pos2(c.x + 3.5, c.y - 1.5)],
                                stroke,
                            );
                            if down_resp.clicked() && idx + 1 < current_order.len() {
                                swap_pair = Some((idx, idx + 1));
                            }
                        }

                        // Active Checkbox
                        let enabled_key = format!("{}_enabled", item_id);
                        let mut enabled = state
                            .values
                            .get(&enabled_key)
                            .and_then(|v| v.as_bool())
                            .unwrap_or(item_cfg.enabled);
                        if ui.checkbox(&mut enabled, "").changed() {
                            state
                                .values
                                .insert(enabled_key.clone(), DynamicValue::Bool(enabled));
                            state.values.insert(
                                format!("{}_{}_enabled", id, item_id),
                                DynamicValue::Bool(enabled),
                            );
                            changed = true;
                        }

                        // Item label with fixed width slot for crisp zero-stair-step alignment
                        ui.add_sized(
                            egui::vec2(max_label_w, ui.spacing().interact_size.y),
                            egui::Label::new(
                                egui::RichText::new(item_cfg.label_for_lang(lang))
                                    .size(12.0)
                                    .color(if enabled {
                                        palette.text_primary
                                    } else {
                                        palette.text_muted
                                    }),
                            )
                            .truncate(),
                        );

                        // Parameter Value
                        let val_key = format!("{}_value", item_id);
                        let mut val = state
                            .values
                            .get(&val_key)
                            .and_then(|v| v.as_f64())
                            .unwrap_or(item_cfg.default);
                        let step = item_cfg.step.unwrap_or(1.0);
                        let suffix = if let Some(ref u) = item_cfg.unit {
                            format!(" {}", u)
                        } else {
                            String::new()
                        };

                        let is_integer = step >= 1.0;
                        let mut slider = egui::Slider::new(&mut val, item_cfg.min..=item_cfg.max)
                            .step_by(step)
                            .show_value(true);

                        if is_integer {
                            let s_clone = suffix.clone();
                            slider = slider
                                .custom_formatter(move |v, _| format!("{:.0}{}", v, s_clone))
                                .custom_parser({
                                    let s_clone = suffix.clone();
                                    move |s| s.trim_end_matches(&s_clone).trim().parse::<f64>().ok()
                                });
                        } else {
                            let s_clone = suffix.clone();
                            slider = slider
                                .custom_formatter(move |v, _| format!("{:.1}{}", v, s_clone))
                                .custom_parser({
                                    let s_clone = suffix.clone();
                                    move |s| s.trim_end_matches(&s_clone).trim().parse::<f64>().ok()
                                });
                        }

                        let slider_resp = ui
                            .add_enabled_ui(enabled, |ui| {
                                ui.add_sized(
                                    egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
                                    slider,
                                )
                            })
                            .inner;

                        let mut val_changed = slider_resp.changed() && enabled;

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

                        if wheel_ticks != 0.0 && slider_resp.hovered() && enabled {
                            let wheel_step = if is_integer { 1.0 } else { 0.1 };
                            let new_val = (val + (wheel_ticks as f64) * wheel_step)
                                .clamp(item_cfg.min, item_cfg.max);
                            let new_val = if is_integer {
                                new_val.round()
                            } else {
                                (new_val * 10.0).round() / 10.0
                            };
                            if (new_val - val).abs() > f64::EPSILON {
                                val = new_val;
                                val_changed = true;
                            }
                        }

                        if val_changed {
                            state
                                .values
                                .insert(val_key.clone(), DynamicValue::Number(val));
                            state.values.insert(
                                format!("{}_{}_value", id, item_id),
                                DynamicValue::Number(val),
                            );
                            changed = true;
                        }
                    });
                }
            }

            if let Some((i1, i2)) = swap_pair {
                current_order.swap(i1, i2);
                state
                    .values
                    .insert(order_key, DynamicValue::StringList(current_order.clone()));
                state
                    .values
                    .insert(id.clone(), DynamicValue::StringList(current_order.clone()));
                if let Some(cfg_path) = config_path {
                    let mut cfg = core_contracts::CompanionConfig::load_or_default(cfg_path);
                    cfg.modifier_stack_orders
                        .insert(format!("{}:{}", plugin_id, id), current_order.clone());
                    cfg.modifier_stack_orders
                        .insert(id.clone(), current_order.clone());
                    if let Err(e) = cfg.save(cfg_path) {
                        warn!("Failed to persist modifier stack order to config: {}", e);
                    }
                }
                changed = true;
            }
        });
    }

    changed
}
