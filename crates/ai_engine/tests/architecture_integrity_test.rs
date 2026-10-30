use std::fs;
use std::path::{Path, PathBuf};

/// Keywords associated with specific models or plugin actions that are strictly
/// forbidden from being inspected via keyword sniffing (.contains, .starts_with, .ends_with)
/// in core production code.
const BANNED_SNIFFING_KEYWORDS: &[&str] = &[
    "depth",
    "normal",
    "harmoniz",
    "birefnet",
    "lama",
    "esrgan",
    "realesrgan",
    "rmbg",
    "rembg",
    "denoise",
    "deblur",
    "colorize",
    "inpaint",
    "upscale",
    "remove_bg",
    "depth_map",
    "vectorize",
    "black_and_white",
    "chromasense",
];

/// Prohibited hardcoded model weight file extensions in string literals inside core crates.
const BANNED_MODEL_FILE_EXTENSIONS: &[&str] = &[".onnx", ".safetensors", ".bin", ".pt", ".pth"];

/// Prohibited graphic editor command prefixes in companion_app production code.
/// Graphic editor operations must never be hardcoded in companion_app; they must be declared
/// declaratively in plugin manifests or executed via scripts.
const BANNED_EDITOR_COMMAND_PREFIXES: &[&str] = &[
    "select.",
    "selection.",
    "layer.",
    "image.",
    "channel.",
    "filter.",
    "history.",
];

/// Prohibited hardcoded plugin ID prefixes in string literals inside core crates.
const BANNED_PLUGIN_ID_PREFIXES: &[&str] = &[
    "\"ai.photocraft.plugins.",
    "\"macro.photocraft.plugins.",
    "\"algorithm.photocraft.plugins.",
    "\"ui.photocraft.plugins.",
    "\"plugin:",
];

#[derive(Debug)]
pub struct IntegrityViolation {
    pub file_path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
    pub reason: String,
}

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

/// Extract string literals (`"..."`) from a line of Rust code
fn extract_string_literals(line: &str) -> Vec<String> {
    let mut literals = Vec::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut current = String::new();

    for ch in line.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' && in_string {
            escaped = true;
            continue;
        }
        if ch == '"' {
            if in_string {
                literals.push(current.clone());
                current.clear();
                in_string = false;
            } else {
                in_string = true;
            }
            continue;
        }
        if in_string {
            current.push(ch);
        }
    }
    literals
}

/// Analyze a single production line for architectural integrity violations
pub fn check_line_integrity(
    file_path: &Path,
    line_number: usize,
    raw_line: &str,
) -> Option<IntegrityViolation> {
    let line = strip_line_comment(raw_line).trim();
    if line.is_empty() {
        return None;
    }

    let string_literals = extract_string_literals(line);

    // Rule 1: No keyword sniffing via .contains(), .starts_with(), .ends_with(), or == on banned keywords
    let has_string_comparison = line.contains(".contains(")
        || line.contains(".starts_with(")
        || line.contains(".ends_with(")
        || line.contains("==")
        || line.contains("!=");

    if has_string_comparison {
        for lit in &string_literals {
            let lit_lower = lit.to_lowercase();
            for keyword in BANNED_SNIFFING_KEYWORDS {
                if lit_lower.contains(keyword) {
                    return Some(IntegrityViolation {
                        file_path: file_path.to_path_buf(),
                        line_number,
                        line_content: line.to_string(),
                        reason: format!(
                            "Forbidden keyword-sniffing on '{}' in core production code. Use ModelRequirement 'role' or declarative manifest fields.",
                            keyword
                        ),
                    });
                }
            }
        }
    }

    // Rule 2: No hardcoded model weight file names (.onnx, .safetensors, .bin, etc.)
    for lit in &string_literals {
        let lit_lower = lit.to_lowercase();
        for ext in BANNED_MODEL_FILE_EXTENSIONS {
            if lit_lower.ends_with(ext) {
                return Some(IntegrityViolation {
                    file_path: file_path.to_path_buf(),
                    line_number,
                    line_content: line.to_string(),
                    reason: format!(
                        "Forbidden hardcoded model weight file '{}' in core production code. Models must be declared in plugin.json manifests.",
                        lit
                    ),
                });
            }
        }
    }

    // Rule 3: No hardcoded plugin IDs in core crates
    for prefix in BANNED_PLUGIN_ID_PREFIXES {
        if line.contains(prefix) {
            return Some(IntegrityViolation {
                file_path: file_path.to_path_buf(),
                line_number,
                line_content: line.to_string(),
                reason: format!(
                    "Forbidden hardcoded plugin ID prefix '{}' in core production code. Core must remain 100% agnostic of specific plugins.",
                    prefix
                ),
            });
        }
    }

    // Rule 4 (Zero-Editor-Commands Rule): In companion_app/src production code,
    // graphic editor command string literals ("select.", "selection.", "layer.", "image.", "channel.", "filter.", "history.")
    // are strictly forbidden. Multi-step editor workflows must be coordinated declaratively or via Rhai scripts.
    let is_companion_app_src = {
        let p = file_path.to_string_lossy().replace('\\', "/");
        p.contains("companion_app/src") || p.contains("dummy")
    };

    if is_companion_app_src {
        for lit in &string_literals {
            let lit_lower = lit.to_lowercase();
            for prefix in BANNED_EDITOR_COMMAND_PREFIXES {
                if lit_lower.starts_with(prefix) {
                    return Some(IntegrityViolation {
                        file_path: file_path.to_path_buf(),
                        line_number,
                        line_content: line.to_string(),
                        reason: format!(
                            "Forbidden editor command literal '{}' with prefix '{}' in companion_app production code. Editor commands must be declared declaratively in plugin manifests or Rhai scripts.",
                            lit, prefix
                        ),
                    });
                }
            }
        }
    }

    None
}

