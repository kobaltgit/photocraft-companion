use std::fs;
use std::path::{Path, PathBuf};

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

fn collect_all_dynamic_tool_sources() -> Vec<(PathBuf, String)> {
    let root = find_workspace_root();
    let dynamic_tool_dir = root
        .join("crates")
        .join("overlay_ui")
        .join("src")
        .join("dynamic_tool");

    let mut files = Vec::new();
    fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect(&path, out);
                } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    out.push(path);
                }
            }
        }
    }
    collect(&dynamic_tool_dir, &mut files);

    let mut results = Vec::new();
    for f in files {
        let content = fs::read_to_string(&f)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", f.display()));
        results.push((f, content));
    }
    results
}

#[test]
fn test_no_magic_number_fixed_label_widths_in_dynamic_tool() {
    let sources = collect_all_dynamic_tool_sources();
    assert!(!sources.is_empty(), "No dynamic_tool sources found!");

    let mut all_content = String::new();

    // Rule 1: Zero Magic-Number Label Widths in dynamic stack/tool layout
    for (path, content) in &sources {
        all_content.push_str(content);
        all_content.push('\n');

        for (idx, line) in content.lines().enumerate() {
            let trimmed = strip_line_comment(line).trim();
            let line_num = idx + 1;

            if trimmed.contains("let label_width =") {
                panic!(
                    "Magic number detected in {}:{} -> '{}'. Label widths must be calculated dynamically based on longest text!",
                    path.display(),
                    line_num,
                    trimmed
                );
            }

            if let Some(pos) = trimmed.find("allocate_space(vec2(") {
                let after = &trimmed[pos + "allocate_space(vec2(".len()..];
                if after.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                    panic!(
                        "Hardcoded slot width detected in {}:{} -> '{}'. Use dynamic label width measurement!",
                        path.display(),
                        line_num,
                        trimmed
                    );
                }
            }
        }
    }

    // Verify dynamic text measurement is present in dynamic_tool
    assert!(
        all_content.contains("layout_no_wrap"),
        "dynamic_tool must dynamically measure text length with layout_no_wrap to prevent stair-stepping (лесенка)!"
    );
    assert!(
        all_content.contains("max_label_w"),
        "dynamic_tool must compute max_label_w dynamically across items!"
    );
}

#[test]
fn test_dynamic_sliders_have_custom_formatters() {
    let sources = collect_all_dynamic_tool_sources();
    assert!(!sources.is_empty(), "No dynamic_tool sources found!");

    let mut all_content = String::new();
    for (_, content) in &sources {
        all_content.push_str(content);
        all_content.push('\n');
    }

    // Rule 2: Dynamic tool modifier sliders must use custom_formatter and custom_parser
    assert!(
        all_content.contains(".custom_formatter("),
        "dynamic_tool must use custom_formatter on sliders to display clean integers without '.0'!"
    );
    assert!(
        all_content.contains(".custom_parser("),
        "dynamic_tool must provide custom_parser for sliders with units!"
    );
    assert!(
        all_content.contains("format!(\"{:.0}") || all_content.contains("format!(\"{:.0}{}\""),
        "dynamic_tool must format integer steps with '{{:.0}}'!"
    );
    assert!(
        all_content.contains("format!(\"{:.1}") || all_content.contains("format!(\"{:.1}{}\""),
        "dynamic_tool must format fractional steps with '{{:.1}}'!"
    );
}

#[test]
fn test_mouse_wheel_step_discrete_quantization() {
    let sources = collect_all_dynamic_tool_sources();
    assert!(!sources.is_empty(), "No dynamic_tool sources found!");

    let mut all_content = String::new();
    for (path, content) in &sources {
        assert!(
            !content.contains("smooth_scroll_delta"),
            "{} must NOT use inertial smooth_scroll_delta; use discrete Event::MouseWheel instead!",
            path.display()
        );
        all_content.push_str(content);
        all_content.push('\n');
    }

    // Rule 3: Mouse wheel scrolling must quantize to exact step increments (1.0 for integer, 0.1 for float)
    assert!(
        all_content.contains("Event::MouseWheel"),
        "dynamic_tool must handle mouse wheel scrolling via discrete Event::MouseWheel events!"
    );
    assert!(
        all_content.contains("let wheel_step = if is_integer { 1.0 } else { 0.1 };"),
        "dynamic_tool must use exact 1.0 step for integer sliders and 0.1 for fractional sliders on wheel scroll!"
    );
}
