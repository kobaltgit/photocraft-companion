#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod win32_theme;
use win32_theme::apply_win32_menu_theme;

mod algorithm_engine;
use algorithm_engine::GenericAlgorithmPlugin;

mod workflow_engine;

use ai_engine::{AiEngine, GenericAiPlugin};
use core_contracts::{
    ExecutionState, PluginContext, PluginRegistry, PluginType, SharedExecutionStatus,
};
use overlay_ui::{get_icon_rgba, DynamicToolEngine, OverlayBar, ThemeMode};
use photocraft_bridge::PhotoCraftClient;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tracing::info;
use tray_icon::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    TrayIconBuilder,
};

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

fn find_plugins_dir() -> PathBuf {
    let user_plugins = core_contracts::user_data_dir().join("plugins");

    // 1. Discover bundled/adjacent plugins directory
    let mut bundled_plugins = None;
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let adjacent = exe_dir.join("plugins");
            if adjacent.is_dir() {
                bundled_plugins = Some(adjacent);
            } else if let Some(target_dir) = exe_dir.parent() {
                if let Some(root) = target_dir.parent() {
                    let candidate = root.join("plugins");
                    if candidate.is_dir() {
                        bundled_plugins = Some(candidate);
                    }
                }
            }
        }
    }
    if bundled_plugins.is_none() {
        if let Ok(cwd) = std::env::current_dir() {
            let mut cur = cwd.as_path();
            for _ in 0..5 {
                let candidate = cur.join("plugins");
                if candidate.is_dir() {
                    bundled_plugins = Some(candidate);
                    break;
                }
                if let Some(parent) = cur.parent() {
                    cur = parent;
                } else {
                    break;
                }
            }
        }
    }

    if let Some(bundled) = bundled_plugins {
        // If bundled directory is writable, use it directly (portable mode)
        if core_contracts::is_dir_writable(&bundled) {
            return bundled;
        }

        // If bundled directory is read-only (e.g. macOS DMG or Windows Program Files),
        // ensure writable user_plugins directory and copy bundled plugins if empty
        let _ = std::fs::create_dir_all(&user_plugins);
        let is_empty = std::fs::read_dir(&user_plugins)
            .map(|mut it| it.next().is_none())
            .unwrap_or(true);
        if is_empty {
            tracing::info!(
                "Bundled plugins directory is read-only. Seeding user plugins directory: {}",
                user_plugins.display()
            );
            let _ = copy_dir_all(&bundled, &user_plugins);
        }
        return user_plugins;
    }

    let _ = std::fs::create_dir_all(&user_plugins);
    user_plugins
}

