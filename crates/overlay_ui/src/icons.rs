use resvg::tiny_skia;
use resvg::usvg;
use std::collections::HashMap;
use std::path::Path;

pub const ICON_DARK_SVG: &str = include_str!("../../../assets/icon-dark.svg");
pub const ICON_LIGHT_SVG: &str = include_str!("../../../assets/icon_light.svg");

pub const DISCORD_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="256" height="199" viewBox="0 0 256 199"><path fill="currentColor" d="M216.856 16.597A208.5 208.5 0 0 0 164.042 0c-2.275 4.113-4.933 9.645-6.766 14.046q-29.538-4.442-58.533 0c-1.832-4.4-4.55-9.933-6.846-14.046a207.8 207.8 0 0 0-52.855 16.638C5.618 67.147-3.443 116.4 1.087 164.956c22.169 16.555 43.653 26.612 64.775 33.193A161 161 0 0 0 79.735 175.3a136.4 136.4 0 0 1-21.846-10.632a109 109 0 0 0 5.356-4.237c42.122 19.702 87.89 19.702 129.51 0a132 132 0 0 0 5.355 4.237a136 136 0 0 1-21.886 10.653c4.006 8.02 8.638 15.67 13.873 22.848c21.142-6.58 42.646-16.637 64.815-33.213c5.316-56.288-9.08-105.09-38.056-148.36ZM85.474 135.095c-12.645 0-23.015-11.805-23.015-26.18s10.149-26.2 23.015-26.2s23.236 11.804 23.015 26.2c.02 14.375-10.148 26.18-23.015 26.18m85.051 0c-12.645 0-23.014-11.805-23.014-26.18s10.148-26.2 23.014-26.2c12.867 0 23.236 11.804 23.015 26.2c0 14.375-10.148 26.18-23.015 26.18"/></svg>"##;
pub const DISCORD_COLOR_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="256" height="199" viewBox="0 0 256 199"><path fill="#5865f2" d="M216.856 16.597A208.5 208.5 0 0 0 164.042 0c-2.275 4.113-4.933 9.645-6.766 14.046q-29.538-4.442-58.533 0c-1.832-4.4-4.55-9.933-6.846-14.046a207.8 207.8 0 0 0-52.855 16.638C5.618 67.147-3.443 116.4 1.087 164.956c22.169 16.555 43.653 26.612 64.775 33.193A161 161 0 0 0 79.735 175.3a136.4 136.4 0 0 1-21.846-10.632a109 109 0 0 0 5.356-4.237c42.122 19.702 87.89 19.702 129.51 0a132 132 0 0 0 5.355 4.237a136 136 0 0 1-21.886 10.653c4.006 8.02 8.638 15.67 13.873 22.848c21.142-6.58 42.646-16.637 64.815-33.213c5.316-56.288-9.08-105.09-38.056-148.36ZM85.474 135.095c-12.645 0-23.015-11.805-23.015-26.18s10.149-26.2 23.015-26.2s23.236 11.804 23.015 26.2c.02 14.375-10.148 26.18-23.015 26.18m85.051 0c-12.645 0-23.014-11.805-23.014-26.18s10.148-26.2 23.014-26.2c12.867 0 23.236 11.804 23.015 26.2c0 14.375-10.148 26.18-23.015 26.18"/></svg>"##;

/// Theme adapter for SVG content: replaces currentColor and adapts monochrome shapes
pub fn adapt_svg_for_theme(svg_data: &str, is_dark: bool) -> String {
    let main_color = if is_dark { "#E6E6E6" } else { "#1E1E1E" };
    let mut s = svg_data.replace("currentColor", main_color);
    if is_dark {
        // If author created icon with black strokes, make them visible on dark panel
        s = s
            .replace("#000000", main_color)
            .replace("#000\"", &format!("{}\"", main_color))
            .replace("\"black\"", &format!("\"{}\"", main_color));
    } else {
        // If author created icon with white strokes, make them visible on light panel
        s = s
            .replace("#ffffff", main_color)
            .replace("#FFFFFF", main_color)
            .replace("#fff\"", &format!("{}\"", main_color))
            .replace("\"white\"", &format!("\"{}\"", main_color));
    }
    s
}

