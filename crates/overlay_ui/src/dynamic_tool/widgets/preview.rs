use crate::dynamic_tool::state::{DynamicValue, DynamicWindowState};
use crate::theme::ThemePalette;
use core_contracts::{Language, WidgetNode};
use eframe::egui::{self, Color32, Sense, Stroke, Vec2};
use std::collections::HashMap;

pub use super::preview_params::*;

#[allow(clippy::too_many_arguments)]
pub fn render_image_preview(
    ui: &mut egui::Ui,
    plugin_id: &str,
    node: &WidgetNode,
    state: &mut DynamicWindowState,
    palette: &ThemePalette,
    lang: Language,
    preview_thumbnails: &mut HashMap<String, (image::RgbaImage, f32)>,
    preview_textures: &mut HashMap<String, (egui::TextureHandle, u64)>,
) -> bool {
    let mut changed = false;

    if let WidgetNode::ImagePreview { height, .. } = node {
        let preview_h = (*height as f32).max(60.0);
        let preview_w = ui.available_width();
        let (rect, resp) =
            ui.allocate_exact_size(Vec2::new(preview_w, preview_h), Sense::click_and_drag());

        let is_model_missing = state
            .values
            .get("model_missing")
            .or_else(|| state.values.get("models_missing"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

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
            let (hash, rendered) =
                if let Some(lens_params) = extract_lens_blur_params(&state.values) {
                    let scaled_params = lens_params.scaled(*scale_k);
                    let h = hash_lens_blur_params(&scaled_params, *scale_k);
                    let needs_render = match preview_textures.get(plugin_id) {
                        Some((_, cached_hash)) => *cached_hash != h,
                        None => true,
                    };
                    let rend = if needs_render {
                        let default_depth = image::GrayImage::from_pixel(
                            thumbnail.width(),
                            thumbnail.height(),
                            image::Luma([128]),
                        );
                        let depth = state.role_mask.as_ref().unwrap_or(&default_depth);
                        Some(raster_engine::render_lens_blur_preview(
                            thumbnail,
                            depth,
                            &scaled_params,
                        ))
                    } else {
                        None
                    };
                    (h, rend)
                } else {
                    let raw_params = extract_extrude_params(&state.values);
                    let scaled_params = raw_params.scaled(*scale_k);
                    let h = hash_extrude_params(&scaled_params, *scale_k);
                    let needs_render = match preview_textures.get(plugin_id) {
                        Some((_, cached_hash)) => *cached_hash != h,
                        None => true,
                    };
                    let rend = if needs_render {
                        Some(raster_engine::render_extrude_preview(
                            thumbnail,
                            &scaled_params,
                        ))
                    } else {
                        None
                    };
                    (h, rend)
                };

            if let Some(img) = rendered {
                let (rw, rh) = img.dimensions();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(
                    [rw as usize, rh as usize],
                    img.as_raw(),
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
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );

                    // Click-to-Focus interaction
                    if let Some(pos) = resp.interact_pointer_pos() {
                        if (resp.clicked() || resp.dragged())
                            && draw_rect.contains(pos)
                            && draw_rect.width() > 0.0
                            && draw_rect.height() > 0.0
                        {
                            let u = ((pos.x - draw_rect.min.x) / draw_rect.width()).clamp(0.0, 1.0);
                            let v =
                                ((pos.y - draw_rect.min.y) / draw_rect.height()).clamp(0.0, 1.0);
                            if let Some(ref mask) = state.role_mask {
                                let (mw, mh) = mask.dimensions();
                                if mw > 0 && mh > 0 {
                                    let px = ((u * (mw - 1) as f32).round() as u32).min(mw - 1);
                                    let py = ((v * (mh - 1) as f32).round() as u32).min(mh - 1);
                                    let val = mask.get_pixel(px, py)[0];
                                    state.values.insert(
                                        "focus_distance".to_string(),
                                        DynamicValue::Number(val as f64),
                                    );
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }
        } else if !is_model_missing {
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

        // Missing models vector warning badge
        if is_model_missing {
            let center = rect.center();
            let badge_text = match lang {
                Language::Ru => "Модели нейросети не установлены",
                _ => "AI models are not installed",
            };
            let font_id = egui::FontId::proportional(12.0);
            let badge_galley = ui.painter().layout_no_wrap(
                badge_text.to_string(),
                font_id.clone(),
                Color32::from_rgb(254, 243, 199),
            );
            let text_w = badge_galley.size().x;
            let badge_w = (text_w + 44.0).min(rect.width() - 16.0).max(120.0);
            let badge_h = 32.0;
            let badge_rect = egui::Rect::from_center_size(center, Vec2::new(badge_w, badge_h));

            let bg_amber = Color32::from_rgba_unmultiplied(45, 25, 10, 230);
            let stroke_amber = Color32::from_rgb(245, 158, 11);
            ui.painter().rect_filled(badge_rect, 6.0, bg_amber);
            ui.painter().rect_stroke(
                badge_rect,
                6.0,
                Stroke::new(1.2, stroke_amber),
                egui::StrokeKind::Inside,
            );

            // Vector exclamation mark icon
            let icon_c = egui::pos2(badge_rect.min.x + 16.0, center.y);
            ui.painter()
                .circle_stroke(icon_c, 6.5, Stroke::new(1.3, stroke_amber));
            ui.painter().line_segment(
                [
                    egui::pos2(icon_c.x, icon_c.y - 3.2),
                    egui::pos2(icon_c.x, icon_c.y + 0.6),
                ],
                Stroke::new(1.5, stroke_amber),
            );
            ui.painter()
                .circle_filled(egui::pos2(icon_c.x, icon_c.y + 3.2), 1.0, stroke_amber);

            // Badge text
            ui.painter().text(
                egui::pos2(badge_rect.min.x + 28.0, center.y),
                egui::Align2::LEFT_CENTER,
                badge_text,
                font_id,
                Color32::from_rgb(254, 243, 199),
            );
        }

        // 3. Busy indicator (pulsing vector circle)
        if state
            .values
            .get("is_busy")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            let center = rect.center();
            let time = ui.input(|i| i.time);
            let pulse = (time * 3.5).sin() * 5.0;
            ui.painter().circle_stroke(
                center,
                (24.0 + pulse as f32).max(10.0),
                Stroke::new(2.5, palette.accent_blue),
            );
            let busy_text = match lang {
                Language::Ru => "Анализ сцены...",
                _ => "Analyzing scene...",
            };
            let font_id = egui::FontId::proportional(12.0);
            ui.painter().text(
                center + Vec2::new(0.0, 42.0),
                egui::Align2::CENTER_CENTER,
                busy_text,
                font_id,
                palette.text_primary,
            );
            ui.ctx().request_repaint();
        }

        // 4. Border frame
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
                let angle =
                    std::f32::consts::PI * 0.25 + (i as f32 / 12.0) * (std::f32::consts::PI * 1.5);
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
            state.role_mask = None;
            ui.ctx().request_repaint();
            changed = true;
        }
    }

    changed
}
