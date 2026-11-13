use super::*;

#[test]
fn test_rhai_native_helpers_and_language_switching() {
    let manifest = PluginManifest {
        id: "ui.photocraft.plugins.rhai_test".to_string(),
        name: "Rhai Test".to_string(),
        interactive_ui: Some(InteractiveUiConfig {
            window_title: "Title En".to_string(),
            window_title_ru: Some("Заголовок Ру".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut engine = DynamicToolEngine::new_with_language(&[manifest], None, Language::En);
    assert_eq!(engine.current_language(), Language::En);
    assert_eq!(
        engine
            .states
            .get("ui.photocraft.plugins.rhai_test")
            .unwrap()
            .title,
        "Title En"
    );

    // Switch language to Russian
    engine.set_language(Language::Ru);
    assert_eq!(engine.current_language(), Language::Ru);
    assert_eq!(
        engine
            .states
            .get("ui.photocraft.plugins.rhai_test")
            .unwrap()
            .title,
        "Заголовок Ру"
    );

    // Test Rhai execution of registered functions
    let hex_res = engine
        .rhai_engine
        .eval::<String>("rgb_to_hex(0, 255, 0)")
        .unwrap();
    assert_eq!(hex_res, "#00ff00");

    let t_res = engine
        .rhai_engine
        .eval::<String>(r#"t("Hello", "Привет")"#)
        .unwrap();
    assert_eq!(t_res, "Привет");

    let lang_res = engine
        .rhai_engine
        .eval::<String>("current_language()")
        .unwrap();
    assert_eq!(lang_res, "ru");

    let color_name = engine
        .rhai_engine
        .eval::<String>("get_named_color(255, 0, 0)")
        .unwrap();
    assert_eq!(color_name, "Красный");
}

#[test]

fn test_rhai_sandbox_operation_limits() {
    let manifest = PluginManifest {
        id: "ui.photocraft.plugins.infinite_loop".to_string(),
        name: "Infinite Loop Plugin".to_string(),
        interactive_ui: Some(InteractiveUiConfig {
            window_title: "Test".to_string(),
            ..Default::default()
        }),
        ..Default::default()
    };

    let engine = DynamicToolEngine::new_with_language(&[manifest], None, Language::En);
    // Evaluating infinite loop must terminate with operations limit exceeded error without hanging the thread
    let res = engine
        .rhai_engine
        .eval::<()>("let count = 0; while true { count += 1; }");
    assert!(res.is_err(), "Evaluation must fail due to operation limit!");
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("Too many operations") || err_msg.contains("operations"),
        "Error should indicate operation limit exceeded, got: {err_msg}"
    );
}

#[test]

fn test_execute_raster_process_rhai_ipc() {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let engine = DynamicToolEngine::new(&[], Some(tx));

    let script = r#"
            let params = #{
                task: "extrude",
                angle: 120.0,
                distance: 35.0
            };
            execute_raster_process("extrude", params);
        "#;

    engine.rhai_engine.eval::<()>(script).unwrap();

    let action = rx.try_recv().expect("Expected PluginIpcAction");
    assert_eq!(action.command, "raster.process");
    assert_eq!(action.params["task"], "extrude");
    assert_eq!(action.params["params"]["distance"], 35.0);
}

#[test]

fn test_popcraft_3d_rhai_script_state_change() {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let engine = DynamicToolEngine::new(&[], Some(tx));

    let script = include_str!("../../../../../plugins/popcraft_3d/script.rhai");
    let ast = engine
        .rhai_engine
        .compile(script)
        .expect("Script must compile");

    let mut scope = rhai::Scope::new();
    let mut state_map = rhai::Map::new();
    state_map.insert("action_preset_youtube".into(), rhai::Dynamic::from(true));
    state_map.insert("extrude_pad_distance".into(), rhai::Dynamic::from(10.0));

    let res_map: rhai::Map = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (state_map,))
        .expect("on_state_changed must succeed");

    assert_eq!(
        res_map.get("action_preset_youtube").unwrap().as_bool(),
        Ok(false)
    );
    assert_eq!(
        res_map.get("extrude_pad_angle").unwrap().as_float(),
        Ok(120.0)
    );
    assert_eq!(res_map.get("shadow_enabled").unwrap().as_bool(), Ok(true));
    assert_eq!(res_map.get("perspective").unwrap().as_float(), Ok(0.25));
    assert_eq!(
        res_map
            .get("body_start_color")
            .unwrap()
            .clone()
            .into_string(),
        Ok("#f59e0b".to_string())
    );

    // Test Apply trigger with shadow disabled and perspective set
    let mut apply_map = rhai::Map::new();
    apply_map.insert("action_apply".into(), rhai::Dynamic::from(true));
    apply_map.insert("shadow_enabled".into(), rhai::Dynamic::from(false));
    apply_map.insert("perspective".into(), rhai::Dynamic::from(0.3));
    apply_map.insert("extrude_pad_angle".into(), rhai::Dynamic::from(45.0));
    apply_map.insert("extrude_pad_distance".into(), rhai::Dynamic::from(20.0));
    apply_map.insert(
        "body_start_color".into(),
        rhai::Dynamic::from("#ff0000".to_string()),
    );
    apply_map.insert(
        "body_end_color".into(),
        rhai::Dynamic::from("#0000ff".to_string()),
    );
    apply_map.insert("fog_as_separate_layer".into(), rhai::Dynamic::from(true));

    let _: rhai::Map = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (apply_map,))
        .expect("Apply must succeed");

    let action = rx.try_recv().expect("Apply must send IPC action");
    assert_eq!(action.command, "raster.process");
    assert_eq!(action.params["task"], "extrude_2_5d");
    assert_eq!(action.params["params"]["angle_deg"], 45.0);
    assert_eq!(action.params["params"]["shadow_opacity"], 0.0);
    assert_eq!(action.params["params"]["perspective"], 0.3);
}

#[test]
fn test_selection_modifier_stack_rhai_script() {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let engine = DynamicToolEngine::new(&[], Some(tx));

    let script = include_str!("../../../../../plugins/selection_mod/script.rhai");
    let ast = engine
        .rhai_engine
        .compile(script)
        .expect("Selection mod script must compile");

    let mut scope = rhai::Scope::new();

    // 1. Tool window opened with active selection snapshots working base
    let mut open_map = rhai::Map::new();
    open_map.insert("is_open".into(), rhai::Dynamic::from(true));
    let res_map1: rhai::Map = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (open_map,))
        .expect("First on_state_changed must succeed");

    assert_eq!(res_map1.get("base_saved").unwrap().as_bool(), Ok(true));
    let save_base_action = rx.try_recv().expect("Must save base working selection");
    assert_eq!(save_base_action.command, "select.saveSelection");
    assert_eq!(save_base_action.params["name"], "__pc_companion_base_sel");
    assert_eq!(
        save_base_action.params["channel"],
        "__pc_companion_base_sel"
    );
    assert_eq!(save_base_action.params["operation"], "replace");
    assert!(rx.try_recv().is_err(), "Only base selection must be saved");

    // 2. Reset action (Zero Data-Loss UX: resets UI modifiers without loading or altering canvas)
    let mut reset_map = res_map1.clone();
    reset_map.insert("contract_enabled".into(), rhai::Dynamic::from(true));
    reset_map.insert("contract_value".into(), rhai::Dynamic::from(10.0));
    reset_map.insert("action_reset".into(), rhai::Dynamic::from(true));
    let res_map2: rhai::Map = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (reset_map,))
        .expect("Reset on_state_changed must succeed");

    assert_eq!(res_map2.get("action_reset").unwrap().as_bool(), Ok(false));
    assert_eq!(
        res_map2.get("contract_enabled").unwrap().as_bool(),
        Ok(false)
    );
    assert_eq!(res_map2.get("contract_value").unwrap().as_float(), Ok(0.0));

    // Canvas selection is saved as base, NOT loaded/destroyed
    let reset_sync_action = rx.try_recv().expect("Must sync canvas to base on reset");
    assert_eq!(reset_sync_action.command, "select.saveSelection");
    assert_eq!(
        reset_sync_action.params["channel"],
        "__pc_companion_base_sel"
    );
    assert_eq!(reset_sync_action.params["operation"], "replace");
    assert!(
        rx.try_recv().is_err(),
        "select.loadSelection must NOT be called on reset (Zero Data-Loss)"
    );

    // 3. Apply custom modifier stack order
    let mut apply_map = res_map2;
    apply_map.insert("action_apply".into(), rhai::Dynamic::from(true));
    // Custom order: contract -> smooth -> feather
    let custom_order: rhai::Array = vec![
        rhai::Dynamic::from("contract".to_string()),
        rhai::Dynamic::from("smooth".to_string()),
        rhai::Dynamic::from("feather".to_string()),
    ];
    apply_map.insert(
        "modifiers_order".into(),
        rhai::Dynamic::from_array(custom_order),
    );
    apply_map.insert("contract_enabled".into(), rhai::Dynamic::from(true));
    apply_map.insert("contract_value".into(), rhai::Dynamic::from(8.0));
    apply_map.insert("smooth_enabled".into(), rhai::Dynamic::from(true));
    apply_map.insert("smooth_value".into(), rhai::Dynamic::from(4.0));
    apply_map.insert("feather_enabled".into(), rhai::Dynamic::from(false));
    apply_map.insert("feather_value".into(), rhai::Dynamic::from(2.0));

    let res_map3: rhai::Map = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (apply_map,))
        .expect("Apply on_state_changed must succeed");

    assert_eq!(res_map3.get("action_apply").unwrap().as_bool(), Ok(false));
    assert_eq!(res_map3.get("base_saved").unwrap().as_bool(), Ok(true));

    let load_base_action = rx.try_recv().expect("Must load base working selection");
    assert_eq!(load_base_action.command, "select.loadSelection");
    assert_eq!(
        load_base_action.params["channel"],
        "__pc_companion_base_sel"
    );
    assert_eq!(load_base_action.params["operation"], "new");

    let mod1_action = rx
        .try_recv()
        .expect("Must execute first modifier (contract)");
    assert_eq!(mod1_action.command, "select.modify.contract");
    assert_eq!(mod1_action.params["radius"], 8.0);
    assert_eq!(mod1_action.params["applyAtCanvasBounds"], false);

    let mod2_action = rx
        .try_recv()
        .expect("Must execute second modifier (smooth)");
    assert_eq!(mod2_action.command, "select.modify.smooth");
    assert_eq!(mod2_action.params["radius"], 4.0);
    assert_eq!(mod2_action.params["applyAtCanvasBounds"], false);

    // 4. Test unchecking commits current selection as new working base and zeroes radius
    let mut uncheck_map = res_map3;
    uncheck_map.insert("contract_enabled".into(), rhai::Dynamic::from(false));
    let res_map4: rhai::Map = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (uncheck_map,))
        .expect("Uncheck on_state_changed must succeed");
    assert_eq!(res_map4.get("contract_value").unwrap().as_float(), Ok(0.0));
    let uncheck_commit = rx.try_recv().expect("Must save base on uncheck");
    assert_eq!(uncheck_commit.command, "select.saveSelection");
    assert_eq!(uncheck_commit.params["channel"], "__pc_companion_base_sel");

    // 5. Test external selection reactivity and bounds change in PhotoCraft
    let ext_map = res_map4.clone();
    let bounds_1: rhai::Array = vec![
        rhai::Dynamic::from(10 as rhai::INT),
        rhai::Dynamic::from(20 as rhai::INT),
        rhai::Dynamic::from(100 as rhai::INT),
        rhai::Dynamic::from(200 as rhai::INT),
    ];
    let res_sel1: rhai::Map = engine
        .rhai_engine
        .call_fn(
            &mut scope,
            &ast,
            "on_selection_state",
            (true, rhai::Dynamic::from_array(bounds_1), ext_map),
        )
        .expect("on_selection_state 3-args must succeed");
    assert_eq!(res_sel1.get("base_saved").unwrap().as_bool(), Ok(true));

    // When bounds change externally in PhotoCraft, base_saved becomes false to snapshot new contour on apply
    let bounds_2: rhai::Array = vec![
        rhai::Dynamic::from(30 as rhai::INT),
        rhai::Dynamic::from(40 as rhai::INT),
        rhai::Dynamic::from(150 as rhai::INT),
        rhai::Dynamic::from(250 as rhai::INT),
    ];
    let res_sel2: rhai::Map = engine
        .rhai_engine
        .call_fn(
            &mut scope,
            &ast,
            "on_selection_state",
            (true, rhai::Dynamic::from_array(bounds_2), res_sel1),
        )
        .expect("on_selection_state changed bounds must succeed");
    assert_eq!(res_sel2.get("base_saved").unwrap().as_bool(), Ok(false));

    // When selection is deselected in PhotoCraft
    let res_sel3: rhai::Map = engine
        .rhai_engine
        .call_fn(
            &mut scope,
            &ast,
            "on_selection_state",
            (false, rhai::Dynamic::UNIT, res_sel2),
        )
        .expect("on_selection_state false must succeed");
    assert_eq!(res_sel3.get("base_saved").unwrap().as_bool(), Ok(false));

    // 6. Test window closing cleanup deletes temporary alpha channel
    let mut close_map = res_map4;
    close_map.insert("is_open".into(), rhai::Dynamic::from(false));
    let res_map5: rhai::Map = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (close_map,))
        .expect("Close on_state_changed must succeed");
    assert_eq!(res_map5.get("base_saved").unwrap().as_bool(), Ok(false));
    let del1 = rx.try_recv().expect("Must delete base_sel channel");
    assert_eq!(del1.command, "channel.delete");
    assert_eq!(del1.params["channel"], "__pc_companion_base_sel");
    assert!(rx.try_recv().is_err(), "Only base_sel must be deleted");
}

