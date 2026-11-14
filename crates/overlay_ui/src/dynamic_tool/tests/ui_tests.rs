use super::*;

#[test]
fn test_vector_pad_and_preview_initialization() {
    let cfg = InteractiveUiConfig {
        window_title: "3D Extrude".to_string(),
        widgets: vec![
            WidgetNode::VectorPad {
                id: "extrude_vector".to_string(),
                label: "Vector".to_string(),
                label_ru: Some("Вектор".to_string()),
                max_distance: 100.0,
                default_angle_deg: 135.0,
                default_distance: 24.0,
            },
            WidgetNode::ImagePreview {
                id: "preview_pane".to_string(),
                height: 180.0,
            },
        ],
        ..Default::default()
    };

    let state = DynamicWindowState::new(&cfg);
    assert_eq!(
        state.values.get("extrude_vector_angle").unwrap().as_f64(),
        Some(135.0)
    );
    assert_eq!(
        state
            .values
            .get("extrude_vector_distance")
            .unwrap()
            .as_f64(),
        Some(24.0)
    );
    assert_eq!(
        state.values.get("preview_pane_preview").unwrap().as_bool(),
        Some(true)
    );
}

#[test]
fn test_dynamic_tool_preview_storage_methods() {
    let mut engine = DynamicToolEngine::new(&[], None);
    assert!(engine.get_preview_image("plugin.test").is_none());

    let test_img = image::RgbaImage::new(32, 32);
    engine.set_preview_image("plugin.test", test_img.clone(), 0.5);

    let preview = engine.get_preview_image("plugin.test");
    assert!(preview.is_some());
    let (stored_img, scale) = preview.unwrap();
    assert_eq!(*scale, 0.5);
    assert_eq!(stored_img.dimensions(), (32, 32));
}

#[test]

fn test_extract_extrude_params_and_hashing() {
    let mut values = HashMap::new();
    values.insert("vector_angle".to_string(), DynamicValue::Number(45.0));
    values.insert("vector_distance".to_string(), DynamicValue::Number(30.0));
    values.insert(
        "body_start_color".to_string(),
        DynamicValue::String("#ff0000".to_string()),
    );
    values.insert(
        "body_end_color".to_string(),
        DynamicValue::String("#00ff00".to_string()),
    );

    let params = extract_extrude_params(&values);
    assert_eq!(params.angle_deg, 45.0);
    assert_eq!(params.distance, 30.0);
    assert_eq!(params.body_gradient_start, image::Rgba([255, 0, 0, 255]));
    assert_eq!(params.body_gradient_end, image::Rgba([0, 255, 0, 255]));

    let hash1 = hash_extrude_params(&params, 1.0);
    let hash2 = hash_extrude_params(&params, 1.0);
    let hash_scaled = hash_extrude_params(&params, 0.5);
    assert_eq!(hash1, hash2);
    assert_ne!(hash1, hash_scaled);
}

#[test]

