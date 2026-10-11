use crate::{CompanionError, PluginManifest, Result};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use tracing::info;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

/// Installs a `.pcpl` package (ZIP-based PhotoCraft Plugin Package) into the plugins directory.
pub fn install_pcpl(archive_path: impl AsRef<Path>, plugins_dir: impl AsRef<Path>) -> Result<PluginManifest> {
    let archive_path = archive_path.as_ref();
    let plugins_dir = plugins_dir.as_ref();

    let file = File::open(archive_path).map_err(|e| CompanionError::Io(e))?;
    let mut archive = ZipArchive::new(file).map_err(|e| {
        CompanionError::Plugin(format!("Failed to open .pcpl archive: {e}"))
    })?;

    // 1. Locate and parse `plugin.json` to get the plugin ID and metadata
    let mut manifest_content = String::new();
    let mut manifest_found = false;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| {
            CompanionError::Plugin(format!("Failed to read archive entry: {e}"))
        })?;
        let name = entry.name().to_string();
        if name == "plugin.json" || name.ends_with("/plugin.json") {
            entry.read_to_string(&mut manifest_content).map_err(|e| CompanionError::Io(e))?;
            manifest_found = true;
            break;
        }
    }

    if !manifest_found {
        return Err(CompanionError::Plugin(
            "Invalid .pcpl package: 'plugin.json' manifest not found".into(),
        ));
    }

    let mut manifest: PluginManifest = serde_json::from_str(&manifest_content).map_err(|e| {
        CompanionError::Plugin(format!("Failed to parse plugin.json: {e}"))
    })?;

    // 2. Target folder: plugins/<manifest.id>
    let sanitized_id = manifest.id.replace('/', "_").replace('\\', "_");
    let target_dir = plugins_dir.join(&sanitized_id);
    fs::create_dir_all(&target_dir).map_err(|e| CompanionError::Io(e))?;

    // 3. Extract all files
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| {
            CompanionError::Plugin(format!("Failed to read archive entry: {e}"))
        })?;

        let entry_name = entry.name().to_string();
        // Skip leading root folder if archive was zipped with root folder
        let relative_path = if let Some(stripped) = entry_name.strip_prefix(&format!("{}/", sanitized_id)) {
            PathBuf::from(stripped)
        } else {
            PathBuf::from(&entry_name)
        };

        if relative_path.as_os_str().is_empty() {
            continue;
        }

        let out_path = target_dir.join(&relative_path);

        if entry.is_dir() {
            fs::create_dir_all(&out_path).map_err(|e| CompanionError::Io(e))?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).map_err(|e| CompanionError::Io(e))?;
            }
            let mut out_file = File::create(&out_path).map_err(|e| CompanionError::Io(e))?;
            io::copy(&mut entry, &mut out_file).map_err(|e| CompanionError::Io(e))?;
        }
    }

    manifest.plugin_dir = Some(target_dir.clone());
    info!(
        "Successfully installed .pcpl plugin '{}' (v{}) into {}",
        manifest.name,
        manifest.version,
        target_dir.display()
    );

    Ok(manifest)
}

