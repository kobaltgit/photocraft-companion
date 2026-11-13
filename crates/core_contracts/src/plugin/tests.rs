use super::*;

#[test]
fn test_plugin_result() {
    let ok_res = PluginResult::ok("Completed");
    assert!(ok_res.success);
    assert_eq!(ok_res.message, "Completed");
    assert!(ok_res.data.is_none());

    let data = serde_json::json!({ "output": 42 });
    let ok_data_res = PluginResult::ok_with_data("Done", data.clone());
    assert!(ok_data_res.success);
    assert_eq!(ok_data_res.message, "Done");
    assert_eq!(ok_data_res.data, Some(data));

    let err_res = PluginResult::err("Failed");
    assert!(!err_res.success);
    assert_eq!(err_res.message, "Failed");
    assert!(err_res.data.is_none());
}

#[test]
fn test_plugin_type_default() {
    assert_eq!(PluginType::default(), PluginType::NoCode);
}

#[test]
fn test_plugin_manifest_model_installed() {
    let temp_dir = std::env::temp_dir().join(format!(
        "manifest_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let models_dir = temp_dir.join("models");
    std::fs::create_dir_all(&models_dir).unwrap();

    let model_file = models_dir.join("test_model.onnx");
    std::fs::write(&model_file, b"dummy onnx").unwrap();

    let manifest = PluginManifest {
        id: "test.plugin".to_string(),
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "A test plugin".to_string(),
        author: "Tester".to_string(),
        plugin_type: PluginType::NativeAi,
        icon: None,
        commands: vec![],
        models: vec![
            ModelRequirement {
                role: Some("primary".to_string()),
                name: "test_model.onnx".to_string(),
                required: true,
                size_mb: Some(1),
                url: None,
                sha256: None,
            },
            ModelRequirement {
                role: Some("secondary".to_string()),
                name: "optional_model.onnx".to_string(),
                required: false,
                size_mb: None,
                url: None,
                sha256: None,
            },
        ],
        action_id: None,
        entrypoint: None,
        ai: None,
        enabled: true,
        plugin_dir: Some(temp_dir.clone()),
        ..Default::default()
    };

    assert!(manifest.is_model_installed(&manifest.models[0]));
    assert!(!manifest.is_model_installed(&manifest.models[1]));
    assert_eq!(
        manifest.model_by_role("primary").map(|m| m.name.as_str()),
        Some("test_model.onnx")
    );
    assert_eq!(
        manifest.model_by_role("secondary").map(|m| m.name.as_str()),
        Some("optional_model.onnx")
    );
    assert!(manifest.model_by_role("non_existent").is_none());

    // Because optional_model is declared but missing, all_models_installed is FALSE:
    assert!(!manifest.all_models_installed());
    // But required models are installed:
    assert!(manifest.required_models_installed());

    // Empty models manifest returns true
    let empty_manifest = PluginManifest::default();
    assert!(empty_manifest.all_models_installed());
    assert!(empty_manifest.required_models_installed());

    // Once optional_model is installed, all_models_installed becomes TRUE:
    let opt_file = models_dir.join("optional_model.onnx");
    std::fs::write(&opt_file, b"dummy opt").unwrap();
    assert!(manifest.all_models_installed());

    // Now test flat plugin_dir placement
    let flat_model = temp_dir.join("flat_model.onnx");
    std::fs::write(&flat_model, b"dummy flat").unwrap();
    let flat_req = ModelRequirement {
        role: None,
        name: "flat_model.onnx".to_string(),
        required: true,
        size_mb: None,
        url: None,
        sha256: None,
    };
    assert!(manifest.is_model_installed(&flat_req));

    // Test 0-byte file and .part file
    let zero_byte_model = models_dir.join("zero_byte.onnx");
    std::fs::write(&zero_byte_model, b"").unwrap();
    let zero_req = ModelRequirement {
        role: None,
        name: "zero_byte.onnx".to_string(),
        required: true,
        size_mb: None,
        url: None,
        sha256: None,
    };
    assert!(!manifest.is_model_installed(&zero_req));

    let part_model = models_dir.join("in_progress.onnx.part");
    std::fs::write(&part_model, b"partial data").unwrap();
    let part_req = ModelRequirement {
        role: None,
        name: "in_progress.onnx.part".to_string(),
        required: true,
        size_mb: None,
        url: None,
        sha256: None,
    };
    assert!(!manifest.is_model_installed(&part_req));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_ai_pipeline_config_paradigms_serialization() {
    let json_data = r#"{
        "task": "image_to_image",
        "tile_size": 512,
        "tile_pad": 32,
        "normalize": "zero_to_one"
    }"#;

    let cfg: AiPipelineConfig = serde_json::from_str(json_data).unwrap();
    assert_eq!(cfg.task, AiTask::ImageToImage);
    assert_eq!(cfg.tile_size, Some(512));
    assert_eq!(cfg.tile_pad, Some(32));
    assert_eq!(cfg.normalize, Some(NormalizationMode::ZeroToOne));

    // Test super_resolution
    let json_sr = r#"{"task": "super_resolution", "scale": 4}"#;
    let cfg_sr: AiPipelineConfig = serde_json::from_str(json_sr).unwrap();
    assert_eq!(cfg_sr.task, AiTask::SuperResolution);
    assert_eq!(cfg_sr.scale, Some(4));

    // Test inpainting
    let json_inp = r#"{"task": "inpainting", "normalize": "raw_255"}"#;
    let cfg_inp: AiPipelineConfig = serde_json::from_str(json_inp).unwrap();
    assert_eq!(cfg_inp.task, AiTask::Inpainting);
    assert_eq!(cfg_inp.normalize, Some(NormalizationMode::Raw255));

    // Test depth_map / segmentation_mask with ImageNet & GrayscaleLayer
    let json_depth = r#"{
        "task": "segmentation_mask",
        "input_size": 518,
        "normalize": "image_net",
        "output_type": "grayscale_layer"
    }"#;
    let cfg_depth: AiPipelineConfig = serde_json::from_str(json_depth).unwrap();
    assert_eq!(cfg_depth.task, AiTask::SegmentationMask);
    assert_eq!(cfg_depth.input_size, Some(518));
    assert_eq!(cfg_depth.normalize, Some(NormalizationMode::ImageNet));
    assert_eq!(cfg_depth.output_type, Some(AiOutputType::GrayscaleLayer));
}

#[test]
fn test_plugin_manifest_native_algorithm_deserialization() {
    let manifest_json = r#"{
        "id": "vectorize.tracer",
        "name": "Vectorize Image",
        "version": "1.0.0",
        "description": "Convert raster images to vector SVG",
        "plugin_type": "native_algorithm",
        "algorithm": {
            "task": "vector_trace",
            "mode": "spline",
            "color_mode": "color",
            "filter_speckle": 4,
            "corner_threshold": 60,
            "copy_clipboard": true,
            "export_file": false
        }
    }"#;

    let manifest: PluginManifest = serde_json::from_str(manifest_json).unwrap();
    assert_eq!(manifest.plugin_type, PluginType::NativeAlgorithm);
    assert!(manifest.algorithm.is_some());
    let algo = manifest.algorithm.unwrap();
    assert_eq!(algo.task, "vector_trace");
    assert_eq!(algo.mode.as_deref(), Some("spline"));
    assert_eq!(algo.color_mode.as_deref(), Some("color"));
    assert_eq!(algo.filter_speckle, Some(4));
    assert_eq!(algo.corner_threshold, Some(60));
    assert!(algo.copy_clipboard);
    assert!(!algo.export_file);
}