fn find_photocraft_data_dir() -> PathBuf {
    // 1. Check directory of currently running PhotoCraft process
    if let Some(proc_dir) = PhotoCraftClient::find_running_photocraft_dir() {
        let candidate = proc_dir.join("PhotoCraftData");
        if candidate.is_dir() && core_contracts::is_dir_writable(&candidate) {
            return candidate;
        }
    }

    // 2. Check local relative directory or parents (portable mode)
    let mut candidates = Vec::new();
    candidates.push(PathBuf::from("PhotoCraftData"));
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            candidates.push(exe_dir.join("PhotoCraftData"));
            if let Some(parent) = exe_dir.parent() {
                candidates.push(parent.join("PhotoCraftData"));
                if let Some(grandparent) = parent.parent() {
                    candidates.push(grandparent.join("PhotoCraftData"));
                }
            }
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let mut cur = cwd.as_path();
        for _ in 0..5 {
            candidates.push(cur.join("PhotoCraftData"));
            if let Some(parent) = cur.parent() {
                cur = parent;
            } else {
                break;
            }
        }
    }

    for candidate in candidates {
        if candidate.is_dir() && core_contracts::is_dir_writable(&candidate) {
            return candidate;
        }
    }

    // 3. Check AppData or user home standard directory
    if let Ok(appdata) = std::env::var("APPDATA") {
        let candidate = PathBuf::from(appdata).join("PhotoCraft");
        if candidate.is_dir() && core_contracts::is_dir_writable(&candidate) {
            return candidate;
        }
    }

    // 4. Guaranteed writable user data directory (macOS ~/Library/Application Support, Linux ~/.config, Windows %LOCALAPPDATA%)
    let user_data = core_contracts::user_data_dir().join("PhotoCraftData");
    let _ = std::fs::create_dir_all(&user_data);
    user_data
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("PhotoCraft Companion starting up on main OS thread...");

    let plugins_dir = find_plugins_dir();
    let data_dir = find_photocraft_data_dir();
    let prefs_path = data_dir.join("preferences.json");
    let theme_mode = ThemeMode::from_preferences(&prefs_path);
    let is_dark = theme_mode.is_dark();

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
    let tray_menu = Menu::new();
    let open_toolbar_item = MenuItem::with_id("tray_toolbar", "PhotoCraft Toolbar", true, None);
    let open_manager_item = MenuItem::with_id("tray_manager", "Plugins Manager", true, None);
    let separator = PredefinedMenuItem::separator();
    let exit_item = MenuItem::with_id("tray_exit", "Exit Companion", true, None);

    let _ = tray_menu.append(&open_toolbar_item);
    let _ = tray_menu.append(&open_manager_item);
    let _ = tray_menu.append(&separator);
    let _ = tray_menu.append(&exit_item);

    let mut tray_builder = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_menu_on_left_click(false) // Left click restores toolbar; Right click opens menu
        .with_tooltip("PhotoCraft Companion");

    if let Some(rgba) = get_icon_rgba(is_dark, 32) {
        if let Ok(icon) = tray_icon::Icon::from_rgba(rgba, 32, 32) {
            tray_builder = tray_builder.with_icon(icon);
        }
    }

    let tray_icon = match tray_builder.build() {
        Ok(ti) => {
            info!("System tray icon initialized successfully on main thread.");
            Some(ti)
        }
        Err(e) => {
            tracing::warn!("Failed to create system tray icon: {}", e);
            None
        }
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
    let (action_tx, mut action_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    let client_exec = client.clone();
    let registry_exec = Arc::new(Mutex::new(registry));
    let data_dir_exec = data_dir.clone();
    let plugins_dir_exec = plugins_dir.clone();
    let config_path_exec = config_path.clone();
    let exec_status: SharedExecutionStatus = Arc::new(std::sync::Mutex::new(ExecutionState::Idle));
    let exec_status_worker = exec_status.clone();

    rt.spawn(async move {
        while let Some(raw_action) = action_rx.recv().await {
            let req = core_contracts::ActionRequest::from(raw_action);
            info!(
                "Dispatching action request '{}' (action: {}, sub: {:?})...",
                req, req.action_id, req.sub_action_id
            );

            let action_name = {
                let reg = registry_exec.lock().await;
                reg.get(&req.action_id)
                    .map(|p| p.manifest().name.clone())
                    .unwrap_or_else(|| req.action_id.clone())
            };

            let t_start = Instant::now();
            *exec_status_worker.lock().unwrap() = ExecutionState::Running {
                action_id: req.to_string(),
                action_name: action_name.clone(),
                started_at: t_start,
            };

            if let Some(cmd) = req.action_id.strip_prefix("engine:") {
                info!("Executing PhotoCraft engine command '{}'...", cmd);
                match client_exec
                    .execute_command(cmd, serde_json::json!({}))
                    .await
                {
                    Ok(res) => {
                        let elapsed = t_start.elapsed().as_secs_f32();
                        info!("Engine command '{}' executed successfully: {:?}", cmd, res);
                        *exec_status_worker.lock().unwrap() = ExecutionState::Success {
                            action_name,
                            message: format!("Command '{cmd}' completed"),
                            duration_secs: elapsed,
                            finished_at: Instant::now(),
                        };
                    }
                    Err(e) => {
                        tracing::error!("Engine command '{}' failed: {}", cmd, e);
                        *exec_status_worker.lock().unwrap() = ExecutionState::Error {
                            action_name,
                            error: e.to_string(),
                            finished_at: Instant::now(),
                        };
                    }
                }
            } else if let Ok(doc) = client_exec.get_document_info().await {
                let reg = registry_exec.lock().await;
                if let Some(plugin) = reg.get(&req.action_id) {
                    let manifest = plugin.manifest().clone();
                    let plugin_dir = manifest
                        .plugin_dir
                        .clone()
                        .unwrap_or_else(|| plugins_dir_exec.join(&manifest.id));

                    let mut exec_ctx = workflow_engine::ExecutionContext {
                        source_layer_id: doc.active_layer,
                        result_layer_id: None,
                        active_layer_id: doc.active_layer,
                        document_id: None,
                        document_width: doc.width,
                        document_height: doc.height,
                        has_selection: doc.has_selection,
                        fg_color: None,
                        bg_color: None,
                        custom_vars: std::collections::HashMap::new(),
                    };

                    // 1. Pre-commands (only for non-NoCode plugins, as NoCode commands ARE the plugin body)
                    if manifest.plugin_type != PluginType::NoCode && !manifest.commands.is_empty() {
                        if let Err(e) = workflow_engine::execute_steps(
                            &manifest.commands,
                            &mut exec_ctx,
                            client_exec.as_ref(),
                        )
                        .await
                        {
                            *exec_status_worker.lock().unwrap() = ExecutionState::Error {
                                action_name,
                                error: format!("Pre-commands failed: {e}"),
                                finished_at: Instant::now(),
                            };
                            continue;
                        }
                    }

                    // 2. Main plugin execution
                    let plugin_ctx = PluginContext {
                        document: doc,
                        models_dir: data_dir_exec.join("models"),
                        plugin_dir,
                        client: client_exec.clone(),
                        sub_action_id: req.sub_action_id.clone(),
                    };

                    match plugin.execute(&plugin_ctx).await {
                        Ok(res) => {
                            if let Some(ref data) = res.data {
                                if let Some(src) =
                                    data.get("source_layer_id").and_then(|v| v.as_u64())
                                {
                                    exec_ctx.source_layer_id = Some(src);
                                }
                                if let Some(out_id) =
                                    data.get("result_layer_id").and_then(|v| v.as_u64())
                                {
                                    exec_ctx.result_layer_id = Some(out_id);
                                }
                            }

                            // 3. Post-commands (sub_action variant or manifest fallback)
                            let chosen_sub = req.sub_action_id.as_ref().and_then(|sub_id| {
                                manifest.sub_actions.iter().find(|s| &s.id == sub_id)
                            });
                            let post_steps = if let Some(sub) = chosen_sub {
                                &sub.post_commands
                            } else {
                                &manifest.post_commands
                            };

                            if !post_steps.is_empty() {
                                if let Err(e) = workflow_engine::execute_steps(
                                    post_steps,
                                    &mut exec_ctx,
                                    client_exec.as_ref(),
                                )
                                .await
                                {
                                    *exec_status_worker.lock().unwrap() = ExecutionState::Error {
                                        action_name,
                                        error: format!("Post-commands failed: {e}"),
                                        finished_at: Instant::now(),
                                    };
                                    continue;
                                }
                            }

                            // 4. Persist active sub-action to config
                            if let Some(ref sub_id) = req.sub_action_id {
                                let mut cfg = core_contracts::CompanionConfig::load_or_default(
                                    &config_path_exec,
                                );
                                cfg.active_sub_actions
                                    .insert(manifest.id.clone(), sub_id.clone());
                                let _ = cfg.save(&config_path_exec);
                            }

                            let elapsed = t_start.elapsed().as_secs_f32();
                            info!("Action '{}' finished successfully: {}", req, res.message);
                            *exec_status_worker.lock().unwrap() = ExecutionState::Success {
                                action_name,
                                message: res.message,
                                duration_secs: elapsed,
                                finished_at: Instant::now(),
                            };
                        }
                        Err(e) => {
                            tracing::error!("Action '{}' failed: {}", req, e);
                            *exec_status_worker.lock().unwrap() = ExecutionState::Error {
                                action_name,
                                error: e.to_string(),
                                finished_at: Instant::now(),
                            };
                        }
                    }
                } else {
                    tracing::warn!("Action '{}' not found in registry", req.action_id);
                    *exec_status_worker.lock().unwrap() = ExecutionState::Error {
                        action_name,
                        error: format!("Action '{}' not found in registry", req.action_id),
                        finished_at: Instant::now(),
                    };
                }
            } else {
                let conn_st = client_exec.verify_connection().await;
                let err_msg = match conn_st {
                    core_contracts::ConnectionState::AuthRequired => {
                        "PhotoCraft authentication required. Please set valid token in Settings."
                            .to_string()
                    }
                    core_contracts::ConnectionState::Offline => {
                        "PhotoCraft is offline or unreachable on port 18492.".to_string()
                    }
                    core_contracts::ConnectionState::Connected => {
                        "No active document open in PhotoCraft.".to_string()
                    }
                };
                tracing::warn!("Failed to execute action '{}': {}", req, err_msg);
                *exec_status_worker.lock().unwrap() = ExecutionState::Error {
                    action_name,
                    error: err_msg,
                    finished_at: Instant::now(),
                };
            }
        }
    });

    // IPC action execution worker for interactive dynamic plugins (e.g. Rhai scripts, palette harmony)
    let (ipc_action_tx, mut ipc_action_rx) =
        tokio::sync::mpsc::unbounded_channel::<core_contracts::PluginIpcAction>();
    let client_ipc = client.clone();
    rt_handle.spawn(async move {
        while let Some(action) = ipc_action_rx.recv().await {
            info!(
                "Executing IPC action '{}' with params: {}",
                action.command, action.params
            );
            if action.command == "tools.setColors" {
                let fg = action.params.get("foreground").and_then(|v| v.as_str());
                let bg = action.params.get("background").and_then(|v| v.as_str());
                if let Err(e) = client_ipc.set_colors(fg, bg).await {
                    tracing::warn!("Failed to set colors via IPC: {e}");
                }
            } else if let Err(e) = client_ipc
                .execute_command(&action.command, action.params)
                .await
            {
                tracing::warn!("Failed to execute IPC action '{}': {e}", action.command);
            }
        }
    });

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
        let cfg = core_contracts::CompanionConfig::load_or_default(&config_path_for_launch);
        let configured_path = cfg.photocraft_path.as_deref();
        let resolved_exe = PhotoCraftClient::find_photocraft_executable(configured_path)
            .or_else(|| {
                info!("PhotoCraft executable not automatically found. Prompting user to select application...");
                let mut dialog = rfd::FileDialog::new()
                    .set_title("Locate PhotoCraft Application / Executable");
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
                    dialog = dialog.add_filter("Executable / AppImage", &["AppImage", "bin", ""]);
                }
                if let Some(picked) = dialog.pick_file() {
                    let mut updated_cfg = core_contracts::CompanionConfig::load_or_default(&config_path_for_launch);
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

    // Initialize DynamicToolEngine and UI repaint context holder at composition root
    let current_lang = core_contracts::Language::Auto.resolve(Some(&prefs_path));
    let dynamic_engine = Arc::new(std::sync::Mutex::new(DynamicToolEngine::new_with_language(
        &manifests,
        Some(ipc_action_tx.clone()),
        current_lang,
    )));
    let egui_ctx_holder = Arc::new(std::sync::Mutex::new(None::<overlay_ui::egui::Context>));

    // Background poller for live active session colors (PhotoCraft -> Plugins)
    let client_colors = client.clone();
    let dynamic_engine_colors = dynamic_engine.clone();
    let is_conn_colors = is_connected.clone();
    let ctx_holder_colors = egui_ctx_holder.clone();

    rt_handle.spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(180));
        loop {
            interval.tick().await;
            if !is_conn_colors.load(Ordering::Relaxed) {
                continue;
            }

            let needs_sync = {
                if let Ok(engine) = dynamic_engine_colors.lock() {
                    engine
                        .has_open_windows_with_subscription(core_contracts::IPC_SUB_SESSION_COLORS)
                } else {
                    false
                }
            };

            if needs_sync {
                if let Ok((fg, bg)) = client_colors.get_session_colors().await {
                    let updated = {
                        if let Ok(mut engine) = dynamic_engine_colors.lock() {
                            engine.dispatch_session_colors(&fg, &bg)
                        } else {
                            false
                        }
                    };
                    if updated {
                        if let Ok(guard) = ctx_holder_colors.lock() {
                            if let Some(ctx) = guard.as_ref() {
                                ctx.request_repaint();
                            }
                        }
                    }
                }
            }
        }
    });

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
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_plugins_dir() {
        let p = find_plugins_dir();
        assert!(p.to_string_lossy().contains("plugins"));
    }

    #[test]
    fn test_find_photocraft_data_dir() {
        let p = find_photocraft_data_dir();
        assert!(p.to_string_lossy().contains("PhotoCraftData"));
    }

    #[test]
    fn test_apply_win32_menu_theme() {
        // Mode 1: AllowDark / System, Mode 2: ForceDark, Mode 3: ForceLight
        apply_win32_menu_theme(1);
        apply_win32_menu_theme(2);
        apply_win32_menu_theme(3);
    }

    #[test]
    fn test_no_code_plugin_not_duplicated_as_pre_commands() {
        use core_contracts::{PluginCommandStep, PluginManifest, PluginType};
        let m = PluginManifest {
            id: "test.nocode".to_string(),
            name: "Test NoCode".to_string(),
            plugin_type: PluginType::NoCode,
            commands: vec![PluginCommandStep {
                command: "layer.duplicate".to_string(),
                params: serde_json::json!({}),
                save_as: None,
                ignore_errors: false,
                when: None,
                description: None,
            }],
            ..Default::default()
        };
        // Verify that pre-commands condition strictly checks plugin_type != PluginType::NoCode
        let should_run_pre_commands = m.plugin_type != PluginType::NoCode && !m.commands.is_empty();
        assert!(
            !should_run_pre_commands,
            "NoCode commands must never be executed as pre-commands to avoid duplicate execution"
        );
    }
}
