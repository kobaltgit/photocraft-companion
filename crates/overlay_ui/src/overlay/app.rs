use super::*;
use crate::icons::{get_egui_icon_data, get_system_window_icon_size};
use eframe::egui;

impl eframe::App for CompanionOverlayApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_tray_flags(ctx);
        if self.tray_restore_cooldown_frames > 0 {
            self.tray_restore_cooldown_frames -= 1;
            ctx.request_repaint();
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Set direct synchronous tray handlers once for instant reaction (0ms, no channel delays)
        if !self.tray_thread_started {
            self.tray_thread_started = true;
            let ctx_tray = ctx.clone();
            let restore_flag = self.restore_toolbar_flag.clone();
            let manager_flag = self.open_manager_flag.clone();
            let reset_flag = self.reset_layout_flag.clone();

            tray_icon::menu::MenuEvent::set_event_handler(Some(
                move |event: tray_icon::menu::MenuEvent| match event.id.as_ref() {
                    "tray_exit" => {
                        std::process::exit(0);
                    }
                    "tray_toolbar" => {
                        restore_flag.store(true, Ordering::Relaxed);
                        ctx_tray.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        ctx_tray.send_viewport_cmd(egui::ViewportCommand::Focus);
                        ctx_tray.request_repaint();
                    }
                    "tray_manager" => {
                        manager_flag.store(true, Ordering::Relaxed);
                        ctx_tray.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        ctx_tray.send_viewport_cmd(egui::ViewportCommand::Focus);
                        ctx_tray.request_repaint();
                    }
                    "tray_reset_layout" => {
                        reset_flag.store(true, Ordering::Relaxed);
                        ctx_tray.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        ctx_tray.send_viewport_cmd(egui::ViewportCommand::Focus);
                        ctx_tray.request_repaint();
                    }
                    _ => {}
                },
            ));

            let ctx_icon = ctx.clone();
            let restore_flag_icon = self.restore_toolbar_flag.clone();
            tray_icon::TrayIconEvent::set_event_handler(Some(
                move |event: tray_icon::TrayIconEvent| {
                    if let tray_icon::TrayIconEvent::Click {
                        button: tray_icon::MouseButton::Left,
                        button_state: tray_icon::MouseButtonState::Up,
                        ..
                    } = event
                    {
                        restore_flag_icon.store(true, Ordering::Relaxed);
                        ctx_icon.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                        ctx_icon.send_viewport_cmd(egui::ViewportCommand::Focus);
                        ctx_icon.request_repaint();
                    }
                },
            ));
        }

        // Process asynchronous tray commands instantly
        self.process_tray_flags(&ctx);

        // Heartbeat repaint timer for background state & connection updates (restoring v2.0.3 responsiveness)
        let repaint_interval = self.calculate_repaint_interval();
        ui.ctx().request_repaint_after(repaint_interval);

        let palette = ThemePalette::for_mode(self.toolstrip.current_theme);
        ui.ctx().set_visuals(palette.to_egui_visuals());

        let connected = self.is_connected.load(Ordering::Relaxed);

        // Adjust window size if connection state changed (only when not parked in tray)
        if connected != self.last_connected_state {
            self.last_connected_state = connected;
            if !self.is_hidden_to_tray {
                if connected {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                            egui::WindowLevel::AlwaysOnTop,
                        ));
                    let size = self.toolstrip.calculate_size();
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
                    let target_pos = if let Some(pos) = self.last_saved_pos {
                        if crate::win32_window::is_position_visible_on_screen(pos, [size.x, size.y])
                        {
                            Some(pos)
                        } else {
                            Some(crate::win32_window::calculate_primary_screen_center(
                                size.x, size.y,
                            ))
                        }
                    } else {
                        Some(crate::win32_window::calculate_primary_screen_center(
                            size.x, size.y,
                        ))
                    };
                    if let Some(pos) = target_pos {
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                                pos[0], pos[1],
                            )));
                    }
                } else {
                    let level = if self.offline_modal.is_pinned {
                        egui::WindowLevel::AlwaysOnTop
                    } else {
                        egui::WindowLevel::Normal
                    };
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
                    let size = egui::Vec2::new(300.0, 280.0);
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
                    let pos = if let Some(p) = self.last_saved_offline_pos {
                        if crate::win32_window::is_position_visible_on_screen(p, [size.x, size.y]) {
                            p
                        } else {
                            crate::win32_window::calculate_primary_screen_center(size.x, size.y)
                        }
                    } else {
                        crate::win32_window::calculate_primary_screen_center(size.x, size.y)
                    };
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                            pos[0], pos[1],
                        )));
                }
            }
        }

        // Dynamic language sync across toolstrip, offline_modal, and dynamic engine (TTL 2000ms cache)
        if self.manager.current_language == core_contracts::Language::Auto {
            let now = std::time::Instant::now();
            let should_resolve = self
                .last_lang_resolve_time
                .map(|t| now.duration_since(t).as_millis() >= 2000)
                .unwrap_or(true);

            if should_resolve {
                self.last_lang_resolve_time = Some(now);
                let prefs_for_resolve = if !self.manager.photocraft_path.trim().is_empty() {
                    let p = std::path::Path::new(self.manager.photocraft_path.trim());
                    let p_exe = p
                        .parent()
                        .map(|dir| dir.join("PhotoCraftData").join("preferences.json"));
                    p_exe.unwrap_or_else(|| self.manager.prefs_path.clone())
                } else {
                    self.manager.prefs_path.clone()
                };
                let detected = core_contracts::Language::Auto.resolve(Some(&prefs_for_resolve));
                if detected != self.manager.resolved_auto_language {
                    self.manager.resolved_auto_language = detected;
                }
            }
        }
        let eff_lang = self.manager.effective_language();
        if self.toolstrip.language != eff_lang {
            self.toolstrip.language = eff_lang;
            if let Ok(mut guard) = self.tray_icon_handle.lock() {
                if let Some(tray) = guard.as_mut() {
                    let new_menu = create_tray_menu(eff_lang);
                    tray.set_menu(Some(Box::new(new_menu)));
                }
            }
        }
        if self.offline_modal.resolved_auto_language != self.manager.resolved_auto_language {
            self.offline_modal.resolved_auto_language = self.manager.resolved_auto_language;
        }
        if self.offline_modal.current_language != self.manager.current_language {
            self.offline_modal.current_language = self.manager.current_language;
        }

        // Reactive sync: When plugins are toggled in Manager, immediately update toolstrip buttons & resize
        let current_enabled: Vec<String> = self
            .manager
            .manifests
            .iter()
            .filter(|m| m.enabled)
            .map(|m| m.id.clone())
            .collect();
        let toolstrip_actions: Vec<String> = self
            .toolstrip
            .buttons
            .iter()
            .map(|b| b.action.clone())
            .collect();
        if current_enabled != toolstrip_actions {
            self.toolstrip.buttons = OverlayBar::from_manifests(&self.manager.manifests);
            if connected && !self.is_hidden_to_tray {
                let size = self.toolstrip.calculate_size();
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            }
        }

        // Process any pending button clicks from toolstrip
        if let Some(action) = self.toolstrip.pending_action.take() {
            let handled_by_dynamic = if let Ok(mut engine) = self.dynamic_engine.lock() {
                if engine.has_plugin(&action) {
                    engine.toggle_window(&action);
                    true
                } else {
                    false
                }
            } else {
                false
            };

            if !handled_by_dynamic {
                if let Some(ref tx) = self.action_tx {
                    let _ = tx.send(action);
                }
            }
        }

        // Sync theme when user changes it in manager
        if self.manager.current_theme != self.toolstrip.current_theme {
            self.toolstrip.current_theme = self.manager.current_theme;
        }

        // Track theme change for dynamic widget palette and color styling
        let current_theme = self.toolstrip.current_theme;
        if current_theme != self.last_theme {
            self.last_theme = current_theme;
        }

        // Render main toolbar panel if not hidden to tray
        if !self.is_hidden_to_tray {
            // Render dynamic interactive plugin windows (HueWheel, Sliders, Banners, etc.)
            // Only render secondary viewports when tray restore cooldown reaches 0 to avoid eframe event loop panic
            if self.tray_restore_cooldown_frames == 0 {
                let mut open_mgr = false;
                if let Ok(mut engine) = self.dynamic_engine.lock() {
                    let effective_lang = self.manager.effective_language();
                    if engine.current_language() != effective_lang {
                        engine.set_language(effective_lang);
                    }
                    engine.render(&ctx, &palette);
                    self.toolstrip.dynamic_badges = engine.get_active_badges();
                    if engine.open_manager_requested {
                        engine.open_manager_requested = false;
                        open_mgr = true;
                    }
                }
                if open_mgr {
                    self.open_manager();
                }
            }

            if connected {
                let has_sel = if let Ok(engine) = self.dynamic_engine.lock() {
                    engine.has_selection
                } else {
                    false
                };
                self.toolstrip.has_selection = has_sel;
                self.toolstrip.render(ui, &palette);
                self.toolstrip.render_sub_actions_flyout(&ctx, &palette);

                // Auto-save orientation changes
                if self.toolstrip.orientation_changed
                    || self.toolstrip.is_vertical != self.last_saved_vertical
                {
                    self.toolstrip.orientation_changed = false;
                    self.last_saved_vertical = self.toolstrip.is_vertical;
                    self.save_layout_state();
                }

                // Auto-save window position when mouse drag ends
                let pointer_down = ui.ctx().input(|i| i.pointer.any_down());
                if !pointer_down {
                    if let Some(outer_rect) = ui.ctx().input(|i| i.viewport().outer_rect) {
                        let min = outer_rect.min;
                        if min.x.is_finite()
                            && min.y.is_finite()
                            && min.x >= -100.0
                            && min.y >= -100.0
                            && (min.x.abs() > 0.001 || min.y.abs() > 0.001)
                        {
                            let cur_pos = [min.x, min.y];
                            let needs_save = match self.last_saved_pos {
                                Some(last) => {
                                    (last[0] - cur_pos[0]).abs() > 1.0
                                        || (last[1] - cur_pos[1]).abs() > 1.0
                                }
                                None => true,
                            };
                            if needs_save {
                                self.last_saved_pos = Some(cur_pos);
                                self.save_layout_state();
                            }
                        }
                    }
                }

                if self.toolstrip.open_manager_requested {
                    self.toolstrip.open_manager_requested = false;
                    self.open_manager();
                }
                if self.toolstrip.hide_requested {
                    self.toolstrip.hide_requested = false;
                    self.hide_toolbar_to_tray(&ctx);
                }

                // BUG-023: Render external floating tooltip popup viewport if any tool button is hovered
                if let Some((tt_text, btn_rect)) = self.toolstrip.hovered_tooltip.take() {
                    let window_min = ui
                        .ctx()
                        .input(|i| i.viewport().outer_rect)
                        .map(|r| r.min)
                        .unwrap_or(egui::pos2(0.0, 0.0));

                    let tt_pos = if self.toolstrip.is_vertical {
                        window_min + egui::vec2(btn_rect.right() + 6.0, btn_rect.top() + 4.0)
                    } else {
                        window_min + egui::vec2(btn_rect.left(), btn_rect.bottom() + 6.0)
                    };

                    let font_id = egui::FontId::proportional(11.0);
                    let galley =
                        ui.painter()
                            .layout(tt_text.clone(), font_id, palette.text_primary, 480.0);
                    let galley_size = galley.size();
                    let tt_width = (galley_size.x + 20.0).clamp(40.0, 520.0);
                    let tt_height = (galley_size.y + 10.0).max(24.0);

                    let vp = egui::ViewportBuilder::default()
                        .with_decorations(false)
                        .with_transparent(true)
                        .with_always_on_top()
                        .with_resizable(false)
                        .with_taskbar(false)
                        .with_position(tt_pos)
                        .with_inner_size(egui::vec2(tt_width, tt_height));

                    ui.ctx().show_viewport_immediate(
                        egui::ViewportId::from_hash_of("toolstrip_hover_tooltip_popup"),
                        vp,
                        |ctx, _class| {
                            egui::CentralPanel::default()
                                .frame(
                                    egui::Frame::new()
                                        .fill(palette.card_inner)
                                        .stroke(egui::Stroke::new(1.0, palette.border_stroke))
                                        .corner_radius(4.0)
                                        .inner_margin(egui::Margin::symmetric(8, 3)),
                                )
                                .show(ctx, |ui| {
                                    ui.label(
                                        egui::RichText::new(&tt_text)
                                            .size(11.0)
                                            .color(palette.text_primary),
                                    );
                                });
                        },
                    );
                }

                // Render Execution Status Mini-HUD / Toast Viewport
                if let Some(ref status_arc) = self.exec_status {
                    let mut reset_to_idle = false;
                    let hud_data = {
                        let guard = status_arc.lock().unwrap();
                        match &*guard {
                            core_contracts::ExecutionState::Idle => None,
                            core_contracts::ExecutionState::Running {
                                action_id,
                                action_name,
                                started_at,
                            } => {
                                let elapsed = started_at.elapsed().as_secs_f32();
                                Some((
                                    format!(
                                        "{}: {} ({:.1}s)",
                                        action_name,
                                        self.manager.t("Processing...", "Обработка..."),
                                        elapsed
                                    ),
                                    palette.accent_blue,
                                    Some(action_id.clone()),
                                    true,
                                ))
                            }
                            core_contracts::ExecutionState::Success {
                                action_name,
                                message: _,
                                duration_secs,
                                finished_at,
                            } => {
                                if finished_at.elapsed() > std::time::Duration::from_secs(3) {
                                    reset_to_idle = true;
                                    None
                                } else {
                                    Some((
                                        format!(
                                            "{}: {} ({:.1}s)",
                                            action_name,
                                            self.manager.t("Done", "Готово"),
                                            duration_secs
                                        ),
                                        egui::Color32::from_rgb(0x28, 0xa7, 0x45),
                                        None,
                                        false,
                                    ))
                                }
                            }
                            core_contracts::ExecutionState::Error {
                                action_name,
                                error,
                                finished_at,
                            } => {
                                if finished_at.elapsed() > std::time::Duration::from_secs(4) {
                                    reset_to_idle = true;
                                    None
                                } else {
                                    let err_preview = if error.len() > 35 {
                                        &error[..35]
                                    } else {
                                        error
                                    };
                                    Some((
                                        format!("{}: {}", action_name, err_preview),
                                        egui::Color32::from_rgb(0xe5, 0x39, 0x35),
                                        None,
                                        false,
                                    ))
                                }
                            }
                        }
                    };

                    if reset_to_idle {
                        *status_arc.lock().unwrap() = core_contracts::ExecutionState::Idle;
                    }

                    if let Some((hud_text, hud_color, active_action, is_running)) = hud_data {
                        self.toolstrip.is_busy = is_running;
                        self.toolstrip.active_action = active_action;

                        if is_running {
                            ui.ctx()
                                .request_repaint_after(std::time::Duration::from_millis(80));
                        }

                        let window_rect = ui
                            .ctx()
                            .input(|i| i.viewport().outer_rect)
                            .unwrap_or(egui::Rect::ZERO);

                        let hud_pos = if self.toolstrip.is_vertical {
                            window_rect.min + egui::vec2(window_rect.width() + 6.0, 4.0)
                        } else {
                            window_rect.min + egui::vec2(0.0, window_rect.height() + 6.0)
                        };

                        let hud_galley = ui.painter().layout_no_wrap(
                            hud_text.clone(),
                            egui::FontId::proportional(11.0),
                            palette.text_primary,
                        );
                        let hud_width = (hud_galley.size().x + 32.0).clamp(80.0, 360.0);
                        let hud_height = 26.0;

                        let vp = egui::ViewportBuilder::default()
                            .with_decorations(false)
                            .with_transparent(true)
                            .with_always_on_top()
                            .with_resizable(false)
                            .with_taskbar(false)
                            .with_position(hud_pos)
                            .with_inner_size(egui::vec2(hud_width, hud_height));

                        ui.ctx().show_viewport_immediate(
                            egui::ViewportId::from_hash_of("toolstrip_execution_hud_toast"),
                            vp,
                            |ctx, _class| {
                                egui::CentralPanel::default()
                                    .frame(
                                        egui::Frame::new()
                                            .fill(palette.card_inner)
                                            .stroke(egui::Stroke::new(1.0, hud_color))
                                            .corner_radius(4.0)
                                            .inner_margin(egui::Margin::symmetric(10, 4)),
                                    )
                                    .show(ctx, |ui| {
                                        ui.horizontal(|ui| {
                                            let (circle_rect, _) = ui.allocate_exact_size(
                                                egui::vec2(8.0, 12.0),
                                                egui::Sense::hover(),
                                            );
                                            ui.painter().circle_filled(
                                                circle_rect.center(),
                                                3.0,
                                                hud_color,
                                            );
                                            ui.label(
                                                egui::RichText::new(&hud_text)
                                                    .size(11.0)
                                                    .strong()
                                                    .color(hud_color),
                                            );
                                        });
                                    });
                            },
                        );
                    } else {
                        self.toolstrip.is_busy = false;
                        self.toolstrip.active_action = None;
                    }
                }
            } else {
                self.offline_modal.is_process_detected = self
                    .is_process_running
                    .as_ref()
                    .map(|f| f())
                    .unwrap_or_else(|| {
                        crate::win32_window::find_photocraft_window_rect().is_some()
                    });
                self.offline_modal.connection_state = *self.connection_state.read().unwrap();
                self.offline_modal.render(ui, &palette);

                // Auto-save offline window position when drag ends
                let pointer_down = ui.ctx().input(|i| i.pointer.any_down());
                if !pointer_down {
                    if let Some(outer_rect) = ui.ctx().input(|i| i.viewport().outer_rect) {
                        let min = outer_rect.min;
                        if min.x.is_finite()
                            && min.y.is_finite()
                            && min.x >= -100.0
                            && min.y >= -100.0
                            && (min.x.abs() > 0.001 || min.y.abs() > 0.001)
                        {
                            let cur_pos = [min.x, min.y];
                            let needs_save = match self.last_saved_offline_pos {
                                Some(last) => {
                                    (last[0] - cur_pos[0]).abs() > 1.0
                                        || (last[1] - cur_pos[1]).abs() > 1.0
                                }
                                None => true,
                            };
                            if needs_save {
                                self.last_saved_offline_pos = Some(cur_pos);
                                self.save_layout_state();
                            }
                        }
                    }
                }

                if self.offline_modal.hide_requested {
                    self.offline_modal.hide_requested = false;
                    self.hide_toolbar_to_tray(&ctx);
                }
                if self.offline_modal.open_manager_requested {
                    self.offline_modal.open_manager_requested = false;
                    self.open_manager();
                }
                if self.offline_modal.launch_requested {
                    self.offline_modal.launch_requested = false;
                    if let Some(ref cb) = self.on_launch_requested {
                        cb();
                    }
                }
            }
        }

        // Render Plugin Manager in a separate window viewport
        // Secondary viewports are rendered only when tray restore cooldown has settled (BUG-109)
        if self.show_manager && self.tray_restore_cooldown_frames == 0 {
            let default_size = [740.0, 480.0];
            let manager_size = self.last_saved_manager_size.unwrap_or(default_size);

            let manager_pos = if let Some(pos) = self.last_saved_manager_pos {
                if crate::win32_window::is_position_visible_on_screen(pos, manager_size) {
                    pos
                } else {
                    crate::win32_window::calculate_primary_screen_center(
                        manager_size[0],
                        manager_size[1],
                    )
                }
            } else {
                crate::win32_window::calculate_primary_screen_center(
                    manager_size[0],
                    manager_size[1],
                )
            };

            let mut vp = egui::ViewportBuilder::default()
                .with_title("PhotoCraft - Plugins Manager")
                .with_inner_size(manager_size)
                .with_min_inner_size([600.0, 380.0])
                .with_resizable(true)
                .with_taskbar(true);

            if self.manager_just_opened {
                vp = vp.with_position(egui::pos2(manager_pos[0], manager_pos[1]));
            }

            if let Some(icon) = get_egui_icon_data(get_system_window_icon_size()) {
                vp = vp.with_icon(icon);
            }

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("plugin_manager_window"),
                vp,
                |ctx, _class| {
                    if !self.manager_icons_calibrated
                        && crate::win32_window::apply_crisp_window_icons_by_title(
                            "PhotoCraft - Plugins Manager",
                        )
                    {
                        self.manager_icons_calibrated = true;
                    }

                    if self.manager_just_opened {
                        self.manager_just_opened = false;
                    }

                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_manager = false;
                        self.save_layout_state();
                    }

                    // Dynamically capture position & client inner size when user finishes interaction
                    let pointer_down = ctx.input(|i| i.pointer.any_down());
                    if !pointer_down {
                        let outer_pos =
                            ctx.input(|i| i.viewport().outer_rect.map(|r| [r.min.x, r.min.y]));
                        let inner_sz =
                            ctx.input(|i| i.viewport().inner_rect.map(|r| [r.width(), r.height()]));

                        if let (Some(pos), Some(size)) = (outer_pos, inner_sz) {
                            if pos[0].is_finite()
                                && pos[1].is_finite()
                                && size[0] >= 400.0
                                && size[1] >= 300.0
                                && (pos[0].abs() > 0.001 || pos[1].abs() > 0.001)
                            {
                                let pos_changed = match self.last_saved_manager_pos {
                                    Some(last) => {
                                        (last[0] - pos[0]).abs() > 1.0
                                            || (last[1] - pos[1]).abs() > 1.0
                                    }
                                    None => true,
                                };
                                let size_changed = match self.last_saved_manager_size {
                                    Some(last) => {
                                        (last[0] - size[0]).abs() > 1.0
                                            || (last[1] - size[1]).abs() > 1.0
                                    }
                                    None => true,
                                };

                                if pos_changed || size_changed {
                                    self.last_saved_manager_pos = Some(pos);
                                    self.last_saved_manager_size = Some(size);
                                    self.save_layout_state();
                                }
                            }
                        }
                    }

                    egui::CentralPanel::default()
                        .frame(egui::Frame::new().fill(palette.bg_fill).inner_margin(0))
                        .show(ctx, |ui| {
                            self.manager.render_settings_window(
                                ui,
                                &palette,
                                &mut self.show_manager,
                            );
                            if self.manager.launch_requested {
                                self.manager.launch_requested = false;
                                if let Some(ref cb) = self.on_launch_requested {
                                    cb();
                                }
                            }
                        });
                },
            );
        }
    }
}
