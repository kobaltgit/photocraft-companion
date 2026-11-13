use super::*;
use eframe::egui::{self, RichText, Sense, Stroke, Ui, Vec2};

impl ToolstripApp {
    pub fn render(&mut self, ui: &mut Ui, palette: &ThemePalette) {
        self.hovered_tooltip = None;

        // Draw frameless panel background matching PhotoCraft tool palettes with 2px corner radius
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 2.0, palette.panel_fill);
        ui.painter().rect_stroke(
            rect,
            2.0,
            Stroke::new(1.0, palette.border_stroke),
            egui::StrokeKind::Inside,
        );

        let is_busy = self.is_busy;
        let active_action = self.active_action.clone();
        let is_dark = palette.is_dark;
        let has_selection = self.has_selection;

        let mut sub_action_open = None;
        let mut sub_action_toggle = None;
        let current_lang = self.language;
        let render_button_items = |ui: &mut Ui,
                                   buttons: &mut [ButtonConfig],
                                   icon_cache: &mut crate::icons::SvgIconCache,
                                   pending: &mut Option<String>,
                                   hovered_tt: &mut Option<(String, egui::Rect)>,
                                   pointer_down: &mut Option<(String, std::time::Instant)>,
                                   long_press_fired: &mut Option<String>,
                                   suppress_click: &mut Option<String>,
                                   sub_open: &mut Option<(String, egui::Rect)>,
                                   sub_toggle: &mut Option<(String, egui::Rect)>,
                                   badges: &std::collections::HashMap<
            String,
            core_contracts::DynamicBadgeState,
        >| {
            ui.spacing_mut().item_spacing = Vec2::new(4.0, 4.0);
            for btn in buttons.iter_mut() {
                let is_this_active = active_action
                    .as_deref()
                    .map(|a| a == btn.action || a.starts_with(&format!("{}#", btn.action)))
                    .unwrap_or(false);

                let has_sub_actions = !btn.sub_actions.is_empty();

                let effective_icon_path = btn.effective_icon_path();
                let active_sub = btn
                    .active_sub_action
                    .as_ref()
                    .and_then(|sub_id| btn.sub_actions.iter().find(|s| &s.id == sub_id));

                let cache_key = match active_sub {
                    Some(sub) => format!("{}:{}", btn.action, sub.id),
                    None => btn.action.clone(),
                };
                let maybe_texture = icon_cache.get_or_load(
                    ui.ctx(),
                    &cache_key,
                    effective_icon_path.as_deref(),
                    is_dark,
                );

                let btn_name = btn.name_for_lang(current_lang);
                let tooltip = if let Some(sub) = active_sub {
                    let sub_name = sub.name_for_lang(current_lang);
                    if btn_name.ends_with(')') {
                        format!("{}: {}", btn_name, sub_name)
                    } else {
                        format!("{} ({})", btn_name, sub_name)
                    }
                } else {
                    btn_name.to_string()
                };

                let b = if let Some(tex) = maybe_texture {
                    let img = egui::Image::new(&tex).fit_to_exact_size(Vec2::new(20.0, 20.0));
                    egui::Button::image(img)
                        .min_size(Vec2::new(32.0, 32.0))
                        .corner_radius(3.0)
                } else {
                    let display_label = if is_this_active {
                        "..."
                    } else if btn.label.len() <= 3 {
                        &btn.label
                    } else {
                        &btn.label[..3]
                    };

                    let text_color = if is_this_active {
                        palette.accent_blue
                    } else {
                        palette.text_primary
                    };

                    egui::Button::new(
                        RichText::new(display_label)
                            .strong()
                            .size(11.0)
                            .color(text_color),
                    )
                    .min_size(Vec2::new(32.0, 32.0))
                    .corner_radius(3.0)
                };

                let resp = ui.add_enabled(!is_busy || is_this_active, b);
                if is_this_active {
                    ui.painter().rect_stroke(
                        resp.rect,
                        3.0,
                        Stroke::new(1.5, palette.accent_blue),
                        egui::StrokeKind::Inside,
                    );
                }

                if has_selection && btn.requires_selection {
                    let dot_center = resp.rect.right_top() + Vec2::new(-4.5, 4.5);
                    let glow_color = if is_dark {
                        egui::Color32::from_rgba_unmultiplied(56, 189, 248, 120)
                    } else {
                        egui::Color32::from_rgba_unmultiplied(14, 165, 233, 120)
                    };
                    let dot_color = if is_dark {
                        egui::Color32::from_rgb(56, 189, 248)
                    } else {
                        egui::Color32::from_rgb(14, 165, 233)
                    };
                    ui.painter().circle_filled(dot_center, 3.5, glow_color);
                    ui.painter().circle_filled(dot_center, 2.0, dot_color);
                }

                if let Some(badge) = badges.get(&btn.action) {
                    let center = egui::pos2(resp.rect.right() - 8.0, resp.rect.top() + 8.0);
                    if badge.is_pulsing {
                        let t = ui.input(|i| i.time);
                        let phase = t.fract();
                        let is_lit = phase < 0.6;
                        let alpha = if is_lit { 1.0f32 } else { 0.25f32 };
                        let [r, g, b, _] = badge.pulse_color_rgba;
                        let col =
                            egui::Color32::from_rgba_premultiplied(r, g, b, (255.0 * alpha) as u8);
                        ui.painter().circle_filled(center, 3.5, col);
                        let next_switch = if is_lit {
                            (0.6 - phase).max(0.05)
                        } else {
                            (1.0 - phase).max(0.05)
                        };
                        ui.ctx()
                            .request_repaint_after(std::time::Duration::from_secs_f64(next_switch));
                    }

                    if let Some(ref lbl) = badge.label {
                        if !lbl.is_empty() {
                            let font_id = egui::FontId::proportional(9.0);
                            let galley = ui.painter().layout_no_wrap(
                                lbl.clone(),
                                font_id,
                                palette.text_primary,
                            );
                            let label_pos = if badge.is_pulsing {
                                egui::pos2(
                                    center.x - galley.size().x - 3.0,
                                    center.y - galley.size().y * 0.5,
                                )
                            } else {
                                egui::pos2(
                                    resp.rect.right() - galley.size().x - 4.0,
                                    resp.rect.top() + 2.0,
                                )
                            };
                            ui.painter().galley(label_pos, galley, palette.text_primary);
                        }
                    }
                }

                if has_sub_actions {
                    let p1 = resp.rect.right_bottom() - Vec2::new(6.0, 1.0);
                    let p2 = resp.rect.right_bottom() - Vec2::new(1.0, 6.0);
                    let p3 = resp.rect.right_bottom() - Vec2::new(1.0, 1.0);
                    ui.painter().add(egui::Shape::convex_polygon(
                        vec![p1, p2, p3],
                        palette.text_muted,
                        egui::Stroke::NONE,
                    ));
                }

                if resp.hovered() {
                    *hovered_tt = Some((tooltip, resp.rect));
                }

                if has_sub_actions {
                    if resp.secondary_clicked() {
                        *sub_toggle = Some((btn.action.clone(), resp.rect));
                    } else if resp.is_pointer_button_down_on() {
                        if long_press_fired.as_deref() == Some(&btn.action) {
                            // Already fired long-press: keep sub-actions open steadily, NO flicker!
                        } else {
                            match pointer_down.as_ref() {
                                Some((act, start)) if act == &btn.action => {
                                    if start.elapsed().as_millis() >= 250 {
                                        *sub_open = Some((btn.action.clone(), resp.rect));
                                        *long_press_fired = Some(btn.action.clone());
                                        *suppress_click = Some(btn.action.clone());
                                        ui.ctx().request_repaint_after(
                                            std::time::Duration::from_millis(50),
                                        );
                                    }
                                }
                                _ => {
                                    *pointer_down =
                                        Some((btn.action.clone(), std::time::Instant::now()));
                                }
                            }
                        }
                    }
                }

                if resp.clicked() && !is_busy {
                    let is_chevron_click = has_sub_actions
                        && ui
                            .input(|i| i.pointer.interact_pos())
                            .map(|pos| {
                                pos.x >= resp.rect.right() - 10.0
                                    && pos.y >= resp.rect.bottom() - 10.0
                            })
                            .unwrap_or(false);

                    if is_chevron_click {
                        *sub_toggle = Some((btn.action.clone(), resp.rect));
                    } else if suppress_click.as_deref() == Some(&btn.action) {
                        *suppress_click = None;
                    } else if has_sub_actions {
                        if let Some(ref sub_id) = btn.active_sub_action {
                            *pending = Some(format!("{}#{}", btn.action, sub_id));
                        } else {
                            *pending = Some(btn.action.clone());
                        }
                    } else {
                        *pending = Some(btn.action.clone());
                    }
                }
            }
        };

