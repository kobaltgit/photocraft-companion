use crate::catalog::build_registry_index;
use crate::models::ValidatedPlugin;
use crate::validator::{validate_plugin_readme, validate_single_plugin, validate_svg_icon};
use core_contracts::PluginManifest;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_validate_svg_icon_pure_vector() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10"/></svg>"#;
    let tmp_file = std::env::temp_dir().join("test_pure_vector.svg");
    fs::write(&tmp_file, svg).unwrap();
    assert!(validate_svg_icon(&tmp_file).is_ok());
    let _ = fs::remove_file(tmp_file);
}

#[test]
fn test_validate_svg_icon_rejects_embedded_raster() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><image href="data:image/png;base64,iVBORw0KGgo"/></svg>"#;
    let tmp_file = std::env::temp_dir().join("test_raster.svg");
    fs::write(&tmp_file, svg).unwrap();
    assert!(validate_svg_icon(&tmp_file).is_err());
    let _ = fs::remove_file(tmp_file);
}

#[test]
fn test_validate_svg_icon_rejects_raster_in_attributes() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg"><rect fill="url(data:image/jpeg;base64,abc)"/></svg>"#;
    let tmp_file = std::env::temp_dir().join("test_raster_attr.svg");
    fs::write(&tmp_file, svg).unwrap();
    assert!(validate_svg_icon(&tmp_file).is_err());
    let _ = fs::remove_file(tmp_file);
}

#[test]
fn test_registry_index_generation() {
    let manifest = PluginManifest {
        id: "test.plugin".to_string(),
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "Test desc".to_string(),
        author: "Tester".to_string(),
        plugin_type: core_contracts::PluginType::NoCode,
        icon: Some("icon.svg".to_string()),
        commands: vec![],
        post_commands: vec![],
        sub_actions: vec![],
        input: core_contracts::PluginInputSource::ActiveLayer,
        output_mode: core_contracts::PluginOutputMode::InPlace,
        models: vec![],
        action_id: None,
        name_ru: None,
        description_ru: None,
        locales: Default::default(),
        entrypoint: None,
        ai: None,
        algorithm: None,
        interactive_ui: None,
        runtime_tools: vec![],
        enabled: true,
        plugin_dir: None,
    };

    let validated = vec![ValidatedPlugin {
        dir_name: "test_plugin".to_string(),
        dir_path: PathBuf::from("plugins/test_plugin"),
        manifest,
        icon_path: Some(PathBuf::from("plugins/test_plugin/icon.svg")),
    }];

    let index = build_registry_index(&validated, "https://github.com/test/repo");
    assert_eq!(index.total_plugins, 1);
    assert_eq!(index.plugins[0].id, "test.plugin");
    assert_eq!(
        index.plugins[0].download_url,
        "https://github.com/test/repo/releases/download/plugins/test_plugin.pcpl"
    );
    assert_eq!(
        index.plugins[0].icon_url,
        Some("https://github.com/test/repo/raw/main/plugins/test_plugin/icon.svg".to_string())
    );
}

#[test]
fn test_validate_plugin_readme_missing() {
    let tmp_dir = std::env::temp_dir().join("test_plugin_no_readme");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).unwrap();

    let result = validate_plugin_readme(&tmp_dir, None);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Missing mandatory 'README.md'"));

    let _ = fs::remove_dir_all(tmp_dir);
}

#[test]
fn test_validate_plugin_readme_success() {
    let tmp_dir = std::env::temp_dir().join("test_plugin_valid_readme");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).unwrap();

    let readme_content = r#"# My Plugin
**English:**
Awesome plugin description for tests.
### Use Cases:
- Case 1: Editing layers

---
**Русский:**
Отличный плагин для тестирования валидатора.
### Практическое применение:
- Сценарий 1: Обработка слоев
"#;
    fs::write(tmp_dir.join("README.md"), readme_content).unwrap();

    assert!(validate_plugin_readme(&tmp_dir, None).is_ok());

    let _ = fs::remove_dir_all(tmp_dir);
}

#[test]
fn test_validate_plugin_readme_rejects_monolingual() {
    let tmp_dir = std::env::temp_dir().join("test_plugin_mono_readme");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).unwrap();

    // Only English, no Russian
    let readme_content =
        "# My Plugin\n**English:** Only english description here that is long enough.";
    fs::write(tmp_dir.join("README.md"), readme_content).unwrap();

    let result = validate_plugin_readme(&tmp_dir, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Russian"));

    let _ = fs::remove_dir_all(tmp_dir);
}

#[test]
fn test_validate_plugin_readme_native_ai_rejects_missing_weights_section() {
    let tmp_dir = std::env::temp_dir().join("test_plugin_ai_no_weights");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).unwrap();

    let readme_content = r#"# My AI Plugin
**English:**
Awesome AI plugin description for tests.
### Use Cases:
- Case 1: Processing images