fn test_modifier_stack_config_persistence() {
    let manifest_json = r#"{
            "id": "ui.photocraft.plugins.selection_mod",
            "name": "Selection Modifiers",
            "version": "1.0.0",
            "description": "Test",
            "author": "Test",
            "plugin_type": "interactive_ui",
            "interactive_ui": {
                "window_title": "Selection Modifiers",
                "widgets": [
                    {
                        "type": "modifier_stack",
                        "id": "modifiers",
                        "label": "Modifiers",
                        "allow_reordering": true,
                        "items": [
                            { "id": "border", "label": "Border", "min": 0.0, "max": 100.0, "default": 0.0, "enabled": false },
                            { "id": "smooth", "label": "Smooth", "min": 0.0, "max": 50.0, "default": 0.0, "enabled": false },
                            { "id": "expand", "label": "Expand", "min": 0.0, "max": 100.0, "default": 0.0, "enabled": false }
                        ]
                    }
                ]
            }
        }"#;
    let manifest: PluginManifest = serde_json::from_str(manifest_json).unwrap();
    let mut engine = DynamicToolEngine::new(&[manifest], None);

    // Verify default order
    let state = engine
        .states
        .get("ui.photocraft.plugins.selection_mod")
        .unwrap();
    let order = state
        .values
        .get("modifiers_order")
        .unwrap()
        .as_string_list()
        .unwrap();
    assert_eq!(order, &["border", "smooth", "expand"]);

    // Apply config with custom order
    let mut config = core_contracts::CompanionConfig::default();
    config.modifier_stack_orders.insert(
        "ui.photocraft.plugins.selection_mod:modifiers".to_string(),
        vec![
            "expand".to_string(),
            "border".to_string(),
            "smooth".to_string(),
        ],
    );
    engine.apply_saved_modifier_stack_orders(&config);

    let state = engine
        .states
        .get("ui.photocraft.plugins.selection_mod")
        .unwrap();
    let order = state
        .values
        .get("modifiers_order")
        .unwrap()
        .as_string_list()
        .unwrap();
    assert_eq!(order, &["expand", "border", "smooth"]);
}

#[test]
fn test_split_preview_right_layout_mode() {
    let manifest_json = r#"{
        "id": "ui.photocraft.plugins.split_test",
        "name": "Split Test",
        "version": "1.0.0",
        "description": "Test",
        "author": "Test",
        "plugin_type": "interactive_ui",
        "interactive_ui": {
            "window_title": "Split Test Window",
            "layout_mode": "split_preview_right",
            "widgets": [
                {
                    "type": "image_preview",
                    "id": "preview_pane",
                    "height": 180.0
                },
                {
                    "type": "slider",
                    "id": "depth",
                    "label": "Depth",
                    "min": 0.0,
                    "max": 100.0,
                    "default": 10.0
                }
            ]
        }
    }"#;
    let manifest: PluginManifest = serde_json::from_str(manifest_json).unwrap();
    assert_eq!(
        manifest
            .interactive_ui
            .as_ref()
            .unwrap()
            .layout_mode
            .as_deref(),
        Some("split_preview_right")
    );

    let mut engine = DynamicToolEngine::new(&[manifest], None);
    assert_eq!(
        engine
            .states
            .get("ui.photocraft.plugins.split_test")
            .unwrap()
            .default_width,
        580.0
    );

    engine.toggle_window("ui.photocraft.plugins.split_test");
    assert!(engine.is_window_open("ui.photocraft.plugins.split_test"));

    let ctx = egui::Context::default();
    let palette = ThemePalette::for_mode(crate::theme::ThemeMode::Dark);
    let mut out = ctx.run_ui(egui::RawInput::default(), |_ui| {
        engine.render(&ctx, &palette);
    });
    out.textures_delta.clear();
}

#[test]
fn test_single_column_layout_mode() {
    let manifest_json = r#"{
        "id": "ui.photocraft.plugins.single_test",
        "name": "Single Test",
        "version": "1.0.0",
        "description": "Test",
        "author": "Test",
        "plugin_type": "interactive_ui",
        "interactive_ui": {
            "window_title": "Single Test Window",
            "layout_mode": "single_column",
            "widgets": [
                {
                    "type": "slider",
                    "id": "param",
                    "label": "Param",
                    "min": 0.0,
                    "max": 50.0,
                    "default": 5.0
                }
            ]
        }
    }"#;
    let manifest: PluginManifest = serde_json::from_str(manifest_json).unwrap();
    let mut engine = DynamicToolEngine::new(&[manifest], None);
    assert_eq!(
        engine
            .states
            .get("ui.photocraft.plugins.single_test")
            .unwrap()
            .default_width,
        320.0
    );

    engine.toggle_window("ui.photocraft.plugins.single_test");
    assert!(engine.is_window_open("ui.photocraft.plugins.single_test"));

    let ctx = egui::Context::default();
    let palette = ThemePalette::for_mode(crate::theme::ThemeMode::Dark);
    let mut out = ctx.run_ui(egui::RawInput::default(), |_ui| {
        engine.render(&ctx, &palette);
    });
    out.textures_delta.clear();
}

