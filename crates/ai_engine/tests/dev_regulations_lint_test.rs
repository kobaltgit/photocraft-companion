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

// =========================================================================
// 1. App Version Synchronization Guard (Rule 7.B)
// =========================================================================

#[test]
fn test_app_version_synchronized_across_all_release_files() {
    let root = find_workspace_root();

    // 1. Cargo.toml [workspace.package] version
    let cargo_toml_path = root.join("Cargo.toml");
    let cargo_content = fs::read_to_string(&cargo_toml_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", cargo_toml_path.display()));
    let mut cargo_version = None;
    let mut in_workspace_package = false;
    for line in cargo_content.lines() {
        let trimmed = line.trim();
        if trimmed == "[workspace.package]" {
            in_workspace_package = true;
            continue;
        }
        if in_workspace_package && trimmed.starts_with('[') {
            break;
        }
        if in_workspace_package && trimmed.starts_with("version") {
            if let Some(val) = trimmed.split('=').nth(1) {
                cargo_version = Some(val.trim().trim_matches('"').to_string());
                break;
            }
        }
    }
    let version_cargo = cargo_version.expect("Could not extract version from Cargo.toml");

    // 2. scripts/package_release.ps1: $Version = "X.Y.Z"
    let ps1_path = root.join("scripts").join("package_release.ps1");
    let ps1_content = fs::read_to_string(&ps1_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", ps1_path.display()));
    let mut ps1_version = None;
    for line in ps1_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("$Version") {
            if let Some(val) = trimmed.split('=').nth(1) {
                ps1_version = Some(val.trim().trim_matches('"').to_string());
                break;
            }
        }
    }
    let version_ps1 =
        ps1_version.expect("Could not extract $Version from scripts/package_release.ps1");

    // 3. scripts/package_release.sh: VERSION="X.Y.Z"
    let sh_path = root.join("scripts").join("package_release.sh");
    let sh_content = fs::read_to_string(&sh_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", sh_path.display()));
    let mut sh_version = None;
    for line in sh_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("VERSION=") {
            if let Some(val) = trimmed.split('=').nth(1) {
                sh_version = Some(val.trim().trim_matches('"').to_string());
                break;
            }
        }
    }
    let version_sh = sh_version.expect("Could not extract VERSION from scripts/package_release.sh");

    // 4. packaging/windows/installer.iss: #define MyAppVersion "X.Y.Z"
    let iss_path = root.join("packaging").join("windows").join("installer.iss");
    let iss_content = fs::read_to_string(&iss_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", iss_path.display()));
    let mut iss_version = None;
    for line in iss_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#define MyAppVersion") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 {
                iss_version = Some(parts[2].trim_matches('"').to_string());
                break;
            }
        }
    }
    let version_iss = iss_version
        .expect("Could not extract #define MyAppVersion from packaging/windows/installer.iss");

    // 5. packaging/macos/Info.plist: <key>CFBundleShortVersionString</key>\n<string>X.Y.Z</string>
    let plist_path = root.join("packaging").join("macos").join("Info.plist");
    let plist_content = fs::read_to_string(&plist_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", plist_path.display()));
    let mut plist_version = None;
    let mut next_is_version = false;
    for line in plist_content.lines() {
        let trimmed = line.trim();
        if trimmed.contains("<key>CFBundleShortVersionString</key>") {
            next_is_version = true;
            continue;
        }
        if next_is_version {
            if let Some(start) = trimmed.find("<string>") {
                if let Some(end) = trimmed.find("</string>") {
                    plist_version = Some(trimmed[start + 8..end].to_string());
                    break;
                }
            }
        }
    }
    let version_plist = plist_version
        .expect("Could not extract CFBundleShortVersionString from packaging/macos/Info.plist");

    assert_eq!(
        version_cargo, version_ps1,
        "Version mismatch: Cargo.toml has '{version_cargo}' but package_release.ps1 has '{version_ps1}'"
    );
    assert_eq!(
        version_cargo, version_sh,
        "Version mismatch: Cargo.toml has '{version_cargo}' but package_release.sh has '{version_sh}'"
    );
    assert_eq!(
        version_cargo, version_iss,
        "Version mismatch: Cargo.toml has '{version_cargo}' but installer.iss has '{version_iss}'"
    );
    assert_eq!(
        version_cargo, version_plist,
        "Version mismatch: Cargo.toml has '{version_cargo}' but Info.plist has '{version_plist}'"
    );
}

