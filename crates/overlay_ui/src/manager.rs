use crate::theme::{ThemeMode, ThemePalette};
use core_contracts::{CompanionConfig, Language, PluginManifest, PluginType};
use eframe::egui::{self, Color32, RichText, Stroke, Ui, Vec2};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    General,
    Plugins,
    Hotkeys,
    About,
}

pub struct PluginManagerApp {
    pub active_tab: ActiveTab,
    pub manifests: Vec<PluginManifest>,
    pub current_language: Language,
    pub current_theme: ThemeMode,
    pub host: String,
    pub port: u16,
    pub token: String,
    pub status_text: String,
    pub prefs_path: PathBuf,
    pub plugins_dir: PathBuf,
    pub config_path: PathBuf,
    pub hotkey_overrides: HashMap<String, String>,
    pub downloading_model: Option<String>,
    pub is_connected: Arc<AtomicBool>,
    pub recording_hotkey_for: Option<String>,
}

impl PluginManagerApp {
    pub fn new(
        manifests: Vec<PluginManifest>,
        prefs_path: impl Into<PathBuf>,
        is_connected: Arc<AtomicBool>,
    ) -> Self {
        let prefs_path = prefs_path.into();
        let data_dir = prefs_path.parent().unwrap_or(&prefs_path).to_path_buf();
        let config_path = CompanionConfig::resolve_config_path(&data_dir);
        let cfg = CompanionConfig::load_or_default(&config_path);

        let lang = if cfg.language == "ru" {
            Language::Ru
        } else if cfg.language == "en" {
            Language::En
        } else {
            Language::Auto.resolve(Some(&prefs_path))
        };

        let theme = match cfg.theme.as_str() {
            "light" => ThemeMode::Light,
            "system" => ThemeMode::System,
            _ => ThemeMode::from_preferences(&prefs_path),
        };

        // Auto-read token from PhotoCraftData if present and not in config
        let token = if let Some(ref t) = cfg.token {
            t.clone()
        } else {
            let token_path = data_dir.join("control.token");
            std::fs::read_to_string(&token_path)
                .map(|s| s.trim().to_string())
                .unwrap_or_default()
        };

        let plugins_dir = if PathBuf::from("plugins").is_dir() {
            PathBuf::from("plugins")
        } else {
            data_dir.parent().unwrap_or(&data_dir).join("plugins")
        };

        Self {
            active_tab: ActiveTab::General,
            manifests,
            current_language: lang,
            current_theme: theme,
            host: cfg.host,
            port: cfg.port,
            token,
            status_text: String::new(),
            prefs_path,
            plugins_dir,
            config_path,
            hotkey_overrides: cfg.hotkey_overrides,
            downloading_model: None,
            is_connected,
            recording_hotkey_for: None,
        }
    }

    pub fn t<'a>(&self, en: &'a str, ru: &'a str) -> &'a str {
        match self.current_language {
            Language::Ru => ru,
            _ => en,
        }
    }

    pub fn save_settings(&mut self) -> anyhow::Result<()> {
        let mut cfg = CompanionConfig::load_or_default(&self.config_path);
        cfg.host = self.host.clone();
        cfg.port = self.port;
        cfg.token = if self.token.trim().is_empty() {
            None
        } else {
            Some(self.token.trim().to_string())
        };
        cfg.theme = match self.current_theme {
            ThemeMode::Dark => "dark".to_string(),
            ThemeMode::Light => "light".to_string(),
            ThemeMode::System => "system".to_string(),
        };
        cfg.language = match self.current_language {
            Language::Ru => "ru".to_string(),
            _ => "en".to_string(),
        };
        cfg.enabled_plugins = self
            .manifests
            .iter()
            .filter(|m| m.enabled)
            .map(|m| m.id.clone())
            .collect();
        cfg.hotkey_overrides = self.hotkey_overrides.clone();

        cfg.save(&self.config_path)?;
        info!("Saved settings to {}", self.config_path.display());

        self.status_text = self
            .t("Settings saved successfully!", "Настройки успешно сохранены!")
            .to_string();
        Ok(())
    }

