use core_contracts::Language;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    General,
    Plugins,
    Store,
    About,
}

#[derive(Debug, Clone)]
pub struct ModelDownloadProgress {
    pub model_name: String,
    pub plugin_id: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bytes_sec: f32,
}

#[derive(Debug, Clone)]
pub enum ModelDownloadEvent {
    Progress(ModelDownloadProgress),
    Finished { model_name: String },
    Error { model_name: String, error: String },
}

#[derive(Debug, Clone)]
pub struct PluginReadmeState {
    pub plugin_name: String,
    pub plugin_dir: PathBuf,
    pub raw_content: String,
    pub active_lang: Language,
}
