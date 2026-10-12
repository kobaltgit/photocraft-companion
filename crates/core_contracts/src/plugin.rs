use crate::{DocumentInfo, Result};
use async_trait::async_trait;
use image::RgbaImage;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

/// Manifest definition for any plugin (No-Code, AI, Script, or External)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
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
}

impl PluginManifest {
    pub fn is_model_installed(&self, model: &ModelRequirement) -> bool {
        if let Some(ref dir) = self.plugin_dir {
            let direct = dir.join("models").join(&model.name);
            if direct.exists() {
                return true;
            }
            let flat = dir.join(&model.name);
            if flat.exists() {
                return true;
            }
        }
        // Fallback global check in models/
        let global = PathBuf::from("models").join(&model.name);
        if global.exists() {
            return true;
        }
        let fallback = PathBuf::from(r"D:\Portable Soft\ArtCraft\photocraft-0.6.0-windows-x64-portable\PhotoCraftData\models").join(&model.name);
        fallback.exists()
    }

    pub fn all_models_installed(&self) -> bool {
        self.models
            .iter()
            .filter(|m| m.required)
            .all(|m| self.is_model_installed(m))
    }
}

impl Default for PluginManifest {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            version: "1.0.0".to_string(),
            description: String::new(),
            author: default_author(),
            plugin_type: PluginType::default(),
            icon: None,
            commands: Vec::new(),
            post_commands: Vec::new(),
            input: PluginInputSource::default(),
            output_mode: PluginOutputMode::default(),
            models: Vec::new(),
            action_id: None,
            entrypoint: None,
            ai: None,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PluginType {
    /// Declarative macro running PhotoCraft engine commands sequentially
    #[default]
    NoCode,
    /// Native Rust/C++ AI inference or high-perf memory processing
    NativeAi,
    /// Script-based automation (e.g. Rhai / Lua / JS)
    Script,
    /// External CLI process communicating via stdin/stdout or clipboard
    ExternalCli,
}

/// A step in a declarative No-Code pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommandStep {
    pub command: String,
    #[serde(default)]
    pub params: serde_json::Value,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequirement {
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
                    name: "test_model.onnx".to_string(),
                    required: true,
                    size_mb: Some(1),
                    url: None,
                    sha256: None,
                },
                ModelRequirement {
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
        // Since optional_model is not required, all required models are installed:
        assert!(manifest.all_models_installed());

        // Now test flat plugin_dir placement
        let flat_model = temp_dir.join("flat_model.onnx");
        std::fs::write(&flat_model, b"dummy flat").unwrap();
        let flat_req = ModelRequirement {
            name: "flat_model.onnx".to_string(),
            required: true,
            size_mb: None,
            url: None,
            sha256: None,
        };
        assert!(manifest.is_model_installed(&flat_req));

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
    }
}
