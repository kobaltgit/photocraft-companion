use super::*;
use eframe::egui::{self, Sense, Stroke, Vec2};

impl ToolstripApp {
    pub fn render_sub_actions_flyout(&mut self, ctx: &egui::Context, palette: &ThemePalette) {
        let (action_id, anchor_rect) = match self.open_sub_actions_for.clone() {
            Some(pair) => pair,
            None => return,
        };

        let btn_idx = match self.buttons.iter().position(|b| b.action == action_id) {
            Some(idx) => idx,
            None => {
                self.open_sub_actions_for = None;
                return;
            }
        };

        if self.buttons[btn_idx].sub_actions.is_empty() {
            self.open_sub_actions_for = None;
            return;
        }

        let window_min = ctx
            .input(|i| i.viewport().outer_rect)
            .map(|r| r.min)
            .unwrap_or(egui::pos2(0.0, 0.0));

        let sub_count = self.buttons[btn_idx].sub_actions.len() as f32;
        let flyout_w = 230.0_f32;
        let flyout_h = (sub_count * 34.0 + 8.0).max(42.0);

        let mut flyout_pos = if self.is_vertical {
            window_min + egui::vec2(anchor_rect.right() + 4.0, anchor_rect.top())
        } else {
            window_min + egui::vec2(anchor_rect.left(), anchor_rect.bottom() + 4.0)
        };

        if let Some(mon_size) = ctx.input(|i| i.viewport().monitor_size) {
            if flyout_pos.x + flyout_w > mon_size.x {
                flyout_pos.x = (window_min.x + anchor_rect.left() - flyout_w - 4.0).max(0.0);
            }
        }

        let vp = egui::ViewportBuilder::default()
            .with_decorations(false)
            .with_transparent(true)
            .with_always_on_top()
            .with_resizable(false)
            .with_taskbar(false)
            .with_position(flyout_pos)
            .with_inner_size(egui::vec2(flyout_w, flyout_h));

        let mut chosen_sub = None;
        let mut should_close = false;
        let current_lang = self.language;
        let is_dark = palette.is_dark;
        let btn = &self.buttons[btn_idx];

        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("toolstrip_sub_actions_flyout"),
            vp,
            |sub_ctx, _class| {
                if sub_ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                    should_close = true;
                }
                if sub_ctx.input(|i| i.viewport().close_requested()) {
                    should_close = true;
                }

                sub_ctx.set_visuals(palette.to_egui_visuals());

                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(palette.card_inner)
                            .stroke(Stroke::new(1.0, palette.border_stroke))
                            .corner_radius(4.0)
                            .inner_margin(egui::Margin::same(4)),
                    )
                    .show(sub_ctx, |ui| {
                        ui.set_min_width(flyout_w - 8.0);
                        ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);

                        for sub in &btn.sub_actions {
                            let is_active = btn.active_sub_action.as_deref() == Some(&sub.id);
                            let sub_label = sub.name_for_lang(current_lang);
                            let sub_icon_path = btn.sub_action_icon_path(sub);

                            let (row_rect, row_resp) = ui.allocate_exact_size(
                                Vec2::new(ui.available_width(), 32.0),
                                Sense::click(),
                            );

                            if row_resp.hovered() {
                                ui.painter().rect_filled(
                                    row_rect,
                                    3.0,
                                    palette.button_secondary_hover,
                                );
                            } else if is_active {
                                ui.painter().rect_filled(row_rect, 3.0, palette.panel_fill);
                            }

                            if is_active {
                                let bar_rect = egui::Rect::from_min_size(
                                    row_rect.left_top() + Vec2::new(1.0, 3.0),
                                    Vec2::new(3.0, row_rect.height() - 6.0),
                                );
                                ui.painter().rect_filled(bar_rect, 1.0, palette.accent_blue);
                            }

                            let icon_rect = egui::Rect::from_center_size(
                                row_rect.left_center() + Vec2::new(16.0, 0.0),
                                Vec2::splat(20.0),
                            );
                            let cache_key = format!("{}#{}", btn.action, sub.id);
                            let maybe_tex = self.icon_cache.get_or_load(
                                ui.ctx(),
                                &cache_key,
                                sub_icon_path.as_deref(),
                                is_dark,
                            );

                            if let Some(tex) = maybe_tex {
                                let img =
                                    egui::Image::new(&tex).fit_to_exact_size(Vec2::splat(20.0));
                                img.paint_at(ui, icon_rect);
                            } else {
                                ui.painter().rect_stroke(
                                    icon_rect.shrink(3.0),
                                    2.0,
                                    Stroke::new(1.0, palette.text_muted),
                                    egui::StrokeKind::Inside,
                                );
                            }

                            let text_color = if is_active {
                                palette.accent_blue
                            } else if row_resp.hovered() {
                                palette.text_primary
                            } else {
                                palette.text_muted
                            };

                            ui.painter().text(
                                row_rect.left_center() + Vec2::new(34.0, 0.0),
                                egui::Align2::LEFT_CENTER,
                                sub_label,
                                egui::FontId::proportional(12.0),
                                text_color,
                            );

                            if row_resp.clicked() {
                                chosen_sub = Some(sub.id.clone());
                            }
                        }
                    });
            },
        );

        if let Some(sub_id) = chosen_sub {
            let btn = &mut self.buttons[btn_idx];
            btn.active_sub_action = Some(sub_id.clone());
            self.pending_action = Some(format!("{}#{}", btn.action, sub_id));
            self.open_sub_actions_for = None;
        } else if should_close {
            self.open_sub_actions_for = None;
        }
    }
}