// =========================================================================
// 2. Crate Package Dependency Isolation Guard (AGENTS_MASTER_GUIDE.md §2.1)
// =========================================================================

#[test]
fn test_crate_dependency_isolation() {
    let root = find_workspace_root();
    let crates_dir = root.join("crates");

    fn extract_dependencies(cargo_path: &Path) -> Vec<String> {
        let content = fs::read_to_string(cargo_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", cargo_path.display()));
        let mut deps = Vec::new();
        let mut in_deps_section = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let section = trimmed.trim_matches('[').trim_matches(']');
                in_deps_section = section == "dependencies"
                    || section.ends_with(".dependencies")
                    || section == "dev-dependencies";
                continue;
            }
            if in_deps_section && !trimmed.is_empty() && !trimmed.starts_with('#') {
                if let Some(dep_name) = trimmed.split('=').next() {
                    let clean = dep_name.trim();
                    if !clean.is_empty() {
                        deps.push(clean.to_string());
                    }
                }
            }
        }
        deps
    }

    // core_contracts MUST NOT depend on any internal workspace crates
    let core_deps = extract_dependencies(&crates_dir.join("core_contracts").join("Cargo.toml"));
    for forbidden in &[
        "photocraft_bridge",
        "ai_engine",
        "overlay_ui",
        "companion_app",
        "plugin_validator",
    ] {
        assert!(
            !core_deps.contains(&forbidden.to_string()),
            "Violation: core_contracts must have zero internal crate dependencies, but depends on '{forbidden}'"
        );
    }

    // photocraft_bridge MUST NOT depend on ai_engine or overlay_ui
    let bridge_deps =
        extract_dependencies(&crates_dir.join("photocraft_bridge").join("Cargo.toml"));
    for forbidden in &[
        "ai_engine",
        "overlay_ui",
        "companion_app",
        "plugin_validator",
    ] {
        assert!(
            !bridge_deps.contains(&forbidden.to_string()),
            "Violation: photocraft_bridge must not depend on '{forbidden}'"
        );
    }

    // ai_engine MUST NOT depend on photocraft_bridge or overlay_ui
    let ai_deps = extract_dependencies(&crates_dir.join("ai_engine").join("Cargo.toml"));
    for forbidden in &[
        "photocraft_bridge",
        "overlay_ui",
        "companion_app",
        "plugin_validator",
    ] {
        assert!(
            !ai_deps.contains(&forbidden.to_string()),
            "Violation: ai_engine must not depend on '{forbidden}'"
        );
    }

    // overlay_ui MUST NOT depend on photocraft_bridge or ai_engine
    let ui_deps = extract_dependencies(&crates_dir.join("overlay_ui").join("Cargo.toml"));
    for forbidden in &[
        "photocraft_bridge",
        "ai_engine",
        "companion_app",
        "plugin_validator",
    ] {
        assert!(
            !ui_deps.contains(&forbidden.to_string()),
            "Violation: overlay_ui must not depend on '{forbidden}'"
        );
    }
}

// =========================================================================
// 3. Resource Guard Configuration & Release Scripts (Rule 1)
// =========================================================================

