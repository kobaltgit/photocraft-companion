use crate::{DocumentInfo, Result};
use async_trait::async_trait;
use image::RgbaImage;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::action::{AlgorithmConfig, PluginCommandStep, SubAction};
use super::pipeline::AiPipelineConfig;
use super::widget::InteractiveUiConfig;

fn default_author() -> String {
    "PhotoCraft Community".to_string()
}

fn default_true() -> bool {
    true
}

/// Localized metadata for a plugin or sub-action
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PluginLocaleInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Binary/CLI runtime dependency needed by a plugin (e.g. ffmpeg, vtracer)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuntimeToolRequirement {
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url_windows_x64: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url_linux_x64: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url_macos_arm64: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url_macos_x64: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

/// Manifest definition for any plugin (No-Code, AI, Script, or External)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_ru: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description_ru: Option<String>,
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub locales: std::collections::HashMap<String, PluginLocaleInfo>,
    #[serde(default = "default_author")]
    pub author: String,
    #[serde(default)]
    pub plugin_type: PluginType,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub commands: Vec<PluginCommandStep>,
    #[serde(default)]
    pub post_commands: Vec<PluginCommandStep>,
    #[serde(default)]
    pub sub_actions: Vec<SubAction>,
    #[serde(default)]
    pub input: PluginInputSource,
    #[serde(default)]
    pub output_mode: PluginOutputMode,
    #[serde(default)]
    pub models: Vec<ModelRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runtime_tools: Vec<RuntimeToolRequirement>,
    #[serde(default)]
    pub action_id: Option<String>,
    #[serde(default)]
    pub entrypoint: Option<String>,
    #[serde(default)]
    pub ai: Option<AiPipelineConfig>,
    #[serde(default)]
    pub algorithm: Option<AlgorithmConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interactive_ui: Option<InteractiveUiConfig>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(skip)]
    pub plugin_dir: Option<PathBuf>,
}

/// Source of image data for plugin execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PluginInputSource {
    #[default]
    ActiveLayer,
    Selection,
    Canvas,
}

