use crate::models::ValidatedPlugin;
use anyhow::{bail, Context, Result};
use core_contracts::{PluginManifest, PluginType};
use std::fs;
use std::path::Path;
use std::time::Duration;

pub fn collect_and_validate_plugins(
    plugins_dir: &Path,
    skip_network: bool,
) -> Result<Vec<ValidatedPlugin>> {
    if !plugins_dir.exists() {
        bail!(
            "Plugins directory does not exist: {}",
            plugins_dir.display()
        );
    }

    let mut entries = fs::read_dir(plugins_dir)
        .with_context(|| format!("Failed to read directory: {}", plugins_dir.display()))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect::<Vec<_>>();

    entries.sort_by_key(|e| e.file_name());

    let mut validated = Vec::new();
    let mut total_errors = 0;

    for entry in entries {
        let dir_path = entry.path();
        let dir_name = entry.file_name().to_string_lossy().to_string();

        print!("Checking plugin [{}]... ", dir_name);

        match validate_single_plugin(&dir_path, skip_network) {
            Ok(plugin) => {
                println!("OK (id: {})", plugin.manifest.id);
                validated.push(plugin);
            }
            Err(err) => {
                println!("FAILED ❌");
                eprintln!("  Error in {}: {:#}", dir_path.display(), err);
                total_errors += 1;
            }
        }
    }

    if total_errors > 0 {
        bail!("Validation failed with {} error(s)", total_errors);
    }

    Ok(validated)
}

pub fn validate_single_plugin(dir: &Path, skip_network: bool) -> Result<ValidatedPlugin> {
    let manifest_path = dir.join("plugin.json");
    if !manifest_path.exists() {
        bail!("Missing plugin.json manifest in {}", dir.display());
    }

    let manifest_content = fs::read_to_string(&manifest_path)
        .with_context(|| format!("Failed to read {}", manifest_path.display()))?;

    let manifest: PluginManifest = serde_json::from_str(&manifest_content)
        .with_context(|| format!("Invalid JSON structure in {}", manifest_path.display()))?;

    // Validate Manifest Fields
    if manifest.id.trim().is_empty() {
        bail!("Field 'id' cannot be empty");
    }
    if manifest.name.trim().is_empty() {
        bail!("Field 'name' cannot be empty");
    }
    if manifest.version.trim().is_empty() {
        bail!("Field 'version' cannot be empty");
    }

    // Validate Icon if specified or if icon.svg exists
    let icon_filename = manifest.icon.as_deref().unwrap_or("icon.svg");
    let icon_path = dir.join(icon_filename);

    if icon_path.exists() {
        validate_svg_icon(&icon_path)
            .with_context(|| format!("Icon validation failed for {}", icon_path.display()))?;
    } else if manifest.icon.is_some() {
        bail!(
            "Declared icon '{}' was not found in {}",
            icon_filename,
            dir.display()
        );
    }

    // Validate NativeAi Models
    if manifest.plugin_type == core_contracts::PluginType::NativeAi {
        if manifest.models.is_empty() {
            bail!(
                "NativeAi plugin '{}' must declare at least one model in 'models'",
                manifest.id
            );
        }
        let has_required = manifest.models.iter().any(|m| m.required);
        if !has_required {
            bail!(
                "NativeAi plugin '{}' has invalid model configuration: at least one model must have 'required: true' (all models are marked required: false)",
                manifest.id
            );
        }
    }

    // Validate Models: ensure self-containment and strict isolation
    for model in &manifest.models {
        if model.name.trim().is_empty() {
            bail!("Model requirement has empty 'name'");
        }

        // Strict isolation: model.name must be a simple filename, no path traversal or cross-plugin references
        if model.name.contains('/') || model.name.contains('\\') || model.name.contains("..") {
            bail!(
                "Model '{}' contains forbidden path separators or parent directory traversal. Models must be strictly isolated to the plugin's own directory.",
                model.name
            );
        }

        if manifest.plugin_type == PluginType::NativeAi && model.url.is_none() {
            bail!(
                "Model '{}' in NativeAi plugin is missing a download 'url'. Every plugin must be self-contained and independently downloadable.",
                model.name
            );
        }

        if let Some(ref role) = model.role {
            if role.trim().is_empty() {
                bail!("Model '{}' has empty 'role'", model.name);
            }
        }

        // Validate sha256 checksum format
        if let Some(ref hash) = model.sha256 {
            let hash_clean = hash.trim();
            if hash_clean.len() != 64 || !hash_clean.chars().all(|c| c.is_ascii_hexdigit()) {
                bail!(
                    "Model '{}' has invalid sha256 checksum: expected 64 hex characters, got '{}'",
                    model.name,
                    hash
                );
            }
        }

        // Validate URL and check HTTP HEAD connectivity
        if let Some(ref url) = model.url {
            if !url.starts_with("http://") && !url.starts_with("https://") {
                bail!(
                    "Model '{}' download url must start with http:// or https://: '{}'",
                    model.name,
                    url
                );
            }

            if !skip_network {
                check_model_url_head(url).with_context(|| {
                    format!("Network check failed for model '{}' URL", model.name)
                })?;
            }
        }
    }

    // Validate SubActions
    for sub in &manifest.sub_actions {
        if sub.id.trim().is_empty() {
            bail!("Sub-action has empty 'id' in plugin '{}'", manifest.id);
        }
        if sub.name.trim().is_empty() {
            bail!("Sub-action '{}' has empty 'name'", sub.id);
        }
        if let Some(ref icon_file) = sub.icon {
            let sub_icon_path = dir.join(icon_file);
            if sub_icon_path.exists() {
                validate_svg_icon(&sub_icon_path)
                    .with_context(|| format!("Sub-action '{}' icon validation failed", sub.id))?;
            } else {
                bail!(
                    "Sub-action '{}' icon '{}' not found in {}",
                    sub.id,
                    icon_file,
                    dir.display()
                );
            }
        }
    }

    // Validate Mandatory README.md Documentation (including model weights links for NativeAi)
    validate_plugin_readme(dir, Some(&manifest))
        .with_context(|| format!("Documentation validation failed for {}", dir.display()))?;

    Ok(ValidatedPlugin {
        dir_name: dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        dir_path: dir.to_path_buf(),
        manifest,
        icon_path: if icon_path.exists() {
            Some(icon_path)
        } else {
            None
        },
    })
}

