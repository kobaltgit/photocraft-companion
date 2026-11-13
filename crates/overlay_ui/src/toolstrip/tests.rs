use super::*;

fn create_test_buttons(count: usize) -> Vec<ButtonConfig> {
    (0..count)
        .map(|i| ButtonConfig {
            label: format!("Tool {}", i + 1),
            action: format!("action_{}", i + 1),
            icon: if i == 0 {
                Some("wand".to_string())
            } else {
                Some("bw".to_string())
            },
            icon_path: None,
            plugin_dir: None,
            sub_actions: Vec::new(),
            active_sub_action: None,
            name_ru: None,
            locales: Default::default(),
            requires_selection: false,
        })
        .collect()
}

#[test]
fn test_vertical_toolstrip_geometry_no_clipping() {
    for count in [1, 2, 4, 8, 9, 14, 16, 18] {
        let ctx = egui::Context::default();
        let buttons = create_test_buttons(count);
        let mut app = ToolstripApp::new(buttons, ThemeMode::Dark);
        app.is_vertical = true;

        let expected_size = app.calculate_size();
        let expected_cols = count.div_ceil(MAX_BUTTONS_PER_TIER).max(1) as f32;
        let expected_w = 50.0 + (expected_cols - 1.0) * BUTTON_SLOT_SIZE_V;
        assert_eq!(expected_size.x, expected_w);
        assert!(expected_size.y >= (count.min(MAX_BUTTONS_PER_TIER) as f32) * 36.0 + 58.0);

        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, expected_size)),
            ..Default::default()
        };

        let mut full_output = ctx.run_ui(raw_input, |ctx| {
            let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, expected_size);
            let mut ui = egui::Ui::new(
                ctx.clone(),
                egui::Id::new("test_vertical"),
                egui::UiBuilder::new().max_rect(rect),
            );
            app.render(&mut ui, &ThemePalette::for_mode(ThemeMode::Dark));

            let used_rect = ui.min_rect();
            // Ensure content never overflows the window bounds in vertical mode
            assert!(
                used_rect.max.y <= expected_size.y,
                "Vertical mode clipped at bottom for count {}: used y {} > expected y {}",
                count,
                used_rect.max.y,
                expected_size.y
            );
            assert!(
                used_rect.max.x <= expected_size.x,
                "Vertical mode clipped at right for count {}: used x {} > expected x {}",
                count,
                used_rect.max.x,
                expected_size.x
            );
        });
        full_output.textures_delta.clear();
    }
}

#[test]
fn test_horizontal_toolstrip_geometry_no_clipping() {
    for count in [1, 2, 4, 8, 9, 14, 16, 18] {
        let ctx = egui::Context::default();
        let buttons = create_test_buttons(count);
        let mut app = ToolstripApp::new(buttons, ThemeMode::Dark);
        app.is_vertical = false;

        let expected_size = app.calculate_size();
        let expected_rows = count.div_ceil(MAX_BUTTONS_PER_TIER).max(1) as f32;
        let expected_h = 78.0 + (expected_rows - 1.0) * BUTTON_SLOT_SIZE_V;
        assert_eq!(expected_size.y, expected_h);

        let raw_input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, expected_size)),
            ..Default::default()
        };

        let mut full_output = ctx.run_ui(raw_input, |ctx| {
            let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, expected_size);
            let mut ui = egui::Ui::new(
                ctx.clone(),
                egui::Id::new("test_horizontal"),
                egui::UiBuilder::new().max_rect(rect),
            );
            app.render(&mut ui, &ThemePalette::for_mode(ThemeMode::Dark));

            let used_rect = ui.min_rect();
            assert!(
                used_rect.max.y <= expected_size.y,
                "Horizontal mode clipped at bottom for count {}: used y {} > expected y {}",
                count,
                used_rect.max.y,
                expected_size.y
            );
            assert!(
                used_rect.max.x <= expected_size.x,
                "Horizontal mode clipped at right for count {}: used x {} > expected x {}",
                count,
                used_rect.max.x,
                expected_size.x
            );
        });
        full_output.textures_delta.clear();
    }
}

