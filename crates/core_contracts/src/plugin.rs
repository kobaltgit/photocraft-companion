use crate::{DocumentInfo, Result};
use async_trait::async_trait;
use image::RgbaImage;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Localized metadata for a plugin or sub-action
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct PluginLocaleInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
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

/// Declarative AI pipeline specification for zero-code ONNX execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AiTask {
    ImageToImage,     // e.g. Denoise, deblur, restoration, colorization 1:1
    SegmentationMask, // e.g. RMBG background removal
    SuperResolution,  // e.g. Real-ESRGAN 2x
    Inpainting,       // e.g. LaMa zero-prompt filling
}

/// Normalization modes for tensor inputs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum NormalizationMode {
    #[default]
    ZeroToOne, // pixel / 255.0 ([0.0, 1.0])
    Centered, // (pixel / 255.0 - 0.5) / 1.0 (RMBG)
    #[serde(alias = "raw_255")]
    Raw255, // raw pixel 0.0..255.0 (LaMa)
    ImageNet, // (pixel / 255.0 - mean) / std (Depth Anything, ViT)
}

/// Output interpretation for single-channel or segmentation models
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AiOutputType {
    #[default]
    AlphaCutout,
    GrayscaleLayer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPipelineConfig {
    pub task: AiTask,
    #[serde(default)]
    pub scale: Option<u32>,
    #[serde(default)]
    pub tile_size: Option<u32>,
    #[serde(default)]
    pub tile_pad: Option<u32>,
    #[serde(default)]
    pub normalize: Option<NormalizationMode>,
    #[serde(default)]
    pub input_size: Option<u32>,
    #[serde(default)]
    pub output_type: Option<AiOutputType>,
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

fn default_author() -> String {
    "PhotoCraft Community".to_string()
}

fn default_true() -> bool {
    true
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

/// Visual style variant for informative UI banners
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BannerStyle {
    #[default]
    Info,
    Warning,
    Error,
    Success,
}

/// Abstract declarative UI widget node for dynamic plugin panels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WidgetNode {
    ColorSlot {
        id: String,
        label: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label_ru: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        default_hex: Option<String>,
        #[serde(default)]
        read_only: bool,
        #[serde(default)]
        selectable: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target_binding: Option<String>,
    },
    Slider {
        id: String,
        label: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label_ru: Option<String>,
        min: f64,
        max: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        step: Option<f64>,
        default: f64,
    },
    Dropdown {
        id: String,
        label: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label_ru: Option<String>,
        options: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        options_ru: Option<Vec<String>>,
        #[serde(default)]
        default_index: usize,
    },
    Toggle {
        id: String,
        label: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label_ru: Option<String>,
        #[serde(default)]
        default: bool,
    },
    Button {
        id: String,
        label: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label_ru: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        icon: Option<String>,
    },
    HueWheel {
        id: String,
        source_color_id: String,
        #[serde(default = "default_true")]
        show_pointer: bool,
    },
    Banner {
        id: String,
        message_binding: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        style_binding: Option<String>,
    },
    Label {
        text_binding: String,
    },
    Separator,
    Row {
        #[serde(default)]
        items: Vec<WidgetNode>,
    },
}

impl WidgetNode {
    pub fn label_for_lang(&self, lang: crate::Language) -> &str {
        let is_ru = matches!(lang, crate::Language::Ru);
        match self {
            WidgetNode::ColorSlot {
                label, label_ru, ..
            }
            | WidgetNode::Slider {
                label, label_ru, ..
            }
            | WidgetNode::Dropdown {
                label, label_ru, ..
            }
            | WidgetNode::Toggle {
                label, label_ru, ..
            }
            | WidgetNode::Button {
                label, label_ru, ..
            } => {
                if is_ru {
                    label_ru.as_deref().unwrap_or(label)
                } else {
                    label
                }
            }
            _ => "",
        }
    }

    pub fn options_for_lang(&self, lang: crate::Language) -> &[String] {
        let is_ru = matches!(lang, crate::Language::Ru);
        match self {
            WidgetNode::Dropdown {
                options,
                options_ru,
                ..
            } => {
                if is_ru {
                    options_ru.as_deref().unwrap_or(options.as_slice())
                } else {
                    options.as_slice()
                }
            }
            _ => &[],
        }
    }
}

/// Configuration for dynamic interactive UI plugins
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct InteractiveUiConfig {
    pub window_title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_title_ru: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_width: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_height: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub widgets: Vec<WidgetNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ipc_subscriptions: Vec<String>,
}

/// Standard IPC subscription topic for listening to active session colors in PhotoCraft
pub const IPC_SUB_SESSION_COLORS: &str = "session.colors";

impl InteractiveUiConfig {
    pub fn window_title_for_lang(&self, lang: crate::Language) -> &str {
        if matches!(lang, crate::Language::Ru) {
            self.window_title_ru
                .as_deref()
                .unwrap_or(&self.window_title)
        } else {
            &self.window_title
        }
    }
}

/// Abstract action DTO sent between interactive plugin UI/script and companion engine
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginIpcAction {
    pub command: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

impl PluginIpcAction {
    pub fn new(command: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            command: command.into(),
            params,
        }
    }

    pub fn command_only(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            params: serde_json::Value::Null,
        }
    }
}

/// Declarative algorithm pipeline specification for non-neural tasks (vectorization, 3D LUT, etc.)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlgorithmConfig {
    pub task: String,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub color_mode: Option<String>,
    #[serde(default)]
    pub filter_speckle: Option<usize>,
    #[serde(default)]
    pub corner_threshold: Option<i32>,
    #[serde(default = "default_true")]
    pub copy_clipboard: bool,
    #[serde(default = "default_true")]
    pub export_file: bool,
}