#[test]
fn test_plugin_cannot_use_another_plugin_model() {
    let base_tmp = std::env::temp_dir().join("test_model_cross_plugin_isolation");
    let _ = std::fs::remove_dir_all(&base_tmp);

    let plugin_a = base_tmp.join("plugin_a");
    let plugin_b = base_tmp.join("plugin_b");
    std::fs::create_dir_all(plugin_a.join("models")).unwrap();
    std::fs::create_dir_all(plugin_b.join("models")).unwrap();

    // Model exists strictly in plugin_a
    let model_a_file = plugin_a.join("models").join("isolated_model.onnx");
    std::fs::write(&model_a_file, b"FAKE_ONNX_DATA").unwrap();

    // Plugin B attempts to access isolated_model.onnx
    let manifest_b = PluginManifest {
        id: "ai.photocraft.plugins.plugin_b".to_string(),
        name: "Plugin B".to_string(),
        version: "1.0.0".to_string(),
        description: "Plugin B description".to_string(),
        author: "Author".to_string(),
        plugin_type: PluginType::NativeAi,
        models: vec![ModelRequirement {
            name: "isolated_model.onnx".to_string(),
            required: true,
            role: Some("custom_role".to_string()),
            size_mb: Some(10),
            sha256: None,
            url: Some("https://example.com/isolated_model.onnx".to_string()),
        }],
        plugin_dir: Some(plugin_b.clone()),
        ..Default::default()
    };

    // Plugin B MUST NOT find model in plugin A's directory!
    let model_req = &manifest_b.models[0];
    assert!(
        !manifest_b.is_model_installed(model_req),
        "Violation: Plugin B was able to see and use Plugin A's model!"
    );
    assert!(
        !manifest_b.all_models_installed(),
        "Violation: Plugin B falsely reported all_models_installed when model belongs to Plugin A!"
    );

    let _ = std::fs::remove_dir_all(base_tmp);
}

