use crate::theme::{ThemeMode, ThemePalette};
use core_contracts::ButtonConfig;
use eframe::egui::{self, RichText, Sense, Stroke, Ui, Vec2};

pub struct ToolstripApp {
    pub is_vertical: bool,
    pub current_theme: ThemeMode,
    pub language: core_contracts::Language,
    pub buttons: Vec<ButtonConfig>,
    pub pending_action: Option<String>,
    pub open_manager_requested: bool,
    pub hide_requested: bool,
    pub hovered_tooltip: Option<(String, egui::Rect)>,
    pub is_busy: bool,
    pub active_action: Option<String>,
    pub orientation_changed: bool,
    pub icon_cache: crate::icons::SvgIconCache,
    pub open_sub_actions_for: Option<(String, egui::Rect)>,
    pub pointer_down_start: Option<(String, std::time::Instant)>,
    pub suppress_click_for: Option<String>,
    pub long_press_fired_for: Option<String>,
}

impl ToolstripApp {
    pub fn new(buttons: Vec<ButtonConfig>, current_theme: ThemeMode) -> Self {
        Self {
            is_vertical: false, // Per spec: default is horizontal
            current_theme,
            language: core_contracts::Language::Auto,
            buttons,
            pending_action: None,
            open_manager_requested: false,
            hide_requested: false,
            hovered_tooltip: None,
            is_busy: false,
            active_action: None,
            orientation_changed: false,
            icon_cache: crate::icons::SvgIconCache::new(),
            open_sub_actions_for: None,
            pointer_down_start: None,
            suppress_click_for: None,
            long_press_fired_for: None,
        }
    }

    pub fn with_language(mut self, language: core_contracts::Language) -> Self {
        self.language = language;
        self
    }

    pub fn with_orientation(mut self, is_vertical: bool) -> Self {
        self.is_vertical = is_vertical;
        self
    }

    /// Dynamic sizing:
    /// Fully adaptive calculation based on count N of active plugins.
    /// Supports arbitrary plugin counts by any 3rd party developer without clipping.
    pub fn calculate_size(&self) -> Vec2 {
        let count = self.buttons.len().max(1) as f32;
        if self.is_vertical {
            // Header + tools (count * 36px) + comfortable bottom margin = count * 36.0 + 68.0
            // Width 50px gives comfortable breathing room for 3-button service row (43px) and tool buttons (32px)
            Vec2::new(50.0, count * 36.0 + 68.0)
        } else {
            // Each button stride 38px, margins 16px, min 116px for service controls
            let tools_width = count * 38.0 + 16.0;
            // Height 78px comfortably holds service row (22px) + separator (6px) + buttons (34px) + padding
            Vec2::new(tools_width.max(116.0), 78.0)
        }
    }

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

