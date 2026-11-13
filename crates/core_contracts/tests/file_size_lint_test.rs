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

/// Strip #[cfg(test)] modules to calculate pure production lines
fn count_production_lines(content: &str) -> usize {
    let mut prod_lines = 0;
    let mut in_test_cfg = false;
    let mut brace_depth: i32 = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("#[cfg(test)]") {
            in_test_cfg = true;
            brace_depth = 0;
            continue;
        }

        if in_test_cfg {
            for ch in trimmed.chars() {
                if ch == '{' {
                    brace_depth += 1;
                } else if ch == '}' {
                    brace_depth -= 1;
                    if brace_depth <= 0 {
                        in_test_cfg = false;
                        brace_depth = 0;
                    }
                }
            }
            continue;
        }

        prod_lines += 1;
    }

    prod_lines
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
                if dir_name != "target" && !dir_name.starts_with('.') {
                    collect_rs_files(&path, out);
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                out.push(path);
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum FileTier {
    ProductionLogic,
    ModuleRoot,
    TestFile,
    StaticDataTable,
}

impl FileTier {
    fn max_allowed_lines(&self) -> usize {
        match self {
            FileTier::ProductionLogic => 800,
            FileTier::ModuleRoot => 800,
            FileTier::TestFile => 1200,
            FileTier::StaticDataTable => 2000,
        }
    }
}

fn classify_file(rel_path: &str, file_name: &str) -> FileTier {
    if rel_path.contains("/tests/")
        || rel_path.starts_with("tests/")
        || file_name == "tests.rs"
        || file_name.ends_with("_test.rs")
        || file_name.ends_with("_tests.rs")
    {
        FileTier::TestFile
    } else if rel_path.ends_with("named_colors.rs")
        || rel_path.contains("table_part")
        || rel_path.contains("dictionary")
    {
        FileTier::StaticDataTable
    } else if file_name == "mod.rs" || file_name == "lib.rs" {
        FileTier::ModuleRoot
    } else {
        FileTier::ProductionLogic
    }
}

/// Backlog of legacy monolithic files undergoing phased decomposition (Plan 002-modular-codebase-refactoring)
/// ALL monolithic files have been fully refactored into focused modular directory packages!
const REFACTORING_BACKLOG: &[&str] = &[];

#[test]
fn test_code_file_size_and_modular_architecture_guard() {
    let root = find_workspace_root();
    let crates_dir = root.join("crates");

    let mut files = Vec::new();
    collect_rs_files(&crates_dir, &mut files);

    let mut violations = Vec::new();
    let mut warnings = Vec::new();

    for file in files {
        let rel_path = file
            .strip_prefix(&root)
            .unwrap_or(&file)
            .to_string_lossy()
            .replace('\\', "/");
        let file_name = file
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let content = fs::read_to_string(&file).expect("Failed to read file");
        let tier = classify_file(&rel_path, &file_name);
        let max_allowed = tier.max_allowed_lines();

        let line_count = if tier == FileTier::TestFile {
            content.lines().count()
        } else {
            count_production_lines(&content)
        };

        if REFACTORING_BACKLOG.contains(&rel_path.as_str()) {
            // Documented backlog file scheduled for decomposition in 002-modular-codebase-refactoring
            continue;
        }

        if line_count > max_allowed {
            violations.push(format!(
                "  ❌ {} ({:?}): {} lines (exceeds hard cap of {} lines!)",
                rel_path, tier, line_count, max_allowed
            ));
        } else if tier == FileTier::ProductionLogic && line_count > 500 {
            warnings.push(format!(
                "  ⚠️  {} ({:?}): {} lines (approaching limit, target is <= 500 lines)",
                rel_path, tier, line_count
            ));
        }
    }

    if !warnings.is_empty() {
        println!("\n[Code File Size Notice - Files approaching threshold (500-800 lines)]:");
        for w in &warnings {
            println!("{w}");
        }
    }

    if !violations.is_empty() {
        panic!(
            "\n[RULE 10 VIOLATION: Code File Size Guard]\nFound files exceeding hard limit (800 lines):\n{}\n\nAction required: Decompose bloated files into submodules (`<file>/mod.rs` + components) according to Spec 002.\n",
            violations.join("\n")
        );
    }
}
