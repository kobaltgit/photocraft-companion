use anyhow::{bail, Context, Result};
use core_contracts::{PluginManifest, PluginType};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryIndex {
    pub generated_at: String,
    pub total_plugins: usize,
    pub plugins: Vec<RegistryEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub plugin_type: String,
    pub download_url: String,
    pub icon_url: Option<String>,
    pub models: Vec<RegistryModelEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegistryModelEntry {
    pub name: String,
    pub required: bool,
    pub size_mb: Option<u64>,
    pub url: Option<String>,
    pub sha256: Option<String>,
}

const DEFAULT_REPO_URL: &str = "https://github.com/kobaltgit/photocraft-companion";
const CATALOG_START_TAG: &str = "<!-- PLUGINS_CATALOG_START -->";
const CATALOG_END_TAG: &str = "<!-- PLUGINS_CATALOG_END -->";

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let mut validate_dir: Option<PathBuf> = None;
    let mut build_registry = false;
    let mut package_plugins = false;
    let mut update_readme_paths: Vec<PathBuf> = Vec::new();
    let mut repo_url = DEFAULT_REPO_URL.to_string();
    let mut skip_network = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--validate" => {
                let target = if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    i += 1;
                    PathBuf::from(&args[i])
                } else {
                    PathBuf::from("plugins")
                };
                validate_dir = Some(target);
            }
            "--build-registry" => {
                build_registry = true;
            }
            "--package-plugins" => {
                package_plugins = true;
            }
            "--update-readme" => {
                if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    i += 1;
                    update_readme_paths.push(PathBuf::from(&args[i]));
                } else {
                    update_readme_paths.push(PathBuf::from("README.md"));
                }
            }
            "--repo-url" => {
                if i + 1 < args.len() {
                    i += 1;
                    repo_url = args[i].clone();
                }
            }
            "--skip-network" => {
                skip_network = true;
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            unknown => {
                eprintln!("Unknown argument: {unknown}");
                print_help();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    if validate_dir.is_none() && !build_registry && update_readme_paths.is_empty() {
        // Default behavior: validate plugins directory
        validate_dir = Some(PathBuf::from("plugins"));
    }

    let plugins_path = validate_dir.unwrap_or_else(|| PathBuf::from("plugins"));

    println!("=== PhotoCraft Plugin Ecosystem Validator & Registry Generator ===");
    println!("Scanning directory: {}", plugins_path.display());

    let manifests = collect_and_validate_plugins(&plugins_path, skip_network)?;

    println!(
        "\n✅ Successfully validated {} plugin(s) with zero errors!\n",
        manifests.len()
    );

    if build_registry {
        println!("Generating registry index...");
        let registry = build_registry_index(&manifests, &repo_url);
        let dist_dir = Path::new("dist");
        if !dist_dir.exists() {
            fs::create_dir_all(dist_dir)
                .with_context(|| format!("Failed to create {}", dist_dir.display()))?;
        }
        let registry_path = dist_dir.join("registry.json");
        let json_str = serde_json::to_string_pretty(&registry)
            .context("Failed to serialize registry index")?;
        fs::write(&registry_path, json_str)
            .with_context(|| format!("Failed to write {}", registry_path.display()))?;
        println!("✅ Registry generated at: {}", registry_path.display());
    }

    if package_plugins {
        println!("Packaging lightweight .pcpl plugins into dist/...");
        let dist_dir = Path::new("dist");
        if !dist_dir.exists() {
            fs::create_dir_all(dist_dir)
                .with_context(|| format!("Failed to create {}", dist_dir.display()))?;
        }
        for plugin in &manifests {
            let output_pcpl = dist_dir.join(format!("{}.pcpl", plugin.dir_name));
            package_lightweight_pcpl(&plugin.dir_path, &output_pcpl)?;
            println!("  📦 Created package: {}", output_pcpl.display());
        }
        println!("✅ All .pcpl packages created successfully in dist/!");
    }

    if !update_readme_paths.is_empty() {
        for readme_path in &update_readme_paths {
            update_readme_catalog(readme_path, &manifests, &repo_url)?;
        }
    }

    Ok(())
}

fn print_help() {
    println!(
        r#"PhotoCraft Plugin Validator & Catalog Builder

USAGE:
    plugin_validator [OPTIONS]

OPTIONS:
    --validate [DIR]           Validate plugins in directory (default: plugins)
    --build-registry           Generate dist/registry.json index
    --package-plugins          Build lightweight .pcpl archives in dist/ (excluding .onnx models)
    --update-readme [FILE]     Update markdown catalog table in specified README file
    --repo-url <URL>           Base repository URL (default: https://github.com/kobaltgit/photocraft-companion)
    --skip-network             Skip HTTP HEAD connectivity check for models
    --help, -h                 Show this help information
"#
    );
}

/// Creates a lightweight `.pcpl` package by excluding large `.onnx` model files
pub fn package_lightweight_pcpl(plugin_dir: &Path, output_path: &Path) -> Result<()> {
    use std::io::{Read, Write};
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    let file = fs::File::create(output_path)
        .with_context(|| format!("Failed to create {}", output_path.display()))?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    fn add_dir(
        zip: &mut ZipWriter<fs::File>,
        dir: &Path,
        prefix: &Path,
        options: SimpleFileOptions,
    ) -> Result<()> {
        let entries = fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory {}", dir.display()))?;

        for entry in entries.flatten() {
            let path = entry.path();
            let relative = path
                .strip_prefix(prefix)
                .with_context(|| format!("Failed to strip prefix from {}", path.display()))?;
            let name_str = relative.to_string_lossy().replace('\\', "/");

            if path.is_dir() {
                // Skip packaging models directory if it only contains .onnx
                zip.add_directory(&name_str, options)?;
                add_dir(zip, &path, prefix, options)?;
            } else {
                // Strictly exclude heavy model weight files (*.onnx, *.bin, *.safetensors)
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let ext_lower = ext.to_ascii_lowercase();
                    if ext_lower == "onnx"
                        || ext_lower == "bin"
                        || ext_lower == "safetensors"
                        || ext_lower == "part"
                    {
                        continue;
                    }
                }
                zip.start_file(&name_str, options)?;
                let mut f = fs::File::open(&path)
                    .with_context(|| format!("Failed to open {}", path.display()))?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)?;
                zip.write_all(&buffer)?;
            }
        }
        Ok(())
    }

    add_dir(&mut zip, plugin_dir, plugin_dir, options)?;
    zip.finish()?;
    Ok(())
}

