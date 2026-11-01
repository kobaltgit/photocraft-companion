pub mod named_colors;
pub mod rhai_bridge;
pub mod state;
pub mod widgets;

#[cfg(test)]
mod tests;

pub use named_colors::{NamedColorEntry, NAMED_COLORS};
pub use state::{DynamicValue, DynamicWindowState};
pub use widgets::hue_wheel::{
    calculate_wcag_contrast, find_nearest_named_color, parse_hex_color, rgb_to_hue_degrees,
    simulate_cvd_color, simulate_cvd_hex,
};
pub use widgets::preview::{extract_extrude_params, hash_extrude_params};

use crate::theme::ThemePalette;
use core_contracts::{BannerStyle, Language, PluginIpcAction, PluginManifest, WidgetNode};
use eframe::egui::{self, Vec2};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, warn};

/// Universal Dynamic Tool Engine managing dynamic windows and Rhai runtime
pub struct DynamicToolEngine {
    manifests: HashMap<String, PluginManifest>,
    states: HashMap<String, DynamicWindowState>,
    pub rhai_engine: rhai::Engine,
    pub rhai_asts: HashMap<String, rhai::AST>,
    #[allow(dead_code)]
    action_tx: Option<UnboundedSender<PluginIpcAction>>,
    current_language: Language,
    lang_arc: Arc<RwLock<Language>>,
    echo_guard: Arc<std::sync::Mutex<Option<(String, String)>>>,
    pub preview_thumbnails: HashMap<String, (image::RgbaImage, f32)>,
    pub preview_textures: HashMap<String, (egui::TextureHandle, u64)>,
    pub has_selection: bool,
    pub config_path: Option<std::path::PathBuf>,
    pub open_manager_requested: bool,
}

impl DynamicToolEngine {
    pub fn new(
        manifests: &[PluginManifest],
        action_tx: Option<UnboundedSender<PluginIpcAction>>,
    ) -> Self {
        Self::new_with_language(manifests, action_tx, Language::En)
    }

