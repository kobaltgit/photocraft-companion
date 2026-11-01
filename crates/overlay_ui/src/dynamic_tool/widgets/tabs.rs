use crate::dynamic_tool::state::{DynamicValue, DynamicWindowState};
use crate::theme::ThemePalette;
use core_contracts::{Language, WidgetNode};
use eframe::egui::{self, Color32, Stroke, Vec2};
use std::collections::HashMap;

#[allow(clippy::too_many_arguments)]
pub fn render_tabs<F>(
    ui: &mut egui::Ui,
    plugin_id: &str,
    node: &WidgetNode,
    state: &mut DynamicWindowState,
    palette: &ThemePalette,
    lang: Language,
    preview_thumbnails: &mut HashMap<String, (image::RgbaImage, f32)>,
    preview_textures: &mut HashMap<String, (egui::TextureHandle, u64)>,
    config_path: Option<&std::path::Path>,
    mut render_child: F,
) -> bool
where
    F: FnMut(
        &mut egui::Ui,
        &str,
        &WidgetNode,
        &mut DynamicWindowState,
        &ThemePalette,
        Language,
        &mut HashMap<String, (image::RgbaImage, f32)>,
        &mut HashMap<String, (egui::TextureHandle, u64)>,
        Option<&std::path::Path>,
    ) -> bool,
{
    let mut changed = false;

    if let WidgetNode::Tabs {
        id,
        tabs,
        default_index,
    } = node
    {
        if tabs.is_empty() {
            return false;
        }

        let mut active_idx = match state.values.get(id) {
            Some(DynamicValue::Index(i)) => *i,
            _ => *default_index,
        };

        if active_idx >= tabs.len() {
            active_idx = 0;
        }

        // Horizontal pill buttons bar
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);
            ui.spacing_mut().button_padding = Vec2::new(14.0, 5.0);
            for (idx, tab) in tabs.iter().enumerate() {
                let is_active = idx == active_idx;
                let title = tab.title_for_lang(lang);

                let text = egui::RichText::new(title).size(12.0).color(if is_active {
                    Color32::WHITE
                } else {
                    palette.text_muted
                });

                let fill = if is_active {
                    palette.accent_blue
                } else {
                    palette.card_fill
                };

                let stroke = if is_active {
                    Stroke::new(1.0, palette.accent_blue)
                } else {
                    Stroke::new(1.0, palette.border_stroke)
                };

                let btn = egui::Button::new(text)
                    .fill(fill)
                    .stroke(stroke)
                    .corner_radius(4.0)
                    .min_size(Vec2::new(0.0, 26.0));

                if ui.add(btn).clicked() && !is_active {
                    state.values.insert(id.clone(), DynamicValue::Index(idx));
                    active_idx = idx;
                    changed = true;
                }
            }
        });

        ui.add_space(4.0);

        // Render widgets of active tab
        if let Some(active_tab) = tabs.get(active_idx) {
            for child in &active_tab.widgets {
                if render_child(
                    ui,
                    plugin_id,
                    child,
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
        }
    }

    changed
}
