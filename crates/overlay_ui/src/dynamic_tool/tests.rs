use super::*;
use core_contracts::*;
#[test]
fn test_find_nearest_named_color_perceptual_accuracy() {
    // Pure red
    assert_eq!(find_nearest_named_color(255, 0, 0, Language::En), "Red");
    assert_eq!(find_nearest_named_color(255, 0, 0, Language::Ru), "Красный");

    // Classical Ultramarine pigment
    assert_eq!(
        find_nearest_named_color(18, 10, 143, Language::En),
        "Ultramarine"
    );
    assert_eq!(
        find_nearest_named_color(18, 10, 143, Language::Ru),
        "Ультрамарин"
    );

    // Pure white
    assert_eq!(
        find_nearest_named_color(255, 255, 255, Language::En),
        "White"
    );
    assert_eq!(
        find_nearest_named_color(255, 255, 255, Language::Ru),
        "Белый"
    );

    // Pure black
    assert_eq!(find_nearest_named_color(0, 0, 0, Language::En), "Black");
    assert_eq!(find_nearest_named_color(0, 0, 0, Language::Ru), "Чёрный");
}

#[test]
fn test_simulate_cvd_color_and_hex() {
    // Normal vision
    assert_eq!(simulate_cvd_hex(255.0, 0.0, 0.0, 0), "#ff0000");

    // Deuteranopia (green blindness)
    let (dr, dg, db) = simulate_cvd_color(0.0, 255.0, 0.0, 1);
    assert!(dr > 80 && dr < 110);
    assert!(dg > 60 && dg < 90);
    assert!(db > 60 && db < 90);
    let d_hex = simulate_cvd_hex(0.0, 255.0, 0.0, 1);
    assert_eq!(d_hex.len(), 7);
    assert!(d_hex.starts_with('#'));

    // Protanopia (red blindness)
    let p_hex = simulate_cvd_hex(255.0, 0.0, 0.0, 2);
    assert_eq!(p_hex.len(), 7);
    assert!(p_hex.starts_with('#'));

    // Tritanopia (blue blindness)
    let t_hex = simulate_cvd_hex(0.0, 0.0, 255.0, 3);
    assert_eq!(t_hex.len(), 7);
    assert!(t_hex.starts_with('#'));
}

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
fn test_chromasense_wcag_and_cvd_standard_compliance() {
    // Light pink #ffb4eb (255, 180, 235) on pure white #ffffff (255, 255, 255)
    let ratio_normal = calculate_wcag_contrast(255.0, 180.0, 235.0, 255.0, 255.0, 255.0);
    assert!(ratio_normal < 3.0);
    assert!((ratio_normal - 1.58).abs() < 0.1);

    // Deuteranopia (profile 1): simulated contrast must also be strictly < 3.0:1 (~1.2:1)
    let (dr1, dg1, db1) = simulate_cvd_color(255.0, 180.0, 235.0, 1);
    let (dr2, dg2, db2) = simulate_cvd_color(255.0, 255.0, 255.0, 1);
    let ratio_deut = calculate_wcag_contrast(
        dr1 as f64, dg1 as f64, db1 as f64, dr2 as f64, dg2 as f64, db2 as f64,
    );
    assert!(ratio_deut < 3.0);
    assert!((ratio_deut - 1.2).abs() < 0.2);

    // Protanopia (profile 2): simulated contrast < 3.0:1 (~1.3:1)
    let (pr1, pg1, pb1) = simulate_cvd_color(255.0, 180.0, 235.0, 2);
    let (pr2, pg2, pb2) = simulate_cvd_color(255.0, 255.0, 255.0, 2);
    let ratio_prot = calculate_wcag_contrast(
        pr1 as f64, pg1 as f64, pb1 as f64, pr2 as f64, pg2 as f64, pb2 as f64,
    );
    assert!(ratio_prot < 3.0);
    assert!((ratio_prot - 1.3).abs() < 0.2);

    // Tritanopia (profile 3): simulated contrast < 3.0:1 (~1.4:1)
    let (tr1, tg1, tb1) = simulate_cvd_color(255.0, 180.0, 235.0, 3);
    let (tr2, tg2, tb2) = simulate_cvd_color(255.0, 255.0, 255.0, 3);
    let ratio_trit = calculate_wcag_contrast(
        tr1 as f64, tg1 as f64, tb1 as f64, tr2 as f64, tg2 as f64, tb2 as f64,
    );
    assert!(ratio_trit < 3.0);
    assert!((ratio_trit - 1.4).abs() < 0.2);
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
fn test_popcraft_3d_rhai_script_state_change() {
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let engine = DynamicToolEngine::new(&[], Some(tx));

    let script = include_str!("../../../../plugins/popcraft_3d/script.rhai");
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

    let script = include_str!("../../../../plugins/selection_mod/script.rhai");
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

    let script = include_str!("../../../../plugins/depth_studio/script.rhai");
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
fn test_rhai_set_badge_and_clear_badge_script() {
    let engine = rhai_bridge::create_rhai_engine(
        None,
        Arc::new(RwLock::new(Language::En)),
        Arc::new(std::sync::Mutex::new(None)),
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

#[test]
fn test_timelapse_studio_ergonomics_and_reset_state_machine() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let engine = DynamicToolEngine::new(&[], Some(tx));

    let script = include_str!("../../../../plugins/timelapse_studio/script.rhai");
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
fn test_single_active_tool_window_policy() {
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

    // 2. Open tool B: Single-Active Tool Policy must automatically close tool A
    engine.toggle_window("tool_b");
    assert!(
        !engine.is_window_open("tool_a"),
        "Tool A should be closed when Tool B opens"
    );
    assert!(engine.is_window_open("tool_b"), "Tool B should be open");

    // 3. Toggle tool B: closes tool B
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
fn test_pending_close_viewports_lifecycle() {
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
    assert!(engine.pending_close_viewports.is_empty());

    // 1. Open tool_a
    engine.toggle_window("tool_a");
    assert!(engine.is_window_open("tool_a"));
    assert!(engine.pending_close_viewports.is_empty());

    // 2. Open tool_b: tool_a is auto-closed and its viewport ID is placed into pending_close_viewports
    engine.toggle_window("tool_b");
    assert!(!engine.is_window_open("tool_a"));
    assert!(engine.is_window_open("tool_b"));
    assert_eq!(
        engine.pending_close_viewports,
        vec![DynamicToolEngine::viewport_id_for_plugin("tool_a")]
    );

    // 3. Clear pending viewports (simulating render pass)
    engine.pending_close_viewports.clear();

    // 4. Close tool_b by toggle
    engine.toggle_window("tool_b");
    assert!(!engine.is_window_open("tool_b"));
    assert_eq!(
        engine.pending_close_viewports,
        vec![DynamicToolEngine::viewport_id_for_plugin("tool_b")]
    );
    engine.pending_close_viewports.clear();

    // 5. Open tool_a again, then close_all_windows
    engine.toggle_window("tool_a");
    engine.pending_close_viewports.clear();
    let closed = engine.close_all_windows();
    assert_eq!(closed, vec!["tool_a".to_string()]);
    assert_eq!(
        engine.pending_close_viewports,
        vec![DynamicToolEngine::viewport_id_for_plugin("tool_a")]
    );
}

#[test]
fn test_timelapse_studio_dynamic_language_switch() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut engine = DynamicToolEngine::new_with_language(&[], Some(tx), Language::En);

    let script = include_str!("../../../../plugins/timelapse_studio/script.rhai");
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
