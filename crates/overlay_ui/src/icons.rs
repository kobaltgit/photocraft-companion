use resvg::tiny_skia;
use resvg::usvg;

pub const ICON_DARK_SVG: &str = include_str!("../../../assets/icon-dark.svg");
pub const ICON_LIGHT_SVG: &str = include_str!("../../../assets/icon_light.svg");

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
}