#[test]
fn test_depth_studio_rhai_script_presets_and_actions() {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let engine = DynamicToolEngine::new(&[], Some(tx));

    let script = include_str!("../../../../../plugins/depth_studio/script.rhai");
    let ast = engine
        .rhai_engine
        .compile(script)
        .expect("Depth Studio script must compile");

    let mut scope = rhai::Scope::new();

    // 1. Test Morning Mist preset triggers fog_enabled and parameters
    let mut mist_map = rhai::Map::new();
    mist_map.insert("action_preset_mist".into(), rhai::Dynamic::from(true));
    mist_map.insert("fog_enabled".into(), rhai::Dynamic::from(false));
    let res_map1: rhai::Map = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (mist_map,))
        .expect("Mist preset on_state_changed must succeed");

    assert_eq!(res_map1.get("fog_enabled").unwrap().as_bool(), Ok(true));
    assert_eq!(
        res_map1.get("fog_color").unwrap().clone().into_string(),
        Ok("#c8d6e5".to_string())
    );
    assert_eq!(res_map1.get("fog_density").unwrap().as_float(), Ok(55.0));
    assert_eq!(res_map1.get("fog_start").unwrap().as_float(), Ok(10.0));
    assert_eq!(
        res_map1.get("action_preset_mist").unwrap().as_bool(),
        Ok(false)
    );

    // 2. Test Portrait Bokeh preset triggers bokeh_enabled and parameters
    let mut portrait_map = rhai::Map::new();
    portrait_map.insert("action_preset_portrait".into(), rhai::Dynamic::from(true));
    portrait_map.insert("bokeh_enabled".into(), rhai::Dynamic::from(false));
    let res_map2: rhai::Map = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (portrait_map,))
        .expect("Portrait preset on_state_changed must succeed");

    assert_eq!(res_map2.get("bokeh_enabled").unwrap().as_bool(), Ok(true));
    assert_eq!(res_map2.get("blur_radius").unwrap().as_float(), Ok(18.0));
    assert_eq!(res_map2.get("dof_width").unwrap().as_float(), Ok(20.0));
    assert_eq!(
        res_map2.get("action_preset_portrait").unwrap().as_bool(),
        Ok(false)
    );

    // 3. Test Refresh Scene action dispatches IPC raster.process
    let mut refresh_map = rhai::Map::new();
    refresh_map.insert("action_refresh_scene".into(), rhai::Dynamic::from(true));
    let res_map3: rhai::Map = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (refresh_map,))
        .expect("Refresh scene on_state_changed must succeed");

    assert_eq!(
        res_map3.get("action_refresh_scene").unwrap().as_bool(),
        Ok(false)
    );
    let refresh_action = rx
        .try_recv()
        .expect("Refresh scene must dispatch IPC action");
    assert_eq!(refresh_action.command, "raster.process");
    assert_eq!(refresh_action.params["task"], "refresh_preview");
    assert_eq!(
        refresh_action.params["plugin_id"],
        "ui.photocraft.plugins.depth_studio"
    );

    // 4. Test Apply to PhotoCraft action dispatches IPC raster.process lens_blur
    let mut apply_map = rhai::Map::new();
    apply_map.insert("action_apply_to_canvas".into(), rhai::Dynamic::from(true));
    apply_map.insert("fog_enabled".into(), rhai::Dynamic::from(true));
    apply_map.insert(
        "fog_color".into(),
        rhai::Dynamic::from("#c8d6e5".to_string()),
    );
    apply_map.insert("fog_density".into(), rhai::Dynamic::from(65.0));
    apply_map.insert("fog_start".into(), rhai::Dynamic::from(15.0));
    apply_map.insert("fog_falloff".into(), rhai::Dynamic::from(1.5));
    apply_map.insert("fog_blend_mode".into(), rhai::Dynamic::from(0 as rhai::INT));
    apply_map.insert("bokeh_enabled".into(), rhai::Dynamic::from(true));
    apply_map.insert("focus_distance".into(), rhai::Dynamic::from(128.0));
    apply_map.insert("dof_width".into(), rhai::Dynamic::from(30.0));
    apply_map.insert("blur_radius".into(), rhai::Dynamic::from(14.0));
    apply_map.insert("specular_bloom".into(), rhai::Dynamic::from(25.0));
    apply_map.insert("output_mode".into(), rhai::Dynamic::from(0 as rhai::INT));

    let res_map4: rhai::Map = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (apply_map,))
        .expect("Apply to canvas on_state_changed must succeed");

    assert_eq!(
        res_map4.get("action_apply_to_canvas").unwrap().as_bool(),
        Ok(false)
    );
    let apply_action = rx
        .try_recv()
        .expect("Apply to canvas must dispatch IPC action");
    assert_eq!(apply_action.command, "raster.process");
    assert_eq!(apply_action.params["task"], "lens_blur");
    assert_eq!(
        apply_action.params["plugin_id"],
        "ui.photocraft.plugins.depth_studio"
    );
    assert_eq!(apply_action.params["fog_as_separate_layer"], true);
    assert_eq!(apply_action.params["params"]["fog_enabled"], true);
    assert_eq!(apply_action.params["params"]["bokeh_enabled"], true);
    assert_eq!(apply_action.params["params"]["blur_radius"], 14.0);
    // fog_blend_mode is 0 (Normal), so no extra layer.setProps action was dispatched
    assert!(rx.try_recv().is_err());

    // 5. Test Apply with fog_blend_mode == 1 (Screen) dispatches layer.setProps {"blend": "Screen"}
    let mut screen_map = rhai::Map::new();
    screen_map.insert("action_apply_to_canvas".into(), rhai::Dynamic::from(true));
    screen_map.insert("fog_enabled".into(), rhai::Dynamic::from(true));
    screen_map.insert("fog_blend_mode".into(), rhai::Dynamic::from(1 as rhai::INT));
    screen_map.insert("fog_as_separate_layer".into(), rhai::Dynamic::from(true));
    let _res_screen: rhai::Map = engine
        .rhai_engine
        .call_fn(&mut scope, &ast, "on_state_changed", (screen_map,))
        .expect("Apply with Screen blend mode must succeed");

    let act1 = rx.try_recv().expect("raster.process must be dispatched");
    assert_eq!(act1.command, "raster.process");
    let act2 = rx
        .try_recv()
        .expect("layer.setProps with Screen blend must follow immediately");
    assert_eq!(act2.command, "layer.setProps");
    assert_eq!(act2.params["blend"], "Screen");

    // 6. Test full cycle with DynamicWindowState and execute_script_state_change
    let mut state = DynamicWindowState {
        values: HashMap::new(),
        is_open: true,
        icons_calibrated: false,
        title: "Depth Studio".to_string(),
        default_width: 780.0,
        default_height: 560.0,
        role_mask: None,
        dynamic_badge: None,
        icon_cache: crate::icons::SvgIconCache::new(),
        last_pos: None,
        last_size: None,
        just_opened: false,
    };
    state
        .values
        .insert("fog_enabled".to_string(), DynamicValue::Bool(false));
    state
        .values
        .insert("action_preset_mist".to_string(), DynamicValue::Bool(true));

    rhai_bridge::execute_script_state_change(&engine.rhai_engine, &ast, &mut state);

    assert_eq!(
        state.values.get("fog_enabled"),
        Some(&DynamicValue::Bool(true))
    );
    assert_eq!(
        state.values.get("fog_density"),
        Some(&DynamicValue::Number(55.0))
    );
    assert_eq!(
        state.values.get("action_preset_mist"),
        Some(&DynamicValue::Bool(false))
    );
}

