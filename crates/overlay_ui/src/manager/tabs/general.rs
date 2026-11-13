use super::super::*;
use eframe::egui::{self, Color32, RichText, Ui};

impl PluginManagerApp {
    /// Tab 1: General settings (Connection status, Language, Theme, Bridge config)
    pub fn render_general_tab(&mut self, ui: &mut Ui, palette: &ThemePalette) {
        ui.heading(
            RichText::new(self.t("General Preferences", "Основные настройки"))
                .size(14.0)
                .color(palette.text_primary),
        );
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(10.0);

        egui::Grid::new("general_grid")
            .num_columns(2)
            .spacing([16.0, 12.0])
            .show(ui, |ui| {
                // Connection Status
                ui.label(
                    RichText::new(self.t("Connection Status:", "Статус подключения:"))
                        .size(12.0)
                        .color(palette.text_primary),
                );
                let state = *self.connection_state.read().unwrap();
                ui.horizontal(|ui| {
                    Self::render_status_bullet(ui, state);
                    ui.add_space(2.0);
                    match state {
                        ConnectionState::Connected => {
                            ui.label(
                                RichText::new(self.t(
                                    "Connected to PhotoCraft (127.0.0.1:18492)",
                                    "Подключено к PhotoCraft (127.0.0.1:18492)",
                                ))
                                .size(12.0)
                                .strong()
                                .color(Color32::from_rgb(0x28, 0xa7, 0x45)),
                            );
                        }
                        ConnectionState::AuthRequired => {
                            ui.label(
                                RichText::new(self.t(
                                    "Auth Required (Token Missing or Invalid)",
                                    "Требуется авторизация (Токен отсутствует или неверен)",
                                ))
                                .size(12.0)
                                .strong()
                                .color(Color32::from_rgb(0xea, 0x86, 0x00)),
                            );
                        }
                        ConnectionState::Offline => {
                            ui.label(
                                RichText::new(self.t(
                                    "Offline (Waiting for PhotoCraft on port 18492...)",
                                    "Офлайн (Ожидание запуска PhotoCraft на порту 18492...)",
                                ))
                                .size(12.0)
                                .color(palette.text_muted),
                            );
                        }
                    }
                });
                ui.end_row();

                // Language
                ui.label(
                    RichText::new(self.t("Language:", "Язык интерфейса:"))
                        .size(12.0)
                        .color(palette.text_primary),
                );
                let current_label = match self.current_language {
                    Language::Auto => {
                        let eff = self.resolved_auto_language.safe_ui_language();
                        if eff == self.resolved_auto_language {
                            format!("Auto ({})", eff.display_name())
                        } else {
                            format!("Auto (PhotoCraft) ({})", Language::En.display_name())
                        }
                    }
                    lang => lang.safe_ui_language().selector_label(),
                };
                egui::ComboBox::from_id_salt("general_lang_select")
                    .selected_text(RichText::new(current_label).color(palette.text_primary))
                    .width(200.0)
                    .show_ui(ui, |ui| {
                        ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
                        let eff = self.resolved_auto_language.safe_ui_language();
                        let auto_label = if eff == self.resolved_auto_language {
                            format!("Auto (PhotoCraft) ({})", eff.display_name())
                        } else {
                            format!("Auto (PhotoCraft) ({})", Language::En.display_name())
                        };
                        ui.selectable_value(
                            &mut self.current_language,
                            Language::Auto,
                            auto_label,
                        );
                        ui.separator();
                        for lang in Language::UI_SUPPORTED {
                            ui.selectable_value(
                                &mut self.current_language,
                                lang,
                                lang.selector_label(),
                            );
                        }
                    });
                ui.end_row();

                // Theme (Strictly 3 options)
                ui.label(
                    RichText::new(self.t("Theme:", "Тема оформления:"))
                        .size(12.0)
                        .color(palette.text_primary),
                );
                let theme_label = match self.current_theme {
                    ThemeMode::Dark => {
                        self.t("Dark (PhotoCraft Graphite)", "Тёмная (Графит PhotoCraft)")
                    }
                    ThemeMode::Light => self.t("Light", "Светлая"),
                    ThemeMode::System => {
                        self.t("System (Windows Sync)", "Системная (Синхронизация Windows)")
                    }
                };
                let label_dark = self.t("Dark", "Тёмная");
                let label_light = self.t("Light", "Светлая");
                let label_system = self.t("System", "Системная");
                egui::ComboBox::from_id_salt("general_theme_select")
                    .selected_text(RichText::new(theme_label).color(palette.text_primary))
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
                        ui.selectable_value(&mut self.current_theme, ThemeMode::Dark, label_dark);
                        ui.selectable_value(&mut self.current_theme, ThemeMode::Light, label_light);
                        ui.selectable_value(
                            &mut self.current_theme,
                            ThemeMode::System,
                            label_system,
                        );
                    });
                ui.end_row();

                // PhotoCraft Path
                ui.label(
                    RichText::new(self.t("PhotoCraft Path:", "Путь к PhotoCraft:"))
                        .size(12.0)
                        .color(palette.text_primary),
                );
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.photocraft_path);
                    if ui
                        .button(
                            RichText::new(self.t("Browse...", "Обзор..."))
                                .size(11.0)
                                .color(Color32::WHITE),
                        )
                        .clicked()
                    {
                        let mut dialog =
                            rfd::FileDialog::new().set_title("Select PhotoCraft Executable");
                        #[cfg(target_os = "windows")]
                        {
                            dialog = dialog.add_filter("Executable", &["exe"]);
                        }
                        #[cfg(target_os = "macos")]
                        {
                            dialog = dialog.add_filter("PhotoCraft App / Executable", &["app", ""]);
                        }
                        #[cfg(target_os = "linux")]
                        {
                            dialog = dialog
                                .add_filter(
                                    "PhotoCraft Executable / AppImage / Desktop Entry",
                                    &["AppImage", "appimage", "desktop", "bin", ""],
                                )
                                .add_filter("All Files (*)", &["*"]);
                        }
                        if let Some(path) = dialog.pick_file() {
                            self.photocraft_path = path.to_string_lossy().to_string();
                        }
                    }
                    if ui
                        .button(
                            RichText::new(self.t("> Launch", "> Запуск"))
                                .size(11.0)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .clicked()
                    {
                        let path_trim = self.photocraft_path.trim();
                        let is_invalid_folder = if !path_trim.is_empty() {
                            let p = std::path::Path::new(path_trim);
                            if p.is_dir() {
                                let has_binary = p.join("photocraft.exe").is_file()
                                    || p.join("PhotoCraft.exe").is_file()
                                    || p.join("photocraft").is_file()
                                    || p.join("PhotoCraft").is_file();
                                !has_binary
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if is_invalid_folder {
                            let msg = self
                                .t(
                                    "Please specify the photocraft.exe binary, not a folder.",
                                    "Пожалуйста, укажите файл photocraft.exe, а не папку.",
                                )
                                .to_string();
                            self.set_status(msg, true);
                        } else {
                            if let Err(e) = self.save_settings() {
                                tracing::error!("Failed to auto-save settings before launch: {e}");
                            }
                            self.launch_requested = true;
                        }
                    }
                });
                ui.end_row();

                // Host
                ui.label(
                    RichText::new(self.t("Bridge Host:", "Хост подключения:"))
                        .size(12.0)
                        .color(palette.text_primary),
                );
                ui.text_edit_singleline(&mut self.host);
                ui.end_row();

                // Port
                ui.label(
                    RichText::new(self.t("Bridge Port:", "Порт подключения:"))
                        .size(12.0)
                        .color(palette.text_primary),
                );
                let mut p = self.port.to_string();
                if ui.text_edit_singleline(&mut p).changed() {
                    if let Ok(num) = p.parse() {
                        self.port = num;
                    }
                }
                ui.end_row();

                // Control Token
                ui.label(
                    RichText::new(self.t("Control Token:", "Токен управления:"))
                        .size(12.0)
                        .color(palette.text_primary),
                );
                ui.horizontal(|ui| {
                    let hint = self.t("(Auto-detected)", "(Автоопределение)");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.token)
                            .hint_text(RichText::new(hint).color(palette.text_muted)),
                    );
                    if ui
                        .button(
                            RichText::new(self.t("Auto-Detect", "Найти токен"))
                                .size(11.0)
                                .color(Color32::WHITE),
                        )
                        .clicked()
                    {
                        let found = detect_default_token(&self.photocraft_path, &self.prefs_path);

                        if let Some(t) = found {
                            self.token = t;
                            let _ = self.save_settings();
                            let is_offline = self
                                .connection_state
                                .read()
                                .map(|s| *s == ConnectionState::Offline)
                                .unwrap_or(false);
                            if is_offline {
                                let is_proc = self
                                    .is_process_running
                                    .as_ref()
                                    .map(|f| f())
                                    .unwrap_or(false);
                                if is_proc {
                                    let msg = self
                                        .t(
                                            "Token saved. PhotoCraft is running, but port 18492 is closed (Set Control port to 18492 in Edit -> Preferences -> Integrations)",
                                            "Токен сохранен. PhotoCraft запущен, но порт 18492 закрыт (Укажите порт 18492 в Редактирование -> Настройки -> Интеграции)",
                                        )
                                        .to_string();
                                    self.set_status(msg, false);
                                } else {
                                    let msg = self
                                        .t(
                                            "Token saved. Start PhotoCraft using [> Launch]",
                                            "Токен сохранен. Запустите PhotoCraft кнопкой [> Запуск]",
                                        )
                                        .to_string();
                                    self.set_status(msg, false);
                                }
                            } else {
                                let msg = self
                                    .t("Token updated successfully", "Токен успешно обновлен")
                                    .to_string();
                                self.set_status(msg, false);
                            }
                        } else {
                            let msg = self
                                .t("control.token not found", "Файл control.token не найден")
                                .to_string();
                            self.set_status(msg, true);
                        }
                    }

                    if ui
                        .button(
                            RichText::new(self.t("Browse...", "Обзор..."))
                                .size(11.0)
                                .color(Color32::WHITE),
                        )
                        .clicked()
                    {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("Select PhotoCraft control.token")
                            .pick_file()
                        {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                let trimmed = content.trim();
                                if !trimmed.is_empty() {
                                    self.token = trimmed.to_string();
                                    let _ = self.save_settings();
                                    let is_offline = self
                                        .connection_state
                                        .read()
                                        .map(|s| *s == ConnectionState::Offline)
                                        .unwrap_or(false);
                                    if is_offline {
                                        let is_proc = self
                                            .is_process_running
                                            .as_ref()
                                            .map(|f| f())
                                            .unwrap_or(false);
                                        if is_proc {
                                            let msg = self
                                                .t(
                                                    "Token saved. PhotoCraft is running, but port 18492 is closed (Set Control port to 18492 in Edit -> Preferences -> Integrations)",
                                                    "Токен сохранен. PhotoCraft запущен, но порт 18492 закрыт (Укажите порт 18492 в Редактирование -> Настройки -> Интеграции)",
                                                )
                                                .to_string();
                                            self.set_status(msg, false);
                                        } else {
                                            let msg = self
                                                .t(
                                                    "Token saved. Start PhotoCraft using [> Launch]",
                                                    "Токен сохранен. Запустите PhotoCraft кнопкой [> Запуск]",
                                                )
                                                .to_string();
                                            self.set_status(msg, false);
                                        }
                                    } else {
                                        let msg = self
                                            .t("Token loaded from file", "Токен загружен из файла")
                                            .to_string();
                                        self.set_status(msg, false);
                                    }
                                }
                            }
                        }
                    }
                });
                ui.end_row();

                if state == ConnectionState::AuthRequired {
                    ui.label("");
                    ui.label(
                        RichText::new(self.t(
                            "Tip: If multiple versions exist on your PC, point to the active photocraft.exe or select its control.token file.",
                            "Подсказка: Если на ПК несколько версий, укажите активный photocraft.exe или выберите его control.token.",
                        ))
                        .size(10.5)
                        .color(Color32::from_rgb(0xea, 0x86, 0x00)),
                    );
                    ui.end_row();
                }
            });
    }
}
