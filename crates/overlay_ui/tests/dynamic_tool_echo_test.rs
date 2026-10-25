use core_contracts::{InteractiveUiConfig, PluginManifest, PluginType};
use overlay_ui::DynamicToolEngine;

fn make_manifest(id: &str, subscriptions: Vec<String>) -> PluginManifest {
    PluginManifest {
        id: id.to_string(),
        name: id.to_string(),
        version: "1.0.0".to_string(),
        description: "Test".to_string(),
        author: "Test".to_string(),
        plugin_type: PluginType::InteractiveUi,
        icon: None,
        models: Vec::new(),
        commands: Vec::new(),
        post_commands: Vec::new(),
        sub_actions: Vec::new(),
        input: Default::default(),
        output_mode: Default::default(),
        action_id: None,
        entrypoint: None,
        ai: None,
        algorithm: None,
        interactive_ui: Some(InteractiveUiConfig {
            window_title: id.to_string(),
            window_title_ru: None,
            default_width: Some(300.0),
            default_height: Some(400.0),
            script: None,
            widgets: Vec::new(),
            ipc_subscriptions: subscriptions,
        }),
        plugin_dir: None,
        name_ru: None,
        description_ru: None,
        locales: Default::default(),
        enabled: true,
    }
}

#[test]
fn test_has_open_windows_with_subscription() {
    let m1 = make_manifest("p1", vec!["session.colors".to_string()]);
    let m2 = make_manifest("p2", vec![]);
    let mut engine = DynamicToolEngine::new(&[m1, m2], None);

    // Initially all windows are closed
    assert!(!engine.has_open_windows_with_subscription("session.colors"));

    // Toggle window 2 (no subscription)
    engine.toggle_window("p2");
    assert!(engine.is_window_open("p2"));
    assert!(!engine.has_open_windows_with_subscription("session.colors"));

    // Toggle window 1 (has subscription)
    engine.toggle_window("p1");
    assert!(engine.is_window_open("p1"));
    assert!(engine.has_open_windows_with_subscription("session.colors"));

    // Close window 1
    engine.toggle_window("p1");
    assert!(!engine.is_window_open("p1"));
    assert!(!engine.has_open_windows_with_subscription("session.colors"));
}

#[test]
fn test_dispatch_session_colors_echo_suppression() {
    let m1 = make_manifest("p1", vec!["session.colors".to_string()]);
    let mut engine = DynamicToolEngine::new(&[m1], None);
    engine.toggle_window("p1");

    // First incoming color dispatch: not an echo of any previous outgoing command
    let updated = engine.dispatch_session_colors("#21c45d", "#ffffff");
    // Since there is no script attached to dummy manifest, it returns false for state mutated,
    // but last_emitted in echo_guard should be updated to (#21c45d, #ffffff).
    assert!(!updated);

    // Immediate second dispatch with the EXACT same colors must be suppressed as echo!
    let suppressed = engine.dispatch_session_colors("#21c45d", "#ffffff");
    assert!(!suppressed);

    // New different colors from PhotoCraft must pass through
    let updated2 = engine.dispatch_session_colors("#ff0000", "#ffffff");
    assert!(!updated2);
}