#[test]
fn test_plugin_type_interactive_ui() {
    assert_eq!(PluginType::default(), PluginType::NoCode);
    let serialized = serde_json::to_string(&PluginType::InteractiveUi).unwrap();
    assert_eq!(serialized, "\"interactive_ui\"");
    let deserialized: PluginType = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, PluginType::InteractiveUi);
}

#[test]
fn test_banner_style_serialization() {
    let styles = [
        (BannerStyle::Info, "\"info\""),
        (BannerStyle::Warning, "\"warning\""),
        (BannerStyle::Error, "\"error\""),
        (BannerStyle::Success, "\"success\""),
    ];
    for (style, expected_json) in styles {
        let serialized = serde_json::to_string(&style).unwrap();
        assert_eq!(serialized, expected_json);
        let deserialized: BannerStyle = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, style);
    }
}

#[test]
fn test_interactive_ui_config_and_widgets_roundtrip() {
    let config = InteractiveUiConfig {
        window_title: "Harmonizer Studio".to_string(),
        window_title_ru: Some("Студия гармонии".to_string()),
        default_width: Some(360.0),
        default_height: Some(540.0),
        script: Some("ui.rhai".to_string()),
        auto_run_model_role: None,
        layout_mode: None,
        dynamic_badge: None,
        api_key_service: None,
        ipc_subscriptions: vec!["photocraft.layer.color_picked".to_string()],
        widgets: vec![
            WidgetNode::Banner {
                id: "status_banner".to_string(),
                message_binding: "banner_msg".to_string(),
                style_binding: Some("banner_style".to_string()),
            },
            WidgetNode::ColorSlot {
                id: "primary_color".to_string(),
                label: "Base Color".to_string(),
                label_ru: Some("Основной цвет".to_string()),
                default_hex: Some("#FF5500".to_string()),
                read_only: false,
                selectable: true,
                target_binding: Some("target_slot".to_string()),
                label_binding: None,
            },
            WidgetNode::HueWheel {
                id: "color_wheel".to_string(),
                source_color_id: "primary_color".to_string(),
                show_pointer: true,
                markers_binding: None,
                connect_mode_binding: None,
            },
            WidgetNode::Slider {
                id: "spread".to_string(),
                label: "Spread Angle".to_string(),
                label_ru: Some("Угол охвата".to_string()),
                min: 0.0,
                max: 180.0,
                step: Some(1.0),
                default: 45.0,
                enabled_binding: None,
            },
            WidgetNode::Dropdown {
                id: "harmony_rule".to_string(),
                label: "Harmony".to_string(),
                label_ru: Some("Гармония".to_string()),
                options: vec![
                    "Complementary".to_string(),
                    "Analogous".to_string(),
                    "Triadic".to_string(),
                ],
                options_ru: Some(vec![
                    "Комплементарная".to_string(),
                    "Аналоговая".to_string(),
                    "Триадическая".to_string(),
                ]),
                default_index: 1,
            },
            WidgetNode::Toggle {
                id: "live_sync".to_string(),
                label: "Live Sync".to_string(),
                label_ru: Some("Живая синхронизация".to_string()),
                default: true,
            },
            WidgetNode::Separator,
            WidgetNode::Label {
                text_binding: "summary_text".to_string(),
            },
            WidgetNode::VectorPad {
                id: "vector_pad".to_string(),
                label: "Extrusion Vector".to_string(),
                label_ru: None,
                max_distance: 80.0,
                default_angle_deg: 45.0,
                default_distance: 20.0,
            },
            WidgetNode::ImagePreview {
                id: "preview_canvas".to_string(),
                height: 200.0,
            },
            WidgetNode::Row {
                items: vec![
                    WidgetNode::ColorSlot {
                        id: "accent_1".to_string(),
                        label: "Harmonic 1".to_string(),
                        label_ru: None,
                        default_hex: None,
                        read_only: true,
                        selectable: false,
                        target_binding: None,
                        label_binding: None,
                    },
                    WidgetNode::ColorSlot {
                        id: "accent_2".to_string(),
                        label: "Harmonic 2".to_string(),
                        label_ru: None,
                        default_hex: None,
                        read_only: true,
                        selectable: false,
                        target_binding: None,
                        label_binding: None,
                    },
                ],
            },
        ],
    };

    let json = serde_json::to_string_pretty(&config).unwrap();
    let deserialized: InteractiveUiConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config, deserialized);
}