#[test]

fn test_rhai_set_badge_and_clear_badge_script() {
    let engine = rhai_bridge::create_rhai_engine(
        None,
        Arc::new(RwLock::new(Language::En)),
        Arc::new(std::sync::Mutex::new(None)),
        Arc::new(std::sync::Mutex::new(HashMap::new())),
    );

    let script = r#"
        state.set_badge(true, "REC", [239, 68, 68, 255]);
        return state;
    "#;

    let ast = engine.compile(script).unwrap();
    let cfg = InteractiveUiConfig::default();
    let mut state = DynamicWindowState::new(&cfg);

    rhai_bridge::execute_script_state_change(&engine, &ast, &mut state);
    assert!(state.dynamic_badge.is_some());
    let badge = state.dynamic_badge.as_ref().unwrap();
    assert!(badge.is_pulsing);
    assert_eq!(badge.label.as_deref(), Some("REC"));
    assert_eq!(badge.pulse_color_rgba, [239, 68, 68, 255]);

    let clear_script = r#"
        state.clear_badge();
        return state;
    "#;
    let clear_ast = engine.compile(clear_script).unwrap();
    rhai_bridge::execute_script_state_change(&engine, &clear_ast, &mut state);
    assert!(state.dynamic_badge.is_none());
}

#[test]
fn test_rhai_unit_fallback_in_state_change() {
    let engine = rhai_bridge::create_rhai_engine(
        None,
        Arc::new(RwLock::new(Language::En)),
        Arc::new(std::sync::Mutex::new(None)),
        Arc::new(std::sync::Mutex::new(HashMap::new())),
    );

    // Function doesn't have return state; (returns ())
    let script = r#"
        fn on_state_changed(state) {
            state["counter"] = 42;
        }
    "#;

    let ast = engine.compile(script).unwrap();
    let cfg = InteractiveUiConfig::default();
    let mut state = DynamicWindowState::new(&cfg);

    rhai_bridge::execute_script_state_change(&engine, &ast, &mut state);
    // Unit fallback preserves state from scope or doesn't panic
    assert!(
        !state.values.contains_key("counter")
            || state.values.get("counter") == Some(&DynamicValue::Number(42.0))
    );
}

#[test]
fn test_rhai_float_to_int_string() {
    let engine = rhai_bridge::create_rhai_engine(
        None,
        Arc::new(RwLock::new(Language::En)),
        Arc::new(std::sync::Mutex::new(None)),
        Arc::new(std::sync::Mutex::new(HashMap::new())),
    );
    let script = r#"
        let count = 1.0;
        let int_val = count.to_int();
        let s = int_val.to_string();
        while s.len() < 6 {
            s = "0" + s;
        }
        s
    "#;
    let res: String = engine.eval(script).unwrap();
    assert_eq!(res, "000001");
}
