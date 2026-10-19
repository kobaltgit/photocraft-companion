use crate::theme::ThemePalette;
use core_contracts::{
    BannerStyle, InteractiveUiConfig, Language, PluginIpcAction, PluginManifest, WidgetNode,
};
use eframe::egui::{self, Color32, Sense, Stroke, Vec2};
use std::collections::HashMap;
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
            WidgetNode::Row { items } => {
                for item in items {
                    Self::init_widget_values(item, values);
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

            let tx_colors = tx;
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
        };

        // Run initial state pass for all loaded scripts
        engine.run_all_initial_scripts();

        engine
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

    pub fn toggle_window(&mut self, plugin_id: &str) {
        if let Some(state) = self.states.get_mut(plugin_id) {
            state.is_open = !state.is_open;
            if state.is_open {
                self.execute_script_state_change(plugin_id);
            }
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
                .with_min_inner_size([280.0, 260.0])
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
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.add_space(2.0);
                            ui.spacing_mut().item_spacing = Vec2::new(6.0, 8.0);

                            if let Some(state) = self.states.get_mut(&plugin_id) {
                                for widget in &config.widgets {
                                    if Self::render_widget_node(
                                        ui,
                                        widget,
                                        state,
                                        palette,
                                        self.current_language,
                                    ) {
                                        state_changed = true;
                                    }
                                }
                            }
                        });
                    });
            });

            if !is_open {
                if let Some(state) = self.states.get_mut(&plugin_id) {
                    state.is_open = false;
                }
            }

            if state_changed {
                self.execute_script_state_change(&plugin_id);
            }
        }
    }

    fn render_widget_node(
        ui: &mut egui::Ui,
        node: &WidgetNode,
        state: &mut DynamicWindowState,
        palette: &ThemePalette,
        lang: Language,
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
                    let sense = if *selectable {
                        Sense::click()
                    } else {
                        Sense::hover()
                    };
                    let (rect, response) = ui.allocate_exact_size(Vec2::new(24.0, 24.0), sense);

                    if *selectable {
                        let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
                        if response.clicked() {
                            if let Some(tb) = target_binding {
                                state
                                    .values
                                    .insert(tb.clone(), DynamicValue::String(id.clone()));
                                changed = true;
                            }
                        }
                    }

                    ui.painter().rect_filled(rect, 4.0, parsed_color);
                    if is_selected {
                        // High-contrast active accent focus outline inside the swatch bounds (never clipped)
                        ui.painter().rect_stroke(
                            rect,
                            4.0,
                            Stroke::new(2.0, palette.accent_blue),
                            egui::StrokeKind::Inside,
                        );
                    } else {
                        ui.painter().rect_stroke(
                            rect,
                            4.0,
                            Stroke::new(1.0, palette.border_stroke),
                            egui::StrokeKind::Inside,
                        );
                    }

                    let label_resp = ui.label(node.label_for_lang(lang));
                    if *selectable && label_resp.interact(Sense::click()).clicked() {
                        if let Some(tb) = target_binding {
                            state
                                .values
                                .insert(tb.clone(), DynamicValue::String(id.clone()));
                            changed = true;
                        }
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if *read_only {
                            ui.add(egui::Label::new(
                                egui::RichText::new(&current_hex)
                                    .monospace()
                                    .color(palette.text_muted),
                            ));
                        } else {
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut current_hex)
                                    .desired_width(75.0)
                                    .font(egui::TextStyle::Monospace),
                            );
                            if response.changed() {
                                state
                                    .values
                                    .insert(id.clone(), DynamicValue::String(current_hex));
                                changed = true;
                            }
                        }
                    });
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
                    ui.add_sized([50.0, 18.0], egui::Label::new(node.label_for_lang(lang)));
                    let mut slider = egui::Slider::new(&mut current_val, *min..=*max);
                    if let Some(s) = step {
                        slider = slider.step_by(*s);
                    }
                    let resp = ui.add(slider);
                    if resp.changed() {
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

            WidgetNode::Separator => {
                ui.separator();
            }

            WidgetNode::Row { items } => {
                ui.horizontal(|ui| {
                    for item in items {
                        if Self::render_widget_node(ui, item, state, palette, lang) {
                            changed = true;
                        }
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
}