#[test]
fn test_tabs_widget_and_auto_ai_role_roundtrip() {
    let config = InteractiveUiConfig {
        window_title: "Depth Studio".to_string(),
        window_title_ru: Some("Студия глубины".to_string()),
        default_width: Some(780.0),
        default_height: Some(540.0),
        script: Some("logic.rhai".to_string()),
        auto_run_model_role: Some("depth_estimation".to_string()),
        layout_mode: Some("split_preview_right".to_string()),
        dynamic_badge: None,
        api_key_service: None,
        ipc_subscriptions: vec![],
        widgets: vec![WidgetNode::Tabs {
            id: "studio_tabs".to_string(),
            default_index: 0,
            tabs: vec![
                TabItem {
                    id: "fog_tab".to_string(),
                    title: "Atmospheric Fog".to_string(),
                    title_ru: Some("Атмосферный туман".to_string()),
                    icon: Some("fog".to_string()),
                    widgets: vec![WidgetNode::Toggle {
                        id: "fog_enabled".to_string(),
                        label: "Enable Fog".to_string(),
                        label_ru: Some("Включить туман".to_string()),
                        default: false,
                    }],
                },
                TabItem {
                    id: "bokeh_tab".to_string(),
                    title: "Lens Bokeh".to_string(),
                    title_ru: Some("Размытие боке".to_string()),
                    icon: Some("bokeh".to_string()),
                    widgets: vec![WidgetNode::Slider {
                        id: "focus_dist".to_string(),
                        label: "Focus Distance".to_string(),
                        label_ru: Some("Дистанция фокуса".to_string()),
                        min: 0.0,
                        max: 255.0,
                        step: Some(1.0),
                        default: 128.0,
                        enabled_binding: None,
                    }],
                },
            ],
        }],
    };

    let json = serde_json::to_string_pretty(&config).unwrap();
    let deserialized: InteractiveUiConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config, deserialized);
    if let WidgetNode::Tabs { tabs, .. } = &deserialized.widgets[0] {
        assert_eq!(tabs.len(), 2);
        assert_eq!(
            tabs[0].title_for_lang(crate::Language::En),
            "Atmospheric Fog"
        );
        assert_eq!(
            tabs[0].title_for_lang(crate::Language::Ru),
            "Атмосферный туман"
        );
    } else {
        panic!("Expected WidgetNode::Tabs");
    }
}

#[test]
fn test_plugin_ipc_action_serialization() {
    let action = PluginIpcAction {
        command: "photocraft.palette.apply".to_string(),
        params: serde_json::json!({
            "colors": ["#FF0000", "#00FF00", "#0000FF"],
            "target_layer_id": 12
        }),
    };

    let json = serde_json::to_string(&action).unwrap();
    let deserialized: PluginIpcAction = serde_json::from_str(&json).unwrap();
    assert_eq!(action, deserialized);

    // Test command without params
    let bare_action_json = r#"{"command": "photocraft.canvas.refresh"}"#;
    let bare_action: PluginIpcAction = serde_json::from_str(bare_action_json).unwrap();
    assert_eq!(bare_action.command, "photocraft.canvas.refresh");
    assert_eq!(bare_action.params, serde_json::Value::Null);
}

