use resvg::tiny_skia;
use resvg::usvg;
use std::collections::HashMap;
use std::path::Path;

pub const ICON_DARK_SVG: &str = include_str!("../../../assets/icon-dark.svg");
pub const ICON_LIGHT_SVG: &str = include_str!("../../../assets/icon_light.svg");

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

        let adapted = adapt_svg_for_theme(svg_str, is_dark);
        // Render at 48x48 for crisp rendering on HiDPI displays
        let rgba = render_svg_icon(&adapted, 48, 48)?;
        let color_image = egui::ColorImage::from_rgba_unmultiplied([48, 48], &rgba);
        let tex_name = format!("icon_{}_{}", action, if is_dark { "dark" } else { "light" });
        let tex = ctx.load_texture(tex_name, color_image, egui::TextureOptions::LINEAR);
        self.textures.insert(action.to_string(), tex.clone());
        Some(tex)
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
}