/// Helper function to create a `.pcpl` package from an existing plugin directory
pub fn create_pcpl(plugin_dir: impl AsRef<Path>, output_path: impl AsRef<Path>) -> Result<()> {
    let plugin_dir = plugin_dir.as_ref();
    let output_path = output_path.as_ref();

    let file = File::create(output_path).map_err(|e| CompanionError::Io(e))?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let prefix = plugin_dir.to_path_buf();
    fn add_dir(
        zip: &mut ZipWriter<File>,
        dir: &Path,
        prefix: &Path,
        options: SimpleFileOptions,
    ) -> Result<()> {
        let entries = fs::read_dir(dir).map_err(|e| CompanionError::Io(e))?;
        for entry in entries.flatten() {
            let path = entry.path();
            let relative = path.strip_prefix(prefix).map_err(|e| {
                CompanionError::Plugin(format!("Path strip prefix error: {e}"))
            })?;
            let name_str = relative.to_string_lossy().replace('\\', "/");

            if path.is_dir() {
                zip.add_directory(&name_str, options).map_err(|e| {
                    CompanionError::Plugin(format!("Zip add_dir error: {e}"))
                })?;
                add_dir(zip, &path, prefix, options)?;
            } else {
                zip.start_file(&name_str, options).map_err(|e| {
                    CompanionError::Plugin(format!("Zip start_file error: {e}"))
                })?;
                let mut f = File::open(&path).map_err(|e| CompanionError::Io(e))?;
                io::copy(&mut f, zip).map_err(|e| CompanionError::Io(e))?;
            }
        }
        Ok(())
    }

    add_dir(&mut zip, plugin_dir, &prefix, options)?;
    zip.finish().map_err(|e| {
        CompanionError::Plugin(format!("Zip finish error: {e}"))
    })?;

    info!("Created .pcpl package: {}", output_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_install_pcpl() {
        let temp_dir = std::env::temp_dir().join(format!("pcpl_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let source_plugin = temp_dir.join("source_plugin");
        let models_dir = source_plugin.join("models");
        fs::create_dir_all(&models_dir).unwrap();

        let manifest_content = r#"{
            "id": "test.plugin.watermark",
            "name": "Watermark Plugin",
            "version": "1.0.0",
            "description": "Test plugin packaging",
            "author": "Unit Tester",
            "plugin_type": "no_code",
            "models": [
                { "name": "dummy.onnx", "required": false }
            ]
        }"#;

        fs::write(source_plugin.join("plugin.json"), manifest_content).unwrap();
        fs::write(source_plugin.join("icon.svg"), "<svg></svg>").unwrap();
        fs::write(models_dir.join("dummy.onnx"), "dummy model content").unwrap();

        let pcpl_path = temp_dir.join("watermark.pcpl");
        create_pcpl(&source_plugin, &pcpl_path).unwrap();
        assert!(pcpl_path.exists());

        let target_plugins_dir = temp_dir.join("installed_plugins");
        fs::create_dir_all(&target_plugins_dir).unwrap();

        let installed_manifest = install_pcpl(&pcpl_path, &target_plugins_dir).unwrap();
        assert_eq!(installed_manifest.id, "test.plugin.watermark");
        assert_eq!(installed_manifest.name, "Watermark Plugin");

        let installed_dir = target_plugins_dir.join("test.plugin.watermark");
        assert!(installed_dir.join("plugin.json").exists());
        assert!(installed_dir.join("icon.svg").exists());
        assert!(installed_dir.join("models").join("dummy.onnx").exists());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_install_pcpl_missing_manifest() {
        let temp_dir = std::env::temp_dir().join(format!("pcpl_err_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let dummy_src = temp_dir.join("src");
        fs::create_dir_all(&dummy_src).unwrap();
        fs::write(dummy_src.join("readme.txt"), "no manifest here").unwrap();

        let pcpl_path = temp_dir.join("invalid.pcpl");
        create_pcpl(&dummy_src, &pcpl_path).unwrap();

        let target_dir = temp_dir.join("target");
        let result = install_pcpl(&pcpl_path, &target_dir);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("plugin.json"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_install_pcpl_malformed_manifest() {
        let temp_dir = std::env::temp_dir().join(format!("pcpl_bad_json_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let dummy_src = temp_dir.join("src");
        fs::create_dir_all(&dummy_src).unwrap();
        fs::write(dummy_src.join("plugin.json"), "NOT VALID JSON").unwrap();

        let pcpl_path = temp_dir.join("bad.pcpl");
        create_pcpl(&dummy_src, &pcpl_path).unwrap();

        let target_dir = temp_dir.join("target");
        let result = install_pcpl(&pcpl_path, &target_dir);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_install_pcpl_invalid_archive() {
        let temp_dir = std::env::temp_dir().join(format!("pcpl_corrupt_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(&temp_dir).unwrap();
        let corrupt_path = temp_dir.join("corrupt.pcpl");
        fs::write(&corrupt_path, b"not a zip file").unwrap();

        let target_dir = temp_dir.join("target");
        let result = install_pcpl(&corrupt_path, &target_dir);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}