#[test]
fn test_plugin_manifest_interactive_ui_deserialization() {
    let manifest_json = r##"{
        "id": "ui.color_harmonizer",
        "name": "Color Harmonizer",
        "version": "1.0.0",
        "description": "Interactive color harmony wheel and palette generator",
        "plugin_type": "interactive_ui",
        "interactive_ui": {
            "window_title": "Color Harmonizer",
            "default_width": 320.0,
            "default_height": 480.0,
            "script": "harmonizer.rhai",
            "widgets": [
                {
                    "type": "banner",
                    "id": "tip_banner",
                    "message_binding": "tip_text"
                },
                {
                    "type": "color_slot",
                    "id": "base_color",
                    "label": "Base Color",
                    "default_hex": "#3498db"
                },
                {
                    "type": "hue_wheel",
                    "id": "wheel",
                    "source_color_id": "base_color"
                },
                {
                    "type": "slider",
                    "id": "roughness",
                    "label": "Angle",
                    "min": 0.0,
                    "max": 360.0,
                    "default": 180.0
                },
                {
                    "type": "dropdown",
                    "id": "mode",
                    "label": "Mode",
                    "options": ["Analogous", "Complementary"]
                },
                {
                    "type": "toggle",
                    "id": "auto_update",
                    "label": "Auto Update"
                },
                {
                    "type": "separator"
                },
                {
                    "type": "label",
                    "text_binding": "status_msg"
                },
                {
                    "type": "row",
                    "items": [
                        {
                            "type": "color_slot",
                            "id": "res_1",
                            "label": "Color 1"
                        }
                    ]
                }
            ],
            "ipc_subscriptions": ["palette.changed"]
        }
    }"##;

    let manifest: PluginManifest = serde_json::from_str(manifest_json).unwrap();
    assert_eq!(manifest.id, "ui.color_harmonizer");
    assert_eq!(manifest.plugin_type, PluginType::InteractiveUi);
    assert!(manifest.interactive_ui.is_some());

    let ui = manifest.interactive_ui.unwrap();
    assert_eq!(ui.window_title, "Color Harmonizer");
    assert_eq!(ui.default_width, Some(320.0));
    assert_eq!(ui.default_height, Some(480.0));
    assert_eq!(ui.script.as_deref(), Some("harmonizer.rhai"));
    assert_eq!(ui.widgets.len(), 9);
    assert_eq!(ui.ipc_subscriptions, vec!["palette.changed"]);

    match &ui.widgets[0] {
        WidgetNode::Banner {
            id,
            message_binding,
            style_binding,
        } => {
            assert_eq!(id, "tip_banner");
            assert_eq!(message_binding, "tip_text");
            assert_eq!(style_binding, &None);
        }
        other => panic!("Expected Banner, got {:?}", other),
    }

    match &ui.widgets[1] {
        WidgetNode::ColorSlot {
            id,
            label,
            default_hex,
            ..
        } => {
            assert_eq!(id, "base_color");
            assert_eq!(label, "Base Color");
            assert_eq!(default_hex.as_deref(), Some("#3498db"));
        }
        other => panic!("Expected ColorSlot, got {:?}", other),
    }

    match &ui.widgets[2] {
        WidgetNode::HueWheel {
            id,
            source_color_id,
            show_pointer,
            ..
        } => {
            assert_eq!(id, "wheel");
            assert_eq!(source_color_id, "base_color");
            assert!(show_pointer);
        }
        other => panic!("Expected HueWheel, got {:?}", other),
    }

    match &ui.widgets[3] {
        WidgetNode::Slider {
            id,
            label,
            min,
            max,
            default,
            step,
            ..
        } => {
            assert_eq!(id, "roughness");
            assert_eq!(label, "Angle");
            assert_eq!(*min, 0.0);
            assert_eq!(*max, 360.0);
            assert_eq!(*default, 180.0);
            assert_eq!(step, &None);
        }
        other => panic!("Expected Slider, got {:?}", other),
    }

    match &ui.widgets[4] {
        WidgetNode::Dropdown {
            id,
            label,
            options,
            default_index,
            ..
        } => {
            assert_eq!(id, "mode");
            assert_eq!(label, "Mode");
            assert_eq!(options, &["Analogous", "Complementary"]);
            assert_eq!(*default_index, 0);
        }
        other => panic!("Expected Dropdown, got {:?}", other),
    }

    match &ui.widgets[5] {
        WidgetNode::Toggle {
            id, label, default, ..
        } => {
            assert_eq!(id, "auto_update");
            assert_eq!(label, "Auto Update");
            assert!(!default);
        }
        other => panic!("Expected Toggle, got {:?}", other),
    }

    match &ui.widgets[6] {
        WidgetNode::Separator => {}
        other => panic!("Expected Separator, got {:?}", other),
    }

    match &ui.widgets[7] {
        WidgetNode::Label { text_binding } => {
            assert_eq!(text_binding, "status_msg");
        }
        other => panic!("Expected Label, got {:?}", other),
    }

    match &ui.widgets[8] {
        WidgetNode::Row { items } => {
            assert_eq!(items.len(), 1);
            assert!(matches!(items[0], WidgetNode::ColorSlot { .. }));
        }
        other => panic!("Expected Row, got {:?}", other),
    }
}