        let mut sub_action_open = None;
        let mut sub_action_toggle = None;
        let current_lang = self.language;
        let render_button_items =
            |ui: &mut Ui,
             buttons: &mut [ButtonConfig],
             icon_cache: &mut crate::icons::SvgIconCache,
             pending: &mut Option<String>,
             hovered_tt: &mut Option<(String, egui::Rect)>,
             pointer_down: &mut Option<(String, std::time::Instant)>,
             long_press_fired: &mut Option<String>,
             suppress_click: &mut Option<String>,
             sub_open: &mut Option<(String, egui::Rect)>,
             sub_toggle: &mut Option<(String, egui::Rect)>| {
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
                                            ui.ctx().request_repaint();
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

                ui.vertical_centered(|ui| {
                    render_button_items(
                        ui,
                        &mut self.buttons,
                        &mut self.icon_cache,
                        &mut self.pending_action,
                        &mut self.hovered_tooltip,
                        &mut self.pointer_down_start,
                        &mut self.long_press_fired_for,
                        &mut self.suppress_click_for,
                        &mut sub_action_open,
                        &mut sub_action_toggle,
                    );
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

                // Tier 2: Large plugin action buttons strictly below service row
                ui.horizontal_centered(|ui| {
                    ui.add_space(4.0);
                    render_button_items(
                        ui,
                        &mut self.buttons,
                        &mut self.icon_cache,
                        &mut self.pending_action,
                        &mut self.hovered_tooltip,
                        &mut self.pointer_down_start,
                        &mut self.long_press_fired_for,
                        &mut self.suppress_click_for,
                        &mut sub_action_open,
                        &mut sub_action_toggle,
                    );
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_buttons(count: usize) -> Vec<ButtonConfig> {
        (0..count)
            .map(|i| ButtonConfig {
                label: format!("Tool {}", i + 1),
                action: format!("action_{}", i + 1),
                icon: if i == 0 {
                    Some("wand".to_string())
                } else {
                    Some("bw".to_string())
                },
                icon_path: None,
                plugin_dir: None,
                sub_actions: Vec::new(),
                active_sub_action: None,
                name_ru: None,
                locales: Default::default(),
            })
            .collect()
    }

    #[test]
    fn test_vertical_toolstrip_geometry_no_clipping() {
        for count in [1, 2, 4] {
            let ctx = egui::Context::default();
            let buttons = create_test_buttons(count);
            let mut app = ToolstripApp::new(buttons, ThemeMode::Dark);
            app.is_vertical = true;

            let expected_size = app.calculate_size();
            assert_eq!(expected_size.x, 50.0);
            assert!(expected_size.y >= (count as f32) * 36.0 + 58.0);

            let raw_input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, expected_size)),
                ..Default::default()
            };

            let mut full_output = ctx.run_ui(raw_input, |ctx| {
                let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, expected_size);
                let mut ui = egui::Ui::new(
                    ctx.clone(),
                    egui::Id::new("test_vertical"),
                    egui::UiBuilder::new().max_rect(rect),
                );
                app.render(&mut ui, &ThemePalette::for_mode(ThemeMode::Dark));

                let used_rect = ui.min_rect();
                // Ensure content never overflows the window bounds in vertical mode
                assert!(
                    used_rect.max.y <= expected_size.y,
                    "Vertical mode clipped at bottom for count {}: used y {} > expected y {}",
                    count,
                    used_rect.max.y,
                    expected_size.y
                );
                assert!(
                    used_rect.max.x <= expected_size.x,
                    "Vertical mode clipped at right for count {}: used x {} > expected x {}",
                    count,
                    used_rect.max.x,
                    expected_size.x
                );
            });
            full_output.textures_delta.clear();
        }
    }

    #[test]
    fn test_horizontal_toolstrip_geometry_no_clipping() {
        for count in [1, 2, 4] {
            let ctx = egui::Context::default();
            let buttons = create_test_buttons(count);
            let mut app = ToolstripApp::new(buttons, ThemeMode::Dark);
            app.is_vertical = false;

            let expected_size = app.calculate_size();
            assert_eq!(expected_size.y, 78.0);

            let raw_input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, expected_size)),
                ..Default::default()
            };

