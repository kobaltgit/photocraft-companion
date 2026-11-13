mod flyout;
mod view;

#[cfg(test)]
mod tests;

use crate::theme::{ThemeMode, ThemePalette};
use core_contracts::ButtonConfig;
use eframe::egui::Vec2;

pub const MAX_BUTTONS_PER_TIER: usize = 8;
pub const BUTTON_SLOT_SIZE_V: f32 = 36.0;
pub const BUTTON_SLOT_SIZE_H: f32 = 38.0;

pub struct ToolstripApp {
    pub is_vertical: bool,
    pub current_theme: ThemeMode,
    pub language: core_contracts::Language,
    pub buttons: Vec<ButtonConfig>,
    pub pending_action: Option<String>,
    pub open_manager_requested: bool,
    pub hide_requested: bool,
    pub hovered_tooltip: Option<(String, egui::Rect)>,
    pub is_busy: bool,
    pub active_action: Option<String>,
    pub has_selection: bool,
    pub orientation_changed: bool,
    pub icon_cache: crate::icons::SvgIconCache,
    pub open_sub_actions_for: Option<(String, egui::Rect)>,
    pub pointer_down_start: Option<(String, std::time::Instant)>,
    pub suppress_click_for: Option<String>,
    pub long_press_fired_for: Option<String>,
    pub dynamic_badges: std::collections::HashMap<String, core_contracts::DynamicBadgeState>,
}

impl ToolstripApp {
    pub fn new(buttons: Vec<ButtonConfig>, current_theme: ThemeMode) -> Self {
        Self {
            is_vertical: false, // Per spec: default is horizontal
            current_theme,
            language: core_contracts::Language::Auto,
            buttons,
            pending_action: None,
            open_manager_requested: false,
            hide_requested: false,
            hovered_tooltip: None,
            is_busy: false,
            active_action: None,
            has_selection: false,
            orientation_changed: false,
            icon_cache: crate::icons::SvgIconCache::new(),
            open_sub_actions_for: None,
            pointer_down_start: None,
            suppress_click_for: None,
            long_press_fired_for: None,
            dynamic_badges: std::collections::HashMap::new(),
        }
    }

    pub fn set_dynamic_badge(
        &mut self,
        action_id: &str,
        badge: Option<core_contracts::DynamicBadgeState>,
    ) {
        if let Some(b) = badge {
            self.dynamic_badges.insert(action_id.to_string(), b);
        } else {
            self.dynamic_badges.remove(action_id);
        }
    }

    pub fn with_language(mut self, language: core_contracts::Language) -> Self {
        self.language = language;
        self
    }

    pub fn with_orientation(mut self, is_vertical: bool) -> Self {
        self.is_vertical = is_vertical;
        self
    }

    /// Dynamic sizing:
    /// Fully adaptive calculation based on count N of active plugins.
    /// Supports arbitrary plugin counts by any 3rd party developer without clipping.
    pub fn calculate_size(&self) -> Vec2 {
        let count = self.buttons.len().max(1);
        if self.is_vertical {
            let cols = count.div_ceil(MAX_BUTTONS_PER_TIER).max(1) as f32;
            let max_rows = count.min(MAX_BUTTONS_PER_TIER) as f32;
            let width = 50.0 + (cols - 1.0) * BUTTON_SLOT_SIZE_V;
            let height = max_rows * BUTTON_SLOT_SIZE_V + 68.0;
            Vec2::new(width, height)
        } else {
            let rows = count.div_ceil(MAX_BUTTONS_PER_TIER).max(1) as f32;
            let max_cols = count.min(MAX_BUTTONS_PER_TIER) as f32;
            let width = (max_cols * BUTTON_SLOT_SIZE_H + 16.0).max(116.0);
            let height = 78.0 + (rows - 1.0) * BUTTON_SLOT_SIZE_V;
            Vec2::new(width, height)
        }
    }
}
