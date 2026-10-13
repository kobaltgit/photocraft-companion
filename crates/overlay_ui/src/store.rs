use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use tracing::{info, warn};

pub const DEFAULT_REGISTRY_URL: &str =
    "https://github.com/kobaltgit/photocraft-companion/releases/download/plugins/registry.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoreModelInfo {
    pub name: String,
    pub required: bool,
    pub size_mb: Option<u64>,
    pub url: Option<String>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StorePluginEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub plugin_type: String, // "native_ai", "no_code", "script", "cli"
    pub download_url: String,
    pub icon_url: Option<String>,
    #[serde(default)]
    pub models: Vec<StoreModelInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoreRegistry {
    pub generated_at: Option<String>,
    pub total_plugins: usize,
    pub plugins: Vec<StorePluginEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreLoadStatus {
    Idle,
    Loading,
    Ready { is_offline: bool },
    Error(String),
}

pub enum StoreEvent {
    RegistryLoaded {
        registry: StoreRegistry,
        is_offline: bool,
    },
    RegistryFailed(String),
    PluginInstalled {
        plugin_id: String,
        manifest: Box<core_contracts::PluginManifest>,
    },
    PluginInstallFailed {
        plugin_id: String,
        error: String,
    },
    IconDownloaded {
        plugin_id: String,
        icon_path: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreCategoryFilter {
    All,
    Ai,
    Macro,
}

pub struct StoreState {
    pub registry: Option<StoreRegistry>,
    pub status: StoreLoadStatus,
    pub search_query: String,
    pub category_filter: StoreCategoryFilter,
    pub installing_plugins: Vec<String>, // plugin ids currently installing
    pub event_tx: Sender<StoreEvent>,
    pub event_rx: Receiver<StoreEvent>,
    pub cache_file: PathBuf,
    pub icons_cache_dir: PathBuf,
}

impl StoreState {
    pub fn new(data_dir: &Path) -> Self {
        let (event_tx, event_rx) = channel();
        let cache_dir = data_dir.join("cache");
        let _ = fs::create_dir_all(&cache_dir);
        let icons_cache_dir = cache_dir.join("icons");
        let _ = fs::create_dir_all(&icons_cache_dir);
        let cache_file = cache_dir.join("registry.json");

        Self {
            registry: None,
            status: StoreLoadStatus::Idle,
            search_query: String::new(),
            category_filter: StoreCategoryFilter::All,
            installing_plugins: Vec::new(),
            event_tx,
            event_rx,
            cache_file,
            icons_cache_dir,
        }
    }

    /// Triggers background fetch of registry.json with disk cache fallback
    pub fn refresh_catalog(&mut self, custom_url: Option<String>) {
        self.status = StoreLoadStatus::Loading;
        let tx = self.event_tx.clone();
        let cache_path = self.cache_file.clone();
        let url = custom_url.unwrap_or_else(|| DEFAULT_REGISTRY_URL.to_string());

        thread::spawn(move || match fetch_registry_data(&url, &cache_path) {
            Ok((registry, is_offline)) => {
                let _ = tx.send(StoreEvent::RegistryLoaded {
                    registry,
                    is_offline,
                });
            }
            Err(err) => {
                let _ = tx.send(StoreEvent::RegistryFailed(err));
            }
        });
    }

    /// Triggers background installation of a plugin by its download_url
    pub fn install_plugin(&mut self, entry: &StorePluginEntry, plugins_dir: PathBuf) {
        if self.installing_plugins.contains(&entry.id) {
            return;
        }
        self.installing_plugins.push(entry.id.clone());

        let tx = self.event_tx.clone();
        let plugin_id = entry.id.clone();
        let download_url = entry.download_url.clone();

        thread::spawn(
            move || match download_and_install_pcpl(&download_url, &plugins_dir) {
                Ok(manifest) => {
                    let _ = tx.send(StoreEvent::PluginInstalled {
                        plugin_id,
                        manifest: Box::new(manifest),
                    });
                }
                Err(e) => {
                    let _ = tx.send(StoreEvent::PluginInstallFailed {
                        plugin_id,
                        error: e,
                    });
                }
            },
        );
    }

    /// Triggers background download of plugin icon if missing from cache
    pub fn ensure_icon_cached(&self, entry: &StorePluginEntry) {
        let icon_url = match &entry.icon_url {
            Some(u) if !u.is_empty() => u.clone(),
            _ => return,
        };

        let target_icon_path = self.get_cached_icon_path(&entry.id);
        if target_icon_path.exists() {
            return;
        }

        let tx = self.event_tx.clone();
        let plugin_id = entry.id.clone();

        thread::spawn(move || {
            let temp_path = target_icon_path.with_extension("svg.tmp");
            if download_file_to_path(&icon_url, &temp_path).is_ok()
                && fs::rename(&temp_path, &target_icon_path).is_ok()
            {
                let _ = tx.send(StoreEvent::IconDownloaded {
                    plugin_id,
                    icon_path: target_icon_path,
                });
            }
        });
    }

    pub fn get_cached_icon_path(&self, plugin_id: &str) -> PathBuf {
        let safe_name = plugin_id.replace(['/', '\\', ':'], "_");
        self.icons_cache_dir.join(format!("{}.svg", safe_name))
    }
}

/// Downloads data from HTTP URL or loads from local cache file on failure
pub fn fetch_registry_data(url: &str, cache_path: &Path) -> Result<(StoreRegistry, bool), String> {
    info!("Fetching marketplace registry from {}", url);

    let mut network_err = None;

    // 1. Try to download via curl or powershell
    let output = Command::new("curl")
        .arg("-s")
        .arg("-L")
        .arg("-f")
        .arg("-H")
        .arg("User-Agent: PhotoCraft-Companion-Store")
        .arg(url)
        .output();

    let json_text = match output {
        Ok(out) if out.status.success() && !out.stdout.is_empty() => {
            String::from_utf8_lossy(&out.stdout).to_string()
        }
        Ok(out) => {
            network_err = Some(format!(
                "curl failed with status code: {:?}",
                out.status.code()
            ));
            String::new()
        }
        Err(e) => {
            if cfg!(target_os = "windows") {
                let ps_cmd = format!(
                    "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; (Invoke-WebRequest -Uri '{}' -UseBasicParsing).Content",
                    url
                );
                let ps_out = Command::new("powershell")
                    .arg("-NoProfile")
                    .arg("-Command")
                    .arg(&ps_cmd)
                    .output();
                match ps_out {
                    Ok(p) if p.status.success() && !p.stdout.is_empty() => {
                        String::from_utf8_lossy(&p.stdout).to_string()
                    }
                    Ok(p) => {
                        network_err = Some(format!(
                            "PowerShell failed: {}",
                            String::from_utf8_lossy(&p.stderr)
                        ));
                        String::new()
                    }
                    Err(pe) => {
                        network_err = Some(format!("PowerShell execution error: {pe}"));
                        String::new()
                    }
                }
            } else {
                network_err = Some(format!("curl error: {e}"));
                String::new()
            }
        }
    };

    if !json_text.is_empty() {
        if let Ok(reg) = serde_json::from_str::<StoreRegistry>(&json_text) {
            // Save to disk cache for offline use
            if let Some(parent) = cache_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Err(e) = fs::write(cache_path, &json_text) {
                warn!("Failed to write store cache: {e}");
            }
            return Ok((reg, false));
        } else {
            network_err = Some("Failed to parse registry.json payload".to_string());
        }
    }

    // 2. Fallback to local cache
    if cache_path.exists() {
        if let Ok(cached_content) = fs::read_to_string(cache_path) {
            if let Ok(reg) = serde_json::from_str::<StoreRegistry>(&cached_content) {
                info!(
                    "Using cached marketplace registry from {}",
                    cache_path.display()
                );
                return Ok((reg, true));
            }
        }
    }

    Err(network_err.unwrap_or_else(|| {
        "Could not connect to plugin repository and no offline cache available.".to_string()
    }))
}

/// Downloads a .pcpl archive to temp dir and unpacks it into plugins_dir
pub fn download_and_install_pcpl(
    download_url: &str,
    plugins_dir: &Path,
) -> Result<core_contracts::PluginManifest, String> {
    let temp_pcpl = std::env::temp_dir().join(format!("pcpl_inst_{}.pcpl", std::process::id()));
    let _ = fs::remove_file(&temp_pcpl);

    download_file_to_path(download_url, &temp_pcpl).map_err(|e| format!("Download error: {e}"))?;

    let manifest_res = core_contracts::install_pcpl(&temp_pcpl, plugins_dir);
    let _ = fs::remove_file(&temp_pcpl);

    manifest_res.map_err(|e| format!("Install package error: {e}"))
}

/// Generic file download helper using curl or powershell
pub fn download_file_to_path(url: &str, target_path: &Path) -> Result<(), String> {
    if let Some(parent) = target_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let output = Command::new("curl")
        .arg("-L")
        .arg("-f")
        .arg("-s")
        .arg("-o")
        .arg(target_path)
        .arg(url)
        .status();

    match output {
        Ok(s) if s.success() => Ok(()),
        _ => {
            if cfg!(target_os = "windows") {
                let ps_cmd = format!(
                    "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; (New-Object System.Net.WebClient).DownloadFile('{}', '{}')",
                    url,
                    target_path.to_string_lossy().replace('\'', "''")
                );
                let ps_out = Command::new("powershell")
                    .arg("-NoProfile")
                    .arg("-Command")
                    .arg(&ps_cmd)
                    .status()
                    .map_err(|e| format!("PowerShell error: {e}"))?;

                if ps_out.success() {
                    Ok(())
                } else {
                    Err("Failed to download file via PowerShell".to_string())
                }
            } else {
                Err("Failed to download file via curl".to_string())
            }
        }
    }
}

/// Compares store version and local version using crate::updater::is_version_newer
pub fn is_store_version_newer(store_version: &str, local_version: &str) -> bool {
    crate::updater::is_version_newer(store_version, local_version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_store_registry() {
        let json_data = r#"{
            "generated_at": "2026-10-13T14:59:11Z",
            "total_plugins": 2,
            "plugins": [
                {
                    "id": "macro.photocraft.plugins.black_and_white",
                    "name": "B&W Film Pop (No-Code)",
                    "version": "1.0.0",
                    "description": "Duplicates active layer, converts to grayscale.",
                    "author": "kobaltGIT",
                    "plugin_type": "no_code",
                    "download_url": "https://example.com/bw.pcpl",
                    "icon_url": "https://example.com/bw.svg",
                    "models": []
                },
                {
                    "id": "ai.photocraft.plugins.inpaint",
                    "name": "AI Generative Fill",
                    "version": "1.2.0",
                    "description": "LaMa generative fill.",
                    "author": "kobaltGIT",
                    "plugin_type": "native_ai",
                    "download_url": "https://example.com/inpaint.pcpl",
                    "icon_url": null,
                    "models": [
                        {
                            "name": "lama.onnx",
                            "required": true,
                            "size_mb": 208,
                            "url": "https://huggingface.co/lama.onnx",
                            "sha256": null
                        }
                    ]
                }
            ]
        }"#;

        let reg: StoreRegistry = serde_json::from_str(json_data).expect("Valid registry JSON");
        assert_eq!(reg.total_plugins, 2);
        assert_eq!(reg.plugins.len(), 2);
        assert_eq!(reg.plugins[0].name, "B&W Film Pop (No-Code)");
        assert_eq!(reg.plugins[1].models.len(), 1);
        assert_eq!(reg.plugins[1].models[0].size_mb, Some(208));
    }

    #[test]
    fn test_version_newer_logic() {
        assert!(is_store_version_newer("1.1.0", "1.0.0"));
        assert!(is_store_version_newer("2.0.0", "1.9.9"));
        assert!(!is_store_version_newer("1.0.0", "1.0.0"));
        assert!(!is_store_version_newer("1.0.0", "1.1.0"));
    }

    #[test]
    fn test_offline_cache_fallback() {
        let temp_dir = std::env::temp_dir().join(format!("pc_store_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);
        let cache_file = temp_dir.join("registry.json");

        let sample_json = r#"{
            "total_plugins": 1,
            "plugins": [
                {
                    "id": "test.offline",
                    "name": "Offline Plugin",
                    "version": "1.0.0",
                    "description": "Works offline",
                    "author": "tester",
                    "plugin_type": "no_code",
                    "download_url": "https://dummy.url",
                    "icon_url": null,
                    "models": []
                }
            ]
        }"#;
        fs::write(&cache_file, sample_json).unwrap();

        // Using an invalid URL should trigger offline cache fallback
        let res = fetch_registry_data("http://127.0.0.1:9999/nonexistent.json", &cache_file);
        assert!(res.is_ok());
        let (registry, is_offline) = res.unwrap();
        assert!(is_offline);
        assert_eq!(registry.plugins.len(), 1);
        assert_eq!(registry.plugins[0].name, "Offline Plugin");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