#[test]
fn test_toolstrip_size_scaling() {
    // 0 buttons defaults to 1 button minimum
    let app_empty_h = ToolstripApp::new(vec![], ThemeMode::Dark);
    assert_eq!(app_empty_h.calculate_size().y, 78.0);
    assert!(app_empty_h.calculate_size().x >= 116.0);

    let mut app_empty_v = ToolstripApp::new(vec![], ThemeMode::Dark);
    app_empty_v.is_vertical = true;
    assert_eq!(app_empty_v.calculate_size().x, 50.0);
    assert_eq!(app_empty_v.calculate_size().y, 1.0 * 36.0 + 68.0);

    // 5 buttons (single tier)
    let buttons5 = create_test_buttons(5);
    let mut app5 = ToolstripApp::new(buttons5, ThemeMode::Light);
    assert_eq!(app5.calculate_size().y, 78.0);
    assert_eq!(app5.calculate_size().x, (5.0f32 * 38.0 + 16.0).max(116.0));

    app5.is_vertical = true;
    assert_eq!(app5.calculate_size().x, 50.0);
    assert_eq!(app5.calculate_size().y, 5.0 * 36.0 + 68.0);

    // 8 buttons (limit of 1 tier)
    let buttons8 = create_test_buttons(8);
    let mut app8 = ToolstripApp::new(buttons8, ThemeMode::Dark);
    assert_eq!(app8.calculate_size().x, 8.0 * 38.0 + 16.0);
    assert_eq!(app8.calculate_size().y, 78.0);
    app8.is_vertical = true;
    assert_eq!(app8.calculate_size().x, 50.0);
    assert_eq!(app8.calculate_size().y, 8.0 * 36.0 + 68.0);

    // 9 buttons (spans to 2 columns in vertical, 2 rows in horizontal)
    let buttons9 = create_test_buttons(9);
    let mut app9 = ToolstripApp::new(buttons9, ThemeMode::Dark);
    assert_eq!(app9.calculate_size().x, 8.0 * 38.0 + 16.0);
    assert_eq!(app9.calculate_size().y, 78.0 + 36.0);
    app9.is_vertical = true;
    assert_eq!(app9.calculate_size().x, 50.0 + 36.0);
    assert_eq!(app9.calculate_size().y, 8.0 * 36.0 + 68.0);

    // 16 buttons (fills 2 tiers completely)
    let buttons16 = create_test_buttons(16);
    let mut app16 = ToolstripApp::new(buttons16, ThemeMode::Dark);
    assert_eq!(app16.calculate_size().x, 8.0 * 38.0 + 16.0);
    assert_eq!(app16.calculate_size().y, 78.0 + 36.0);
    app16.is_vertical = true;
    assert_eq!(app16.calculate_size().x, 50.0 + 36.0);
    assert_eq!(app16.calculate_size().y, 8.0 * 36.0 + 68.0);

    // 17 buttons (spans to 3rd column / 3rd row)
    let buttons17 = create_test_buttons(17);
    let mut app17 = ToolstripApp::new(buttons17, ThemeMode::Dark);
    assert_eq!(app17.calculate_size().x, 8.0 * 38.0 + 16.0);
    assert_eq!(app17.calculate_size().y, 78.0 + 72.0);
    app17.is_vertical = true;
    assert_eq!(app17.calculate_size().x, 50.0 + 72.0);
    assert_eq!(app17.calculate_size().y, 8.0 * 36.0 + 68.0);
}

#[test]
fn test_toolstrip_state_defaults() {
    let app = ToolstripApp::new(vec![], ThemeMode::Dark);
    assert!(!app.is_vertical);
    assert!(app.pending_action.is_none());
    assert!(!app.open_manager_requested);
    assert!(!app.hide_requested);
    assert!(app.hovered_tooltip.is_none());
    assert!(!app.orientation_changed);
    assert!(app.suppress_click_for.is_none());
    assert!(app.long_press_fired_for.is_none());
}

