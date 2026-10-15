use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub is_vertical: bool,
    #[serde(default)]
    pub window_position: Option<[f32; 2]>,
    #[serde(default)]
    pub enabled_plugins: Vec<String>,
    #[serde(default)]
    pub photocraft_path: Option<PathBuf>,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    18492
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_language() -> String {
    "auto".to_string()
}

impl Default for CompanionConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            token: None,
            theme: default_theme(),
            language: default_language(),
            is_vertical: false, // Per spec: default is horizontal
            window_position: None,
            enabled_plugins: Vec::new(),
            photocraft_path: None,
        }
    }
}

impl CompanionConfig {
    pub fn load_or_default(path: impl AsRef<Path>) -> Self {
        let p = path.as_ref();
        let mut cfg = if p.exists() {
            if let Ok(content) = fs::read_to_string(p) {
                serde_json::from_str::<Self>(&content).unwrap_or_default()
            } else {
                Self::default()
            }
        } else {
            Self::default()
        };

        // If photocraft_path is missing in this config, restore from local or exe-adjacent config mirror
        if cfg.photocraft_path.is_none() {
            let mut mirrors = Vec::new();
            mirrors.push(PathBuf::from("companion_config.json"));
            if let Ok(exe) = std::env::current_exe() {
                if let Some(exe_dir) = exe.parent() {
                    mirrors.push(exe_dir.join("companion_config.json"));
                    mirrors.push(exe_dir.join("PhotoCraftData").join("companion_config.json"));
                }
            }
            for m in mirrors {
                if m != p && m.exists() {
                    if let Ok(content) = fs::read_to_string(&m) {
                        if let Ok(other) = serde_json::from_str::<Self>(&content) {
                            if other.photocraft_path.is_some() {
                                cfg.photocraft_path = other.photocraft_path;
                                break;
                            }
                        }
                    }
                }
            }
        }

        info!("Loaded companion configuration from {}", p.display());
        cfg
    }