pub fn validate_svg_icon(path: &Path) -> Result<()> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read SVG file {}", path.display()))?;

    let doc = roxmltree::Document::parse(&content)
        .with_context(|| format!("Failed to parse SVG XML in {}", path.display()))?;

    // Check root element is <svg>
    let root = doc.root_element();
    if root.tag_name().name() != "svg" {
        bail!("Root element in {} is not <svg>", path.display());
    }

    // Strictly forbid embedded raster images (<image> or xlink:href with data:image)
    for node in doc.descendants() {
        let tag = node.tag_name().name();
        if tag.eq_ignore_ascii_case("image") {
            bail!(
                "SVG icon contains forbidden <image> tag in {}. Only pure vector shapes allowed!",
                path.display()
            );
        }
        for attr in node.attributes() {
            let val = attr.value();
            if val.contains("data:image/") {
                bail!(
                    "SVG icon contains embedded raster base64 data in {}. Only pure vector shapes allowed!",
                    path.display()
                );
            }
        }
    }

    Ok(())
}

pub fn validate_plugin_readme(dir: &Path, manifest: Option<&PluginManifest>) -> Result<()> {
    let readme_path = dir.join("README.md");
    if !readme_path.exists() {
        bail!(
            "Missing mandatory 'README.md' documentation in {}. Every plugin must provide clear documentation with practical use cases.",
            dir.display()
        );
    }

    let content = fs::read_to_string(&readme_path)
        .with_context(|| format!("Failed to read {}", readme_path.display()))?;

    if content.trim().len() < 50 {
        bail!(
            "'README.md' in {} is too short or empty (minimum 50 characters required)",
            dir.display()
        );
    }

    let has_english = content.contains("English")
        || content.contains("Use Cases")
        || content.contains("Use cases");
    let has_russian = content.contains("Русский") || content.contains("Практическое применение");

    if !has_english {
        bail!(
            "'README.md' in {} must contain an English documentation section (e.g., '**English:**' or '### Use Cases:')",
            dir.display()
        );
    }

    if !has_russian {
        bail!(
            "'README.md' in {} must contain a Russian documentation section (e.g., '**Русский:**' or '### Практическое применение:')",
            dir.display()
        );
    }

    // Rule 3.1 Item 4: If plugin is NativeAi and requires neural models, README must provide model download / repo links
    if let Some(man) = manifest {
        if man.plugin_type == core_contracts::PluginType::NativeAi && !man.models.is_empty() {
            let has_weights_en = content.contains("Model Weights")
                || content.contains("Weights:")
                || content.contains("Model weights");
            let has_weights_ru =
                content.contains("веса") || content.contains("Веса") || content.contains("весов");

            if !has_weights_en {
                bail!(
                    "'README.md' in {} (NativeAi plugin) must contain a model weights section in English (e.g. '### Model Weights:')",
                    dir.display()
                );
            }
            if !has_weights_ru {
                bail!(
                    "'README.md' in {} (NativeAi plugin) must contain a model weights section in Russian (e.g. '### Ссылки на веса модели:')",
                    dir.display()
                );
            }

            // Ensure README contains at least one direct link or repository link (http:// or https://)
            if !content.contains("http://") && !content.contains("https://") {
                bail!(
                    "'README.md' in {} (NativeAi plugin) must contain direct links (http:// or https://) to Hugging Face, GitHub Releases or model weights downloads",
                    dir.display()
                );
            }

            // Verify that each declared model in manifest is referenced in README (either by name or URL)
            for model in &man.models {
                let model_name_referenced = content.contains(&model.name);
                let model_url_referenced = model
                    .url
                    .as_ref()
                    .map(|u| content.contains(u))
                    .unwrap_or(false);

                if !model_name_referenced && !model_url_referenced {
                    bail!(
                        "'README.md' in {} (NativeAi plugin) does not mention model '{}'. All required models must be documented with download links.",
                        dir.display(),
                        model.name
                    );
                }
            }
        }
    }

    Ok(())
}

pub fn check_model_url_head(url: &str) -> Result<()> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .build()
        .new_agent();

    // Use HEAD request; some CDNs like HuggingFace resolve with 302/307 redirects to Cloudfront/S3
    let response = agent
        .head(url)
        .call()
        .or_else(|_| agent.get(url).header("Range", "bytes=0-0").call())
        .with_context(|| format!("HTTP request failed for URL: {}", url))?;

    let status = response.status().as_u16();
    if (200..400).contains(&status) {
        Ok(())
    } else {
        bail!("HTTP request to {} returned status code {}", url, status);
    }
}
