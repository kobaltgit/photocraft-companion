use crate::theme::ThemePalette;
use core_contracts::{
    BannerStyle, InteractiveUiConfig, Language, PluginIpcAction, PluginManifest, WidgetNode,
};
use eframe::egui::{self, Color32, Sense, Stroke, Vec2};
use raster_engine::{self, ExtrudeParams};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::UnboundedSender;
use tracing::{debug, warn};

/// Dynamic value stored in the widget state
#[derive(Debug, Clone, PartialEq)]
pub enum DynamicValue {
    String(String),
    Number(f64),
    Bool(bool),
    Index(usize),
    BannerStyle(BannerStyle),
    StringList(Vec<String>),
    IntList(Vec<i64>),
}

impl DynamicValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            DynamicValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            DynamicValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            DynamicValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_index(&self) -> Option<usize> {
        match self {
            DynamicValue::Index(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_banner_style(&self) -> Option<BannerStyle> {
        match self {
            DynamicValue::BannerStyle(s) => Some(*s),
            _ => None,
        }
    }

    pub fn as_string_list(&self) -> Option<&[String]> {
        match self {
            DynamicValue::StringList(list) => Some(list.as_slice()),
            _ => None,
        }
    }

    pub fn as_int_list(&self) -> Option<&[i64]> {
        match self {
            DynamicValue::IntList(list) => Some(list.as_slice()),
            _ => None,
        }
    }
}

/// Reactive state for a single interactive tool window
#[derive(Debug, Clone)]
pub struct DynamicWindowState {
    pub values: HashMap<String, DynamicValue>,
    pub is_open: bool,
    pub title: String,
    pub default_width: f32,
    pub default_height: f32,
}

impl DynamicWindowState {
    pub fn new(config: &InteractiveUiConfig) -> Self {
        Self::new_with_lang(config, Language::En)
    }

    pub fn new_with_lang(config: &InteractiveUiConfig, lang: Language) -> Self {
        let mut values = HashMap::new();

        for widget in &config.widgets {
            Self::init_widget_values(widget, &mut values);
        }

        Self {
            values,
            is_open: false,
            title: config.window_title_for_lang(lang).to_string(),
            default_width: config.default_width.unwrap_or(320.0),
            default_height: config.default_height.unwrap_or(480.0),
        }
    }

    fn init_widget_values(widget: &WidgetNode, values: &mut HashMap<String, DynamicValue>) {
        match widget {
            WidgetNode::ColorSlot {
                id,
                default_hex,
                target_binding,
                ..
            } => {
                let hex = default_hex.clone().unwrap_or_else(|| "#ffffff".to_string());
                values.insert(id.clone(), DynamicValue::String(hex));
                if let Some(tb) = target_binding {
                    values
                        .entry(tb.clone())
                        .or_insert_with(|| DynamicValue::String(id.clone()));
                }
            }
            WidgetNode::Slider { id, default, .. } => {
                values.insert(id.clone(), DynamicValue::Number(*default));
            }
            WidgetNode::Dropdown {
                id, default_index, ..
            } => {
                values.insert(id.clone(), DynamicValue::Index(*default_index));
            }
            WidgetNode::Toggle { id, default, .. } => {
                values.insert(id.clone(), DynamicValue::Bool(*default));
            }
            WidgetNode::Button { id, .. } => {
                values.insert(format!("action_{}", id), DynamicValue::Bool(false));
            }
            WidgetNode::Banner {
                id,
                message_binding,
                style_binding,
            } => {
                values
                    .entry(message_binding.clone())
                    .or_insert_with(|| DynamicValue::String(String::new()));
                if let Some(sb) = style_binding {
                    values
                        .entry(sb.clone())
                        .or_insert_with(|| DynamicValue::BannerStyle(BannerStyle::Info));
                }
                values.insert(id.clone(), DynamicValue::String(String::new()));
            }
            WidgetNode::Label { text_binding } => {
                values
                    .entry(text_binding.clone())
                    .or_insert_with(|| DynamicValue::String(String::new()));
            }
            WidgetNode::HueWheel { id, .. } => {
                values.insert(id.clone(), DynamicValue::Number(0.0));
            }
            WidgetNode::VectorPad {
                id,
                default_angle_deg,
                default_distance,
                ..
            } => {
                values.insert(
                    format!("{}_angle", id),
                    DynamicValue::Number(*default_angle_deg),
                );
                values.insert(
                    format!("{}_distance", id),
                    DynamicValue::Number(*default_distance),
                );
            }
            WidgetNode::ImagePreview { id, .. } => {
                values.insert(format!("{}_preview", id), DynamicValue::Bool(true));
            }
            WidgetNode::Row { items } => {
                for item in items {
                    Self::init_widget_values(item, values);
                }
            }
            WidgetNode::ModifierStack { id, items, .. } => {
                let order: Vec<String> = items.iter().map(|it| it.id.clone()).collect();
                values.insert(
                    format!("{}_order", id),
                    DynamicValue::StringList(order.clone()),
                );
                values.insert(id.clone(), DynamicValue::StringList(order));
                for it in items {
                    values.insert(format!("{}_enabled", it.id), DynamicValue::Bool(it.enabled));
                    values.insert(format!("{}_value", it.id), DynamicValue::Number(it.default));
                    values.insert(
                        format!("{}_{}_enabled", id, it.id),
                        DynamicValue::Bool(it.enabled),
                    );
                    values.insert(
                        format!("{}_{}_value", id, it.id),
                        DynamicValue::Number(it.default),
                    );
                }
            }
            WidgetNode::Separator => {}
        }
    }

    pub fn to_rhai_map(&self) -> rhai::Map {
        let mut map = rhai::Map::new();
        for (k, v) in &self.values {
            let key = k.clone().into();
            match v {
                DynamicValue::String(s) => {
                    map.insert(key, rhai::Dynamic::from(s.clone()));
                }
                DynamicValue::Number(n) => {
                    map.insert(key, rhai::Dynamic::from(*n));
                }
                DynamicValue::Bool(b) => {
                    map.insert(key, rhai::Dynamic::from(*b));
                }
                DynamicValue::Index(i) => {
                    map.insert(key, rhai::Dynamic::from(*i as rhai::INT));
                }
                DynamicValue::BannerStyle(s) => {
                    let s_str = match s {
                        BannerStyle::Info => "info",
                        BannerStyle::Warning => "warning",
                        BannerStyle::Error => "error",
                        BannerStyle::Success => "success",
                    };
                    map.insert(key, rhai::Dynamic::from(s_str.to_string()));
                }
                DynamicValue::StringList(list) => {
                    let arr: rhai::Array = list
                        .iter()
                        .map(|s| rhai::Dynamic::from(s.clone()))
                        .collect();
                    map.insert(key, rhai::Dynamic::from_array(arr));
                }
                DynamicValue::IntList(list) => {
                    let arr: rhai::Array = list
                        .iter()
                        .map(|&n| rhai::Dynamic::from(n as rhai::INT))
                        .collect();
                    map.insert(key, rhai::Dynamic::from_array(arr));
                }
            }
        }
        map
    }

    pub fn update_from_rhai_map(&mut self, map: rhai::Map) {
        for (k, v) in map {
            let key = k.to_string();
            if v.is_string() {
                if let Ok(s) = v.into_string() {
                    if let Some(existing) = self.values.get_mut(&key) {
                        if matches!(existing, DynamicValue::BannerStyle(_)) {
                            let style = match s.to_lowercase().as_str() {
                                "warning" => BannerStyle::Warning,
                                "error" => BannerStyle::Error,
                                "success" => BannerStyle::Success,
                                _ => BannerStyle::Info,
                            };
                            *existing = DynamicValue::BannerStyle(style);
                            continue;
                        }
                    }
                    self.values.insert(key, DynamicValue::String(s));
                }
            } else if v.is_float() {
                if let Ok(f) = v.as_float() {
                    self.values.insert(key, DynamicValue::Number(f));
                }
            } else if v.is_int() {
                if let Ok(i) = v.as_int() {
                    if let Some(existing) = self.values.get_mut(&key) {
                        if matches!(existing, DynamicValue::Index(_)) {
                            *existing = DynamicValue::Index(i.max(0) as usize);
                            continue;
                        }
                    }
                    self.values.insert(key, DynamicValue::Number(i as f64));
                }
            } else if v.is_bool() {
                if let Ok(b) = v.as_bool() {
                    self.values.insert(key, DynamicValue::Bool(b));
                }
            } else if v.is_array() {
                if let Ok(arr) = v.clone().into_typed_array::<i64>() {
                    self.values.insert(key, DynamicValue::IntList(arr));
                } else if let Ok(arr) = v.clone().into_typed_array::<String>() {
                    self.values.insert(key, DynamicValue::StringList(arr));
                } else if let Ok(arr) = v.into_array() {
                    let all_ints = !arr.is_empty() && arr.iter().all(|x| x.is_int());
                    if all_ints {
                        let list: Vec<i64> =
                            arr.into_iter().filter_map(|x| x.as_int().ok()).collect();
                        self.values.insert(key, DynamicValue::IntList(list));
                    } else {
                        let mut list = Vec::new();
                        for item in arr {
                            if let Ok(s) = item.clone().into_string() {
                                list.push(s);
                            } else {
                                list.push(item.to_string());
                            }
                        }
                        self.values.insert(key, DynamicValue::StringList(list));
                    }
                }
            } else if v.is_unit() {
                self.values.remove(&key);
            }
        }
    }
}

/// Universal Dynamic Tool Engine managing dynamic windows and Rhai runtime
pub struct DynamicToolEngine {
    manifests: HashMap<String, PluginManifest>,
    states: HashMap<String, DynamicWindowState>,
    rhai_engine: rhai::Engine,
    rhai_asts: HashMap<String, rhai::AST>,
    #[allow(dead_code)]
    action_tx: Option<UnboundedSender<PluginIpcAction>>,
    current_language: Language,
    lang_arc: Arc<RwLock<Language>>,
    echo_guard: Arc<std::sync::Mutex<Option<(String, String)>>>,
    pub preview_thumbnails: HashMap<String, (image::RgbaImage, f32)>,
    pub preview_textures: HashMap<String, (egui::TextureHandle, u64)>,
    pub has_selection: bool,
    pub config_path: Option<std::path::PathBuf>,
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

        let mut rhai_engine = rhai::Engine::new();
        rhai_engine.set_max_expr_depths(64, 64);
        rhai_engine.set_max_operations(50_000);
        let lang_arc = Arc::new(RwLock::new(language));
        let echo_guard: Arc<std::sync::Mutex<Option<(String, String)>>> =
            Arc::new(std::sync::Mutex::new(None));

        // Register photocraft commands in Rhai
        if let Some(tx) = action_tx.clone() {
            let tx_cmd = tx.clone();
            let echo_guard_cmd = echo_guard.clone();
            rhai_engine.register_fn(
                "execute_command",
                move |command: rhai::ImmutableString, params: rhai::Map| {
                    if command.as_str() == "tools.setColors" {
                        let fg = params
                            .get("foreground")
                            .and_then(|v| v.clone().into_string().ok());
                        let bg = params
                            .get("background")
                            .and_then(|v| v.clone().into_string().ok());
                        if let (Some(f), Some(b)) = (fg, bg) {
                            if let Ok(mut g) = echo_guard_cmd.lock() {
                                *g = Some((f.to_lowercase(), b.to_lowercase()));
                            }
                        }
                    }

                    let json_val = rhai::serde::from_dynamic::<serde_json::Value>(
                        &rhai::Dynamic::from(params),
                    )
                    .unwrap_or(serde_json::Value::Null);

                    let action = PluginIpcAction {
                        command: command.to_string(),
                        params: json_val,
                    };
                    let _ = tx_cmd.send(action);
                },
            );

            let tx_bare = tx.clone();
            rhai_engine.register_fn("execute_command", move |command: rhai::ImmutableString| {
                let action = PluginIpcAction {
                    command: command.to_string(),
                    params: serde_json::Value::Null,
                };
                let _ = tx_bare.send(action);
            });

            let tx_colors = tx.clone();
            let echo_guard_colors = echo_guard.clone();
            rhai_engine.register_fn(
                "set_colors",
                move |foreground: rhai::ImmutableString, background: rhai::ImmutableString| {
                    if let Ok(mut g) = echo_guard_colors.lock() {
                        *g = Some((
                            foreground.to_string().to_lowercase(),
                            background.to_string().to_lowercase(),
                        ));
                    }
                    let action = PluginIpcAction {
                        command: "tools.setColors".to_string(),
                        params: serde_json::json!({
                            "foreground": foreground.to_string(),
                            "background": background.to_string()
                        }),
                    };
                    let _ = tx_colors.send(action);
                },
            );

            let tx_raster = tx;
            rhai_engine.register_fn(
                "execute_raster_process",
                move |task: rhai::ImmutableString, params: rhai::Map| {
                    let json_params = rhai::serde::from_dynamic::<serde_json::Value>(
                        &rhai::Dynamic::from(params),
                    )
                    .unwrap_or(serde_json::Value::Null);

                    let action = PluginIpcAction::new(
                        "raster.process",
                        serde_json::json!({
                            "task": task.to_string(),
                            "params": json_params,
                        }),
                    );
                    let _ = tx_raster.send(action);
                },
            );
        }

        // Register internationalization & localization helper functions
        let lang_t = lang_arc.clone();
        rhai_engine.register_fn(
            "t",
            move |en: rhai::ImmutableString, ru: rhai::ImmutableString| -> String {
                let lang = *lang_t.read().unwrap();
                if matches!(lang, Language::Ru) {
                    ru.to_string()
                } else {
                    en.to_string()
                }
            },
        );

        let lang_curr = lang_arc.clone();
        rhai_engine.register_fn("current_language", move || -> String {
            let lang = *lang_curr.read().unwrap();
            if matches!(lang, Language::Ru) {
                "ru".to_string()
            } else {
                "en".to_string()
            }
        });

        // Register color formatting helpers (integer format, zero floating point defects)
        rhai_engine.register_fn("rgb_to_hex", |r: i64, g: i64, b: i64| -> String {
            let r_clamped = r.clamp(0, 255) as u8;
            let g_clamped = g.clamp(0, 255) as u8;
            let b_clamped = b.clamp(0, 255) as u8;
            format!("#{r_clamped:02x}{g_clamped:02x}{b_clamped:02x}")
        });
        rhai_engine.register_fn("rgb_to_hex", |r: f64, g: f64, b: f64| -> String {
            let r_clamped = r.round().clamp(0.0, 255.0) as u8;
            let g_clamped = g.round().clamp(0.0, 255.0) as u8;
            let b_clamped = b.round().clamp(0.0, 255.0) as u8;
            format!("#{r_clamped:02x}{g_clamped:02x}{b_clamped:02x}")
        });

        rhai_engine.register_fn("hex_to_rgb", |hex: rhai::ImmutableString| -> rhai::Map {
            let mut map = rhai::Map::new();
            if let Some(color) = parse_hex_color(&hex) {
                map.insert("r".into(), rhai::Dynamic::from(color.r() as i64));
                map.insert("g".into(), rhai::Dynamic::from(color.g() as i64));
                map.insert("b".into(), rhai::Dynamic::from(color.b() as i64));
            } else {
                map.insert("r".into(), rhai::Dynamic::from(0_i64));
                map.insert("g".into(), rhai::Dynamic::from(0_i64));
                map.insert("b".into(), rhai::Dynamic::from(0_i64));
            }
            map
        });

        // Register named color database (140+ CSS3 & classic artist pigments with weighted perceptual distance)
        let lang_named1 = lang_arc.clone();
        rhai_engine.register_fn("get_named_color", move |r: i64, g: i64, b: i64| -> String {
            let lang = *lang_named1.read().unwrap();
            find_nearest_named_color(r, g, b, lang)
        });
        let lang_named2 = lang_arc.clone();
        rhai_engine.register_fn("get_named_color", move |r: f64, g: f64, b: f64| -> String {
            let lang = *lang_named2.read().unwrap();
            find_nearest_named_color(r.round() as i64, g.round() as i64, b.round() as i64, lang)
        });

        // Register Brettel color vision deficiency (CVD) simulation
        rhai_engine.register_fn(
            "simulate_cvd",
            |r: i64, g: i64, b: i64, profile_index: i64| -> String {
                simulate_cvd_hex(r as f64, g as f64, b as f64, profile_index)
            },
        );
        rhai_engine.register_fn(
            "simulate_cvd",
            |r: f64, g: f64, b: f64, profile_index: i64| -> String {
                simulate_cvd_hex(r, g, b, profile_index)
            },
        );
        rhai_engine.register_fn(
            "simulate_cvd",
            |r: f64, g: f64, b: f64, profile_index: f64| -> String {
                simulate_cvd_hex(r, g, b, profile_index.round() as i64)
            },
        );

        // Register standard WCAG 2.1 contrast ratio calculation
        rhai_engine.register_fn(
            "calculate_contrast",
            |r1: i64, g1: i64, b1: i64, r2: i64, g2: i64, b2: i64| -> f64 {
                calculate_wcag_contrast(
                    r1 as f64, g1 as f64, b1 as f64, r2 as f64, g2 as f64, b2 as f64,
                )
            },
        );
        rhai_engine.register_fn(
            "calculate_contrast",
            |r1: f64, g1: f64, b1: f64, r2: f64, g2: f64, b2: f64| -> f64 {
                calculate_wcag_contrast(r1, g1, b1, r2, g2, b2)
            },
        );

        // Initialize manifests and compile scripts
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
        };

        // Run initial state pass for all loaded scripts
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
                self.preview_thumbnails.remove(plugin_id);
                self.preview_textures.remove(plugin_id);
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

    /// Check if any currently open dynamic window has declared subscription to the given topic
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

    /// Dispatch session colors (foreground, background) to all open windows subscribed to session.colors
    /// Returns true if at least one window state was updated and needs UI repaint.
    pub fn dispatch_session_colors(&mut self, fg: &str, bg: &str) -> bool {
        let fg_lower = fg.to_lowercase();
        let bg_lower = bg.to_lowercase();

        // Echo suppression: check if the incoming colors match what the plugin itself just sent
        {
            if let Ok(mut guard) = self.echo_guard.lock() {
                if let Some((last_fg, last_bg)) = guard.as_ref() {
                    if last_fg == &fg_lower && last_bg == &bg_lower {
                        return false; // Echo suppressed!
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

    /// Dispatch selection state (has_selection, selection_bounds) to all open windows subscribed to document.selection
    /// Returns true if at least one window state was updated and needs UI repaint.
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
                            Err(_) => {
                                // Backward compatibility fallback to 2 arguments: (has_selection, rhai_state)
                                self.rhai_engine.call_fn(
                                    &mut rhai::Scope::new(),
                                    ast,
                                    "on_selection_state",
                                    (has_selection, rhai_state),
                                )
                            }
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

    /// Generates a deterministic ViewportId for an interactive plugin floating window
    pub fn viewport_id_for_plugin(plugin_id: &str) -> egui::ViewportId {
        egui::ViewportId::from_hash_of(format!("dynamic_plugin_window_{}", plugin_id))
    }

    fn execute_script_state_change(&mut self, plugin_id: &str) {
        if let Some(ast) = self.rhai_asts.get(plugin_id) {
            if let Some(state) = self.states.get_mut(plugin_id) {
                let mut scope = rhai::Scope::new();
                let state_map = state.to_rhai_map();

                match self.rhai_engine.call_fn::<rhai::Map>(
                    &mut scope,
                    ast,
                    "on_state_changed",
                    (state_map,),
                ) {
                    Ok(mutated_map) => {
                        state.update_from_rhai_map(mutated_map);
                    }
                    Err(e) => {
                        debug!("on_state_changed in script for '{}': {}", plugin_id, e);
                    }
                }
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

            let vp = egui::ViewportBuilder::default()
                .with_title(&win_title)
                .with_inner_size([default_w, default_h])
                .with_min_inner_size([260.0, 180.0])
                .with_resizable(true)
                .with_always_on_top();

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
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.add_space(2.0);
                                ui.spacing_mut().item_spacing = Vec2::new(6.0, 8.0);

                                if let Some(state) = self.states.get_mut(&plugin_id) {
                                    for widget in &config.widgets {
                                        if Self::render_widget_node(
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
                    });
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
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_widget_node(
        ui: &mut egui::Ui,
        plugin_id: &str,
        node: &WidgetNode,
        state: &mut DynamicWindowState,
        palette: &ThemePalette,
        lang: Language,
        preview_thumbnails: &mut HashMap<String, (image::RgbaImage, f32)>,
        preview_textures: &mut HashMap<String, (egui::TextureHandle, u64)>,
        config_path: Option<&std::path::Path>,
    ) -> bool {
        let mut changed = false;

        match node {
            WidgetNode::ColorSlot {
                id,
                read_only,
                selectable,
                target_binding,
                ..
            } => {
                let mut current_hex = match state.values.get(id) {
                    Some(DynamicValue::String(s)) => s.clone(),
                    _ => "#000000".to_string(),
                };

                let parsed_color = parse_hex_color(&current_hex).unwrap_or(Color32::BLACK);

                let is_selected = target_binding
                    .as_ref()
                    .and_then(|tb| state.values.get(tb))
                    .and_then(|v| v.as_str())
                    .map(|sel_id| sel_id == id)
                    .unwrap_or(false);

                ui.horizontal(|ui| {
                    if *read_only {
                        let (rect, _) =
                            ui.allocate_exact_size(Vec2::new(20.0, 20.0), Sense::hover());
                        ui.painter().rect_filled(rect, 4.0, parsed_color);
                        ui.painter().rect_stroke(
                            rect,
                            4.0,
                            Stroke::new(1.0, palette.border_stroke),
                            egui::StrokeKind::Inside,
                        );
                    } else {
                        let mut srgba = parsed_color;
                        if egui::color_picker::color_edit_button_srgba(
                            ui,
                            &mut srgba,
                            egui::color_picker::Alpha::Opaque,
                        )
                        .changed()
                        {
                            current_hex =
                                format!("#{:02x}{:02x}{:02x}", srgba.r(), srgba.g(), srgba.b());
                            state
                                .values
                                .insert(id.clone(), DynamicValue::String(current_hex.clone()));
                            changed = true;
                        }
                    }

                    let label_text = node.label_for_lang(lang);
                    let label_color = if is_selected {
                        palette.accent_blue
                    } else {
                        palette.text_primary
                    };
                    let label_resp = ui.add(
                        egui::Label::new(
                            egui::RichText::new(label_text)
                                .size(12.0)
                                .color(label_color),
                        )
                        .sense(Sense::click()),
                    );
                    if *selectable && label_resp.clicked() {
                        if let Some(tb) = target_binding {
                            state
                                .values
                                .insert(tb.clone(), DynamicValue::String(id.clone()));
                            changed = true;
                        }
                    }
                });
            }

            WidgetNode::Button { id, icon, .. } => {
                let action_key = format!("action_{}", id);
                let label_text = node.label_for_lang(lang);
                let is_swap = icon.as_deref() == Some("swap");

                let resp = ui
                    .scope(|ui| {
                        if is_swap {
                            // Extra left padding to house the vector swap icon cleanly without touching text
                            ui.spacing_mut().button_padding = Vec2::new(18.0, 4.0);
                        }
                        ui.add(
                            egui::Button::new(egui::RichText::new(label_text).size(12.0))
                                .min_size(Vec2::new(0.0, 24.0)),
                        )
                    })
                    .inner;

                if is_swap {
                    let icon_c = egui::pos2(resp.rect.min.x + 9.5, resp.rect.center().y);
                    let stroke_color = if resp.hovered() {
                        palette.accent_hover
                    } else {
                        palette.text_primary
                    };
                    let stroke = Stroke::new(1.2, stroke_color);

                    // Top arrow pointing right
                    let y_top = icon_c.y - 2.5;
                    ui.painter().line_segment(
                        [
                            egui::pos2(icon_c.x - 4.5, y_top),
                            egui::pos2(icon_c.x + 4.5, y_top),
                        ],
                        stroke,
                    );
                    ui.painter().line_segment(
                        [
                            egui::pos2(icon_c.x + 4.5, y_top),
                            egui::pos2(icon_c.x + 2.0, y_top - 2.0),
                        ],
                        stroke,
                    );
                    ui.painter().line_segment(
                        [
                            egui::pos2(icon_c.x + 4.5, y_top),
                            egui::pos2(icon_c.x + 2.0, y_top + 2.0),
                        ],
                        stroke,
                    );

                    // Bottom arrow pointing left
                    let y_bot = icon_c.y + 2.5;
                    ui.painter().line_segment(
                        [
                            egui::pos2(icon_c.x - 4.5, y_bot),
                            egui::pos2(icon_c.x + 4.5, y_bot),
                        ],
                        stroke,
                    );
                    ui.painter().line_segment(
                        [
                            egui::pos2(icon_c.x - 4.5, y_bot),
                            egui::pos2(icon_c.x - 2.0, y_bot - 2.0),
                        ],
                        stroke,
                    );
                    ui.painter().line_segment(
                        [
                            egui::pos2(icon_c.x - 4.5, y_bot),
                            egui::pos2(icon_c.x - 2.0, y_bot + 2.0),
                        ],
                        stroke,
                    );
                }

                if resp.clicked() {
                    state.values.insert(action_key, DynamicValue::Bool(true));
                    changed = true;
                }
            }

            WidgetNode::Slider {
                id, min, max, step, ..
            } => {
                let mut current_val = match state.values.get(id) {
                    Some(DynamicValue::Number(n)) => *n,
                    _ => *min,
                };

                ui.horizontal(|ui| {
                    ui.label(node.label_for_lang(lang));
                    let step_val = step.unwrap_or(1.0);
                    let is_integer = step_val >= 1.0;
                    let mut slider = egui::Slider::new(&mut current_val, *min..=*max)
                        .step_by(step_val)
                        .show_value(true);

                    if is_integer {
                        slider = slider
                            .custom_formatter(|v, _| format!("{:.0}", v))
                            .custom_parser(|s| s.trim().parse::<f64>().ok());
                    } else {
                        slider = slider
                            .custom_formatter(|v, _| format!("{:.1}", v))
                            .custom_parser(|s| s.trim().parse::<f64>().ok());
                    }

                    let resp = ui.add(slider);
                    let mut slider_changed = resp.changed();

                    let mut wheel_ticks: f32 = 0.0;
                    ui.input(|i| {
                        for event in &i.events {
                            if let egui::Event::MouseWheel { delta, .. } = event {
                                if delta.y > 0.0 {
                                    wheel_ticks += 1.0;
                                } else if delta.y < 0.0 {
                                    wheel_ticks -= 1.0;
                                }
                            }
                        }
                    });

                    if wheel_ticks != 0.0 && resp.hovered() {
                        let wheel_step = if is_integer { 1.0 } else { 0.1 };
                        let new_val =
                            (current_val + (wheel_ticks as f64) * wheel_step).clamp(*min, *max);
                        let new_val = if is_integer {
                            new_val.round()
                        } else {
                            (new_val * 10.0).round() / 10.0
                        };
                        if (new_val - current_val).abs() > f64::EPSILON {
                            current_val = new_val;
                            slider_changed = true;
                        }
                    }

                    if slider_changed {
                        state
                            .values
                            .insert(id.clone(), DynamicValue::Number(current_val));
                        changed = true;
                    }
                });
            }

            WidgetNode::Dropdown { id, .. } => {
                let mut current_idx = match state.values.get(id) {
                    Some(DynamicValue::Index(i)) => *i,
                    _ => 0,
                };

                let options = node.options_for_lang(lang);

                ui.horizontal(|ui| {
                    ui.label(node.label_for_lang(lang));
                    let selected_text = options.get(current_idx).cloned().unwrap_or_default();
                    egui::ComboBox::from_id_salt(id)
                        .selected_text(selected_text)
                        .width(ui.available_width() - 8.0)
                        .show_ui(ui, |ui| {
                            for (idx, opt) in options.iter().enumerate() {
                                if ui.selectable_value(&mut current_idx, idx, opt).changed() {
                                    state
                                        .values
                                        .insert(id.clone(), DynamicValue::Index(current_idx));
                                    changed = true;
                                }
                            }
                        });
                });
            }

            WidgetNode::Toggle { id, .. } => {
                let mut current_bool = match state.values.get(id) {
                    Some(DynamicValue::Bool(b)) => *b,
                    _ => false,
                };

                if ui
                    .checkbox(&mut current_bool, node.label_for_lang(lang))
                    .changed()
                {
                    state
                        .values
                        .insert(id.clone(), DynamicValue::Bool(current_bool));
                    changed = true;
                }
            }

            WidgetNode::HueWheel {
                source_color_id,
                show_pointer,
                ..
            } => {
                let source_hex = {
                    let raw_val = state.values.get(source_color_id).and_then(|v| v.as_str());
                    match raw_val {
                        Some(s) if s.starts_with('#') => s.to_string(),
                        Some(target_id) => state
                            .values
                            .get(target_id)
                            .and_then(|v| v.as_str())
                            .unwrap_or("#ff0000")
                            .to_string(),
                        None => "#ff0000".to_string(),
                    }
                };
                let parsed = parse_hex_color(&source_hex).unwrap_or(Color32::from_rgb(255, 0, 0));
                let hue_deg = rgb_to_hue_degrees(parsed);

                ui.vertical_centered(|ui| {
                    Self::draw_vector_hue_wheel(ui, hue_deg, *show_pointer, palette);
                });
            }

            WidgetNode::Banner {
                message_binding,
                style_binding,
                ..
            } => {
                let msg = match state.values.get(message_binding) {
                    Some(DynamicValue::String(s)) => s.clone(),
                    _ => String::new(),
                };

                if !msg.is_empty() {
                    let style = style_binding
                        .as_ref()
                        .and_then(|sb| state.values.get(sb))
                        .and_then(|v| v.as_banner_style())
                        .unwrap_or(BannerStyle::Info);

                    Self::draw_vector_banner(ui, &msg, style, palette);
                }
            }

            WidgetNode::Label { text_binding } => {
                let text = match state.values.get(text_binding) {
                    Some(DynamicValue::String(s)) => s.clone(),
                    _ => String::new(),
                };
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new(text).strong());
                });
            }

            WidgetNode::VectorPad {
                id,
                max_distance,
                default_angle_deg,
                default_distance,
                ..
            } => {
                let angle_key = format!("{}_angle", id);
                let distance_key = format!("{}_distance", id);

                let mut current_angle = state
                    .values
                    .get(&angle_key)
                    .and_then(|v| v.as_f64())
                    .unwrap_or(*default_angle_deg);
                let mut current_distance = state
                    .values
                    .get(&distance_key)
                    .and_then(|v| v.as_f64())
                    .unwrap_or(*default_distance);

                let label_text = node.label_for_lang(lang);
                if !label_text.is_empty() {
                    ui.label(label_text);
                }

                ui.vertical_centered(|ui| {
                    let pad_diameter = 130.0;
                    let pad_size = Vec2::splat(pad_diameter);
                    let (rect, response) =
                        ui.allocate_exact_size(pad_size, Sense::click_and_drag());
                    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);

                    let center = rect.center();
                    let radius = (pad_diameter * 0.5) - 4.0;

                    if response.dragged() || response.clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let delta = pos - center;
                            let dist_px = (delta.x * delta.x + delta.y * delta.y).sqrt();
                            let norm = (dist_px / radius).clamp(0.0, 1.0);
                            let new_dist = ((norm as f64 * *max_distance) * 10.0).round() / 10.0;

                            let mut rad = delta.y.atan2(delta.x);
                            if rad < 0.0 {
                                rad += std::f32::consts::TAU;
                            }
                            let mut deg = (rad.to_degrees() as f64).round();
                            if deg >= 360.0 {
                                deg -= 360.0;
                            }

                            if (deg - current_angle).abs() > 0.01
                                || (new_dist - current_distance).abs() > 0.01
                            {
                                current_angle = deg;
                                current_distance = new_dist;
                                state
                                    .values
                                    .insert(angle_key.clone(), DynamicValue::Number(current_angle));
                                state.values.insert(
                                    distance_key.clone(),
                                    DynamicValue::Number(current_distance),
                                );
                                changed = true;
                            }
                        }
                    }

                    let painter = ui.painter();

                    // 1. Circular viewfinder background and border
                    painter.circle_filled(center, radius, palette.panel_fill);
                    painter.circle_stroke(center, radius, Stroke::new(1.0, palette.border_stroke));
                    painter.circle_stroke(
                        center,
                        radius * 0.33,
                        Stroke::new(0.5, palette.border_stroke.gamma_multiply(0.4)),
                    );
                    painter.circle_stroke(
                        center,
                        radius * 0.66,
                        Stroke::new(0.5, palette.border_stroke.gamma_multiply(0.4)),
                    );

                    // 2. Crosshair guide lines
                    let cross_stroke = Stroke::new(1.0, palette.border_stroke.gamma_multiply(0.6));
                    painter.line_segment(
                        [
                            egui::pos2(center.x - radius, center.y),
                            egui::pos2(center.x + radius, center.y),
                        ],
                        cross_stroke,
                    );
                    painter.line_segment(
                        [
                            egui::pos2(center.x, center.y - radius),
                            egui::pos2(center.x, center.y + radius),
                        ],
                        cross_stroke,
                    );

                    // Center pivot point
                    painter.circle_filled(center, 2.5, palette.text_muted);

                    // 3. Thrust vector line and handle marker with accent color
                    let rad = (current_angle as f32).to_radians();
                    let dist_ratio = if *max_distance > 0.0 {
                        (current_distance / *max_distance).clamp(0.0, 1.0) as f32
                    } else {
                        0.0
                    };
                    let handle_pos =
                        center + Vec2::new(rad.cos(), rad.sin()) * (radius * dist_ratio);

                    painter
                        .line_segment([center, handle_pos], Stroke::new(2.0, palette.accent_blue));
                    painter.circle_filled(handle_pos, 5.0, palette.accent_blue);
                    painter.circle_stroke(handle_pos, 5.0, Stroke::new(1.5, Color32::WHITE));

                    // 4. Numeric readout of angle and distance below the viewfinder (No prohibited unicode)
                    let readout = format!("{:.0} deg, {:.0} px", current_angle, current_distance);
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(readout)
                            .size(11.0)
                            .monospace()
                            .color(palette.text_muted),
                    );
                });
            }

            WidgetNode::ImagePreview { height, .. } => {
                let preview_h = (*height as f32).max(60.0);
                let preview_w = ui.available_width();
                let (rect, _resp) =
                    ui.allocate_exact_size(Vec2::new(preview_w, preview_h), Sense::hover());

                // 1. Checkerboard transparency background
                let cell_size = 10.0;
                let cols = (rect.width() / cell_size).ceil() as usize;
                let rows = (rect.height() / cell_size).ceil() as usize;
                for r in 0..rows {
                    for c in 0..cols {
                        let is_even = (r + c) % 2 == 0;
                        let c_color = if is_even {
                            Color32::from_rgb(38, 40, 46)
                        } else {
                            Color32::from_rgb(26, 28, 32)
                        };
                        let x0 = rect.min.x + c as f32 * cell_size;
                        let y0 = rect.min.y + r as f32 * cell_size;
                        let x1 = (x0 + cell_size).min(rect.max.x);
                        let y1 = (y0 + cell_size).min(rect.max.y);
                        if x0 < x1 && y0 < y1 {
                            let cell_rect =
                                egui::Rect::from_min_max(egui::pos2(x0, y0), egui::pos2(x1, y1));
                            ui.painter().rect_filled(cell_rect, 0.0, c_color);
                        }
                    }
                }

                // 2. Render preview or vector placeholder
                if let Some((thumbnail, scale_k)) = preview_thumbnails.get(plugin_id) {
                    let raw_params = extract_extrude_params(&state.values);
                    let scaled_params = raw_params.scaled(*scale_k);
                    let hash = hash_extrude_params(&scaled_params, *scale_k);

                    let needs_render = match preview_textures.get(plugin_id) {
                        Some((_, cached_hash)) => *cached_hash != hash,
                        None => true,
                    };

                    if needs_render {
                        let rendered =
                            raster_engine::render_extrude_preview(thumbnail, &scaled_params);
                        let (rw, rh) = rendered.dimensions();
                        let color_image = egui::ColorImage::from_rgba_unmultiplied(
                            [rw as usize, rh as usize],
                            rendered.as_raw(),
                        );
                        let tex_handle = ui.ctx().load_texture(
                            format!("preview_texture_{}", plugin_id),
                            color_image,
                            egui::TextureOptions::LINEAR,
                        );
                        preview_textures.insert(plugin_id.to_string(), (tex_handle, hash));
                    }

                    if let Some((handle, _)) = preview_textures.get(plugin_id) {
                        let img_size = handle.size_vec2();
                        if img_size.x > 0.0 && img_size.y > 0.0 {
                            let padding = 8.0;
                            let avail_w = (rect.width() - padding * 2.0).max(1.0);
                            let avail_h = (rect.height() - padding * 2.0).max(1.0);
                            let fit_scale = (avail_w / img_size.x).min(avail_h / img_size.y);
                            let final_size = img_size * fit_scale;
                            let draw_rect = egui::Rect::from_center_size(rect.center(), final_size);

                            ui.painter().image(
                                handle.id(),
                                draw_rect,
                                egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                ),
                                Color32::WHITE,
                            );
                        }
                    }
                } else {
                    // Vector placeholder: "No active layer loaded" / "Слой не загружен"
                    let center = rect.center();
                    let doc_w = 26.0;
                    let doc_h = 32.0;
                    let doc_rect = egui::Rect::from_center_size(
                        center - Vec2::new(0.0, 12.0),
                        Vec2::new(doc_w, doc_h),
                    );
                    ui.painter().rect_stroke(
                        doc_rect,
                        2.0,
                        Stroke::new(1.2, palette.text_muted.gamma_multiply(0.4)),
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().line_segment(
                        [
                            egui::pos2(doc_rect.min.x + 5.0, doc_rect.min.y + 8.0),
                            egui::pos2(doc_rect.max.x - 5.0, doc_rect.min.y + 8.0),
                        ],
                        Stroke::new(1.0, palette.text_muted.gamma_multiply(0.3)),
                    );
                    ui.painter().line_segment(
                        [
                            egui::pos2(doc_rect.min.x + 5.0, doc_rect.min.y + 14.0),
                            egui::pos2(doc_rect.max.x - 5.0, doc_rect.min.y + 14.0),
                        ],
                        Stroke::new(1.0, palette.text_muted.gamma_multiply(0.3)),
                    );
                    ui.painter().line_segment(
                        [
                            egui::pos2(doc_rect.min.x + 5.0, doc_rect.min.y + 20.0),
                            egui::pos2(doc_rect.max.x - 10.0, doc_rect.min.y + 20.0),
                        ],
                        Stroke::new(1.0, palette.text_muted.gamma_multiply(0.3)),
                    );

                    let placeholder_text = match lang {
                        Language::Ru => "Слой не загружен",
                        _ => "No active layer loaded",
                    };
                    let font_id = egui::FontId::proportional(12.0);
                    ui.painter().text(
                        center + Vec2::new(0.0, 18.0),
                        egui::Align2::CENTER_CENTER,
                        placeholder_text,
                        font_id,
                        palette.text_muted,
                    );
                }

                // 3. Border frame
                ui.painter().rect_stroke(
                    rect,
                    4.0,
                    Stroke::new(1.0, palette.border_stroke),
                    egui::StrokeKind::Inside,
                );

                // 4. Interactive Vector Refresh Button (Reload active layer from PhotoCraft)
                let refresh_btn_size = Vec2::new(22.0, 22.0);
                let refresh_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.max.x - refresh_btn_size.x - 6.0, rect.min.y + 6.0),
                    refresh_btn_size,
                );
                let refresh_id = ui.make_persistent_id(format!("{}_preview_refresh", plugin_id));
                let refresh_resp = ui.interact(refresh_rect, refresh_id, egui::Sense::click());

                let is_hovered = refresh_resp.hovered();
                let bg_col = if is_hovered {
                    palette.button_secondary_hover
                } else {
                    palette.card_fill
                };
                ui.painter().rect_filled(refresh_rect, 3.0, bg_col);
                ui.painter().rect_stroke(
                    refresh_rect,
                    3.0,
                    Stroke::new(
                        1.0,
                        if is_hovered {
                            palette.accent_hover
                        } else {
                            palette.border_stroke
                        },
                    ),
                    egui::StrokeKind::Inside,
                );

                // Vector refresh icon (circular arc + arrow)
                let c = refresh_rect.center();
                let r = 5.0;
                let stroke = Stroke::new(
                    1.4,
                    if is_hovered {
                        palette.accent_blue
                    } else {
                        palette.text_muted
                    },
                );
                let points: Vec<egui::Pos2> = (0..=12)
                    .map(|i| {
                        let angle = std::f32::consts::PI * 0.25
                            + (i as f32 / 12.0) * (std::f32::consts::PI * 1.5);
                        c + Vec2::new(angle.cos() * r, angle.sin() * r)
                    })
                    .collect();
                for w in points.windows(2) {
                    ui.painter().line_segment([w[0], w[1]], stroke);
                }
                if let Some(&last) = points.last() {
                    let arrow1 = last + Vec2::new(-2.5, -2.0);
                    let arrow2 = last + Vec2::new(2.5, -1.0);
                    ui.painter().line_segment([last, arrow1], stroke);
                    ui.painter().line_segment([last, arrow2], stroke);
                }

                let tooltip_text = match lang {
                    Language::Ru => "Обновить слой из PhotoCraft",
                    _ => "Refresh layer from PhotoCraft",
                };
                let clicked = refresh_resp.clicked();
                refresh_resp.on_hover_text(tooltip_text);

                if clicked {
                    preview_thumbnails.remove(plugin_id);
                    preview_textures.remove(plugin_id);
                    ui.ctx().request_repaint();
                    changed = true;
                }
            }

            WidgetNode::Separator => {
                ui.separator();
            }

            WidgetNode::Row { items } => {
                if items.len() > 1 {
                    ui.columns(items.len(), |columns| {
                        for (i, item) in items.iter().enumerate() {
                            if Self::render_widget_node(
                                &mut columns[i],
                                plugin_id,
                                item,
                                state,
                                palette,
                                lang,
                                preview_thumbnails,
                                preview_textures,
                                config_path,
                            ) {
                                changed = true;
                            }
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        for item in items {
                            if Self::render_widget_node(
                                ui,
                                plugin_id,
                                item,
                                state,
                                palette,
                                lang,
                                preview_thumbnails,
                                preview_textures,
                                config_path,
                            ) {
                                changed = true;
                            }
                        }
                    });
                }
            }

            WidgetNode::ModifierStack {
                id,
                label: _,
                label_ru: _,
                items,
                allow_reordering,
            } => {
                ui.set_width(ui.available_width());
                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        egui::RichText::new(node.label_for_lang(lang))
                            .strong()
                            .size(13.0)
                            .color(palette.text_primary),
                    );
                    ui.add_space(2.0);

                    let order_key = format!("{}_order", id);
                    let mut current_order: Vec<String> = match state.values.get(&order_key) {
                        Some(DynamicValue::StringList(list)) => list.clone(),
                        _ => items.iter().map(|it| it.id.clone()).collect(),
                    };

                    let mut swap_pair = None;

                    let font_id = egui::FontId::proportional(12.0);
                    let max_label_w = items
                        .iter()
                        .map(|it| {
                            ui.painter()
                                .layout_no_wrap(
                                    it.label_for_lang(lang).to_string(),
                                    font_id.clone(),
                                    palette.text_primary,
                                )
                                .size()
                                .x
                        })
                        .fold(80.0_f32, |a, b| a.max(b))
                        + 12.0;

                    for (idx, item_id) in current_order.iter().enumerate() {
                        if let Some(item_cfg) = items.iter().find(|it| &it.id == item_id) {
                            ui.horizontal(|ui| {
                                if *allow_reordering {
                                    // Move Up Vector Button
                                    let up_enabled = idx > 0;
                                    let up_resp = ui.add_enabled(
                                        up_enabled,
                                        egui::Button::new("")
                                            .min_size(Vec2::new(18.0, 18.0))
                                            .corner_radius(2.0),
                                    );
                                    let c = up_resp.rect.center();
                                    let stroke_color = if up_enabled {
                                        if up_resp.hovered() {
                                            palette.accent_hover
                                        } else {
                                            palette.text_primary
                                        }
                                    } else {
                                        palette.text_muted
                                    };
                                    let stroke = Stroke::new(1.2, stroke_color);
                                    ui.painter().line_segment(
                                        [
                                            egui::pos2(c.x - 3.5, c.y + 1.5),
                                            egui::pos2(c.x, c.y - 2.0),
                                        ],
                                        stroke,
                                    );
                                    ui.painter().line_segment(
                                        [
                                            egui::pos2(c.x, c.y - 2.0),
                                            egui::pos2(c.x + 3.5, c.y + 1.5),
                                        ],
                                        stroke,
                                    );
                                    if up_resp.clicked() && idx > 0 {
                                        swap_pair = Some((idx, idx - 1));
                                    }

                                    // Move Down Vector Button
                                    let down_enabled = idx + 1 < current_order.len();
                                    let down_resp = ui.add_enabled(
                                        down_enabled,
                                        egui::Button::new("")
                                            .min_size(Vec2::new(18.0, 18.0))
                                            .corner_radius(2.0),
                                    );
                                    let c = down_resp.rect.center();
                                    let stroke_color = if down_enabled {
                                        if down_resp.hovered() {
                                            palette.accent_hover
                                        } else {
                                            palette.text_primary
                                        }
                                    } else {
                                        palette.text_muted
                                    };
                                    let stroke = Stroke::new(1.2, stroke_color);
                                    ui.painter().line_segment(
                                        [
                                            egui::pos2(c.x - 3.5, c.y - 1.5),
                                            egui::pos2(c.x, c.y + 2.0),
                                        ],
                                        stroke,
                                    );
                                    ui.painter().line_segment(
                                        [
                                            egui::pos2(c.x, c.y + 2.0),
                                            egui::pos2(c.x + 3.5, c.y - 1.5),
                                        ],
                                        stroke,
                                    );
                                    if down_resp.clicked() && idx + 1 < current_order.len() {
                                        swap_pair = Some((idx, idx + 1));
                                    }
                                }

                                // Active Checkbox
                                let enabled_key = format!("{}_enabled", item_id);
                                let mut enabled = state
                                    .values
                                    .get(&enabled_key)
                                    .and_then(|v| v.as_bool())
                                    .unwrap_or(item_cfg.enabled);
                                if ui.checkbox(&mut enabled, "").changed() {
                                    state
                                        .values
                                        .insert(enabled_key.clone(), DynamicValue::Bool(enabled));
                                    state.values.insert(
                                        format!("{}_{}_enabled", id, item_id),
                                        DynamicValue::Bool(enabled),
                                    );
                                    changed = true;
                                }

                                // Item label with fixed width slot for crisp zero-stair-step alignment
                                ui.add_sized(
                                    egui::vec2(max_label_w, ui.spacing().interact_size.y),
                                    egui::Label::new(
                                        egui::RichText::new(item_cfg.label_for_lang(lang))
                                            .size(12.0)
                                            .color(if enabled {
                                                palette.text_primary
                                            } else {
                                                palette.text_muted
                                            }),
                                    )
                                    .truncate(),
                                );

                                // Parameter Value (Interactive Slider when enabled, disabled track when disabled)
                                let val_key = format!("{}_value", item_id);
                                let mut val = state
                                    .values
                                    .get(&val_key)
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or(item_cfg.default);
                                let step = item_cfg.step.unwrap_or(1.0);
                                let suffix = if let Some(ref u) = item_cfg.unit {
                                    format!(" {}", u)
                                } else {
                                    String::new()
                                };

                                let is_integer = step >= 1.0;
                                let mut slider =
                                    egui::Slider::new(&mut val, item_cfg.min..=item_cfg.max)
                                        .step_by(step)
                                        .show_value(true);

                                if is_integer {
                                    let s_clone = suffix.clone();
                                    slider = slider
                                        .custom_formatter(move |v, _| {
                                            format!("{:.0}{}", v, s_clone)
                                        })
                                        .custom_parser({
                                            let s_clone = suffix.clone();
                                            move |s| {
                                                s.trim_end_matches(&s_clone)
                                                    .trim()
                                                    .parse::<f64>()
                                                    .ok()
                                            }
                                        });
                                } else {
                                    let s_clone = suffix.clone();
                                    slider = slider
                                        .custom_formatter(move |v, _| {
                                            format!("{:.1}{}", v, s_clone)
                                        })
                                        .custom_parser({
                                            let s_clone = suffix.clone();
                                            move |s| {
                                                s.trim_end_matches(&s_clone)
                                                    .trim()
                                                    .parse::<f64>()
                                                    .ok()
                                            }
                                        });
                                }

                                let slider_resp = ui
                                    .add_enabled_ui(enabled, |ui| {
                                        ui.add_sized(
                                            egui::vec2(
                                                ui.available_width(),
                                                ui.spacing().interact_size.y,
                                            ),
                                            slider,
                                        )
                                    })
                                    .inner;

                                let mut val_changed = slider_resp.changed() && enabled;

                                let mut wheel_ticks: f32 = 0.0;
                                ui.input(|i| {
                                    for event in &i.events {
                                        if let egui::Event::MouseWheel { delta, .. } = event {
                                            if delta.y > 0.0 {
                                                wheel_ticks += 1.0;
                                            } else if delta.y < 0.0 {
                                                wheel_ticks -= 1.0;
                                            }
                                        }
                                    }
                                });

                                if wheel_ticks != 0.0 && slider_resp.hovered() && enabled {
                                    let wheel_step = if is_integer { 1.0 } else { 0.1 };
                                    let new_val = (val + (wheel_ticks as f64) * wheel_step)
                                        .clamp(item_cfg.min, item_cfg.max);
                                    let new_val = if is_integer {
                                        new_val.round()
                                    } else {
                                        (new_val * 10.0).round() / 10.0
                                    };
                                    if (new_val - val).abs() > f64::EPSILON {
                                        val = new_val;
                                        val_changed = true;
                                    }
                                }

                                if val_changed {
                                    state
                                        .values
                                        .insert(val_key.clone(), DynamicValue::Number(val));
                                    state.values.insert(
                                        format!("{}_{}_value", id, item_id),
                                        DynamicValue::Number(val),
                                    );
                                    changed = true;
                                }
                            });
                        }
                    }

                    if let Some((i1, i2)) = swap_pair {
                        current_order.swap(i1, i2);
                        state
                            .values
                            .insert(order_key, DynamicValue::StringList(current_order.clone()));
                        state
                            .values
                            .insert(id.clone(), DynamicValue::StringList(current_order.clone()));
                        if let Some(cfg_path) = config_path {
                            let mut cfg =
                                core_contracts::CompanionConfig::load_or_default(cfg_path);
                            cfg.modifier_stack_orders
                                .insert(format!("{}:{}", plugin_id, id), current_order.clone());
                            cfg.modifier_stack_orders
                                .insert(id.clone(), current_order.clone());
                            if let Err(e) = cfg.save(cfg_path) {
                                warn!("Failed to persist modifier stack order to config: {}", e);
                            }
                        }
                        changed = true;
                    }
                });
            }
        }