#[test]
fn test_dynamic_badge_state_reactivity() {
    let cfg = InteractiveUiConfig {
        window_title: "Test".to_string(),
        ..Default::default()
    };
    let mut state = DynamicWindowState::new(&cfg);
    assert!(!state.is_open);
    assert!(state.dynamic_badge.is_none());

    // Update from Rhai map with is_open and dynamic_badge
    let mut map = rhai::Map::new();
    map.insert("is_open".into(), rhai::Dynamic::from(true));

    let mut badge_map = rhai::Map::new();
    badge_map.insert("is_pulsing".into(), rhai::Dynamic::from(true));
    badge_map.insert("label".into(), rhai::Dynamic::from("REC".to_string()));
    let rgba_arr: rhai::Array = vec![
        rhai::Dynamic::from(255 as rhai::INT),
        rhai::Dynamic::from(0 as rhai::INT),
        rhai::Dynamic::from(0 as rhai::INT),
        rhai::Dynamic::from(255 as rhai::INT),
    ];
    badge_map.insert(
        "pulse_color_rgba".into(),
        rhai::Dynamic::from_array(rgba_arr),
    );
    map.insert("dynamic_badge".into(), rhai::Dynamic::from(badge_map));

    state.update_from_rhai_map(map);

    assert!(state.is_open);
    assert!(state.dynamic_badge.is_some());
    let badge = state.dynamic_badge.clone().unwrap();
    assert!(badge.is_pulsing);
    assert_eq!(badge.label, Some("REC".to_string()));
    assert_eq!(badge.pulse_color_rgba, [255, 0, 0, 255]);

    // Test to_rhai_map roundtrip
    let exported = state.to_rhai_map();
    assert!(exported.get("is_open").unwrap().as_bool().unwrap());
    assert!(exported.contains_key("dynamic_badge"));

    // Reset badge
    let mut reset_map = rhai::Map::new();
    reset_map.insert("is_open".into(), rhai::Dynamic::from(false));
    reset_map.insert("dynamic_badge".into(), rhai::Dynamic::UNIT);

    state.update_from_rhai_map(reset_map);
    assert!(!state.is_open);
    assert!(state.dynamic_badge.is_none());
}

#[test]

