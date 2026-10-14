pub mod icons;
pub mod manager;
pub mod offline_modal;
pub mod store;
pub mod theme;
pub mod toolstrip;
pub mod updater;
pub mod win32_window;

pub use icons::{get_egui_icon_data, get_icon_rgba, get_tray_icon, ICON_DARK_SVG, ICON_LIGHT_SVG};
pub use manager::{ActiveTab, PluginManagerApp};
pub use offline_modal::OfflineModalApp;
pub use store::{
    StoreCategoryFilter, StoreEvent, StoreLoadStatus, StorePluginEntry, StoreRegistry, StoreState,
};
pub use theme::{ThemeMode, ThemePalette};
pub use toolstrip::ToolstripApp;
pub use updater::{UpdateCheckResult, UpdateState};
pub use win32_window::{calculate_toolstrip_spawn_pos, find_photocraft_window_rect};

use core_contracts::{ButtonConfig, CompanionError, Result};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;
use tracing::info;

pub struct OverlayBar {
    pub buttons: Vec<ButtonConfig>,
}

impl OverlayBar {
    pub fn new(buttons: Vec<ButtonConfig>) -> Self {
        Self { buttons }
    }

    pub fn from_manifests(manifests: &[core_contracts::PluginManifest]) -> Vec<ButtonConfig> {
        let mut buttons = Vec::new();
        for m in manifests {
            if m.enabled {
                let icon = m.icon.clone().or_else(|| match m.plugin_type {
                    core_contracts::PluginType::NativeAi => Some("wand".to_string()),
                    core_contracts::PluginType::NoCode => Some("bw".to_string()),
                    core_contracts::PluginType::NativeAlgorithm => Some("vector".to_string()),
                    _ => None,
                });
                let icon_path = match (&m.plugin_dir, &m.icon) {
                    (Some(dir), Some(icon_file)) => {
                        let path = dir.join(icon_file);
                        if path.exists() {
                            Some(path)
                        } else {
                            let default_svg = dir.join("icon.svg");
                            if default_svg.exists() {
                                Some(default_svg)
                            } else {
                                None
                            }
                        }
                    }
                    (Some(dir), None) => {
                        let default_svg = dir.join("icon.svg");
                        if default_svg.exists() {
                            Some(default_svg)
                        } else {
                            None
                        }
                    }
                    _ => None,
                };
                buttons.push(ButtonConfig {
                    label: m.name.clone(),
                    action: m.id.clone(),
                    icon,
                    icon_path,
                });
            }
        }
        if buttons.is_empty() {
            Self::default_buttons()
        } else {
            buttons
        }
    }

    pub fn default_buttons() -> Vec<ButtonConfig> {
        vec![
            ButtonConfig {
                label: "AI Remove BG".to_string(),
                action: "ai.photocraft.plugins.remove_bg".to_string(),
                icon: Some("wand".to_string()),
                icon_path: None,
            },
            ButtonConfig {
                label: "B&W Film Pop".to_string(),
                action: "macro.photocraft.plugins.black_and_white".to_string(),
                icon: Some("bw".to_string()),
                icon_path: None,
            },
        ]
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(CompanionError::Io)?;
        let buttons: Vec<ButtonConfig> =
            serde_json::from_str(&content).map_err(CompanionError::Json)?;
        info!(
            "Loaded {} overlay buttons from {}",
            buttons.len(),
            path.as_ref().display()
        );
        Ok(Self { buttons })
    }

    pub fn load_or_default(path: impl AsRef<Path>) -> Self {
        if path.as_ref().exists() {
            if let Ok(bar) = Self::load_from_file(path.as_ref()) {
                return bar;
            }
        }
        Self::new(Self::default_buttons())
    }
}

