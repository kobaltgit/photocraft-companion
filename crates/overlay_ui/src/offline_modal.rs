use crate::theme::{ThemeMode, ThemePalette};
use core_contracts::{ConnectionState, Language};
use eframe::egui::{self, Color32, RichText, Stroke, Ui, Vec2};

pub struct OfflineModalApp {
    pub current_language: Language,
    pub current_theme: ThemeMode,
    pub connection_state: ConnectionState,
    pub open_manager_requested: bool,
    pub launch_requested: bool,
    pub retry_seconds: f32,
}

impl OfflineModalApp {
    pub fn new(current_language: Language, current_theme: ThemeMode) -> Self {
        Self {
            current_language,
            current_theme,
            connection_state: ConnectionState::Offline,
            open_manager_requested: false,
            launch_requested: false,
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

        let is_auth_req = self.connection_state == ConnectionState::AuthRequired;
        let indicator_color = if is_auth_req {
            Color32::from_rgb(0xea, 0x86, 0x00)
        } else {
            palette.accent_blue
        };

        ui.vertical_centered(|ui| {
            ui.add_space(16.0);

            // Icon / Indicator (clean vector-drawn circular pulse)
            let (circle_rect, _) =
                ui.allocate_exact_size(egui::Vec2::splat(28.0), egui::Sense::hover());
            let cc = circle_rect.center();
            ui.painter()
                .circle_stroke(cc, 11.0, egui::Stroke::new(2.5, indicator_color));
            ui.painter().circle_filled(cc, 5.0, indicator_color);

            ui.add_space(8.0);

            // Title
            let title = if is_auth_req {
                self.t("PhotoCraft Auth Required", "Требуется авторизация")
            } else {
                self.t("PhotoCraft Not Detected", "PhotoCraft не обнаружен")
            };
            ui.heading(
                RichText::new(title)
                    .size(15.0)
                    .strong()
                    .color(palette.text_primary),
            );

            ui.add_space(6.0);

            // Message
            let message = if is_auth_req {
                self.t(
                    "PhotoCraft is running, but requires authentication. Click below to enter your control token.",
                    "PhotoCraft запущен, но требует авторизации. Нажмите ниже, чтобы ввести токен управления.",
                )
            } else {
                self.t(
                    "Please launch PhotoCraft to activate the floating toolstrip and plugins.",
                    "Пожалуйста, запустите PhotoCraft, чтобы панель инструментов и плагины активировались.",
                )
            };
            ui.label(
                RichText::new(message)
                    .size(11.0)
                    .color(palette.text_muted),
            );

            ui.add_space(8.0);

            // Status indicator
            ui.label(
                RichText::new(self.t(
                    "Checking connection every 1.5s...",
                    "Проверка подключения каждые 1.5 сек...",
                ))
                .size(10.0)
                .color(indicator_color),
            );

            ui.add_space(10.0);

            // Primary: Launch PhotoCraft button
            let launch_btn = egui::Button::new(
                RichText::new(self.t("> Launch PhotoCraft", "> Запустить PhotoCraft"))
                    .size(11.0)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(palette.accent_blue)
            .corner_radius(4.0)
            .min_size(Vec2::new(180.0, 26.0));

            if ui.add(launch_btn).clicked() {
                self.launch_requested = true;
            }

            ui.add_space(6.0);

            // Secondary: Open Plugin Manager button
            let btn = egui::Button::new(
                RichText::new(self.t("Open Plugins Manager", "Открыть Менеджер плагинов"))
                    .size(11.0)
                    .color(palette.text_primary),
            )
            .fill(palette.button_secondary)
            .corner_radius(4.0)
            .min_size(Vec2::new(180.0, 24.0));

            if ui.add(btn).clicked() {
                self.open_manager_requested = true;
            }

            ui.add_space(10.0);
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
        assert!(!app.launch_requested);
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
