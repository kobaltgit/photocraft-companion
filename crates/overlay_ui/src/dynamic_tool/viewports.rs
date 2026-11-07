use super::{DynamicToolEngine, DynamicValue};
use core_contracts::DynamicBadgeState;
use eframe::egui;
use std::collections::HashMap;

impl DynamicToolEngine {
    pub fn get_active_badges(&self) -> HashMap<String, DynamicBadgeState> {
        let mut map = HashMap::new();
        for (id, state) in &self.states {
            if let Some(ref badge) = state.dynamic_badge {
                map.insert(id.clone(), badge.clone());
            }
        }
        map
    }

    pub fn viewport_id_for_plugin(plugin_id: &str) -> egui::ViewportId {
        egui::ViewportId::from_hash_of(format!("dynamic_plugin_window_{}", plugin_id))
    }

    pub fn is_window_open(&self, plugin_id: &str) -> bool {
        self.states
            .get(plugin_id)
            .map(|s| s.is_open)
            .unwrap_or(false)
    }

    pub fn is_any_window_open(&self) -> bool {
        self.states.values().any(|s| s.is_open)
    }

    pub fn close_all_windows(&mut self) -> Vec<String> {
        let mut closed_ids = Vec::new();
        for (id, state) in self.states.iter_mut() {
            if state.is_open {
                state.is_open = false;
                state
                    .values
                    .insert("is_open".to_string(), DynamicValue::Bool(false));
                closed_ids.push(id.clone());
            }
        }
        for id in &closed_ids {
            self.pending_close_viewports
                .push(Self::viewport_id_for_plugin(id));
            self.execute_script_state_change(id);
        }
        closed_ids
    }

    pub fn first_open_window_with_subscription(&self, topic: &str) -> Option<String> {
        for (id, state) in &self.states {
            if state.is_open {
                if let Some(manifest) = self.manifests.get(id) {
                    if let Some(ui_cfg) = &manifest.interactive_ui {
                        if ui_cfg.ipc_subscriptions.iter().any(|s| s == topic) {
                            return Some(id.clone());
                        }
                    }
                }
            }
        }
        None
    }

    pub fn has_open_windows_with_subscription(&self, topic: &str) -> bool {
        self.first_open_window_with_subscription(topic).is_some()
    }

    pub fn toggle_window(&mut self, plugin_id: &str) {
        let will_open = !self
            .states
            .get(plugin_id)
            .map(|s| s.is_open)
            .unwrap_or(false);
        let mut ids_to_notify = Vec::new();

        if will_open {
            for (other_id, other_state) in self.states.iter_mut() {
                if other_id != plugin_id && other_state.is_open {
                    other_state.is_open = false;
                    other_state
                        .values
                        .insert("is_open".to_string(), DynamicValue::Bool(false));
                    self.pending_close_viewports
                        .push(Self::viewport_id_for_plugin(other_id));
                    ids_to_notify.push(other_id.clone());
                }
            }
        } else {
            self.pending_close_viewports
                .push(Self::viewport_id_for_plugin(plugin_id));
        }

        if let Some(state) = self.states.get_mut(plugin_id) {
            state.is_open = will_open;
            state
                .values
                .insert("is_open".to_string(), DynamicValue::Bool(state.is_open));
            if state.is_open {
                state.icons_calibrated = false;
                self.preview_thumbnails.remove(plugin_id);
                self.preview_textures.remove(plugin_id);
                state.role_mask = None;
                state
                    .values
                    .insert("base_saved".to_string(), DynamicValue::Bool(false));
            }
            ids_to_notify.push(plugin_id.to_string());
        }

        for id in ids_to_notify {
            self.execute_script_state_change(&id);
        }
    }
}
