use core_contracts::PluginManifest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

#[derive(Debug)]
pub struct ValidatedPlugin {
    pub dir_name: String,
    pub dir_path: PathBuf,
    pub manifest: PluginManifest,
    pub icon_path: Option<PathBuf>,
}
