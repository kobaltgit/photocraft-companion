use super::super::*;
use eframe::egui::{self, Color32, RichText, Stroke, Ui, Vec2};

impl PluginManagerApp {
    /// Extracts lines corresponding to the requested language from a bilingual README
    pub fn extract_readme_lines(raw: &str, lang: Language) -> Vec<&str> {
        let ru_marker = "**\u{0420}\u{0443}\u{0441}\u{0441}\u{043A}\u{0438}\u{0439}:**";
        let en_marker = "**English:**";

        let has_ru = raw.contains(ru_marker);
        let has_en = raw.contains(en_marker);

        if !has_ru && !has_en {
            return raw.lines().collect();
        }

        let mut result = Vec::new();
        let lines: Vec<&str> = raw.lines().collect();

        // 1. Collect top heading lines before any language sections
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with(en_marker) || trimmed.starts_with(ru_marker) {
                break;
            }
            if trimmed.starts_with("# ") {
                result.push(*line);
            }
        }

        // 2. Collect lines for the active language
        let target_marker = match lang {
            Language::Ru => ru_marker,
            _ => en_marker,
        };

        let mut in_block = false;
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with(target_marker) {
                in_block = true;
                continue;
            }
            if in_block {
                if trimmed.starts_with(en_marker) || trimmed.starts_with(ru_marker) {
                    break;
                }
                if trimmed == "---" {
                    break;
                }
                result.push(*line);
            }
        }

        if result.is_empty() {
            raw.lines().collect()
        } else {
            result
        }
    }

    /// Renders markdown body elements
    pub fn render_readme_content(
        ui: &mut Ui,
        raw_content: &str,
        lang: Language,
        palette: &ThemePalette,
    ) {
        let lines = Self::extract_readme_lines(raw_content, lang);

        for line in lines {
            let trimmed = line.trim();

            // Skip shields.io / badges
            if trimmed.starts_with("![")
                || (trimmed.starts_with("[![") && trimmed.contains("](http"))
            {
                continue;
            }

            if trimmed.is_empty() {
                ui.add_space(4.0);
                continue;
            }

            if trimmed == "---" {
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);
                continue;
            }

            // H1 title
            if let Some(h1) = trimmed.strip_prefix("# ") {
                ui.label(
                    RichText::new(h1.trim())
                        .size(15.0)
                        .strong()
                        .color(palette.text_primary),
                );
                ui.add_space(4.0);
                continue;
            }

            // H2 / H3 subheader
            if let Some(h3) = trimmed
                .strip_prefix("### ")
                .or_else(|| trimmed.strip_prefix("## "))
            {
                ui.add_space(6.0);
                ui.label(
                    RichText::new(h3.trim())
                        .size(13.0)
                        .strong()
                        .color(palette.accent_blue),
                );
                ui.add_space(3.0);
                continue;
            }

            // Bullet list item with vector circle marker
            if let Some(bullet_text) = trimmed.strip_prefix("- ") {
                ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    let (dot_rect, _) =
                        ui.allocate_exact_size(Vec2::new(8.0, 14.0), egui::Sense::hover());
                    ui.painter()
                        .circle_filled(dot_rect.center(), 2.0, palette.accent_blue);
                    Self::render_formatted_inline_text(ui, bullet_text.trim(), palette);
                });
                ui.add_space(2.0);
                continue;
            }

            // Regular paragraph line
            Self::render_formatted_inline_text(ui, trimmed, palette);
            ui.add_space(2.0);
        }
    }

    /// Renders inline formatting (hyperlinks, `code`, **bold**, plain text)
    pub fn render_formatted_inline_text(ui: &mut Ui, text: &str, palette: &ThemePalette) {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            let mut remaining = text;

            while !remaining.is_empty() {
                if let Some(bracket_start) = remaining.find('[') {
                    if let Some(bracket_end) = remaining[bracket_start..].find("](") {
                        let full_bracket_end = bracket_start + bracket_end;
                        if let Some(paren_end) = remaining[full_bracket_end + 2..].find(')') {
                            let full_paren_end = full_bracket_end + 2 + paren_end;

                            if bracket_start > 0 {
                                Self::render_plain_inline(ui, &remaining[..bracket_start], palette);
                            }

                            let link_label = &remaining[bracket_start + 1..full_bracket_end];
                            let link_url = &remaining[full_bracket_end + 2..full_paren_end];
                            ui.hyperlink_to(link_label, link_url);

                            remaining = &remaining[full_paren_end + 1..];
                            continue;
                        }
                    }
                }

                Self::render_plain_inline(ui, remaining, palette);
                break;
            }
        });
    }

    /// Helper for plain, code and bold segments
    pub fn render_plain_inline(ui: &mut Ui, mut text: &str, palette: &ThemePalette) {
        while !text.is_empty() {
            let next_backtick = text.find('`');
            let next_bold = text.find("**");

            match (next_backtick, next_bold) {
                (Some(bt), Some(bd)) if bt < bd => {
                    if bt > 0 {
                        ui.label(
                            RichText::new(&text[..bt])
                                .size(11.0)
                                .color(palette.text_primary),
                        );
                    }
                    let rest = &text[bt + 1..];
                    if let Some(closing_bt) = rest.find('`') {
                        let code = &rest[..closing_bt];
                        ui.label(
                            RichText::new(code)
                                .size(11.0)
                                .monospace()
                                .color(palette.accent_blue)
                                .background_color(palette.card_inner),
                        );
                        text = &rest[closing_bt + 1..];
                    } else {
                        ui.label(
                            RichText::new(&text[bt..])
                                .size(11.0)
                                .color(palette.text_primary),
                        );
                        break;
                    }
                }
                (Some(bt), None) => {
                    if bt > 0 {
                        ui.label(
                            RichText::new(&text[..bt])
                                .size(11.0)
                                .color(palette.text_primary),
                        );
                    }
                    let rest = &text[bt + 1..];
                    if let Some(closing_bt) = rest.find('`') {
                        let code = &rest[..closing_bt];
                        ui.label(
                            RichText::new(code)
                                .size(11.0)
                                .monospace()
                                .color(palette.accent_blue)
                                .background_color(palette.card_inner),
                        );
                        text = &rest[closing_bt + 1..];
                    } else {
                        ui.label(
                            RichText::new(&text[bt..])
                                .size(11.0)
                                .color(palette.text_primary),
                        );
                        break;
                    }
                }
                (_, Some(bd)) => {
                    if bd > 0 {
                        ui.label(
                            RichText::new(&text[..bd])
                                .size(11.0)
                                .color(palette.text_primary),
                        );
                    }
                    let rest = &text[bd + 2..];
                    if let Some(closing_bd) = rest.find("**") {
                        let bold = &rest[..closing_bd];
                        ui.label(
                            RichText::new(bold)
                                .size(11.0)
                                .strong()
                                .color(palette.text_primary),
                        );
                        text = &rest[closing_bd + 2..];
                    } else {
                        ui.label(
                            RichText::new(&text[bd..])
                                .size(11.0)
                                .color(palette.text_primary),
                        );
                        break;
                    }
                }
                (None, None) => {
                    ui.label(RichText::new(text).size(11.0).color(palette.text_primary));
                    break;
                }
            }
        }
    }

    /// Renders modal window for viewing plugin documentation
    pub fn render_readme_modal(&mut self, ctx: &egui::Context, palette: &ThemePalette) {
        if self.readme_modal.is_none() {
            return;
        }

        let title_text = self.t("Plugin Documentation", "Документация плагина");
        let ru_btn_text = self.t("Russian", "Русский");
        let en_btn_text = self.t("English", "Английский");
        let open_folder_text = self.t("Open Plugin Folder", "Открыть папку плагина");
        let close_btn_text = self.t("Close", "Закрыть");

        let mut close_modal = false;
        let mut open_folder = None;

        if let Some(ref mut state) = self.readme_modal {
            let mut is_open = true;
            egui::Window::new(title_text)
                .open(&mut is_open)
                .collapsible(false)
                .resizable(true)
                .default_size(Vec2::new(580.0, 440.0))
                .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
                .frame(
                    egui::Frame::new()
                        .fill(palette.panel_fill)
                        .stroke(Stroke::new(1.0, palette.border_stroke))
                        .corner_radius(8.0)
                        .inner_margin(12.0),
                )
                .show(ctx, |ui| {
                    // Header: Plugin Name + Language Switcher
                    ui.horizontal(|ui| {
                        ui.heading(
                            RichText::new(&state.plugin_name)
                                .size(15.0)
                                .strong()
                                .color(palette.text_primary),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let is_ru = state.active_lang == Language::Ru;
                            let ru_btn = egui::Button::new(
                                RichText::new(ru_btn_text)
                                    .size(11.0)
                                    .strong()
                                    .color(if is_ru {
                                        Color32::WHITE
                                    } else {
                                        palette.text_primary
                                    }),
                            )
                            .fill(if is_ru {
                                palette.accent_blue
                            } else {
                                palette.button_secondary
                            })
                            .corner_radius(4.0)
                            .min_size(Vec2::new(65.0, 22.0));

                            if ui.add(ru_btn).clicked() {
                                state.active_lang = Language::Ru;
                            }

                            ui.add_space(4.0);

                            let is_en = state.active_lang != Language::Ru;
                            let en_btn = egui::Button::new(
                                RichText::new(en_btn_text)
                                    .size(11.0)
                                    .strong()
                                    .color(if is_en {
                                        Color32::WHITE
                                    } else {
                                        palette.text_primary
                                    }),
                            )
                            .fill(if is_en {
                                palette.accent_blue
                            } else {
                                palette.button_secondary
                            })
                            .corner_radius(4.0)
                            .min_size(Vec2::new(65.0, 22.0));

                            if ui.add(en_btn).clicked() {
                                state.active_lang = Language::En;
                            }
                        });
                    });

                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);

                    // Body: Scrollable area with markdown scanner
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .max_height(330.0)
                        .show(ui, |ui| {
                            Self::render_readme_content(
                                ui,
                                &state.raw_content,
                                state.active_lang,
                                palette,
                            );
                        });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Footer: Open Plugin Folder and Close button
                    ui.horizontal(|ui| {
                        let folder_btn = egui::Button::new(
                            RichText::new(open_folder_text)
                                .size(11.0)
                                .color(palette.text_primary),
                        )
                        .fill(palette.button_secondary)
                        .corner_radius(4.0)
                        .min_size(Vec2::new(140.0, 24.0));

                        if ui.add(folder_btn).clicked() {
                            open_folder = Some(state.plugin_dir.clone());
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let close_btn = egui::Button::new(
                                RichText::new(close_btn_text)
                                    .size(11.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            )
                            .fill(palette.accent_blue)
                            .corner_radius(4.0)
                            .min_size(Vec2::new(76.0, 24.0));

                            if ui.add(close_btn).clicked() {
                                close_modal = true;
                            }
                        });
                    });
                });

            if !is_open || close_modal {
                self.readme_modal = None;
            }
        }

        if let Some(folder) = open_folder {
            crate::updater::reveal_in_folder(&folder);
        }
    }
}
