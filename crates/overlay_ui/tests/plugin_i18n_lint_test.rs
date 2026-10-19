use std::fs;
use std::path::{Path, PathBuf};

fn collect_files_with_ext(dir: &Path, ext: &str, results: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_files_with_ext(&path, ext, results);
            } else if path.extension().and_then(|s| s.to_str()) == Some(ext) {
                results.push(path);
            }
        }
    }
}

fn find_workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Some(parent) = manifest_dir.parent() {
        if let Some(grandparent) = parent.parent() {
            if grandparent.join("Cargo.toml").exists() {
                return grandparent.to_path_buf();
            }
        }
    }
    PathBuf::from(".")
}

#[test]
fn test_no_double_bilingual_parentheses_in_rhai_scripts() {
    let root = find_workspace_root();
    let plugins_dir = root.join("plugins");
    if !plugins_dir.exists() {
        return;
    }

    let mut rhai_files = Vec::new();
    collect_files_with_ext(&plugins_dir, "rhai", &mut rhai_files);

    for file_path in rhai_files {
        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", file_path.display()));

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }

            // Check for double bilingual pattern inside string literals:
            // e.g. "Emerald Jade (Изумрудный зеленый)"
            let has_latin = trimmed.chars().any(|c| c.is_ascii_alphabetic());
            let has_cyrillic = trimmed
                .chars()
                .any(|c| ('\u{0400}'..='\u{04FF}').contains(&c));

            if has_latin && has_cyrillic && trimmed.contains('(') && trimmed.contains(')') {
                if let Some(open_paren) = trimmed.find('(') {
                    if let Some(close_paren) = trimmed[open_paren..].find(')') {
                        let inside = &trimmed[open_paren + 1..open_paren + close_paren];
                        let inside_has_cyrillic = inside
                            .chars()
                            .any(|c| ('\u{0400}'..='\u{04FF}').contains(&c));
                        let before_paren = &trimmed[..open_paren];
                        let inside_string = before_paren.matches('"').count() % 2 == 1;

                        if inside_string && inside_has_cyrillic {
                            panic!(
                                "Found prohibited double bilingual string literal with Cyrillic parentheses in {}:{}: '{}'. Use t(\"En\", \"Ru\") instead!",
                                file_path.display(),
                                line_num + 1,
                                trimmed
                            );
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn test_interactive_plugin_manifests_localization_integrity() {
    let root = find_workspace_root();
    let plugins_dir = root.join("plugins");
    if !plugins_dir.exists() {
        return;
    }

    let mut manifest_files = Vec::new();
    collect_files_with_ext(&plugins_dir, "json", &mut manifest_files);

    for file_path in manifest_files {
        let content = fs::read_to_string(&file_path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {e}", file_path.display()));

        let manifest: core_contracts::PluginManifest = match serde_json::from_str(&content) {
            Ok(m) => m,
            Err(_) => continue,
        };

        if let Some(interactive_ui) = manifest.interactive_ui {
            for widget in &interactive_ui.widgets {
                check_widget_localization(widget, &file_path);
            }
        }
    }
}

fn check_widget_localization(widget: &core_contracts::WidgetNode, file_path: &Path) {
    match widget {
        core_contracts::WidgetNode::ColorSlot { id, label_ru, .. } => {
            assert!(
                label_ru.is_some() && !label_ru.as_ref().unwrap().trim().is_empty(),
                "Widget ColorSlot '{}' in {} missing 'label_ru'",
                id,
                file_path.display()
            );
            assert!(
                label_ru
                    .as_ref()
                    .unwrap()
                    .chars()
                    .any(|c| ('\u{0400}'..='\u{04FF}').contains(&c)),
                "Widget ColorSlot '{}' in {} 'label_ru' must contain Cyrillic",
                id,
                file_path.display()
            );
        }
        core_contracts::WidgetNode::Slider { id, label_ru, .. } => {
            assert!(
                label_ru.is_some() && !label_ru.as_ref().unwrap().trim().is_empty(),
                "Widget Slider '{}' in {} missing 'label_ru'",
                id,
                file_path.display()
            );
            assert!(
                label_ru
                    .as_ref()
                    .unwrap()
                    .chars()
                    .any(|c| ('\u{0400}'..='\u{04FF}').contains(&c)),
                "Widget Slider '{}' in {} 'label_ru' must contain Cyrillic",
                id,
                file_path.display()
            );
        }
        core_contracts::WidgetNode::Dropdown {
            id,
            options,
            label_ru,
            options_ru,
            ..
        } => {
            assert!(
                label_ru.is_some() && !label_ru.as_ref().unwrap().trim().is_empty(),
                "Widget Dropdown '{}' in {} missing 'label_ru'",
                id,
                file_path.display()
            );
            assert!(
                options_ru.is_some(),
                "Widget Dropdown '{}' in {} missing 'options_ru'",
                id,
                file_path.display()
            );
            let opts_ru = options_ru.as_ref().unwrap();
            assert_eq!(
                options.len(),
                opts_ru.len(),
                "Widget Dropdown '{}' in {} options count mismatch: en={}, ru={}",
                id,
                file_path.display(),
                options.len(),
                opts_ru.len()
            );
        }
        core_contracts::WidgetNode::Toggle { id, label_ru, .. } => {
            assert!(
                label_ru.is_some() && !label_ru.as_ref().unwrap().trim().is_empty(),
                "Widget Toggle '{}' in {} missing 'label_ru'",
                id,
                file_path.display()
            );
            assert!(
                label_ru
                    .as_ref()
                    .unwrap()
                    .chars()
                    .any(|c| ('\u{0400}'..='\u{04FF}').contains(&c)),
                "Widget Toggle '{}' in {} 'label_ru' must contain Cyrillic",
                id,
                file_path.display()
            );
        }
        core_contracts::WidgetNode::Button { id, label_ru, .. } => {
            assert!(
                label_ru.is_some() && !label_ru.as_ref().unwrap().trim().is_empty(),
                "Widget Button '{}' in {} missing 'label_ru'",
                id,
                file_path.display()
            );
            assert!(
                label_ru
                    .as_ref()
                    .unwrap()
                    .chars()
                    .any(|c| ('\u{0400}'..='\u{04FF}').contains(&c)),
                "Widget Button '{}' in {} 'label_ru' must contain Cyrillic",
                id,
                file_path.display()
            );
        }
        core_contracts::WidgetNode::Row { items } => {
            for item in items {
                check_widget_localization(item, file_path);
            }
        }
        _ => {}
    }
}