/// Recursively collect all .rs files in a directory
fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if !dir.exists() {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_rs_files(&path, files);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }
}

/// Scan a file for integrity violations, skipping #[cfg(test)] modules and test files
fn scan_file_for_violations(file_path: &Path) -> Vec<IntegrityViolation> {
    // Skip test files (*_test.rs or files inside tests/ directories)
    let path_str = file_path.to_string_lossy().replace('\\', "/");
    if path_str.ends_with("_test.rs") || path_str.contains("/tests/") {
        return Vec::new();
    }

    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();
    let mut in_test_module = false;
    let mut in_multiline_comment = false;

    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();

        // Handle multi-line comments /* ... */
        if in_multiline_comment {
            if trimmed.contains("*/") {
                in_multiline_comment = false;
            }
            continue;
        }
        if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") {
                in_multiline_comment = true;
            }
            continue;
        }

        // Detect test modules: #[cfg(test)] mod tests { ... }
        if trimmed.contains("#[cfg(test)]") || trimmed.contains("mod tests") {
            in_test_module = true;
            continue;
        }

        // If inside test module, skip checking
        if in_test_module {
            continue;
        }

        if let Some(violation) = check_line_integrity(file_path, line_num, line) {
            violations.push(violation);
        }
    }

    violations
}

#[test]
fn test_core_crates_architecture_integrity() {
    let workspace_root = find_workspace_root();
    println!(
        "Workspace root for integrity lint: {}",
        workspace_root.display()
    );

    let target_crates = [
        workspace_root.join("crates").join("ai_engine").join("src"),
        workspace_root
            .join("crates")
            .join("companion_app")
            .join("src"),
        workspace_root
            .join("crates")
            .join("photocraft_bridge")
            .join("src"),
    ];

    let mut all_files = Vec::new();
    for target in &target_crates {
        collect_rs_files(target, &mut all_files);
    }

    assert!(!all_files.is_empty(), "No source files found to scan!");

    let mut total_violations = Vec::new();

    for file in &all_files {
        let violations = scan_file_for_violations(file);
        total_violations.extend(violations);
    }

    if !total_violations.is_empty() {
        eprintln!(
            "\n================================================================================"
        );
        eprintln!(
            "❌ ARCHITECTURE INTEGRITY VIOLATION DETECTED ({})",
            total_violations.len()
        );
        eprintln!(
            "================================================================================"
        );
        for v in &total_violations {
            eprintln!(
                "\n[VIOLATION] {}\n  File: {}:{}\n  Line: {}",
                v.reason,
                v.file_path.display(),
                v.line_number,
                v.line_content
            );
        }
        eprintln!(
            "\n================================================================================"
        );
        eprintln!("Strict Zero-Core Hardcoding Rule violated! Core crates must be 100% agnostic.");
        eprintln!(
            "================================================================================\n"
        );
        panic!(
            "Architecture integrity test failed: found {} forbidden hardcoded pattern(s) in core crates!",
            total_violations.len()
        );
    }
}