/// A step in a declarative No-Code pipeline or workflow execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginCommandStep {
    pub command: String,
    #[serde(default)]
    pub params: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub save_as: Option<String>,
    #[serde(default)]
    pub ignore_errors: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl PluginCommandStep {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            params: serde_json::Value::Null,
            save_as: None,
            ignore_errors: false,
            when: None,
            description: None,
        }
    }
}

impl Default for PluginCommandStep {
    fn default() -> Self {
        Self {
            command: String::new(),
            params: serde_json::Value::Null,
            save_as: None,
            ignore_errors: false,
            when: None,
            description: None,
        }
    }
}

/// Sub-action variation declared within a plugin (e.g. Cutout vs Layer Mask)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubAction {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_ru: Option<String>,
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub locales: std::collections::HashMap<String, PluginLocaleInfo>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub post_commands: Vec<PluginCommandStep>,
    #[serde(default)]
    pub script: Option<String>,
}

impl SubAction {
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

    pub fn name_for_lang(&self, lang: crate::Language) -> &str {
        self.localized_name(lang.code())
    }
}

/// Request DTO for executing a plugin action, optionally targeting a specific sub-action variant.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ActionRequest {
    pub action_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sub_action_id: Option<String>,
}

impl ActionRequest {
    pub fn new(action_id: impl Into<String>) -> Self {
        Self {
            action_id: action_id.into(),
            sub_action_id: None,
        }
    }

    pub fn with_sub_action(action_id: impl Into<String>, sub_action_id: impl Into<String>) -> Self {
        Self {
            action_id: action_id.into(),
            sub_action_id: Some(sub_action_id.into()),
        }
    }
}

impl std::fmt::Display for ActionRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref sub) = self.sub_action_id {
            write!(f, "{}#{}", self.action_id, sub)
        } else {
            write!(f, "{}", self.action_id)
        }
    }
}

impl From<String> for ActionRequest {
    fn from(s: String) -> Self {
        if let Some((act, sub)) = s.split_once('#') {
            Self::with_sub_action(act, sub)
        } else {
            Self::new(s)
        }
    }
}

