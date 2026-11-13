use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

fn default_max_distance() -> f64 {
    100.0
}

fn default_preview_height() -> f64 {
    220.0
}

#[inline]
fn is_russian_language(lang: crate::Language) -> bool {
    match lang {
        crate::Language::Ru => true,
        crate::Language::Auto => matches!(crate::Language::Auto.resolve(None), crate::Language::Ru),
        _ => false,
    }
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label_binding: Option<String>,
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        enabled_binding: Option<String>,
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        url: Option<String>,
    },
    Segmented {
        id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label_ru: Option<String>,
        options: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        options_ru: Option<Vec<String>>,
        #[serde(default)]
        default_index: usize,
    },
    HueWheel {
        id: String,
        source_color_id: String,
        #[serde(default = "default_true")]
        show_pointer: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        markers_binding: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        connect_mode_binding: Option<String>,
    },
    Banner {
        id: String,
        message_binding: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        style_binding: Option<String>,
    },
    VectorPad {
        id: String,
        label: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label_ru: Option<String>,
        #[serde(default = "default_max_distance")]
        max_distance: f64,
        #[serde(default)]
        default_angle_deg: f64,
        #[serde(default)]
        default_distance: f64,
    },
    ImagePreview {
        id: String,
        #[serde(default = "default_preview_height")]
        height: f64,
    },
    Label {
        text_binding: String,
    },
    Separator,
    Row {
        #[serde(default)]
        items: Vec<WidgetNode>,
    },
    Tabs {
        id: String,
        tabs: Vec<TabItem>,
        #[serde(default)]
        default_index: usize,
    },
    ModifierStack {
        id: String,
        label: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label_ru: Option<String>,
        items: Vec<ModifierItemConfig>,
        #[serde(default)]
        allow_reordering: bool,
    },
    TextInput {
        id: String,
        label: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label_ru: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        placeholder_ru: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        default: Option<String>,
        #[serde(default)]
        is_password: bool,
        #[serde(default)]
        multiline: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        action_button_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        action_button_label: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        action_button_label_ru: Option<String>,
    },
    ProgressBar {
        id: String,
        label: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        label_ru: Option<String>,
        #[serde(default)]
        show_percentage: bool,
    },
}

/// Configuration for an individual tab in a Tabs widget
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabItem {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_ru: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default)]
    pub widgets: Vec<WidgetNode>,
}

impl TabItem {
    pub fn title_for_lang(&self, lang: crate::Language) -> &str {
        if is_russian_language(lang) {
            self.title_ru.as_deref().unwrap_or(&self.title)
        } else {
            &self.title
        }
    }
}

/// Configuration for an individual item in a ModifierStack widget
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModifierItemConfig {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label_ru: Option<String>,
    pub min: f64,
    pub max: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    pub default: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    #[serde(default)]
    pub enabled: bool,
}

impl ModifierItemConfig {
    pub fn label_for_lang(&self, lang: crate::Language) -> &str {
        if is_russian_language(lang) {
            self.label_ru.as_deref().unwrap_or(&self.label)
        } else {
            &self.label
        }
    }
}

impl WidgetNode {
    pub fn label_for_lang(&self, lang: crate::Language) -> &str {
        let is_ru = is_russian_language(lang);
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
            }
            | WidgetNode::VectorPad {
                label, label_ru, ..
            }
            | WidgetNode::ModifierStack {
                label, label_ru, ..
            }
            | WidgetNode::TextInput {
                label, label_ru, ..
            }
            | WidgetNode::ProgressBar {
                label, label_ru, ..
            } => {
                if is_ru {
                    label_ru.as_deref().unwrap_or(label)
                } else {
                    label
                }
            }
            WidgetNode::Segmented {
                label, label_ru, ..
            } => {
                if is_ru {
                    label_ru.as_deref().or(label.as_deref()).unwrap_or("")
                } else {
                    label.as_deref().unwrap_or("")
                }
            }
            _ => "",
        }
    }

    pub fn options_for_lang(&self, lang: crate::Language) -> &[String] {
        let is_ru = is_russian_language(lang);
        match self {
            WidgetNode::Dropdown {
                options,
                options_ru,
                ..
            }
            | WidgetNode::Segmented {
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

    pub fn placeholder_for_lang(&self, lang: crate::Language) -> &str {
        let is_ru = is_russian_language(lang);
        match self {
            WidgetNode::TextInput {
                placeholder,
                placeholder_ru,
                ..
            } => {
                if is_ru {
                    placeholder_ru
                        .as_deref()
                        .or(placeholder.as_deref())
                        .unwrap_or("")
                } else {
                    placeholder.as_deref().unwrap_or("")
                }
            }
            _ => "",
        }
    }

    pub fn action_button_label_for_lang(&self, lang: crate::Language) -> Option<&str> {
        let is_ru = is_russian_language(lang);
        match self {
            WidgetNode::TextInput {
                action_button_label,
                action_button_label_ru,
                ..
            } => {
                if is_ru {
                    action_button_label_ru
                        .as_deref()
                        .or(action_button_label.as_deref())
                } else {
                    action_button_label.as_deref()
                }
            }
            _ => None,
        }
    }
}

/// Dynamic pulsing badge state for plugin buttons in companion UI
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct DynamicBadgeState {
    pub is_pulsing: bool,
    #[serde(default)]
    pub pulse_color_rgba: [u8; 4],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_run_model_role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dynamic_badge: Option<DynamicBadgeState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key_service: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub widgets: Vec<WidgetNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ipc_subscriptions: Vec<String>,
}

/// Standard IPC subscription topic for listening to active session colors in PhotoCraft
pub const IPC_SUB_SESSION_COLORS: &str = "session.colors";

/// Standard IPC subscription topic for requesting active layer image updates in PhotoCraft
pub const IPC_SUB_ACTIVE_LAYER_IMAGE: &str = "active_layer.image";

/// Standard IPC subscription topic for listening to selection state in PhotoCraft
pub const IPC_SUB_SELECTION_STATE: &str = "document.selection";

/// Standard IPC subscription topic for listening to document revision changes in PhotoCraft
pub const IPC_SUB_DOCUMENT_REVISION: &str = "document.revision";

impl InteractiveUiConfig {
    pub fn window_title_for_lang(&self, lang: crate::Language) -> &str {
        if is_russian_language(lang) {
            self.window_title_ru
                .as_deref()
                .unwrap_or(&self.window_title)
        } else {
            &self.window_title
        }
    }
}