        changed
    }

    /// Renders a pure vector HSV hue wheel without bitmaps or tofu glyphs
    fn draw_vector_hue_wheel(
        ui: &mut egui::Ui,
        active_hue_deg: f32,
        show_pointer: bool,
        palette: &ThemePalette,
    ) {
        let size = Vec2::new(140.0, 140.0);
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        let center = rect.center();
        let outer_radius = 60.0;
        let inner_radius = 42.0;

        let painter = ui.painter();
        let num_segments = 36;
        let step_rad = std::f32::consts::TAU / num_segments as f32;

        for i in 0..num_segments {
            let angle1 = i as f32 * step_rad;
            let angle2 = (i + 1) as f32 * step_rad;
            let mid_angle_deg = ((i as f32 + 0.5) / num_segments as f32) * 360.0;
            let seg_color = hsv_to_rgb(mid_angle_deg, 0.95, 0.95);

            let p1 = center + Vec2::new(angle1.cos(), angle1.sin()) * inner_radius;
            let p2 = center + Vec2::new(angle1.cos(), angle1.sin()) * outer_radius;
            let p3 = center + Vec2::new(angle2.cos(), angle2.sin()) * outer_radius;
            let p4 = center + Vec2::new(angle2.cos(), angle2.sin()) * inner_radius;

            painter.add(egui::Shape::convex_polygon(
                vec![p1, p2, p3, p4],
                seg_color,
                Stroke::NONE,
            ));
        }

        // Inner circle background
        painter.circle_filled(center, inner_radius - 1.0, palette.panel_fill);
        painter.circle_stroke(
            center,
            outer_radius,
            Stroke::new(1.0, palette.border_stroke),
        );
        painter.circle_stroke(
            center,
            inner_radius,
            Stroke::new(1.0, palette.border_stroke),
        );

        // Vector pointer
        if show_pointer {
            let rad = active_hue_deg.to_radians();
            let dir = Vec2::new(rad.cos(), rad.sin());
            let pointer_center = center + dir * ((inner_radius + outer_radius) * 0.5);

            painter.circle_filled(pointer_center, 4.5, Color32::WHITE);
            painter.circle_stroke(
                pointer_center,
                4.5,
                Stroke::new(1.5, Color32::from_rgb(20, 20, 20)),
            );
        }
    }

    /// Renders a vector status banner strictly complying with Vector-First UI (no emojis)
    fn draw_vector_banner(
        ui: &mut egui::Ui,
        msg: &str,
        style: BannerStyle,
        palette: &ThemePalette,
    ) {
        let (bg_color, stroke_color) = match style {
            BannerStyle::Info => (
                Color32::from_rgba_unmultiplied(26, 54, 93, 200),
                palette.accent_blue,
            ),
            BannerStyle::Warning => (
                Color32::from_rgba_unmultiplied(120, 53, 15, 200),
                Color32::from_rgb(245, 158, 11),
            ),
            BannerStyle::Error => (
                Color32::from_rgba_unmultiplied(127, 29, 29, 200),
                Color32::from_rgb(239, 68, 68),
            ),
            BannerStyle::Success => (
                Color32::from_rgba_unmultiplied(20, 83, 45, 200),
                Color32::from_rgb(34, 197, 94),
            ),
        };

        egui::Frame::new()
            .fill(bg_color)
            .stroke(Stroke::new(1.0, stroke_color))
            .corner_radius(4.0)
            .inner_margin(egui::Margin::symmetric(8, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Vector icon (circle with accent point)
                    let (icon_rect, _) =
                        ui.allocate_exact_size(Vec2::new(14.0, 14.0), Sense::hover());
                    ui.painter().circle_stroke(
                        icon_rect.center(),
                        5.5,
                        Stroke::new(1.5, stroke_color),
                    );
                    ui.painter()
                        .circle_filled(icon_rect.center(), 2.0, stroke_color);

                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(msg)
                                .size(11.0)
                                .color(palette.text_primary),
                        )
                        .wrap_mode(egui::TextWrapMode::Wrap),
                    );
                });
            });
    }
}