        if self.is_vertical {
            // Vertical Layout: Top service bar + column of tools
            ui.vertical(|ui| {
                ui.add_space(3.0);

                // Row 1: Drag handle centered horizontally
                ui.horizontal(|ui| {
                    let avail = ui.available_width();
                    if avail > 16.0 {
                        ui.add_space((avail - 16.0) * 0.5);
                    }
                    let handle_size = Vec2::new(16.0, 7.0);
                    let (handle_rect, response) =
                        ui.allocate_exact_size(handle_size, Sense::drag());
                    if response.drag_started() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                    let c = handle_rect.center();
                    for dx in [-4.0, 0.0, 4.0] {
                        ui.painter().circle_filled(
                            c + Vec2::new(dx, -1.8),
                            1.0,
                            palette.text_muted,
                        );
                        ui.painter()
                            .circle_filled(c + Vec2::new(dx, 1.8), 1.0, palette.text_muted);
                    }
                });

                ui.add_space(2.0);

                // Row 2: Service controls (Orient, Settings, Minimize)
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(2.0, 0.0);
                    let total_w = 13.0 * 3.0 + 4.0;
                    let avail = ui.available_width();
                    if avail > total_w {
                        ui.add_space((avail - total_w) * 0.5);
                    }

                    // Orientation toggle (Switch to Horizontal)
                    let orient_btn = egui::Button::new("")
                        .min_size(Vec2::new(13.0, 13.0))
                        .corner_radius(2.0);
                    let resp = ui.add(orient_btn);
                    if resp.hovered() {
                        self.hovered_tooltip =
                            Some(("Switch to Horizontal".to_string(), resp.rect));
                    }
                    let c = resp.rect.center();
                    ui.painter().line_segment(
                        [c - Vec2::new(3.0, 0.0), c + Vec2::new(3.0, 0.0)],
                        Stroke::new(1.2, palette.text_muted),
                    );
                    ui.painter().line_segment(
                        [c - Vec2::new(3.0, 0.0), c - Vec2::new(1.5, 2.0)],
                        Stroke::new(1.0, palette.text_muted),
                    );
                    ui.painter().line_segment(
                        [c - Vec2::new(3.0, 0.0), c - Vec2::new(1.5, -2.0)],
                        Stroke::new(1.0, palette.text_muted),
                    );
                    ui.painter().line_segment(
                        [c + Vec2::new(3.0, 0.0), c + Vec2::new(1.5, 2.0)],
                        Stroke::new(1.0, palette.text_muted),
                    );
                    ui.painter().line_segment(
                        [c + Vec2::new(3.0, 0.0), c + Vec2::new(1.5, -2.0)],
                        Stroke::new(1.0, palette.text_muted),
                    );
                    if resp.clicked() {
                        self.is_vertical = false;
                        self.orientation_changed = true;
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::InnerSize(
                            self.calculate_size(),
                        ));
                    }

                    // Settings gear (vector drawn)
                    let gear_btn = egui::Button::new("")
                        .min_size(Vec2::new(13.0, 13.0))
                        .corner_radius(2.0);
                    let resp_gear = ui.add(gear_btn);
                    if resp_gear.hovered() {
                        self.hovered_tooltip =
                            Some(("Plugins Manager".to_string(), resp_gear.rect));
                    }
                    let cg = resp_gear.rect.center();
                    ui.painter()
                        .circle_stroke(cg, 2.3, Stroke::new(1.0, palette.text_muted));
                    ui.painter().line_segment(
                        [cg - Vec2::new(0.0, 3.5), cg + Vec2::new(0.0, 3.5)],
                        Stroke::new(1.0, palette.text_muted),
                    );
                    ui.painter().line_segment(
                        [cg - Vec2::new(3.5, 0.0), cg + Vec2::new(3.5, 0.0)],
                        Stroke::new(1.0, palette.text_muted),
                    );
                    if resp_gear.clicked() {
                        self.open_manager_requested = true;
                    }

                    // Minimize to tray
                    let min_btn = egui::Button::new("")
                        .min_size(Vec2::new(13.0, 13.0))
                        .corner_radius(2.0);
                    let resp_min = ui.add(min_btn);
                    if resp_min.hovered() {
                        self.hovered_tooltip =
                            Some(("Minimize to Tray".to_string(), resp_min.rect));
                    }
                    let cm = resp_min.rect.center();
                    ui.painter().line_segment(
                        [cm - Vec2::new(2.8, 0.0), cm + Vec2::new(2.8, 0.0)],
                        Stroke::new(1.2, palette.text_muted),
                    );
                    if resp_min.clicked() {
                        self.hide_requested = true;
                    }
                });