impl From<&str> for ActionRequest {
    fn from(s: &str) -> Self {
        s.to_string().into()
    }
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
    async fn get_document_info(&self) -> Result<DocumentInfo>;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_result() {
        let ok_res = PluginResult::ok("Completed");
        assert!(ok_res.success);
        assert_eq!(ok_res.message, "Completed");
        assert!(ok_res.data.is_none());

        let data = serde_json::json!({ "output": 42 });
        let ok_data_res = PluginResult::ok_with_data("Done", data.clone());
        assert!(ok_data_res.success);
        assert_eq!(ok_data_res.message, "Done");
        assert_eq!(ok_data_res.data, Some(data));

        let err_res = PluginResult::err("Failed");
        assert!(!err_res.success);
        assert_eq!(err_res.message, "Failed");
        assert!(err_res.data.is_none());
    }

    #[test]
    fn test_plugin_type_default() {
        assert_eq!(PluginType::default(), PluginType::NoCode);
    }

    #[test]
    fn test_plugin_manifest_model_installed() {
        let temp_dir = std::env::temp_dir().join(format!(
            "manifest_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let models_dir = temp_dir.join("models");
        std::fs::create_dir_all(&models_dir).unwrap();

        let model_file = models_dir.join("test_model.onnx");
        std::fs::write(&model_file, b"dummy onnx").unwrap();

        let manifest = PluginManifest {
            id: "test.plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Tester".to_string(),
            plugin_type: PluginType::NativeAi,
            icon: None,
            commands: vec![],
            models: vec![
                ModelRequirement {
                    role: Some("primary".to_string()),
                    name: "test_model.onnx".to_string(),
                    required: true,
                    size_mb: Some(1),
                    url: None,
                    sha256: None,
                },
                ModelRequirement {
                    role: Some("secondary".to_string()),
                    name: "optional_model.onnx".to_string(),
                    required: false,
                    size_mb: None,
                    url: None,
                    sha256: None,
                },
            ],
            action_id: None,
            entrypoint: None,
            ai: None,
            enabled: true,
            plugin_dir: Some(temp_dir.clone()),
            ..Default::default()
        };

        assert!(manifest.is_model_installed(&manifest.models[0]));
        assert!(!manifest.is_model_installed(&manifest.models[1]));
        assert_eq!(
            manifest.model_by_role("primary").map(|m| m.name.as_str()),
            Some("test_model.onnx")
        );
        assert_eq!(
            manifest.model_by_role("secondary").map(|m| m.name.as_str()),
            Some("optional_model.onnx")
        );
        assert!(manifest.model_by_role("non_existent").is_none());

        // Because optional_model is declared but missing, all_models_installed is FALSE:
        assert!(!manifest.all_models_installed());
        // But required models are installed:
        assert!(manifest.required_models_installed());

        // Empty models manifest returns true
        let empty_manifest = PluginManifest::default();
        assert!(empty_manifest.all_models_installed());
        assert!(empty_manifest.required_models_installed());

        // Once optional_model is installed, all_models_installed becomes TRUE:
        let opt_file = models_dir.join("optional_model.onnx");
        std::fs::write(&opt_file, b"dummy opt").unwrap();
        assert!(manifest.all_models_installed());

        // Now test flat plugin_dir placement
        let flat_model = temp_dir.join("flat_model.onnx");
        std::fs::write(&flat_model, b"dummy flat").unwrap();
        let flat_req = ModelRequirement {
            role: None,
            name: "flat_model.onnx".to_string(),
            required: true,
            size_mb: None,
            url: None,
            sha256: None,
        };
        assert!(manifest.is_model_installed(&flat_req));

        // Test 0-byte file and .part file
        let zero_byte_model = models_dir.join("zero_byte.onnx");
        std::fs::write(&zero_byte_model, b"").unwrap();
        let zero_req = ModelRequirement {
            role: None,
            name: "zero_byte.onnx".to_string(),
            required: true,
            size_mb: None,
            url: None,
            sha256: None,
        };
        assert!(!manifest.is_model_installed(&zero_req));

        let part_model = models_dir.join("in_progress.onnx.part");
        std::fs::write(&part_model, b"partial data").unwrap();
        let part_req = ModelRequirement {
            role: None,
            name: "in_progress.onnx.part".to_string(),
            required: true,
            size_mb: None,
            url: None,
            sha256: None,
        };
        assert!(!manifest.is_model_installed(&part_req));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_ai_pipeline_config_paradigms_serialization() {
        let json_data = r#"{
            "task": "image_to_image",
            "tile_size": 512,
            "tile_pad": 32,
            "normalize": "zero_to_one"
        }"#;

        let cfg: AiPipelineConfig = serde_json::from_str(json_data).unwrap();
        assert_eq!(cfg.task, AiTask::ImageToImage);
        assert_eq!(cfg.tile_size, Some(512));
        assert_eq!(cfg.tile_pad, Some(32));
        assert_eq!(cfg.normalize, Some(NormalizationMode::ZeroToOne));

        // Test super_resolution
        let json_sr = r#"{"task": "super_resolution", "scale": 4}"#;
        let cfg_sr: AiPipelineConfig = serde_json::from_str(json_sr).unwrap();
        assert_eq!(cfg_sr.task, AiTask::SuperResolution);
        assert_eq!(cfg_sr.scale, Some(4));

        // Test inpainting
        let json_inp = r#"{"task": "inpainting", "normalize": "raw_255"}"#;
        let cfg_inp: AiPipelineConfig = serde_json::from_str(json_inp).unwrap();
        assert_eq!(cfg_inp.task, AiTask::Inpainting);
        assert_eq!(cfg_inp.normalize, Some(NormalizationMode::Raw255));

        // Test depth_map / segmentation_mask with ImageNet & GrayscaleLayer
        let json_depth = r#"{
            "task": "segmentation_mask",
            "input_size": 518,
            "normalize": "image_net",
            "output_type": "grayscale_layer"
        }"#;
        let cfg_depth: AiPipelineConfig = serde_json::from_str(json_depth).unwrap();
        assert_eq!(cfg_depth.task, AiTask::SegmentationMask);
        assert_eq!(cfg_depth.input_size, Some(518));
        assert_eq!(cfg_depth.normalize, Some(NormalizationMode::ImageNet));
        assert_eq!(cfg_depth.output_type, Some(AiOutputType::GrayscaleLayer));
    }

    #[test]
    fn test_plugin_manifest_native_algorithm_deserialization() {
        let manifest_json = r#"{
            "id": "vectorize.tracer",
            "name": "Vectorize Image",
            "version": "1.0.0",
            "description": "Convert raster images to vector SVG",
            "plugin_type": "native_algorithm",
            "algorithm": {
                "task": "vector_trace",
                "mode": "spline",
                "color_mode": "color",
                "filter_speckle": 4,
                "corner_threshold": 60,
                "copy_clipboard": true,
                "export_file": false
            }
        }"#;

        let manifest: PluginManifest = serde_json::from_str(manifest_json).unwrap();
        assert_eq!(manifest.plugin_type, PluginType::NativeAlgorithm);
        assert!(manifest.algorithm.is_some());
        let algo = manifest.algorithm.unwrap();
        assert_eq!(algo.task, "vector_trace");
        assert_eq!(algo.mode.as_deref(), Some("spline"));
        assert_eq!(algo.color_mode.as_deref(), Some("color"));
        assert_eq!(algo.filter_speckle, Some(4));
        assert_eq!(algo.corner_threshold, Some(60));
        assert!(algo.copy_clipboard);
        assert!(!algo.export_file);
    }