#[test]
fn test_resource_guard_config_and_release_scripts() {
    let root = find_workspace_root();

    // 1. .cargo/config.toml
    let cargo_config_path = root.join(".cargo").join("config.toml");
    assert!(
        cargo_config_path.exists(),
        "Resource Guard violation: .cargo/config.toml is missing!"
    );
    let cargo_config = fs::read_to_string(&cargo_config_path).unwrap();
    assert!(
        cargo_config.contains("jobs = 1"),
        "Resource Guard violation: .cargo/config.toml must specify 'jobs = 1'"
    );
    assert!(
        cargo_config.contains("codegen-units=1"),
        "Resource Guard violation: .cargo/config.toml must specify 'codegen-units=1'"
    );

    // 2. Cargo.toml profiles
    let root_cargo = fs::read_to_string(root.join("Cargo.toml")).unwrap();
    assert!(
        root_cargo.contains("[profile.dev]"),
        "Root Cargo.toml must declare [profile.dev]"
    );
    assert!(
        root_cargo.contains("[profile.test]"),
        "Root Cargo.toml must declare [profile.test]"
    );
    assert!(
        root_cargo.contains("[profile.release]"),
        "Root Cargo.toml must declare [profile.release]"
    );

    // Count occurrences of codegen-units = 1
    let codegen_count = root_cargo
        .lines()
        .filter(|line| line.trim().replace(' ', "") == "codegen-units=1")
        .count();
    assert!(
        codegen_count >= 3,
        "Resource Guard violation: Cargo.toml must have 'codegen-units = 1' across dev, test, and release profiles (found {codegen_count})"
    );

    // 3. scripts/package_release.ps1: every cargo command must have -j 1
    let ps1_path = root.join("scripts").join("package_release.ps1");
    let ps1_content = fs::read_to_string(&ps1_path).unwrap();
    for (idx, line) in ps1_content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("cargo ") {
            assert!(
                trimmed.contains("-j 1") || trimmed.contains("-j1"),
                "Resource Guard violation in scripts/package_release.ps1:{}: cargo command missing '-j 1':\n    {}",
                idx + 1,
                trimmed
            );
        }
    }

    // 4. scripts/package_release.sh: every cargo command must have -j 1
    let sh_path = root.join("scripts").join("package_release.sh");
    let sh_content = fs::read_to_string(&sh_path).unwrap();
    for (idx, line) in sh_content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("cargo ") {
            assert!(
                trimmed.contains("-j 1") || trimmed.contains("-j1"),
                "Resource Guard violation in scripts/package_release.sh:{}: cargo command missing '-j 1':\n    {}",
                idx + 1,
                trimmed
            );
        }
    }
}

// =========================================================================
// 4. Chronicle Bilingual Synchronization Guard (Rule 2.2)
// =========================================================================

#[test]
fn test_chronicle_bilingual_synchronization() {
    let root = find_workspace_root();
    let ru_path = root.join("docs").join("CHRONICLE.md");
    let en_path = root.join("docs").join("CHRONICLE.en.md");

    assert!(ru_path.exists(), "docs/CHRONICLE.md is missing!");
    assert!(en_path.exists(), "docs/CHRONICLE.en.md is missing!");

    let ru_content = fs::read_to_string(&ru_path).unwrap();
    let en_content = fs::read_to_string(&en_path).unwrap();

    fn extract_chapters(content: &str, keyword: &str) -> Vec<(usize, String)> {
        let mut chapters = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(stripped) = trimmed.strip_prefix("## ") {
                let rest = stripped.trim();
                // Find "Глава" or "Chapter"
                if let Some(pos) = rest.find(keyword) {
                    let after = &rest[pos + keyword.len()..];
                    let num_str: String = after
                        .chars()
                        .skip_while(|c| c.is_whitespace() || *c == ':')
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    if let Ok(num) = num_str.parse::<usize>() {
                        chapters.push((num, trimmed.to_string()));
                    }
                }
            }
        }
        chapters
    }

    let ru_chapters = extract_chapters(&ru_content, "Глава");
    let en_chapters = extract_chapters(&en_content, "Chapter");

    assert!(
        !ru_chapters.is_empty(),
        "Failed to extract chapters from docs/CHRONICLE.md"
    );
    assert!(
        !en_chapters.is_empty(),
        "Failed to extract chapters from docs/CHRONICLE.en.md"
    );

    assert_eq!(
        ru_chapters.len(),
        en_chapters.len(),
        "Chronicle desynchronization: docs/CHRONICLE.md has {} chapters, but docs/CHRONICLE.en.md has {} chapters!",
        ru_chapters.len(),
        en_chapters.len()
    );

    for i in 0..ru_chapters.len() {
        let (ru_num, ref ru_header) = ru_chapters[i];
        let (en_num, ref en_header) = en_chapters[i];
        assert_eq!(
            ru_num, en_num,
            "Chronicle chapter number mismatch at index {i}:\n  RU: {ru_header}\n  EN: {en_header}"
        );
    }
}

// =========================================================================
// 5. Plugin Catalog Drift Detection (Rule 7.A)
// =========================================================================

