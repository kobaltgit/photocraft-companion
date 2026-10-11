use egui::{Color32, Stroke, Visuals};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    Dark,
    Light,
    System,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::Dark
    }
}

impl ThemeMode {
    pub fn is_dark(&self) -> bool {
        match self {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => Self::detect_system_dark(),
        }
    }

    pub fn detect_system_dark() -> bool {
        #[cfg(windows)]
        {
            use std::process::Command;
            let output = Command::new("reg")
                .args([
                    "query",
                    r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                    "/v",
                    "AppsUseLightTheme",
                ])
                .output();
            if let Ok(out) = output {
                let s = String::from_utf8_lossy(&out.stdout);
                if s.contains("0x0") {
                    return true;
                } else if s.contains("0x1") {
                    return false;
                }
            }
        }
        true
    }

    pub fn from_preferences(prefs_path: impl AsRef<Path>) -> Self {
        let path = prefs_path.as_ref();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(theme_str) = v
                        .get("interface")
                        .and_then(|i| i.get("theme"))
                        .and_then(|t| t.as_str())
                    {
                        return match theme_str {
                            "studioLight" | "classic" | "adwaita" => ThemeMode::Light,
                            "auto" => ThemeMode::System,
                            _ => ThemeMode::Dark,
                        };
                    }
                }
            }
        }
        ThemeMode::Dark
    }
}

#[derive(Debug, Clone)]
pub struct ThemePalette {
    pub bg_fill: Color32,
    pub panel_fill: Color32,
    pub input_fill: Color32,
    pub border_stroke: Color32,
    pub text_primary: Color32,
    pub text_muted: Color32,
    pub accent_blue: Color32,
    pub accent_hover: Color32,
    pub button_secondary: Color32,
    pub button_secondary_hover: Color32,
    pub card_fill: Color32,
    pub card_inner: Color32,
    pub is_dark: bool,
}

impl ThemePalette {
    pub fn for_mode(mode: ThemeMode) -> Self {
        if mode.is_dark() {
            Self {
                // Strict PhotoCraft dark palette
                bg_fill: Color32::from_rgb(0x24, 0x24, 0x24),
                panel_fill: Color32::from_rgb(0x2d, 0x2d, 0x2d),
                input_fill: Color32::from_rgb(0x1c, 0x1c, 0x1c),
                border_stroke: Color32::from_rgb(0x3e, 0x3e, 0x3e),
                text_primary: Color32::from_rgb(0xf0, 0xf0, 0xf0),
                text_muted: Color32::from_rgb(0x9a, 0x9a, 0x9a),
                accent_blue: Color32::from_rgb(0x2a, 0x82, 0xe4),
                accent_hover: Color32::from_rgb(0x38, 0x8b, 0xfd),
                button_secondary: Color32::from_rgb(0x35, 0x35, 0x35),
                button_secondary_hover: Color32::from_rgb(0x42, 0x42, 0x42),
                card_fill: Color32::from_rgb(0x1e, 0x1e, 0x1e),
                card_inner: Color32::from_rgb(0x18, 0x18, 0x18),
                is_dark: true,
            }
        } else {
            Self {
                // Pure PhotoCraft light palette: clear light surfaces, no dark patches
                bg_fill: Color32::from_rgb(0xee, 0xee, 0xee),
                panel_fill: Color32::from_rgb(0xf8, 0xf8, 0xf8),
                input_fill: Color32::from_rgb(0xff, 0xff, 0xff),
                border_stroke: Color32::from_rgb(0xd2, 0xd2, 0xd2),
                text_primary: Color32::from_rgb(0x1e, 0x1e, 0x1e),
                text_muted: Color32::from_rgb(0x75, 0x75, 0x75),
                accent_blue: Color32::from_rgb(0x2a, 0x82, 0xe4),
                accent_hover: Color32::from_rgb(0x1e, 0x6e, 0xcd),
                button_secondary: Color32::from_rgb(0xe0, 0xe0, 0xe0),
                button_secondary_hover: Color32::from_rgb(0xd4, 0xd4, 0xd4),
                card_fill: Color32::from_rgb(0xff, 0xff, 0xff),
                card_inner: Color32::from_rgb(0xf5, 0xf5, 0xf5),
                is_dark: false,
            }
        }
    }