    #[test]
    fn test_plugin_cannot_use_another_plugin_model() {
        let base_tmp = std::env::temp_dir().join("test_model_cross_plugin_isolation");
        let _ = std::fs::remove_dir_all(&base_tmp);

        let plugin_a = base_tmp.join("plugin_a");
        let plugin_b = base_tmp.join("plugin_b");
        std::fs::create_dir_all(plugin_a.join("models")).unwrap();
        std::fs::create_dir_all(plugin_b.join("models")).unwrap();

        // Model exists strictly in plugin_a
        let model_a_file = plugin_a.join("models").join("isolated_model.onnx");
        std::fs::write(&model_a_file, b"FAKE_ONNX_DATA").unwrap();

        // Plugin B attempts to access isolated_model.onnx
        let manifest_b = PluginManifest {
            id: "ai.photocraft.plugins.plugin_b".to_string(),
            name: "Plugin B".to_string(),
            version: "1.0.0".to_string(),
            description: "Plugin B description".to_string(),
            author: "Author".to_string(),
            plugin_type: PluginType::NativeAi,
            models: vec![ModelRequirement {
                name: "isolated_model.onnx".to_string(),
                required: true,
                role: Some("custom_role".to_string()),
                size_mb: Some(10),
                sha256: None,
                url: Some("https://example.com/isolated_model.onnx".to_string()),
            }],
            plugin_dir: Some(plugin_b.clone()),
            ..Default::default()
        };

        // Plugin B MUST NOT find model in plugin A's directory!
        let model_req = &manifest_b.models[0];
        assert!(
            !manifest_b.is_model_installed(model_req),
            "Violation: Plugin B was able to see and use Plugin A's model!"
        );
        assert!(
            !manifest_b.all_models_installed(),
            "Violation: Plugin B falsely reported all_models_installed when model belongs to Plugin A!"
        );

        let _ = std::fs::remove_dir_all(base_tmp);
    }

