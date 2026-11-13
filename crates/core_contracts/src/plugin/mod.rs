pub mod action;
pub mod manifest;
pub mod pipeline;
pub mod widget;

#[cfg(test)]
mod tests;

pub use action::{ActionRequest, AlgorithmConfig, PluginCommandStep, PluginIpcAction, SubAction};
pub use manifest::{
    BridgeOperations, ExecutionState, ModelRequirement, Plugin, PluginContext, PluginInputSource,
    PluginLocaleInfo, PluginManifest, PluginOutputMode, PluginResult, PluginType,
    RuntimeToolRequirement, SharedExecutionStatus,
};
pub use pipeline::{AiOutputType, AiPipelineConfig, AiTask, NormalizationMode};
pub use widget::{
    BannerStyle, DynamicBadgeState, InteractiveUiConfig, ModifierItemConfig, TabItem, WidgetNode,
    IPC_SUB_ACTIVE_LAYER_IMAGE, IPC_SUB_DOCUMENT_REVISION, IPC_SUB_SELECTION_STATE,
    IPC_SUB_SESSION_COLORS,
};
