use photocraft_bridge::PhotoCraftClient;
use std::path::{Path, PathBuf};

pub fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

pub fn find_plugins_dir() -> PathBuf {
    let user_plugins = core_contracts::user_data_dir().join("plugins");

    // 1. Discover bundled/adjacent plugins directory
    let mut bundled_plugins = None;
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let adjacent = exe_dir.join("plugins");
            if adjacent.is_dir() {
                bundled_plugins = Some(adjacent);
            } else if let Some(target_dir) = exe_dir.parent() {
                if let Some(root) = target_dir.parent() {
                    let candidate = root.join("plugins");
                    if candidate.is_dir() {
                        bundled_plugins = Some(candidate);
                    }
                }
            }
        }
    }
    if bundled_plugins.is_none() {
        if let Ok(cwd) = std::env::current_dir() {
            let mut cur = cwd.as_path();
            for _ in 0..5 {
                let candidate = cur.join("plugins");
                if candidate.is_dir() {
                    bundled_plugins = Some(candidate);
                    break;
                }
                if let Some(parent) = cur.parent() {
                    cur = parent;
                } else {
                    break;
                }
            }
        }
    }

    if let Some(bundled) = bundled_plugins {
        // If bundled directory is writable, use it directly (portable mode)
        if core_contracts::is_dir_writable(&bundled) {
            return bundled;
        }

        // If bundled directory is read-only (e.g. macOS DMG or Windows Program Files),
        // ensure writable user_plugins directory and copy bundled plugins if empty
        let _ = std::fs::create_dir_all(&user_plugins);
        sync_missing_bundled_plugins(&bundled, &user_plugins);
        return user_plugins;
    }

    let _ = std::fs::create_dir_all(&user_plugins);
    user_plugins
}

pub fn sync_missing_bundled_plugins(bundled_dir: &Path, user_dir: &Path) {
    let bundled_entries = match std::fs::read_dir(bundled_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut existing_ids = std::collections::HashSet::new();
    let mut existing_folders = std::collections::HashSet::new();
    if let Ok(user_entries) = std::fs::read_dir(user_dir) {
        for entry in user_entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                existing_folders.insert(entry.file_name().to_string_lossy().to_string());
                let manifest_path = path.join("plugin.json");
                if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                    if let Ok(manifest) =
                        serde_json::from_str::<core_contracts::PluginManifest>(&content)
                    {
                        existing_ids.insert(manifest.id);
                    }
                }
            }
        }
    }

    for entry in bundled_entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let folder_name = entry.file_name().to_string_lossy().to_string();
        if existing_folders.contains(&folder_name) {
            continue;
        }

        let manifest_path = path.join("plugin.json");
        if let Ok(content) = std::fs::read_to_string(&manifest_path) {
            if let Ok(manifest) = serde_json::from_str::<core_contracts::PluginManifest>(&content) {
                if existing_ids.contains(&manifest.id) {
                    continue;
                }
            }
        }

        let target = user_dir.join(&folder_name);
        tracing::info!(
            "Seeding missing bundled plugin '{}' to user directory: {}",
            folder_name,
            target.display()
        );
        let _ = copy_dir_all(&path, &target);
    }
}

pub fn find_photocraft_data_dir() -> PathBuf {
    // 1. Check directory of currently running PhotoCraft process
    if let Some(proc_dir) = PhotoCraftClient::find_running_photocraft_dir() {
        let candidate = proc_dir.join("PhotoCraftData");
        if candidate.is_dir() && core_contracts::is_dir_writable(&candidate) {
            return candidate;
        }
    }

    // 2. Check local relative directory or parents (portable mode)
    let mut candidates = Vec::new();
    candidates.push(PathBuf::from("PhotoCraftData"));
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            candidates.push(exe_dir.join("PhotoCraftData"));
            if let Some(parent) = exe_dir.parent() {
                candidates.push(parent.join("PhotoCraftData"));
                if let Some(grandparent) = parent.parent() {
                    candidates.push(grandparent.join("PhotoCraftData"));
                }
            }
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let mut cur = cwd.as_path();
        for _ in 0..5 {
            candidates.push(cur.join("PhotoCraftData"));
            if let Some(parent) = cur.parent() {
                cur = parent;
            } else {
                break;
            }
        }
    }

    for candidate in candidates {
        if candidate.is_dir() && core_contracts::is_dir_writable(&candidate) {
            return candidate;
        }
    }

    // 3. Check AppData or user home standard directory
    if let Ok(appdata) = std::env::var("APPDATA") {
        let candidate = PathBuf::from(appdata).join("PhotoCraft");
        if candidate.is_dir() && core_contracts::is_dir_writable(&candidate) {
            return candidate;
        }
    }

    // 4. Guaranteed writable user data directory (macOS ~/Library/Application Support, Linux ~/.config, Windows %LOCALAPPDATA%)
    let user_data = core_contracts::user_data_dir().join("PhotoCraftData");
    let _ = std::fs::create_dir_all(&user_data);
    user_data
}
