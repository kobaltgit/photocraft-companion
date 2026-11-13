use super::*;

#[test]
fn test_plugin_manager_app_init() {
    let temp_dir = std::env::temp_dir().join(format!(
        "mgr_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let prefs_file = temp_dir.join("preferences.json");
    std::fs::write(
        &prefs_file,
        r#"{"interface": {"language": "en", "theme": "studioDark"}}"#,
    )
    .unwrap();

    let is_connected = Arc::new(AtomicBool::new(true));
    let app = PluginManagerApp::new(vec![], prefs_file, is_connected.clone());

    assert_eq!(app.active_tab, ActiveTab::General);
    assert_eq!(app.current_language, Language::Auto);
    assert_eq!(app.resolved_auto_language, Language::En);
    assert_eq!(app.current_theme, ThemeMode::Dark);
    assert_eq!(app.host, "127.0.0.1");
    assert_eq!(app.port, 18492);
    assert!(app.is_connected.load(Ordering::SeqCst));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_plugin_manager_app_translations() {
    let temp_dir = std::env::temp_dir().join(format!(
        "mgr_trans_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let prefs_file = temp_dir.join("preferences.json");
    let is_connected = Arc::new(AtomicBool::new(false));

    let mut app = PluginManagerApp::new(vec![], prefs_file, is_connected);
    app.current_language = Language::En;
    assert_eq!(app.t("Save", "Сохранить"), "Save");

    app.current_language = Language::Ru;
    assert_eq!(app.t("Save", "Сохранить"), "Сохранить");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_plugin_manager_app_save_settings() {
    let temp_dir = std::env::temp_dir().join(format!(
        "mgr_save_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let prefs_file = temp_dir.join("preferences.json");
    let is_connected = Arc::new(AtomicBool::new(false));

    let mut app = PluginManagerApp::new(vec![], &prefs_file, is_connected);
    app.config_path = temp_dir.join("companion_config.json");
    app.port = 19999;
    app.current_theme = ThemeMode::Light;
    app.photocraft_path = "D:/apps/photocraft.exe".to_string();

    let res = app.save_settings();
    assert!(res.is_ok());
    assert!(app.config_path.exists());

    let loaded = CompanionConfig::load_or_default(&app.config_path);
    assert_eq!(loaded.port, 19999);
    assert_eq!(loaded.theme, "light");
    assert_eq!(
        loaded.photocraft_path,
        Some(PathBuf::from("D:/apps/photocraft.exe"))
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_plugin_manager_app_status_and_download_events() {
    let temp_dir = std::env::temp_dir().join(format!(
        "mgr_status_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let prefs_file = temp_dir.join("preferences.json");
    let is_connected = Arc::new(AtomicBool::new(false));

    let mut app = PluginManagerApp::new(vec![], &prefs_file, is_connected);
    app.current_language = Language::En;

    // Test set_status
    app.set_status("Operation complete", false);
    assert_eq!(app.status_text, "Operation complete");
    assert!(!app.status_is_error);
    assert!(app.status_timestamp.is_some());

    // Test download events channel
    let (tx, rx) = std::sync::mpsc::channel();
    app.download_event_rx = Some(rx);

    tx.send(ModelDownloadEvent::Progress(ModelDownloadProgress {
        model_name: "test.onnx".to_string(),
        plugin_id: "test_plugin".to_string(),
        downloaded_bytes: 500,
        total_bytes: 1000,
        speed_bytes_sec: 100.0,
    }))
    .unwrap();

    app.poll_download_events();
    assert!(app.active_download.is_some());
    let active = app.active_download.as_ref().unwrap();
    assert_eq!(active.model_name, "test.onnx");
    assert_eq!(active.downloaded_bytes, 500);

    // Finished event
    tx.send(ModelDownloadEvent::Finished {
        model_name: "test.onnx".to_string(),
    })
    .unwrap();

    app.poll_download_events();
    assert!(app.active_download.is_none());
    assert!(app.status_text.contains("test.onnx"));
    assert!(!app.status_is_error);

    // Error event
    tx.send(ModelDownloadEvent::Error {
        model_name: "err.onnx".to_string(),
        error: "Connection timeout".to_string(),
    })
    .unwrap();

    app.poll_download_events();
    assert!(app.active_download.is_none());
    assert!(app.status_text.contains("err.onnx"));
    assert!(app.status_is_error);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_plugin_manager_app_store_events() {
    let temp_dir = std::env::temp_dir().join(format!("pc_store_app_test_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let prefs_path = temp_dir.join("prefs.json");

    let mut app = PluginManagerApp::new(Vec::new(), &prefs_path, Arc::new(AtomicBool::new(true)));

    // 1. Registry loaded event
    let sample_reg = crate::store::StoreRegistry {
        generated_at: Some("2026-10-13T12:00:00Z".to_string()),
        total_plugins: 1,
        plugins: vec![crate::store::StorePluginEntry {
            id: "test.plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Test description".to_string(),
            author: "Test author".to_string(),
            plugin_type: "no_code".to_string(),
            download_url: "https://example.com/test.pcpl".to_string(),
            icon_url: None,
            models: Vec::new(),
        }],
    };

    app.store_state
        .event_tx
        .send(crate::store::StoreEvent::RegistryLoaded {
            registry: sample_reg.clone(),
            is_offline: false,
        })
        .unwrap();

    app.poll_store_events();
    assert_eq!(
        app.store_state.status,
        crate::store::StoreLoadStatus::Ready { is_offline: false }
    );
    assert_eq!(app.store_state.registry, Some(sample_reg));

    // 2. Plugin installed event
    let dummy_manifest = PluginManifest {
        id: "test.plugin".to_string(),
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "Test".to_string(),
        author: "Tester".to_string(),
        plugin_type: PluginType::NoCode,
        enabled: true,
        icon: None,
        models: Vec::new(),
        runtime_tools: Vec::new(),
        commands: Vec::new(),
        post_commands: Vec::new(),
        sub_actions: Vec::new(),
        input: core_contracts::PluginInputSource::ActiveLayer,
        output_mode: core_contracts::PluginOutputMode::InPlace,
        action_id: None,
        name_ru: None,
        description_ru: None,
        locales: Default::default(),
        entrypoint: None,
        ai: None,
        algorithm: None,
        interactive_ui: None,
        plugin_dir: None,
    };

    app.store_state
        .installing_plugins
        .push("test.plugin".to_string());
    app.store_state
        .event_tx
        .send(crate::store::StoreEvent::PluginInstalled {
            plugin_id: "test.plugin".to_string(),
            manifest: Box::new(dummy_manifest.clone()),
        })
        .unwrap();

    app.poll_store_events();
    assert!(!app
        .store_state
        .installing_plugins
        .contains(&"test.plugin".to_string()));
    assert_eq!(app.manifests.len(), 1);
    assert_eq!(app.manifests[0].id, "test.plugin");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_extract_readme_lines() {
    let sample_md = "# Sample Plugin\n\n[![Badge](url)](link)\n\n**English:**\nEnglish description of the plugin.\n\n### Use Cases:\n- Case 1: Testing something.\n\n---\n\n**Русский:**\nРусское описание плагина.\n\n### Практическое применение:\n- Сценарий 1: Тестирование плагина.\n";

    let en_lines = PluginManagerApp::extract_readme_lines(sample_md, Language::En);
    assert!(en_lines.iter().any(|l| l.contains("# Sample Plugin")));
    assert!(en_lines.iter().any(|l| l.contains("English description")));
    assert!(en_lines
        .iter()
        .any(|l| l.contains("Case 1: Testing something.")));
    assert!(!en_lines.iter().any(|l| l.contains("Русское описание")));

    let ru_lines = PluginManagerApp::extract_readme_lines(sample_md, Language::Ru);
    assert!(ru_lines.iter().any(|l| l.contains("# Sample Plugin")));
    assert!(ru_lines.iter().any(|l| l.contains("Русское описание")));
    assert!(ru_lines.iter().any(|l| l.contains("Сценарий 1")));
    assert!(!ru_lines.iter().any(|l| l.contains("English description")));
}

#[test]
fn test_detect_default_token_prioritizes_configured_path() {
    let temp_dir = std::env::temp_dir().join(format!(
        "detect_token_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);

    let app_dir = temp_dir.join("photocraft_app");
    let app_token_dir = app_dir.join("PhotoCraftData");
    let _ = std::fs::create_dir_all(&app_token_dir);
    let app_token_file = app_token_dir.join("control.token");
    std::fs::write(&app_token_file, "app_token_priority_2").unwrap();

    let prefs_dir = temp_dir.join("prefs_dir");
    let _ = std::fs::create_dir_all(&prefs_dir);
    let prefs_token_file = prefs_dir.join("control.token");
    std::fs::write(&prefs_token_file, "prefs_token_priority_3").unwrap();

    let prefs_file = prefs_dir.join("preferences.json");
    std::fs::write(&prefs_file, "{}").unwrap();

    let detected = detect_default_token(&app_dir.to_string_lossy(), &prefs_file);
    assert_eq!(detected, Some("app_token_priority_2".to_string()));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_is_runtime_tool_installed() {
    #[cfg(target_os = "windows")]
    assert!(is_runtime_tool_installed("cmd"));
    #[cfg(not(target_os = "windows"))]
    assert!(is_runtime_tool_installed("sh"));

    assert!(!is_runtime_tool_installed("nonexistent_tool_12345_xyz"));
}

#[test]
fn test_runtime_tool_cache_ttl() {
    clear_runtime_tool_cache();
    let res1 = is_runtime_tool_installed("nonexistent_tool_test_cache");
    assert!(!res1);
    let res2 = is_runtime_tool_installed("nonexistent_tool_test_cache");
    assert!(!res2);
    clear_runtime_tool_cache();
}

#[test]
fn test_models_installed_cache_ttl() {
    invalidate_models_installed_cache(None);
    let manifest = PluginManifest::default();
    let ok1 = is_plugin_models_installed_cached(&manifest);
    assert!(ok1);
    let ok2 = is_plugin_models_installed_cached(&manifest);
    assert!(ok2);
    invalidate_models_installed_cache(Some(&manifest.id));
}
