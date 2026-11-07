use crate::config::{is_dir_writable, user_data_dir};
use crate::{CompanionError, PluginManifest, Result};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use tracing::info;
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};

/// Recursively clears the readonly attribute on all files and folders within a path.
pub fn clear_readonly_recursive(path: &Path) {
    if let Ok(metadata) = fs::metadata(path) {
        let mut permissions = metadata.permissions();
        if permissions.readonly() {
            #[allow(clippy::permissions_set_readonly_false)]
            permissions.set_readonly(false);
            let _ = fs::set_permissions(path, permissions);
        }
    }
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            clear_readonly_recursive(&entry.path());
        }
    }
}

/// Recursively removes a directory with readonly attribute clearing and retry logic for Windows NTFS locks.
pub fn remove_dir_all_safe(path: &Path) -> io::Result<()> {
    if !path.exists() {
        return Ok(());
    }
    clear_readonly_recursive(path);
    if fs::remove_dir_all(path).is_ok() {
        return Ok(());
    }
    for i in 0..5 {
        std::thread::sleep(std::time::Duration::from_millis(30 * (i + 1)));
        clear_readonly_recursive(path);
        if fs::remove_dir_all(path).is_ok() {
            return Ok(());
        }
    }
    fs::remove_dir_all(path)
}

/// Helper function to rename paths with retry logic for Windows locks (e.g. Defender/Indexer).
pub fn rename_with_retry(from: &Path, to: &Path) -> io::Result<()> {
    if fs::rename(from, to).is_ok() {
        return Ok(());
    }
    for i in 0..5 {
        std::thread::sleep(std::time::Duration::from_millis(30 * (i + 1)));
        if fs::rename(from, to).is_ok() {
            return Ok(());
        }
    }
    fs::rename(from, to)
}

