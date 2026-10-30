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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    Offline,
    Connected,
    AuthRequired,
}

use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ButtonConfig {
    pub label: String,
    pub action: String,
    pub icon: Option<String>,
    #[serde(default)]
    pub icon_path: Option<PathBuf>,
    #[serde(default)]
    pub plugin_dir: Option<PathBuf>,
    #[serde(default)]
    pub sub_actions: Vec<SubAction>,
    #[serde(default)]
    pub active_sub_action: Option<String>,
    #[serde(default)]
    pub name_ru: Option<String>,
    #[serde(default)]
    pub locales: std::collections::HashMap<String, PluginLocaleInfo>,
    #[serde(default)]
    pub requires_selection: bool,
}

impl ButtonConfig {
    pub fn name_for_lang(&self, lang: Language) -> &str {
        let code = lang.code();
        if let Some(loc) = self.locales.get(code) {
            if let Some(ref n) = loc.name {
                if !n.trim().is_empty() {
                    return n;
                }
            }
        }
        if (code == "ru" || code.starts_with("ru")) && self.name_ru.is_some() {
            return self.name_ru.as_deref().unwrap();
        }
        &self.label
    }

    pub fn sub_action_icon_path(&self, sub: &SubAction) -> Option<PathBuf> {
        if let (Some(dir), Some(icon_name)) = (&self.plugin_dir, &sub.icon) {
            let p = dir.join(icon_name);
            if p.is_file() {
                return Some(p);
            }
        }
        self.icon_path.clone()
    }

    pub fn effective_icon_path(&self) -> Option<PathBuf> {
        if let Some(ref active_id) = self.active_sub_action {
            if let Some(sub) = self.sub_actions.iter().find(|s| &s.id == active_id) {
                if let Some(p) = self.sub_action_icon_path(sub) {
                    return Some(p);
                }
            }
        }
        self.icon_path.clone()
    }
}

pub mod i18n;
pub use i18n::Language;

pub mod host_i18n;
pub use host_i18n::{is_translatable_param, normalize_term, to_canonical, to_host_locale};

pub mod plugin;
pub use plugin::{
    ActionRequest, AiOutputType, AiPipelineConfig, AiTask, AlgorithmConfig, BannerStyle,
    BridgeOperations, ExecutionState, InteractiveUiConfig, ModelRequirement, ModifierItemConfig,
    NormalizationMode, Plugin, PluginCommandStep, PluginContext, PluginInputSource,
    PluginIpcAction, PluginLocaleInfo, PluginManifest, PluginOutputMode, PluginResult, PluginType,
    SharedExecutionStatus, SubAction, WidgetNode, IPC_SUB_ACTIVE_LAYER_IMAGE,
    IPC_SUB_SELECTION_STATE, IPC_SUB_SESSION_COLORS,
};

pub mod config;
pub use config::{is_dir_writable, user_data_dir, CompanionConfig};

pub mod package;
pub use package::{create_pcpl, install_pcpl};

pub mod registry;
pub use registry::{AiPluginFactory, AlgorithmPluginFactory, NoCodePlugin, PluginRegistry};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentInfo {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub active_layer: Option<u64>,
    pub active_layer_name: Option<String>,
    #[serde(default)]
    pub has_selection: bool,
    #[serde(default)]
    pub selection_bounds: Option<[i32; 4]>,
    #[serde(default)]
    pub active_layer_bounds: Option<[i32; 4]>,
}
