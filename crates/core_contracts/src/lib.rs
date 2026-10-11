use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompanionError {
    #[error("Bridge error: {0}")]
    Bridge(String),
    #[error("AI Engine error: {0}")]
    AiEngine(String),
    #[error("Plugin error: {0}")]
    Plugin(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, CompanionError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonConfig {
    pub label: String,
    pub action: String,
    pub hotkey: Option<String>,
    pub icon: Option<String>,
}

pub mod i18n;
pub use i18n::Language;

pub mod plugin;
pub use plugin::{
    BridgeOperations, ModelRequirement, Plugin, PluginCommandStep, PluginContext,
    PluginManifest, PluginResult, PluginType,
};

pub mod config;
pub use config::CompanionConfig;

pub mod package;
pub use package::{create_pcpl, install_pcpl};

pub mod registry;
pub use registry::{NoCodePlugin, PluginRegistry};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub active_layer: Option<u64>,
    pub active_layer_name: Option<String>,
}