#[test]
fn test_sub_action_and_action_request() {
    let sub = SubAction {
        id: "mask".to_string(),
        locales: std::collections::HashMap::new(),
        name: "Layer Mask".to_string(),
        name_ru: Some("Маска слоя".to_string()),
        icon: Some("mask.svg".to_string()),
        post_commands: vec![PluginCommandStep::new("layer.mask")],
        script: None,
    };

    assert_eq!(sub.name_for_lang(crate::Language::En), "Layer Mask");
    assert_eq!(sub.name_for_lang(crate::Language::Ru), "Маска слоя");

    let req1 = ActionRequest::new("plugin:remove_bg");
    assert_eq!(req1.to_string(), "plugin:remove_bg");
    assert_eq!(req1.sub_action_id, None);

    let req2 = ActionRequest::with_sub_action("plugin:remove_bg", "mask");
    assert_eq!(req2.to_string(), "plugin:remove_bg#mask");
    assert_eq!(req2.sub_action_id.as_deref(), Some("mask"));

    let req3 = ActionRequest::from("plugin:remove_bg#mask".to_string());
    assert_eq!(req3, req2);

    let req4 = ActionRequest::from("plugin:remove_bg");
    assert_eq!(req4, req1);
}

#[test]
fn test_plugin_manifest_resolve_icon_path() {
    let temp_dir = std::env::temp_dir().join(format!(
        "pc_test_icon_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let plugin_dir = temp_dir.join("my_plugin");
    std::fs::create_dir_all(&plugin_dir).unwrap();

    let icon_svg = plugin_dir.join("icon.svg");
    std::fs::write(&icon_svg, "<svg></svg>").unwrap();

    let mut manifest = PluginManifest {
        id: "my_plugin".to_string(),
        name: "My Plugin".to_string(),
        plugin_dir: Some(plugin_dir.clone()),
        ..Default::default()
    };

    // Default icon.svg resolution via plugin_dir
    assert_eq!(manifest.resolve_icon_path(None), Some(icon_svg.clone()));

    // Explicit custom icon resolution
    let custom_icon = plugin_dir.join("custom.svg");
    std::fs::write(&custom_icon, "<svg>custom</svg>").unwrap();
    manifest.icon = Some("custom.svg".to_string());
    assert_eq!(manifest.resolve_icon_path(None), Some(custom_icon));

    // Resolution via root_plugins_dir when plugin_dir is None
    manifest.plugin_dir = None;
    assert_eq!(
        manifest.resolve_icon_path(Some(&temp_dir)),
        Some(plugin_dir.join("custom.svg"))
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_canvas_preview_params_and_runtime_tools_serialization() {
    let manifest = PluginManifest {
        id: "video_cut".to_string(),
        name: "Video Cutter".to_string(),
        runtime_tools: vec![RuntimeToolRequirement {
            name: "ffmpeg".to_string(),
            version: "6.1.1".to_string(),
            url_windows_x64: Some("https://example.com/ffmpeg.zip".to_string()),
            url_linux_x64: None,
            url_macos_arm64: None,
            url_macos_x64: None,
            sha256: Some(
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string(),
            ),
        }],
        ..Default::default()
    };

    let json = serde_json::to_string(&manifest).unwrap();
    let decoded: PluginManifest = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.runtime_tools.len(), 1);
    assert_eq!(decoded.runtime_tools[0].name, "ffmpeg");
    assert_eq!(decoded.runtime_tools[0].version, "6.1.1");
}

#[test]
fn test_dynamic_badge_state_roundtrip() {
    let config = InteractiveUiConfig {
        window_title: "Color Studio".to_string(),
        dynamic_badge: Some(DynamicBadgeState {
            is_pulsing: true,
            pulse_color_rgba: [255, 100, 50, 255],
            label: Some("LIVE".to_string()),
        }),
        ..Default::default()
    };

    let json = serde_json::to_string(&config).unwrap();
    let decoded: InteractiveUiConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(config, decoded);
    let badge = decoded.dynamic_badge.unwrap();
    assert!(badge.is_pulsing);
    assert_eq!(badge.pulse_color_rgba, [255, 100, 50, 255]);
    assert_eq!(badge.label.as_deref(), Some("LIVE"));
}