pub struct CompanionOverlayApp {
    pub toolstrip: ToolstripApp,
    pub manager: PluginManagerApp,
    pub offline_modal: OfflineModalApp,
    pub show_manager: bool,
    pub is_hidden_to_tray: bool,
    pub is_connected: Arc<AtomicBool>,
    pub connection_state: Arc<std::sync::RwLock<core_contracts::ConnectionState>>,
    pub action_tx: Option<UnboundedSender<String>>,
    pub last_connected_state: bool,
    pub last_theme: ThemeMode,
    pub tray_icon_handle: Arc<std::sync::Mutex<Option<tray_icon::TrayIcon>>>,
    pub tray_thread_started: bool,
    pub restore_toolbar_flag: Arc<AtomicBool>,
    pub open_manager_flag: Arc<AtomicBool>,
    pub exec_status: Option<core_contracts::SharedExecutionStatus>,
    pub config_path: std::path::PathBuf,
    pub last_saved_vertical: bool,
    pub last_saved_pos: Option<[f32; 2]>,
}

impl CompanionOverlayApp {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        buttons: Vec<ButtonConfig>,
        manifests: Vec<core_contracts::PluginManifest>,
        prefs_path: impl Into<std::path::PathBuf>,
        is_connected: Arc<AtomicBool>,
        action_tx: Option<UnboundedSender<String>>,
        tray_icon_handle: Arc<std::sync::Mutex<Option<tray_icon::TrayIcon>>>,
        exec_status: Option<core_contracts::SharedExecutionStatus>,
        connection_state: Option<Arc<std::sync::RwLock<core_contracts::ConnectionState>>>,
        on_token_saved: Option<Arc<dyn Fn(Option<String>) + Send + Sync>>,
    ) -> Self {
        let prefs_path = prefs_path.into();
        let data_dir = prefs_path.parent().unwrap_or(&prefs_path).to_path_buf();
        let config_path = core_contracts::CompanionConfig::resolve_config_path(&data_dir);
        let cfg = core_contracts::CompanionConfig::load_or_default(&config_path);

        let current_theme = ThemeMode::from_preferences(&prefs_path);
        let current_lang = core_contracts::Language::Auto.resolve(Some(&prefs_path));

        let conn_state_arc = connection_state.unwrap_or_else(|| {
            let initial = if is_connected.load(Ordering::Relaxed) {
                core_contracts::ConnectionState::Connected
            } else {
                core_contracts::ConnectionState::Offline
            };
            Arc::new(std::sync::RwLock::new(initial))
        });

        let toolstrip = ToolstripApp::new(buttons, current_theme).with_orientation(cfg.is_vertical);
        let mut manager = PluginManagerApp::new(manifests, &prefs_path, is_connected.clone())
            .with_connection_state(conn_state_arc.clone());
        if let Some(cb) = on_token_saved {
            manager = manager.with_token_callback(cb);
        }

        let offline_modal = OfflineModalApp::new(current_lang, current_theme);

        let initial_conn = is_connected.load(Ordering::Relaxed);

        Self {
            toolstrip,
            manager,
            offline_modal,
            show_manager: false,
            is_hidden_to_tray: false,
            is_connected,
            connection_state: conn_state_arc,
            action_tx,
            last_connected_state: initial_conn,
            last_theme: current_theme,
            tray_icon_handle,
            tray_thread_started: false,
            restore_toolbar_flag: Arc::new(AtomicBool::new(false)),
            open_manager_flag: Arc::new(AtomicBool::new(false)),
            exec_status,
            config_path,
            last_saved_vertical: cfg.is_vertical,
            last_saved_pos: cfg.window_position,
        }
    }

    pub fn save_layout_state(&self) {
        let mut cfg = core_contracts::CompanionConfig::load_or_default(&self.config_path);
        cfg.is_vertical = self.last_saved_vertical;
        cfg.window_position = self.last_saved_pos;
        if let Err(e) = cfg.save(&self.config_path) {
            tracing::warn!("Failed to save toolstrip layout config: {}", e);
        }
    }

    /// Hides the floating toolbar into the system tray via standard Visible(false).
    ///
    /// In eframe 0.36.2, when the window is hidden (`Visible(false)`), eframe enters its
    /// `is_invisible_or_minimized` state, which drives UI updates every 100ms
    /// (`INVISIBLE_WINDOW_REPAINT_INTERVAL`) without needing Windows GDI / DWM `WM_PAINT` messages.
    /// This completely avoids the deadlock of parking windows off-screen.
    pub fn hide_toolbar_to_tray(&mut self, ctx: &egui::Context) {
        self.is_hidden_to_tray = true;
        self.save_layout_state();
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
    }

    /// Restores the floating toolbar from the system tray back to its active size, saved position, and visibility.
    pub fn restore_toolbar_from_tray(&mut self, ctx: &egui::Context) {
        self.is_hidden_to_tray = false;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));

        let connected = self.is_connected.load(Ordering::Relaxed);
        if connected {
            let size = self.toolstrip.calculate_size();
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            if let Some(pos) = self.last_saved_pos {
                if pos[0].is_finite() && pos[1].is_finite() && pos[0] >= -100.0 && pos[1] >= -100.0
                {
                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                        pos[0], pos[1],
                    )));
                } else if let Some(pos) = calculate_toolstrip_spawn_pos(size.x) {
                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                        pos[0], pos[1],
                    )));
                }
            } else if let Some(pos) = calculate_toolstrip_spawn_pos(size.x) {
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                    pos[0], pos[1],
                )));
            }
        } else {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                280.0, 160.0,
            )));
            if let Some(pos) = calculate_toolstrip_spawn_pos(280.0) {
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                    pos[0], pos[1],
                )));
            }
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    /// Processes atomic flags triggered by the system tray menu or tray icon clicks.
    pub fn process_tray_flags(&mut self, ctx: &egui::Context) {
        if self.restore_toolbar_flag.swap(false, Ordering::Relaxed) {
            self.restore_toolbar_from_tray(ctx);
        }
        if self.open_manager_flag.swap(false, Ordering::Relaxed) {
            // Restore toolbar first so event loop is in active WindowEvent::RedrawRequested context,
            // preventing the eframe `new_events` panic on secondary viewports (BUG-042).
            self.restore_toolbar_from_tray(ctx);
            self.show_manager = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
    }
}

