use crate::theme::ThemePalette;
use core_contracts::{
    BannerStyle, InteractiveUiConfig, PluginIpcAction, PluginManifest, WidgetNode,
};
use eframe::egui::{self, Color32, Sense, Stroke, Vec2};
use std::collections::HashMap;
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
        let mut values = HashMap::new();

        for widget in &config.widgets {
            Self::init_widget_values(widget, &mut values);
        }

        Self {
            values,
            is_open: false,
            title: config.window_title.clone(),
            default_width: config.default_width.unwrap_or(320.0),
            default_height: config.default_height.unwrap_or(480.0),
        }
    }

    fn init_widget_values(widget: &WidgetNode, values: &mut HashMap<String, DynamicValue>) {
        match widget {
            WidgetNode::ColorSlot {
                id, default_hex, ..
            } => {
                let hex = default_hex.clone().unwrap_or_else(|| "#ffffff".to_string());
                values.insert(id.clone(), DynamicValue::String(hex));
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
}

impl DynamicToolEngine {
    pub fn new(
        manifests: &[PluginManifest],
        action_tx: Option<UnboundedSender<PluginIpcAction>>,
    ) -> Self {
        let mut manifest_map = HashMap::new();
        let mut states = HashMap::new();
        let mut rhai_asts = HashMap::new();

        let mut rhai_engine = rhai::Engine::new();

        // Register photocraft commands in Rhai
        if let Some(tx) = action_tx.clone() {
            let tx_cmd = tx.clone();
            rhai_engine.register_fn(
                "execute_command",
                move |command: rhai::ImmutableString, params: rhai::Map| {
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
            rhai_engine.register_fn(
                "set_colors",
                move |foreground: rhai::ImmutableString, background: rhai::ImmutableString| {
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

        // Initialize manifests and compile scripts
        for m in manifests {
            if let Some(ref ui_cfg) = m.interactive_ui {
                manifest_map.insert(m.id.clone(), m.clone());
                states.insert(m.id.clone(), DynamicWindowState::new(ui_cfg));

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
        };

        // Run initial state pass for all loaded scripts
        engine.run_all_initial_scripts();

        engine
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

    /// Generates a deterministic ViewportId for an interactive plugin floating window
    pub fn viewport_id_for_plugin(plugin_id: &str) -> egui::ViewportId {
        egui::ViewportId::from_hash_of(&format!("dynamic_plugin_window_{}", plugin_id))
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
                            ui.spacing_mut().item_spacing = Vec2::new(6.0, 8.0);

                            if let Some(state) = self.states.get_mut(&plugin_id) {
                                for widget in &config.widgets {
                                    if Self::render_widget_node(ui, widget, state, palette) {
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
    ) -> bool {
        let mut changed = false;

        match node {
            WidgetNode::ColorSlot { id, label, .. } => {
                let mut current_hex = match state.values.get(id) {
                    Some(DynamicValue::String(s)) => s.clone(),
                    _ => "#000000".to_string(),
                };

                let parsed_color = parse_hex_color(&current_hex).unwrap_or(Color32::BLACK);

                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(Vec2::new(24.0, 24.0), Sense::hover());
                    ui.painter().rect_filled(rect, 4.0, parsed_color);
                    ui.painter().rect_stroke(
                        rect,
                        4.0,
                        Stroke::new(1.0, palette.border_stroke),
                        egui::StrokeKind::Inside,
                    );

                    ui.label(label);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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
                    });
                });
            }

            WidgetNode::Slider {
                id,
                label,
                min,
                max,
                step,
                ..
            } => {
                let mut current_val = match state.values.get(id) {
                    Some(DynamicValue::Number(n)) => *n,
                    _ => *min,
                };

                ui.horizontal(|ui| {
                    ui.add_sized([50.0, 18.0], egui::Label::new(label));
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

            WidgetNode::Dropdown {
                id, label, options, ..
            } => {
                let mut current_idx = match state.values.get(id) {
                    Some(DynamicValue::Index(i)) => *i,
                    _ => 0,
                };

                ui.horizontal(|ui| {
                    ui.label(label);
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

            WidgetNode::Toggle { id, label, .. } => {
                let mut current_bool = match state.values.get(id) {
                    Some(DynamicValue::Bool(b)) => *b,
                    _ => false,
                };

                if ui.checkbox(&mut current_bool, label).changed() {
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
                let source_hex = match state.values.get(source_color_id) {
                    Some(DynamicValue::String(s)) => s.clone(),
                    _ => "#ff0000".to_string(),
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
                        if Self::render_widget_node(ui, item, state, palette) {
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

                    ui.label(
                        egui::RichText::new(msg)
                            .size(11.0)
                            .color(palette.text_primary),
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

#[cfg(test)]
mod tests {
    use super::*;

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
            default_width: Some(300.0),
            default_height: Some(400.0),
            script: None,
            widgets: vec![
                WidgetNode::ColorSlot {
                    id: "primary".to_string(),
                    label: "Color".to_string(),
                    default_hex: Some("#123456".to_string()),
                },
                WidgetNode::Slider {
                    id: "size".to_string(),
                    label: "Size".to_string(),
                    min: 1.0,
                    max: 100.0,
                    step: Some(1.0),
                    default: 50.0,
                },
                WidgetNode::Toggle {
                    id: "enabled".to_string(),
                    label: "Enabled".to_string(),
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
        assert_eq!(state.values.get("size").unwrap().as_f64(), Some(50.0));
        assert_eq!(state.values.get("enabled").unwrap().as_bool(), Some(true));

        let map = state.to_rhai_map();
        assert_eq!(map.len(), 3);

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
}
