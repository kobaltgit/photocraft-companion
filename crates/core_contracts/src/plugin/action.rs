use crate::plugin::PluginLocaleInfo;
use serde::{Deserialize, Serialize};

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

fn default_true() -> bool {
    true
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