#[test]
fn test_readme_plugins_catalog_not_drifted() {
    let root = find_workspace_root();
    let plugins_dir = root.join("plugins");
    let readme_en_path = root.join("README.md");
    let readme_ru_path = root.join("README.ru.md");

    assert!(readme_en_path.exists(), "README.md is missing!");
    assert!(readme_ru_path.exists(), "README.ru.md is missing!");

    let readme_en = fs::read_to_string(&readme_en_path).unwrap();
    let readme_ru = fs::read_to_string(&readme_ru_path).unwrap();

    const START_TAG: &str = "<!-- PLUGINS_CATALOG_START -->";
    const END_TAG: &str = "<!-- PLUGINS_CATALOG_END -->";

    fn extract_catalog_section<'a>(content: &'a str, filename: &str) -> &'a str {
        let start = content.find(START_TAG).unwrap_or_else(|| {
            panic!("Missing '{START_TAG}' in {filename}");
        });
        let end = content.find(END_TAG).unwrap_or_else(|| {
            panic!("Missing '{END_TAG}' in {filename}");
        });
        assert!(
            start < end,
            "Malformed catalog tags in {filename}: start must precede end"
        );
        &content[start + START_TAG.len()..end]
    }

    let en_catalog = extract_catalog_section(&readme_en, "README.md");
    let ru_catalog = extract_catalog_section(&readme_ru, "README.ru.md");

    // Scan all plugin folders
    let entries = fs::read_dir(&plugins_dir).unwrap();
    let mut plugin_dirs = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("plugin.json").exists() {
            let dir_name = path.file_name().unwrap().to_string_lossy().to_string();
            plugin_dirs.push(dir_name);
        }
    }
    plugin_dirs.sort();

    assert!(
        !plugin_dirs.is_empty(),
        "Expected at least 1 plugin in plugins/"
    );

    for dir_name in &plugin_dirs {
        assert!(
            en_catalog.contains(dir_name),
            "Drift detected: Plugin '{dir_name}' exists in plugins/ but is missing from README.md catalog table! Run plugin_validator to update."
        );
        assert!(
            ru_catalog.contains(dir_name),
            "Drift detected: Plugin '{dir_name}' exists in plugins/ but is missing from README.ru.md catalog table! Run plugin_validator to update."
        );
    }
}

// =========================================================================
// 6. Git Model Weights Ban Guard (.gitignore Guard - Rule 7.A)
// =========================================================================

#[test]
fn test_git_model_weights_prohibition() {
    let root = find_workspace_root();
    let gitignore_path = root.join(".gitignore");
    assert!(gitignore_path.exists(), ".gitignore file is missing!");

    let gitignore = fs::read_to_string(&gitignore_path).unwrap();

    // Verify all neural model weight extensions are banned in .gitignore
    for ext in &["*.onnx", "*.safetensors", "*.bin", "*.pt", "*.pth"] {
        assert!(
            gitignore.contains(ext),
            "Violation: .gitignore must contain ignore pattern for heavy weights '{ext}'"
        );
    }
}

// =========================================================================
// 7. Plugin Model Isolation and Self-Containment Guard
// =========================================================================

