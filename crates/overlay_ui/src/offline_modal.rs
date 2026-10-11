use crate::theme::{ThemeMode, ThemePalette};
use core_contracts::Language;
use eframe::egui::{self, Color32, RichText, Stroke, Ui, Vec2};

pub struct OfflineModalApp {
    pub current_language: Language,
    pub current_theme: ThemeMode,
    pub open_manager_requested: bool,
    pub retry_seconds: f32,
}

impl OfflineModalApp {
    pub fn new(current_language: Language, current_theme: ThemeMode) -> Self {
        Self {
            current_language,
            current_theme,
            open_manager_requested: false,
            retry_seconds: 1.5,
        }
    }

    pub fn t<'a>(&self, en: &'a str, ru: &'a str) -> &'a str {
        match self.current_language {
            Language::Ru => ru,
            _ => en,
        }
    }

    pub fn render(&mut self, ui: &mut Ui, palette: &ThemePalette) {
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 8.0, palette.panel_fill);
        ui.painter().rect_stroke(
            rect,
            8.0,
            Stroke::new(1.0, palette.border_stroke),
            egui::StrokeKind::Inside,
        );

        ui.vertical_centered(|ui| {
            ui.add_space(16.0);

            // Icon / Indicator
            ui.label(
                RichText::new("⏳")
                    .size(28.0)
                    .color(palette.accent_blue),
            );

            ui.add_space(8.0);

            // Title
            ui.heading(
                RichText::new(self.t("PhotoCraft Not Detected", "PhotoCraft не обнаружен"))
                    .size(15.0)
                    .strong()
                    .color(palette.text_primary),
            );

            ui.add_space(6.0);

            // Message
            ui.label(
                RichText::new(self.t(
                    "Please launch PhotoCraft to activate the floating toolstrip and plugins.",
                    "Пожалуйста, запустите PhotoCraft, чтобы панель инструментов и плагины активировались.",
                ))
                .size(11.0)
                .color(palette.text_muted),
            );

            ui.add_space(10.0);

            // Status indicator
            ui.label(
                RichText::new(self.t(
                    "Checking connection every 1.5s...",
                    "Проверка подключения каждые 1.5 сек...",
                ))
                .size(10.0)
                .color(palette.accent_blue),
            );

            ui.add_space(14.0);

            // Open Plugin Manager button
            let btn = egui::Button::new(
                RichText::new(self.t("Open Plugins Manager", "Открыть Менеджер плагинов"))
                    .size(11.0)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(palette.accent_blue)
            .corner_radius(4.0)
            .min_size(Vec2::new(180.0, 26.0));

            if ui.add(btn).clicked() {
                self.open_manager_requested = true;
            }

            ui.add_space(12.0);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offline_modal_defaults() {
        let app = OfflineModalApp::new(Language::En, ThemeMode::Dark);
        assert_eq!(app.current_language, Language::En);
        assert_eq!(app.current_theme, ThemeMode::Dark);
        assert!(!app.open_manager_requested);
        assert_eq!(app.retry_seconds, 1.5);
    }

    #[test]
    fn test_offline_modal_translations() {
        let app_en = OfflineModalApp::new(Language::En, ThemeMode::Dark);
        assert_eq!(app_en.t("English", "Russian"), "English");

        let app_ru = OfflineModalApp::new(Language::Ru, ThemeMode::Dark);
        assert_eq!(app_ru.t("English", "Russian"), "Russian");

        let app_auto = OfflineModalApp::new(Language::Auto, ThemeMode::Dark);
        assert_eq!(app_auto.t("English", "Russian"), "English");
    }
}