    /// Renders top horizontal tabs
    pub fn render_tabs_header(&mut self, ui: &mut Ui, palette: &ThemePalette) {
        let tabs = [
            (ActiveTab::General, self.t("General", "Общие")),
            (ActiveTab::Plugins, self.t("Plugins", "Плагины")),
            (ActiveTab::Hotkeys, self.t("Hotkeys", "Горячие клавиши")),
            (ActiveTab::About, self.t("About", "О программе")),
        ];

        ui.horizontal(|ui| {
            ui.add_space(4.0);
            for (tab, title) in tabs {
                let is_active = self.active_tab == tab;
                let bg_color = if is_active {
                    palette.accent_blue
                } else {
                    palette.button_secondary
                };
                let text_color = if is_active {
                    Color32::WHITE
                } else {
                    palette.text_primary
                };

                let btn = egui::Button::new(RichText::new(title).size(12.0).strong().color(text_color))
                    .fill(bg_color)
                    .corner_radius(6.0)
                    .min_size(Vec2::new(96.0, 26.0));

                if ui.add(btn).clicked() {
                    self.active_tab = tab;
                }
                ui.add_space(4.0);
            }
        });
    }

    pub fn render_status_bullet(ui: &mut Ui, connected: bool) {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(10.0), egui::Sense::hover());
        let color = if connected {
            Color32::from_rgb(0x28, 0xa7, 0x45)
        } else {
            Color32::from_rgb(0xea, 0x86, 0x00)
        };
        ui.painter().circle_filled(rect.center(), 3.5, color);
    }

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
                let connected = self.is_connected.load(Ordering::Relaxed);
                ui.horizontal(|ui| {
                    Self::render_status_bullet(ui, connected);
                    ui.add_space(2.0);
                    if connected {
                        ui.label(
                            RichText::new(self.t("Connected to PhotoCraft (127.0.0.1:18492)", "Подключено к PhotoCraft (127.0.0.1:18492)"))
                                .size(12.0)
                                .strong()
                                .color(Color32::from_rgb(0x28, 0xa7, 0x45)),
                        );
                    } else {
                        ui.label(
                            RichText::new(self.t("Offline (Waiting for PhotoCraft on port 18492...)", "Офлайн (Ожидание запуска PhotoCraft на порту 18492...)"))
                                .size(12.0)
                                .color(Color32::from_rgb(0xea, 0x86, 0x00)),
                        );
                    }
                });
                ui.end_row();

                // Language
                ui.label(
                    RichText::new(self.t("Language:", "Язык интерфейса:"))
                        .size(12.0)
                        .color(palette.text_primary),
                );
                let lang_label = match self.current_language {
                    Language::Ru => "Русский (RU)",
                    _ => "English (EN)",
                };
                egui::ComboBox::from_id_salt("general_lang_select")
                    .selected_text(RichText::new(lang_label).color(palette.text_primary))
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
                        ui.selectable_value(&mut self.current_language, Language::En, "English (EN)");
                        ui.selectable_value(&mut self.current_language, Language::Ru, "Русский (RU)");
                    });
                ui.end_row();

                // Theme (Strictly 3 options)
                ui.label(
                    RichText::new(self.t("Theme:", "Тема оформления:"))
                        .size(12.0)
                        .color(palette.text_primary),
                );
                let theme_label = match self.current_theme {
                    ThemeMode::Dark => self.t("Dark (PhotoCraft Graphite)", "Тёмная (Графит PhotoCraft)"),
                    ThemeMode::Light => self.t("Light", "Светлая"),
                    ThemeMode::System => self.t("System (Windows Sync)", "Системная (Синхронизация Windows)"),
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
                        ui.selectable_value(&mut self.current_theme, ThemeMode::System, label_system);
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
                    ui.text_edit_singleline(&mut self.token);
                    if ui
                        .button(
                            RichText::new(self.t("Auto-Detect", "Найти токен"))
                                .size(11.0)
                                .color(Color32::WHITE),
                        )
                        .clicked()
                    {
                        let token_path = self
                            .prefs_path
                            .parent()
                            .unwrap_or(&self.prefs_path)
                            .join("control.token");
                        if let Ok(s) = std::fs::read_to_string(&token_path) {
                            self.token = s.trim().to_string();
                            self.status_text = self
                                .t("Token updated successfully", "Токен успешно обновлен")
                                .to_string();
                        } else {
                            self.status_text = self
                                .t("control.token not found", "Файл control.token не найден")
                                .to_string();
                        }
                    }
                });
                ui.end_row();
            });
    }

    /// Tab 2: Plugins management with .pcpl install and model status
    pub fn render_plugins_tab(&mut self, ui: &mut Ui, palette: &ThemePalette) {
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new(self.t("Installed Plugins", "Установленные плагины"))
                    .size(14.0)
                    .color(palette.text_primary),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Install .pcpl button
                let install_btn = egui::Button::new(
                    RichText::new(self.t("+ Install Plugin (.pcpl)", "+ Установить плагин (.pcpl)"))
                        .size(11.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(palette.accent_blue)
                .corner_radius(4.0)
                .min_size(Vec2::new(140.0, 24.0));

                if ui.add(install_btn).clicked() {
                    if let Some(file) = rfd::FileDialog::new()
                        .add_filter("PhotoCraft Plugin (*.pcpl)", &["pcpl"])
                        .pick_file()
                    {
                        match core_contracts::install_pcpl(&file, &self.plugins_dir) {
                            Ok(m) => {
                                self.status_text = format!(
                                    "● {}",
                                    self.t("Plugin installed successfully!", "Плагин успешно установлен!")
                                );
                                self.manifests.push(m);
                            }
                            Err(e) => {
                                self.status_text = format!("Error: {e}");
                            }
                        }
                    }
                }
            });
        });

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);

        let msg_model_ready = self.t("● Model ready", "● Модель готова");
        let msg_model_missing = self.t("⚠️ Model missing", "⚠️ Требуется модель");
        let msg_download = self.t("⬇ Download model", "⬇ Скачать модель");
        let msg_download_started = self.t("Model download started...", "Загрузка модели начата...");
        let mut download_notice = None;

        for manifest in &mut self.manifests {
            let all_models_ok = manifest.all_models_installed();

            egui::Frame::new()
                .fill(palette.card_inner)
                .stroke(Stroke::new(1.0, palette.border_stroke))
                .corner_radius(6.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let type_str = match manifest.plugin_type {
                            PluginType::NoCode => "NO-CODE",
                            PluginType::NativeAi => "AI GPU",
                            PluginType::Script => "SCRIPT",
                            PluginType::ExternalCli => "CLI",
                        };

                        let type_color = match manifest.plugin_type {
                            PluginType::NativeAi => palette.accent_blue,
                            PluginType::NoCode => Color32::from_rgb(0x28, 0xa7, 0x45),
                            _ => palette.text_muted,
                        };

                        ui.label(
                            RichText::new(type_str)
                                .size(9.0)
                                .color(type_color)
                                .strong(),
                        );
                        ui.label(
                            RichText::new(&manifest.name)
                                .size(12.0)
                                .strong()
                                .color(palette.text_primary),
                        );
                        ui.label(
                            RichText::new(format!("v{}", manifest.version))
                                .size(10.0)
                                .color(palette.text_muted),
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.checkbox(&mut manifest.enabled, "");
                        });
                    });

                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(&manifest.description)
                            .size(11.0)
                            .color(palette.text_muted),
                    );

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if let Some(ref hk) = manifest.default_hotkey {
                            ui.label(
                                RichText::new(format!("[{}]", hk))
                                    .size(10.0)
                                    .color(palette.accent_blue),
                            );
                        }

                        ui.label(
                            RichText::new(format!("• {}", manifest.author))
                                .size(10.0)
                                .color(palette.text_muted),
                        );

                        // Models status badge
                        if !manifest.models.is_empty() {
                            if all_models_ok {
                                ui.label(
                                    RichText::new(msg_model_ready)
                                        .size(10.0)
                                        .color(Color32::from_rgb(0x28, 0xa7, 0x45)),
                                );
                            } else {
                                ui.label(
                                    RichText::new(msg_model_missing)
                                        .size(10.0)
                                        .color(Color32::from_rgb(0xea, 0x86, 0x00)),
                                );

                                // Button to trigger download if model has URL
                                if let Some(req) = manifest.models.first() {
                                    if let Some(ref url) = req.url {
                                        let dl_btn = egui::Button::new(
                                            RichText::new(msg_download).size(10.0).color(Color32::WHITE),
                                        )
                                        .fill(palette.button_secondary)
                                        .corner_radius(2.0);

                                        if ui.add(dl_btn).clicked() {
                                            let target_dir = manifest
                                                .plugin_dir
                                                .clone()
                                                .unwrap_or_else(|| PathBuf::from("plugins").join(&manifest.id))
                                                .join("models");
                                            let _ = std::fs::create_dir_all(&target_dir);
                                            let target_file = target_dir.join(&req.name);
                                            let url_clone = url.clone();

                                            std::thread::spawn(move || {
                                                let _ = std::process::Command::new("curl.exe")
                                                    .args(["-L", "-o", target_file.to_str().unwrap_or(""), &url_clone])
                                                    .output();
                                            });
                                            download_notice = Some(msg_download_started);
                                        }
                                    }
                                }
                            }
                        }
                    });
                });
            ui.add_space(4.0);
        }

        if let Some(msg) = download_notice {
            self.status_text = msg.to_string();
        }
    }

    /// Tab 3: Hotkeys configuration with interactive key capture
    pub fn render_hotkeys_tab(&mut self, ui: &mut Ui, palette: &ThemePalette) {
        ui.heading(
            RichText::new(self.t("Keyboard Shortcuts", "Горячие клавиши"))
                .size(14.0)
                .color(palette.text_primary),
        );
        ui.add_space(4.0);
        ui.label(
            RichText::new(self.t(
                "Click a shortcut button to record a key combination. Press Esc to cancel.",
                "Нажмите на кнопку с хоткеем, чтобы записать сочетание клавиш. Нажмите Esc для отмены.",
            ))
            .size(11.0)
            .color(palette.text_muted),
        );
        ui.add_space(6.0);
        ui.separator();
        ui.add_space(8.0);

        // Check for key recording events
        let mut recorded_hotkey = None;
        let mut stop_recording = false;
        let mut clear_recording = false;

        if let Some(ref target_id) = self.recording_hotkey_for {
            ui.input(|i| {
                if i.key_pressed(egui::Key::Escape) {
                    stop_recording = true;
                } else if i.key_pressed(egui::Key::Backspace) || i.key_pressed(egui::Key::Delete) {
                    clear_recording = true;
                } else {
                    for event in &i.events {
                        if let egui::Event::Key {
                            key,
                            pressed: true,
                            modifiers,
                            ..
                        } = event
                        {
                            let key_str = format!("{:?}", key);
                            // Avoid registering bare modifiers
                            let is_modifier = key_str.starts_with("Control")
                                || key_str.starts_with("Shift")
                                || key_str.starts_with("Alt")
                                || key_str.starts_with("Super")
                                || key_str.starts_with("Meta");

                            if !is_modifier {
                                let mut parts = Vec::new();
                                if modifiers.ctrl || modifiers.command {
                                    parts.push("Ctrl");
                                }
                                if modifiers.alt {
                                    parts.push("Alt");
                                }
                                if modifiers.shift {
                                    parts.push("Shift");
                                }
                                let clean_key = match key_str.as_str() {
                                    "Num0" => "0",
                                    "Num1" => "1",
                                    "Num2" => "2",
                                    "Num3" => "3",
                                    "Num4" => "4",
                                    "Num5" => "5",
                                    "Num6" => "6",
                                    "Num7" => "7",
                                    "Num8" => "8",
                                    "Num9" => "9",
                                    other => other,
                                };
                                parts.push(clean_key);
                                recorded_hotkey = Some((target_id.clone(), parts.join("+")));
                                break;
                            }
                        }
                    }
                }
            });
        }

        if stop_recording {
            self.recording_hotkey_for = None;
        } else if clear_recording {
            if let Some(target_id) = self.recording_hotkey_for.take() {
                self.hotkey_overrides.remove(&target_id);
                for m in &mut self.manifests {
                    if m.id == target_id {
                        m.default_hotkey = None;
                    }
                }
            }
        } else if let Some((target_id, combo)) = recorded_hotkey {
            self.hotkey_overrides.insert(target_id.clone(), combo.clone());
            for m in &mut self.manifests {
                if m.id == target_id {
                    m.default_hotkey = Some(combo.clone());
                }
            }
            self.recording_hotkey_for = None;
        }

        let clear_tooltip = self.t("Clear hotkey", "Очистить хоткей").to_string();
        let none_text = self.t("None", "Нет").to_string();
        let press_keys_text = self.t("Press keys...", "Нажмите клавиши...").to_string();

        for m in &mut self.manifests {
            let plugin_id = m.id.clone();
            let is_recording = self.recording_hotkey_for.as_deref() == Some(&plugin_id);

            egui::Frame::new()
                .fill(palette.card_inner)
                .stroke(Stroke::new(
                    1.0,
                    if is_recording {
                        palette.accent_blue
                    } else {
                        palette.border_stroke
                    },
                ))
                .corner_radius(6.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(&m.name)
                                    .size(12.0)
                                    .strong()
                                    .color(palette.text_primary),
                            );
                            ui.label(
                                RichText::new(&m.description)
                                    .size(10.0)
                                    .color(palette.text_muted),
                            );
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Clear hotkey button with standard cross symbol
                            let clear_btn = egui::Button::new(
                                RichText::new("×")
                                    .size(14.0)
                                    .strong()
                                    .color(palette.text_muted),
                            )
                            .min_size(Vec2::new(20.0, 24.0))
                            .corner_radius(4.0)
                            .fill(Color32::TRANSPARENT);

                            if ui
                                .add(clear_btn)
                                .on_hover_text(&clear_tooltip)
                                .clicked()
                            {
                                self.hotkey_overrides.remove(&plugin_id);
                                m.default_hotkey = None;
                            }

                            // Interactive hotkey capture button
                            let current_val = self
                                .hotkey_overrides
                                .get(&plugin_id)
                                .cloned()
                                .or_else(|| m.default_hotkey.clone())
                                .unwrap_or_else(|| none_text.clone());

                            let button_text = if is_recording {
                                let mut active_mods = Vec::new();
                                ui.input(|i| {
                                    if i.modifiers.ctrl || i.modifiers.command {
                                        active_mods.push("Ctrl");
                                    }
                                    if i.modifiers.alt {
                                        active_mods.push("Alt");
                                    }
                                    if i.modifiers.shift {
                                        active_mods.push("Shift");
                                    }
                                });
                                if active_mods.is_empty() {
                                    press_keys_text.clone()
                                } else {
                                    format!("{}+...", active_mods.join("+"))
                                }
                            } else {
                                current_val
                            };

                            let btn_color = if is_recording {
                                palette.accent_blue
                            } else {
                                palette.button_secondary
                            };
                            let text_color = if is_recording {
                                Color32::WHITE
                            } else {
                                palette.text_primary
                            };

                            let rec_btn = egui::Button::new(
                                RichText::new(button_text)
                                    .size(11.0)
                                    .strong()
                                    .color(text_color),
                            )
                            .fill(btn_color)
                            .corner_radius(4.0)
                            .min_size(Vec2::new(110.0, 24.0));

                            if ui.add(rec_btn).clicked() {
                                if is_recording {
                                    self.recording_hotkey_for = None;
                                } else {
                                    self.recording_hotkey_for = Some(plugin_id.clone());
                                }
                            }
                        });
                    });
                });
            ui.add_space(4.0);
        }
    }

    /// Tab 4: About
    pub fn render_about_tab(&mut self, ui: &mut Ui, palette: &ThemePalette) {
        ui.heading(
            RichText::new("PhotoCraft Companion")
                .size(16.0)
                .color(palette.text_primary),
        );
        ui.label(
            RichText::new("Version 1.0.0 (Rust Cargo Workspace & DirectML GPU)")
                .size(11.0)
                .color(palette.text_muted),
        );
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(10.0);

        ui.label(
            RichText::new(self.t(
                "Extensible companion suite for the PhotoCraft graphics editor with local neural background removal, No-Code macro pipelines, .pcpl package manager, and seamless floating toolstrip.",
                "Расширяемый компаньон для графического редактора PhotoCraft с локальным нейросетевым удалением фона, No-Code макросами, менеджером пакетов .pcpl и плавающей панелью инструментов.",
            ))
            .size(12.0)
            .color(palette.text_primary),
        );

        ui.add_space(12.0);
        ui.label(
            RichText::new(self.t(
                "• Engine: Pure Rust + tokio JSON-RPC Bridge (Port 18492)\n• AI Hardware: DirectML (NVIDIA RTX / AMD / Intel) with CPU fallback\n• GUI: Immediate-mode egui/eframe with PhotoCraft graphite palette\n• License: MIT OR Apache-2.0",
                "• Движок: Чистый Rust + tokio JSON-RPC сокет (порт 18492)\n• Аппаратный ИИ: DirectML (NVIDIA RTX / AMD / Intel) с откатом на CPU\n• Интерфейс: Immediate-mode egui/eframe в фирменной графитовой палитре\n• Лицензия: MIT OR Apache-2.0",
            ))
            .size(11.0)
            .color(palette.text_muted),
        );
    }

    /// Master render function with horizontal tabs and pinned bottom buttons
    pub fn render_settings_window(
        &mut self,
        ui: &mut Ui,
        palette: &ThemePalette,
        is_open: &mut bool,
    ) {
        let card_fill = palette.card_fill;

        // 1. Pinned bottom panel: status/connection indicator on left, [ Close ] [ Save ] on right
        egui::Panel::bottom("manager_bottom_panel")
            .frame(egui::Frame::new().fill(palette.bg_fill).inner_margin(egui::Margin::symmetric(14, 10)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if !self.status_text.is_empty() {
                        ui.label(
                            RichText::new(&self.status_text)
                                .size(11.0)
                                .color(palette.accent_blue),
                        );
                    } else {
                        let connected = self.is_connected.load(Ordering::Relaxed);
                        Self::render_status_bullet(ui, connected);
                        ui.add_space(2.0);
                        let text = if connected {
                            self.t(
                                "Connected (127.0.0.1:18492)",
                                "Подключено (127.0.0.1:18492)",
                            )
                        } else {
                            self.t(
                                "Offline (Socket 18492)",
                                "Офлайн (Сокет 18492)",
                            )
                        };
                        let color = if connected {
                            Color32::from_rgb(0x28, 0xa7, 0x45)
                        } else {
                            palette.text_muted
                        };
                        ui.label(RichText::new(text).size(11.0).color(color));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let save_btn = egui::Button::new(
                            RichText::new(self.t("Save", "Сохранить"))
                                .size(11.0)
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(palette.accent_blue)
                        .corner_radius(4.0)
                        .min_size(Vec2::new(76.0, 26.0));

                        if ui.add(save_btn).clicked() {
                            let _ = self.save_settings();
                        }

                        ui.add_space(8.0);

                        let close_btn = egui::Button::new(
                            RichText::new(self.t("Close", "Закрыть"))
                                .size(11.0)
                                .color(palette.text_primary),
                        )
                        .fill(palette.button_secondary)
                        .corner_radius(4.0)
                        .min_size(Vec2::new(70.0, 26.0));

                        if ui.add(close_btn).clicked() {
                            *is_open = false;
                        }
                    });
                });
            });

        // 2. Central panel: Tabs header and scrollable card fills remaining space
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(palette.bg_fill).inner_margin(egui::Margin::symmetric(14, 8)))
            .show(ui, |ui| {
                self.render_tabs_header(ui, palette);
                ui.add_space(6.0);

                egui::Frame::new()
                    .fill(card_fill)
                    .stroke(Stroke::new(1.0, palette.border_stroke))
                    .corner_radius(8.0)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| match self.active_tab {
                                ActiveTab::General => self.render_general_tab(ui, palette),
                                ActiveTab::Plugins => self.render_plugins_tab(ui, palette),
                                ActiveTab::Hotkeys => self.render_hotkeys_tab(ui, palette),
                                ActiveTab::About => self.render_about_tab(ui, palette),
                            });
                    });
            });
    }
}

