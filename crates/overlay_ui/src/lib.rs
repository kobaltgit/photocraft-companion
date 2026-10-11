pub mod icons;
pub mod manager;
pub mod offline_modal;
pub mod theme;
pub mod toolstrip;
pub mod win32_window;

pub use icons::{get_egui_icon_data, get_icon_rgba, get_tray_icon, ICON_DARK_SVG, ICON_LIGHT_SVG};
pub use manager::{ActiveTab, PluginManagerApp};
pub use offline_modal::OfflineModalApp;
pub use theme::{ThemeMode, ThemePalette};
pub use toolstrip::ToolstripApp;
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
                let icon = match m.plugin_type {
                    core_contracts::PluginType::NativeAi => Some("wand".to_string()),
                    core_contracts::PluginType::NoCode => Some("bw".to_string()),
                    _ => None,
                };
                buttons.push(ButtonConfig {
                    label: m.name.clone(),
                    action: m.id.clone(),
                    hotkey: m.default_hotkey.clone(),
                    icon,
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
                hotkey: Some("Ctrl+Shift+B".to_string()),
                icon: Some("wand".to_string()),
            },
            ButtonConfig {
                label: "B&W Film Pop".to_string(),
                action: "macro.photocraft.plugins.black_and_white".to_string(),
                hotkey: Some("Ctrl+Shift+U".to_string()),
                icon: Some("bw".to_string()),
            },
        ]
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content =
            std::fs::read_to_string(path.as_ref()).map_err(|e| CompanionError::Io(e))?;
        let buttons: Vec<ButtonConfig> =
            serde_json::from_str(&content).map_err(|e| CompanionError::Json(e))?;
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
    pub action_tx: Option<UnboundedSender<String>>,
    pub last_connected_state: bool,
    pub last_theme: ThemeMode,
    pub tray_icon_handle: Arc<std::sync::Mutex<Option<tray_icon::TrayIcon>>>,
    pub tray_thread_started: bool,
    pub restore_toolbar_flag: Arc<AtomicBool>,
    pub open_manager_flag: Arc<AtomicBool>,
}

impl CompanionOverlayApp {
    pub fn new(
        buttons: Vec<ButtonConfig>,
        manifests: Vec<core_contracts::PluginManifest>,
        prefs_path: impl Into<std::path::PathBuf>,
        is_connected: Arc<AtomicBool>,
        action_tx: Option<UnboundedSender<String>>,
        tray_icon_handle: Arc<std::sync::Mutex<Option<tray_icon::TrayIcon>>>,
    ) -> Self {
        let prefs_path = prefs_path.into();
        let current_theme = ThemeMode::from_preferences(&prefs_path);
        let current_lang = core_contracts::Language::Auto.resolve(Some(&prefs_path));

        let toolstrip = ToolstripApp::new(buttons, current_theme);
        let manager = PluginManagerApp::new(manifests, &prefs_path, is_connected.clone());
        let offline_modal = OfflineModalApp::new(current_lang, current_theme);

        let initial_conn = is_connected.load(Ordering::Relaxed);

        Self {
            toolstrip,
            manager,
            offline_modal,
            show_manager: false,
            is_hidden_to_tray: false,
            is_connected,
            action_tx,
            last_connected_state: initial_conn,
            last_theme: current_theme,
            tray_icon_handle,
            tray_thread_started: false,
            restore_toolbar_flag: Arc::new(AtomicBool::new(false)),
            open_manager_flag: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl eframe::App for CompanionOverlayApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Set direct synchronous tray handlers once for instant reaction (0ms, no channel delays)
        if !self.tray_thread_started {
            self.tray_thread_started = true;
            let ctx_tray = ctx.clone();
            let restore_flag = self.restore_toolbar_flag.clone();
            let manager_flag = self.open_manager_flag.clone();

            tray_icon::menu::MenuEvent::set_event_handler(Some(move |event: tray_icon::menu::MenuEvent| {
                match event.id.as_ref() {
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
                        ctx_tray.send_viewport_cmd(egui::ViewportCommand::Focus);
                        ctx_tray.request_repaint();
                    }
                    _ => {}
                }
            }));

            let ctx_icon = ctx.clone();
            let restore_flag_icon = self.restore_toolbar_flag.clone();
            tray_icon::TrayIconEvent::set_event_handler(Some(move |event: tray_icon::TrayIconEvent| {
                if let tray_icon::TrayIconEvent::Click { button: tray_icon::MouseButton::Left, .. } = event {
                    restore_flag_icon.store(true, Ordering::Relaxed);
                    ctx_icon.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                    ctx_icon.send_viewport_cmd(egui::ViewportCommand::Focus);
                    ctx_icon.request_repaint();
                }
            }));
        }

        // Process asynchronous tray commands instantly
        if self.restore_toolbar_flag.swap(false, Ordering::Relaxed) {
            self.is_hidden_to_tray = false;
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Visible(true));
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Focus);
        }
        if self.open_manager_flag.swap(false, Ordering::Relaxed) {
            self.show_manager = true;
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Focus);
        }

        // Keep reactive loop active so IPC connection state is updated
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(50));

        let palette = ThemePalette::for_mode(self.toolstrip.current_theme);
        ui.ctx().set_visuals(palette.to_egui_visuals());

        let connected = self.is_connected.load(Ordering::Relaxed);

        // Adjust window size if connection state changed
        if connected != self.last_connected_state {
            self.last_connected_state = connected;
            if connected {
                let size = self.toolstrip.calculate_size();
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
                if let Some(pos) = calculate_toolstrip_spawn_pos(size.x) {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(pos[0], pos[1])));
                }
            } else {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::Vec2::new(280.0, 160.0)));
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
            if connected {
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
                if self.toolstrip.open_manager_requested {
                    self.toolstrip.open_manager_requested = false;
                    self.show_manager = true;
                }
                if self.toolstrip.hide_requested {
                    self.toolstrip.hide_requested = false;
                    self.is_hidden_to_tray = true;
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Visible(false));
                }

                // BUG-023: Render external floating tooltip popup viewport if any tool button is hovered
                if let Some((tt_text, btn_rect)) = self.toolstrip.hovered_tooltip.take() {
                    let window_min = ui.ctx().input(|i| i.viewport().outer_rect)
                        .map(|r| r.min)
                        .unwrap_or(egui::pos2(0.0, 0.0));

                    let tt_pos = if self.toolstrip.is_vertical {
                        window_min + egui::vec2(btn_rect.right() + 6.0, btn_rect.top() + 4.0)
                    } else {
                        window_min + egui::vec2(btn_rect.left(), btn_rect.bottom() + 6.0)
                    };

                    let char_count = tt_text.chars().count();
                    let tt_width = (char_count as f32 * 7.5 + 20.0).clamp(60.0, 320.0);
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
                                        .inner_margin(egui::Margin::symmetric(8, 3))
                                )
                                .show(ctx, |ui| {
                                    ui.label(egui::RichText::new(&tt_text).size(11.0).color(palette.text_primary));
                                });
                        },
                    );
                }
            } else {
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
                        ctx.send_viewport_cmd(egui::ViewportCommand::Icon(Some(std::sync::Arc::new(icon))));
                    }
                    egui::CentralPanel::default()
                        .frame(egui::Frame::new().fill(palette.bg_fill).inner_margin(0))
                        .show(ctx, |ui| {
                            self.manager
                                .render_settings_window(ui, &palette, &mut self.show_manager);
                        });
                },
            );
        }
    }
}