impl eframe::App for CompanionOverlayApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_tray_flags(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Set direct synchronous tray handlers once for instant reaction (0ms, no channel delays)
        if !self.tray_thread_started {
            self.tray_thread_started = true;
            let ctx_tray = ctx.clone();
            let restore_flag = self.restore_toolbar_flag.clone();
            let manager_flag = self.open_manager_flag.clone();

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

        // Keep reactive loop active so IPC connection state is updated
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(50));

        let palette = ThemePalette::for_mode(self.toolstrip.current_theme);
        ui.ctx().set_visuals(palette.to_egui_visuals());

        let connected = self.is_connected.load(Ordering::Relaxed);

        // Adjust window size if connection state changed (only when not parked in tray)
        if connected != self.last_connected_state {
            self.last_connected_state = connected;
            if !self.is_hidden_to_tray {
                if connected {
                    let size = self.toolstrip.calculate_size();
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
                    if let Some(pos) = self.last_saved_pos {
                        if pos[0].is_finite()
                            && pos[1].is_finite()
                            && pos[0] >= -100.0
                            && pos[1] >= -100.0
                        {
                            ui.ctx()
                                .send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                                    egui::pos2(pos[0], pos[1]),
                                ));
                        } else if let Some(pos) = calculate_toolstrip_spawn_pos(size.x) {
                            ui.ctx()
                                .send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                                    egui::pos2(pos[0], pos[1]),
                                ));
                        }
                    } else if let Some(pos) = calculate_toolstrip_spawn_pos(size.x) {
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                                pos[0], pos[1],
                            )));
                    }
                } else {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(
                            280.0, 160.0,
                        )));
                }
            }
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
            if let Some(ref tx) = self.action_tx {
                let _ = tx.send(action);
            }
        }

        // Sync theme when user changes it in manager
        if self.manager.current_theme != self.toolstrip.current_theme {
            self.toolstrip.current_theme = self.manager.current_theme;
        }

        // Dynamically update icons across all locations (Tray, Toolbar, Manager) on theme switch
        let current_theme = self.toolstrip.current_theme;
        if current_theme != self.last_theme {
            self.last_theme = current_theme;
            let is_dark = current_theme.is_dark();

            // 1. Update Tray Icon
            if let Ok(mut guard) = self.tray_icon_handle.lock() {
                if let Some(ti) = guard.as_mut() {
                    let _ = ti.set_icon(get_tray_icon(is_dark, 32));
                }
            }

            // 2. Update Toolbar Viewport Icon
            if let Some(icon) = get_egui_icon_data(is_dark, 32) {
                ctx.send_viewport_cmd(egui::ViewportCommand::Icon(Some(std::sync::Arc::new(icon))));
            }
        }

        // Render main toolbar panel if not hidden to tray
        if !self.is_hidden_to_tray {
            if connected {
                self.toolstrip.render(ui, &palette);

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
                    self.show_manager = true;
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

                    let text_galley = ui.painter().layout_no_wrap(
                        tt_text.clone(),
                        egui::FontId::proportional(11.0),
                        palette.text_primary,
                    );
                    let tt_width = (text_galley.size().x + 18.0).clamp(40.0, 320.0);
                    let tt_height = 24.0;

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
                self.offline_modal.connection_state = *self.connection_state.read().unwrap();
                self.offline_modal.render(ui, &palette);
                if self.offline_modal.open_manager_requested {
                    self.offline_modal.open_manager_requested = false;
                    self.show_manager = true;
                }
            }
        }

        // Render Plugin Manager in a separate window viewport
        if self.show_manager {
            let is_dark = self.toolstrip.current_theme.is_dark();
            let mut vp = egui::ViewportBuilder::default()
                .with_title("PhotoCraft - Plugins Manager")
                .with_inner_size([740.0, 480.0])
                .with_min_inner_size([600.0, 380.0])
                .with_resizable(true)
                .with_taskbar(true);
            if let Some(icon) = get_egui_icon_data(is_dark, 64) {
                vp = vp.with_icon(icon);
            }

            ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("plugin_manager_window"),
                vp,
                |ctx, _class| {
                    if ctx.input(|i| i.viewport().close_requested()) {
                        self.show_manager = false;
                    }
                    if let Some(icon) = get_egui_icon_data(is_dark, 64) {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Icon(Some(
                            std::sync::Arc::new(icon),
                        )));
                    }
                    egui::CentralPanel::default()
                        .frame(egui::Frame::new().fill(palette.bg_fill).inner_margin(0))
                        .show(ctx, |ui| {
                            self.manager.render_settings_window(
                                ui,
                                &palette,
                                &mut self.show_manager,
                            );
                        });
                },
            );
        }
    }
}