            let mut full_output = ctx.run_ui(raw_input, |ctx| {
                let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, expected_size);
                let mut ui = egui::Ui::new(
                    ctx.clone(),
                    egui::Id::new("test_horizontal"),
                    egui::UiBuilder::new().max_rect(rect),
                );
                app.render(&mut ui, &ThemePalette::for_mode(ThemeMode::Dark));

                let used_rect = ui.min_rect();
                assert!(
                    used_rect.max.y <= expected_size.y,
                    "Horizontal mode clipped at bottom for count {}: used y {} > expected y {}",
                    count,
                    used_rect.max.y,
                    expected_size.y
                );
                assert!(
                    used_rect.max.x <= expected_size.x,
                    "Horizontal mode clipped at right for count {}: used x {} > expected x {}",
                    count,
                    used_rect.max.x,
                    expected_size.x
                );
            });
            full_output.textures_delta.clear();
        }
    }

    #[test]
    fn test_toolstrip_size_scaling() {
        // 0 buttons defaults to 1 button minimum
        let app_empty_h = ToolstripApp::new(vec![], ThemeMode::Dark);
        assert_eq!(app_empty_h.calculate_size().y, 78.0);
        assert!(app_empty_h.calculate_size().x >= 116.0);

        let mut app_empty_v = ToolstripApp::new(vec![], ThemeMode::Dark);
        app_empty_v.is_vertical = true;
        assert_eq!(app_empty_v.calculate_size().x, 50.0);
        assert_eq!(app_empty_v.calculate_size().y, 1.0 * 36.0 + 68.0);

        // 5 buttons
        let buttons5 = create_test_buttons(5);
        let mut app5 = ToolstripApp::new(buttons5, ThemeMode::Light);
        assert_eq!(app5.calculate_size().y, 78.0);
        assert_eq!(app5.calculate_size().x, (5.0f32 * 38.0 + 16.0).max(116.0));

        app5.is_vertical = true;
        assert_eq!(app5.calculate_size().x, 50.0);
        assert_eq!(app5.calculate_size().y, 5.0 * 36.0 + 68.0);
    }

    #[test]
    fn test_toolstrip_state_defaults() {
        let app = ToolstripApp::new(vec![], ThemeMode::Dark);
        assert!(!app.is_vertical);
        assert!(app.pending_action.is_none());
        assert!(!app.open_manager_requested);
        assert!(!app.hide_requested);
        assert!(app.hovered_tooltip.is_none());
        assert!(!app.orientation_changed);
        assert!(app.suppress_click_for.is_none());
        assert!(app.long_press_fired_for.is_none());
    }

    #[test]
    fn test_toolstrip_with_orientation() {
        let app_v = ToolstripApp::new(vec![], ThemeMode::Dark).with_orientation(true);
        assert!(app_v.is_vertical);
        assert_eq!(app_v.calculate_size().x, 50.0);

        let app_h = ToolstripApp::new(vec![], ThemeMode::Dark).with_orientation(false);
        assert!(!app_h.is_vertical);
        assert_eq!(app_h.calculate_size().y, 78.0);
    }

    #[test]
    fn test_toolstrip_sub_actions_state() {
        let sub1 = core_contracts::SubAction {
            id: "cutout".to_string(),
            name: "Cutout".to_string(),
            name_ru: None,
            locales: Default::default(),
            icon: Some("cutout.svg".to_string()),
            post_commands: vec![],
            script: None,
        };
        let sub2 = core_contracts::SubAction {
            id: "mask".to_string(),
            name: "Layer Mask".to_string(),
            name_ru: None,
            locales: Default::default(),
            icon: Some("mask.svg".to_string()),
            post_commands: vec![],
            script: None,
        };

        let btn = ButtonConfig {
            label: "Remove BG".to_string(),
            action: "ai.photocraft.plugins.remove_bg".to_string(),
            icon: Some("wand".to_string()),
            icon_path: None,
            plugin_dir: None,
            sub_actions: vec![sub1, sub2],
            active_sub_action: Some("mask".to_string()),
            name_ru: None,
            locales: Default::default(),
        };

        let app = ToolstripApp::new(vec![btn], ThemeMode::Dark);
        assert_eq!(app.buttons[0].sub_actions.len(), 2);
        assert_eq!(app.buttons[0].active_sub_action.as_deref(), Some("mask"));
    }

    #[test]
    fn test_sub_action_icon_path_resolution() {
        let temp_dir = std::env::temp_dir().join(format!(
            "sub_icon_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&temp_dir);
        let mask_svg = temp_dir.join("mask.svg");
        std::fs::write(&mask_svg, "<svg></svg>").unwrap();

        let sub = core_contracts::SubAction {
            id: "mask".to_string(),
            name: "Layer Mask".to_string(),
            name_ru: None,
            locales: Default::default(),
            icon: Some("mask.svg".to_string()),
            post_commands: vec![],
            script: None,
        };

        let btn = ButtonConfig {
            label: "Remove BG".to_string(),
            action: "ai.photocraft.plugins.remove_bg".to_string(),
            icon: Some("wand".to_string()),
            icon_path: None,
            plugin_dir: Some(temp_dir.clone()),
            sub_actions: vec![sub],
            active_sub_action: Some("mask".to_string()),
            name_ru: None,
            locales: Default::default(),
        };

        let resolved = btn.effective_icon_path();
        assert_eq!(resolved, Some(mask_svg));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