#[derive(Debug)]
pub struct ValidatedPlugin {
    pub dir_name: String,
    pub dir_path: PathBuf,
    pub manifest: PluginManifest,
    pub icon_path: Option<PathBuf>,
}

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

fn validate_single_plugin(dir: &Path, skip_network: bool) -> Result<ValidatedPlugin> {
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

fn validate_svg_icon(path: &Path) -> Result<()> {
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

fn check_model_url_head(url: &str) -> Result<()> {
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

pub fn build_registry_index(plugins: &[ValidatedPlugin], repo_url: &str) -> RegistryIndex {
    let entries = plugins
        .iter()
        .map(|p| {
            let m = &p.manifest;
            let download_url = format!(
                "{}/releases/download/plugins/{}.pcpl",
                repo_url.trim_end_matches('/'),
                p.dir_name
            );

            let icon_url = if p.icon_path.is_some() {
                Some(format!(
                    "{}/raw/main/plugins/{}/icon.svg",
                    repo_url.trim_end_matches('/'),
                    p.dir_name
                ))
            } else {
                None
            };

            let plugin_type_str = match m.plugin_type {
                core_contracts::PluginType::NoCode => "no_code",
                core_contracts::PluginType::NativeAi => "native_ai",
                core_contracts::PluginType::NativeAlgorithm => "native_algorithm",
                core_contracts::PluginType::Script => "script",
                core_contracts::PluginType::ExternalCli => "external_cli",
            };

            let models = m
                .models
                .iter()
                .map(|mod_req| RegistryModelEntry {
                    name: mod_req.name.clone(),
                    required: mod_req.required,
                    size_mb: mod_req.size_mb,
                    url: mod_req.url.clone(),
                    sha256: mod_req.sha256.clone(),
                })
                .collect();

            RegistryEntry {
                id: m.id.clone(),
                name: m.name.clone(),
                version: m.version.clone(),
                description: m.description.clone(),
                author: m.author.clone(),
                plugin_type: plugin_type_str.to_string(),
                download_url,
                icon_url,
                models,
            }
        })
        .collect::<Vec<_>>();

    RegistryIndex {
        generated_at: chrono_like_now(),
        total_plugins: entries.len(),
        plugins: entries,
    }
}

fn chrono_like_now() -> String {
    // Generate ISO8601-like UTC string without heavy external crate dependencies
    use std::time::SystemTime;
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // Days since epoch
    let days = secs / 86400;
    let rem_secs = secs % 86400;
    let hours = rem_secs / 3600;
    let minutes = (rem_secs % 3600) / 60;
    let seconds = rem_secs % 60;

    // Approximate civil year/month/day
    let mut year = 1970i64;
    let mut d = days as i64;
    loop {
        let leap = if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) {
            366
        } else {
            365
        };
        if d < leap {
            break;
        }
        d -= leap;
        year += 1;
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1;
    for &md in &month_days {
        if d < md {
            break;
        }
        d -= md;
        month += 1;
    }
    let day = d + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

pub fn generate_markdown_catalog_table(
    plugins: &[ValidatedPlugin],
    repo_url: &str,
    is_russian: bool,
) -> String {
    let mut out = String::new();

    if is_russian {
        out.push_str("| Иконка | Название и плагин | Тип | Версия | Автор | Пакет (.pcpl) |\n");
        out.push_str("| :---: | :--- | :---: | :---: | :--- | :---: |\n");
    } else {
        out.push_str(
            "| Icon | Plugin & Description | Type | Version | Author | Package (.pcpl) |\n",
        );
        out.push_str("| :---: | :--- | :---: | :---: | :--- | :---: |\n");
    }

    for p in plugins {
        let m = &p.manifest;
        let pcpl_url = format!(
            "{}/releases/download/plugins/{}.pcpl",
            repo_url.trim_end_matches('/'),
            p.dir_name
        );

        let icon_cell = if p.icon_path.is_some() {
            format!(
                "<img src=\"plugins/{}/icon.svg\" width=\"28\" height=\"28\">",
                p.dir_name
            )
        } else {
            "🧩".to_string()
        };

        let type_badge = match m.plugin_type {
            core_contracts::PluginType::NativeAi => "`🤖 AI (ONNX)`",
            core_contracts::PluginType::NativeAlgorithm => "`📐 Algorithm`",
            core_contracts::PluginType::NoCode => "`⚡ Macro`",
            core_contracts::PluginType::Script => "`📜 Script`",
            core_contracts::PluginType::ExternalCli => "`💻 CLI`",
        };

        let download_badge = if is_russian {
            format!("[📥 Скачать .pcpl]({})", pcpl_url)
        } else {
            format!("[📥 Download .pcpl]({})", pcpl_url)
        };

        out.push_str(&format!(
            "| {} | **{}**<br>_{}_ | {} | `{}` | {} | {} |\n",
            icon_cell, m.name, m.description, type_badge, m.version, m.author, download_badge
        ));
    }

    out
}

pub fn update_readme_catalog(
    readme_path: &Path,
    plugins: &[ValidatedPlugin],
    repo_url: &str,
) -> Result<()> {
    if !readme_path.exists() {
        bail!("README file not found: {}", readme_path.display());
    }

    let content = fs::read_to_string(readme_path)
        .with_context(|| format!("Failed to read {}", readme_path.display()))?;

    let is_russian = readme_path
        .file_name()
        .map(|f| f.to_string_lossy().contains(".ru."))
        .unwrap_or(false);

    let start_idx = content.find(CATALOG_START_TAG).with_context(|| {
        format!(
            "Marker '{}' not found in {}",
            CATALOG_START_TAG,
            readme_path.display()
        )
    })?;

    let end_idx = content.find(CATALOG_END_TAG).with_context(|| {
        format!(
            "Marker '{}' not found in {}",
            CATALOG_END_TAG,
            readme_path.display()
        )
    })?;

    if start_idx >= end_idx {
        bail!(
            "Marker '{}' is after '{}' in {}",
            CATALOG_START_TAG,
            CATALOG_END_TAG,
            readme_path.display()
        );
    }

    let table = generate_markdown_catalog_table(plugins, repo_url, is_russian);

    let mut new_content = String::new();
    new_content.push_str(&content[..start_idx + CATALOG_START_TAG.len()]);
    new_content.push('\n');
    new_content.push('\n');
    new_content.push_str(&table);
    new_content.push('\n');
    new_content.push_str(&content[end_idx..]);

    fs::write(readme_path, new_content)
        .with_context(|| format!("Failed to update {}", readme_path.display()))?;

    println!("✅ Updated catalog table in: {}", readme_path.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
            input: core_contracts::PluginInputSource::ActiveLayer,
            output_mode: core_contracts::PluginOutputMode::InPlace,
            models: vec![],
            action_id: None,
            entrypoint: None,
            ai: None,
            algorithm: None,
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
            entrypoint: None,
            ai: None,
            algorithm: None,
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
            entrypoint: None,
            ai: None,
            algorithm: None,
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
            entrypoint: None,
            ai: None,
            algorithm: None,
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
}