pub fn render_svg_icon(svg_data: &str, width: u32, height: u32) -> Option<Vec<u8>> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_str(svg_data, &opt).ok()?;
    let mut pixmap = tiny_skia::Pixmap::new(width, height)?;

    let sx = width as f32 / tree.size().width();
    let sy = height as f32 / tree.size().height();
    let transform = tiny_skia::Transform::from_scale(sx, sy);

    resvg::render(&tree, transform, &mut pixmap.as_mut());
    Some(pixmap.take())
}

pub fn get_icon_rgba(is_dark: bool, size: u32) -> Option<Vec<u8>> {
    let svg = if is_dark {
        ICON_DARK_SVG
    } else {
        ICON_LIGHT_SVG
    };
    render_svg_icon(svg, size, size)
}

pub fn get_egui_icon_data(is_dark: bool, size: u32) -> Option<egui::IconData> {
    let rgba = get_icon_rgba(is_dark, size)?;
    Some(egui::IconData {
        rgba,
        width: size,
        height: size,
    })
}

pub fn get_tray_icon(is_dark: bool, size: u32) -> Option<tray_icon::Icon> {
    let rgba = get_icon_rgba(is_dark, size)?;
    tray_icon::Icon::from_rgba(rgba, size, size).ok()
}

/// Dynamic cache of parsed and GPU-uploaded SVG textures for overlay buttons
#[derive(Default)]
pub struct SvgIconCache {
    pub textures: HashMap<String, egui::TextureHandle>,
    pub last_is_dark: Option<bool>,
}