impl eframe::App for PluginManagerApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let palette = ThemePalette::for_mode(self.current_theme);
        ui.ctx().set_visuals(palette.to_egui_visuals());

        let mut is_open = true;
        self.render_settings_window(ui, &palette, &mut is_open);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_app_init() {
        let temp_dir = std::env::temp_dir().join(format!("mgr_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let prefs_file = temp_dir.join("preferences.json");
        std::fs::write(&prefs_file, r#"{"interface": {"language": "en", "theme": "studioDark"}}"#).unwrap();

        let is_connected = Arc::new(AtomicBool::new(true));
        let app = PluginManagerApp::new(vec![], prefs_file, is_connected.clone());

        assert_eq!(app.active_tab, ActiveTab::General);
        assert_eq!(app.current_language, Language::En);
        assert_eq!(app.current_theme, ThemeMode::Dark);
        assert_eq!(app.host, "127.0.0.1");
        assert_eq!(app.port, 18492);
        assert!(app.is_connected.load(Ordering::SeqCst));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_plugin_manager_app_translations() {
        let temp_dir = std::env::temp_dir().join(format!("mgr_trans_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let prefs_file = temp_dir.join("preferences.json");
        let is_connected = Arc::new(AtomicBool::new(false));

        let mut app = PluginManagerApp::new(vec![], prefs_file, is_connected);
        app.current_language = Language::En;
        assert_eq!(app.t("Save", "Сохранить"), "Save");

        app.current_language = Language::Ru;
        assert_eq!(app.t("Save", "Сохранить"), "Сохранить");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_plugin_manager_app_save_settings() {
        let temp_dir = std::env::temp_dir().join(format!("mgr_save_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let prefs_file = temp_dir.join("preferences.json");
        let is_connected = Arc::new(AtomicBool::new(false));

        let mut app = PluginManagerApp::new(vec![], &prefs_file, is_connected);
        app.config_path = temp_dir.join("companion_config.json");
        app.port = 19999;
        app.current_theme = ThemeMode::Light;
        app.hotkey_overrides.insert("test.plugin".to_string(), "Ctrl+Alt+P".to_string());

        let res = app.save_settings();
        assert!(res.is_ok());
        assert!(app.config_path.exists());

        let loaded = CompanionConfig::load_or_default(&app.config_path);
        assert_eq!(loaded.port, 19999);
        assert_eq!(loaded.theme, "light");
        assert_eq!(loaded.hotkey_overrides.get("test.plugin"), Some(&"Ctrl+Alt+P".to_string()));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}