/// Helper: parses #RRGGBB hex string
pub fn parse_hex_color(hex: &str) -> Option<Color32> {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        let r = u8::from_str_radix(&clean[0..2], 16).ok()?;
        let g = u8::from_str_radix(&clean[2..4], 16).ok()?;
        let b = u8::from_str_radix(&clean[4..6], 16).ok()?;
        Some(Color32::from_rgb(r, g, b))
    } else if clean.len() == 3 {
        let r = u8::from_str_radix(&clean[0..1], 16).ok()?;
        let g = u8::from_str_radix(&clean[1..2], 16).ok()?;
        let b = u8::from_str_radix(&clean[2..3], 16).ok()?;
        Some(Color32::from_rgb(r * 17, g * 17, b * 17))
    } else {
        None
    }
}

pub fn extract_extrude_params(values: &HashMap<String, DynamicValue>) -> ExtrudeParams {
    let mut params = ExtrudeParams::default();

    for (k, v) in values {
        let k_lower = k.to_lowercase();
        if let Some(num) = v.as_f64() {
            if k_lower.ends_with("_light_angle")
                || k_lower == "light_angle"
                || k_lower == "light_angle_deg"
            {
                params.light_angle_deg = num as f32;
            } else if k_lower.ends_with("_light_intensity") || k_lower == "light_intensity" {
                params.light_intensity = num as f32;
            } else if k_lower.ends_with("_angle") || k_lower == "angle" || k_lower == "angle_deg" {
                params.angle_deg = num as f32;
            } else if k_lower.ends_with("_distance") || k_lower == "distance" {
                params.distance = num as f32;
            } else if k_lower.ends_with("_bevel_radius") || k_lower == "bevel_radius" {
                params.bevel_radius = num as f32;
            } else if k_lower.ends_with("_bevel_highlight")
                || k_lower == "bevel_highlight"
                || k_lower == "highlight"
            {
                params.bevel_highlight = num as f32;
            } else if k_lower.ends_with("_shadow_blur") || k_lower == "shadow_blur" {
                params.shadow_blur = num as f32;
            } else if k_lower.ends_with("_shadow_opacity") || k_lower == "shadow_opacity" {
                params.shadow_opacity = num as f32;
            } else if k_lower.ends_with("_shadow_distance") || k_lower == "shadow_distance" {
                params.shadow_distance = num as f32;
            } else if k_lower.ends_with("_perspective") || k_lower == "perspective" {
                params.perspective = num as f32;
            }
        } else if let Some(s) = v.as_str() {
            if (k_lower.ends_with("_start_color")
                || k_lower.ends_with("body_start")
                || k_lower.ends_with("_start"))
                && s.starts_with('#')
            {
                if let Some(c) = parse_hex_color(s) {
                    params.body_gradient_start = image::Rgba([c.r(), c.g(), c.b(), 255]);
                }
            } else if (k_lower.ends_with("_end_color")
                || k_lower.ends_with("body_end")
                || k_lower.ends_with("_end"))
                && s.starts_with('#')
            {
                if let Some(c) = parse_hex_color(s) {
                    params.body_gradient_end = image::Rgba([c.r(), c.g(), c.b(), 255]);
                }
            } else if (k_lower.ends_with("front_color") || k_lower == "front") && s.starts_with('#')
            {
                if let Some(c) = parse_hex_color(s) {
                    params.front_color = Some(image::Rgba([c.r(), c.g(), c.b(), 255]));
                }
            }
        } else if let Some(b) = v.as_bool() {
            if (k_lower.ends_with("shadow_enabled") || k_lower == "shadow_enabled") && !b {
                params.shadow_opacity = 0.0;
            }
        }
    }

    params
}