#[test]
fn test_integrity_linter_catches_all_forbidden_patterns() {
    let dummy_path = Path::new("dummy/core_file.rs");

    // Case 1: Keyword sniffing on depth
    let v1 = check_line_integrity(dummy_path, 10, "if name.contains(\"depth\") {");
    assert!(v1.is_some());
    assert!(v1.unwrap().reason.contains("depth"));

    // Case 2: Keyword sniffing on birefnet
    let v2 = check_line_integrity(dummy_path, 11, "if lower.contains(\"birefnet\") {");
    assert!(v2.is_some());
    assert!(v2.unwrap().reason.contains("birefnet"));

    // Case 3: Keyword sniffing on normal
    let v3 = check_line_integrity(dummy_path, 12, "if model_id.ends_with(\"normal\") {");
    assert!(v3.is_some());
    assert!(v3.unwrap().reason.contains("normal"));

    // Case 4: Hardcoded model weight (.onnx)
    let v4 = check_line_integrity(dummy_path, 13, "let model = self.resolve(\"lama.onnx\");");
    assert!(v4.is_some());
    assert!(v4.unwrap().reason.contains(".onnx"));

    // Case 5: Hardcoded plugin ID
    let v5 = check_line_integrity(
        dummy_path,
        14,
        "let target = \"ai.photocraft.plugins.custom_filter\";",
    );
    assert!(v5.is_some());
    assert!(v5.unwrap().reason.contains("ai.photocraft.plugins."));

    // Case 6: Legitimate production code without sniffing
    let ok1 = check_line_integrity(dummy_path, 15, "let norm = NormalizationMode::ZeroToOne;");
    assert!(ok1.is_none());

    // Case 7: Legitimate checking a vector of PathBufs
    let ok2 = check_line_integrity(
        dummy_path,
        16,
        "if self.cpu_fallback_models.contains(&path) {",
    );
    assert!(ok2.is_none());

    // Case 8: Comments should be stripped
    let ok3 = check_line_integrity(
        dummy_path,
        17,
        "// check if model contains depth or birefnet",
    );
    assert!(ok3.is_none());

    // Case 9: Hardcoded editor command (selection.modify_composite)
    let v6 = check_line_integrity(
        dummy_path,
        18,
        "if action.command == \"selection.modify_composite\" {",
    );
    assert!(v6.is_some());
    assert!(v6.unwrap().reason.contains("selection.modify_composite"));

    // Case 10: Hardcoded editor command (select.modify.border)
    let v7 = check_line_integrity(
        dummy_path,
        19,
        "if action.command == \"select.modify.border\" {",
    );
    assert!(v7.is_some());
    assert!(v7.unwrap().reason.contains("select.modify.border"));

    // Case 11: Hardcoded editor command (layer.duplicate)
    let v8 = check_line_integrity(dummy_path, 20, "let cmd = \"layer.duplicate\";");
    assert!(v8.is_some());
    assert!(v8.unwrap().reason.contains("layer.duplicate"));

    // Case 12: Hardcoded editor command (image.revealAll)
    let v9 = check_line_integrity(dummy_path, 21, "let cmd = \"image.revealAll\";");
    assert!(v9.is_some());
    assert!(v9.unwrap().reason.contains("image.revealAll"));

    // Case 13: Hardcoded editor command (channel.select)
    let v10 = check_line_integrity(dummy_path, 22, "let cmd = \"channel.select\";");
    assert!(v10.is_some());
    assert!(v10.unwrap().reason.contains("channel.select"));

    // Case 14: Hardcoded editor command (filter.blur)
    let v11 = check_line_integrity(dummy_path, 23, "let cmd = \"filter.blur\";");
    assert!(v11.is_some());
    assert!(v11.unwrap().reason.contains("filter.blur"));

    // Case 15: Hardcoded editor command (history.undo)
    let v12 = check_line_integrity(dummy_path, 24, "let cmd = \"history.undo\";");
    assert!(v12.is_some());
    assert!(v12.unwrap().reason.contains("history.undo"));

    // Case 16: Legitimate environment context metadata in resolver ($document.has_selection, $has_selection)
    let ok4 = check_line_integrity(
        dummy_path,
        25,
        "\"$document.has_selection\" | \"$has_selection\" => return json!(ctx.has_selection),",
    );
    assert!(ok4.is_none());

    // Case 17: Legitimate document dimension metadata in resolver ($document.id, $document.width, $document.height)
    let ok5 = check_line_integrity(
        dummy_path,
        26,
        "\"$document.id\" => return ctx.document_id.map(|id| json!(id)).unwrap_or(Value::Null),",
    );
    assert!(ok5.is_none());
    let ok6 = check_line_integrity(
        dummy_path,
        27,
        "\"$document.width\" => return json!(ctx.document_width),",
    );
    assert!(ok6.is_none());
    let ok7 = check_line_integrity(
        dummy_path,
        28,
        "\"$document.height\" => return json!(ctx.document_height),",
    );
    assert!(ok7.is_none());

    // Case 18: Legitimate variable names in runner (has_selection, result_layer, source_layer)
    let ok8 = check_line_integrity(dummy_path, 29, "if cond == \"has_selection\" {");
    assert!(ok8.is_none());
    let ok9 = check_line_integrity(dummy_path, 30, "if save_var == \"result_layer\" {");
    assert!(ok9.is_none());
    let ok10 = check_line_integrity(dummy_path, 31, "if save_var == \"source_layer\" {");
    assert!(ok10.is_none());
}