#[test]
fn test_toolstrip_with_orientation() {
    let app_v = ToolstripApp::new(vec![], ThemeMode::Dark).with_orientation(true);
    assert!(app_v.is_vertical);
    assert_eq!(app_v.calculate_size().x, 50.0);

    let app_h = ToolstripApp::new(vec![], ThemeMode::Dark).with_orientation(false);
    assert!(!app_h.is_vertical);
    assert_eq!(app_h.calculate_size().y, 78.0);
}

#[test]
fn test_toolstrip_sub_actions_state() {
    let sub1 = core_contracts::SubAction {
        id: "cutout".to_string(),
        name: "Cutout".to_string(),
        name_ru: None,
        locales: Default::default(),
        icon: Some("cutout.svg".to_string()),
        post_commands: vec![],
        script: None,
    };
    let sub2 = core_contracts::SubAction {
        id: "mask".to_string(),
        name: "Layer Mask".to_string(),
        name_ru: None,
        locales: Default::default(),
        icon: Some("mask.svg".to_string()),
        post_commands: vec![],
        script: None,
    };

    let btn = ButtonConfig {
        label: "Remove BG".to_string(),
        action: "ai.photocraft.plugins.remove_bg".to_string(),
        icon: Some("wand".to_string()),
        icon_path: None,
        plugin_dir: None,
        sub_actions: vec![sub1, sub2],
        active_sub_action: Some("mask".to_string()),
        name_ru: None,
        locales: Default::default(),
        requires_selection: false,
    };

    let app = ToolstripApp::new(vec![btn], ThemeMode::Dark);
    assert_eq!(app.buttons[0].sub_actions.len(), 2);
    assert_eq!(app.buttons[0].active_sub_action.as_deref(), Some("mask"));
}

#[test]
fn test_sub_action_icon_path_resolution() {
    let temp_dir = std::env::temp_dir().join(format!(
        "sub_icon_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let mask_svg = temp_dir.join("mask.svg");
    std::fs::write(&mask_svg, "<svg></svg>").unwrap();

    let sub = core_contracts::SubAction {
        id: "mask".to_string(),
        name: "Layer Mask".to_string(),
        name_ru: None,
        locales: Default::default(),
        icon: Some("mask.svg".to_string()),
        post_commands: vec![],
        script: None,
    };

    let btn = ButtonConfig {
        label: "Remove BG".to_string(),
        action: "ai.photocraft.plugins.remove_bg".to_string(),
        icon: Some("wand".to_string()),
        icon_path: None,
        plugin_dir: Some(temp_dir.clone()),
        sub_actions: vec![sub],
        active_sub_action: Some("mask".to_string()),
        name_ru: None,
        locales: Default::default(),
        requires_selection: false,
    };

    let resolved = btn.effective_icon_path();
    assert_eq!(resolved, Some(mask_svg));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_dynamic_badge_lifecycle() {
    let btn = ButtonConfig {
        label: "Harmonizer".to_string(),
        action: "ui.harmonizer".to_string(),
        ..Default::default()
    };
    let mut app = ToolstripApp::new(vec![btn], ThemeMode::Dark);
    assert!(app.dynamic_badges.is_empty());

    let badge = core_contracts::DynamicBadgeState {
        is_pulsing: true,
        pulse_color_rgba: [255, 0, 0, 255],
        label: Some("REC".to_string()),
    };

    app.set_dynamic_badge("ui.harmonizer", Some(badge.clone()));
    assert_eq!(app.dynamic_badges.get("ui.harmonizer"), Some(&badge));

    app.set_dynamic_badge("ui.harmonizer", None);
    assert!(!app.dynamic_badges.contains_key("ui.harmonizer"));
}
