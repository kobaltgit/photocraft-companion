use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

mod catalog;
mod models;
mod packager;
mod validator;

#[cfg(test)]
mod tests;

pub use catalog::*;
pub use models::*;
pub use packager::*;
pub use validator::*;

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
