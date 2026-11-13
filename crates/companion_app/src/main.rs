#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ai_engine::{AiEngine, GenericAiPlugin};
use algorithm_engine::GenericAlgorithmPlugin;
use core_contracts::{
    ExecutionState, PluginContext, PluginRegistry, PluginType, SharedExecutionStatus,
};
use discovery::{find_photocraft_data_dir, find_plugins_dir};
use dispatcher::{spawn_action_dispatcher, spawn_ipc_action_dispatcher};
use overlay_ui::{
    get_system_small_icon_size, get_tray_icon, DynamicToolEngine, OverlayBar, ThemeMode,
};
use photocraft_bridge::PhotoCraftClient;
use session_sync::spawn_session_sync_worker;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::info;
use tray_icon::TrayIconBuilder;
use win32_theme::apply_win32_menu_theme;

mod algorithm_engine;
mod cloud_engine;
mod discovery;
mod dispatcher;
mod raster;
mod session_sync;
mod win32_theme;
mod workflow_engine;

#[cfg(test)]
mod tests;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("PhotoCraft Companion starting up on main OS thread...");

    // On Linux, the GTK library must be initialized on the main thread before creating any
    // tray icons or menu items, otherwise tray-icon/libappindicator panics immediately.
    #[cfg(target_os = "linux")]
    let gtk_initialized = match gtk::init() {
        Ok(_) => {
            info!("GTK initialized successfully on Linux main thread.");
            true
        }
        Err(e) => {
            tracing::warn!("Failed to initialize GTK on Linux: {e}. System tray may be disabled.");
            false
        }
    };
    #[cfg(not(target_os = "linux"))]
    let gtk_initialized = true;

    let plugins_dir = find_plugins_dir();
    let data_dir = find_photocraft_data_dir();
    let prefs_path = data_dir.join("preferences.json");
    let theme_mode = ThemeMode::from_preferences(&prefs_path);

    // Set Win32 context menu theme strictly matching the selected theme (no white popups in dark mode)
    let win32_mode = match theme_mode {
        ThemeMode::Dark => 2,   // ForceDark
        ThemeMode::Light => 3,  // ForceLight
        ThemeMode::System => 1, // AllowDark / System
    };
    apply_win32_menu_theme(win32_mode);

    // Initialize background async Tokio runtime for networking & engine bridge
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let _rt_guard = rt.enter();
    let rt_handle = rt.handle().clone();

    // 1. Initialize Plugin Registry with dynamic AI Factory
    // Any Native AI plugin placed in plugins/ will be discovered and instantiated automatically!
    println!("\n=== DISCOVERING & LOADING PLUGINS ===");
    let ai = Arc::new(AiEngine::new(&data_dir));
    let ai_for_factory = ai.clone();
    let ai_factory: core_contracts::AiPluginFactory = Arc::new(move |manifest| {
        Ok(Arc::new(GenericAiPlugin::new(
            manifest.clone(),
            ai_for_factory.clone(),
        )))
    });

    let algo_factory: core_contracts::AlgorithmPluginFactory =
        Arc::new(move |manifest| Ok(Arc::new(GenericAlgorithmPlugin::new(manifest.clone()))));

    let mut registry = PluginRegistry::new(&plugins_dir)
        .with_ai_factory(ai_factory)
        .with_algorithm_factory(algo_factory);

    // Scan for all plugins (No-Code, Native AI, Native Algorithm, Script, CLI) automatically from `plugins/`
    let _loaded = registry.scan_and_load()?;
    println!(
        "  Total registered plugin manifests: {}",
        registry.list_manifests().len()
    );
    for m in registry.list_manifests() {
        let p_type = match m.plugin_type {
            PluginType::NoCode => "No-Code Macro",
            PluginType::NativeAi => "Native AI / GPU",
            PluginType::NativeAlgorithm => "Native Algorithm",
            PluginType::Script => "Script Automation",
            PluginType::ExternalCli => "External CLI Tool",
            PluginType::InteractiveUi => "Interactive UI Panel",
        };
        println!(
            "  - [{}] {} (v{}) by {}",
            p_type, m.name, m.version, m.author
        );
    }

    // 2. Load companion configuration first to obtain user token and connection settings
    let config_path = core_contracts::CompanionConfig::resolve_config_path(&data_dir);
    let companion_cfg = core_contracts::CompanionConfig::load_or_default(&config_path);
    let initial_token = companion_cfg
        .token
        .clone()
        .or_else(PhotoCraftClient::default_token);

    // 4. Setup Bridge Client and 3-State Connection Monitor
    let client = Arc::new(PhotoCraftClient::new(companion_cfg.port, initial_token));
    let is_connected = Arc::new(AtomicBool::new(false));
    let connection_state = Arc::new(std::sync::RwLock::new(
        core_contracts::ConnectionState::Offline,
    ));

    // Try initial connection synchronously on startup
    let client_init = client.clone();
    let is_conn_init = is_connected.clone();
    let conn_state_init = connection_state.clone();
    rt.block_on(async {
        let state = client_init.verify_connection().await;
        *conn_state_init.write().unwrap() = state;
        match state {
            core_contracts::ConnectionState::Connected => {
                is_conn_init.store(true, Ordering::Relaxed);
                let _ = client_init.connect().await;
                info!("Successfully connected to live PhotoCraft instance on startup!");
                if let Ok(doc) = client_init.get_document_info().await {
                    println!("=== CURRENT ACTIVE DOCUMENT ===");
                    println!("Name:         {}", doc.name);
                    println!("Resolution:   {}x{}", doc.width, doc.height);
                    println!("Active Layer: {:?}", doc.active_layer);
                }
            }
            core_contracts::ConnectionState::AuthRequired => {
                is_conn_init.store(false, Ordering::Relaxed);
                println!("PhotoCraft is running on port {}, but authentication is required. Please set valid control.token in Settings.", companion_cfg.port);
            }
            core_contracts::ConnectionState::Offline => {
                is_conn_init.store(false, Ordering::Relaxed);
                println!("PhotoCraft is not yet detected on port {}. Entering background polling mode (1.5s)...", companion_cfg.port);
            }
        }
    });

    // Background monitor task (checks verify_connection every 1.5s per specification)
    let is_conn_monitor = is_connected.clone();
    let conn_state_monitor = connection_state.clone();
    let client_monitor = client.clone();
    rt.spawn(async move {
        loop {
            let state = client_monitor.verify_connection().await;
            let prev_state = *conn_state_monitor.read().unwrap();
            if state != prev_state {
                *conn_state_monitor.write().unwrap() = state;
                match state {
                    core_contracts::ConnectionState::Connected => {
                        is_conn_monitor.store(true, Ordering::Relaxed);
                        let _ = client_monitor.connect().await;
                        info!("PhotoCraft connected and authenticated successfully on socket 18492.");
                    }
                    core_contracts::ConnectionState::AuthRequired => {
                        is_conn_monitor.store(false, Ordering::Relaxed);
                        tracing::warn!("PhotoCraft detected on port 18492, but authentication failed or token is missing.");
                    }
                    core_contracts::ConnectionState::Offline => {
                        is_conn_monitor.store(false, Ordering::Relaxed);
                        tracing::warn!("PhotoCraft was closed or socket 18492 unreachable.");
                    }
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        }
    });

    let args: Vec<String> = std::env::args().collect();
    let is_test_run = args.iter().any(|a| a == "--test" || a == "--benchmark");
    let is_daemon_only = args.iter().any(|a| a == "--daemon" || a == "-d");
    let start_with_manager = args.iter().any(|a| a == "--manager" || a == "-m");

    // Initialize System Tray Icon on the main OS thread using user's SVG icon
    let tray_icon = if gtk_initialized {
        let app_cfg = core_contracts::CompanionConfig::load_or_default(&config_path);
        let cfg_lang = core_contracts::Language::from_str_code(&app_cfg.language);
        let resolved_auto = core_contracts::Language::Auto.resolve(Some(&prefs_path));
        let startup_lang = if cfg_lang == core_contracts::Language::Auto {
            resolved_auto.safe_ui_language()
        } else {
            cfg_lang.safe_ui_language()
        };
        let tray_menu = overlay_ui::create_tray_menu(startup_lang);

        let mut tray_builder = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_menu_on_left_click(false) // Left click restores toolbar; Right click opens menu
            .with_tooltip("PhotoCraft Companion");

        let tray_size = get_system_small_icon_size();
        if let Some(icon) = get_tray_icon(tray_size) {
            tray_builder = tray_builder.with_icon(icon);
        }

        match tray_builder.build() {
            Ok(ti) => {
                info!("System tray icon initialized successfully on main thread.");
                Some(ti)
            }
            Err(e) => {
                tracing::warn!("Failed to create system tray icon: {}", e);
                None
            }
        }
    } else {
        tracing::warn!("Skipping system tray icon initialization because GTK is not initialized.");
        None
    };
    #[allow(clippy::arc_with_non_send_sync)]
    let tray_icon_handle = Arc::new(std::sync::Mutex::new(tray_icon));

    if is_test_run {
        println!("\n>>> Running Native AI Plugin Execution via PluginRegistry <<<");
        let client_test = client.clone();
        let data_dir_test = data_dir.clone();
        let plugins_dir_test = plugins_dir.clone();
        let reg_test = registry;

        rt.block_on(async {
            let doc_info = client_test.get_document_info().await?;
            for manifest in reg_test.list_manifests() {
                println!(
                    "\n>>> Testing registered plugin execution: {} [{}] <<<",
                    manifest.name, manifest.id
                );
                let p_dir = manifest
                    .plugin_dir
                    .clone()
                    .unwrap_or_else(|| plugins_dir_test.join(&manifest.id));
                let ctx = PluginContext {
                    document: doc_info.clone(),
                    models_dir: data_dir_test.join("models"),
                    plugin_dir: p_dir,
                    client: client_test.clone(),
                    sub_action_id: None,
                };
                let t0 = Instant::now();
                match reg_test.execute(&manifest.id, &ctx).await {
                    Ok(result) => {
                        println!(
                            "  Result: {} in {:.2}s",
                            result.message,
                            t0.elapsed().as_secs_f32()
                        );
                    }
                    Err(e) => {
                        println!("  Error: {} in {:.2}s", e, t0.elapsed().as_secs_f32());
                    }
                }
            }
            Ok::<(), anyhow::Error>(())
        })?;
        return Ok(());
    }

    // Action execution worker for toolstrip clicks
    let manifests = registry.list_manifests().to_vec();
    let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let client_exec = client.clone();
    let registry_exec = Arc::new(Mutex::new(registry));
    let data_dir_exec = data_dir.clone();
    let plugins_dir_exec = plugins_dir.clone();
    let config_path_exec = config_path.clone();
    let prefs_path_exec = prefs_path.clone();
    let exec_status: SharedExecutionStatus = Arc::new(std::sync::Mutex::new(ExecutionState::Idle));
    let exec_status_worker = exec_status.clone();

    spawn_action_dispatcher(
        &rt,
        action_rx,
        client_exec,
        registry_exec,
        data_dir_exec,
        plugins_dir_exec,
        config_path_exec,
        prefs_path_exec,
        exec_status_worker,
    );

    // Initialize DynamicToolEngine and UI repaint context holder at composition root
    let (ipc_action_tx, ipc_action_rx) =
        tokio::sync::mpsc::unbounded_channel::<core_contracts::PluginIpcAction>();
    let app_cfg = core_contracts::CompanionConfig::load_or_default(&config_path);
    let cfg_lang = core_contracts::Language::from_str_code(&app_cfg.language);
    let resolved_auto = core_contracts::Language::Auto.resolve(Some(&prefs_path));
    let current_lang = if cfg_lang == core_contracts::Language::Auto {
        resolved_auto
    } else {
        cfg_lang
    };
    let mut dyn_engine =
        DynamicToolEngine::new_with_language(&manifests, Some(ipc_action_tx.clone()), current_lang);
    dyn_engine.set_config_path(config_path.clone());
    let dynamic_engine = Arc::new(std::sync::Mutex::new(dyn_engine));
    let egui_ctx_holder = Arc::new(std::sync::Mutex::new(None::<overlay_ui::egui::Context>));

    // IPC action execution worker for interactive dynamic plugins (e.g. Rhai scripts, palette harmony)
    let client_ipc = client.clone();
    let dynamic_engine_ipc = dynamic_engine.clone();
    let egui_ctx_holder_ipc = egui_ctx_holder.clone();
    let config_path_ipc = config_path.clone();

    spawn_ipc_action_dispatcher(
        &rt_handle,
        ipc_action_rx,
        client_ipc,
        dynamic_engine_ipc,
        egui_ctx_holder_ipc,
        config_path_ipc,
    );

    if is_daemon_only {
        println!("\nPhotoCraft Companion is active in system tray! (Press Ctrl+C to stop)");
        // Register direct tray exit handler for daemon mode
        tray_icon::menu::MenuEvent::set_event_handler(Some(|event: tray_icon::menu::MenuEvent| {
            if event.id.as_ref() == "tray_exit" {
                info!("Tray exit clicked in daemon mode, terminating.");
                std::process::exit(0);
            }
        }));

        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }

    // Default & Manager mode: launch the floating toolstrip and overlay on the main OS thread
    println!("\n>>> Launching PhotoCraft Floating Toolstrip & Companion Overlay <<<");
    let buttons = OverlayBar::from_manifests(&manifests);

    let client_for_ui = client.clone();
    let rt_handle_ui = rt_handle.clone();
    let on_token_saved: Arc<dyn Fn(Option<String>) + Send + Sync> = Arc::new(move |new_token| {
        let c = client_for_ui.clone();
        rt_handle_ui.spawn(async move {
            c.set_token(new_token).await;
            info!("PhotoCraftClient token dynamically updated from UI settings.");
        });
    });

    let client_for_launch = client.clone();
    let port = companion_cfg.port;
    let config_path_for_launch = config_path.clone();
    let rt_handle_launch = rt_handle.clone();
    let on_launch_requested: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
        let current_cfg = core_contracts::CompanionConfig::load_or_default(&config_path_for_launch);
        let resolved_exe =
            PhotoCraftClient::find_photocraft_executable(current_cfg.photocraft_path.as_deref())
                .or_else(|| {
                    let mut dialog =
                        rfd::FileDialog::new().set_title("Locate PhotoCraft Executable");
                    #[cfg(target_os = "windows")]
                    {
                        dialog = dialog.add_filter("Executable", &["exe"]);
                    }
                    #[cfg(target_os = "macos")]
                    {
                        dialog = dialog.add_filter("PhotoCraft App / Executable", &["app", ""]);
                    }
                    #[cfg(target_os = "linux")]
                    {
                        dialog = dialog
                            .add_filter(
                                "PhotoCraft Executable / AppImage / Desktop Entry",
                                &["AppImage", "appimage", "desktop", "bin", ""],
                            )
                            .add_filter("All Files (*)", &["*"]);
                    }
                    if let Some(picked) = dialog.pick_file() {
                        let mut updated_cfg = core_contracts::CompanionConfig::load_or_default(
                            &config_path_for_launch,
                        );
                        updated_cfg.photocraft_path = Some(picked.clone());
                        if let Err(e) = updated_cfg.save(&config_path_for_launch) {
                            tracing::warn!("Failed to save photocraft_path: {e}");
                        }
                        PhotoCraftClient::find_photocraft_executable(Some(&picked))
                    } else {
                        None
                    }
                });

        if let Some(exe_path) = resolved_exe {
            info!(
                "Attempting to launch PhotoCraft from: {}",
                exe_path.display()
            );
            match PhotoCraftClient::launch_photocraft(&exe_path, port, None) {
                Ok(_child) => {
                    info!("PhotoCraft process spawned successfully!");
                    PhotoCraftClient::invalidate_process_running_cache();
                    let c = client_for_launch.clone();
                    rt_handle_launch.spawn(async move {
                        for _ in 0..10 {
                            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                            if let Some(tok) = PhotoCraftClient::default_token() {
                                c.set_token(Some(tok)).await;
                                info!("Updated PhotoCraftClient token after auto-launch.");
                                break;
                            }
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to launch PhotoCraft: {e}");
                }
            }
        } else {
            tracing::warn!("PhotoCraft executable not selected or not found.");
        }
    });

    // Background poller for live active session colors and document info
    spawn_session_sync_worker(
        &rt_handle,
        client.clone(),
        dynamic_engine.clone(),
        is_connected.clone(),
        egui_ctx_holder.clone(),
        ai,
        &manifests,
    );

    let is_process_running: Arc<dyn Fn() -> bool + Send + Sync> =
        Arc::new(photocraft_bridge::PhotoCraftClient::is_photocraft_process_running);

    overlay_ui::launch_companion_overlay(
        buttons,
        manifests,
        prefs_path,
        start_with_manager,
        is_connected,
        Some(action_tx),
        tray_icon_handle,
        Some(exec_status),
        Some(connection_state),
        Some(on_token_saved),
        Some(on_launch_requested),
        Some(ipc_action_tx),
        dynamic_engine,
        egui_ctx_holder,
        Some(is_process_running),
        Some(plugins_dir),
    )?;

    Ok(())
}
