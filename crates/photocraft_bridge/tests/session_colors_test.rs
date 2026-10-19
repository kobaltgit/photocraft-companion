use photocraft_bridge::parse_session_colors;
use serde_json::json;

#[test]
fn test_parse_session_colors_success() {
    let json = json!({
        "foreground": [0.13, 0.77, 0.36, 1.0],
        "background": [1.0, 1.0, 1.0, 1.0]
    });
    let (fg, bg) = parse_session_colors(&json);
    assert_eq!(fg, "#21c45c");
    assert_eq!(bg, "#ffffff");
}

#[test]
fn test_parse_session_colors_fallback() {
    let empty = json!({});
    let (fg, bg) = parse_session_colors(&empty);
    assert_eq!(fg, "#000000");
    assert_eq!(bg, "#ffffff");

    let malformed = json!({
        "foreground": [0.5],
        "background": "invalid"
    });
    let (fg2, bg2) = parse_session_colors(&malformed);
    assert_eq!(fg2, "#000000");
    assert_eq!(bg2, "#ffffff");
}

#[test]
fn test_parse_session_colors_clamping() {
    let clamped = json!({
        "foreground": [-0.5, 0.0, 2.0],
        "background": [0.5, 1.0, 0.25]
    });
    let (fg, bg) = parse_session_colors(&clamped);
    assert_eq!(fg, "#0000ff");
    assert_eq!(bg, "#80ff40");
}