fn test_timelapse_studio_ergonomics_and_reset_state_machine() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let engine = DynamicToolEngine::new(&[], Some(tx));

    let script = include_str!("../../../../../plugins/timelapse_studio/script.rhai");
    let ast = engine
        .rhai_engine
        .compile(script)
        .expect("Timelapse Studio script must compile");

    let mut scope = rhai::Scope::new();

    // 1. Initial on_state_changed initializes state
    let init_map = rhai::Map::new();
    let mut state: rhai::Map = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (init_map,))
        .expect("Init must succeed");
    assert_eq!(state.get("initialized").unwrap().as_bool(), Ok(true));
    assert_eq!(state.get("is_recording").unwrap().as_bool(), Ok(false));
    assert_eq!(state.get("frame_count").unwrap().as_int(), Ok(0));

    // 2. Start recording (Fresh record): frame_count becomes 1, is_recording = true, REC badge set
    state.insert("action_btn_start_record".into(), rhai::Dynamic::from(true));
    state = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (state,))
        .expect("Start record must succeed");
    assert_eq!(state.get("is_recording").unwrap().as_bool(), Ok(true));
    assert_eq!(state.get("frame_count").unwrap().as_int(), Ok(1));
    assert!(state.contains_key("dynamic_badge"));

    // 3. Revision event increments frame_count
    state = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_revision", (42 as rhai::INT, state))
        .expect("on_revision must succeed");
    assert_eq!(state.get("frame_count").unwrap().as_int(), Ok(2));

    // 4. Pause recording: is_recording becomes false, badge cleared, frame_count preserved
    state.insert("action_btn_pause_record".into(), rhai::Dynamic::from(true));
    state = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (state,))
        .expect("Pause must succeed");
    assert_eq!(state.get("is_recording").unwrap().as_bool(), Ok(false));
    assert_eq!(state.get("frame_count").unwrap().as_int(), Ok(2));

    // 5. Resume recording with Record button: preserves frame_count = 2, does not reset to 1
    state.insert("action_btn_start_record".into(), rhai::Dynamic::from(true));
    state = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (state,))
        .expect("Resume must succeed");
    assert_eq!(state.get("is_recording").unwrap().as_bool(), Ok(true));
    assert_eq!(state.get("frame_count").unwrap().as_int(), Ok(2));

    // 6. Stop recording: is_recording becomes false, ready for export
    state.insert("action_btn_stop_record".into(), rhai::Dynamic::from(true));
    state = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (state,))
        .expect("Stop must succeed");
    assert_eq!(state.get("is_recording").unwrap().as_bool(), Ok(false));
    assert_eq!(state.get("frame_count").unwrap().as_int(), Ok(2));

    // 7. Reset session: frame_count reset to 0, session ready for clean restart
    state.insert("action_btn_reset_session".into(), rhai::Dynamic::from(true));
    state = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (state,))
        .expect("Reset must succeed");
    assert_eq!(state.get("is_recording").unwrap().as_bool(), Ok(false));
    assert_eq!(state.get("frame_count").unwrap().as_int(), Ok(0));
}
#[test]
fn test_independent_multi_window_policy() {
    let m1 = PluginManifest {
        id: "tool_a".to_string(),
        name: "Tool A".to_string(),
        interactive_ui: Some(core_contracts::InteractiveUiConfig {
            window_title: "Tool A".to_string(),
            default_width: Some(300.0),
            default_height: Some(200.0),
            ..Default::default()
        }),
        ..Default::default()
    };
    let m2 = PluginManifest {
        id: "tool_b".to_string(),
        name: "Tool B".to_string(),
        interactive_ui: Some(core_contracts::InteractiveUiConfig {
            window_title: "Tool B".to_string(),
            default_width: Some(300.0),
            default_height: Some(200.0),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut engine = DynamicToolEngine::new(&[m1, m2], None);

    assert!(!engine.is_window_open("tool_a"));
    assert!(!engine.is_window_open("tool_b"));

    // 1. Open tool A
    engine.toggle_window("tool_a");
    assert!(engine.is_window_open("tool_a"));
    assert!(!engine.is_window_open("tool_b"));

    // 2. Open tool B: v2.0.3 Multi-Window Architecture ensures tool A remains open independently
    engine.toggle_window("tool_b");
    assert!(
        engine.is_window_open("tool_a"),
        "Tool A should remain open when Tool B opens (independent lifecycle)"
    );
    assert!(engine.is_window_open("tool_b"), "Tool B should be open");

    // 3. Toggle tool A: closes tool A, tool B remains open
    engine.toggle_window("tool_a");
    assert!(!engine.is_window_open("tool_a"));
    assert!(engine.is_window_open("tool_b"));

    // 4. Toggle tool B: closes tool B
    engine.toggle_window("tool_b");
    assert!(!engine.is_window_open("tool_a"));
    assert!(!engine.is_window_open("tool_b"));
}

#[test]
fn test_close_all_windows_and_is_any_window_open() {
    let m1 = PluginManifest {
        id: "tool_1".to_string(),
        name: "Tool 1".to_string(),
        interactive_ui: Some(core_contracts::InteractiveUiConfig {
            window_title: "Tool 1".to_string(),
            default_width: Some(300.0),
            default_height: Some(200.0),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut engine = DynamicToolEngine::new(&[m1], None);
    assert!(!engine.is_any_window_open());

    engine.toggle_window("tool_1");
    assert!(engine.is_any_window_open());
    assert!(engine.is_window_open("tool_1"));

    let closed = engine.close_all_windows();
    assert_eq!(closed, vec!["tool_1".to_string()]);
    assert!(!engine.is_any_window_open());
    assert!(!engine.is_window_open("tool_1"));
}

#[test]
fn test_app_icon_rgba_caching() {
    let sz = 32;
    let icon1 = crate::icons::get_icon_rgba(sz);
    assert!(icon1.is_some());
    let icon2 = crate::icons::get_icon_rgba(sz);
    assert!(icon2.is_some());
    assert_eq!(icon1.unwrap(), icon2.unwrap());
}

#[test]
fn test_declarative_window_closing_lifecycle() {
    let m1 = PluginManifest {
        id: "tool_a".to_string(),
        interactive_ui: Some(InteractiveUiConfig {
            default_width: Some(300.0),
            default_height: Some(200.0),
            ..Default::default()
        }),
        ..Default::default()
    };
    let m2 = PluginManifest {
        id: "tool_b".to_string(),
        interactive_ui: Some(InteractiveUiConfig {
            default_width: Some(300.0),
            default_height: Some(200.0),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut engine = DynamicToolEngine::new(&[m1, m2], None);

    // 1. Open tool_a
    engine.toggle_window("tool_a");
    assert!(engine.is_window_open("tool_a"));

    // 2. Open tool_b: in v2.0.3 architecture, tool_a and tool_b remain open independently
    engine.toggle_window("tool_b");
    assert!(engine.is_window_open("tool_a"));
    assert!(engine.is_window_open("tool_b"));

    // 3. Close tool_b by toggle: tool_b closes, tool_a stays open
    engine.toggle_window("tool_b");
    assert!(!engine.is_window_open("tool_b"));
    assert!(engine.is_window_open("tool_a"));

    // 4. close_all_windows closes tool_a
    let closed = engine.close_all_windows();
    assert_eq!(closed, vec!["tool_a".to_string()]);
    assert!(!engine.is_window_open("tool_a"));
}

#[test]
fn test_execute_tool_async_lifecycle() {
    let jobs = Arc::new(std::sync::Mutex::new(HashMap::new()));
    let engine = rhai_bridge::create_rhai_engine(
        None,
        Arc::new(RwLock::new(Language::En)),
        Arc::new(std::sync::Mutex::new(None)),
        jobs.clone(),
    );

    let script = r#"
        let started = execute_tool_async("dummy_tool", ["--version"], "job_test_1");
        started
    "#;
    let ast = engine.compile(script).unwrap();
    let mut scope = rhai::Scope::new();
    let started: bool = engine.eval_ast_with_scope(&mut scope, &ast).unwrap();
    assert!(started);

    let has_job = {
        let j = jobs.lock().unwrap();
        j.contains_key("job_test_1")
    };
    assert!(has_job);

    // Poll via Rhai
    let poll_script = r#"
        let info = poll_tool_job("job_test_1");
        info["status"]
    "#;
    let poll_ast = engine.compile(poll_script).unwrap();
    let status: String = engine.eval_ast(&poll_ast).unwrap();
    assert!(status == "running" || status == "completed");
}

#[test]
fn test_timelapse_studio_dynamic_language_switch() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut engine = DynamicToolEngine::new_with_language(&[], Some(tx), Language::En);

    let script = include_str!("../../../../../plugins/timelapse_studio/script.rhai");
    let ast = engine
        .rhai_engine
        .compile(script)
        .expect("Timelapse Studio script must compile");

    let mut scope = rhai::Scope::new();

    // 1. Init in English
    let init_map = rhai::Map::new();
    let mut state: rhai::Map = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (init_map,))
        .expect("Init must succeed");

    let session_info_en = state
        .get("session_info")
        .unwrap()
        .clone()
        .into_string()
        .unwrap();
    let stats_info_en = state
        .get("stats_info")
        .unwrap()
        .clone()
        .into_string()
        .unwrap();
    assert!(session_info_en.contains("Status: Idle"));
    assert!(stats_info_en.contains("Recorded frames: 0 | Video length: ~0.0s"));

    // 2. Switch engine language to Russian
    engine.set_language(Language::Ru);

    // 3. Re-run on_state_changed (as happens dynamically via execute_script_state_change)
    state = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (state,))
        .expect("State update after language switch must succeed");

    let session_info_ru = state
        .get("session_info")
        .unwrap()
        .clone()
        .into_string()
        .unwrap();
    let stats_info_ru = state
        .get("stats_info")
        .unwrap()
        .clone()
        .into_string()
        .unwrap();
    assert!(session_info_ru.contains("Статус: Ожидание"));
    assert!(stats_info_ru.contains("Записано кадров: 0 | Длина видео: ~0.0с"));

    // 4. Increment a frame, check that time unit suffix uses Russian "с" instead of hardcoded "s"
    state.insert("action_btn_start_record".into(), rhai::Dynamic::from(true));
    state = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (state,))
        .expect("Start record must succeed");

    let stats_rec_ru = state
        .get("stats_info")
        .unwrap()
        .clone()
        .into_string()
        .unwrap();
    assert!(stats_rec_ru.contains("Примерная длина: ~"));
    assert!(stats_rec_ru.ends_with("с"));
    assert!(!stats_rec_ru.ends_with("s"));

    // 5. Switch back to English, check that time unit suffix flips back to "s"
    engine.set_language(Language::En);
    state.remove("action_btn_start_record");
    state = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (state,))
        .expect("State update back to English must succeed");

    let stats_rec_en = state
        .get("stats_info")
        .unwrap()
        .clone()
        .into_string()
        .unwrap();
    assert!(stats_rec_en.contains("Estimated length: ~"));
    assert!(stats_rec_en.ends_with("s"));
    assert!(!stats_rec_en.ends_with("с"));
}

#[test]
fn test_dynamic_tool_geometry_persistence() {
    let temp_dir = std::env::temp_dir().join(format!(
        "dyn_tool_geom_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let config_path = temp_dir.join("companion_config.json");

    let plugin_id = "ui.photocraft.plugins.geom_test";
    let manifest = PluginManifest {
        id: plugin_id.to_string(),
        name: "Geom Test".to_string(),
        interactive_ui: Some(InteractiveUiConfig {
            window_title: "Geom Test".to_string(),
            default_width: Some(350.0),
            default_height: Some(420.0),
            ..Default::default()
        }),
        ..Default::default()
    };

    // 1. Create engine and set config path
    let mut engine = DynamicToolEngine::new(std::slice::from_ref(&manifest), None);
    engine.set_config_path(config_path.clone());

    // 2. Initial state: last_pos and last_size are None
    let initial_state = engine.states.get(plugin_id).unwrap();
    assert_eq!(initial_state.last_pos, None);
    assert_eq!(initial_state.last_size, None);

    // 3. Save geometry via save_plugin_geometry
    let test_pos = [320.0, 240.0];
    let test_size = [450.0, 520.0];
    engine.save_plugin_geometry(plugin_id, test_pos, test_size);

    // 4. Verify saved in file
    let loaded_cfg = core_contracts::CompanionConfig::load_or_default(&config_path);
    let (saved_pos, saved_size) = loaded_cfg.get_plugin_geometry(plugin_id);
    assert_eq!(saved_pos, Some(test_pos));
    assert_eq!(saved_size, Some(test_size));

    // 5. Create fresh engine instance and verify geometry loads on set_config_path
    let mut fresh_engine = DynamicToolEngine::new(&[manifest], None);
    fresh_engine.set_config_path(config_path);
    let loaded_state = fresh_engine.states.get(plugin_id).unwrap();
    assert_eq!(loaded_state.last_pos, Some(test_pos));
    assert_eq!(loaded_state.last_size, Some(test_size));

    let _ = std::fs::remove_dir_all(&temp_dir);
}
