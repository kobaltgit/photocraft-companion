use std::fs;
use std::path::PathBuf;

/// Locate workspace root containing Cargo.toml with [workspace]
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

/// Strip line comments (`// ...`) outside string literals
fn strip_line_comment(line: &str) -> &str {
    let mut in_string = false;
    let mut escaped = false;
    let mut chars = line.char_indices().peekable();
    while let Some((i, ch)) = chars.next() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && in_string {
            escaped = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if !in_string && ch == '/' {
            if let Some(&(_, '/')) = chars.peek() {
                return line[..i].trim_end();
            }
        }
    }
    line
}

#[test]
fn test_no_magic_number_fixed_label_widths_in_dynamic_tool() {
    let root = find_workspace_root();
    let dynamic_tool_path = root
        .join("crates")
        .join("overlay_ui")
        .join("src")
        .join("dynamic_tool.rs");
    let content = fs::read_to_string(&dynamic_tool_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", dynamic_tool_path.display()));

    // Rule 1: Zero Magic-Number Label Widths in dynamic stack/tool layout
    // Prohibit patterns like `let label_width = 90.0;` or `allocate_space(vec2(90.0, ...))` for item labels
    for (idx, line) in content.lines().enumerate() {
        let trimmed = strip_line_comment(line).trim();
        let line_num = idx + 1;

        if trimmed.contains("let label_width =") {
            panic!(
                "Magic number detected in {}:{} -> '{}'. Label widths must be calculated dynamically based on longest text!",
                dynamic_tool_path.display(),
                line_num,
                trimmed
            );
        }

        if let Some(pos) = trimmed.find("allocate_space(vec2(") {
            let after = &trimmed[pos + "allocate_space(vec2(".len()..];
            if after.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                panic!(
                    "Hardcoded slot width detected in {}:{} -> '{}'. Use dynamic label width measurement!",
                    dynamic_tool_path.display(),
                    line_num,
                    trimmed
                );
            }
        }
    }

    // Verify dynamic text measurement is present in dynamic_tool.rs
    assert!(
        content.contains("layout_no_wrap"),
        "dynamic_tool.rs must dynamically measure text length with layout_no_wrap to prevent stair-stepping (лесенка)!"
    );
    assert!(
        content.contains("max_label_w"),
        "dynamic_tool.rs must compute max_label_w dynamically across items!"
    );
}

#[test]
fn test_dynamic_sliders_have_custom_formatters() {
    let root = find_workspace_root();
    let dynamic_tool_path = root
        .join("crates")
        .join("overlay_ui")
        .join("src")
        .join("dynamic_tool.rs");
    let content = fs::read_to_string(&dynamic_tool_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", dynamic_tool_path.display()));

    // Rule 2: Dynamic tool modifier sliders must use custom_formatter and custom_parser
    // to render clean integers ("0 px", "15 px") without ugly trailing ".0 px".
    assert!(
        content.contains(".custom_formatter("),
        "dynamic_tool.rs must use custom_formatter on sliders to display clean integers without '.0'!"
    );
    assert!(
        content.contains(".custom_parser("),
        "dynamic_tool.rs must provide custom_parser for sliders with units!"
    );
    assert!(
        content.contains("format!(\"{:.0}{}\""),
        "dynamic_tool.rs must format integer steps with '{{:.0}}'!"
    );
    assert!(
        content.contains("format!(\"{:.1}{}\""),
        "dynamic_tool.rs must format fractional steps with '{{:.1}}'!"
    );
}

#[test]
fn test_mouse_wheel_step_discrete_quantization() {
    let root = find_workspace_root();
    let dynamic_tool_path = root
        .join("crates")
        .join("overlay_ui")
        .join("src")
        .join("dynamic_tool.rs");
    let content = fs::read_to_string(&dynamic_tool_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", dynamic_tool_path.display()));

    // Rule 3: Mouse wheel scrolling must quantize to exact step increments (1.0 for integer, 0.1 for float)
    // Prohibit inertial smooth_scroll_delta which causes multi-frame lag and drifting
    assert!(
        !content.contains("smooth_scroll_delta"),
        "dynamic_tool.rs must NOT use inertial smooth_scroll_delta; use discrete Event::MouseWheel instead!"
    );
    assert!(
        content.contains("Event::MouseWheel"),
        "dynamic_tool.rs must handle mouse wheel scrolling via discrete Event::MouseWheel events!"
    );
    assert!(
        content.contains("let wheel_step = if is_integer { 1.0 } else { 0.1 };"),
        "dynamic_tool.rs must use exact 1.0 step for integer sliders and 0.1 for fractional sliders on wheel scroll!"
    );
}