#[test]
fn test_plugin_model_isolation_and_self_containment() {
    let root = find_workspace_root();
    let plugins_dir = root.join("plugins");
    assert!(plugins_dir.exists(), "plugins/ directory missing!");

    // 1. Audit all registered plugins in plugins/
    let entries = fs::read_dir(&plugins_dir).unwrap();
    let mut checked_plugins = 0;
    for entry in entries.flatten() {
        let p_path = entry.path();
        let manifest_path = p_path.join("plugin.json");
        if !manifest_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&manifest_path).unwrap();
        let manifest: core_contracts::PluginManifest = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {e}", manifest_path.display()));

        // Check self-containment of models
        for model in &manifest.models {
            // Model name must not contain directory traversal or separators
            assert!(
                !model.name.contains(".."),
                "Model isolation violation in plugin '{}': model '{}' contains directory traversal '..'",
                manifest.id,
                model.name
            );
            assert!(
                !model.name.contains('/') && !model.name.contains('\\'),
                "Model isolation violation in plugin '{}': model '{}' contains path separators. Models must be clean filenames.",
                manifest.id,
                model.name
            );

            // Self-containment: NativeAi models must have their own download URL
            if manifest.plugin_type == core_contracts::PluginType::NativeAi {
                let url = model.url.as_deref().unwrap_or("").trim();
                assert!(
                    !url.is_empty(),
                    "Self-containment violation in plugin '{}': NativeAi model '{}' has empty url",
                    manifest.id,
                    model.name
                );
                assert!(
                    url.starts_with("http://") || url.starts_with("https://"),
                    "Self-containment violation in plugin '{}': model '{}' url must be valid HTTP/HTTPS: {}",
                    manifest.id,
                    model.name,
                    url
                );
            }
        }
        checked_plugins += 1;
    }

    assert!(
        checked_plugins >= 6,
        "Expected at least 6 plugins checked, got {checked_plugins}"
    );

    // 2. Behavioral verification of model isolation at runtime
    let temp_dir =
        std::env::temp_dir().join(format!("photocraft_isolation_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp_dir);
    let plugin_a_dir = temp_dir.join("plugin_alpha");
    let plugin_b_dir = temp_dir.join("plugin_beta");
    let alpha_models_dir = plugin_a_dir.join("models");

    fs::create_dir_all(&alpha_models_dir).unwrap();
    fs::create_dir_all(&plugin_b_dir).unwrap();

    let alpha_model_file = alpha_models_dir.join("model_alpha.onnx");
    fs::write(&alpha_model_file, b"DUMMY_ONNX_WEIGHTS_ALPHA").unwrap();

    // Plugin Alpha manifest
    let alpha_manifest = core_contracts::PluginManifest {
        id: "plugin_alpha".into(),
        name: "Alpha".into(),
        plugin_type: core_contracts::PluginType::NativeAi,
        models: vec![core_contracts::ModelRequirement {
            name: "model_alpha.onnx".into(),
            role: Some("alpha_role".into()),
            url: Some("https://example.com/alpha.onnx".into()),
            sha256: None,
            size_mb: Some(24),
            required: true,
        }],
        plugin_dir: Some(plugin_a_dir.clone()),
        ..Default::default()
    };

    // Plugin Beta manifest (trying to use Plugin Alpha's model)
    let beta_manifest = core_contracts::PluginManifest {
        id: "plugin_beta".into(),
        name: "Beta".into(),
        plugin_type: core_contracts::PluginType::NativeAi,
        models: vec![core_contracts::ModelRequirement {
            name: "model_alpha.onnx".into(),
            role: Some("alpha_role".into()),
            url: Some("https://example.com/alpha.onnx".into()),
            sha256: None,
            size_mb: Some(24),
            required: true,
        }],
        plugin_dir: Some(plugin_b_dir.clone()),
        ..Default::default()
    };

    // Alpha has model installed
    assert!(
        alpha_manifest.is_model_installed(&alpha_manifest.models[0]),
        "Alpha plugin should detect its own model"
    );

    // Beta MUST NOT detect Alpha's model as installed
    assert!(
        !beta_manifest.is_model_installed(&beta_manifest.models[0]),
        "Violation: Plugin Beta must NOT see Plugin Alpha's installed model!"
    );

    // Beta attempting directory traversal must also fail
    let hacked_model = core_contracts::ModelRequirement {
        name: "../plugin_alpha/models/model_alpha.onnx".into(),
        role: Some("hacked".into()),
        url: Some("https://example.com/alpha.onnx".into()),
        sha256: None,
        size_mb: Some(24),
        required: true,
    };
    assert!(
        !beta_manifest.is_model_installed(&hacked_model),
        "Violation: Path traversal to another plugin's model directory must be rejected!"
    );

    // 3. Engine-level resolution isolation
    let engine = ai_engine::AiEngine::new(temp_dir.join("empty_engine_models"));
    let alpha_resolved = engine.resolve_model_path("model_alpha.onnx", Some(&plugin_a_dir));
    assert!(
        alpha_resolved.is_some(),
        "Engine must resolve model for plugin_alpha"
    );

    let beta_resolved = engine.resolve_model_path("model_alpha.onnx", Some(&plugin_b_dir));
    assert!(
        beta_resolved.is_none(),
        "Violation: Engine must NOT resolve plugin_alpha's model when executing plugin_beta!"
    );

    let traversal_resolved = engine.resolve_model_path(
        "../plugin_alpha/models/model_alpha.onnx",
        Some(&plugin_b_dir),
    );
    assert!(
        traversal_resolved.is_none(),
        "Violation: Engine must reject path traversal across plugin directories!"
    );

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}