/// Helper function to launch the unified floating companion overlay
pub fn launch_companion_overlay(
    buttons: Vec<ButtonConfig>,
    manifests: Vec<core_contracts::PluginManifest>,
    prefs_path: impl Into<std::path::PathBuf>,
    start_with_manager: bool,
    is_connected: Arc<AtomicBool>,
    action_tx: Option<UnboundedSender<String>>,
    tray_icon_handle: Arc<std::sync::Mutex<Option<tray_icon::TrayIcon>>>,
) -> anyhow::Result<()> {
    let mut app = CompanionOverlayApp::new(
        buttons,
        manifests,
        prefs_path,
        is_connected,
        action_tx,
        tray_icon_handle,
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
        if let Some(pos) = calculate_toolstrip_spawn_pos(initial_size.x) {
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
) -> anyhow::Result<()> {
    let is_connected = Arc::new(AtomicBool::new(false));
    let app = PluginManagerApp::new(manifests, prefs_path, is_connected);
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
mod tests {
    use super::*;

    #[test]
    fn test_overlay_bar_default_buttons() {
        let default_btns = OverlayBar::default_buttons();
        assert_eq!(default_btns.len(), 2);
        assert_eq!(default_btns[0].action, "ai.photocraft.plugins.remove_bg");
        assert_eq!(default_btns[1].action, "macro.photocraft.plugins.black_and_white");
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
            default_hotkey: Some("Ctrl+1".to_string()),
            icon: None,
            commands: vec![],
            models: vec![],
            action_id: None,
            entrypoint: None,
            enabled: true,
            plugin_dir: None,
        };
        let m2 = core_contracts::PluginManifest {
            id: "p2".to_string(),
            name: "P2".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            author: "".to_string(),
            plugin_type: core_contracts::PluginType::NoCode,
            default_hotkey: None,
            icon: None,
            commands: vec![],
            models: vec![],
            action_id: None,
            entrypoint: None,
            enabled: false,
            plugin_dir: None,
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
        let temp_dir = std::env::temp_dir().join(format!("overlay_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let bar_file = temp_dir.join("buttons.json");

        let sample_buttons = vec![
            ButtonConfig {
                label: "Custom Action".to_string(),
                action: "custom.action".to_string(),
                hotkey: Some("F5".to_string()),
                icon: Some("custom".to_string()),
            }
        ];
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
}