/// Output integration mode of plugin result in PhotoCraft
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PluginOutputMode {
    #[default]
    InPlace,
    NewLayer,
    Mask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PluginType {
    /// Declarative macro running PhotoCraft engine commands sequentially
    #[default]
    NoCode,
    /// Native Rust/C++ AI inference or high-perf memory processing
    NativeAi,
    /// Native algorithmic processing without neural weights (e.g. VTracer vectorization)
    NativeAlgorithm,
    /// Script-based automation (e.g. Rhai / Lua / JS)
    Script,
    /// External CLI process communicating via stdin/stdout or clipboard
    ExternalCli,
    /// Dynamic interactive user interface plugin (with custom widgets or script bindings)
    InteractiveUi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequirement {
    #[serde(default)]
    pub role: Option<String>,
    pub name: String,
    pub required: bool,
    pub size_mb: Option<u64>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub sha256: Option<String>,
}

impl PluginManifest {
    pub fn is_model_installed(&self, model: &ModelRequirement) -> bool {
        // Enforce strict model isolation: model name must be a pure filename without path traversal
        if model.name.contains("..") || model.name.contains('/') || model.name.contains('\\') {
            return false;
        }

        let is_valid_file = |path: &PathBuf| {
            if let Ok(meta) = std::fs::metadata(path) {
                meta.is_file()
                    && meta.len() > 0
                    && !path
                        .extension()
                        .map(|ext| ext.eq_ignore_ascii_case("part"))
                        .unwrap_or(false)
            } else {
                false
            }
        };

        if let Some(ref dir) = self.plugin_dir {
            let direct = dir.join("models").join(&model.name);
            if is_valid_file(&direct) {
                return true;
            }
            let flat = dir.join(&model.name);
            if is_valid_file(&flat) {
                return true;
            }
        }
        // Fallback global check in models/
        let global = PathBuf::from("models").join(&model.name);
        if is_valid_file(&global) {
            return true;
        }
        let user_models = crate::user_data_dir().join("models").join(&model.name);
        if is_valid_file(&user_models) {
            return true;
        }
        let user_plugin_models = crate::user_data_dir()
            .join("plugins")
            .join(&self.id)
            .join("models")
            .join(&model.name);
        if is_valid_file(&user_plugin_models) {
            return true;
        }
        let fallback = PathBuf::from("PhotoCraftData")
            .join("models")
            .join(&model.name);
        is_valid_file(&fallback)
    }

    pub fn all_models_installed(&self) -> bool {
        if self.models.is_empty() {
            return true;
        }
        self.models.iter().all(|m| self.is_model_installed(m))
    }

    pub fn required_models_installed(&self) -> bool {
        let required_models: Vec<_> = self.models.iter().filter(|m| m.required).collect();
        if required_models.is_empty() {
            return self.all_models_installed();
        }
        required_models.iter().all(|m| self.is_model_installed(m))
    }

    pub fn model_by_role(&self, role: &str) -> Option<&ModelRequirement> {
        self.models.iter().find(|m| m.role.as_deref() == Some(role))
    }

    pub fn localized_name(&self, lang: &str) -> &str {
        let lang_code = lang.to_ascii_lowercase();
        if let Some(locale) = self.locales.get(&lang_code) {
            if let Some(ref name) = locale.name {
                return name;
            }
        }
        if let Some((base, _)) = lang_code.split_once('-') {
            if let Some(locale) = self.locales.get(base) {
                if let Some(ref name) = locale.name {
                    return name;
                }
            }
        }
        if lang_code == "ru" || lang_code.starts_with("ru") {
            if let Some(ref name_ru) = self.name_ru {
                return name_ru;
            }
        }
        &self.name
    }

    pub fn localized_description(&self, lang: &str) -> &str {
        let lang_code = lang.to_ascii_lowercase();
        if let Some(locale) = self.locales.get(&lang_code) {
            if let Some(ref desc) = locale.description {
                return desc;
            }
        }
        if let Some((base, _)) = lang_code.split_once('-') {
            if let Some(locale) = self.locales.get(base) {
                if let Some(ref desc) = locale.description {
                    return desc;
                }
            }
        }
        if lang_code == "ru" || lang_code.starts_with("ru") {
            if let Some(ref desc_ru) = self.description_ru {
                return desc_ru;
            }
        }
        &self.description
    }

    pub fn name_for_lang(&self, lang: crate::Language) -> &str {
        self.localized_name(lang.code())
    }

    pub fn description_for_lang(&self, lang: crate::Language) -> &str {
        self.localized_description(lang.code())
    }

    /// Resolves the absolute path to the plugin's icon file (e.g. icon.svg).
    ///
    /// Priority:
    /// 1. `self.plugin_dir.join(icon)` if explicitly configured;
    /// 2. `self.plugin_dir.join("icon.svg")` as default;
    /// 3. Fallback to `root_plugins_dir.join(&self.id)` if `plugin_dir` is not populated.
    pub fn resolve_icon_path(&self, root_plugins_dir: Option<&Path>) -> Option<PathBuf> {
        let is_valid_file = |path: &Path| -> bool {
            if let Ok(meta) = std::fs::metadata(path) {
                meta.is_file() && meta.len() > 0
            } else {
                false
            }
        };

        // 1. Try directly with self.plugin_dir
        if let Some(ref dir) = self.plugin_dir {
            if let Some(ref icon_file) = self.icon {
                let path = dir.join(icon_file);
                if is_valid_file(&path) {
                    return Some(path);
                }
            }
            let default_svg = dir.join("icon.svg");
            if is_valid_file(&default_svg) {
                return Some(default_svg);
            }
        }

        // 2. Fallback using root_plugins_dir if provided
        if let Some(root) = root_plugins_dir {
            let candidate_dir = root.join(&self.id);
            if let Some(ref icon_file) = self.icon {
                let path = candidate_dir.join(icon_file);
                if is_valid_file(&path) {
                    return Some(path);
                }
            }
            let default_svg = candidate_dir.join("icon.svg");
            if is_valid_file(&default_svg) {
                return Some(default_svg);
            }
        }

        None
    }
}

impl Default for PluginManifest {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            version: "1.0.0".to_string(),
            description: String::new(),
            name_ru: None,
            description_ru: None,
            locales: std::collections::HashMap::new(),
            author: default_author(),
            plugin_type: PluginType::default(),
            icon: None,
            commands: Vec::new(),
            post_commands: Vec::new(),
            sub_actions: Vec::new(),
            input: PluginInputSource::default(),
            output_mode: PluginOutputMode::default(),
            models: Vec::new(),
            runtime_tools: Vec::new(),
            action_id: None,
            entrypoint: None,
            ai: None,
            algorithm: None,
            interactive_ui: None,
            enabled: true,
            plugin_dir: None,
        }
    }
}