    pub fn save(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let p = path.as_ref();
        let target_path = if let Some(parent) = p.parent() {
            if !is_dir_writable(parent) {
                let user_dir = user_data_dir();
                let _ = fs::create_dir_all(&user_dir);
                user_dir.join("companion_config.json")
            } else {
                let _ = fs::create_dir_all(parent);
                p.to_path_buf()
            }
        } else {
            p.to_path_buf()
        };

        let json = serde_json::to_string_pretty(self)?;
        match fs::write(&target_path, &json) {
            Ok(_) => {
                info!("Saved companion configuration to {}", target_path.display());
            }
            Err(e) => {
                // If writing to original path failed with permission error or read-only error, fallback to user_data_dir!
                let fallback = user_data_dir().join("companion_config.json");
                if fallback != target_path {
                    let _ = fs::create_dir_all(user_data_dir());
                    fs::write(&fallback, &json)?;
                    info!(
                        "Fallback: saved companion configuration to user data: {}",
                        fallback.display()
                    );
                } else {
                    return Err(e.into());
                }
            }
        }

        // Also mirror save to companion executable directory IF it is writable (never fail if read-only)
        if let Ok(exe) = std::env::current_exe() {
            if let Some(exe_dir) = exe.parent() {
                if is_dir_writable(exe_dir) {
                    let exe_cfg = exe_dir.join("companion_config.json");
                    if exe_cfg != target_path {
                        let _ = fs::write(&exe_cfg, &json);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn resolve_config_path(data_dir: &Path) -> PathBuf {
        // 1. If existing config exists in data_dir, use it
        let candidate = data_dir.join("companion_config.json");
        if candidate.exists() {
            return candidate;
        }

        // 2. Check user_data_dir()
        let user_cfg = user_data_dir().join("companion_config.json");
        if user_cfg.exists() {
            return user_cfg;
        }

        // 3. Check local directory
        let local = PathBuf::from("companion_config.json");
        if local.exists() {
            if data_dir.is_dir() && candidate != local && is_dir_writable(data_dir) {
                if let Some(parent) = candidate.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                if fs::copy(&local, &candidate).is_ok() {
                    info!(
                        "Migrated existing companion_config.json from {} to {}",
                        local.display(),
                        candidate.display()
                    );
                    return candidate;
                }
            }
            return local;
        }

        if data_dir.is_dir() && is_dir_writable(data_dir) {
            return candidate;
        }

        // Default to user_data_dir
        user_cfg
    }
}

/// Returns true if a directory can be written to by creating and deleting a temporary test file.
pub fn is_dir_writable(dir: &Path) -> bool {
    if !dir.exists() {
        if let Some(parent) = dir.parent() {
            return is_dir_writable(parent);
        }
        return false;
    }
    let test_file = dir.join(format!(".write_test_{}", std::process::id()));
    if fs::write(&test_file, b"ok").is_ok() {
        let _ = fs::remove_file(&test_file);
        true
    } else {
        false
    }
}

/// Returns the user-writable data directory appropriate for the current operating system:
/// - macOS: ~/Library/Application Support/PhotoCraft Companion (fallback ~/PhotoCraftData)
/// - Linux: $XDG_DATA_HOME/photocraft-companion or ~/.local/share/photocraft-companion
/// - Windows: %LOCALAPPDATA%\PhotoCraft Companion (or %APPDATA%\PhotoCraft Companion)
pub fn user_data_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let app_support = PathBuf::from(&home)
                .join("Library")
                .join("Application Support")
                .join("PhotoCraft Companion");
            if (app_support.exists() || fs::create_dir_all(&app_support).is_ok())
                && is_dir_writable(&app_support)
            {
                return app_support;
            }
            let fallback = PathBuf::from(&home).join("PhotoCraftData");
            let _ = fs::create_dir_all(&fallback);
            return fallback;
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            let p = PathBuf::from(xdg).join("photocraft-companion");
            if (p.exists() || fs::create_dir_all(&p).is_ok()) && is_dir_writable(&p) {
                return p;
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let p = PathBuf::from(&home)
                .join(".local")
                .join("share")
                .join("photocraft-companion");
            if (p.exists() || fs::create_dir_all(&p).is_ok()) && is_dir_writable(&p) {
                return p;
            }
            let fallback = PathBuf::from(&home).join("PhotoCraftData");
            let _ = fs::create_dir_all(&fallback);
            return fallback;
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let p = PathBuf::from(local).join("PhotoCraft Companion");
            let _ = fs::create_dir_all(&p);
            return p;
        }
        if let Ok(appdata) = std::env::var("APPDATA") {
            let p = PathBuf::from(appdata).join("PhotoCraft Companion");
            let _ = fs::create_dir_all(&p);
            return p;
        }
        if let Ok(profile) = std::env::var("USERPROFILE") {
            let p = PathBuf::from(profile).join("PhotoCraftData");
            let _ = fs::create_dir_all(&p);
            return p;
        }
    }

    // Generic fallback
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        let p = PathBuf::from(home).join("PhotoCraftData");
        let _ = fs::create_dir_all(&p);
        return p;
    }

    PathBuf::from("PhotoCraftData")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_companion_config_save_load() {
        let temp_dir = std::env::temp_dir().join(format!(
            "cfg_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let cfg_path = temp_dir.join("companion_config.json");

        let cfg = CompanionConfig {
            port: 19999,
            theme: "light".to_string(),
            is_vertical: true,
            enabled_plugins: vec!["test.plugin".to_string()],
            photocraft_path: Some(PathBuf::from("D:/apps/photocraft.exe")),
            ..CompanionConfig::default()
        };

        cfg.save(&cfg_path).unwrap();
        assert!(cfg_path.exists());

        let loaded = CompanionConfig::load_or_default(&cfg_path);
        assert_eq!(loaded.port, 19999);
        assert_eq!(loaded.theme, "light");
        assert!(loaded.is_vertical);
        assert_eq!(loaded.enabled_plugins, vec!["test.plugin"]);
        assert_eq!(
            loaded.photocraft_path,
            Some(PathBuf::from("D:/apps/photocraft.exe"))
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_companion_config_defaults() {
        let cfg = CompanionConfig::default();
        assert_eq!(cfg.host, "127.0.0.1");
        assert_eq!(cfg.port, 18492);
        assert_eq!(cfg.theme, "dark");
        assert_eq!(cfg.language, "auto");
        assert!(!cfg.is_vertical);
        assert!(cfg.token.is_none());
        assert!(cfg.window_position.is_none());
        assert!(cfg.enabled_plugins.is_empty());
        assert!(cfg.photocraft_path.is_none());
    }

    #[test]
    fn test_companion_config_load_nonexistent() {
        let non_existent = PathBuf::from("non_existent_folder_xyz/config.json");
        let cfg = CompanionConfig::load_or_default(&non_existent);
        assert_eq!(cfg.port, 18492);
        assert_eq!(cfg.host, "127.0.0.1");
    }

    #[test]
    fn test_resolve_config_path() {
        let temp_dir = std::env::temp_dir().join(format!(
            "cfg_resolve_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::create_dir_all(&temp_dir);
        let resolved = CompanionConfig::resolve_config_path(&temp_dir);
        assert_eq!(resolved, temp_dir.join("companion_config.json"));
        let _ = fs::remove_dir_all(&temp_dir);

        let non_dir = Path::new("non_existent_dir_99999");
        let fallback = CompanionConfig::resolve_config_path(non_dir);
        assert_eq!(fallback, user_data_dir().join("companion_config.json"));
    }

    #[test]
    fn test_user_data_dir_writable() {
        let dir = user_data_dir();
        assert!(is_dir_writable(&dir));
    }
}
