use std::fs;
use std::path::PathBuf;

fn find_workspace_root() -> PathBuf {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()));
    let mut current = manifest_dir.clone();
    for _ in 0..5 {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = fs::read_to_string(&cargo_toml) {
                if content.contains("[workspace]") {
                    return current;
                }
            }
        }
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }
    manifest_dir
}

#[test]
fn test_dynamic_tool_uses_multi_viewport_and_prohibits_egui_window_new() {
    let root = find_workspace_root();
    let dynamic_tool_path = root
        .join("crates")
        .join("overlay_ui")
        .join("src")
        .join("dynamic_tool.rs");

    assert!(
        dynamic_tool_path.exists(),
        "dynamic_tool.rs not found at {}",
        dynamic_tool_path.display()
    );

    let content = fs::read_to_string(&dynamic_tool_path)
        .unwrap_or_else(|e| panic!("Failed to read dynamic_tool.rs: {e}"));

    // 1. Strict prohibition of egui::Window::new (which confines rendering to the 44px toolbar viewport)
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }
        assert!(
            !trimmed.contains("egui::Window::new") && !trimmed.contains("Window::new("),
            "Multi-Viewport Violation in dynamic_tool.rs line {}: '{}'. \
             Interactive plugin windows MUST NOT use egui::Window::new (which squashes windows into the 44px toolbar). \
             They MUST use ctx.show_viewport_immediate to float as independent OS windows!",
            line_idx + 1,
            trimmed
        );
    }

    // 2. Strict requirements for multi-viewport implementation
    assert!(
        content.contains("show_viewport_immediate"),
        "dynamic_tool.rs must use ctx.show_viewport_immediate for interactive floating tool windows!"
    );
    assert!(
        content.contains(".with_always_on_top()"),
        "dynamic_tool.rs must configure ViewportBuilder with .with_always_on_top() so tool palettes float above canvas!"
    );
    assert!(
        content.contains(".with_inner_size("),
        "dynamic_tool.rs must configure ViewportBuilder with .with_inner_size() from plugin manifest!"
    );
    assert!(
        content.contains(".with_min_inner_size("),
        "dynamic_tool.rs must configure ViewportBuilder with .with_min_inner_size()!"
    );
    assert!(
        content.contains(".with_resizable(true)"),
        "dynamic_tool.rs must configure ViewportBuilder with .with_resizable(true)!"
    );
    assert!(
        content.contains("viewport_id_for_plugin"),
        "dynamic_tool.rs must use deterministic ViewportId generation!"
    );
}

#[test]
fn test_all_interactive_ui_manifests_declare_sane_viewport_dimensions() {
    let root = find_workspace_root();
    let plugins_dir = root.join("plugins");

    assert!(plugins_dir.exists(), "plugins/ directory not found!");

    let mut interactive_plugins_count = 0;

    for entry in fs::read_dir(&plugins_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("plugin.json");
        if !manifest_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&manifest_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {e}", manifest_path.display()));

        let plugin_type = json
            .get("plugin_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if plugin_type == "interactive_ui" {
            interactive_plugins_count += 1;
            let interactive_ui = json.get("interactive_ui").unwrap_or_else(|| {
                panic!(
                    "Plugin {} is interactive_ui but missing 'interactive_ui' config block!",
                    path.display()
                )
            });

            let title = interactive_ui
                .get("window_title")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            assert!(
                !title.is_empty(),
                "Plugin {} must declare a non-empty 'window_title'!",
                path.display()
            );

            let width = interactive_ui
                .get("default_width")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            assert!(
                (240.0..=1200.0).contains(&width),
                "Plugin {} default_width ({width}) must be within [240.0, 1200.0] for sane OS window rendering!",
                path.display()
            );

            let height = interactive_ui
                .get("default_height")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            assert!(
                (200.0..=1600.0).contains(&height),
                "Plugin {} default_height ({height}) must be within [200.0, 1600.0] for sane OS window rendering!",
                path.display()
            );
        }
    }

    assert!(
        interactive_plugins_count >= 1,
        "Expected at least 1 interactive UI plugin (e.g. ChromaSense), found {interactive_plugins_count}"
    );
}