                ui.add_space(2.0);
                ui.separator();
                ui.add_space(2.0);

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);
                    let cols_count =
                        self.buttons.len().div_ceil(MAX_BUTTONS_PER_TIER).max(1) as f32;
                    let total_cols_w = 32.0 * cols_count + (cols_count - 1.0) * 4.0;
                    let avail = ui.available_width();
                    if avail > total_cols_w {
                        ui.add_space((avail - total_cols_w) * 0.5);
                    }

                    for chunk in self.buttons.chunks_mut(MAX_BUTTONS_PER_TIER) {
                        ui.allocate_ui_with_layout(
                            Vec2::new(32.0, ui.available_height()),
                            egui::Layout::top_down(egui::Align::Center),
                            |ui| {
                                render_button_items(
                                    ui,
                                    chunk,
                                    &mut self.icon_cache,
                                    &mut self.pending_action,
                                    &mut self.hovered_tooltip,
                                    &mut self.pointer_down_start,
                                    &mut self.long_press_fired_for,
                                    &mut self.suppress_click_for,
                                    &mut sub_action_open,
                                    &mut sub_action_toggle,
                                    &self.dynamic_badges,
                                );
                            },
                        );
                    }
                });
                ui.add_space(3.0);
            });
        } else {
            // Horizontal Layout (Two-Tier): Tier 1 service row ON TOP, Tier 2 plugin buttons BELOW
            ui.vertical(|ui| {
                ui.add_space(3.0);

                // Tier 1: Service controls header
                ui.horizontal(|ui| {
                    ui.add_space(3.0);

                    // Drag handle
                    let handle_size = Vec2::new(20.0, 14.0);
                    let (handle_rect, response) =
                        ui.allocate_exact_size(handle_size, Sense::drag());
                    if response.drag_started() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                    let c = handle_rect.center();
                    for dx in [-6.0, -2.0, 2.0, 6.0] {
                        ui.painter().circle_filled(
                            c + Vec2::new(dx, -2.5),
                            1.0,
                            palette.text_muted,
                        );
                        ui.painter()
                            .circle_filled(c + Vec2::new(dx, 2.5), 1.0, palette.text_muted);
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(2.0);

                        // Minimize to tray (clean vector line)
                        let min_btn = egui::Button::new("")
                            .min_size(Vec2::new(14.0, 14.0))
                            .corner_radius(2.0);
                        let resp_min = ui.add(min_btn);
                        if resp_min.hovered() {
                            self.hovered_tooltip =
                                Some(("Minimize to Tray".to_string(), resp_min.rect));
                        }
                        let cm = resp_min.rect.center();
                        ui.painter().line_segment(
                            [cm - Vec2::new(3.0, 0.0), cm + Vec2::new(3.0, 0.0)],
                            Stroke::new(1.2, palette.text_muted),
                        );
                        if resp_min.clicked() {
                            self.hide_requested = true;
                        }

                        // Settings gear (clean vector drawn)
                        let gear_btn = egui::Button::new("")
                            .min_size(Vec2::new(14.0, 14.0))
                            .corner_radius(2.0);
                        let resp_gear = ui.add(gear_btn);
                        if resp_gear.hovered() {
                            self.hovered_tooltip =
                                Some(("Plugins Manager".to_string(), resp_gear.rect));
                        }
                        let cg = resp_gear.rect.center();
                        ui.painter()
                            .circle_stroke(cg, 2.6, Stroke::new(1.0, palette.text_muted));
                        ui.painter().line_segment(
                            [cg - Vec2::new(0.0, 4.0), cg + Vec2::new(0.0, 4.0)],
                            Stroke::new(1.0, palette.text_muted),
                        );
                        ui.painter().line_segment(
                            [cg - Vec2::new(4.0, 0.0), cg + Vec2::new(4.0, 0.0)],
                            Stroke::new(1.0, palette.text_muted),
                        );
                        if resp_gear.clicked() {
                            self.open_manager_requested = true;
                        }

                        // Toggle orientation (Switch to Vertical)
                        let orient_btn = egui::Button::new("")
                            .min_size(Vec2::new(14.0, 14.0))
                            .corner_radius(2.0);
                        let resp_orient = ui.add(orient_btn);
                        if resp_orient.hovered() {
                            self.hovered_tooltip =
                                Some(("Switch to Vertical".to_string(), resp_orient.rect));
                        }
                        let c = resp_orient.rect.center();
                        ui.painter().line_segment(
                            [c - Vec2::new(0.0, 3.5), c + Vec2::new(0.0, 3.5)],
                            Stroke::new(1.2, palette.text_muted),
                        );
                        ui.painter().line_segment(
                            [c - Vec2::new(0.0, 3.5), c - Vec2::new(2.0, 1.5)],
                            Stroke::new(1.0, palette.text_muted),
                        );
                        ui.painter().line_segment(
                            [c - Vec2::new(0.0, 3.5), c - Vec2::new(-2.0, 1.5)],
                            Stroke::new(1.0, palette.text_muted),
                        );
                        ui.painter().line_segment(
                            [c + Vec2::new(0.0, 3.5), c + Vec2::new(2.0, -1.5)],
                            Stroke::new(1.0, palette.text_muted),
                        );
                        ui.painter().line_segment(
                            [c + Vec2::new(0.0, 3.5), c + Vec2::new(-2.0, -1.5)],
                            Stroke::new(1.0, palette.text_muted),
                        );
                        if resp_orient.clicked() {
                            self.is_vertical = true;
                            self.orientation_changed = true;
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::InnerSize(
                                self.calculate_size(),
                            ));
                        }
                    });
                });

                ui.add_space(2.0);
                ui.separator();
                ui.add_space(2.0);

                // Tier 2: Large plugin action buttons strictly below service row (multi-row if > 8 buttons)
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(0.0, 4.0);
                    for chunk in self.buttons.chunks_mut(MAX_BUTTONS_PER_TIER) {
                        ui.horizontal(|ui| {
                            ui.add_space(4.0);
                            render_button_items(
                                ui,
                                chunk,
                                &mut self.icon_cache,
                                &mut self.pending_action,
                                &mut self.hovered_tooltip,
                                &mut self.pointer_down_start,
                                &mut self.long_press_fired_for,
                                &mut self.suppress_click_for,
                                &mut sub_action_open,
                                &mut sub_action_toggle,
                                &self.dynamic_badges,
                            );
                        });
                    }
                });
            });
        }

        if let Some(ref target) = sub_action_open {
            self.open_sub_actions_for = Some(target.clone());
        } else if let Some(ref target) = sub_action_toggle {
            if self
                .open_sub_actions_for
                .as_ref()
                .map(|(a, _)| a == &target.0)
                .unwrap_or(false)
            {
                self.open_sub_actions_for = None;
            } else {
                self.open_sub_actions_for = Some(target.clone());
            }
        }

        // When mouse is released anywhere on screen, reset long-press timer and hold state
        if !ui.input(|i| i.pointer.any_down()) {
            self.pointer_down_start = None;
            self.long_press_fired_for = None;
        }

        // Close sub-actions flyout if clicking anywhere else in the toolstrip window
        if let Some((_, anchor_rect)) = self.open_sub_actions_for {
            if sub_action_open.is_none()
                && sub_action_toggle.is_none()
                && self.suppress_click_for.is_none()
                && self.long_press_fired_for.is_none()
                && ui.input(|i| i.pointer.any_click())
            {
                if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                    if !anchor_rect.contains(pos) {
                        self.open_sub_actions_for = None;
                    }
                }
            }
        }
    }
}