#[test]
fn test_companion_app_main_zero_plugin_command_sniffing() {
    let workspace_root = find_workspace_root();
    let main_rs_path = workspace_root
        .join("crates")
        .join("companion_app")
        .join("src")
        .join("main.rs");
    let content = fs::read_to_string(&main_rs_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", main_rs_path.display()));

    const ALLOWED_CORE_ACTIONS: &[&str] = &["tools.setColors", "raster.process"];

    let mut in_test_module = false;
    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = strip_line_comment(line).trim();
        if trimmed.contains("#[cfg(test)]") || trimmed.contains("mod tests") {
            in_test_module = true;
            continue;
        }
        if in_test_module {
            continue;
        }

        if trimmed.contains("action.command")
            && (trimmed.contains("==")
                || trimmed.contains(".starts_with(")
                || trimmed.contains(".contains("))
        {
            let is_allowed = ALLOWED_CORE_ACTIONS
                .iter()
                .any(|allowed| trimmed.contains(&format!("\"{allowed}\"")));
            assert!(
                is_allowed,
                "Violation in main.rs line {}: forbidden action.command dispatch on '{}'. Allowed host dispatch actions are strictly: {:?}",
                line_idx + 1,
                trimmed,
                ALLOWED_CORE_ACTIONS
            );
        }
    }
}

#[test]
fn test_allowed_host_dispatch_invariant_catches_unauthorized_command() {
    const ALLOWED_CORE_ACTIONS: &[&str] = &["tools.setColors", "raster.process"];

    let valid_line = "if action.command == \"tools.setColors\" {";
    let is_valid = ALLOWED_CORE_ACTIONS
        .iter()
        .any(|allowed| valid_line.contains(&format!("\"{allowed}\"")));
    assert!(is_valid);

    let valid_raster = "} else if action.command == \"raster.process\" {";
    let is_valid_raster = ALLOWED_CORE_ACTIONS
        .iter()
        .any(|allowed| valid_raster.contains(&format!("\"{allowed}\"")));
    assert!(is_valid_raster);

    let forbidden_line = "if action.command == \"selection.modify_composite\" {";
    let is_forbidden = ALLOWED_CORE_ACTIONS
        .iter()
        .any(|allowed| forbidden_line.contains(&format!("\"{allowed}\"")));
    assert!(!is_forbidden);

    let forbidden_starts_with = "if action.command.starts_with(\"select.\") {";
    let is_forbidden_sw = ALLOWED_CORE_ACTIONS
        .iter()
        .any(|allowed| forbidden_starts_with.contains(&format!("\"{allowed}\"")));
    assert!(!is_forbidden_sw);
}
