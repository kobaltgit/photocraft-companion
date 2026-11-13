mod app;
mod bar;
mod launchers;
mod tray;

#[cfg(test)]
mod tests;

pub use bar::OverlayBar;
pub use launchers::{launch_companion_overlay, launch_plugin_manager};
pub use tray::{create_tray_menu, tray_t};

use crate::dynamic_tool::DynamicToolEngine;
use crate::manager::PluginManagerApp;
use crate::offline_modal::OfflineModalApp;
use crate::theme::{ThemeMode, ThemePalette};
use crate::toolstrip::ToolstripApp;
use core_contracts::ButtonConfig;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedSender;

pub struct CompanionOverlayApp {
    pub toolstrip: ToolstripApp,
    pub manager: PluginManagerApp,
    pub offline_modal: OfflineModalApp,
    pub show_manager: bool,
    pub is_hidden_to_tray: bool,
    pub tray_restore_cooldown_frames: u8,
    pub is_connected: Arc<AtomicBool>,
    pub connection_state: Arc<std::sync::RwLock<core_contracts::ConnectionState>>,
    pub action_tx: Option<UnboundedSender<String>>,
    pub last_connected_state: bool,
    pub last_theme: ThemeMode,
    pub tray_icon_handle: Arc<std::sync::Mutex<Option<tray_icon::TrayIcon>>>,
    pub tray_thread_started: bool,
    pub restore_toolbar_flag: Arc<AtomicBool>,
    pub open_manager_flag: Arc<AtomicBool>,
    pub reset_layout_flag: Arc<AtomicBool>,
    pub exec_status: Option<core_contracts::SharedExecutionStatus>,
    pub config_path: std::path::PathBuf,
    pub last_saved_vertical: bool,
    pub last_saved_pos: Option<[f32; 2]>,
    pub last_saved_offline_pos: Option<[f32; 2]>,
    pub last_saved_manager_pos: Option<[f32; 2]>,
    pub last_saved_manager_size: Option<[f32; 2]>,
    pub manager_just_opened: bool,
    pub manager_icons_calibrated: bool,
    pub on_launch_requested: Option<Arc<dyn Fn() + Send + Sync>>,
    pub dynamic_engine: Arc<std::sync::Mutex<DynamicToolEngine>>,
    pub is_process_running: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
    pub last_lang_resolve_time: Option<std::time::Instant>,
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
        _ipc_action_tx: Option<UnboundedSender<core_contracts::PluginIpcAction>>,
        dynamic_engine: Arc<std::sync::Mutex<DynamicToolEngine>>,
    ) -> Self {
        let prefs_path = prefs_path.into();
        let data_dir = prefs_path.parent().unwrap_or(&prefs_path).to_path_buf();
        let config_path = core_contracts::CompanionConfig::resolve_config_path(&data_dir);
        let cfg = core_contracts::CompanionConfig::load_or_default(&config_path);

        let current_theme = ThemeMode::from_preferences(&prefs_path);
        let prefs_for_resolve = if let Some(ref p) = cfg.photocraft_path {
            let p_exe = p
                .parent()
                .map(|dir| dir.join("PhotoCraftData").join("preferences.json"));
            p_exe.unwrap_or_else(|| prefs_path.clone())
        } else {
            prefs_path.clone()
        };
        let cfg_lang = core_contracts::Language::from_str_code(&cfg.language);
        let resolved_auto = core_contracts::Language::Auto.resolve(Some(&prefs_for_resolve));
        let effective_lang = if cfg_lang == core_contracts::Language::Auto {
            resolved_auto
        } else {
            cfg_lang
        };

        let conn_state_arc = connection_state.unwrap_or_else(|| {
            let initial = if is_connected.load(Ordering::Relaxed) {
                core_contracts::ConnectionState::Connected
            } else {
                core_contracts::ConnectionState::Offline
            };
            Arc::new(std::sync::RwLock::new(initial))
        });

        let mut toolstrip = ToolstripApp::new(buttons, current_theme)
            .with_orientation(cfg.is_vertical)
            .with_language(effective_lang);

        for m in &manifests {
            if let Some(ref ui) = m.interactive_ui {
                if let Some(ref badge) = ui.dynamic_badge {
                    toolstrip.set_dynamic_badge(&m.id, Some(badge.clone()));
                }
            }
        }

        for btn in &mut toolstrip.buttons {
            if let Some(sub_id) = cfg.active_sub_actions.get(&btn.action) {
                if btn.sub_actions.iter().any(|s| &s.id == sub_id) {
                    btn.active_sub_action = Some(sub_id.clone());
                }
            }
        }
        let mut manager =
            PluginManagerApp::new(manifests.clone(), &prefs_path, is_connected.clone())
                .with_connection_state(conn_state_arc.clone());
        manager.config_path = config_path.clone();
        manager.current_language = cfg_lang;
        manager.resolved_auto_language = resolved_auto;
        if manager.photocraft_path.is_empty() {
            if let Some(ref p) = cfg.photocraft_path {
                manager.photocraft_path = p.to_string_lossy().to_string();
            }
        }
        if let Some(cb) = on_token_saved {
            manager = manager.with_token_callback(cb);
        }

        let mut offline_modal = OfflineModalApp::new(cfg_lang, current_theme);
        offline_modal.resolved_auto_language = resolved_auto;

        let initial_conn = is_connected.load(Ordering::Relaxed);

        if let Ok(mut engine) = dynamic_engine.lock() {
            engine.set_config_path(config_path.clone());
        }

        Self {
            toolstrip,
            manager,
            offline_modal,
            show_manager: false,
            is_hidden_to_tray: false,
            tray_restore_cooldown_frames: 0,
            is_connected,
            connection_state: conn_state_arc,
            action_tx,
            last_connected_state: initial_conn,
            last_theme: current_theme,
            tray_icon_handle,
            tray_thread_started: false,
            restore_toolbar_flag: Arc::new(AtomicBool::new(false)),
            open_manager_flag: Arc::new(AtomicBool::new(false)),
            reset_layout_flag: Arc::new(AtomicBool::new(false)),
            exec_status,
            config_path,
            last_saved_vertical: cfg.is_vertical,
            last_saved_pos: cfg.window_position,
            last_saved_offline_pos: cfg.offline_window_position,
            last_saved_manager_pos: cfg.manager_window_position,
            last_saved_manager_size: cfg.manager_window_size,
            manager_just_opened: true,
            manager_icons_calibrated: false,
            on_launch_requested: None,
            dynamic_engine,
            is_process_running: None,
            last_lang_resolve_time: None,
        }
    }

    pub fn with_launch_callback(mut self, cb: Arc<dyn Fn() + Send + Sync>) -> Self {
        self.on_launch_requested = Some(cb);
        self
    }

    pub fn with_process_detector(mut self, cb: Arc<dyn Fn() -> bool + Send + Sync>) -> Self {
        self.is_process_running = Some(cb.clone());
        self.manager.is_process_running = Some(cb);
        self
    }

    pub fn set_dynamic_badge(
        &mut self,
        action_id: &str,
        badge: Option<core_contracts::DynamicBadgeState>,
    ) {
        self.toolstrip.set_dynamic_badge(action_id, badge);
    }

    pub fn with_plugins_dir(mut self, plugins_dir: impl Into<std::path::PathBuf>) -> Self {
        self.manager = self.manager.with_plugins_dir(plugins_dir);
        self
    }

    pub fn save_layout_state(&self) {
        let mut cfg = core_contracts::CompanionConfig::load_or_default(&self.config_path);
        cfg.is_vertical = self.last_saved_vertical;
        cfg.window_position = self.last_saved_pos;
        cfg.offline_window_position = self.last_saved_offline_pos;
        cfg.manager_window_position = self.last_saved_manager_pos;
        cfg.manager_window_size = self.last_saved_manager_size;
        if let Err(e) = cfg.save(&self.config_path) {
            tracing::warn!("Failed to save toolstrip layout config: {}", e);
        }
    }

    pub fn reset_layout(&mut self, ctx: Option<&egui::Context>) {
        self.last_saved_pos = None;
        self.last_saved_offline_pos = None;
        self.last_saved_manager_pos = None;
        self.last_saved_manager_size = None;
        self.toolstrip.is_vertical = false;
        self.last_saved_vertical = false;
        if let Ok(mut engine) = self.dynamic_engine.lock() {
            engine.clear_plugin_geometries();
        }
        let mut cfg = core_contracts::CompanionConfig::load_or_default(&self.config_path);
        cfg.window_position = None;
        cfg.offline_window_position = None;
        cfg.manager_window_position = None;
        cfg.manager_window_size = None;
        cfg.is_vertical = false;
        cfg.clear_plugin_geometries();
        if let Err(e) = cfg.save(&self.config_path) {
            tracing::warn!("Failed to save reset layout config: {}", e);
        }
        if let Some(ctx) = ctx {
            let initial_pos = crate::win32_window::calculate_toolstrip_spawn_pos(
                self.toolstrip.calculate_size().x,
            );
            if let Some(pos) = initial_pos {
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                    pos[0], pos[1],
                )));
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
                self.toolstrip.calculate_size(),
            ));
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
        self.tray_restore_cooldown_frames = 2;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        ctx.request_repaint();

        let connected = self.is_connected.load(Ordering::Relaxed);
        if connected {
            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                egui::WindowLevel::AlwaysOnTop,
            ));
            let size = self.toolstrip.calculate_size();
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            let target_pos = if let Some(pos) = self.last_saved_pos {
                if crate::win32_window::is_position_visible_on_screen(pos, [size.x, size.y]) {
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
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                    pos[0], pos[1],
                )));
            }
        } else {
            let level = if self.offline_modal.is_pinned {
                egui::WindowLevel::AlwaysOnTop
            } else {
                egui::WindowLevel::Normal
            };
            ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
            let size = egui::Vec2::new(300.0, 280.0);
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            let pos = if let Some(p) = self.last_saved_offline_pos {
                if crate::win32_window::is_position_visible_on_screen(p, [size.x, size.y]) {
                    p
                } else {
                    crate::win32_window::calculate_primary_screen_center(size.x, size.y)
                }
            } else {
                crate::win32_window::calculate_primary_screen_center(size.x, size.y)
            };
            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                pos[0], pos[1],
            )));
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
    }

    /// Processes atomic flags triggered by the system tray menu or tray icon clicks.
    pub fn open_manager(&mut self) {
        self.show_manager = true;
        self.manager_just_opened = true;
        self.manager_icons_calibrated = false;
    }

    pub fn process_tray_flags(&mut self, ctx: &egui::Context) {
        if self.restore_toolbar_flag.swap(false, Ordering::Relaxed) {
            self.restore_toolbar_from_tray(ctx);
        }
        if self.open_manager_flag.swap(false, Ordering::Relaxed) {
            // Restore toolbar first so event loop is in active WindowEvent::RedrawRequested context,
            // preventing the eframe `new_events` panic on secondary viewports (BUG-042).
            self.restore_toolbar_from_tray(ctx);
            self.open_manager();
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
        if self.reset_layout_flag.swap(false, Ordering::Relaxed) {
            self.restore_toolbar_from_tray(ctx);
            self.reset_layout(Some(ctx));
            ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
        }
    }

    /// Calculates the optimal heartbeat repaint interval based on the current UI state.
    ///
    /// Heartbeat repaint timer for background state & connection updates.
    /// - Active visible mode (connected): 50 ms (20 FPS) pure v2.0.3 benchmark ensuring instantaneous hover, click, and animation responsiveness.
    /// - Offline modal mode: 250 ms calm pacing preventing GPU / DWM message pump flooding.
    /// - Hidden in system tray: 500 ms (2 FPS) low-power mode with 0.0% CPU when parked.
    pub fn calculate_repaint_interval(&self) -> std::time::Duration {
        if self.is_hidden_to_tray {
            std::time::Duration::from_millis(500)
        } else if !self.is_connected.load(Ordering::Relaxed) {
            std::time::Duration::from_millis(250)
        } else {
            std::time::Duration::from_millis(50)
        }
    }
}