/// Installs a `.pcpl` package (ZIP-based PhotoCraft Plugin Package) into the plugins directory.
pub fn install_pcpl(
    archive_path: impl AsRef<Path>,
    plugins_dir: impl AsRef<Path>,
) -> Result<PluginManifest> {
    let archive_path = archive_path.as_ref();
    let plugins_dir = plugins_dir.as_ref();

    let file = File::open(archive_path).map_err(CompanionError::Io)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|e| CompanionError::Plugin(format!("Failed to open .pcpl archive: {e}")))?;

    // 1. Locate and parse `plugin.json` to get the plugin ID and metadata
    let mut manifest_content = String::new();
    let mut manifest_found = false;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| CompanionError::Plugin(format!("Failed to read archive entry: {e}")))?;
        let name = entry.name().to_string();
        if name == "plugin.json" || name.ends_with("/plugin.json") {
            entry
                .read_to_string(&mut manifest_content)
                .map_err(CompanionError::Io)?;
            manifest_found = true;
            break;
        }
    }

    if !manifest_found {
        return Err(CompanionError::Plugin(
            "Invalid .pcpl package: 'plugin.json' manifest not found".into(),
        ));
    }

    let mut manifest: PluginManifest = serde_json::from_str(&manifest_content)
        .map_err(|e| CompanionError::Plugin(format!("Failed to parse plugin.json: {e}")))?;

    // 2. Check writable target directory with fallback to user_data_dir()
    let effective_plugins_dir = if is_dir_writable(plugins_dir) {
        plugins_dir.to_path_buf()
    } else {
        let fallback = user_data_dir().join("plugins");
        let _ = fs::create_dir_all(&fallback);
        fallback
    };
    let _ = fs::create_dir_all(&effective_plugins_dir);

    // 3. Target folder: check if an existing folder holds this plugin ID to preserve folder name
    let sanitized_id = manifest.id.replace(['/', '\\'], "_");
    let existing_target = fs::read_dir(&effective_plugins_dir)
        .ok()
        .and_then(|entries| {
            entries.flatten().find_map(|entry| {
                let path = entry.path();
                if path.is_dir() {
                    let manifest_path = path.join("plugin.json");
                    if let Ok(content) = fs::read_to_string(&manifest_path) {
                        if let Ok(existing_manifest) =
                            serde_json::from_str::<PluginManifest>(&content)
                        {
                            if existing_manifest.id == manifest.id {
                                return Some(path);
                            }
                        }
                    }
                    let folder_name = entry.file_name().to_string_lossy().to_string();
                    let short_id = manifest.id.rsplit('.').next().unwrap_or(&manifest.id);
                    if folder_name == sanitized_id || folder_name == short_id {
                        return Some(path);
                    }
                }
                None
            })
        });
    let target_dir = existing_target.unwrap_or_else(|| effective_plugins_dir.join(&sanitized_id));

    const MAX_TOTAL_UNCOMPRESSED: u64 = 512 * 1024 * 1024; // 512 MB Zip-Bomb Guard
    let mut total_uncompressed: u64 = 0;

    // Temporary directory for atomic extraction
    let temp_target =
        effective_plugins_dir.join(format!("{}.tmp.{}", sanitized_id, std::process::id()));
    if temp_target.exists() {
        let _ = remove_dir_all_safe(&temp_target);
    }
    fs::create_dir_all(&temp_target).map_err(CompanionError::Io)?;

    let install_result = (|| -> Result<()> {
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| {
                CompanionError::Plugin(format!("Failed to read archive entry: {e}"))
            })?;

            total_uncompressed = total_uncompressed.saturating_add(entry.size());
            if total_uncompressed > MAX_TOTAL_UNCOMPRESSED {
                return Err(CompanionError::Plugin(
                    "Security violation: archive uncompressed size exceeds limit (512 MB)".into(),
                ));
            }

            let enclosed = match entry.enclosed_name() {
                Some(p) => p.to_path_buf(),
                None => {
                    return Err(CompanionError::Plugin(format!(
                        "Security violation: path traversal detected in archive entry '{}'",
                        entry.name()
                    )));
                }
            };

            // 1. Correct handling of archive structure:
            // If archive was zipped with root folder (<sanitized_id>/...), strip it
            let relative_path = if let Ok(stripped) = enclosed.strip_prefix(&sanitized_id) {
                stripped.to_path_buf()
            } else {
                enclosed
            };

            if relative_path.as_os_str().is_empty() {
                continue;
            }

            // 2. Strict verification of relative path components
            for comp in relative_path.components() {
                match comp {
                    std::path::Component::Normal(_) => {}
                    _ => {
                        return Err(CompanionError::Plugin(format!(
                            "Security violation: non-normal path component in archive entry '{}'",
                            entry.name()
                        )));
                    }
                }
            }

            // 3. Prohibition of symlinks in plugin archives on Unix
            #[cfg(unix)]
            {
                if entry
                    .unix_mode()
                    .map(|m| (m & 0o170000) == 0o120000)
                    .unwrap_or(false)
                {
                    return Err(CompanionError::Plugin(format!(
                        "Security violation: symlink entries forbidden in .pcpl archives: '{}'",
                        entry.name()
                    )));
                }
            }

            let out_path = temp_target.join(&relative_path);
            if entry.is_dir() {
                fs::create_dir_all(&out_path).map_err(CompanionError::Io)?;
            } else {
                if let Some(parent) = out_path.parent() {
                    fs::create_dir_all(parent).map_err(CompanionError::Io)?;
                }
                let mut out_file = File::create(&out_path).map_err(CompanionError::Io)?;
                io::copy(&mut entry, &mut out_file).map_err(CompanionError::Io)?;
            }
        }
        Ok(())
    })();

    if let Err(e) = install_result {
        let _ = remove_dir_all_safe(&temp_target);
        return Err(e);
    }

    // 4. Data protection: preserve existing models/ folder with heavy neural weights (.onnx)
    let old_models = target_dir.join("models");
    let new_models = temp_target.join("models");
    if old_models.is_dir() && !new_models.exists() {
        let _ = rename_with_retry(&old_models, &new_models);
    }

    // 5. Atomic replacement of target directory
    if target_dir.exists() {
        let _ = remove_dir_all_safe(&target_dir);
    }
    rename_with_retry(&temp_target, &target_dir).map_err(CompanionError::Io)?;

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

    let file = File::create(output_path).map_err(CompanionError::Io)?;
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
        let entries = fs::read_dir(dir).map_err(CompanionError::Io)?;
        for entry in entries.flatten() {
            let path = entry.path();
            let relative = path
                .strip_prefix(prefix)
                .map_err(|e| CompanionError::Plugin(format!("Path strip prefix error: {e}")))?;
            let name_str = relative.to_string_lossy().replace('\\', "/");

            if path.is_dir() {
                zip.add_directory(&name_str, options)
                    .map_err(|e| CompanionError::Plugin(format!("Zip add_dir error: {e}")))?;
                add_dir(zip, &path, prefix, options)?;
            } else {
                zip.start_file(&name_str, options)
                    .map_err(|e| CompanionError::Plugin(format!("Zip start_file error: {e}")))?;
                let mut f = File::open(&path).map_err(CompanionError::Io)?;
                io::copy(&mut f, zip).map_err(CompanionError::Io)?;
            }
        }
        Ok(())
    }

    add_dir(&mut zip, plugin_dir, &prefix, options)?;
    zip.finish()
        .map_err(|e| CompanionError::Plugin(format!("Zip finish error: {e}")))?;

    info!("Created .pcpl package: {}", output_path.display());
    Ok(())
}

