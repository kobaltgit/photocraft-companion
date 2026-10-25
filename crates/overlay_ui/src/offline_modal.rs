use crate::theme::{ThemeMode, ThemePalette};
use core_contracts::{ConnectionState, Language};
use eframe::egui::{self, Color32, RichText, Stroke, Ui, Vec2};

pub struct OfflineModalApp {
    pub current_language: Language,
    pub current_theme: ThemeMode,
    pub connection_state: ConnectionState,
    pub open_manager_requested: bool,
    pub launch_requested: bool,
    pub hide_requested: bool,
    pub is_pinned: bool,
    pub retry_seconds: f32,
    pub is_process_detected: bool,
    pub resolved_auto_language: Language,
}

impl OfflineModalApp {
    pub fn new(current_language: Language, current_theme: ThemeMode) -> Self {
        Self {
            current_language,
            current_theme,
            connection_state: ConnectionState::Offline,
            open_manager_requested: false,
            launch_requested: false,
            hide_requested: false,
            is_pinned: true,
            retry_seconds: 1.5,
            is_process_detected: false,
            resolved_auto_language: Language::En,
        }
    }

    pub fn effective_language(&self) -> Language {
        match self.current_language {
            Language::Auto => self.resolved_auto_language.safe_ui_language(),
            manual => manual.safe_ui_language(),
        }
    }

