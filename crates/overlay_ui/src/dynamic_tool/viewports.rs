use super::{DynamicToolEngine, DynamicValue, DynamicWindowState};
use core_contracts::{DynamicBadgeState, WidgetNode};
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

    pub fn set_dynamic_badge(&mut self, plugin_id: &str, badge: Option<DynamicBadgeState>) {
        if let Some(state) = self.states.get_mut(plugin_id) {
            state.dynamic_badge = badge;
        }
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

    pub fn has_active_subscription(&self, topic: &str) -> bool {
        if self.has_open_windows_with_subscription(topic) {
            return true;
        }
        for (id, state) in &self.states {
            if state.dynamic_badge.is_some() {
                if let Some(m) = self.manifests.get(id) {
                    if let Some(ref ui) = m.interactive_ui {
                        if ui.ipc_subscriptions.iter().any(|s| s == topic) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn toggle_window(&mut self, plugin_id: &str) {
        if let Some(state) = self.states.get_mut(plugin_id) {
            state.is_open = !state.is_open;
            state
                .values
                .insert("is_open".to_string(), DynamicValue::Bool(state.is_open));
            if state.is_open {
                state.just_opened = true;
                state.icons_calibrated = false;
                self.preview_thumbnails.remove(plugin_id);
                self.preview_textures.remove(plugin_id);
                state.role_mask = None;
                state
                    .values
                    .insert("base_saved".to_string(), DynamicValue::Bool(false));

                // Pre-populate API key from CompanionConfig deterministically via api_key_service
                let service_opt = self
                    .manifests
                    .get(plugin_id)
                    .and_then(|m| m.interactive_ui.as_ref())
                    .and_then(|u| u.api_key_service.as_deref());

                if let Some(service) = service_opt {
                    if let Some(ref cfg_path) = self.config_path {
                        let cfg = core_contracts::CompanionConfig::load_or_default(cfg_path);
                        if let Some(key_val) = cfg.api_keys.get(service) {
                            if !key_val.is_empty() {
                                state.values.insert(
                                    "api_key".to_string(),
                                    DynamicValue::String(key_val.clone()),
                                );
                            }
                        }
                    }
                } else if let Some(ref cfg_path) = self.config_path {
                    let cfg = core_contracts::CompanionConfig::load_or_default(cfg_path);
                    for key_val in cfg.api_keys.values() {
                        if !key_val.is_empty() {
                            let curr = state
                                .values
                                .get("api_key")
                                .and_then(|v| match v {
                                    DynamicValue::String(s) => Some(s.as_str()),
                                    _ => None,
                                })
                                .unwrap_or("");
                            if curr.is_empty() {
                                state.values.insert(
                                    "api_key".to_string(),
                                    DynamicValue::String(key_val.clone()),
                                );
                            }
                        }
                    }
                }
            }
            self.execute_script_state_change(plugin_id);
        }
    }

    pub fn save_plugin_geometry(&self, plugin_id: &str, pos: [f32; 2], size: [f32; 2]) {
        if let Some(ref config_path) = self.config_path {
            let mut cfg = core_contracts::CompanionConfig::load_or_default(config_path);
            cfg.set_plugin_geometry(plugin_id, pos, size);
            let _ = cfg.save(config_path);
        }
    }

    pub fn clear_plugin_geometries(&mut self) {
        for state in self.states.values_mut() {
            state.last_pos = None;
            state.last_size = None;
        }
        if let Some(ref config_path) = self.config_path {
            let mut cfg = core_contracts::CompanionConfig::load_or_default(config_path);
            cfg.clear_plugin_geometries();
            let _ = cfg.save(config_path);
        }
    }

    pub fn get_state(&self, plugin_id: &str) -> Option<&DynamicWindowState> {
        self.states.get(plugin_id)
    }

    pub fn set_widget_value(&mut self, plugin_id: &str, key: &str, value: DynamicValue) {
        if let Some(state) = self.states.get_mut(plugin_id) {
            state.values.insert(key.to_string(), value);
        }
    }

    pub fn get_widget_value(&self, plugin_id: &str, key: &str) -> Option<&DynamicValue> {
        self.states.get(plugin_id).and_then(|s| s.values.get(key))
    }

    pub fn set_busy(&mut self, plugin_id: &str, busy: bool) {
        if let Some(state) = self.states.get_mut(plugin_id) {
            state
                .values
                .insert("is_busy".to_string(), DynamicValue::Bool(busy));
        }
    }

    pub fn is_busy(&self, plugin_id: &str) -> bool {
        self.states
            .get(plugin_id)
            .and_then(|s| s.values.get("is_busy"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    pub fn clear_role_mask(&mut self, plugin_id: &str) {
        if let Some(state) = self.states.get_mut(plugin_id) {
            state.role_mask = None;
        }
        self.preview_textures.remove(plugin_id);
    }

    pub fn clear_preview_and_role_mask(&mut self, plugin_id: &str) {
        self.preview_thumbnails.remove(plugin_id);
        self.preview_textures.remove(plugin_id);
        self.clear_role_mask(plugin_id);
    }

    pub fn get_role_mask(&self, plugin_id: &str) -> Option<&image::GrayImage> {
        self.states
            .get(plugin_id)
            .and_then(|s| s.role_mask.as_ref())
    }

    pub fn apply_saved_modifier_stack_orders(&mut self, config: &core_contracts::CompanionConfig) {
        for (plugin_id, manifest) in &self.manifests {
            if let Some(ref ui_cfg) = manifest.interactive_ui {
                if let Some(state) = self.states.get_mut(plugin_id) {
                    for widget in &ui_cfg.widgets {
                        Self::apply_saved_order_to_widget(plugin_id, widget, state, config);
                    }
                }
            }
        }
    }

    fn apply_saved_order_to_widget(
        plugin_id: &str,
        widget: &WidgetNode,
        state: &mut DynamicWindowState,
        config: &core_contracts::CompanionConfig,
    ) {
        match widget {
            WidgetNode::ModifierStack { id, items, .. } => {
                let qualified_key = format!("{}:{}", plugin_id, id);
                if let Some(saved) = config
                    .modifier_stack_orders
                    .get(&qualified_key)
                    .or_else(|| config.modifier_stack_orders.get(id))
                {
                    let mut order: Vec<String> = Vec::new();
                    for item_id in saved {
                        if items.iter().any(|it| &it.id == item_id) && !order.contains(item_id) {
                            order.push(item_id.clone());
                        }
                    }
                    for it in items {
                        if !order.contains(&it.id) {
                            order.push(it.id.clone());
                        }
                    }
                    state.values.insert(
                        format!("{}_order", id),
                        DynamicValue::StringList(order.clone()),
                    );
                    state
                        .values
                        .insert(id.clone(), DynamicValue::StringList(order));
                }
            }
            WidgetNode::Row { items } => {
                for item in items {
                    Self::apply_saved_order_to_widget(plugin_id, item, state, config);
                }
            }
            WidgetNode::Tabs { tabs, .. } => {
                for tab in tabs {
                    for item in &tab.widgets {
                        Self::apply_saved_order_to_widget(plugin_id, item, state, config);
                    }
                }
            }
            _ => {}
        }
    }
}