/// Decodes a base64 string (supporting data URL prefix like `data:image/webp;base64,`, whitespace, and padding).
pub fn decode_base64(input: &str) -> Result<Vec<u8>> {
    let clean = if let Some(idx) = input.find(',') {
        if input.starts_with("data:") {
            &input[idx + 1..]
        } else {
            input
        }
    } else {
        input
    };

    let mut out = Vec::with_capacity((clean.len() * 3) / 4);
    let mut chunk = [0u8; 4];
    let mut chunk_len = 0;

    for b in clean.bytes() {
        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' | b'\r' | b'\n' | b' ' | b'\t' => continue,
            _ => {
                return Err(CompanionError::Bridge(format!(
                    "Invalid base64 character: '{}'",
                    b as char
                )));
            }
        };
        chunk[chunk_len] = val;
        chunk_len += 1;
        if chunk_len == 4 {
            out.push((chunk[0] << 2) | (chunk[1] >> 4));
            out.push(((chunk[1] & 0x0f) << 4) | (chunk[2] >> 2));
            out.push(((chunk[2] & 0x03) << 6) | (chunk[3] & 0x3f));
            chunk_len = 0;
        }
    }

    match chunk_len {
        0 => {}
        2 => {
            out.push((chunk[0] << 2) | (chunk[1] >> 4));
        }
        3 => {
            out.push((chunk[0] << 2) | (chunk[1] >> 4));
            out.push(((chunk[1] & 0x0f) << 4) | (chunk[2] >> 2));
        }
        1 => {
            return Err(CompanionError::Bridge(
                "Invalid base64 length: trailing single character".into(),
            ));
        }
        _ => unreachable!(),
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_install_pcpl() {
        let temp_dir = std::env::temp_dir().join(format!(
            "pcpl_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
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
        let temp_dir = std::env::temp_dir().join(format!(
            "pcpl_err_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
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
        let temp_dir = std::env::temp_dir().join(format!(
            "pcpl_bad_json_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
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
        let temp_dir = std::env::temp_dir().join(format!(
            "pcpl_corrupt_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();
        let corrupt_path = temp_dir.join("corrupt.pcpl");
        fs::write(&corrupt_path, b"not a zip file").unwrap();

        let target_dir = temp_dir.join("target");
        let result = install_pcpl(&corrupt_path, &target_dir);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_install_pcpl_with_root_folder_prefix() {
        let temp_dir = std::env::temp_dir().join(format!(
            "pcpl_prefix_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let src_dir = temp_dir.join("packaged_dir").join("test.plugin.prefixed");
        fs::create_dir_all(&src_dir).unwrap();

        let manifest = r#"{
            "id": "test.plugin.prefixed",
            "name": "Prefixed Plugin",
            "version": "1.0.0",
            "description": "Prefixed archive test",
            "author": "Unit Tester",
            "plugin_type": "no_code"
        }"#;
        fs::write(src_dir.join("plugin.json"), manifest).unwrap();
        fs::write(src_dir.join("script.py"), "print('hello')").unwrap();

        let pcpl_path = temp_dir.join("prefixed.pcpl");
        create_pcpl(temp_dir.join("packaged_dir"), &pcpl_path).unwrap();

        let target_plugins = temp_dir.join("installed");
        let installed = install_pcpl(&pcpl_path, &target_plugins).unwrap();
        assert_eq!(installed.id, "test.plugin.prefixed");

        let dest = target_plugins.join("test.plugin.prefixed");
        assert!(dest.join("plugin.json").exists());
        assert!(dest.join("script.py").exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_install_pcpl_preserves_existing_models() {
        let temp_dir = std::env::temp_dir().join(format!(
            "pcpl_models_preserve_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let src_dir = temp_dir.join("update_src");
        fs::create_dir_all(&src_dir).unwrap();

        let manifest = r#"{
            "id": "test.plugin.ai_model",
            "name": "AI Model Plugin",
            "version": "1.1.0",
            "description": "Update without models in archive",
            "author": "Unit Tester",
            "plugin_type": "native_ai"
        }"#;
        fs::write(src_dir.join("plugin.json"), manifest).unwrap();

        let pcpl_path = temp_dir.join("update.pcpl");
        create_pcpl(&src_dir, &pcpl_path).unwrap();

        let target_plugins = temp_dir.join("installed");
        let existing_plugin_dir = target_plugins.join("test.plugin.ai_model");
        let existing_models_dir = existing_plugin_dir.join("models");
        fs::create_dir_all(&existing_models_dir).unwrap();
        fs::write(
            existing_models_dir.join("heavy_weight.onnx"),
            b"heavy weights",
        )
        .unwrap();

        let installed = install_pcpl(&pcpl_path, &target_plugins).unwrap();
        assert_eq!(installed.id, "test.plugin.ai_model");

        let weight_path = existing_plugin_dir.join("models").join("heavy_weight.onnx");
        assert!(
            weight_path.exists(),
            "Existing heavy models must be preserved during .pcpl update!"
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_install_pcpl_rejects_zip_slip() {
        let temp_dir = std::env::temp_dir().join(format!(
            "pcpl_slip_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();
        let pcpl_path = temp_dir.join("slip.pcpl");

        let file = File::create(&pcpl_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        let manifest = r#"{
            "id": "test.plugin.malicious",
            "name": "Malicious",
            "version": "1.0.0",
            "description": "Exploit",
            "author": "Attacker",
            "plugin_type": "no_code"
        }"#;
        zip.start_file("plugin.json", options).unwrap();
        io::copy(&mut manifest.as_bytes(), &mut zip).unwrap();

        zip.start_file("../../escaped_file.txt", options).unwrap();
        io::copy(&mut "malicious content".as_bytes(), &mut zip).unwrap();
        zip.finish().unwrap();

        let target_plugins = temp_dir.join("installed");
        let result = install_pcpl(&pcpl_path, &target_plugins);
        assert!(result.is_err(), "Zip Slip must be rejected!");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Security violation") || err_msg.contains("path traversal"),
            "Error should mention security violation: {err_msg}"
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_install_pcpl_preserves_existing_custom_folder_name() {
        let temp_dir = std::env::temp_dir().join(format!(
            "pcpl_custom_folder_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        // 1. Create existing plugin folder under short name "harmonic_studio"
        let plugins_dir = temp_dir.join("plugins");
        let existing_plugin_dir = plugins_dir.join("harmonic_studio");
        fs::create_dir_all(&existing_plugin_dir).unwrap();
        let existing_manifest = r#"{
            "id": "ui.photocraft.plugins.harmonic_studio",
            "name": "Harmonic Studio",
            "version": "1.0.0",
            "description": "Old version",
            "author": "kobaltGIT",
            "plugin_type": "interactive_ui"
        }"#;
        fs::write(existing_plugin_dir.join("plugin.json"), existing_manifest).unwrap();

        // 2. Build a new .pcpl package with updated version 1.1.0
        let pcpl_path = temp_dir.join("update.pcpl");
        let file = File::create(&pcpl_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        let new_manifest = r#"{
            "id": "ui.photocraft.plugins.harmonic_studio",
            "name": "Harmonic Studio",
            "version": "1.1.0",
            "description": "Updated version",
            "author": "kobaltGIT",
            "plugin_type": "interactive_ui"
        }"#;
        zip.start_file("plugin.json", options).unwrap();
        io::copy(&mut new_manifest.as_bytes(), &mut zip).unwrap();
        zip.start_file("logic.rhai", options).unwrap();
        io::copy(&mut "// updated logic".as_bytes(), &mut zip).unwrap();
        zip.finish().unwrap();

        // 3. Install .pcpl
        let manifest = install_pcpl(&pcpl_path, &plugins_dir).expect("Installation must succeed");
        assert_eq!(manifest.version, "1.1.0");

        // 4. Verify that existing folder "harmonic_studio" was updated, and no duplicate folder was created
        assert_eq!(manifest.plugin_dir, Some(existing_plugin_dir.clone()));
        assert!(existing_plugin_dir.join("logic.rhai").exists());
        assert!(!plugins_dir
            .join("ui.photocraft.plugins.harmonic_studio")
            .exists());

        let _ = remove_dir_all_safe(&temp_dir);
    }

    #[test]
    fn test_install_pcpl_handles_readonly_files_on_replace() {
        let temp_dir = std::env::temp_dir().join(format!(
            "pcpl_ro_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        let plugins_dir = temp_dir.join("plugins");
        let plugin_dir = plugins_dir.join("test_plugin");
        fs::create_dir_all(&plugin_dir).unwrap();

        let old_manifest = r#"{
            "id": "test_plugin",
            "name": "Test",
            "version": "1.0.0",
            "description": "Old",
            "author": "kobaltGIT",
            "plugin_type": "no_code"
        }"#;
        let manifest_file = plugin_dir.join("plugin.json");
        fs::write(&manifest_file, old_manifest).unwrap();

        // Mark existing file readonly
        let mut perms = fs::metadata(&manifest_file).unwrap().permissions();
        perms.set_readonly(true);
        fs::set_permissions(&manifest_file, perms).unwrap();

        // Build .pcpl package
        let pcpl_path = temp_dir.join("pkg.pcpl");
        let file = File::create(&pcpl_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        let new_manifest = r#"{
            "id": "test_plugin",
            "name": "Test",
            "version": "2.0.0",
            "description": "New",
            "author": "kobaltGIT",
            "plugin_type": "no_code"
        }"#;
        zip.start_file("plugin.json", options).unwrap();
        io::copy(&mut new_manifest.as_bytes(), &mut zip).unwrap();
        zip.finish().unwrap();

        // Install replacing readonly file
        let installed = install_pcpl(&pcpl_path, &plugins_dir)
            .expect("Must replace readonly files without Access Denied");
        assert_eq!(installed.version, "2.0.0");

        let _ = remove_dir_all_safe(&temp_dir);
    }

    #[test]
    fn test_install_pcpl_fallback_when_target_readonly() {
        use std::path::PathBuf;

        let temp_dir = std::env::temp_dir().join(format!(
            "pcpl_fallback_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&temp_dir).unwrap();

        let pcpl_path = temp_dir.join("dummy.pcpl");
        let file = File::create(&pcpl_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        let manifest = r#"{
            "id": "test.fallback.plugin",
            "name": "Fallback",
            "version": "1.0.0",
            "description": "Fallback",
            "author": "kobaltGIT",
            "plugin_type": "no_code"
        }"#;
        zip.start_file("plugin.json", options).unwrap();
        io::copy(&mut manifest.as_bytes(), &mut zip).unwrap();
        zip.finish().unwrap();

        #[cfg(windows)]
        let readonly_plugins =
            PathBuf::from("C:\\Program Files\\PhotoCraft Companion\\fake_plugins");
        #[cfg(not(windows))]
        let readonly_plugins = PathBuf::from("/etc/photocraft/fake_plugins");

        let installed =
            install_pcpl(&pcpl_path, &readonly_plugins).expect("Must fallback to user_data_dir");
        assert_eq!(installed.name, "Fallback");
        assert!(installed.plugin_dir.is_some());
        let target = installed.plugin_dir.unwrap();
        assert_ne!(target, readonly_plugins);
        assert!(
            target.display().to_string().contains("PhotoCraft")
                || target.display().to_string().contains("plugins")
        );

        let _ = remove_dir_all_safe(&target);
        let _ = remove_dir_all_safe(&temp_dir);
    }
}