pub fn hash_extrude_params(p: &ExtrudeParams, scale: f32) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    p.angle_deg.to_bits().hash(&mut hasher);
    p.distance.to_bits().hash(&mut hasher);
    if let Some(fc) = p.front_color {
        fc.0.hash(&mut hasher);
    }
    p.body_gradient_start.0.hash(&mut hasher);
    p.body_gradient_end.0.hash(&mut hasher);
    p.bevel_radius.to_bits().hash(&mut hasher);
    p.bevel_highlight.to_bits().hash(&mut hasher);
    p.light_angle_deg.to_bits().hash(&mut hasher);
    p.light_intensity.to_bits().hash(&mut hasher);
    p.shadow_blur.to_bits().hash(&mut hasher);
    p.shadow_opacity.to_bits().hash(&mut hasher);
    p.shadow_distance.to_bits().hash(&mut hasher);
    p.perspective.to_bits().hash(&mut hasher);
    scale.to_bits().hash(&mut hasher);
    hasher.finish()
}

/// Helper: calculates hue in degrees (0.0..360.0) from Color32
pub fn rgb_to_hue_degrees(color: Color32) -> f32 {
    let r = color.r() as f32 / 255.0;
    let g = color.g() as f32 / 255.0;
    let b = color.b() as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    if delta < 1e-5 {
        return 0.0;
    }

    let hue = if (max - r).abs() < 1e-5 {
        ((g - b) / delta) % 6.0
    } else if (max - g).abs() < 1e-5 {
        ((b - r) / delta) + 2.0
    } else {
        ((r - g) / delta) + 4.0
    };

    let mut deg = hue * 60.0;
    if deg < 0.0 {
        deg += 360.0;
    }
    deg
}