impl SvgIconCache {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            last_is_dark: None,
        }
    }

    /// Load or retrieve cached texture for an action from file path
    pub fn get_or_load(
        &mut self,
        ctx: &egui::Context,
        action: &str,
        svg_path: Option<&Path>,
        is_dark: bool,
    ) -> Option<egui::TextureHandle> {
        // Invalidate cache if theme changed
        if self.last_is_dark != Some(is_dark) {
            self.textures.clear();
            self.last_is_dark = Some(is_dark);
        }

        if let Some(tex) = self.textures.get(action) {
            return Some(tex.clone());
        }

        let path = svg_path?;
        if !path.is_file() {
            return None;
        }

        let content = std::fs::read_to_string(path).ok()?;
        self.load_from_svg_str(ctx, action, &content, is_dark)
    }

    /// Helper to render SVG string, upload to egui context, and store in cache
    pub fn load_from_svg_str(
        &mut self,
        ctx: &egui::Context,
        action: &str,
        svg_str: &str,
        is_dark: bool,
    ) -> Option<egui::TextureHandle> {
        if self.last_is_dark != Some(is_dark) {
            self.textures.clear();
            self.last_is_dark = Some(is_dark);
        }

        if let Some(tex) = self.textures.get(action) {
            return Some(tex.clone());
        }

        let adapted = adapt_svg_for_theme(svg_str, is_dark);
        // Render at 48x48 for crisp rendering on HiDPI displays
        let rgba = render_svg_icon(&adapted, 48, 48)?;
        let color_image = egui::ColorImage::from_rgba_unmultiplied([48, 48], &rgba);
        let tex_name = format!("icon_{}_{}", action, if is_dark { "dark" } else { "light" });
        let tex = ctx.load_texture(tex_name, color_image, egui::TextureOptions::LINEAR);
        self.textures.insert(action.to_string(), tex.clone());
        Some(tex)
    }

    /// Load or retrieve cached texture for the Discord community button
    pub fn get_or_load_discord(
        &mut self,
        ctx: &egui::Context,
        is_hovered: bool,
        is_dark: bool,
    ) -> Option<egui::TextureHandle> {
        let action = if is_hovered {
            "discord_hover"
        } else {
            "discord_idle"
        };
        let svg = if is_hovered {
            DISCORD_COLOR_SVG
        } else {
            DISCORD_SVG
        };
        self.load_from_svg_str(ctx, action, svg, is_dark)
    }

    pub fn clear(&mut self) {
        self.textures.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_svg_icon_valid() {
        let rgba = render_svg_icon(ICON_DARK_SVG, 32, 32);
        assert!(rgba.is_some());
        let bytes = rgba.unwrap();
        assert_eq!(bytes.len(), 32 * 32 * 4);
    }

    #[test]
    fn test_render_svg_icon_invalid() {
        assert!(render_svg_icon("not an svg", 32, 32).is_none());
        assert!(render_svg_icon("", 32, 32).is_none());
    }

    #[test]
    fn test_get_icon_rgba() {
        let dark = get_icon_rgba(true, 16);
        assert!(dark.is_some());
        assert_eq!(dark.unwrap().len(), 16 * 16 * 4);

        let light = get_icon_rgba(false, 16);
        assert!(light.is_some());
        assert_eq!(light.unwrap().len(), 16 * 16 * 4);
    }

    #[test]
    fn test_get_egui_icon_data() {
        let icon_data = get_egui_icon_data(true, 24);
        assert!(icon_data.is_some());
        let data = icon_data.unwrap();
        assert_eq!(data.width, 24);
        assert_eq!(data.height, 24);
        assert_eq!(data.rgba.len(), 24 * 24 * 4);
    }

    #[test]
    fn test_get_tray_icon() {
        let tray_icon = get_tray_icon(false, 32);
        assert!(tray_icon.is_some());
    }

    #[test]
    fn test_adapt_svg_for_theme() {
        let sample = r##"<svg><path stroke="currentColor" fill="#ffffff" /></svg>"##;
        let dark = adapt_svg_for_theme(sample, true);
        assert!(dark.contains("#E6E6E6"));
        assert!(dark.contains("#ffffff")); // preserved in dark mode

        let light = adapt_svg_for_theme(sample, false);
        assert!(light.contains("#1E1E1E"));
        assert!(!light.contains("#ffffff")); // replaced in light mode
    }

    #[test]
    fn test_svg_icon_cache_lifecycle() {
        let ctx = egui::Context::default();
        let mut cache = SvgIconCache::new();
        let sample_svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><circle cx="12" cy="12" r="10" stroke="currentColor" fill="none"/></svg>"#;

        // Load in dark mode
        let t1 = cache.load_from_svg_str(&ctx, "test_action", sample_svg, true);
        assert!(t1.is_some());
        assert_eq!(cache.textures.len(), 1);
        assert_eq!(cache.last_is_dark, Some(true));

        // Switch to light mode: cache must be invalidated and re-populated
        let t2 = cache.load_from_svg_str(&ctx, "test_action", sample_svg, false);
        assert!(t2.is_some());
        assert_eq!(cache.textures.len(), 1);
        assert_eq!(cache.last_is_dark, Some(false));
    }

    #[test]
    fn test_render_discord_svg_valid() {
        let rgba_idle = render_svg_icon(DISCORD_SVG, 24, 24);
        assert!(rgba_idle.is_some());
        assert_eq!(rgba_idle.unwrap().len(), 24 * 24 * 4);

        let rgba_color = render_svg_icon(DISCORD_COLOR_SVG, 24, 24);
        assert!(rgba_color.is_some());
        assert_eq!(rgba_color.unwrap().len(), 24 * 24 * 4);
    }

    #[test]
    fn test_discord_cache_get_or_load() {
        let ctx = egui::Context::default();
        let mut cache = SvgIconCache::new();

        let t1 = cache.get_or_load_discord(&ctx, false, true);
        assert!(t1.is_some());
        let t2 = cache.get_or_load_discord(&ctx, true, true);
        assert!(t2.is_some());
        assert_eq!(cache.textures.len(), 2);
    }
}
