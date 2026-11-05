use core_contracts::{BannerStyle, InteractiveUiConfig, Language, WidgetNode};
use std::collections::HashMap;

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
    pub icons_calibrated: bool,
    pub title: String,
    pub default_width: f32,
    pub default_height: f32,
    pub role_mask: Option<image::GrayImage>,
    pub dynamic_badge: Option<core_contracts::DynamicBadgeState>,
    pub timer_interval_ms: Option<u64>,
    pub last_tick: Option<std::time::Instant>,
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
            icons_calibrated: false,
            title: config.window_title_for_lang(lang).to_string(),
            default_width: config.default_width.unwrap_or_else(|| {
                if config.layout_mode.as_deref() == Some("split_preview_right") {
                    580.0
                } else {
                    320.0
                }
            }),
            default_height: config.default_height.unwrap_or(480.0),
            role_mask: None,
            dynamic_badge: config.dynamic_badge.clone(),
            timer_interval_ms: None,
            last_tick: None,
        }
    }

    pub fn init_widget_values(widget: &WidgetNode, values: &mut HashMap<String, DynamicValue>) {
        match widget {
            WidgetNode::ColorSlot {
                id,
                default_hex,
                target_binding,
                label_binding,
                ..
            } => {
                let hex = default_hex.clone().unwrap_or_else(|| "#ffffff".to_string());
                values.insert(id.clone(), DynamicValue::String(hex));
                if let Some(tb) = target_binding {
                    values
                        .entry(tb.clone())
                        .or_insert_with(|| DynamicValue::String(id.clone()));
                }
                if let Some(lb) = label_binding {
                    values
                        .entry(lb.clone())
                        .or_insert_with(|| DynamicValue::String(String::new()));
                }
            }
            WidgetNode::Slider {
                id,
                default,
                enabled_binding,
                ..
            } => {
                values.insert(id.clone(), DynamicValue::Number(*default));
                if let Some(eb) = enabled_binding {
                    values
                        .entry(eb.clone())
                        .or_insert_with(|| DynamicValue::Bool(true));
                }
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
            WidgetNode::HueWheel {
                id,
                markers_binding,
                connect_mode_binding,
                ..
            } => {
                values.insert(id.clone(), DynamicValue::Number(0.0));
                if let Some(mb) = markers_binding {
                    values
                        .entry(mb.clone())
                        .or_insert_with(|| DynamicValue::StringList(Vec::new()));
                }
                if let Some(cmb) = connect_mode_binding {
                    values
                        .entry(cmb.clone())
                        .or_insert_with(|| DynamicValue::String("none".to_string()));
                }
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
            WidgetNode::Tabs {
                id,
                tabs,
                default_index,
            } => {
                values.insert(id.clone(), DynamicValue::Index(*default_index));
                for tab in tabs {
                    for w in &tab.widgets {
                        Self::init_widget_values(w, values);
                    }
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
        map.insert("is_open".into(), rhai::Dynamic::from(self.is_open));
        if let Some(interval) = self.timer_interval_ms {
            map.insert(
                "timer_interval_ms".into(),
                rhai::Dynamic::from(interval as rhai::INT),
            );
        }
        if let Some(ref badge) = self.dynamic_badge {
            let mut b_map = rhai::Map::new();
            b_map.insert("is_pulsing".into(), rhai::Dynamic::from(badge.is_pulsing));
            if let Some(ref lbl) = badge.label {
                b_map.insert("label".into(), rhai::Dynamic::from(lbl.clone()));
            }
            let rgba_arr: rhai::Array = badge
                .pulse_color_rgba
                .iter()
                .map(|&c| rhai::Dynamic::from(c as rhai::INT))
                .collect();
            b_map.insert(
                "pulse_color_rgba".into(),
                rhai::Dynamic::from_array(rgba_arr),
            );
            map.insert("dynamic_badge".into(), rhai::Dynamic::from(b_map));
        }
        for (k, v) in &self.values {
            let key = k.clone().into();
            match v {
                DynamicValue::String(s) => {
                    map.insert(key, rhai::Dynamic::from(s.clone()));
                }
                DynamicValue::Number(n) => {
                    if n.fract() == 0.0 && *n >= (i64::MIN as f64) && *n <= (i64::MAX as f64) {
                        map.insert(key, rhai::Dynamic::from(*n as rhai::INT));
                    } else {
                        map.insert(key, rhai::Dynamic::from(*n));
                    }
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
        if self.dynamic_badge.is_some() && !map.contains_key("dynamic_badge") {
            self.dynamic_badge = None;
        }
        for (k, v) in map {
            let key = k.to_string();

            if key == "timer_interval_ms" {
                let int_val = if v.is_int() {
                    v.as_int().ok()
                } else if v.is_float() {
                    v.as_float().ok().map(|f| f as i64)
                } else {
                    None
                };
                if let Some(i) = int_val {
                    if i > 0 {
                        if self.timer_interval_ms != Some(i as u64) {
                            self.last_tick = Some(std::time::Instant::now());
                        }
                        self.timer_interval_ms = Some(i as u64);
                    } else {
                        self.timer_interval_ms = None;
                        self.last_tick = None;
                    }
                } else if v.is_unit() {
                    self.timer_interval_ms = None;
                    self.last_tick = None;
                }
                continue;
            }

            if key == "dynamic_badge" {
                if let Some(b_map) = v.clone().try_cast::<rhai::Map>() {
                    let is_pulsing = b_map
                        .get("is_pulsing")
                        .and_then(|x| x.as_bool().ok())
                        .unwrap_or(false);
                    let label = b_map
                        .get("label")
                        .and_then(|x| x.clone().into_string().ok());
                    let mut pulse_color_rgba = [255, 50, 50, 255];
                    if let Some(rgba_val) = b_map.get("pulse_color_rgba") {
                        if let Ok(arr) = rgba_val.clone().into_array() {
                            if arr.len() >= 4 {
                                pulse_color_rgba = [
                                    arr[0].as_int().unwrap_or(255) as u8,
                                    arr[1].as_int().unwrap_or(50) as u8,
                                    arr[2].as_int().unwrap_or(50) as u8,
                                    arr[3].as_int().unwrap_or(255) as u8,
                                ];
                            }
                        }
                    }
                    self.dynamic_badge = Some(core_contracts::DynamicBadgeState {
                        is_pulsing,
                        pulse_color_rgba,
                        label,
                    });
                } else if v.is_unit() {
                    self.dynamic_badge = None;
                }
                continue;
            }

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
                    if key == "is_open" {
                        self.is_open = b;
                    }
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
                if key == "timer_interval_ms" {
                    self.timer_interval_ms = None;
                    self.last_tick = None;
                } else if key == "dynamic_badge" {
                    self.dynamic_badge = None;
                }
                self.values.remove(&key);
            }
        }
    }
}
