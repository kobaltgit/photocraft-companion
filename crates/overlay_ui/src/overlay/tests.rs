#![allow(clippy::arc_with_non_send_sync, clippy::field_reassign_with_default)]

use super::*;
use crate::calculate_toolstrip_spawn_pos;
use eframe::egui;

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
        plugin_dir: None,
        sub_actions: Vec::new(),
        active_sub_action: None,
        name_ru: None,
        locales: std::collections::HashMap::new(),
        requires_selection: false,
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
    let dynamic_engine = Arc::new(std::sync::Mutex::new(DynamicToolEngine::new(&[], None)));
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
        None,
        dynamic_engine.clone(),
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
        None,
        dynamic_engine,
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
    let dynamic_engine = Arc::new(std::sync::Mutex::new(DynamicToolEngine::new(&[], None)));
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
        None,
        dynamic_engine,
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

    // 4. Trigger reset_layout_flag: restores toolbar and resets layout state
    app.reset_layout_flag.store(true, Ordering::Relaxed);
    app.process_tray_flags(&ctx);
    assert!(!app.reset_layout_flag.load(Ordering::Relaxed));
    assert_eq!(app.last_saved_pos, None);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_tray_restore_cooldown_frames_lifecycle() {
    let temp_dir = std::env::temp_dir().join(format!(
        "tray_cooldown_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let prefs_file = temp_dir.join("preferences.json");
    std::fs::write(&prefs_file, "{}").unwrap();

    let is_connected = Arc::new(AtomicBool::new(true));
    let dynamic_engine = Arc::new(std::sync::Mutex::new(DynamicToolEngine::new(&[], None)));
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
        None,
        dynamic_engine,
    );

    let ctx = egui::Context::default();
    app.hide_toolbar_to_tray(&ctx);
    assert!(app.is_hidden_to_tray);
    assert_eq!(app.tray_restore_cooldown_frames, 0);

    // Restoring from tray activates a 2-frame cooldown buffer (BUG-109)
    app.restore_toolbar_from_tray(&ctx);
    assert!(!app.is_hidden_to_tray);
    assert_eq!(app.tray_restore_cooldown_frames, 2);

    // Frame ticks decrement cooldown
    if app.tray_restore_cooldown_frames > 0 {
        app.tray_restore_cooldown_frames -= 1;
    }
    assert_eq!(app.tray_restore_cooldown_frames, 1);

    if app.tray_restore_cooldown_frames > 0 {
        app.tray_restore_cooldown_frames -= 1;
    }
    assert_eq!(app.tray_restore_cooldown_frames, 0);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_manager_geometry_persistence() {
    let temp_dir = std::env::temp_dir().join(format!(
        "manager_persist_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let prefs_file = temp_dir.join("preferences.json");
    std::fs::write(&prefs_file, "{}").unwrap();

    let is_connected = Arc::new(AtomicBool::new(false));
    let dynamic_engine = Arc::new(std::sync::Mutex::new(DynamicToolEngine::new(&[], None)));
    let mut app = CompanionOverlayApp::new(
        OverlayBar::default_buttons(),
        vec![],
        &prefs_file,
        is_connected.clone(),
        None,
        Arc::new(std::sync::Mutex::new(None)),
        None,
        None,
        None,
        None,
        dynamic_engine.clone(),
    );

    assert!(app.last_saved_manager_pos.is_none());
    assert!(app.last_saved_manager_size.is_none());

    app.last_saved_manager_pos = Some([510.0, 220.0]);
    app.last_saved_manager_size = Some([800.0, 520.0]);
    app.save_layout_state();

    let saved_cfg = core_contracts::CompanionConfig::load_or_default(&app.config_path);
    assert_eq!(saved_cfg.manager_window_position, Some([510.0, 220.0]));
    assert_eq!(saved_cfg.manager_window_size, Some([800.0, 520.0]));

    let restored_app = CompanionOverlayApp::new(
        OverlayBar::default_buttons(),
        vec![],
        &prefs_file,
        is_connected,
        None,
        Arc::new(std::sync::Mutex::new(None)),
        None,
        None,
        None,
        None,
        dynamic_engine,
    );

    assert_eq!(restored_app.last_saved_manager_pos, Some([510.0, 220.0]));
    assert_eq!(restored_app.last_saved_manager_size, Some([800.0, 520.0]));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_screen_boundary_and_centering() {
    let center = crate::win32_window::calculate_primary_screen_center(300.0, 280.0);
    assert!(center[0] >= 20.0);
    assert!(center[1] >= 20.0);

    // Center on screen must be recognized as visible
    assert!(crate::win32_window::is_position_visible_on_screen(
        center,
        [300.0, 280.0]
    ));

    // Completely invalid NaN / Infinite coordinates must be rejected
    assert!(!crate::win32_window::is_position_visible_on_screen(
        [f32::NAN, 100.0],
        [300.0, 280.0]
    ));
    assert!(!crate::win32_window::is_position_visible_on_screen(
        [100.0, f32::INFINITY],
        [300.0, 280.0]
    ));
}

#[test]
fn test_calculate_repaint_interval_lifecycle() {
    let temp_dir = std::env::temp_dir().join(format!(
        "repaint_interval_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let prefs_file = temp_dir.join("preferences.json");
    std::fs::write(&prefs_file, "{}").unwrap();

    let plugin_with_ui = core_contracts::PluginManifest {
        id: "interactive_tool".to_string(),
        name: "Interactive Tool".to_string(),
        interactive_ui: Some(core_contracts::InteractiveUiConfig {
            window_title: "Interactive Tool".to_string(),
            default_width: Some(300.0),
            default_height: Some(200.0),
            ..Default::default()
        }),
        ..Default::default()
    };

    let dynamic_engine = Arc::new(std::sync::Mutex::new(DynamicToolEngine::new(
        &[plugin_with_ui],
        None,
    )));
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
        None,
        dynamic_engine.clone(),
    );

    // 1. Stable 20 FPS heartbeat (50ms) in active visible mode restores buttery v2.0.3 responsiveness
    assert_eq!(
        app.calculate_repaint_interval(),
        std::time::Duration::from_millis(50)
    );

    // 2. Secondary windows open (Manager / Dynamic tools) keep responsive 50ms heartbeat
    app.show_manager = true;
    assert_eq!(
        app.calculate_repaint_interval(),
        std::time::Duration::from_millis(50)
    );
    app.show_manager = false;

    assert!(!dynamic_engine.lock().unwrap().is_any_window_open());
    dynamic_engine
        .lock()
        .unwrap()
        .toggle_window("interactive_tool");
    assert!(dynamic_engine.lock().unwrap().is_any_window_open());
    assert_eq!(
        app.calculate_repaint_interval(),
        std::time::Duration::from_millis(50)
    );

    // 3. Power-saving idle mode (500ms) when toolbar is hidden in system tray
    app.is_hidden_to_tray = true;
    assert_eq!(
        app.calculate_repaint_interval(),
        std::time::Duration::from_millis(500)
    );

    // 4. Restored from tray returns to 50ms active heartbeat
    let ctx = egui::Context::default();
    app.restore_toolbar_from_tray(&ctx);
    assert_eq!(
        app.calculate_repaint_interval(),
        std::time::Duration::from_millis(50)
    );

    // 5. Offline modal mode (when disconnected from PhotoCraft) uses calm 250ms interval
    app.is_connected.store(false, Ordering::Relaxed);
    assert_eq!(
        app.calculate_repaint_interval(),
        std::time::Duration::from_millis(250)
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_reset_layout_clears_plugin_geometries() {
    let temp_dir = std::env::temp_dir().join(format!(
        "reset_layout_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let prefs_file = temp_dir.join("preferences.json");
    std::fs::write(&prefs_file, "{}").unwrap();
    let config_path = core_contracts::CompanionConfig::resolve_config_path(&temp_dir);

    let mut initial_cfg = core_contracts::CompanionConfig::default();
    initial_cfg.window_position = Some([100.0, 100.0]);
    initial_cfg.is_vertical = true;
    initial_cfg.set_plugin_geometry("interactive_tool", [200.0, 200.0], [400.0, 300.0]);
    initial_cfg.save(&config_path).unwrap();

    let plugin_with_ui = core_contracts::PluginManifest {
        id: "interactive_tool".to_string(),
        name: "Interactive Tool".to_string(),
        interactive_ui: Some(core_contracts::InteractiveUiConfig {
            window_title: "Interactive Tool".to_string(),
            default_width: Some(300.0),
            default_height: Some(200.0),
            ..Default::default()
        }),
        ..Default::default()
    };

    let dynamic_engine = Arc::new(std::sync::Mutex::new(DynamicToolEngine::new(
        &[plugin_with_ui],
        None,
    )));
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
        None,
        dynamic_engine.clone(),
    );

    // Pre-condition: dynamic engine should have loaded plugin geometry from config
    {
        let engine = dynamic_engine.lock().unwrap();
        let state = engine.get_state("interactive_tool").expect("state exists");
        assert_eq!(state.last_pos, Some([200.0, 200.0]));
        assert_eq!(state.last_size, Some([400.0, 300.0]));
    }

    // Execute reset layout
    app.reset_layout(None);

    // 1. Verify in-memory dynamic_engine states are cleared
    {
        let engine = dynamic_engine.lock().unwrap();
        let state = engine.get_state("interactive_tool").expect("state exists");
        assert_eq!(state.last_pos, None);
        assert_eq!(state.last_size, None);
    }

    // 2. Verify saved configuration on disk has plugin geometries cleared and orientation reset
    let loaded_cfg = core_contracts::CompanionConfig::load_or_default(&config_path);
    assert!(loaded_cfg.plugin_window_positions.is_empty());
    assert!(loaded_cfg.plugin_window_sizes.is_empty());
    assert_eq!(loaded_cfg.window_position, None);
    assert!(!loaded_cfg.is_vertical);
    assert!(!app.toolstrip.is_vertical);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_tray_menu_multilingual_localization() {
    assert_eq!(
        tray_t(
            "PhotoCraft Toolbar",
            "Панель PhotoCraft",
            core_contracts::Language::Ru
        ),
        "Панель PhotoCraft"
    );
    assert_eq!(
        tray_t(
            "PhotoCraft Toolbar",
            "Панель PhotoCraft",
            core_contracts::Language::En
        ),
        "PhotoCraft Toolbar"
    );
    assert_eq!(
        tray_t(
            "PhotoCraft Toolbar",
            "Панель PhotoCraft",
            core_contracts::Language::De
        ),
        "PhotoCraft-Symbolleiste"
    );

    assert_eq!(
        tray_t(
            "Plugins Manager",
            "Менеджер плагинов",
            core_contracts::Language::Ru
        ),
        "Менеджер плагинов"
    );
    assert_eq!(
        tray_t(
            "Plugins Manager",
            "Менеджер плагинов",
            core_contracts::Language::En
        ),
        "Plugins Manager"
    );

    assert_eq!(
        tray_t(
            "Reset Layout",
            "Сбросить расположение",
            core_contracts::Language::Ru
        ),
        "Сбросить расположение"
    );
    assert_eq!(
        tray_t(
            "Reset Layout",
            "Сбросить расположение",
            core_contracts::Language::En
        ),
        "Reset Layout"
    );
    assert_eq!(
        tray_t(
            "Reset Layout",
            "Сбросить расположение",
            core_contracts::Language::De
        ),
        "Layout zurücksetzen"
    );
    assert_eq!(
        tray_t(
            "Reset Layout",
            "Сбросить расположение",
            core_contracts::Language::Fr
        ),
        "Réinitialiser la disposition"
    );

    assert_eq!(
        tray_t(
            "Exit Companion",
            "Выход из Компаньона",
            core_contracts::Language::Ru
        ),
        "Выход из Компаньона"
    );
    assert_eq!(
        tray_t(
            "Exit Companion",
            "Выход из Компаньона",
            core_contracts::Language::En
        ),
        "Exit Companion"
    );

    // Building the menu should succeed across languages without panics
    let _menu_ru = create_tray_menu(core_contracts::Language::Ru);
    let _menu_en = create_tray_menu(core_contracts::Language::En);
    let _menu_de = create_tray_menu(core_contracts::Language::De);
}