---
**Русский:**
Отличный ИИ-плагин для тестирования валидатора.
### Практическое применение:
- Сценарий 1: Обработка изображений
"#;
    fs::write(tmp_dir.join("README.md"), readme_content).unwrap();

    let manifest = PluginManifest {
        id: "test.ai".to_string(),
        name: "Test AI".to_string(),
        version: "1.0.0".to_string(),
        description: "Test".to_string(),
        author: "Tester".to_string(),
        plugin_type: core_contracts::PluginType::NativeAi,
        icon: None,
        commands: vec![],
        post_commands: vec![],
        sub_actions: vec![],
        input: core_contracts::PluginInputSource::ActiveLayer,
        output_mode: core_contracts::PluginOutputMode::InPlace,
        models: vec![core_contracts::ModelRequirement {
            name: "my_model.onnx".to_string(),
            role: None,
            required: true,
            size_mb: Some(50),
            url: Some("https://example.com/my_model.onnx".to_string()),
            sha256: None,
        }],
        action_id: None,
        name_ru: None,
        description_ru: None,
        locales: Default::default(),
        entrypoint: None,
        ai: None,
        algorithm: None,
        interactive_ui: None,
        runtime_tools: vec![],
        enabled: true,
        plugin_dir: None,
    };

    let result = validate_plugin_readme(&tmp_dir, Some(&manifest));
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("must contain a model weights section in English"));

    let _ = fs::remove_dir_all(tmp_dir);
}

#[test]
fn test_validate_plugin_readme_native_ai_rejects_missing_model_reference() {
    let tmp_dir = std::env::temp_dir().join("test_plugin_ai_missing_model_ref");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).unwrap();

    let readme_content = r#"# My AI Plugin
**English:**
Awesome AI plugin description for tests.
### Use Cases:
- Case 1: Processing images
### Model Weights:
- [other_model.onnx](https://example.com/other.onnx)

---
**Русский:**
Отличный ИИ-плагин для тестирования валидатора.
### Практическое применение:
- Сценарий 1: Обработка изображений
### Ссылки на веса модели:
- [other_model.onnx](https://example.com/other.onnx)
"#;
    fs::write(tmp_dir.join("README.md"), readme_content).unwrap();

    let manifest = PluginManifest {
        id: "test.ai".to_string(),
        name: "Test AI".to_string(),
        version: "1.0.0".to_string(),
        description: "Test".to_string(),
        author: "Tester".to_string(),
        plugin_type: core_contracts::PluginType::NativeAi,
        icon: None,
        commands: vec![],
        post_commands: vec![],
        sub_actions: vec![],
        input: core_contracts::PluginInputSource::ActiveLayer,
        output_mode: core_contracts::PluginOutputMode::InPlace,
        models: vec![core_contracts::ModelRequirement {
            name: "my_model.onnx".to_string(),
            role: None,
            required: true,
            size_mb: Some(50),
            url: Some("https://example.com/my_model.onnx".to_string()),
            sha256: None,
        }],
        action_id: None,
        name_ru: None,
        description_ru: None,
        locales: Default::default(),
        entrypoint: None,
        ai: None,
        algorithm: None,
        interactive_ui: None,
        runtime_tools: vec![],
        enabled: true,
        plugin_dir: None,
    };

    let result = validate_plugin_readme(&tmp_dir, Some(&manifest));
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("does not mention model 'my_model.onnx'"));

    let _ = fs::remove_dir_all(tmp_dir);
}

#[test]
fn test_validate_plugin_readme_native_ai_success() {
    let tmp_dir = std::env::temp_dir().join("test_plugin_ai_readme_ok");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).unwrap();

    let readme_content = r#"# My AI Plugin
**English:**
Awesome AI plugin description for tests.
### Use Cases:
- Case 1: Processing images
### Model Weights:
- [my_model.onnx](https://example.com/my_model.onnx)

---
**Русский:**
Отличный ИИ-плагин для тестирования валидатора.
### Практическое применение:
- Сценарий 1: Обработка изображений
### Ссылки на веса модели:
- [my_model.onnx](https://example.com/my_model.onnx)
"#;
    fs::write(tmp_dir.join("README.md"), readme_content).unwrap();

    let manifest = PluginManifest {
        id: "test.ai".to_string(),
        name: "Test AI".to_string(),
        version: "1.0.0".to_string(),
        description: "Test".to_string(),
        author: "Tester".to_string(),
        plugin_type: core_contracts::PluginType::NativeAi,
        icon: None,
        commands: vec![],
        post_commands: vec![],
        sub_actions: vec![],
        input: core_contracts::PluginInputSource::ActiveLayer,
        output_mode: core_contracts::PluginOutputMode::InPlace,
        models: vec![core_contracts::ModelRequirement {
            name: "my_model.onnx".to_string(),
            role: None,
            required: true,
            size_mb: Some(50),
            url: Some("https://example.com/my_model.onnx".to_string()),
            sha256: None,
        }],
        action_id: None,
        name_ru: None,
        description_ru: None,
        locales: Default::default(),
        entrypoint: None,
        ai: None,
        algorithm: None,
        interactive_ui: None,
        runtime_tools: vec![],
        enabled: true,
        plugin_dir: None,
    };

    assert!(validate_plugin_readme(&tmp_dir, Some(&manifest)).is_ok());

    let _ = fs::remove_dir_all(tmp_dir);
}

#[test]
fn test_validate_plugin_rejects_native_ai_with_all_optional_models() {
    let tmp_dir = std::env::temp_dir().join("test_plugin_all_optional");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).unwrap();

    let readme_content = "# Test\n**English:** Test desc\n### Use Cases:\n- C1\n**Русский:** Тест\n### Практическое применение:\n- С1\n";
    fs::write(tmp_dir.join("README.md"), readme_content).unwrap();

    let manifest_content = r#"{
        "id": "test.native_ai.fake",
        "name": "Fake AI Plugin",
        "version": "1.0.0",
        "description": "Testing validator bypass",
        "plugin_type": "native_ai",
        "models": [
            {
                "name": "fake_model.onnx",
                "required": false
            }
        ]
    }"#;
    fs::write(tmp_dir.join("plugin.json"), manifest_content).unwrap();

    let result = validate_single_plugin(&tmp_dir, true);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("at least one model must have 'required: true'"));

    let _ = fs::remove_dir_all(tmp_dir);
}