/// Helper function to launch the unified floating companion overlay
#[allow(clippy::too_many_arguments)]
pub fn launch_companion_overlay(
    buttons: Vec<ButtonConfig>,
    manifests: Vec<core_contracts::PluginManifest>,
    prefs_path: impl Into<std::path::PathBuf>,
    start_with_manager: bool,
    is_connected: Arc<AtomicBool>,
    action_tx: Option<UnboundedSender<String>>,
    tray_icon_handle: Arc<std::sync::Mutex<Option<tray_icon::TrayIcon>>>,
    exec_status: Option<core_contracts::SharedExecutionStatus>,
    connection_state: Option<Arc<std::sync::RwLock<core_contracts::ConnectionState>>>,
    on_token_saved: Option<Arc<dyn Fn(Option<String>) + Send + Sync>>,
) -> anyhow::Result<()> {
    let mut app = CompanionOverlayApp::new(
        buttons,
        manifests,
        prefs_path,
        is_connected,
        action_tx,
        tray_icon_handle,
        exec_status,
        connection_state,
        on_token_saved,
    );
    if start_with_manager {
        app.show_manager = true;
    }
    let is_dark = app.toolstrip.current_theme.is_dark();
    let connected = app.is_connected.load(Ordering::Relaxed);

    let initial_size = if connected {
        app.toolstrip.calculate_size()
    } else {
        egui::Vec2::new(280.0, 160.0)
    };

    let mut viewport = egui::ViewportBuilder::default()
        .with_decorations(false)
        .with_transparent(true)
        .with_always_on_top()
        .with_resizable(false)
        .with_taskbar(false) // Floating toolbar must NOT create an item on Windows Taskbar!
        .with_inner_size(initial_size)
        .with_title("PhotoCraft Toolbar");

    if connected {
        if let Some(pos) = app.last_saved_pos {
            if pos[0].is_finite() && pos[1].is_finite() && pos[0] >= -100.0 && pos[1] >= -100.0 {
                viewport = viewport.with_position(egui::Pos2::new(pos[0], pos[1]));
            } else if let Some(pos) = calculate_toolstrip_spawn_pos(initial_size.x) {
                viewport = viewport.with_position(egui::Pos2::new(pos[0], pos[1]));
            }
        } else if let Some(pos) = calculate_toolstrip_spawn_pos(initial_size.x) {
            viewport = viewport.with_position(egui::Pos2::new(pos[0], pos[1]));
        }
    }

    if let Some(icon) = get_egui_icon_data(is_dark, 32) {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "PhotoCraft Toolbar",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
    .map_err(|e| anyhow::anyhow!("GUI Error: {e}"))
}

/// Helper function to launch the standalone Plugin Manager window
pub fn launch_plugin_manager(
    manifests: Vec<core_contracts::PluginManifest>,
    prefs_path: impl Into<std::path::PathBuf>,
    connection_state: Option<Arc<std::sync::RwLock<core_contracts::ConnectionState>>>,
    on_token_saved: Option<Arc<dyn Fn(Option<String>) + Send + Sync>>,
) -> anyhow::Result<()> {
    let is_connected = Arc::new(AtomicBool::new(false));
    let mut app = PluginManagerApp::new(manifests, prefs_path, is_connected);
    if let Some(cs) = connection_state {
        app = app.with_connection_state(cs);
    }
    if let Some(cb) = on_token_saved {
        app = app.with_token_callback(cb);
    }
    let is_dark = app.current_theme.is_dark();

    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_inner_size([740.0, 480.0])
        .with_min_inner_size([600.0, 380.0])
        .with_taskbar(true)
        .with_title("PhotoCraft - Plugins Manager");

    if let Some(icon) = get_egui_icon_data(is_dark, 64) {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "PhotoCraft Plugins Manager",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
    .map_err(|e| anyhow::anyhow!("GUI Error: {e}"))
}

#[cfg(test)]
#[allow(clippy::arc_with_non_send_sync)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_bar_default_buttons() {
        let default_btns = OverlayBar::default_buttons();
        assert_eq!(default_btns.len(), 2);
        assert_eq!(default_btns[0].action, "ai.photocraft.plugins.remove_bg");
        assert_eq!(
            default_btns[1].action,
            "macro.photocraft.plugins.black_and_white"
        );
    }

    #[test]
    fn test_overlay_bar_from_manifests() {
        let m1 = core_contracts::PluginManifest {
            id: "p1".to_string(),
            name: "P1".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            author: "".to_string(),
            plugin_type: core_contracts::PluginType::NativeAi,
            icon: None,
            commands: vec![],
            models: vec![],
            action_id: None,
            entrypoint: None,
            ai: None,
            enabled: true,
            plugin_dir: None,
            ..Default::default()
        };
        let m2 = core_contracts::PluginManifest {
            id: "p2".to_string(),
            name: "P2".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            author: "".to_string(),
            plugin_type: core_contracts::PluginType::NoCode,
            icon: None,
            commands: vec![],
            models: vec![],
            action_id: None,
            entrypoint: None,
            ai: None,
            enabled: false,
            plugin_dir: None,
            ..Default::default()
        };

        let btns = OverlayBar::from_manifests(&[m1, m2]);
        assert_eq!(btns.len(), 1);
        assert_eq!(btns[0].action, "p1");
        assert_eq!(btns[0].icon, Some("wand".to_string()));

        // Empty manifests fallback to defaults
        let empty_fallback = OverlayBar::from_manifests(&[]);
        assert_eq!(empty_fallback.len(), 2);
    }

    #[test]
    fn test_overlay_bar_save_and_load() {
        let temp_dir = std::env::temp_dir().join(format!(
            "overlay_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&temp_dir);
        let bar_file = temp_dir.join("buttons.json");

        let sample_buttons = vec![ButtonConfig {
            label: "Custom Action".to_string(),
            action: "custom.action".to_string(),
            icon: Some("custom".to_string()),
            icon_path: None,
        }];
        std::fs::write(&bar_file, serde_json::to_string(&sample_buttons).unwrap()).unwrap();

        let bar = OverlayBar::load_from_file(&bar_file).unwrap();
        assert_eq!(bar.buttons.len(), 1);
        assert_eq!(bar.buttons[0].action, "custom.action");

        let default_bar = OverlayBar::load_or_default(temp_dir.join("non_existent.json"));
        assert_eq!(default_bar.buttons.len(), 2);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_calculate_toolstrip_spawn_pos() {
        let pos = calculate_toolstrip_spawn_pos(120.0);
        assert!(pos.is_some());
        let [x, y] = pos.unwrap();
        assert!(x >= 50.0);
        assert!(y >= 50.0);
    }

    #[test]
    fn test_toolstrip_persistence_config_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!(
            "persist_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&temp_dir);
        let prefs_file = temp_dir.join("preferences.json");
        std::fs::write(&prefs_file, "{}").unwrap();

        let is_connected = Arc::new(AtomicBool::new(false));
        let app = CompanionOverlayApp::new(
            OverlayBar::default_buttons(),
            vec![],
            &prefs_file,
            is_connected.clone(),
            None,
            Arc::new(std::sync::Mutex::new(None)),
            None,
            None,
            None,
        );

        // Initial default state
        assert!(!app.last_saved_vertical);
        assert!(app.last_saved_pos.is_none());
        assert!(!app.toolstrip.is_vertical);

        // Modify and save state
        let mut app_mut = app;
        app_mut.last_saved_vertical = true;
        app_mut.last_saved_pos = Some([420.0, 180.0]);
        app_mut.save_layout_state();

        // Verify saved on disk
        let saved_cfg = core_contracts::CompanionConfig::load_or_default(&app_mut.config_path);
        assert!(saved_cfg.is_vertical);
        assert_eq!(saved_cfg.window_position, Some([420.0, 180.0]));

        // Re-instantiate app from the same directory - state must be restored!
        let app_restored = CompanionOverlayApp::new(
            OverlayBar::default_buttons(),
            vec![],
            &prefs_file,
            is_connected,
            None,
            Arc::new(std::sync::Mutex::new(None)),
            None,
            None,
            None,
        );

        assert!(app_restored.last_saved_vertical);
        assert_eq!(app_restored.last_saved_pos, Some([420.0, 180.0]));
        assert!(app_restored.toolstrip.is_vertical);
        assert_eq!(app_restored.toolstrip.calculate_size().x, 50.0);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_tray_hide_and_open_manager_lifecycle() {
        let temp_dir = std::env::temp_dir().join(format!(
            "tray_lifecycle_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&temp_dir);
        let prefs_file = temp_dir.join("preferences.json");
        std::fs::write(&prefs_file, "{}").unwrap();

        let is_connected = Arc::new(AtomicBool::new(true));
        let mut app = CompanionOverlayApp::new(
            OverlayBar::default_buttons(),
            vec![],
            &prefs_file,
            is_connected,
            None,
            Arc::new(std::sync::Mutex::new(None)),
            None,
            None,
            None,
        );

        let ctx = egui::Context::default();
        app.last_saved_pos = Some([300.0, 200.0]);

        // 1. Hide toolbar to tray: must preserve saved position and set is_hidden_to_tray = true
        app.hide_toolbar_to_tray(&ctx);
        assert!(app.is_hidden_to_tray);
        assert!(!app.show_manager);
        assert_eq!(app.last_saved_pos, Some([300.0, 200.0]));

        // 2. Trigger open_manager_flag while toolbar is hidden to tray:
        // Both manager opens and toolbar is safely restored to prevent eframe secondary viewport panic (BUG-042).
        app.open_manager_flag.store(true, Ordering::Relaxed);
        app.process_tray_flags(&ctx);
        assert!(app.show_manager);
        assert!(!app.is_hidden_to_tray);
        assert!(!app.open_manager_flag.load(Ordering::Relaxed));

        // 3. Re-hide toolbar and trigger restore_toolbar_flag from tray
        app.hide_toolbar_to_tray(&ctx);
        assert!(app.is_hidden_to_tray);
        app.restore_toolbar_flag.store(true, Ordering::Relaxed);
        app.process_tray_flags(&ctx);
        assert!(!app.is_hidden_to_tray);
        assert!(!app.restore_toolbar_flag.load(Ordering::Relaxed));
        assert_eq!(app.last_saved_pos, Some([300.0, 200.0]));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
