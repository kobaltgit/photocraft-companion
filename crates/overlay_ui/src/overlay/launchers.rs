use super::*;
use crate::icons::{get_egui_icon_data, get_system_window_icon_size};
use eframe::egui;

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
    on_launch_requested: Option<Arc<dyn Fn() + Send + Sync>>,
    ipc_action_tx: Option<UnboundedSender<core_contracts::PluginIpcAction>>,
    dynamic_engine: Arc<std::sync::Mutex<DynamicToolEngine>>,
    egui_ctx_holder: Arc<std::sync::Mutex<Option<egui::Context>>>,
    is_process_running: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
    plugins_dir: Option<std::path::PathBuf>,
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
        ipc_action_tx,
        dynamic_engine,
    );
    if let Some(dir) = plugins_dir {
        app = app.with_plugins_dir(dir);
    }
    if let Some(cb) = on_launch_requested {
        app = app.with_launch_callback(cb);
    }
    if let Some(cb) = is_process_running {
        app = app.with_process_detector(cb);
    }
    if start_with_manager {
        app.show_manager = true;
        app.manager_just_opened = true;
        app.manager_icons_calibrated = false;
    }
    let connected = app.is_connected.load(Ordering::Relaxed);

    let initial_size = if connected {
        app.toolstrip.calculate_size()
    } else {
        egui::Vec2::new(300.0, 280.0)
    };

    let is_pinned = app.offline_modal.is_pinned;
    let mut viewport = egui::ViewportBuilder::default()
        .with_decorations(false)
        .with_transparent(true)
        .with_resizable(false)
        .with_taskbar(false) // Floating toolbar must NOT create an item on Windows Taskbar!
        .with_inner_size(initial_size)
        .with_title("PhotoCraft Toolbar");

    if connected || is_pinned {
        viewport = viewport.with_always_on_top();
    }

    if connected {
        if let Some(pos) = app.last_saved_pos {
            if crate::win32_window::is_position_visible_on_screen(
                pos,
                [initial_size.x, initial_size.y],
            ) {
                viewport = viewport.with_position(egui::Pos2::new(pos[0], pos[1]));
            } else {
                let center = crate::win32_window::calculate_primary_screen_center(
                    initial_size.x,
                    initial_size.y,
                );
                viewport = viewport.with_position(egui::Pos2::new(center[0], center[1]));
            }
        } else {
            let center = crate::win32_window::calculate_primary_screen_center(
                initial_size.x,
                initial_size.y,
            );
            viewport = viewport.with_position(egui::Pos2::new(center[0], center[1]));
        }
    } else {
        let pos = if let Some(p) = app.last_saved_offline_pos {
            if crate::win32_window::is_position_visible_on_screen(
                p,
                [initial_size.x, initial_size.y],
            ) {
                p
            } else {
                crate::win32_window::calculate_primary_screen_center(initial_size.x, initial_size.y)
            }
        } else {
            crate::win32_window::calculate_primary_screen_center(initial_size.x, initial_size.y)
        };
        viewport = viewport.with_position(egui::Pos2::new(pos[0], pos[1]));
    }

    if let Some(icon) = get_egui_icon_data(get_system_window_icon_size()) {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "PhotoCraft Toolbar",
        options,
        Box::new(move |cc| {
            if let Ok(mut holder) = egui_ctx_holder.lock() {
                *holder = Some(cc.egui_ctx.clone());
            }
            Ok(Box::new(app))
        }),
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
    let prefs_buf: std::path::PathBuf = prefs_path.into();
    let data_dir = prefs_buf.parent().unwrap_or(&prefs_buf).to_path_buf();
    let config_path = core_contracts::CompanionConfig::resolve_config_path(&data_dir);
    let cfg = core_contracts::CompanionConfig::load_or_default(&config_path);
    let default_size = [740.0, 480.0];
    let manager_size = cfg.manager_window_size.unwrap_or(default_size);
    let manager_pos = if let Some(pos) = cfg.manager_window_position {
        if crate::win32_window::is_position_visible_on_screen(pos, manager_size) {
            pos
        } else {
            crate::win32_window::calculate_primary_screen_center(manager_size[0], manager_size[1])
        }
    } else {
        crate::win32_window::calculate_primary_screen_center(manager_size[0], manager_size[1])
    };

    let is_connected = Arc::new(AtomicBool::new(false));
    let mut app = PluginManagerApp::new(manifests, prefs_buf, is_connected);
    app.config_path = config_path;
    if let Some(cs) = connection_state {
        app = app.with_connection_state(cs);
    }
    if let Some(cb) = on_token_saved {
        app = app.with_token_callback(cb);
    }

    #[allow(unused_mut)]
    let mut viewport = eframe::egui::ViewportBuilder::default()
        .with_inner_size(manager_size)
        .with_position(egui::pos2(manager_pos[0], manager_pos[1]))
        .with_min_inner_size([600.0, 380.0])
        .with_taskbar(true)
        .with_title("PhotoCraft - Plugins Manager");

    if let Some(icon) = get_egui_icon_data(get_system_window_icon_size()) {
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
