use serde::{Deserialize, Serialize};

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
