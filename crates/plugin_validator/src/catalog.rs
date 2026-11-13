use crate::models::{RegistryEntry, RegistryIndex, RegistryModelEntry, ValidatedPlugin};
use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

pub const DEFAULT_REPO_URL: &str = "https://github.com/kobaltgit/photocraft-companion";
pub const CATALOG_START_TAG: &str = "<!-- PLUGINS_CATALOG_START -->";
pub const CATALOG_END_TAG: &str = "<!-- PLUGINS_CATALOG_END -->";

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
                core_contracts::PluginType::InteractiveUi => "interactive_ui",
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

pub fn chrono_like_now() -> String {
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
            core_contracts::PluginType::InteractiveUi => "`🎨 UI Panel`",
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
