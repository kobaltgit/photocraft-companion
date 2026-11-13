use core_contracts::{ButtonConfig, CompanionError, Result};
use std::path::Path;
use tracing::info;

pub struct OverlayBar {
    pub buttons: Vec<ButtonConfig>,
}

impl OverlayBar {
    pub fn new(buttons: Vec<ButtonConfig>) -> Self {
        Self { buttons }
    }

    pub fn from_manifests(manifests: &[core_contracts::PluginManifest]) -> Vec<ButtonConfig> {
        let mut buttons = Vec::new();
        for m in manifests {
            if m.enabled {
                let icon = m.icon.clone().or_else(|| match m.plugin_type {
                    core_contracts::PluginType::NativeAi => Some("wand".to_string()),
                    core_contracts::PluginType::NoCode => Some("bw".to_string()),
                    core_contracts::PluginType::NativeAlgorithm => Some("vector".to_string()),
                    core_contracts::PluginType::InteractiveUi => Some("palette".to_string()),
                    _ => None,
                });
                let icon_path = m.resolve_icon_path(None);
                let active_sub_action = m.sub_actions.first().map(|s| s.id.clone());
                let requires_selection = m.input == core_contracts::PluginInputSource::Selection
                    || m.interactive_ui
                        .as_ref()
                        .map(|ui| {
                            ui.ipc_subscriptions
                                .iter()
                                .any(|s| s == core_contracts::IPC_SUB_SELECTION_STATE)
                        })
                        .unwrap_or(false);
                buttons.push(ButtonConfig {
                    label: m.name.clone(),
                    action: m.id.clone(),
                    icon,
                    icon_path,
                    plugin_dir: m.plugin_dir.clone(),
                    sub_actions: m.sub_actions.clone(),
                    active_sub_action,
                    name_ru: m.name_ru.clone(),
                    locales: m.locales.clone(),
                    requires_selection,
                });
            }
        }
        if buttons.is_empty() {
            Self::default_buttons()
        } else {
            buttons
        }
    }

    pub fn default_buttons() -> Vec<ButtonConfig> {
        vec![
            ButtonConfig {
                label: "AI Remove BG".to_string(),
                action: "ai.photocraft.plugins.remove_bg".to_string(),
                icon: Some("wand".to_string()),
                icon_path: None,
                plugin_dir: None,
                sub_actions: Vec::new(),
                active_sub_action: None,
                name_ru: None,
                locales: std::collections::HashMap::new(),
                requires_selection: false,
            },
            ButtonConfig {
                label: "B&W Film Pop".to_string(),
                action: "macro.photocraft.plugins.black_and_white".to_string(),
                icon: Some("bw".to_string()),
                icon_path: None,
                plugin_dir: None,
                sub_actions: Vec::new(),
                active_sub_action: None,
                name_ru: None,
                locales: std::collections::HashMap::new(),
                requires_selection: false,
            },
        ]
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(CompanionError::Io)?;
        let buttons: Vec<ButtonConfig> =
            serde_json::from_str(&content).map_err(CompanionError::Json)?;
        info!(
            "Loaded {} overlay buttons from {}",
            buttons.len(),
            path.as_ref().display()
        );
        Ok(Self { buttons })
    }

    pub fn load_or_default(path: impl AsRef<Path>) -> Self {
        if path.as_ref().exists() {
            if let Ok(bar) = Self::load_from_file(path.as_ref()) {
                return bar;
            }
        }
        Self::new(Self::default_buttons())
    }
}