/// Helper: calculates Color32 from HSV
pub fn hsv_to_rgb(h_deg: f32, s: f32, v: f32) -> Color32 {
    let h = (h_deg % 360.0) / 60.0;
    let c = v * s;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = if (0.0..1.0).contains(&h) {
        (c, x, 0.0)
    } else if (1.0..2.0).contains(&h) {
        (x, c, 0.0)
    } else if (2.0..3.0).contains(&h) {
        (0.0, c, x)
    } else if (3.0..4.0).contains(&h) {
        (0.0, x, c)
    } else if (4.0..5.0).contains(&h) {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Color32::from_rgb(
        ((r1 + m) * 255.0).round() as u8,
        ((g1 + m) * 255.0).round() as u8,
        ((b1 + m) * 255.0).round() as u8,
    )
}

/// Named color entry for perceptual color matching
#[derive(Debug, Clone, Copy)]
pub struct NamedColorEntry {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub en: &'static str,
    pub ru: &'static str,
}

pub static NAMED_COLORS: &[NamedColorEntry] = &[
    // --- Classical Fine Art Pigments ---
    NamedColorEntry {
        r: 18,
        g: 10,
        b: 143,
        en: "Ultramarine",
        ru: "Ультрамарин",
    },
    NamedColorEntry {
        r: 233,
        g: 116,
        b: 81,
        en: "Burnt Sienna",
        ru: "Жжёная сиена",
    },
    NamedColorEntry {
        r: 214,
        g: 138,
        b: 89,
        en: "Raw Sienna",
        ru: "Натуральная сиена",
    },
    NamedColorEntry {
        r: 138,
        g: 51,
        b: 36,
        en: "Burnt Umber",
        ru: "Жжёная умбра",
    },
    NamedColorEntry {
        r: 115,
        g: 74,
        b: 18,
        en: "Raw Umber",
        ru: "Натуральная умбра",
    },
    NamedColorEntry {
        r: 203,
        g: 157,
        b: 68,
        en: "Yellow Ochre",
        ru: "Жёлтая охра",
    },
    NamedColorEntry {
        r: 150,
        g: 0,
        b: 24,
        en: "Carmine",
        ru: "Кармин",
    },
    NamedColorEntry {
        r: 38,
        g: 97,
        b: 156,
        en: "Lapis Lazuli",
        ru: "Лазурит",
    },
    NamedColorEntry {
        r: 0,
        g: 71,
        b: 171,
        en: "Cobalt Blue",
        ru: "Кобальт синий",
    },
    NamedColorEntry {
        r: 42,
        g: 82,
        b: 190,
        en: "Cerulean Blue",
        ru: "Церулеум",
    },
    NamedColorEntry {
        r: 64,
        g: 130,
        b: 109,
        en: "Viridian",
        ru: "Виридиан",
    },
    NamedColorEntry {
        r: 227,
        g: 0,
        b: 34,
        en: "Cadmium Red",
        ru: "Кадмий красный",
    },
    NamedColorEntry {
        r: 255,
        g: 246,
        b: 0,
        en: "Cadmium Yellow",
        ru: "Кадмий жёлтый",
    },
    NamedColorEntry {
        r: 237,
        g: 135,
        b: 34,
        en: "Cadmium Orange",
        ru: "Кадмий оранжевый",
    },
    NamedColorEntry {
        r: 0,
        g: 15,
        b: 137,
        en: "Phthalo Blue",
        ru: "Фталоцианиновый синий",
    },
    NamedColorEntry {
        r: 18,
        g: 53,
        b: 36,
        en: "Phthalo Green",
        ru: "Фталоцианиновый зелёный",
    },
    NamedColorEntry {
        r: 0,
        g: 49,
        b: 83,
        en: "Prussian Blue",
        ru: "Берлинская лазурь",
    },
    NamedColorEntry {
        r: 200,
        g: 8,
        b: 21,
        en: "Venetian Red",
        ru: "Венецианский красный",
    },
    NamedColorEntry {
        r: 227,
        g: 66,
        b: 52,
        en: "Vermilion",
        ru: "Киноварь",
    },
    NamedColorEntry {
        r: 250,
        g: 218,
        b: 94,
        en: "Naples Yellow",
        ru: "Неаполитанский жёлтый",
    },
    NamedColorEntry {
        r: 80,
        g: 200,
        b: 120,
        en: "Emerald Green",
        ru: "Изумрудный зелёный",
    },
    NamedColorEntry {
        r: 227,
        g: 38,
        b: 54,
        en: "Alizarin Crimson",
        ru: "Краплак (Ализарин)",
    },
    NamedColorEntry {
        r: 11,
        g: 218,
        b: 81,
        en: "Malachite",
        ru: "Малахит",
    },
    NamedColorEntry {
        r: 254,
        g: 254,
        b: 254,
        en: "Titanium White",
        ru: "Титановые белила",
    },
    NamedColorEntry {
        r: 252,
        g: 252,
        b: 250,
        en: "Zinc White",
        ru: "Цинковые белила",
    },
    NamedColorEntry {
        r: 35,
        g: 31,
        b: 32,
        en: "Bone Black",
        ru: "Кость жжёная",
    },
    NamedColorEntry {
        r: 46,
        g: 44,
        b: 42,
        en: "Lamp Black",
        ru: "Сажа газовая",
    },
    // --- Standard W3C / CSS3 Named Colors ---
    NamedColorEntry {
        r: 0,
        g: 0,
        b: 0,
        en: "Black",
        ru: "Чёрный",
    },
    NamedColorEntry {
        r: 255,
        g: 255,
        b: 255,
        en: "White",
        ru: "Белый",
    },
    NamedColorEntry {
        r: 255,
        g: 0,
        b: 0,
        en: "Red",
        ru: "Красный",
    },
    NamedColorEntry {
        r: 0,
        g: 255,
        b: 0,
        en: "Lime",
        ru: "Лайм",
    },
    NamedColorEntry {
        r: 0,
        g: 0,
        b: 255,
        en: "Blue",
        ru: "Синий",
    },
    NamedColorEntry {
        r: 255,
        g: 255,
        b: 0,
        en: "Yellow",
        ru: "Жёлтый",
    },
    NamedColorEntry {
        r: 0,
        g: 255,
        b: 255,
        en: "Cyan",
        ru: "Циан (Голубой)",
    },
    NamedColorEntry {
        r: 255,
        g: 0,
        b: 255,
        en: "Magenta",
        ru: "Маджента (Пурпурный)",
    },
    NamedColorEntry {
        r: 192,
        g: 192,
        b: 192,
        en: "Silver",
        ru: "Серебристый",
    },
    NamedColorEntry {
        r: 128,
        g: 128,
        b: 128,
        en: "Gray",
        ru: "Серый",
    },
    NamedColorEntry {
        r: 128,
        g: 0,
        b: 0,
        en: "Maroon",
        ru: "Бордовый",
    },
    NamedColorEntry {
        r: 128,
        g: 128,
        b: 0,
        en: "Olive",
        ru: "Оливковый",
    },
    NamedColorEntry {
        r: 0,
        g: 128,
        b: 0,
        en: "Green",
        ru: "Зелёный",
    },
    NamedColorEntry {
        r: 128,
        g: 0,
        b: 128,
        en: "Purple",
        ru: "Фиолетовый",
    },
    NamedColorEntry {
        r: 0,
        g: 128,
        b: 128,
        en: "Teal",
        ru: "Морская волна",
    },
    NamedColorEntry {
        r: 0,
        g: 0,
        b: 128,
        en: "Navy",
        ru: "Тёмно-синий",
    },
    NamedColorEntry {
        r: 240,
        g: 248,
        b: 255,
        en: "Alice Blue",
        ru: "Серо-голубой",
    },
    NamedColorEntry {
        r: 250,
        g: 235,
        b: 215,
        en: "Antique White",
        ru: "Античный белый",
    },
    NamedColorEntry {
        r: 127,
        g: 255,
        b: 212,
        en: "Aquamarine",
        ru: "Аквамарин",
    },
    NamedColorEntry {
        r: 240,
        g: 255,
        b: 255,
        en: "Azure",
        ru: "Лазурный",
    },
    NamedColorEntry {
        r: 245,
        g: 245,
        b: 220,
        en: "Beige",
        ru: "Бежевый",
    },
    NamedColorEntry {
        r: 255,
        g: 228,
        b: 196,
        en: "Bisque",
        ru: "Бисквитный",
    },
    NamedColorEntry {
        r: 255,
        g: 235,
        b: 205,
        en: "Blanched Almond",
        ru: "Миндальный",
    },
    NamedColorEntry {
        r: 138,
        g: 43,
        b: 226,
        en: "Blue Violet",
        ru: "Сине-фиолетовый",
    },
    NamedColorEntry {
        r: 165,
        g: 42,
        b: 42,
        en: "Brown",
        ru: "Коричневый",
    },
    NamedColorEntry {
        r: 222,
        g: 184,
        b: 135,
        en: "Burlywood",
        ru: "Древесный",
    },
    NamedColorEntry {
        r: 95,
        g: 158,
        b: 160,
        en: "Cadet Blue",
        ru: "Кадетский синий",
    },
    NamedColorEntry {
        r: 127,
        g: 255,
        b: 0,
        en: "Chartreuse",
        ru: "Шартрез",
    },
    NamedColorEntry {
        r: 210,
        g: 105,
        b: 30,
        en: "Chocolate",
        ru: "Шоколадный",
    },
    NamedColorEntry {
        r: 255,
        g: 127,
        b: 80,
        en: "Coral",
        ru: "Коралл",
    },
    NamedColorEntry {
        r: 100,
        g: 149,
        b: 237,
        en: "Cornflower Blue",
        ru: "Васильковый",
    },
    NamedColorEntry {
        r: 255,
        g: 248,
        b: 220,
        en: "Cornsilk",
        ru: "Кукурузный",
    },
    NamedColorEntry {
        r: 220,
        g: 20,
        b: 60,
        en: "Crimson",
        ru: "Малиновый",
    },
    NamedColorEntry {
        r: 0,
        g: 0,
        b: 139,
        en: "Dark Blue",
        ru: "Тёмно-синий",
    },
    NamedColorEntry {
        r: 0,
        g: 139,
        b: 139,
        en: "Dark Cyan",
        ru: "Тёмный циан",
    },
    NamedColorEntry {
        r: 184,
        g: 134,
        b: 11,
        en: "Dark Goldenrod",
        ru: "Тёмный золотарник",
    },
    NamedColorEntry {
        r: 169,
        g: 169,
        b: 169,
        en: "Dark Gray",
        ru: "Тёмно-серый",
    },
    NamedColorEntry {
        r: 0,
        g: 100,
        b: 0,
        en: "Dark Green",
        ru: "Тёмно-зелёный",
    },
    NamedColorEntry {
        r: 189,
        g: 183,
        b: 107,
        en: "Dark Khaki",
        ru: "Тёмный хаки",
    },
    NamedColorEntry {
        r: 139,
        g: 0,
        b: 139,
        en: "Dark Magenta",
        ru: "Тёмная маджента",
    },
    NamedColorEntry {
        r: 85,
        g: 107,
        b: 47,
        en: "Dark Olive Green",
        ru: "Тёмно-оливковый",
    },
    NamedColorEntry {
        r: 255,
        g: 140,
        b: 0,
        en: "Dark Orange",
        ru: "Тёмно-оранжевый",
    },
    NamedColorEntry {
        r: 153,
        g: 50,
        b: 204,
        en: "Dark Orchid",
        ru: "Тёмная орхидея",
    },
    NamedColorEntry {
        r: 139,
        g: 0,
        b: 0,
        en: "Dark Red",
        ru: "Тёмно-красный",
    },
    NamedColorEntry {
        r: 233,
        g: 150,
        b: 122,
        en: "Dark Salmon",
        ru: "Тёмно-лососевый",
    },
    NamedColorEntry {
        r: 143,
        g: 188,
        b: 143,
        en: "Dark Sea Green",
        ru: "Серо-зелёный",
    },
    NamedColorEntry {
        r: 72,
        g: 61,
        b: 139,
        en: "Dark Slate Blue",
        ru: "Тёмный серо-синий",
    },
    NamedColorEntry {
        r: 47,
        g: 79,
        b: 79,
        en: "Dark Slate Gray",
        ru: "Тёмный шиферно-серый",
    },
    NamedColorEntry {
        r: 0,
        g: 206,
        b: 209,
        en: "Dark Turquoise",
        ru: "Тёмно-бирюзовый",
    },
    NamedColorEntry {
        r: 148,
        g: 0,
        b: 211,
        en: "Dark Violet",
        ru: "Тёмно-фиолетовый",
    },
    NamedColorEntry {
        r: 255,
        g: 20,
        b: 147,
        en: "Deep Pink",
        ru: "Ярко-розовый",
    },
    NamedColorEntry {
        r: 0,
        g: 191,
        b: 255,
        en: "Deep Sky Blue",
        ru: "Глубокий небесный",
    },
    NamedColorEntry {
        r: 105,
        g: 105,
        b: 105,
        en: "Dim Gray",
        ru: "Тускло-серый",
    },
    NamedColorEntry {
        r: 30,
        g: 144,
        b: 255,
        en: "Dodger Blue",
        ru: "Васильковый яркий",
    },
    NamedColorEntry {
        r: 178,
        g: 34,
        b: 34,
        en: "Firebrick",
        ru: "Огнеупорный кирпич",
    },
    NamedColorEntry {
        r: 255,
        g: 250,
        b: 240,
        en: "Floral White",
        ru: "Цветочный белый",
    },
    NamedColorEntry {
        r: 34,
        g: 139,
        b: 34,
        en: "Forest Green",
        ru: "Лесной зелёный",
    },
    NamedColorEntry {
        r: 220,
        g: 220,
        b: 220,
        en: "Gainsboro",
        ru: "Гейнсборо",
    },
    NamedColorEntry {
        r: 248,
        g: 248,
        b: 255,
        en: "Ghost White",
        ru: "Призрачно-белый",
    },
    NamedColorEntry {
        r: 255,
        g: 215,
        b: 0,
        en: "Gold",
        ru: "Золотой",
    },
    NamedColorEntry {
        r: 218,
        g: 165,
        b: 32,
        en: "Goldenrod",
        ru: "Золотарник",
    },
    NamedColorEntry {
        r: 173,
        g: 255,
        b: 47,
        en: "Green Yellow",
        ru: "Жёлто-зелёный",
    },
    NamedColorEntry {
        r: 240,
        g: 255,
        b: 240,
        en: "Honeydew",
        ru: "Медовая роса",
    },
    NamedColorEntry {
        r: 255,
        g: 105,
        b: 180,
        en: "Hot Pink",
        ru: "Насыщенно-розовый",
    },
    NamedColorEntry {
        r: 205,
        g: 92,
        b: 92,
        en: "Indian Red",
        ru: "Индийский красный",
    },
    NamedColorEntry {
        r: 75,
        g: 0,
        b: 130,
        en: "Indigo",
        ru: "Индиго",
    },
    NamedColorEntry {
        r: 255,
        g: 255,
        b: 240,
        en: "Ivory",
        ru: "Слоновая кость",
    },
    NamedColorEntry {
        r: 240,
        g: 230,
        b: 140,
        en: "Khaki",
        ru: "Хаки",
    },
    NamedColorEntry {
        r: 230,
        g: 230,
        b: 250,
        en: "Lavender",
        ru: "Лаванда",
    },
    NamedColorEntry {
        r: 255,
        g: 240,
        b: 245,
        en: "Lavender Blush",
        ru: "Розовая лаванда",
    },
    NamedColorEntry {
        r: 124,
        g: 252,
        b: 0,
        en: "Lawn Green",
        ru: "Газонный зелёный",
    },
    NamedColorEntry {
        r: 255,
        g: 250,
        b: 205,
        en: "Lemon Chiffon",
        ru: "Лимонный шифон",
    },
    NamedColorEntry {
        r: 173,
        g: 216,
        b: 230,
        en: "Light Blue",
        ru: "Светло-синий",
    },
    NamedColorEntry {
        r: 240,
        g: 128,
        b: 128,
        en: "Light Coral",
        ru: "Светло-коралловый",
    },
    NamedColorEntry {
        r: 224,
        g: 255,
        b: 255,
        en: "Light Cyan",
        ru: "Светлый циан",
    },
    NamedColorEntry {
        r: 250,
        g: 250,
        b: 210,
        en: "Light Goldenrod Yellow",
        ru: "Светло-золотарниковый",
    },
    NamedColorEntry {
        r: 211,
        g: 211,
        b: 211,
        en: "Light Gray",
        ru: "Светло-серый",
    },
    NamedColorEntry {
        r: 144,
        g: 238,
        b: 144,
        en: "Light Green",
        ru: "Светло-зелёный",
    },
    NamedColorEntry {
        r: 255,
        g: 182,
        b: 193,
        en: "Light Pink",
        ru: "Светло-розовый",
    },
    NamedColorEntry {
        r: 255,
        g: 160,
        b: 122,
        en: "Light Salmon",
        ru: "Светло-лососевый",
    },
    NamedColorEntry {
        r: 32,
        g: 178,
        b: 170,
        en: "Light Sea Green",
        ru: "Светлый морской зелёный",
    },
    NamedColorEntry {
        r: 135,
        g: 206,
        b: 250,
        en: "Light Sky Blue",
        ru: "Светло-небесный",
    },
    NamedColorEntry {
        r: 119,
        g: 136,
        b: 153,
        en: "Light Slate Gray",
        ru: "Светлый шиферно-серый",
    },
    NamedColorEntry {
        r: 176,
        g: 196,
        b: 222,
        en: "Light Steel Blue",
        ru: "Светло-стальной",
    },
    NamedColorEntry {
        r: 255,
        g: 255,
        b: 224,
        en: "Light Yellow",
        ru: "Светло-жёлтый",
    },
    NamedColorEntry {
        r: 50,
        g: 205,
        b: 50,
        en: "Lime Green",
        ru: "Лаймово-зелёный",
    },
    NamedColorEntry {
        r: 250,
        g: 240,
        b: 230,
        en: "Linen",
        ru: "Льняной",
    },
    NamedColorEntry {
        r: 102,
        g: 205,
        b: 170,
        en: "Medium Aquamarine",
        ru: "Умеренный аквамарин",
    },
    NamedColorEntry {
        r: 0,
        g: 0,
        b: 205,
        en: "Medium Blue",
        ru: "Умеренно-синий",
    },
    NamedColorEntry {
        r: 186,
        g: 85,
        b: 211,
        en: "Medium Orchid",
        ru: "Умеренная орхидея",
    },
    NamedColorEntry {
        r: 147,
        g: 112,
        b: 219,
        en: "Medium Purple",
        ru: "Умеренно-пурпурный",
    },
    NamedColorEntry {
        r: 60,
        g: 179,
        b: 113,
        en: "Medium Sea Green",
        ru: "Умеренный морской зелёный",
    },
    NamedColorEntry {
        r: 123,
        g: 104,
        b: 238,
        en: "Medium Slate Blue",
        ru: "Умеренный серо-синий",
    },
    NamedColorEntry {
        r: 0,
        g: 250,
        b: 154,
        en: "Medium Spring Green",
        ru: "Весенний зелёный",
    },
    NamedColorEntry {
        r: 72,
        g: 209,
        b: 204,
        en: "Medium Turquoise",
        ru: "Умеренно-бирюзовый",
    },
    NamedColorEntry {
        r: 199,
        g: 21,
        b: 133,
        en: "Medium Violet Red",
        ru: "Умеренно-фиолетово-красный",
    },
    NamedColorEntry {
        r: 25,
        g: 25,
        b: 112,
        en: "Midnight Blue",
        ru: "Полночный синий",
    },
    NamedColorEntry {
        r: 245,
        g: 255,
        b: 250,
        en: "Mint Cream",
        ru: "Мятный крем",
    },
    NamedColorEntry {
        r: 255,
        g: 228,
        b: 181,
        en: "Moccasin",
        ru: "Мокасиновый",
    },
    NamedColorEntry {
        r: 255,
        g: 222,
        b: 173,
        en: "Navajo White",
        ru: "Белый навахо",
    },
    NamedColorEntry {
        r: 253,
        g: 245,
        b: 230,
        en: "Old Lace",
        ru: "Старое кружево",
    },
    NamedColorEntry {
        r: 107,
        g: 142,
        b: 35,
        en: "Olive Drab",
        ru: "Болотный",
    },
    NamedColorEntry {
        r: 255,
        g: 165,
        b: 0,
        en: "Orange",
        ru: "Оранжевый",
    },
    NamedColorEntry {
        r: 255,
        g: 69,
        b: 0,
        en: "Orange Red",
        ru: "Красно-оранжевый",
    },
    NamedColorEntry {
        r: 218,
        g: 112,
        b: 214,
        en: "Orchid",
        ru: "Орхидея",
    },
    NamedColorEntry {
        r: 238,
        g: 232,
        b: 170,
        en: "Pale Goldenrod",
        ru: "Бледно-золотарниковый",
    },
    NamedColorEntry {
        r: 152,
        g: 251,
        b: 152,
        en: "Pale Green",
        ru: "Бледно-зелёный",
    },
    NamedColorEntry {
        r: 175,
        g: 238,
        b: 238,
        en: "Pale Turquoise",
        ru: "Бледно-бирюзовый",
    },
    NamedColorEntry {
        r: 219,
        g: 112,
        b: 147,
        en: "Pale Violet Red",
        ru: "Бледно-фиолетово-красный",
    },
    NamedColorEntry {
        r: 255,
        g: 239,
        b: 213,
        en: "Papaya Whip",
        ru: "Папайя",
    },
    NamedColorEntry {
        r: 255,
        g: 218,
        b: 185,
        en: "Peach Puff",
        ru: "Персиковый",
    },
    NamedColorEntry {
        r: 205,
        g: 133,
        b: 63,
        en: "Peru",
        ru: "Перу",
    },
    NamedColorEntry {
        r: 255,
        g: 192,
        b: 203,
        en: "Pink",
        ru: "Розовый",
    },
    NamedColorEntry {
        r: 221,
        g: 160,
        b: 221,
        en: "Plum",
        ru: "Сливовый",
    },
    NamedColorEntry {
        r: 176,
        g: 224,
        b: 230,
        en: "Powder Blue",
        ru: "Пороховой синий",
    },
    NamedColorEntry {
        r: 188,
        g: 143,
        b: 143,
        en: "Rosy Brown",
        ru: "Розовато-коричневый",
    },
    NamedColorEntry {
        r: 65,
        g: 105,
        b: 225,
        en: "Royal Blue",
        ru: "Королевский синий",
    },
    NamedColorEntry {
        r: 139,
        g: 69,
        b: 19,
        en: "Saddle Brown",
        ru: "Кожано-коричневый",
    },
    NamedColorEntry {
        r: 250,
        g: 128,
        b: 114,
        en: "Salmon",
        ru: "Лососевый",
    },
    NamedColorEntry {
        r: 244,
        g: 164,
        b: 96,
        en: "Sandy Brown",
        ru: "Песочно-коричневый",
    },
    NamedColorEntry {
        r: 46,
        g: 139,
        b: 87,
        en: "Sea Green",
        ru: "Морской зелёный",
    },
    NamedColorEntry {
        r: 255,
        g: 245,
        b: 238,
        en: "Seashell",
        ru: "Морская ракушка",
    },
    NamedColorEntry {
        r: 160,
        g: 82,
        b: 45,
        en: "Sienna",
        ru: "Сиена",
    },
    NamedColorEntry {
        r: 135,
        g: 206,
        b: 235,
        en: "Sky Blue",
        ru: "Небесно-голубой",
    },
    NamedColorEntry {
        r: 106,
        g: 90,
        b: 205,
        en: "Slate Blue",
        ru: "Серо-синий",
    },
    NamedColorEntry {
        r: 112,
        g: 128,
        b: 144,
        en: "Slate Gray",
        ru: "Шиферно-серый",
    },
    NamedColorEntry {
        r: 255,
        g: 250,
        b: 250,
        en: "Snow",
        ru: "Белоснежный",
    },
    NamedColorEntry {
        r: 0,
        g: 255,
        b: 127,
        en: "Spring Green",
        ru: "Ярко-весенний зелёный",
    },
    NamedColorEntry {
        r: 70,
        g: 130,
        b: 180,
        en: "Steel Blue",
        ru: "Стальной синий",
    },
    NamedColorEntry {
        r: 210,
        g: 180,
        b: 140,
        en: "Tan",
        ru: "Загар",
    },
    NamedColorEntry {
        r: 216,
        g: 191,
        b: 216,
        en: "Thistle",
        ru: "Чертополох",
    },
    NamedColorEntry {
        r: 255,
        g: 99,
        b: 71,
        en: "Tomato",
        ru: "Томатный",
    },
    NamedColorEntry {
        r: 64,
        g: 224,
        b: 208,
        en: "Turquoise",
        ru: "Бирюзовый",
    },
    NamedColorEntry {
        r: 238,
        g: 130,
        b: 238,
        en: "Violet",
        ru: "Фиолетовый",
    },
    NamedColorEntry {
        r: 245,
        g: 222,
        b: 179,
        en: "Wheat",
        ru: "Пшеничный",
    },
    NamedColorEntry {
        r: 245,
        g: 245,
        b: 245,
        en: "White Smoke",
        ru: "Дымчато-белый",
    },
    NamedColorEntry {
        r: 154,
        g: 205,
        b: 50,
        en: "Yellow Green",
        ru: "Жёлто-оливковый",
    },
];

/// Finds nearest named shade by weighted perceptual distance:
/// delta_c = sqrt(2*dr^2 + 4*dg^2 + 3*db^2)
pub fn find_nearest_named_color(r: i64, g: i64, b: i64, lang: Language) -> String {
    let r_clamped = r.clamp(0, 255);
    let g_clamped = g.clamp(0, 255);
    let b_clamped = b.clamp(0, 255);

    let mut best_entry = &NAMED_COLORS[0];
    let mut min_dist = i64::MAX;

    for entry in NAMED_COLORS {
        let dr = r_clamped - entry.r as i64;
        let dg = g_clamped - entry.g as i64;
        let db = b_clamped - entry.b as i64;
        let dist = 2 * dr * dr + 4 * dg * dg + 3 * db * db;
        if dist < min_dist {
            min_dist = dist;
            best_entry = entry;
        }
    }

    if matches!(lang, Language::Ru) {
        best_entry.ru.to_string()
    } else {
        best_entry.en.to_string()
    }
}

/// Simulates color vision deficiency (CVD) perception using the Brettel-Vienot model
pub fn simulate_cvd_color(r: f64, g: f64, b: f64, profile_index: i64) -> (u8, u8, u8) {
    let r_clamped = r.clamp(0.0, 255.0);
    let g_clamped = g.clamp(0.0, 255.0);
    let b_clamped = b.clamp(0.0, 255.0);

    let (sim_r, sim_g, sim_b) = match profile_index {
        1 => {
            // Deuteranopia (green-blind)
            let nr = 0.625 * r_clamped + 0.375 * g_clamped;
            let ng = 0.70 * r_clamped + 0.30 * g_clamped;
            let nb = 0.30 * g_clamped + 0.70 * b_clamped;
            (nr, ng, nb)
        }
        2 => {
            // Protanopia (red-blind)
            let nr = 0.56667 * r_clamped + 0.43333 * g_clamped;
            let ng = 0.55833 * r_clamped + 0.44167 * g_clamped;
            let nb = 0.24167 * g_clamped + 0.75833 * b_clamped;
            (nr, ng, nb)
        }
        3 => {
            // Tritanopia (blue-blind)
            let nr = 0.95 * r_clamped + 0.05 * g_clamped;
            let ng = 0.43333 * g_clamped + 0.56667 * b_clamped;
            let nb = 0.475 * g_clamped + 0.525 * b_clamped;
            (nr, ng, nb)
        }
        _ => (r_clamped, g_clamped, b_clamped),
    };

    (
        sim_r.round().clamp(0.0, 255.0) as u8,
        sim_g.round().clamp(0.0, 255.0) as u8,
        sim_b.round().clamp(0.0, 255.0) as u8,
    )
}

/// Simulates color vision deficiency and outputs standard #rrggbb hex string
pub fn simulate_cvd_hex(r: f64, g: f64, b: f64, profile_index: i64) -> String {
    let (sr, sg, sb) = simulate_cvd_color(r, g, b, profile_index);
    format!("#{sr:02x}{sg:02x}{sb:02x}")
}

/// Calculates standard WCAG 2.1 relative luminance and contrast ratio between two sRGB colors
pub fn calculate_wcag_contrast(r1: f64, g1: f64, b1: f64, r2: f64, g2: f64, b2: f64) -> f64 {
    fn srgb_to_linear(c: f64) -> f64 {
        let v = (c / 255.0).clamp(0.0, 1.0);
        if v <= 0.04045 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }
    let l1 =
        0.2126 * srgb_to_linear(r1) + 0.7152 * srgb_to_linear(g1) + 0.0722 * srgb_to_linear(b1);
    let l2 =
        0.2126 * srgb_to_linear(r2) + 0.7152 * srgb_to_linear(g2) + 0.0722 * srgb_to_linear(b2);
    let max = l1.max(l2);
    let min = l1.min(l2);
    (max + 0.05) / (min + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_wcag_contrast_symmetry_and_accuracy() {
        // Pure black vs pure white: standard maximum 21.0:1
        let ratio_bw = calculate_wcag_contrast(0.0, 0.0, 0.0, 255.0, 255.0, 255.0);
        assert!((ratio_bw - 21.0).abs() < 0.1);

        // Symmetry: contrast(A, B) == contrast(B, A)
        let ratio_wb = calculate_wcag_contrast(255.0, 255.0, 255.0, 0.0, 0.0, 0.0);
        assert_eq!(ratio_bw, ratio_wb);

        // Identical colors: 1.0:1
        let ratio_ident = calculate_wcag_contrast(100.0, 150.0, 200.0, 100.0, 150.0, 200.0);
        assert!((ratio_ident - 1.0).abs() < 0.001);

        // Royal Blue #2563eb (37, 99, 235) on White (255, 255, 255): ~5.17:1
        let ratio_blue_white = calculate_wcag_contrast(37.0, 99.0, 235.0, 255.0, 255.0, 255.0);
        let ratio_white_blue = calculate_wcag_contrast(255.0, 255.0, 255.0, 37.0, 99.0, 235.0);
        assert_eq!(ratio_blue_white, ratio_white_blue);
        assert!(ratio_blue_white > 4.5 && ratio_blue_white < 6.0);
    }

    #[test]
    fn test_parse_hex_color() {
        assert_eq!(
            parse_hex_color("#2563eb"),
            Some(Color32::from_rgb(0x25, 0x63, 0xeb))
        );
        assert_eq!(
            parse_hex_color("#fff"),
            Some(Color32::from_rgb(255, 255, 255))
        );
        assert_eq!(parse_hex_color("invalid"), None);
    }

    #[test]
    fn test_rgb_to_hue_degrees() {
        let red = Color32::from_rgb(255, 0, 0);
        let green = Color32::from_rgb(0, 255, 0);
        let blue = Color32::from_rgb(0, 0, 255);

        assert!((rgb_to_hue_degrees(red) - 0.0).abs() < 1.0);
        assert!((rgb_to_hue_degrees(green) - 120.0).abs() < 1.0);
        assert!((rgb_to_hue_degrees(blue) - 240.0).abs() < 1.0);
    }

    #[test]
    fn test_dynamic_window_state_rhai_roundtrip() {
        let cfg = InteractiveUiConfig {
            window_title: "Test Panel".to_string(),
            window_title_ru: Some("Тестовая панель".to_string()),
            default_width: Some(300.0),
            default_height: Some(400.0),
            script: None,
            widgets: vec![
                WidgetNode::ColorSlot {
                    id: "primary".to_string(),
                    label: "Color".to_string(),
                    label_ru: Some("Цвет".to_string()),
                    default_hex: Some("#123456".to_string()),
                    read_only: false,
                    selectable: true,
                    target_binding: Some("active_target".to_string()),
                },
                WidgetNode::Button {
                    id: "swap".to_string(),
                    label: "Swap".to_string(),
                    label_ru: Some("Поменять".to_string()),
                    icon: Some("swap".to_string()),
                },
                WidgetNode::Slider {
                    id: "size".to_string(),
                    label: "Size".to_string(),
                    label_ru: Some("Размер".to_string()),
                    min: 1.0,
                    max: 100.0,
                    step: Some(1.0),
                    default: 50.0,
                },
                WidgetNode::Toggle {
                    id: "enabled".to_string(),
                    label: "Enabled".to_string(),
                    label_ru: Some("Включено".to_string()),
                    default: true,
                },
            ],
            ipc_subscriptions: vec![],
        };

        let mut state = DynamicWindowState::new(&cfg);
        assert_eq!(
            state.values.get("primary").unwrap().as_str(),
            Some("#123456")
        );
        assert_eq!(
            state.values.get("active_target").unwrap().as_str(),
            Some("primary")
        );
        assert_eq!(
            state.values.get("action_swap").unwrap().as_bool(),
            Some(false)
        );
        assert_eq!(state.values.get("size").unwrap().as_f64(), Some(50.0));
        assert_eq!(state.values.get("enabled").unwrap().as_bool(), Some(true));

        let map = state.to_rhai_map();
        assert_eq!(map.len(), 5);

        let mut mutated = map.clone();
        mutated.insert("primary".into(), rhai::Dynamic::from("#abcdef".to_string()));
        mutated.insert("size".into(), rhai::Dynamic::from(75.0f64));

        state.update_from_rhai_map(mutated);
        assert_eq!(
            state.values.get("primary").unwrap().as_str(),
            Some("#abcdef")
        );
        assert_eq!(state.values.get("size").unwrap().as_f64(), Some(75.0));
    }

    #[test]
    fn test_viewport_id_deterministic_and_unique() {
        let id1 = DynamicToolEngine::viewport_id_for_plugin("plugin.a");
        let id2 = DynamicToolEngine::viewport_id_for_plugin("plugin.a");
        let id3 = DynamicToolEngine::viewport_id_for_plugin("plugin.b");

        assert_eq!(id1, id2, "ViewportId must be deterministic for same plugin");
        assert_ne!(id1, id3, "ViewportId must be unique for different plugins");
    }

    #[test]
    fn test_dynamic_tool_engine_lifecycle_and_dimensions() {
        let manifest = PluginManifest {
            id: "ui.photocraft.plugins.test_tool".to_string(),
            name: "Test Tool".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Author".to_string(),
            plugin_type: core_contracts::PluginType::InteractiveUi,
            icon: None,
            commands: vec![],
            models: vec![],
            interactive_ui: Some(InteractiveUiConfig {
                window_title: "Test Floating Palette".to_string(),
                window_title_ru: None,
                default_width: Some(360.0),
                default_height: Some(480.0),
                script: None,
                widgets: vec![],
                ipc_subscriptions: vec![],
            }),
            ..Default::default()
        };

        let mut engine = DynamicToolEngine::new(&[manifest], None);
        assert!(engine.has_plugin("ui.photocraft.plugins.test_tool"));
        assert!(!engine.is_window_open("ui.photocraft.plugins.test_tool"));

        // Toggle window open
        engine.toggle_window("ui.photocraft.plugins.test_tool");
        assert!(engine.is_window_open("ui.photocraft.plugins.test_tool"));

        let state = engine
            .states
            .get("ui.photocraft.plugins.test_tool")
            .unwrap();
        assert_eq!(state.default_width, 360.0);
        assert_eq!(state.default_height, 480.0);
        assert_eq!(state.title, "Test Floating Palette");

        // Toggle window closed
        engine.toggle_window("ui.photocraft.plugins.test_tool");
        assert!(!engine.is_window_open("ui.photocraft.plugins.test_tool"));
    }

    #[test]
    fn test_find_nearest_named_color_perceptual_accuracy() {
        // Pure red
        assert_eq!(find_nearest_named_color(255, 0, 0, Language::En), "Red");
        assert_eq!(find_nearest_named_color(255, 0, 0, Language::Ru), "Красный");

        // Classical Ultramarine pigment
        assert_eq!(
            find_nearest_named_color(18, 10, 143, Language::En),
            "Ultramarine"
        );
        assert_eq!(
            find_nearest_named_color(18, 10, 143, Language::Ru),
            "Ультрамарин"
        );

        // Pure white
        assert_eq!(
            find_nearest_named_color(255, 255, 255, Language::En),
            "White"
        );
        assert_eq!(
            find_nearest_named_color(255, 255, 255, Language::Ru),
            "Белый"
        );

        // Pure black
        assert_eq!(find_nearest_named_color(0, 0, 0, Language::En), "Black");
        assert_eq!(find_nearest_named_color(0, 0, 0, Language::Ru), "Чёрный");
    }

    #[test]
    fn test_simulate_cvd_color_and_hex() {
        // Normal vision
        assert_eq!(simulate_cvd_hex(255.0, 0.0, 0.0, 0), "#ff0000");

        // Deuteranopia (green blindness)
        let (dr, dg, db) = simulate_cvd_color(0.0, 255.0, 0.0, 1);
        assert!(dr > 80 && dr < 110);
        assert!(dg > 60 && dg < 90);
        assert!(db > 60 && db < 90);
        let d_hex = simulate_cvd_hex(0.0, 255.0, 0.0, 1);
        assert_eq!(d_hex.len(), 7);
        assert!(d_hex.starts_with('#'));

        // Protanopia (red blindness)
        let p_hex = simulate_cvd_hex(255.0, 0.0, 0.0, 2);
        assert_eq!(p_hex.len(), 7);
        assert!(p_hex.starts_with('#'));

        // Tritanopia (blue blindness)
        let t_hex = simulate_cvd_hex(0.0, 0.0, 255.0, 3);
        assert_eq!(t_hex.len(), 7);
        assert!(t_hex.starts_with('#'));
    }

    #[test]
    fn test_rhai_native_helpers_and_language_switching() {
        let manifest = PluginManifest {
            id: "ui.photocraft.plugins.rhai_test".to_string(),
            name: "Rhai Test".to_string(),
            interactive_ui: Some(InteractiveUiConfig {
                window_title: "Title En".to_string(),
                window_title_ru: Some("Заголовок Ру".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let mut engine = DynamicToolEngine::new_with_language(&[manifest], None, Language::En);
        assert_eq!(engine.current_language(), Language::En);
        assert_eq!(
            engine
                .states
                .get("ui.photocraft.plugins.rhai_test")
                .unwrap()
                .title,
            "Title En"
        );

        // Switch language to Russian
        engine.set_language(Language::Ru);
        assert_eq!(engine.current_language(), Language::Ru);
        assert_eq!(
            engine
                .states
                .get("ui.photocraft.plugins.rhai_test")
                .unwrap()
                .title,
            "Заголовок Ру"
        );

        // Test Rhai execution of registered functions
        let hex_res = engine
            .rhai_engine
            .eval::<String>("rgb_to_hex(0, 255, 0)")
            .unwrap();
        assert_eq!(hex_res, "#00ff00");

        let t_res = engine
            .rhai_engine
            .eval::<String>(r#"t("Hello", "Привет")"#)
            .unwrap();
        assert_eq!(t_res, "Привет");

        let lang_res = engine
            .rhai_engine
            .eval::<String>("current_language()")
            .unwrap();
        assert_eq!(lang_res, "ru");

        let color_name = engine
            .rhai_engine
            .eval::<String>("get_named_color(255, 0, 0)")
            .unwrap();
        assert_eq!(color_name, "Красный");
    }

    #[test]
    fn test_chromasense_wcag_and_cvd_standard_compliance() {
        // Light pink #ffb4eb (255, 180, 235) on pure white #ffffff (255, 255, 255)
        let ratio_normal = calculate_wcag_contrast(255.0, 180.0, 235.0, 255.0, 255.0, 255.0);
        assert!(ratio_normal < 3.0);
        assert!((ratio_normal - 1.58).abs() < 0.1);

        // Deuteranopia (profile 1): simulated contrast must also be strictly < 3.0:1 (~1.2:1)
        let (dr1, dg1, db1) = simulate_cvd_color(255.0, 180.0, 235.0, 1);
        let (dr2, dg2, db2) = simulate_cvd_color(255.0, 255.0, 255.0, 1);
        let ratio_deut = calculate_wcag_contrast(
            dr1 as f64, dg1 as f64, db1 as f64, dr2 as f64, dg2 as f64, db2 as f64,
        );
        assert!(ratio_deut < 3.0);
        assert!((ratio_deut - 1.2).abs() < 0.2);

        // Protanopia (profile 2): simulated contrast < 3.0:1 (~1.3:1)
        let (pr1, pg1, pb1) = simulate_cvd_color(255.0, 180.0, 235.0, 2);
        let (pr2, pg2, pb2) = simulate_cvd_color(255.0, 255.0, 255.0, 2);
        let ratio_prot = calculate_wcag_contrast(
            pr1 as f64, pg1 as f64, pb1 as f64, pr2 as f64, pg2 as f64, pb2 as f64,
        );
        assert!(ratio_prot < 3.0);
        assert!((ratio_prot - 1.3).abs() < 0.2);

        // Tritanopia (profile 3): simulated contrast < 3.0:1 (~1.4:1)
        let (tr1, tg1, tb1) = simulate_cvd_color(255.0, 180.0, 235.0, 3);
        let (tr2, tg2, tb2) = simulate_cvd_color(255.0, 255.0, 255.0, 3);
        let ratio_trit = calculate_wcag_contrast(
            tr1 as f64, tg1 as f64, tb1 as f64, tr2 as f64, tg2 as f64, tb2 as f64,
        );
        assert!(ratio_trit < 3.0);
        assert!((ratio_trit - 1.4).abs() < 0.2);
    }

    #[test]
    fn test_rhai_sandbox_operation_limits() {
        let manifest = PluginManifest {
            id: "ui.photocraft.plugins.infinite_loop".to_string(),
            name: "Infinite Loop Plugin".to_string(),
            interactive_ui: Some(InteractiveUiConfig {
                window_title: "Test".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };

        let engine = DynamicToolEngine::new_with_language(&[manifest], None, Language::En);
        // Evaluating infinite loop must terminate with operations limit exceeded error without hanging the thread
        let res = engine
            .rhai_engine
            .eval::<()>("let count = 0; while true { count += 1; }");
        assert!(res.is_err(), "Evaluation must fail due to operation limit!");
        let err_msg = res.unwrap_err().to_string();
        assert!(
            err_msg.contains("Too many operations") || err_msg.contains("operations"),
            "Error should indicate operation limit exceeded, got: {err_msg}"
        );
    }

    #[test]
    fn test_vector_pad_and_preview_initialization() {
        let cfg = InteractiveUiConfig {
            window_title: "3D Extrude".to_string(),
            widgets: vec![
                WidgetNode::VectorPad {
                    id: "extrude_vector".to_string(),
                    label: "Vector".to_string(),
                    label_ru: Some("Вектор".to_string()),
                    max_distance: 100.0,
                    default_angle_deg: 135.0,
                    default_distance: 24.0,
                },
                WidgetNode::ImagePreview {
                    id: "preview_pane".to_string(),
                    height: 180.0,
                },
            ],
            ..Default::default()
        };

        let state = DynamicWindowState::new(&cfg);
        assert_eq!(
            state.values.get("extrude_vector_angle").unwrap().as_f64(),
            Some(135.0)
        );
        assert_eq!(
            state
                .values
                .get("extrude_vector_distance")
                .unwrap()
                .as_f64(),
            Some(24.0)
        );
        assert_eq!(
            state.values.get("preview_pane_preview").unwrap().as_bool(),
            Some(true)
        );
    }

    #[test]
    fn test_dynamic_tool_preview_storage_methods() {
        let mut engine = DynamicToolEngine::new(&[], None);
        assert!(engine.get_preview_image("plugin.test").is_none());

        let test_img = image::RgbaImage::new(32, 32);
        engine.set_preview_image("plugin.test", test_img.clone(), 0.5);

        let preview = engine.get_preview_image("plugin.test");
        assert!(preview.is_some());
        let (stored_img, scale) = preview.unwrap();
        assert_eq!(*scale, 0.5);
        assert_eq!(stored_img.dimensions(), (32, 32));
    }

    #[test]
    fn test_execute_raster_process_rhai_ipc() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let engine = DynamicToolEngine::new(&[], Some(tx));

        let script = r#"
            let params = #{
                task: "extrude",
                angle: 120.0,
                distance: 35.0
            };
            execute_raster_process("extrude", params);
        "#;

        engine.rhai_engine.eval::<()>(script).unwrap();

        let action = rx.try_recv().expect("Expected PluginIpcAction");
        assert_eq!(action.command, "raster.process");
        assert_eq!(action.params["task"], "extrude");
        assert_eq!(action.params["params"]["distance"], 35.0);
    }

    #[test]
    fn test_extract_extrude_params_and_hashing() {
        let mut values = HashMap::new();
        values.insert("vector_angle".to_string(), DynamicValue::Number(45.0));
        values.insert("vector_distance".to_string(), DynamicValue::Number(30.0));
        values.insert(
            "body_start_color".to_string(),
            DynamicValue::String("#ff0000".to_string()),
        );
        values.insert(
            "body_end_color".to_string(),
            DynamicValue::String("#00ff00".to_string()),
        );

        let params = extract_extrude_params(&values);
        assert_eq!(params.angle_deg, 45.0);
        assert_eq!(params.distance, 30.0);
        assert_eq!(params.body_gradient_start, image::Rgba([255, 0, 0, 255]));
        assert_eq!(params.body_gradient_end, image::Rgba([0, 255, 0, 255]));

        let hash1 = hash_extrude_params(&params, 1.0);
        let hash2 = hash_extrude_params(&params, 1.0);
        let hash_scaled = hash_extrude_params(&params, 0.5);
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash_scaled);
    }

    #[test]
    fn test_popcraft_3d_rhai_script_state_change() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let engine = DynamicToolEngine::new(&[], Some(tx));

        let script = include_str!("../../../plugins/popcraft_3d/script.rhai");
        let ast = engine
            .rhai_engine
            .compile(script)
            .expect("Script must compile");

        let mut scope = rhai::Scope::new();
        let mut state_map = rhai::Map::new();
        state_map.insert("action_preset_youtube".into(), rhai::Dynamic::from(true));
        state_map.insert("extrude_pad_distance".into(), rhai::Dynamic::from(10.0));

        let res_map: rhai::Map = engine
            .rhai_engine
            .call_fn(&mut scope, &ast, "on_state_changed", (state_map,))
            .expect("on_state_changed must succeed");

        assert_eq!(
            res_map.get("action_preset_youtube").unwrap().as_bool(),
            Ok(false)
        );
        assert_eq!(
            res_map.get("extrude_pad_angle").unwrap().as_float(),
            Ok(120.0)
        );
        assert_eq!(res_map.get("shadow_enabled").unwrap().as_bool(), Ok(true));
        assert_eq!(res_map.get("perspective").unwrap().as_float(), Ok(0.25));
        assert_eq!(
            res_map
                .get("body_start_color")
                .unwrap()
                .clone()
                .into_string(),
            Ok("#f59e0b".to_string())
        );

        // Test Apply trigger with shadow disabled and perspective set
        let mut apply_map = rhai::Map::new();
        apply_map.insert("action_apply".into(), rhai::Dynamic::from(true));
        apply_map.insert("shadow_enabled".into(), rhai::Dynamic::from(false));
        apply_map.insert("perspective".into(), rhai::Dynamic::from(0.3));
        apply_map.insert("extrude_pad_angle".into(), rhai::Dynamic::from(45.0));
        apply_map.insert("extrude_pad_distance".into(), rhai::Dynamic::from(20.0));
        apply_map.insert(
            "body_start_color".into(),
            rhai::Dynamic::from("#ff0000".to_string()),
        );
        apply_map.insert(
            "body_end_color".into(),
            rhai::Dynamic::from("#0000ff".to_string()),
        );
        apply_map.insert("output_mode".into(), rhai::Dynamic::from(0 as rhai::INT));

        let _: rhai::Map = engine
            .rhai_engine
            .call_fn(&mut scope, &ast, "on_state_changed", (apply_map,))
            .expect("Apply must succeed");

        let action = rx.try_recv().expect("Apply must send IPC action");
        assert_eq!(action.command, "raster.process");
        assert_eq!(action.params["task"], "extrude_2_5d");
        assert_eq!(action.params["params"]["angle_deg"], 45.0);
        assert_eq!(action.params["params"]["shadow_opacity"], 0.0);
        assert_eq!(action.params["params"]["perspective"], 0.3);
    }

    #[test]
    fn test_selection_modifier_stack_rhai_script() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let engine = DynamicToolEngine::new(&[], Some(tx));

        let script = include_str!("../../../plugins/selection_mod/script.rhai");
        let ast = engine
            .rhai_engine
            .compile(script)
            .expect("Selection mod script must compile");

        let mut scope = rhai::Scope::new();

        // 1. Tool window opened with active selection snapshots working base
        let mut open_map = rhai::Map::new();
        open_map.insert("is_open".into(), rhai::Dynamic::from(true));
        let res_map1: rhai::Map = engine
            .rhai_engine
            .call_fn(&mut scope, &ast, "on_state_changed", (open_map,))
            .expect("First on_state_changed must succeed");

        assert_eq!(res_map1.get("base_saved").unwrap().as_bool(), Ok(true));
        let save_base_action = rx.try_recv().expect("Must save base working selection");
        assert_eq!(save_base_action.command, "select.saveSelection");
        assert_eq!(save_base_action.params["name"], "__pc_companion_base_sel");
        assert_eq!(
            save_base_action.params["channel"],
            "__pc_companion_base_sel"
        );
        assert_eq!(save_base_action.params["operation"], "replace");
        assert!(rx.try_recv().is_err(), "Only base selection must be saved");

        // 2. Reset action (Zero Data-Loss UX: resets UI modifiers without loading or altering canvas)
        let mut reset_map = res_map1.clone();
        reset_map.insert("contract_enabled".into(), rhai::Dynamic::from(true));
        reset_map.insert("contract_value".into(), rhai::Dynamic::from(10.0));
        reset_map.insert("action_reset".into(), rhai::Dynamic::from(true));
        let res_map2: rhai::Map = engine
            .rhai_engine
            .call_fn(&mut scope, &ast, "on_state_changed", (reset_map,))
            .expect("Reset on_state_changed must succeed");

        assert_eq!(res_map2.get("action_reset").unwrap().as_bool(), Ok(false));
        assert_eq!(
            res_map2.get("contract_enabled").unwrap().as_bool(),
            Ok(false)
        );
        assert_eq!(res_map2.get("contract_value").unwrap().as_float(), Ok(0.0));

        // Canvas selection is saved as base, NOT loaded/destroyed
        let reset_sync_action = rx.try_recv().expect("Must sync canvas to base on reset");
        assert_eq!(reset_sync_action.command, "select.saveSelection");
        assert_eq!(
            reset_sync_action.params["channel"],
            "__pc_companion_base_sel"
        );
        assert_eq!(reset_sync_action.params["operation"], "replace");
        assert!(
            rx.try_recv().is_err(),
            "select.loadSelection must NOT be called on reset (Zero Data-Loss)"
        );

        // 3. Apply custom modifier stack order
        let mut apply_map = res_map2;
        apply_map.insert("action_apply".into(), rhai::Dynamic::from(true));
        // Custom order: contract -> smooth -> feather
        let custom_order: rhai::Array = vec![
            rhai::Dynamic::from("contract".to_string()),
            rhai::Dynamic::from("smooth".to_string()),
            rhai::Dynamic::from("feather".to_string()),
        ];
        apply_map.insert(
            "modifiers_order".into(),
            rhai::Dynamic::from_array(custom_order),
        );
        apply_map.insert("contract_enabled".into(), rhai::Dynamic::from(true));
        apply_map.insert("contract_value".into(), rhai::Dynamic::from(8.0));
        apply_map.insert("smooth_enabled".into(), rhai::Dynamic::from(true));
        apply_map.insert("smooth_value".into(), rhai::Dynamic::from(4.0));
        apply_map.insert("feather_enabled".into(), rhai::Dynamic::from(false));
        apply_map.insert("feather_value".into(), rhai::Dynamic::from(2.0));

        let res_map3: rhai::Map = engine
            .rhai_engine
            .call_fn(&mut scope, &ast, "on_state_changed", (apply_map,))
            .expect("Apply on_state_changed must succeed");

        assert_eq!(res_map3.get("action_apply").unwrap().as_bool(), Ok(false));
        assert_eq!(res_map3.get("base_saved").unwrap().as_bool(), Ok(true));

        let load_base_action = rx.try_recv().expect("Must load base working selection");
        assert_eq!(load_base_action.command, "select.loadSelection");
        assert_eq!(
            load_base_action.params["channel"],
            "__pc_companion_base_sel"
        );
        assert_eq!(load_base_action.params["operation"], "new");

        let mod1_action = rx
            .try_recv()
            .expect("Must execute first modifier (contract)");
        assert_eq!(mod1_action.command, "select.modify.contract");
        assert_eq!(mod1_action.params["radius"], 8.0);
        assert_eq!(mod1_action.params["applyAtCanvasBounds"], false);

        let mod2_action = rx
            .try_recv()
            .expect("Must execute second modifier (smooth)");
        assert_eq!(mod2_action.command, "select.modify.smooth");
        assert_eq!(mod2_action.params["radius"], 4.0);
        assert_eq!(mod2_action.params["applyAtCanvasBounds"], false);

        // 4. Test unchecking commits current selection as new working base and zeroes radius
        let mut uncheck_map = res_map3;
        uncheck_map.insert("contract_enabled".into(), rhai::Dynamic::from(false));
        let res_map4: rhai::Map = engine
            .rhai_engine
            .call_fn(&mut scope, &ast, "on_state_changed", (uncheck_map,))
            .expect("Uncheck on_state_changed must succeed");
        assert_eq!(res_map4.get("contract_value").unwrap().as_float(), Ok(0.0));
        let uncheck_commit = rx.try_recv().expect("Must save base on uncheck");
        assert_eq!(uncheck_commit.command, "select.saveSelection");
        assert_eq!(uncheck_commit.params["channel"], "__pc_companion_base_sel");

        // 5. Test external selection reactivity and bounds change in PhotoCraft
        let ext_map = res_map4.clone();
        let bounds_1: rhai::Array = vec![
            rhai::Dynamic::from(10 as rhai::INT),
            rhai::Dynamic::from(20 as rhai::INT),
            rhai::Dynamic::from(100 as rhai::INT),
            rhai::Dynamic::from(200 as rhai::INT),
        ];
        let res_sel1: rhai::Map = engine
            .rhai_engine
            .call_fn(
                &mut scope,
                &ast,
                "on_selection_state",
                (true, rhai::Dynamic::from_array(bounds_1), ext_map),
            )
            .expect("on_selection_state 3-args must succeed");
        assert_eq!(res_sel1.get("base_saved").unwrap().as_bool(), Ok(true));

        // When bounds change externally in PhotoCraft, base_saved becomes false to snapshot new contour on apply
        let bounds_2: rhai::Array = vec![
            rhai::Dynamic::from(30 as rhai::INT),
            rhai::Dynamic::from(40 as rhai::INT),
            rhai::Dynamic::from(150 as rhai::INT),
            rhai::Dynamic::from(250 as rhai::INT),
        ];
        let res_sel2: rhai::Map = engine
            .rhai_engine
            .call_fn(
                &mut scope,
                &ast,
                "on_selection_state",
                (true, rhai::Dynamic::from_array(bounds_2), res_sel1),
            )
            .expect("on_selection_state changed bounds must succeed");
        assert_eq!(res_sel2.get("base_saved").unwrap().as_bool(), Ok(false));

        // When selection is deselected in PhotoCraft
        let res_sel3: rhai::Map = engine
            .rhai_engine
            .call_fn(
                &mut scope,
                &ast,
                "on_selection_state",
                (false, rhai::Dynamic::UNIT, res_sel2),
            )
            .expect("on_selection_state false must succeed");
        assert_eq!(res_sel3.get("base_saved").unwrap().as_bool(), Ok(false));

        // 6. Test window closing cleanup deletes temporary alpha channel
        let mut close_map = res_map4;
        close_map.insert("is_open".into(), rhai::Dynamic::from(false));
        let res_map5: rhai::Map = engine
            .rhai_engine
            .call_fn(&mut scope, &ast, "on_state_changed", (close_map,))
            .expect("Close on_state_changed must succeed");
        assert_eq!(res_map5.get("base_saved").unwrap().as_bool(), Ok(false));
        let del1 = rx.try_recv().expect("Must delete base_sel channel");
        assert_eq!(del1.command, "channel.delete");
        assert_eq!(del1.params["channel"], "__pc_companion_base_sel");
        assert!(rx.try_recv().is_err(), "Only base_sel must be deleted");
    }

    #[test]
    fn test_modifier_stack_config_persistence() {
        let manifest_json = r#"{
            "id": "ui.photocraft.plugins.selection_mod",
            "name": "Selection Modifiers",
            "version": "1.0.0",
            "description": "Test",
            "author": "Test",
            "plugin_type": "interactive_ui",
            "interactive_ui": {
                "window_title": "Selection Modifiers",
                "widgets": [
                    {
                        "type": "modifier_stack",
                        "id": "modifiers",
                        "label": "Modifiers",
                        "allow_reordering": true,
                        "items": [
                            { "id": "border", "label": "Border", "min": 0.0, "max": 100.0, "default": 0.0, "enabled": false },
                            { "id": "smooth", "label": "Smooth", "min": 0.0, "max": 50.0, "default": 0.0, "enabled": false },
                            { "id": "expand", "label": "Expand", "min": 0.0, "max": 100.0, "default": 0.0, "enabled": false }
                        ]
                    }
                ]
            }
        }"#;
        let manifest: PluginManifest = serde_json::from_str(manifest_json).unwrap();
        let mut engine = DynamicToolEngine::new(&[manifest], None);

        // Verify default order
        let state = engine
            .states
            .get("ui.photocraft.plugins.selection_mod")
            .unwrap();
        let order = state
            .values
            .get("modifiers_order")
            .unwrap()
            .as_string_list()
            .unwrap();
        assert_eq!(order, &["border", "smooth", "expand"]);

        // Apply config with custom order
        let mut config = core_contracts::CompanionConfig::default();
        config.modifier_stack_orders.insert(
            "ui.photocraft.plugins.selection_mod:modifiers".to_string(),
            vec![
                "expand".to_string(),
                "border".to_string(),
                "smooth".to_string(),
            ],
        );
        engine.apply_saved_modifier_stack_orders(&config);

        let state = engine
            .states
            .get("ui.photocraft.plugins.selection_mod")
            .unwrap();
        let order = state
            .values
            .get("modifiers_order")
            .unwrap()
            .as_string_list()
            .unwrap();
        assert_eq!(order, &["expand", "border", "smooth"]);
    }
}
