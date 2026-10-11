use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    pub hotkey_overrides: HashMap<String, String>,
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
            hotkey_overrides: HashMap::new(),
        }
    }
}

impl CompanionConfig {
    pub fn load_or_default(path: impl AsRef<Path>) -> Self {
        let p = path.as_ref();
        if p.exists() {
            if let Ok(content) = fs::read_to_string(p) {
                if let Ok(cfg) = serde_json::from_str::<Self>(&content) {
                    info!("Loaded companion configuration from {}", p.display());
                    return cfg;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(p, json)?;
        info!("Saved companion configuration to {}", p.display());
        Ok(())
    }

    pub fn resolve_config_path(data_dir: &Path) -> PathBuf {
        let candidate = data_dir.join("companion_config.json");
        if candidate.exists() || data_dir.is_dir() {
            return candidate;
        }
        PathBuf::from("companion_config.json")
    }
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
            ..CompanionConfig::default()
        };

        cfg.save(&cfg_path).unwrap();
        assert!(cfg_path.exists());

        let loaded = CompanionConfig::load_or_default(&cfg_path);
        assert_eq!(loaded.port, 19999);
        assert_eq!(loaded.theme, "light");
        assert!(loaded.is_vertical);
        assert_eq!(loaded.enabled_plugins, vec!["test.plugin"]);

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
        assert!(cfg.hotkey_overrides.is_empty());
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
        assert_eq!(fallback, PathBuf::from("companion_config.json"));
    }
}