    #[test]
    fn test_plugin_type_interactive_ui() {
        assert_eq!(PluginType::default(), PluginType::NoCode);
        let serialized = serde_json::to_string(&PluginType::InteractiveUi).unwrap();
        assert_eq!(serialized, "\"interactive_ui\"");
        let deserialized: PluginType = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, PluginType::InteractiveUi);
    }

    #[test]
    fn test_banner_style_serialization() {
        let styles = [
            (BannerStyle::Info, "\"info\""),
            (BannerStyle::Warning, "\"warning\""),
            (BannerStyle::Error, "\"error\""),
            (BannerStyle::Success, "\"success\""),
        ];
        for (style, expected_json) in styles {
            let serialized = serde_json::to_string(&style).unwrap();
            assert_eq!(serialized, expected_json);
            let deserialized: BannerStyle = serde_json::from_str(&serialized).unwrap();
            assert_eq!(deserialized, style);
        }
    }

    #[test]
    fn test_interactive_ui_config_and_widgets_roundtrip() {
        let config = InteractiveUiConfig {
            window_title: "Harmonizer Studio".to_string(),
            window_title_ru: Some("Студия гармонии".to_string()),
            default_width: Some(360.0),
            default_height: Some(540.0),
            script: Some("ui.rhai".to_string()),
            ipc_subscriptions: vec!["photocraft.layer.color_picked".to_string()],
            widgets: vec![
                WidgetNode::Banner {
                    id: "status_banner".to_string(),
                    message_binding: "banner_msg".to_string(),
                    style_binding: Some("banner_style".to_string()),
                },
                WidgetNode::ColorSlot {
                    id: "primary_color".to_string(),
                    label: "Base Color".to_string(),
                    label_ru: Some("Основной цвет".to_string()),
                    default_hex: Some("#FF5500".to_string()),
                    read_only: false,
                    selectable: true,
                    target_binding: Some("target_slot".to_string()),
                },
                WidgetNode::HueWheel {
                    id: "color_wheel".to_string(),
                    source_color_id: "primary_color".to_string(),
                    show_pointer: true,
                },
                WidgetNode::Slider {
                    id: "spread".to_string(),
                    label: "Spread Angle".to_string(),
                    label_ru: Some("Угол охвата".to_string()),
                    min: 0.0,
                    max: 180.0,
                    step: Some(1.0),
                    default: 45.0,
                },
                WidgetNode::Dropdown {
                    id: "harmony_rule".to_string(),
                    label: "Harmony".to_string(),
                    label_ru: Some("Гармония".to_string()),
                    options: vec![
                        "Complementary".to_string(),
                        "Analogous".to_string(),
                        "Triadic".to_string(),
                    ],
                    options_ru: Some(vec![
                        "Комплементарная".to_string(),
                        "Аналоговая".to_string(),
                        "Триадическая".to_string(),
                    ]),
                    default_index: 1,
                },
                WidgetNode::Toggle {
                    id: "live_sync".to_string(),
                    label: "Live Sync".to_string(),
                    label_ru: Some("Живая синхронизация".to_string()),
                    default: true,
                },
                WidgetNode::Separator,
                WidgetNode::Label {
                    text_binding: "summary_text".to_string(),
                },
                WidgetNode::Row {
                    items: vec![
                        WidgetNode::ColorSlot {
                            id: "accent_1".to_string(),
                            label: "Harmonic 1".to_string(),
                            label_ru: None,
                            default_hex: None,
                            read_only: true,
                            selectable: false,
                            target_binding: None,
                        },
                        WidgetNode::ColorSlot {
                            id: "accent_2".to_string(),
                            label: "Harmonic 2".to_string(),
                            label_ru: None,
                            default_hex: None,
                            read_only: true,
                            selectable: false,
                            target_binding: None,
                        },
                    ],
                },
            ],
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: InteractiveUiConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_plugin_ipc_action_serialization() {
        let action = PluginIpcAction {
            command: "photocraft.palette.apply".to_string(),
            params: serde_json::json!({
                "colors": ["#FF0000", "#00FF00", "#0000FF"],
                "target_layer_id": 12
            }),
        };

        let json = serde_json::to_string(&action).unwrap();
        let deserialized: PluginIpcAction = serde_json::from_str(&json).unwrap();
        assert_eq!(action, deserialized);

        // Test command without params
        let bare_action_json = r#"{"command": "photocraft.canvas.refresh"}"#;
        let bare_action: PluginIpcAction = serde_json::from_str(bare_action_json).unwrap();
        assert_eq!(bare_action.command, "photocraft.canvas.refresh");
        assert_eq!(bare_action.params, serde_json::Value::Null);
    }

    #[test]
    fn test_plugin_manifest_interactive_ui_deserialization() {
        let manifest_json = r##"{
            "id": "ui.color_harmonizer",
            "name": "Color Harmonizer",
            "version": "1.0.0",
            "description": "Interactive color harmony wheel and palette generator",
            "plugin_type": "interactive_ui",
            "interactive_ui": {
                "window_title": "Color Harmonizer",
                "default_width": 320.0,
                "default_height": 480.0,
                "script": "harmonizer.rhai",
                "widgets": [
                    {
                        "type": "banner",
                        "id": "tip_banner",
                        "message_binding": "tip_text"
                    },
                    {
                        "type": "color_slot",
                        "id": "base_color",
                        "label": "Base Color",
                        "default_hex": "#3498db"
                    },
                    {
                        "type": "hue_wheel",
                        "id": "wheel",
                        "source_color_id": "base_color"
                    },
                    {
                        "type": "slider",
                        "id": "roughness",
                        "label": "Angle",
                        "min": 0.0,
                        "max": 360.0,
                        "default": 180.0
                    },
                    {
                        "type": "dropdown",
                        "id": "mode",
                        "label": "Mode",
                        "options": ["Analogous", "Complementary"]
                    },
                    {
                        "type": "toggle",
                        "id": "auto_update",
                        "label": "Auto Update"
                    },
                    {
                        "type": "separator"
                    },
                    {
                        "type": "label",
                        "text_binding": "status_msg"
                    },
                    {
                        "type": "row",
                        "items": [
                            {
                                "type": "color_slot",
                                "id": "res_1",
                                "label": "Color 1"
                            }
                        ]
                    }
                ],
                "ipc_subscriptions": ["palette.changed"]
            }
        }"##;

        let manifest: PluginManifest = serde_json::from_str(manifest_json).unwrap();
        assert_eq!(manifest.id, "ui.color_harmonizer");
        assert_eq!(manifest.plugin_type, PluginType::InteractiveUi);
        assert!(manifest.interactive_ui.is_some());

        let ui = manifest.interactive_ui.unwrap();
        assert_eq!(ui.window_title, "Color Harmonizer");
        assert_eq!(ui.default_width, Some(320.0));
        assert_eq!(ui.default_height, Some(480.0));
        assert_eq!(ui.script.as_deref(), Some("harmonizer.rhai"));
        assert_eq!(ui.widgets.len(), 9);
        assert_eq!(ui.ipc_subscriptions, vec!["palette.changed"]);

        match &ui.widgets[0] {
            WidgetNode::Banner {
                id,
                message_binding,
                style_binding,
            } => {
                assert_eq!(id, "tip_banner");
                assert_eq!(message_binding, "tip_text");
                assert_eq!(style_binding, &None);
            }
            other => panic!("Expected Banner, got {:?}", other),
        }

        match &ui.widgets[1] {
            WidgetNode::ColorSlot {
                id,
                label,
                default_hex,
                ..
            } => {
                assert_eq!(id, "base_color");
                assert_eq!(label, "Base Color");
                assert_eq!(default_hex.as_deref(), Some("#3498db"));
            }
            other => panic!("Expected ColorSlot, got {:?}", other),
        }

        match &ui.widgets[2] {
            WidgetNode::HueWheel {
                id,
                source_color_id,
                show_pointer,
            } => {
                assert_eq!(id, "wheel");
                assert_eq!(source_color_id, "base_color");
                assert!(show_pointer);
            }
            other => panic!("Expected HueWheel, got {:?}", other),
        }

        match &ui.widgets[3] {
            WidgetNode::Slider {
                id,
                label,
                min,
                max,
                default,
                step,
                ..
            } => {
                assert_eq!(id, "roughness");
                assert_eq!(label, "Angle");
                assert_eq!(*min, 0.0);
                assert_eq!(*max, 360.0);
                assert_eq!(*default, 180.0);
                assert_eq!(step, &None);
            }
            other => panic!("Expected Slider, got {:?}", other),
        }

        match &ui.widgets[4] {
            WidgetNode::Dropdown {
                id,
                label,
                options,
                default_index,
                ..
            } => {
                assert_eq!(id, "mode");
                assert_eq!(label, "Mode");
                assert_eq!(options, &["Analogous", "Complementary"]);
                assert_eq!(*default_index, 0);
            }
            other => panic!("Expected Dropdown, got {:?}", other),
        }

        match &ui.widgets[5] {
            WidgetNode::Toggle {
                id, label, default, ..
            } => {
                assert_eq!(id, "auto_update");
                assert_eq!(label, "Auto Update");
                assert!(!default);
            }
            other => panic!("Expected Toggle, got {:?}", other),
        }

        match &ui.widgets[6] {
            WidgetNode::Separator => {}
            other => panic!("Expected Separator, got {:?}", other),
        }

        match &ui.widgets[7] {
            WidgetNode::Label { text_binding } => {
                assert_eq!(text_binding, "status_msg");
            }
            other => panic!("Expected Label, got {:?}", other),
        }

        match &ui.widgets[8] {
            WidgetNode::Row { items } => {
                assert_eq!(items.len(), 1);
                assert!(matches!(items[0], WidgetNode::ColorSlot { .. }));
            }
            other => panic!("Expected Row, got {:?}", other),
        }
    }

    #[test]
    fn test_sub_action_and_action_request() {
        let sub = SubAction {
            id: "mask".to_string(),
            locales: std::collections::HashMap::new(),
            name: "Layer Mask".to_string(),
            name_ru: Some("Маска слоя".to_string()),
            icon: Some("mask.svg".to_string()),
            post_commands: vec![PluginCommandStep::new("layer.mask")],
            script: None,
        };

        assert_eq!(sub.name_for_lang(crate::Language::En), "Layer Mask");
        assert_eq!(sub.name_for_lang(crate::Language::Ru), "Маска слоя");

        let req1 = ActionRequest::new("plugin:remove_bg");
        assert_eq!(req1.to_string(), "plugin:remove_bg");
        assert_eq!(req1.sub_action_id, None);

        let req2 = ActionRequest::with_sub_action("plugin:remove_bg", "mask");
        assert_eq!(req2.to_string(), "plugin:remove_bg#mask");
        assert_eq!(req2.sub_action_id.as_deref(), Some("mask"));

        let req3 = ActionRequest::from("plugin:remove_bg#mask".to_string());
        assert_eq!(req3, req2);

        let req4 = ActionRequest::from("plugin:remove_bg");
        assert_eq!(req4, req1);
    }
}