    pub fn t<'a>(&self, en: &'a str, ru: &'a str) -> &'a str {
        match self.effective_language() {
            Language::Ru => ru,
            Language::En => en,
            other => crate::i18n::lookup(en, other).unwrap_or(en),
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

        // Header with Drag Handle, Pin Toggle and Minimize to Tray
        ui.horizontal(|ui| {
            ui.add_space(8.0);

            // Left spacer to visually balance right controls
            ui.allocate_exact_size(Vec2::new(36.0, 20.0), egui::Sense::hover());

            // Centered Drag Handle
            let available_drag_w = ui.available_width() - 44.0;
            let (handle_rect, handle_resp) = ui.allocate_exact_size(
                Vec2::new(available_drag_w.max(20.0), 20.0),
                egui::Sense::drag(),
            );
            if handle_resp.drag_started() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            let hc = handle_rect.center();
            ui.painter().line_segment(
                [egui::pos2(hc.x - 14.0, hc.y), egui::pos2(hc.x + 14.0, hc.y)],
                Stroke::new(1.5, palette.border_stroke),
            );

            // Right header controls: Pin & Minimize
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(6.0);

                // Minimize to tray button
                let (min_rect, min_resp) =
                    ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::click());
                if min_resp.clicked() {
                    self.hide_requested = true;
                }
                let min_hover = min_resp.hovered();
                let min_color = if min_hover {
                    palette.text_primary
                } else {
                    palette.text_muted
                };
                let mc = min_rect.center();
                ui.painter().line_segment(
                    [
                        egui::pos2(mc.x - 4.5, mc.y + 3.0),
                        egui::pos2(mc.x + 4.5, mc.y + 3.0),
                    ],
                    Stroke::new(1.5, min_color),
                );

                ui.add_space(2.0);

                // Pin (Always on top) button
                let (pin_rect, pin_resp) =
                    ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::click());
                if pin_resp.clicked() {
                    self.is_pinned = !self.is_pinned;
                    let level = if self.is_pinned {
                        egui::WindowLevel::AlwaysOnTop
                    } else {
                        egui::WindowLevel::Normal
                    };
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::WindowLevel(level));
                }
                let pin_hover = pin_resp.hovered();
                let pin_color = if self.is_pinned {
                    palette.accent_blue
                } else if pin_hover {
                    palette.text_primary
                } else {
                    palette.text_muted
                };
                let pc = pin_rect.center();
                ui.painter()
                    .circle_filled(egui::pos2(pc.x, pc.y - 3.0), 3.0, pin_color);
                ui.painter().line_segment(
                    [egui::pos2(pc.x, pc.y), egui::pos2(pc.x, pc.y + 5.0)],
                    Stroke::new(1.5, pin_color),
                );
            });
        });

        let is_auth_req = self.connection_state == ConnectionState::AuthRequired;
        let indicator_color = if is_auth_req {
            Color32::from_rgb(0xea, 0x86, 0x00)
        } else if self.is_process_detected {
            Color32::from_rgb(0x28, 0xa7, 0x45)
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
            } else if self.is_process_detected {
                self.t("PhotoCraft Socket Offline", "PhotoCraft запущен без сокета")
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
                    "PhotoCraft is running, but the control token was rejected. If multiple versions exist on your PC, an inactive token may have been detected.",
                    "PhotoCraft запущен, но токен не подошел. Возможно, на ПК несколько версий и подтянулся токен от неактивной.",
                )
            } else if self.is_process_detected {
                self.t(
                    "PhotoCraft is running, but the control socket is not reachable. Relaunch via the button below or check Settings.",
                    "PhotoCraft запущен, но сокет управления недоступен. Перезапустите редактор кнопкой ниже или проверьте настройки.",
                )
            } else {
                self.t(
                    "Launch PhotoCraft via the button below to automatically connect the toolbar and plugins.",
                    "Запустите PhotoCraft кнопкой ниже для автоматического подключения панели и плагинов.",
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

            // Primary Button
            let primary_text = if is_auth_req {
                self.t("Enter Token in Settings", "Указать токен в настройках")
            } else {
                self.t("> Launch PhotoCraft", "> Запустить PhotoCraft")
            };
            let primary_btn = egui::Button::new(
                RichText::new(primary_text)
                    .size(11.0)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(palette.accent_blue)
            .corner_radius(4.0)
            .min_size(Vec2::new(190.0, 26.0));

            if ui.add(primary_btn).clicked() {
                if is_auth_req {
                    self.open_manager_requested = true;
                } else {
                    self.launch_requested = true;
                }
            }

            ui.add_space(6.0);

            // Secondary Button
            let secondary_text = if is_auth_req {
                self.t("> Relaunch PhotoCraft", "> Перезапустить PhotoCraft")
            } else {
                self.t("Open Plugins Manager", "Открыть Менеджер плагинов")
            };
            let secondary_btn = egui::Button::new(
                RichText::new(secondary_text)
                    .size(11.0)
                    .color(palette.text_primary),
            )
            .fill(palette.button_secondary)
            .corner_radius(4.0)
            .min_size(Vec2::new(190.0, 24.0));

            if ui.add(secondary_btn).clicked() {
                if is_auth_req {
                    self.launch_requested = true;
                } else {
                    self.open_manager_requested = true;
                }
            }

            ui.add_space(8.0);

            // Contextual Tip
            let tip_text = if is_auth_req {
                self.t(
                    "Tip: Open Settings and manually specify the active PhotoCraft folder or select its control.token using \"Browse...\".",
                    "Подсказка: В настройках укажите папку нужного PhotoCraft или выберите его control.token через «Обзор...».",
                )
            } else if cfg!(target_os = "macos") {
                self.t(
                    "If launching PhotoCraft for the first time, allow it in System Settings -> Privacy & Security, or open once via Right Click -> Open in Finder.",
                    "Если вы запускаете PhotoCraft впервые, разрешите его в «Системных настройках -> Конфиденциальность», либо откройте один раз через Правый клик -> «Открыть» в Finder.",
                )
            } else if cfg!(target_os = "windows") {
                self.t(
                    "If PhotoCraft is blocked on first run, click \"More info\" -> \"Run anyway\" in Windows SmartScreen, or launch PhotoCraft manually.",
                    "Если Windows SmartScreen блокирует первый запуск, нажмите «Подробнее» -> «Выполнить в любом случае», либо запустите PhotoCraft вручную.",
                )
            } else {
                self.t(
                    "Ensure PhotoCraft or AppImage has execute permissions (chmod +x), or launch it from terminal.",
                    "Убедитесь, что у файла PhotoCraft или AppImage есть права на выполнение (chmod +x), либо запустите его из терминала.",
                )
            };
            ui.label(
                RichText::new(tip_text)
                    .size(9.5)
                    .color(palette.text_muted),
            );

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
        assert!(!app.hide_requested);
        assert!(app.is_pinned);
        assert_eq!(app.retry_seconds, 1.5);
        assert!(!app.is_process_detected);
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

    #[test]
    fn test_offline_modal_first_run_tip() {
        let app_en = OfflineModalApp::new(Language::En, ThemeMode::Dark);
        let tip_en = if cfg!(target_os = "macos") {
            app_en.t(
                "If launching PhotoCraft for the first time, allow it in System Settings -> Privacy & Security, or open once via Right Click -> Open in Finder.",
                "Если вы запускаете PhotoCraft впервые, разрешите его в «Системных настройках -> Конфиденциальность», либо откройте один раз через Правый клик -> «Открыть» в Finder.",
            )
        } else if cfg!(target_os = "windows") {
            app_en.t(
                "If PhotoCraft is blocked on first run, click \"More info\" -> \"Run anyway\" in Windows SmartScreen, or launch PhotoCraft manually.",
                "Если Windows SmartScreen блокирует первый запуск, нажмите «Подробнее» -> «Выполнить в любом случае», либо запустите PhotoCraft вручную.",
            )
        } else {
            app_en.t(
                "Ensure PhotoCraft or AppImage has execute permissions (chmod +x), or launch it from terminal.",
                "Убедитесь, что у файла PhotoCraft или AppImage есть права на выполнение (chmod +x), либо запустите его из терминала.",
            )
        };
        assert!(!tip_en.is_empty());
        assert!(tip_en.chars().any(|c| c.is_ascii_alphabetic()));

        let app_ru = OfflineModalApp::new(Language::Ru, ThemeMode::Dark);
        let tip_ru = if cfg!(target_os = "macos") {
            app_ru.t(
                "If launching PhotoCraft for the first time, allow it in System Settings -> Privacy & Security, or open once via Right Click -> Open in Finder.",
                "Если вы запускаете PhotoCraft впервые, разрешите его в «Системных настройках -> Конфиденциальность», либо откройте один раз через Правый клик -> «Открыть» в Finder.",
            )
        } else if cfg!(target_os = "windows") {
            app_ru.t(
                "If PhotoCraft is blocked on first run, click \"More info\" -> \"Run anyway\" in Windows SmartScreen, or launch PhotoCraft manually.",
                "Если Windows SmartScreen блокирует первый запуск, нажмите «Подробнее» -> «Выполнить в любом случае», либо запустите PhotoCraft вручную.",
            )
        } else {
            app_ru.t(
                "Ensure PhotoCraft or AppImage has execute permissions (chmod +x), or launch it from terminal.",
                "Убедитесь, что у файла PhotoCraft или AppImage есть права на выполнение (chmod +x), либо запустите его из терминала.",
            )
        };
        assert!(!tip_ru.is_empty());
        assert!(tip_ru
            .chars()
            .any(|c| ('\u{0400}'..='\u{04FF}').contains(&c)));
    }

    #[test]
    fn test_offline_modal_auth_required_tip() {
        let app_en = OfflineModalApp::new(Language::En, ThemeMode::Dark);
        let tip_en = app_en.t(
            "Tip: Open Settings and manually specify the active PhotoCraft folder or select its control.token using \"Browse...\".",
            "Подсказка: В настройках укажите папку нужного PhotoCraft или выберите его control.token через «Обзор...».",
        );
        assert!(tip_en.contains("multiple") || tip_en.contains("Settings"));

        let app_ru = OfflineModalApp::new(Language::Ru, ThemeMode::Dark);
        let tip_ru = app_ru.t(
            "Tip: Open Settings and manually specify the active PhotoCraft folder or select its control.token using \"Browse...\".",
            "Подсказка: В настройках укажите папку нужного PhotoCraft или выберите его control.token через «Обзор...».",
        );
        assert!(tip_ru.contains("настройках") || tip_ru.contains("PhotoCraft"));
    }
}