    pub fn new_with_language(
        manifests: &[PluginManifest],
        action_tx: Option<UnboundedSender<PluginIpcAction>>,
        language: Language,
    ) -> Self {
        let mut manifest_map = HashMap::new();
        let mut states = HashMap::new();
        let mut rhai_asts = HashMap::new();

        let lang_arc = Arc::new(RwLock::new(language));
        let echo_guard: Arc<std::sync::Mutex<Option<(String, String)>>> =
            Arc::new(std::sync::Mutex::new(None));

        let rhai_engine = rhai_bridge::create_rhai_engine(
            action_tx.clone(),
            lang_arc.clone(),
            echo_guard.clone(),
        );

        for m in manifests {
            if let Some(ref ui_cfg) = m.interactive_ui {
                manifest_map.insert(m.id.clone(), m.clone());
                states.insert(
                    m.id.clone(),
                    DynamicWindowState::new_with_lang(ui_cfg, language),
                );

                if let Some(ref script_file) = ui_cfg.script {
                    if let Some(ref dir) = m.plugin_dir {
                        let script_path = dir.join(script_file);
                        if script_path.exists() {
                            match rhai_engine.compile_file(script_path.clone()) {
                                Ok(ast) => {
                                    debug!(
                                        "Successfully compiled Rhai script for plugin '{}': {}",
                                        m.id,
                                        script_path.display()
                                    );
                                    rhai_asts.insert(m.id.clone(), ast);
                                }
                                Err(e) => {
                                    warn!(
                                        "Failed to compile Rhai script '{}' for plugin '{}': {}",
                                        script_path.display(),
                                        m.id,
                                        e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut engine = Self {
            manifests: manifest_map,
            states,
            rhai_engine,
            rhai_asts,
            action_tx,
            current_language: language,
            lang_arc,
            echo_guard,
            preview_thumbnails: HashMap::new(),
            preview_textures: HashMap::new(),
            has_selection: false,
            config_path: None,
            open_manager_requested: false,
        };

        engine.run_all_initial_scripts();
        engine
    }

    pub fn set_config_path(&mut self, path: std::path::PathBuf) {
        let cfg = core_contracts::CompanionConfig::load_or_default(&path);
        self.apply_saved_modifier_stack_orders(&cfg);
        self.config_path = Some(path);
    }

    pub fn with_config_path(mut self, path: std::path::PathBuf) -> Self {
        self.set_config_path(path);
        self
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

    pub fn current_language(&self) -> Language {
        self.current_language
    }

    pub fn set_language(&mut self, lang: Language) {
        if self.current_language != lang {
            self.current_language = lang;
            if let Ok(mut l) = self.lang_arc.write() {
                *l = lang;
            }
            for (id, manifest) in &self.manifests {
                if let Some(ref ui_cfg) = manifest.interactive_ui {
                    if let Some(state) = self.states.get_mut(id) {
                        state.title = ui_cfg.window_title_for_lang(lang).to_string();
                    }
                }
            }
            self.run_all_initial_scripts();
        }
    }

    fn run_all_initial_scripts(&mut self) {
        let plugin_ids: Vec<String> = self.rhai_asts.keys().cloned().collect();
        for id in plugin_ids {
            self.execute_script_state_change(&id);
        }
    }

    pub fn has_plugin(&self, plugin_id: &str) -> bool {
        self.states.contains_key(plugin_id)
    }

    pub fn set_preview_image(&mut self, plugin_id: &str, img: image::RgbaImage, scale: f32) {
        self.preview_thumbnails
            .insert(plugin_id.to_string(), (img, scale));
        self.preview_textures.remove(plugin_id);
    }

    pub fn get_preview_image(&self, plugin_id: &str) -> Option<&(image::RgbaImage, f32)> {
        self.preview_thumbnails.get(plugin_id)
    }

    pub fn set_role_mask(&mut self, plugin_id: &str, mask: image::GrayImage) {
        if let Some(state) = self.states.get_mut(plugin_id) {
            state.role_mask = Some(mask);
        }
        self.preview_textures.remove(plugin_id);
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

    pub fn toggle_window(&mut self, plugin_id: &str) {
        if let Some(state) = self.states.get_mut(plugin_id) {
            state.is_open = !state.is_open;
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
            self.execute_script_state_change(plugin_id);
        }
    }

    pub fn is_window_open(&self, plugin_id: &str) -> bool {
        self.states
            .get(plugin_id)
            .map(|s| s.is_open)
            .unwrap_or(false)
    }

    pub fn has_open_windows_with_subscription(&self, topic: &str) -> bool {
        for (id, state) in &self.states {
            if state.is_open {
                if let Some(manifest) = self.manifests.get(id) {
                    if let Some(ui_cfg) = &manifest.interactive_ui {
                        if ui_cfg.ipc_subscriptions.iter().any(|s| s == topic) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn dispatch_session_colors(&mut self, fg: &str, bg: &str) -> bool {
        let fg_lower = fg.to_lowercase();
        let bg_lower = bg.to_lowercase();

        {
            if let Ok(mut guard) = self.echo_guard.lock() {
                if let Some((last_fg, last_bg)) = guard.as_ref() {
                    if last_fg == &fg_lower && last_bg == &bg_lower {
                        return false;
                    }
                }
                *guard = Some((fg_lower.clone(), bg_lower.clone()));
            }
        }

        let mut any_updated = false;
        for (id, state) in &mut self.states {
            if !state.is_open {
                continue;
            }
            let is_subscribed = self
                .manifests
                .get(id)
                .and_then(|m| m.interactive_ui.as_ref())
                .map(|u| {
                    u.ipc_subscriptions
                        .iter()
                        .any(|s| s == core_contracts::IPC_SUB_SESSION_COLORS)
                })
                .unwrap_or(false);

            if is_subscribed {
                if let Some(ast) = self.rhai_asts.get(id) {
                    let rhai_state = state.to_rhai_map();
                    let res: std::result::Result<rhai::Map, _> = self.rhai_engine.call_fn(
                        &mut rhai::Scope::new(),
                        ast,
                        "on_session_colors",
                        (fg_lower.clone(), bg_lower.clone(), rhai_state),
                    );
                    if let Ok(updated_state) = res {
                        state.update_from_rhai_map(updated_state);
                        any_updated = true;
                    }
                }
            }
        }
        any_updated
    }

    pub fn dispatch_selection_state(
        &mut self,
        has_selection: bool,
        selection_bounds: Option<[i32; 4]>,
    ) -> bool {
        let mut any_updated = false;
        for (id, state) in &mut self.states {
            if !state.is_open {
                continue;
            }
            let is_subscribed = self
                .manifests
                .get(id)
                .and_then(|m| m.interactive_ui.as_ref())
                .map(|u| {
                    u.ipc_subscriptions
                        .iter()
                        .any(|s| s == core_contracts::IPC_SUB_SELECTION_STATE)
                })
                .unwrap_or(false);

            if is_subscribed {
                let prev_has = state.values.get("has_selection").and_then(|v| v.as_bool());
                let prev_bounds = state
                    .values
                    .get("selection_bounds")
                    .and_then(|v| v.as_int_list());
                let new_bounds_vec: Option<Vec<i64>> = selection_bounds
                    .map(|b| vec![b[0] as i64, b[1] as i64, b[2] as i64, b[3] as i64]);

                let has_changed = prev_has != Some(has_selection);
                let bounds_changed = match (&prev_bounds, &new_bounds_vec) {
                    (Some(pb), Some(nb)) => *pb != nb.as_slice(),
                    (None, None) => false,
                    _ => true,
                };

                if has_changed || bounds_changed {
                    state.values.insert(
                        "has_selection".to_string(),
                        DynamicValue::Bool(has_selection),
                    );
                    if let Some(b) = new_bounds_vec {
                        state
                            .values
                            .insert("selection_bounds".to_string(), DynamicValue::IntList(b));
                    } else {
                        state.values.remove("selection_bounds");
                    }
                    any_updated = true;

                    if let Some(ast) = self.rhai_asts.get(id) {
                        let rhai_state = state.to_rhai_map();
                        let bounds_dynamic: rhai::Dynamic = match selection_bounds {
                            Some(b) => {
                                let arr: rhai::Array = b
                                    .iter()
                                    .map(|&n| rhai::Dynamic::from(n as rhai::INT))
                                    .collect();
                                rhai::Dynamic::from_array(arr)
                            }
                            None => rhai::Dynamic::UNIT,
                        };

                        let res: std::result::Result<rhai::Map, _> = self.rhai_engine.call_fn(
                            &mut rhai::Scope::new(),
                            ast,
                            "on_selection_state",
                            (has_selection, bounds_dynamic, rhai_state.clone()),
                        );

                        let final_res = match res {
                            Ok(updated) => Ok(updated),
                            Err(_) => self.rhai_engine.call_fn(
                                &mut rhai::Scope::new(),
                                ast,
                                "on_selection_state",
                                (has_selection, rhai_state),
                            ),
                        };

                        if let Ok(updated_state) = final_res {
                            state.update_from_rhai_map(updated_state);
                        }
                    }
                }
            }
        }
        any_updated
    }

    pub fn viewport_id_for_plugin(plugin_id: &str) -> egui::ViewportId {
        egui::ViewportId::from_hash_of(format!("dynamic_plugin_window_{}", plugin_id))
    }

    fn execute_script_state_change(&mut self, plugin_id: &str) {
        if let Some(ast) = self.rhai_asts.get(plugin_id) {
            if let Some(state) = self.states.get_mut(plugin_id) {
                rhai_bridge::execute_script_state_change(&self.rhai_engine, ast, state);
            }
        } else if let Some(state) = self.states.get_mut(plugin_id) {
            let action_keys: Vec<String> = state
                .values
                .iter()
                .filter_map(|(k, v)| {
                    if k.starts_with("action_") && v.as_bool() == Some(true) {
                        Some(k.clone())
                    } else {
                        None
                    }
                })
                .collect();
            for key in action_keys {
                state.values.insert(key, DynamicValue::Bool(false));
            }
        }
    }

    pub fn render(&mut self, ctx: &egui::Context, palette: &ThemePalette) {
        let plugin_ids: Vec<String> = self
            .states
            .iter()
            .filter(|(_, s)| s.is_open)
            .map(|(id, _)| id.clone())
            .collect();

        for plugin_id in plugin_ids {
            let manifest = match self.manifests.get(&plugin_id) {
                Some(m) => m.clone(),
                None => continue,
            };
            let config = match manifest.interactive_ui {
                Some(ref cfg) => cfg.clone(),
                None => continue,
            };

            let mut is_open = true;
            let mut state_changed = false;

            let (win_title, default_w, default_h) = {
                let state = match self.states.get(&plugin_id) {
                    Some(s) => s,
                    None => continue,
                };
                (
                    state.title.clone(),
                    state.default_width,
                    state.default_height,
                )
            };

            let viewport_id = Self::viewport_id_for_plugin(&plugin_id);

            let is_split = config.layout_mode.as_deref() == Some("split_preview_right");
            let min_w = if is_split { 460.0 } else { 260.0 };
            let min_h = if is_split { 220.0 } else { 180.0 };

            let mut vp = egui::ViewportBuilder::default()
                .with_title(&win_title)
                .with_inner_size([default_w, default_h])
                .with_min_inner_size([min_w, min_h])
                .with_resizable(true)
                .with_always_on_top();

            let win_icon_sz = crate::icons::get_system_window_icon_size();
            if let Some(icon) = crate::icons::get_egui_icon_data(win_icon_sz) {
                vp = vp.with_icon(icon);
            }

            ctx.show_viewport_immediate(viewport_id, vp, |sub_ctx, _class| {
                if sub_ctx.input(|i| i.viewport().close_requested()) {
                    is_open = false;
                }

                sub_ctx.set_visuals(palette.to_egui_visuals());

                egui::CentralPanel::default()
                    .frame(
                        egui::Frame::new()
                            .fill(palette.bg_fill)
                            .inner_margin(egui::Margin::symmetric(14, 12)),
                    )
                    .show(sub_ctx, |ui| {
                        let is_split = config.layout_mode.as_deref() == Some("split_preview_right");
                        let preview_widget = config
                            .widgets
                            .iter()
                            .find(|w| matches!(w, WidgetNode::ImagePreview { .. }));

                        let models_missing =
                            !manifest.models.is_empty() && !manifest.all_models_installed();

                        if let Some(state) = self.states.get_mut(&plugin_id) {
                            if !state.icons_calibrated
                                && crate::win32_window::apply_crisp_window_icons_by_title(
                                    &win_title,
                                )
                            {
                                state.icons_calibrated = true;
                            }
                            state.values.insert(
                                "model_missing".to_string(),
                                DynamicValue::Bool(models_missing),
                            );
                        }

                        if is_split {
                            ui.horizontal_top(|ui| {
                                ui.allocate_ui_with_layout(
                                    Vec2::new(310.0, ui.available_height()),
                                    egui::Layout::top_down(egui::Align::LEFT),
                                    |left_ui| {
                                        egui::ScrollArea::vertical()
                                            .auto_shrink([false, false])
                                            .show(left_ui, |ui| {
                                                ui.add_space(2.0);
                                                ui.spacing_mut().item_spacing = Vec2::new(6.0, 8.0);

                                                if models_missing {
                                                    let warn_text = match self.current_language {
                                                        Language::Ru => {
                                                            "Модели нейросети не установлены"
                                                        }
                                                        _ => "AI models are not installed",
                                                    };
                                                    widgets::draw_vector_banner(
                                                        ui,
                                                        warn_text,
                                                        BannerStyle::Warning,
                                                        palette,
                                                    );
                                                    let btn_text = match self.current_language {
                                                        Language::Ru => "Открыть Менеджер плагинов",
                                                        _ => "Open Plugin Manager",
                                                    };
                                                    if ui
                                                        .button(
                                                            egui::RichText::new(btn_text)
                                                                .size(11.0),
                                                        )
                                                        .clicked()
                                                    {
                                                        self.open_manager_requested = true;
                                                    }
                                                    ui.add_space(4.0);
                                                }

                                                if let Some(state) = self.states.get_mut(&plugin_id)
                                                {
                                                    for widget in &config.widgets {
                                                        if !matches!(
                                                            widget,
                                                            WidgetNode::ImagePreview { .. }
                                                        ) && widgets::render_widget_node(
                                                            ui,
                                                            &plugin_id,
                                                            widget,
                                                            state,
                                                            palette,
                                                            self.current_language,
                                                            &mut self.preview_thumbnails,
                                                            &mut self.preview_textures,
                                                            self.config_path.as_deref(),
                                                        ) {
                                                            state_changed = true;
                                                        }
                                                    }
                                                }
                                            });
                                    },
                                );

                                ui.separator();

                                if let Some(preview_node) = preview_widget {
                                    let avail_h = (ui.available_height() - 2.0).max(60.0);
                                    let adapted_preview = match preview_node {
                                        WidgetNode::ImagePreview { id, .. } => {
                                            WidgetNode::ImagePreview {
                                                id: id.clone(),
                                                height: avail_h as f64,
                                            }
                                        }
                                        other => other.clone(),
                                    };
                                    if let Some(state) = self.states.get_mut(&plugin_id) {
                                        if widgets::render_widget_node(
                                            ui,
                                            &plugin_id,
                                            &adapted_preview,
                                            state,
                                            palette,
                                            self.current_language,
                                            &mut self.preview_thumbnails,
                                            &mut self.preview_textures,
                                            self.config_path.as_deref(),
                                        ) {
                                            state_changed = true;
                                        }
                                    }
                                }
                            });
                        } else {
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.add_space(2.0);
                                    ui.spacing_mut().item_spacing = Vec2::new(6.0, 8.0);

                                    if models_missing {
                                        let warn_text = match self.current_language {
                                            Language::Ru => "Модели нейросети не установлены",
                                            _ => "AI models are not installed",
                                        };
                                        widgets::draw_vector_banner(
                                            ui,
                                            warn_text,
                                            BannerStyle::Warning,
                                            palette,
                                        );
                                        let btn_text = match self.current_language {
                                            Language::Ru => "Открыть Менеджер плагинов",
                                            _ => "Open Plugin Manager",
                                        };
                                        if ui
                                            .button(egui::RichText::new(btn_text).size(11.0))
                                            .clicked()
                                        {
                                            self.open_manager_requested = true;
                                        }
                                        ui.add_space(4.0);
                                    }

                                    if let Some(state) = self.states.get_mut(&plugin_id) {
                                        for widget in &config.widgets {
                                            if widgets::render_widget_node(
                                                ui,
                                                &plugin_id,
                                                widget,
                                                state,
                                                palette,
                                                self.current_language,
                                                &mut self.preview_thumbnails,
                                                &mut self.preview_textures,
                                                self.config_path.as_deref(),
                                            ) {
                                                state_changed = true;
                                            }
                                        }
                                    }
                                });
                        }
                    });

                if state_changed {
                    sub_ctx.request_repaint();
                }
            });

            if !is_open {
                let mut closed = false;
                if let Some(state) = self.states.get_mut(&plugin_id) {
                    if state.is_open {
                        state.is_open = false;
                        state
                            .values
                            .insert("is_open".to_string(), DynamicValue::Bool(false));
                        closed = true;
                    }
                }
                if closed {
                    self.execute_script_state_change(&plugin_id);
                }
            }

            if state_changed {
                self.execute_script_state_change(&plugin_id);
                ctx.request_repaint();
                ctx.request_repaint_of(viewport_id);
            }
        }
    }
}