    pub fn to_egui_visuals(&self) -> Visuals {
        let mut v = if self.is_dark {
            Visuals::dark()
        } else {
            Visuals::light()
        };

        v.panel_fill = self.bg_fill;
        v.window_fill = self.panel_fill;
        v.window_stroke = Stroke::new(1.0, self.border_stroke);
        v.faint_bg_color = self.input_fill;
        v.extreme_bg_color = self.input_fill;

        // Flat, crisp desktop controls
        v.widgets.noninteractive.bg_fill = self.panel_fill;
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.border_stroke);
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.text_primary);
        v.widgets.noninteractive.corner_radius = 3.0.into();

        v.widgets.inactive.bg_fill = self.button_secondary;
        v.widgets.inactive.bg_stroke = Stroke::new(1.0, self.border_stroke);
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, self.text_primary);
        v.widgets.inactive.corner_radius = 3.0.into();

        v.widgets.hovered.bg_fill = self.button_secondary_hover;
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, self.accent_blue);
        v.widgets.hovered.fg_stroke = Stroke::new(1.0, self.text_primary);
        v.widgets.hovered.corner_radius = 3.0.into();

        v.widgets.active.bg_fill = self.accent_blue;
        v.widgets.active.bg_stroke = Stroke::new(1.0, self.accent_hover);
        v.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
        v.widgets.active.corner_radius = 3.0.into();

        v.widgets.open.bg_fill = self.panel_fill;
        v.widgets.open.bg_stroke = Stroke::new(1.0, self.accent_blue);
        v.widgets.open.fg_stroke = Stroke::new(1.0, self.text_primary);
        v.widgets.open.corner_radius = 3.0.into();

        v.menu_corner_radius = 4.0.into();
        v.popup_shadow = egui::epaint::Shadow::NONE;

        v.selection.bg_fill = self.accent_blue;
        v.selection.stroke = Stroke::new(1.0, Color32::WHITE);

        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_mode_is_dark() {
        assert!(ThemeMode::Dark.is_dark());
        assert!(!ThemeMode::Light.is_dark());
    }

    #[test]
    fn test_theme_mode_from_preferences() {
        let temp_dir = std::env::temp_dir().join(format!("theme_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&temp_dir);

        // Light themes
        let light_prefs = temp_dir.join("light.json");
        std::fs::write(&light_prefs, r#"{"interface": {"theme": "studioLight"}}"#).unwrap();
        assert_eq!(ThemeMode::from_preferences(&light_prefs), ThemeMode::Light);

        let classic_prefs = temp_dir.join("classic.json");
        std::fs::write(&classic_prefs, r#"{"interface": {"theme": "classic"}}"#).unwrap();
        assert_eq!(ThemeMode::from_preferences(&classic_prefs), ThemeMode::Light);

        // Dark theme
        let dark_prefs = temp_dir.join("dark.json");
        std::fs::write(&dark_prefs, r#"{"interface": {"theme": "studioDark"}}"#).unwrap();
        assert_eq!(ThemeMode::from_preferences(&dark_prefs), ThemeMode::Dark);

        // Auto/System
        let auto_prefs = temp_dir.join("auto.json");
        std::fs::write(&auto_prefs, r#"{"interface": {"theme": "auto"}}"#).unwrap();
        assert_eq!(ThemeMode::from_preferences(&auto_prefs), ThemeMode::System);

        // Non-existent or invalid file defaults to Dark
        let missing = temp_dir.join("missing.json");
        assert_eq!(ThemeMode::from_preferences(&missing), ThemeMode::Dark);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_theme_palette_dark_and_light() {
        let dark_palette = ThemePalette::for_mode(ThemeMode::Dark);
        assert!(dark_palette.is_dark);
        assert_eq!(dark_palette.panel_fill, Color32::from_rgb(0x2d, 0x2d, 0x2d));

        let light_palette = ThemePalette::for_mode(ThemeMode::Light);
        assert!(!light_palette.is_dark);
        assert_eq!(light_palette.panel_fill, Color32::from_rgb(0xf8, 0xf8, 0xf8));

        // Visuals check
        let dark_visuals = dark_palette.to_egui_visuals();
        assert!(dark_visuals.dark_mode);
        assert_eq!(dark_visuals.window_fill, dark_palette.panel_fill);

        let light_visuals = light_palette.to_egui_visuals();
        assert!(!light_visuals.dark_mode);
        assert_eq!(light_visuals.window_fill, light_palette.panel_fill);
    }
}

