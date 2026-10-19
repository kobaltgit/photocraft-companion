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

/// Scan a file for integrity violations, skipping #[cfg(test)] modules
fn scan_file_for_violations(file_path: &Path) -> Vec<IntegrityViolation> {
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
}