#[test]
fn test_validate_plugin_rejects_native_ai_with_empty_models() {
    let tmp_dir = std::env::temp_dir().join("test_plugin_empty_models");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).unwrap();

    let readme_content = "# Test\n**English:** Test desc\n### Use Cases:\n- C1\n**Русский:** Тест\n### Практическое применение:\n- С1\n";
    fs::write(tmp_dir.join("README.md"), readme_content).unwrap();

    let manifest_content = r#"{
        "id": "test.native_ai.empty",
        "name": "Empty AI Plugin",
        "version": "1.0.0",
        "description": "Testing empty models",
        "plugin_type": "native_ai",
        "models": []
    }"#;
    fs::write(tmp_dir.join("plugin.json"), manifest_content).unwrap();

    let result = validate_single_plugin(&tmp_dir, true);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("must declare at least one model in 'models'"));

    let _ = fs::remove_dir_all(tmp_dir);
}

#[test]
fn test_validate_plugin_rejects_cross_plugin_path_traversal() {
    let tmp_dir = std::env::temp_dir().join("test_plugin_path_traversal");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).unwrap();

    let readme_content = "# Test\n**English:** Test desc\n### Use Cases:\n- C1\n**Русский:** Тест\n### Практическое применение:\n- С1\n";
    fs::write(tmp_dir.join("README.md"), readme_content).unwrap();

    let manifest_content = r#"{
        "id": "test.native_ai.traversal",
        "name": "Traversal AI Plugin",
        "version": "1.0.0",
        "description": "Testing path traversal",
        "plugin_type": "native_ai",
        "models": [
            {
                "name": "../other_plugin/models/other.onnx",
                "required": true,
                "url": "https://example.com/other.onnx"
            }
        ]
    }"#;
    fs::write(tmp_dir.join("plugin.json"), manifest_content).unwrap();

    let result = validate_single_plugin(&tmp_dir, true);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("contains forbidden path separators or parent directory traversal"));

    let _ = fs::remove_dir_all(tmp_dir);
}

#[test]
fn test_validate_plugin_rejects_native_ai_without_model_url() {
    let tmp_dir = std::env::temp_dir().join("test_plugin_missing_url");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).unwrap();

    let readme_content = "# Test\n**English:** Test desc\n### Use Cases:\n- C1\n**Русский:** Тест\n### Практическое применение:\n- С1\n";
    fs::write(tmp_dir.join("README.md"), readme_content).unwrap();

    let manifest_content = r#"{
        "id": "test.native_ai.no_url",
        "name": "No URL AI Plugin",
        "version": "1.0.0",
        "description": "Testing missing model url",
        "plugin_type": "native_ai",
        "models": [
            {
                "name": "isolated_model.onnx",
                "required": true
            }
        ]
    }"#;
    fs::write(tmp_dir.join("plugin.json"), manifest_content).unwrap();

    let result = validate_single_plugin(&tmp_dir, true);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("is missing a download 'url'. Every plugin must be self-contained"));

    let _ = fs::remove_dir_all(tmp_dir);
}

#[test]
fn test_validate_sub_actions_success_and_invalid() {
    let tmp_dir = std::env::temp_dir().join("test_plugin_sub_actions");
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).unwrap();

    let readme_content = "# Test\n**English:** Test desc\n### Use Cases:\n- C1\n**Русский:** Тест\n### Практическое применение:\n- С1\n";
    fs::write(tmp_dir.join("README.md"), readme_content).unwrap();

    let manifest_invalid = r#"{
        "id": "test.sub.invalid",
        "name": "Sub Plugin",
        "version": "1.0.0",
        "description": "Testing sub-actions",
        "plugin_type": "no_code",
        "sub_actions": [
            {
                "id": "",
                "name": "Invalid"
            }
        ]
    }"#;
    fs::write(tmp_dir.join("plugin.json"), manifest_invalid).unwrap();
    let res = validate_single_plugin(&tmp_dir, true);
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("Sub-action has empty 'id'"));

    let _ = fs::remove_dir_all(tmp_dir);
}