/// Execution status of an action for floating UI HUD visualization and progress tracking
#[derive(Debug, Clone)]
pub enum ExecutionState {
    Idle,
    Running {
        action_id: String,
        action_name: String,
        started_at: std::time::Instant,
    },
    Success {
        action_name: String,
        message: String,
        duration_secs: f32,
        finished_at: std::time::Instant,
    },
    Error {
        action_name: String,
        error: String,
        finished_at: std::time::Instant,
    },
}

pub type SharedExecutionStatus = Arc<std::sync::Mutex<ExecutionState>>;

/// Context provided to a plugin during execution
pub struct PluginContext {
    pub document: DocumentInfo,
    pub models_dir: PathBuf,
    pub plugin_dir: PathBuf,
    pub client: Arc<dyn BridgeOperations + Send + Sync>,
    pub sub_action_id: Option<String>,
}

impl PluginContext {
    pub fn new(
        document: DocumentInfo,
        models_dir: PathBuf,
        plugin_dir: PathBuf,
        client: Arc<dyn BridgeOperations + Send + Sync>,
    ) -> Self {
        Self {
            document,
            models_dir,
            plugin_dir,
            client,
            sub_action_id: None,
        }
    }

    pub fn with_sub_action(mut self, sub_action_id: Option<String>) -> Self {
        self.sub_action_id = sub_action_id;
        self
    }
}

/// Abstraction over PhotoCraft bridge operations for plugin isolation
#[async_trait]
pub trait BridgeOperations {
    async fn execute_command(
        &self,
        command: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value>;
    async fn copy_active_layer_image(&self) -> Result<RgbaImage>;
    async fn paste_image_in_place(&self, image: &RgbaImage) -> Result<serde_json::Value>;
    async fn rename_layer(&self, layer_id: u64, new_name: &str) -> Result<serde_json::Value>;
    async fn set_layer_props(
        &self,
        layer_id: u64,
        props: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let mut full_props = props;
        if let Some(map) = full_props.as_object_mut() {
            map.entry("layer")
                .or_insert_with(|| serde_json::json!(layer_id));
            map.entry("layerId")
                .or_insert_with(|| serde_json::json!(layer_id));
        }
        self.execute_command("layer.setProps", full_props).await
    }
    async fn get_document_info(&self) -> Result<DocumentInfo>;
    async fn deselect(&self) -> Result<serde_json::Value> {
        self.execute_command("select.deselect", serde_json::json!({}))
            .await
    }
    async fn merge_down(&self) -> Result<serde_json::Value> {
        self.execute_command("layer.mergeDown", serde_json::json!({}))
            .await
    }
    async fn copy_active_layer_and_selection_mask(
        &self,
    ) -> Result<(RgbaImage, Option<image::GrayImage>)> {
        let doc = self.get_document_info().await?;
        if !doc.has_selection {
            let img = self.copy_active_layer_image().await?;
            return Ok((img, None));
        }

        let cutout = self.copy_active_layer_image().await?;
        let _ = self
            .execute_command("select.deselect", serde_json::json!({}))
            .await;
        let full_img = self.copy_active_layer_image().await?;
        let _ = self
            .execute_command("select.reselect", serde_json::json!({}))
            .await;

        let (w, h) = (full_img.width(), full_img.height());
        let mut mask = image::GrayImage::new(w, h);
        let (bx0, by0) = doc
            .selection_bounds
            .map(|b| (b[0].max(0) as u32, b[1].max(0) as u32))
            .unwrap_or((0, 0));

        for cy in 0..cutout.height() {
            for cx in 0..cutout.width() {
                if cutout.get_pixel(cx, cy)[3] > 0 {
                    let tx = bx0 + cx;
                    let ty = by0 + cy;
                    if tx < w && ty < h {
                        mask.put_pixel(tx, ty, image::Luma([255]));
                    }
                }
            }
        }

        Ok((full_img, Some(mask)))
    }
}

/// Core interface implemented by any PhotoCraft plugin
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Unique plugin identifier
    fn id(&self) -> &str;

    /// Plugin manifest
    fn manifest(&self) -> &PluginManifest;

    /// Execute the primary plugin action
    async fn execute(&self, ctx: &PluginContext) -> Result<PluginResult>;
}

/// Execution result returned by a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl PluginResult {
    pub fn ok(msg: impl Into<String>) -> Self {
        Self {
            success: true,
            message: msg.into(),
            data: None,
        }
    }

    pub fn ok_with_data(msg: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            success: true,
            message: msg.into(),
            data: Some(data),
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            message: msg.into(),
            data: None,
        }
    }
}
