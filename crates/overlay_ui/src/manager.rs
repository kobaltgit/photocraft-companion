use crate::theme::{ThemeMode, ThemePalette};
use core_contracts::{CompanionConfig, ConnectionState, Language, PluginManifest, PluginType};
use eframe::egui::{self, Color32, RichText, Stroke, Ui, Vec2};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    General,
    Plugins,
    Store,
    About,
}

#[derive(Debug, Clone)]
pub struct ModelDownloadProgress {
    pub model_name: String,
    pub plugin_id: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bytes_sec: f32,
}

pub enum ModelDownloadEvent {
    Progress(ModelDownloadProgress),
    Finished { model_name: String },
    Error { model_name: String, error: String },
}

#[derive(Debug, Clone)]
pub struct PluginReadmeState {
    pub plugin_name: String,
    pub plugin_dir: PathBuf,
    pub raw_content: String,
    pub active_lang: Language,
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
    pub status_timestamp: Option<std::time::Instant>,
    pub status_is_error: bool,
    pub prefs_path: PathBuf,
    pub plugins_dir: PathBuf,
    pub config_path: PathBuf,
    pub downloading_model: Option<String>,
    pub active_download: Option<ModelDownloadProgress>,
    pub download_event_rx: Option<std::sync::mpsc::Receiver<ModelDownloadEvent>>,
    pub is_connected: Arc<AtomicBool>,
    pub connection_state: Arc<std::sync::RwLock<ConnectionState>>,
    pub on_token_saved: Option<Arc<dyn Fn(Option<String>) + Send + Sync>>,
    pub update_state: Arc<std::sync::Mutex<crate::updater::UpdateState>>,
    pub download_rx: Option<std::sync::mpsc::Receiver<Result<(PathBuf, bool), String>>>,
    pub show_restart_modal: bool,
    pub pending_restart_script: Option<PathBuf>,
    pub store_state: crate::store::StoreState,
    pub store_icon_cache: crate::icons::SvgIconCache,
    pub readme_modal: Option<PluginReadmeState>,
    pub photocraft_path: String,
    pub launch_requested: bool,
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

        let photocraft_path = cfg
            .photocraft_path
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

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

        let init_state = if is_connected.load(Ordering::Relaxed) {
            ConnectionState::Connected
        } else {
            ConnectionState::Offline
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
            status_timestamp: None,
            status_is_error: false,
            prefs_path,
            plugins_dir,
            config_path,
            downloading_model: None,
            active_download: None,
            download_event_rx: None,
            is_connected,
            connection_state: Arc::new(std::sync::RwLock::new(init_state)),
            on_token_saved: None,
            update_state: Arc::new(std::sync::Mutex::new(crate::updater::UpdateState::Idle)),
            download_rx: None,
            show_restart_modal: false,
            pending_restart_script: None,
            store_state: crate::store::StoreState::new(&data_dir),
            store_icon_cache: crate::icons::SvgIconCache::new(),
            readme_modal: None,
            photocraft_path,
            launch_requested: false,
        }
    }

    pub fn with_connection_state(
        mut self,
        connection_state: Arc<std::sync::RwLock<ConnectionState>>,
    ) -> Self {
        self.connection_state = connection_state;
        self
    }

    pub fn with_token_callback(
        mut self,
        callback: Arc<dyn Fn(Option<String>) + Send + Sync>,
    ) -> Self {
        self.on_token_saved = Some(callback);
        self
    }

    pub fn t<'a>(&self, en: &'a str, ru: &'a str) -> &'a str {
        match self.current_language {
            Language::Ru => ru,
            _ => en,
        }
    }

    pub fn set_status(&mut self, text: impl Into<String>, is_error: bool) {
        self.status_text = text.into();
        self.status_timestamp = Some(std::time::Instant::now());
        self.status_is_error = is_error;
    }

    pub fn poll_download_events(&mut self) {
        let mut events = Vec::new();
        if let Some(ref rx) = self.download_event_rx {
            while let Ok(event) = rx.try_recv() {
                events.push(event);
            }
        }
        for event in events {
            match event {
                ModelDownloadEvent::Progress(prog) => {
                    self.active_download = Some(prog);
                }
                ModelDownloadEvent::Finished { model_name } => {
                    self.active_download = None;
                    let msg_success = self.t(
                        "Model downloaded successfully!",
                        "Модель успешно загружена!",
                    );
                    self.set_status(format!("{}: {}", model_name, msg_success), false);
                }
                ModelDownloadEvent::Error { model_name, error } => {
                    self.active_download = None;
                    let msg_err = if error == "SHA256 checksum verification failed" {
                        self.t("Checksum mismatch", "Несовпадение контрольной суммы")
                    } else {
                        self.t("Download error", "Ошибка загрузки")
                    };
                    self.set_status(format!("{}: {}: {}", model_name, msg_err, error), true);
                }
            }
        }
    }

    pub fn poll_store_events(&mut self) {
        let mut events = Vec::new();
        while let Ok(event) = self.store_state.event_rx.try_recv() {
            events.push(event);
        }

        for event in events {
            match event {
                crate::store::StoreEvent::RegistryLoaded {
                    registry,
                    is_offline,
                } => {
                    self.store_state.status = crate::store::StoreLoadStatus::Ready { is_offline };
                    self.store_state.registry = Some(registry);
                    let msg = if is_offline {
                        self.t(
                            "Catalog loaded from offline cache",
                            "Каталог загружен из оффлайн-кэша",
                        )
                    } else {
                        self.t("Catalog updated successfully", "Каталог успешно обновлен")
                    };
                    self.set_status(msg, false);
                }
                crate::store::StoreEvent::RegistryFailed(err) => {
                    self.store_state.status = crate::store::StoreLoadStatus::Error(err.clone());
                    let msg_prefix = self.t("Catalog loading failed", "Ошибка загрузки каталога");
                    self.set_status(format!("{}: {}", msg_prefix, err), true);
                }
                crate::store::StoreEvent::PluginInstalled {
                    plugin_id,
                    manifest,
                } => {
                    self.store_state
                        .installing_plugins
                        .retain(|id| id != &plugin_id);

                    // If plugin already exists locally, update it in place; otherwise append
                    if let Some(pos) = self.manifests.iter().position(|m| m.id == plugin_id) {
                        self.manifests[pos] = *manifest;
                    } else {
                        self.manifests.push(*manifest);
                    }

                    let msg_inst = self.t(
                        "Plugin installed successfully!",
                        "Плагин успешно установлен!",
                    );
                    self.set_status(format!("{}: {}", plugin_id, msg_inst), false);
                }
                crate::store::StoreEvent::PluginInstallFailed { plugin_id, error } => {
                    self.store_state
                        .installing_plugins
                        .retain(|id| id != &plugin_id);
                    let msg_err = self.t("Installation failed", "Ошибка установки");
                    self.set_status(format!("{}: {}: {}", plugin_id, msg_err, error), true);
                }
                crate::store::StoreEvent::IconDownloaded { .. } => {
                    // Cache will be queried on next frame
                }
            }
        }
    }

    pub fn save_settings(&mut self) -> anyhow::Result<()> {
        let mut cfg = CompanionConfig::load_or_default(&self.config_path);
        cfg.host = self.host.clone();
        cfg.port = self.port;
        let token_opt = if self.token.trim().is_empty() {
            None
        } else {
            Some(self.token.trim().to_string())
        };
        cfg.token = token_opt.clone();
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
        cfg.photocraft_path = if self.photocraft_path.trim().is_empty() {
            None
        } else {
            Some(PathBuf::from(self.photocraft_path.trim()))
        };

        cfg.save(&self.config_path)?;
        info!("Saved settings to {}", self.config_path.display());

        // Notify runtime client immediately
        if let Some(ref cb) = self.on_token_saved {
            cb(token_opt);
        }

        let saved_msg = self
            .t(
                "Settings saved successfully!",
                "Настройки успешно сохранены!",
            )
            .to_string();
        self.set_status(saved_msg, false);
        Ok(())
    }

    /// Renders top horizontal tabs
    pub fn render_tabs_header(&mut self, ui: &mut Ui, palette: &ThemePalette) {
        let tabs = [
            (ActiveTab::General, self.t("General", "Общие")),
            (ActiveTab::Plugins, self.t("Plugins", "Плагины")),
            (ActiveTab::Store, self.t("Store", "Магазин")),
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

                let btn =
                    egui::Button::new(RichText::new(title).size(12.0).strong().color(text_color))
                        .fill(bg_color)
                        .corner_radius(6.0)
                        .min_size(Vec2::new(96.0, 26.0));

                if ui.add(btn).clicked() {
                    self.active_tab = tab;
                }
                ui.add_space(4.0);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(4.0);
                let is_dark = self.current_theme.is_dark();
                let btn_size = Vec2::splat(26.0);
                let (rect, response) = ui.allocate_exact_size(btn_size, egui::Sense::click());

                let is_hovered = response.hovered();
                let bg_color = if is_hovered {
                    palette.button_secondary_hover
                } else {
                    palette.button_secondary
                };

                ui.painter().rect_filled(rect, 6.0, bg_color);

                if let Some(tex) =
                    self.store_icon_cache
                        .get_or_load_discord(ui.ctx(), is_hovered, is_dark)
                {
                    let icon_size = Vec2::new(16.0, 12.44);
                    let icon_rect = egui::Rect::from_center_size(rect.center(), icon_size);
                    ui.painter().image(
                        tex.id(),
                        icon_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }

                let response = response.on_hover_text(self.t(
                    "Join Discord Community",
                    "Присоединиться к Discord-сообществу",
                ));

                if response.clicked() {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(
                        crate::updater::COMPANION_DISCORD_URL,
                    ));
                }
            });
        });
    }

    pub fn render_status_bullet(ui: &mut Ui, state: ConnectionState) {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(10.0), egui::Sense::hover());
        let color = match state {
            ConnectionState::Connected => Color32::from_rgb(0x28, 0xa7, 0x45),
            ConnectionState::AuthRequired => Color32::from_rgb(0xea, 0x86, 0x00),
            ConnectionState::Offline => Color32::from_rgb(0x88, 0x88, 0x88),
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
                let lang_label = match self.current_language {
                    Language::Ru => "Русский (RU)",
                    _ => "English (EN)",
                };
                egui::ComboBox::from_id_salt("general_lang_select")
                    .selected_text(RichText::new(lang_label).color(palette.text_primary))
                    .width(180.0)
                    .show_ui(ui, |ui| {
                        ui.style_mut().visuals.widgets.inactive.bg_fill = Color32::TRANSPARENT;
                        ui.selectable_value(
                            &mut self.current_language,
                            Language::En,
                            "English (EN)",
                        );
                        ui.selectable_value(
                            &mut self.current_language,
                            Language::Ru,
                            "Русский (RU)",
                        );
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
                            dialog = dialog.add_filter(
                                "PhotoCraft Executable / AppImage / Desktop Entry",
                                &["AppImage", "appimage", "desktop", "bin", ""],
                            );
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
                        if let Err(e) = self.save_settings() {
                            tracing::error!("Failed to auto-save settings before launch: {e}");
                        }
                        self.launch_requested = true;
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
                            let msg = self
                                .t("Token updated successfully", "Токен успешно обновлен")
                                .to_string();
                            self.set_status(msg, false);
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
                                    let msg = self
                                        .t("Token loaded from file", "Токен загружен из файла")
                                        .to_string();
                                    self.set_status(msg, false);
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
                    RichText::new(
                        self.t("+ Install Plugin (.pcpl)", "+ Установить плагин (.pcpl)"),
                    )
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
                                let msg = self
                                    .t(
                                        "Plugin installed successfully!",
                                        "Плагин успешно установлен!",
                                    )
                                    .to_string();
                                self.set_status(msg, false);
                                self.manifests.push(m);
                            }
                            Err(e) => {
                                self.set_status(format!("Error: {e}"), true);
                            }
                        }
                    }
                }

                // Open plugins directory button
                let open_folder_btn = egui::Button::new(
                    RichText::new(self.t("Open Plugins Folder", "Папка с плагинами"))
                        .size(11.0)
                        .color(palette.text_primary),
                )
                .fill(palette.button_secondary)
                .corner_radius(4.0)
                .min_size(Vec2::new(130.0, 24.0));

                if ui.add(open_folder_btn).clicked() {
                    let dir = if self.plugins_dir.exists() {
                        self.plugins_dir.clone()
                    } else if let Some(parent) = self
                        .manifests
                        .iter()
                        .find_map(|m| m.plugin_dir.as_ref().and_then(|p| p.parent()))
                    {
                        parent.to_path_buf()
                    } else {
                        self.plugins_dir.clone()
                    };
                    let _ = std::fs::create_dir_all(&dir);
                    crate::updater::reveal_in_folder(&dir);
                }
            });
        });

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);

        let msg_model_ready = self.t("Model ready", "Модель готова");
        let msg_model_missing = self.t("Model missing", "Требуется модель");
        let msg_download = self.t("Download model", "Скачать модель");
        let msg_download_started = self.t("Model download started...", "Загрузка модели начата...");
        let msg_view_doc = self.t(
            "View Documentation & Use Cases",
            "Документация и сценарии использования",
        );
        let msg_no_doc = self.t(
            "No documentation found for this plugin.",
            "Документация для этого плагина не найдена.",
        );
        let plugins_dir = self.plugins_dir.clone();
        let cur_lang = self.current_language;
        let mut download_notice = None;
        let mut open_readme = None;

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
                            PluginType::NativeAlgorithm => "ALGO",
                            PluginType::Script => "SCRIPT",
                            PluginType::ExternalCli => "CLI",
                            PluginType::InteractiveUi => "UI PANEL",
                        };

                        let type_color = match manifest.plugin_type {
                            PluginType::NativeAi => palette.accent_blue,
                            PluginType::NativeAlgorithm => Color32::from_rgb(0x17, 0xa2, 0xb8),
                            PluginType::NoCode => Color32::from_rgb(0x28, 0xa7, 0x45),
                            PluginType::InteractiveUi => Color32::from_rgb(0x9b, 0x59, 0xb6),
                            _ => palette.text_muted,
                        };

                        ui.label(RichText::new(type_str).size(9.0).color(type_color).strong());
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

                        // Documentation & Use Cases vector icon button
                        let (doc_rect, doc_resp) =
                            ui.allocate_exact_size(Vec2::new(16.0, 16.0), egui::Sense::click());
                        let doc_resp = doc_resp.on_hover_text(msg_view_doc);
                        let doc_color = if doc_resp.hovered() {
                            palette.accent_blue
                        } else {
                            palette.text_muted
                        };

                        // Vector document sheet with folded corner and text lines
                        let icon_rect = doc_rect.shrink(1.5);
                        let fold_size = 3.5;
                        let p1 = egui::pos2(icon_rect.min.x, icon_rect.min.y);
                        let p2 = egui::pos2(icon_rect.max.x - fold_size, icon_rect.min.y);
                        let p3 = egui::pos2(icon_rect.max.x, icon_rect.min.y + fold_size);
                        let p4 = egui::pos2(icon_rect.max.x, icon_rect.max.y);
                        let p5 = egui::pos2(icon_rect.min.x, icon_rect.max.y);

                        let stroke = Stroke::new(1.0, doc_color);
                        ui.painter().line_segment([p1, p2], stroke);
                        ui.painter().line_segment([p2, p3], stroke);
                        ui.painter().line_segment([p3, p4], stroke);
                        ui.painter().line_segment([p4, p5], stroke);
                        ui.painter().line_segment([p5, p1], stroke);

                        let p_fold =
                            egui::pos2(icon_rect.max.x - fold_size, icon_rect.min.y + fold_size);
                        ui.painter().line_segment([p2, p_fold], stroke);
                        ui.painter().line_segment([p_fold, p3], stroke);

                        let lx1 = icon_rect.min.x + 2.5;
                        let lx2 = icon_rect.max.x - 2.5;
                        let ly1 = icon_rect.min.y + 6.0;
                        let ly2 = icon_rect.min.y + 9.0;
                        ui.painter()
                            .line_segment([egui::pos2(lx1, ly1), egui::pos2(lx2, ly1)], stroke);
                        ui.painter().line_segment(
                            [egui::pos2(lx1, ly2), egui::pos2(lx2 - 2.0, ly2)],
                            stroke,
                        );

                        if doc_resp.clicked() {
                            let plugin_dir = manifest
                                .plugin_dir
                                .clone()
                                .unwrap_or_else(|| plugins_dir.join(&manifest.id));
                            let readme_path = plugin_dir.join("README.md");
                            let raw_content = match std::fs::read_to_string(&readme_path) {
                                Ok(content) => content,
                                Err(_) => {
                                    let alt_path = PathBuf::from("plugins")
                                        .join(&manifest.id)
                                        .join("README.md");
                                    std::fs::read_to_string(&alt_path).unwrap_or_else(|_| {
                                        format!("# {}\n\n{}", manifest.name, msg_no_doc)
                                    })
                                }
                            };
                            open_readme = Some((manifest.name.clone(), plugin_dir, raw_content));
                        }

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
                        ui.label(
                            RichText::new(format!("by {}", manifest.author))
                                .size(10.0)
                                .color(palette.text_muted),
                        );

                        // Models status badge (clean vector circle dot + label)
                        if !manifest.models.is_empty() {
                            let (dot_rect, _) =
                                ui.allocate_exact_size(Vec2::new(8.0, 10.0), egui::Sense::hover());
                            let dot_color = if all_models_ok {
                                Color32::from_rgb(0x28, 0xa7, 0x45)
                            } else {
                                Color32::from_rgb(0xea, 0x86, 0x00)
                            };
                            ui.painter()
                                .circle_filled(dot_rect.center(), 2.8, dot_color);

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

                                // Check if this plugin's model is currently downloading
                                let is_curr_downloading = self
                                    .active_download
                                    .as_ref()
                                    .map(|d| d.plugin_id == manifest.id)
                                    .unwrap_or(false);

                                if is_curr_downloading {
                                    if let Some(ref d) = self.active_download {
                                        let frac = if d.total_bytes > 0 {
                                            (d.downloaded_bytes as f32 / d.total_bytes as f32)
                                                .clamp(0.0, 1.0)
                                        } else {
                                            0.0
                                        };
                                        let dl_mb = d.downloaded_bytes as f32 / (1024.0 * 1024.0);
                                        let tot_mb = d.total_bytes as f32 / (1024.0 * 1024.0);
                                        let progress_text = format!(
                                            "{:.0}% ({:.1} / {:.1} MB)",
                                            frac * 100.0,
                                            dl_mb,
                                            tot_mb
                                        );

                                        let bar = egui::ProgressBar::new(frac)
                                            .text(
                                                RichText::new(progress_text)
                                                    .size(9.0)
                                                    .color(Color32::WHITE),
                                            )
                                            .desired_width(140.0)
                                            .corner_radius(2.0);
                                        ui.add(bar);
                                    }
                                } else if let Some(req) = manifest
                                    .models
                                    .iter()
                                    .find(|m| !manifest.is_model_installed(m) && m.url.is_some())
                                {
                                    if let Some(ref url) = req.url {
                                        let dl_btn = egui::Button::new(
                                            RichText::new(msg_download)
                                                .size(10.0)
                                                .color(Color32::WHITE),
                                        )
                                        .fill(palette.button_secondary)
                                        .corner_radius(2.0);

                                        if ui.add(dl_btn).clicked() {
                                            let target_dir = {
                                                let candidate = manifest
                                                    .plugin_dir
                                                    .clone()
                                                    .unwrap_or_else(|| {
                                                        PathBuf::from("plugins").join(&manifest.id)
                                                    })
                                                    .join("models");
                                                if std::fs::create_dir_all(&candidate).is_ok()
                                                    && core_contracts::is_dir_writable(&candidate)
                                                {
                                                    candidate
                                                } else {
                                                    let fallback = core_contracts::user_data_dir()
                                                        .join("plugins")
                                                        .join(&manifest.id)
                                                        .join("models");
                                                    let _ = std::fs::create_dir_all(&fallback);
                                                    fallback
                                                }
                                            };
                                            let target_file = target_dir.join(&req.name);
                                            let part_file =
                                                target_dir.join(format!("{}.part", req.name));
                                            let url_clone = url.clone();
                                            let model_name = req.name.clone();
                                            let plugin_id = manifest.id.clone();
                                            let total_bytes =
                                                req.size_mb.map(|mb| mb * 1024 * 1024).unwrap_or(0);

                                            let (tx, rx) = std::sync::mpsc::channel();
                                            self.download_event_rx = Some(rx);
                                            self.active_download = Some(ModelDownloadProgress {
                                                model_name: model_name.clone(),
                                                plugin_id: plugin_id.clone(),
                                                downloaded_bytes: 0,
                                                total_bytes,
                                                speed_bytes_sec: 0.0,
                                            });

                                            let done_flag = Arc::new(AtomicBool::new(false));
                                            let done_flag_monitor = done_flag.clone();

                                            // Worker thread: executes curl writing to .part file
                                            let part_file_worker = part_file.clone();
                                            let target_file_worker = target_file.clone();
                                            let model_name_worker = model_name.clone();
                                            let req_sha256 = req.sha256.clone();
                                            let tx_worker = tx.clone();
                                            std::thread::spawn(move || {
                                                // Clean up any stale partial download first
                                                let _ = std::fs::remove_file(&part_file_worker);

                                                let mut cmd = if cfg!(windows) {
                                                    if std::path::Path::new(
                                                        "C:\\Windows\\System32\\curl.exe",
                                                    )
                                                    .exists()
                                                    {
                                                        std::process::Command::new(
                                                            "C:\\Windows\\System32\\curl.exe",
                                                        )
                                                    } else {
                                                        std::process::Command::new("curl.exe")
                                                    }
                                                } else if std::path::Path::new("/usr/bin/curl")
                                                    .exists()
                                                {
                                                    std::process::Command::new("/usr/bin/curl")
                                                } else {
                                                    std::process::Command::new("curl")
                                                };
                                                cmd.arg("--create-dirs")
                                                    .arg("-L")
                                                    .arg("-f")
                                                    .arg("-o")
                                                    .arg(&part_file_worker)
                                                    .arg(&url_clone);
                                                #[cfg(windows)]
                                                {
                                                    use std::os::windows::process::CommandExt;
                                                    cmd.creation_flags(0x08000000);
                                                }
                                                let status = cmd.status();

                                                done_flag.store(true, Ordering::SeqCst);

                                                match status {
                                                    Ok(exit_status) if exit_status.success() => {
                                                        // Verify SHA256 checksum if provided in manifest
                                                        if let Some(expected_sha256) = &req_sha256 {
                                                            use std::io::Read;
                                                            let matches = match std::fs::File::open(&part_file_worker) {
                                                                Ok(mut file) => {
                                                                    use sha2::{Digest, Sha256};
                                                                    let mut hasher = Sha256::new();
                                                                    let mut buffer = [0u8; 64 * 1024];
                                                                    while let Ok(n) = file.read(&mut buffer) {
                                                                        if n == 0 {
                                                                            break;
                                                                        }
                                                                        hasher.update(&buffer[..n]);
                                                                    }
                                                                    let actual_hex = hex::encode(hasher.finalize());
                                                                    actual_hex.eq_ignore_ascii_case(expected_sha256.trim())
                                                                }
                                                                Err(_) => false,
                                                            };

                                                            if !matches {
                                                                let _ = std::fs::remove_file(&part_file_worker);
                                                                let _ = tx_worker.send(
                                                                    ModelDownloadEvent::Error {
                                                                        model_name: model_name_worker,
                                                                        error: "SHA256 checksum verification failed".to_string(),
                                                                    },
                                                                );
                                                                return;
                                                            }
                                                        }

                                                        // Atomic rename from .part to final file
                                                        if let Err(e) = std::fs::rename(
                                                            &part_file_worker,
                                                            &target_file_worker,
                                                        ) {
                                                            let _ = tx_worker.send(
                                                                ModelDownloadEvent::Error {
                                                                    model_name: model_name_worker,
                                                                    error: e.to_string(),
                                                                },
                                                            );
                                                        } else {
                                                            let _ = tx_worker.send(
                                                                ModelDownloadEvent::Finished {
                                                                    model_name: model_name_worker,
                                                                },
                                                            );
                                                        }
                                                    }
                                                    Ok(exit_status) => {
                                                        let _ =
                                                            std::fs::remove_file(&part_file_worker);
                                                        let _ = tx_worker.send(
                                                            ModelDownloadEvent::Error {
                                                                model_name: model_name_worker,
                                                                error: format!(
                                                                    "curl exited with code {:?}",
                                                                    exit_status.code()
                                                                ),
                                                            },
                                                        );
                                                    }
                                                    Err(e) => {
                                                        let _ =
                                                            std::fs::remove_file(&part_file_worker);
                                                        let _ = tx_worker.send(
                                                            ModelDownloadEvent::Error {
                                                                model_name: model_name_worker,
                                                                error: e.to_string(),
                                                            },
                                                        );
                                                    }
                                                }
                                            });

                                            // Monitor thread: polls .part file size every 100ms
                                            let part_file_monitor = part_file.clone();
                                            let model_name_monitor = model_name.clone();
                                            let plugin_id_monitor = plugin_id.clone();
                                            let tx_monitor = tx;
                                            std::thread::spawn(move || {
                                                let mut last_bytes = 0u64;
                                                let mut last_time = std::time::Instant::now();

                                                while !done_flag_monitor.load(Ordering::SeqCst) {
                                                    std::thread::sleep(
                                                        std::time::Duration::from_millis(100),
                                                    );
                                                    if let Ok(meta) =
                                                        std::fs::metadata(&part_file_monitor)
                                                    {
                                                        let cur_bytes = meta.len();
                                                        let now = std::time::Instant::now();
                                                        let dt = now
                                                            .duration_since(last_time)
                                                            .as_secs_f32();
                                                        let speed = if dt > 0.05
                                                            && cur_bytes >= last_bytes
                                                        {
                                                            (cur_bytes - last_bytes) as f32 / dt
                                                        } else {
                                                            0.0
                                                        };
                                                        last_bytes = cur_bytes;
                                                        last_time = now;

                                                        let _ = tx_monitor.send(
                                                            ModelDownloadEvent::Progress(
                                                                ModelDownloadProgress {
                                                                    model_name: model_name_monitor
                                                                        .clone(),
                                                                    plugin_id: plugin_id_monitor
                                                                        .clone(),
                                                                    downloaded_bytes: cur_bytes,
                                                                    total_bytes,
                                                                    speed_bytes_sec: speed,
                                                                },
                                                            ),
                                                        );
                                                    }
                                                }
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

        if let Some((name, plugin_dir, raw_content)) = open_readme {
            self.readme_modal = Some(PluginReadmeState {
                plugin_name: name,
                plugin_dir,
                raw_content,
                active_lang: cur_lang,
            });
        }

        if let Some(msg) = download_notice {
            self.set_status(msg, false);
        }
    }

    /// Tab 3: About
    pub fn render_about_tab(&mut self, ui: &mut Ui, palette: &ThemePalette) {
        // Poll any background download completion
        if let Some(ref rx) = self.download_rx {
            if let Ok(res) = rx.try_recv() {
                let mut state_guard = self.update_state.lock().unwrap();
                match res {
                    Ok((path, is_portable)) => {
                        *state_guard = crate::updater::UpdateState::Downloaded {
                            target_path: path,
                            is_portable,
                        };
                    }
                    Err(e) => {
                        *state_guard = crate::updater::UpdateState::Error(e);
                    }
                }
                self.download_rx = None;
            }
        }

        ui.heading(
            RichText::new("PhotoCraft Companion")
                .size(16.0)
                .color(palette.text_primary),
        );
        ui.label(
            RichText::new(format!(
                "Version {} (Rust Cargo Workspace & DirectML GPU)",
                env!("CARGO_PKG_VERSION")
            ))
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

        ui.add_space(10.0);

        // Community links
        ui.horizontal_wrapped(|ui| {
            ui.hyperlink_to(
                RichText::new(self.t("GitHub Repository", "Репозиторий GitHub"))
                    .size(11.0)
                    .color(palette.accent_blue),
                crate::updater::COMPANION_GITHUB_URL,
            );

            ui.label(RichText::new("|").size(11.0).color(palette.text_muted));

            ui.hyperlink_to(
                RichText::new(self.t("Discord Community", "Сообщество Discord"))
                    .size(11.0)
                    .color(palette.accent_blue),
                crate::updater::COMPANION_DISCORD_URL,
            );

            ui.label(RichText::new("|").size(11.0).color(palette.text_muted));

            ui.hyperlink_to(
                RichText::new(self.t(
                    "Report Issue / Feedback",
                    "Сообщить об ошибке / Предложить идею",
                ))
                .size(11.0)
                .color(palette.accent_blue),
                crate::updater::COMPANION_ISSUES_URL,
            );
        });

        ui.add_space(10.0);

        // Updater Card
        egui::Frame::new()
            .fill(palette.card_inner)
            .stroke(Stroke::new(1.0, palette.border_stroke))
            .corner_radius(6.0)
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(self.t("Software Updates", "Обновления программы"))
                            .size(13.0)
                            .strong()
                            .color(palette.text_primary),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let check_btn = egui::Button::new(
                            RichText::new(self.t("Check for Updates", "Проверить обновления"))
                                .size(11.0)
                                .color(Color32::WHITE),
                        )
                        .fill(palette.accent_blue)
                        .corner_radius(4.0)
                        .min_size(Vec2::new(130.0, 24.0));

                        let is_checking = {
                            let guard = self.update_state.lock().unwrap();
                            matches!(*guard, crate::updater::UpdateState::Checking)
                        };

                        if !is_checking && ui.add(check_btn).clicked() {
                            let state_arc = self.update_state.clone();
                            {
                                let mut guard = state_arc.lock().unwrap();
                                *guard = crate::updater::UpdateState::Checking;
                            }
                            let current_ver = env!("CARGO_PKG_VERSION").to_string();
                            std::thread::spawn(move || {
                                match crate::updater::fetch_latest_release(crate::updater::COMPANION_REPO) {
                                    Ok(res) => {
                                        let is_newer = crate::updater::is_version_newer(&res.version, &current_ver);
                                        let mut guard = state_arc.lock().unwrap();
                                        if is_newer {
                                            *guard = crate::updater::UpdateState::UpdateAvailable { result: res };
                                        } else {
                                            *guard = crate::updater::UpdateState::UpToDate {
                                                checked_version: res.version,
                                            };
                                        }
                                    }
                                    Err(err) => {
                                        let mut guard = state_arc.lock().unwrap();
                                        *guard = crate::updater::UpdateState::Error(err);
                                    }
                                }
                            });
                        }
                    });
                });

                ui.add_space(8.0);

                // Render current update state
                let state_clone = {
                    let guard = self.update_state.lock().unwrap();
                    guard.clone()
                };

                match state_clone {
                    crate::updater::UpdateState::Idle => {
                        ui.label(
                            RichText::new(self.t(
                                "Click 'Check for Updates' to verify the latest release on GitHub.",
                                "Нажмите «Проверить обновления», чтобы узнать о свежих релизах на GitHub.",
                            ))
                            .size(11.0)
                            .color(palette.text_muted),
                        );
                    }
                    crate::updater::UpdateState::Checking => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(
                                RichText::new(self.t("Checking GitHub for updates...", "Проверка обновлений на GitHub..."))
                                    .size(11.0)
                                    .color(palette.accent_blue),
                            );
                        });
                        ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
                    }
                    crate::updater::UpdateState::UpToDate { checked_version } => {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(self.t("You have the latest version installed", "У вас установлена самая последняя версия"))
                                    .size(11.0)
                                    .color(Color32::from_rgb(0x28, 0xa7, 0x45)),
                            );
                            ui.label(
                                RichText::new(format!("(v{})", checked_version))
                                    .size(10.0)
                                    .color(palette.text_muted),
                            );
                        });
                    }
                    crate::updater::UpdateState::UpdateAvailable { result } => {
                        ui.label(
                            RichText::new(format!(
                                "{}: v{} -> v{}",
                                self.t("New version available", "Доступна новая версия"),
                                env!("CARGO_PKG_VERSION"),
                                result.version
                            ))
                            .size(12.0)
                            .strong()
                            .color(palette.accent_blue),
                        );

                        if !result.release_notes.is_empty() {
                            ui.add_space(4.0);
                            let notes_preview = if result.release_notes.len() > 180 {
                                format!("{}...", &result.release_notes[..180])
                            } else {
                                result.release_notes.clone()
                            };
                            ui.label(
                                RichText::new(notes_preview)
                                    .size(10.0)
                                    .color(palette.text_muted),
                            );
                        }

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.hyperlink_to(
                                RichText::new(self.t("View Release Notes", "Список изменений"))
                                    .size(10.0)
                                    .color(palette.accent_blue),
                                &result.html_url,
                            );

                            for asset in &result.assets {
                                let label = if asset.is_installer {
                                    self.t("Download Setup", "Скачать установщик")
                                } else {
                                    self.t("Download Portable", "Скачать Portable")
                                };

                                let btn = egui::Button::new(
                                    RichText::new(label)
                                        .size(10.0)
                                        .color(Color32::WHITE),
                                )
                                .fill(palette.accent_blue)
                                .corner_radius(4.0);

                                if ui.add(btn).clicked() {
                                    let downloads_dir = crate::updater::get_downloads_dir();
                                    let target_path = downloads_dir.join(&asset.name);
                                    let (tx, rx) = std::sync::mpsc::channel();
                                    self.download_rx = Some(rx);

                                    {
                                        let mut guard = self.update_state.lock().unwrap();
                                        *guard = crate::updater::UpdateState::Downloading {
                                            asset_name: asset.name.clone(),
                                            target_path: target_path.clone(),
                                            is_portable: asset.is_portable,
                                        };
                                    }

                                    crate::updater::download_file_background(
                                        asset.download_url.clone(),
                                        target_path,
                                        asset.is_portable,
                                        tx,
                                    );
                                }
                            }
                        });
                    }
                    crate::updater::UpdateState::Downloading {
                        ref asset_name,
                        ref target_path,
                        ..
                    } => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(
                                RichText::new(format!(
                                    "{} {}: {}",
                                    self.t("Downloading", "Загрузка"),
                                    asset_name,
                                    target_path.display()
                                ))
                                .size(11.0)
                                .color(palette.accent_blue),
                            );
                        });
                        ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
                    }
                    crate::updater::UpdateState::Downloaded {
                        ref target_path,
                        is_portable,
                    } => {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(self.t("Update downloaded successfully!", "Обновление успешно загружено!"))
                                    .size(11.0)
                                    .strong()
                                    .color(Color32::from_rgb(0x28, 0xa7, 0x45)),
                            );
                        });

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            if is_portable {
                                let install_btn = egui::Button::new(
                                    RichText::new(self.t("Apply and Restart", "Установить и перезапустить"))
                                        .size(11.0)
                                        .color(Color32::WHITE),
                                )
                                .fill(palette.accent_blue)
                                .corner_radius(4.0);

                                if ui.add(install_btn).clicked() {
                                    match crate::updater::prepare_portable_update(target_path) {
                                        Ok(script) => {
                                            self.pending_restart_script = Some(script);
                                            self.show_restart_modal = true;
                                        }
                                        Err(e) => {
                                            let mut guard = self.update_state.lock().unwrap();
                                            *guard = crate::updater::UpdateState::Error(e);
                                        }
                                    }
                                }
                            } else {
                                let run_btn = egui::Button::new(
                                    RichText::new(self.t("Run Installer", "Запустить установщик"))
                                        .size(11.0)
                                        .color(Color32::WHITE),
                                )
                                .fill(palette.accent_blue)
                                .corner_radius(4.0);

                                if ui.add(run_btn).clicked() {
                                    if let Err(e) = crate::updater::launch_installer(target_path) {
                                        let mut guard = self.update_state.lock().unwrap();
                                        *guard = crate::updater::UpdateState::Error(e);
                                    }
                                }
                            }

                            let show_btn = egui::Button::new(
                                RichText::new(self.t("Show in Folder", "Показать в папке"))
                                    .size(11.0)
                                    .color(palette.text_primary),
                            )
                            .corner_radius(4.0);

                            if ui.add(show_btn).clicked() {
                                crate::updater::reveal_in_folder(target_path);
                            }
                        });
                    }
                    crate::updater::UpdateState::ApplyingUpdate => {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(
                                RichText::new(self.t("Restarting application...", "Перезапуск приложения..."))
                                    .size(11.0)
                                    .color(palette.accent_blue),
                            );
                        });
                    }
                    crate::updater::UpdateState::Error(ref err) => {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("{}: {}", self.t("Error", "Ошибка"), err))
                                    .size(11.0)
                                    .color(Color32::from_rgb(0xe5, 0x39, 0x35)),
                            );
                        });
                    }
                }
            });

        ui.add_space(12.0);
        ui.label(
            RichText::new(self.t(
                "- Engine: Pure Rust + tokio JSON-RPC Bridge (Port 18492)\n- AI Hardware: DirectML (NVIDIA RTX / AMD / Intel) with CPU fallback\n- GUI: Immediate-mode egui/eframe with PhotoCraft graphite palette\n- License: MIT OR Apache-2.0",
                "- Движок: Чистый Rust + tokio JSON-RPC сокет (порт 18492)\n- Аппаратный ИИ: DirectML (NVIDIA RTX / AMD / Intel) с откатом на CPU\n- Интерфейс: Immediate-mode egui/eframe в фирменной графитовой палитре\n- Лицензия: MIT OR Apache-2.0",
            ))
            .size(11.0)
            .color(palette.text_muted),
        );
    }

    /// Tab 3: Marketplace / Catalog
    pub fn render_store_tab(&mut self, ui: &mut Ui, palette: &ThemePalette) {
        // Auto-fetch catalog if idle
        if self.store_state.status == crate::store::StoreLoadStatus::Idle {
            self.store_state.refresh_catalog(None);
        }

        // Top controls: Title, Refresh button, Search and Category filters
        ui.horizontal(|ui| {
            ui.heading(
                RichText::new(self.t("Plugin Marketplace", "Магазин расширений"))
                    .size(14.0)
                    .color(palette.text_primary),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let is_loading = self.store_state.status == crate::store::StoreLoadStatus::Loading;
                let refresh_btn = egui::Button::new(
                    RichText::new(if is_loading {
                        self.t("Loading...", "Загрузка...")
                    } else {
                        self.t("Refresh Catalog", "Обновить каталог")
                    })
                    .size(11.0)
                    .color(palette.text_primary),
                )
                .fill(palette.button_secondary)
                .corner_radius(4.0);

                if ui.add_enabled(!is_loading, refresh_btn).clicked() {
                    self.store_state.refresh_catalog(None);
                }
            });
        });

        ui.add_space(8.0);

        // Filter and Search Bar
        let search_hint = self.t("Search plugins...", "Поиск плагинов...");
        ui.horizontal(|ui| {
            // Search field
            ui.add(
                egui::TextEdit::singleline(&mut self.store_state.search_query)
                    .hint_text(search_hint)
                    .desired_width(200.0),
            );

            ui.add_space(8.0);

            // Category filter pills
            let filters = [
                (crate::store::StoreCategoryFilter::All, self.t("All", "Все")),
                (
                    crate::store::StoreCategoryFilter::Ai,
                    self.t("AI Models", "Нейросети (AI)"),
                ),
                (
                    crate::store::StoreCategoryFilter::Macro,
                    self.t("Macros", "Макросы"),
                ),
            ];

            for (filt, label) in filters {
                let is_sel = self.store_state.category_filter == filt;
                let bg = if is_sel {
                    palette.accent_blue
                } else {
                    palette.button_secondary
                };
                let fg = if is_sel {
                    Color32::WHITE
                } else {
                    palette.text_primary
                };
                let btn = egui::Button::new(RichText::new(label).size(10.0).color(fg))
                    .fill(bg)
                    .corner_radius(4.0)
                    .min_size(Vec2::new(50.0, 20.0));
                if ui.add(btn).clicked() {
                    self.store_state.category_filter = filt;
                }
            }
        });

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(6.0);

        // Catalog Status Banner (Offline cache notice or Loading/Error)
        match self.store_state.status {
            crate::store::StoreLoadStatus::Loading => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(
                        RichText::new(self.t(
                            "Checking plugin repository...",
                            "Проверка репозитория плагинов...",
                        ))
                        .size(11.0)
                        .color(palette.accent_blue),
                    );
                });
                ui.add_space(6.0);
            }
            crate::store::StoreLoadStatus::Ready { is_offline: true } => {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(self.t(
                            "Offline Mode: Showing cached catalog.",
                            "Офлайн-режим: отображается кэшированный каталог.",
                        ))
                        .size(11.0)
                        .color(palette.text_muted),
                    );
                });
                ui.add_space(4.0);
            }
            crate::store::StoreLoadStatus::Error(ref err) => {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(format!(
                            "{}: {}",
                            self.t("Catalog error", "Ошибка каталога"),
                            err
                        ))
                        .size(11.0)
                        .color(Color32::from_rgb(0xe5, 0x39, 0x35)),
                    );
                });
                ui.add_space(6.0);
            }
            _ => {}
        }

        let plugins = match &self.store_state.registry {
            Some(reg) => reg.plugins.clone(),
            None => Vec::new(),
        };

        if plugins.is_empty() && self.store_state.status != crate::store::StoreLoadStatus::Loading {
            ui.label(
                RichText::new(self.t(
                    "No plugins available in catalog.",
                    "В каталоге нет доступных плагинов.",
                ))
                .size(12.0)
                .color(palette.text_muted),
            );
            return;
        }

        let query = self.store_state.search_query.trim().to_lowercase();
        let cat_filter = self.store_state.category_filter;

        for entry in &plugins {
            // Filter by search query
            if !query.is_empty() {
                let matches_name = entry.name.to_lowercase().contains(&query);
                let matches_author = entry.author.to_lowercase().contains(&query);
                let matches_desc = entry.description.to_lowercase().contains(&query);
                if !matches_name && !matches_author && !matches_desc {
                    continue;
                }
            }

            // Filter by category
            match cat_filter {
                crate::store::StoreCategoryFilter::Ai => {
                    if entry.plugin_type != "native_ai" {
                        continue;
                    }
                }
                crate::store::StoreCategoryFilter::Macro => {
                    if entry.plugin_type == "native_ai" {
                        continue;
                    }
                }
                crate::store::StoreCategoryFilter::All => {}
            }

            // Ensure author SVG icon is cached in background
            self.store_state.ensure_icon_cached(entry);

            // Determine installation status and local version
            let local_manifest = self.manifests.iter().find(|m| m.id == entry.id);
            let is_installed = local_manifest.is_some();
            let is_installing = self.store_state.installing_plugins.contains(&entry.id);
            let has_update = if let Some(local) = local_manifest {
                crate::store::is_store_version_newer(&entry.version, &local.version)
            } else {
                false
            };

            // Render plugin card
            egui::Frame::new()
                .fill(palette.card_inner)
                .stroke(Stroke::new(1.0, palette.border_stroke))
                .corner_radius(6.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // 1. Vector SVG Icon (24x24)
                        let cached_icon_path = self.store_state.get_cached_icon_path(&entry.id);
                        let is_dark = self.current_theme.is_dark();
                        let texture = if cached_icon_path.exists() {
                            self.store_icon_cache.get_or_load(
                                ui.ctx(),
                                &entry.id,
                                Some(&cached_icon_path),
                                is_dark,
                            )
                        } else {
                            None
                        };

                        if let Some(tex) = texture {
                            ui.image((tex.id(), Vec2::new(24.0, 24.0)));
                        } else {
                            // Vector fallback badge if icon not downloaded yet
                            let badge_text = if entry.plugin_type == "native_ai" {
                                "AI"
                            } else {
                                "FX"
                            };
                            let (badge_rect, _) =
                                ui.allocate_exact_size(Vec2::new(24.0, 24.0), egui::Sense::hover());
                            ui.painter().rect_filled(badge_rect, 4.0, palette.card_fill);
                            ui.painter().text(
                                badge_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                badge_text,
                                egui::FontId::proportional(10.0),
                                palette.accent_blue,
                            );
                        }

                        ui.add_space(4.0);

                        // 2. Title, version, author and description
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                let type_str = if entry.plugin_type == "native_ai" {
                                    "AI GPU"
                                } else {
                                    "NO-CODE"
                                };
                                let type_color = if entry.plugin_type == "native_ai" {
                                    palette.accent_blue
                                } else {
                                    Color32::from_rgb(0x28, 0xa7, 0x45)
                                };

                                ui.label(
                                    RichText::new(type_str).size(9.0).color(type_color).strong(),
                                );
                                ui.label(
                                    RichText::new(&entry.name)
                                        .size(12.0)
                                        .strong()
                                        .color(palette.text_primary),
                                );
                                ui.label(
                                    RichText::new(format!("v{}", entry.version))
                                        .size(10.0)
                                        .color(palette.text_muted),
                                );

                                if has_update {
                                    ui.label(
                                        RichText::new(
                                            self.t("Update Available", "Доступно обновление"),
                                        )
                                        .size(9.0)
                                        .color(Color32::from_rgb(0xea, 0x86, 0x00))
                                        .strong(),
                                    );
                                }
                            });

                            ui.label(
                                RichText::new(&entry.description)
                                    .size(11.0)
                                    .color(palette.text_muted),
                            );

                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("by {}", entry.author))
                                        .size(10.0)
                                        .color(palette.text_muted),
                                );

                                if let Some(m) = entry.models.first() {
                                    if let Some(mb) = m.size_mb {
                                        ui.label(
                                            RichText::new(format!(
                                                "{}: ~{} MB",
                                                self.t("Model size", "Размер модели"),
                                                mb
                                            ))
                                            .size(10.0)
                                            .color(palette.text_muted),
                                        );
                                    }
                                }
                            });
                        });

                        // 3. Action button on the right
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if is_installing {
                                ui.spinner();
                                ui.label(
                                    RichText::new(self.t("Installing...", "Установка..."))
                                        .size(11.0)
                                        .color(palette.accent_blue),
                                );
                            } else if has_update {
                                let update_label = format!(
                                    "{} (v{})",
                                    self.t("Update", "Обновить"),
                                    entry.version
                                );
                                let upd_btn = egui::Button::new(
                                    RichText::new(update_label)
                                        .size(10.0)
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .fill(Color32::from_rgb(0x28, 0xa7, 0x45))
                                .corner_radius(4.0)
                                .min_size(Vec2::new(100.0, 24.0));

                                if ui.add(upd_btn).clicked() {
                                    self.store_state
                                        .install_plugin(entry, self.plugins_dir.clone());
                                }
                            } else if is_installed {
                                let installed_btn = egui::Button::new(
                                    RichText::new(self.t("Installed", "Установлен"))
                                        .size(10.0)
                                        .color(palette.text_muted),
                                )
                                .fill(palette.button_secondary)
                                .corner_radius(4.0)
                                .min_size(Vec2::new(90.0, 24.0));

                                ui.add_enabled(false, installed_btn);
                            } else {
                                let inst_btn = egui::Button::new(
                                    RichText::new(self.t("Install", "Установить"))
                                        .size(10.0)
                                        .strong()
                                        .color(Color32::WHITE),
                                )
                                .fill(palette.accent_blue)
                                .corner_radius(4.0)
                                .min_size(Vec2::new(90.0, 24.0));

                                if ui.add(inst_btn).clicked() {
                                    self.store_state
                                        .install_plugin(entry, self.plugins_dir.clone());
                                }
                            }
                        });
                    });
                });

            ui.add_space(4.0);
        }
    }

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

    /// Master render function with horizontal tabs and pinned bottom buttons
    pub fn render_settings_window(
        &mut self,
        ui: &mut Ui,
        palette: &ThemePalette,
        is_open: &mut bool,
    ) {
        let card_fill = palette.card_fill;

        // Poll background model download events
        self.poll_download_events();
        // Poll store catalog events
        self.poll_store_events();

        // 1. Pinned bottom panel: status/connection indicator on left, [ Close ] [ Save ] on right
        egui::Panel::bottom("manager_bottom_panel")
            .frame(
                egui::Frame::new()
                    .fill(palette.bg_fill)
                    .inner_margin(egui::Margin::symmetric(14, 10)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if let Some(ref d) = self.active_download {
                        // Request frequent repaint while downloading
                        ui.ctx()
                            .request_repaint_after(std::time::Duration::from_millis(100));

                        ui.spinner();
                        ui.add_space(2.0);

                        let frac = if d.total_bytes > 0 {
                            (d.downloaded_bytes as f32 / d.total_bytes as f32).clamp(0.0, 1.0)
                        } else {
                            0.0
                        };
                        let dl_mb = d.downloaded_bytes as f32 / (1024.0 * 1024.0);
                        let tot_mb = d.total_bytes as f32 / (1024.0 * 1024.0);
                        let speed_mb = d.speed_bytes_sec / (1024.0 * 1024.0);

                        let downloading_prefix = self.t("Downloading", "Загрузка");
                        let status_line = format!(
                            "{} {}: {:.0}% ({:.1} / {:.1} MB) | {:.1} MB/s",
                            downloading_prefix,
                            d.model_name,
                            frac * 100.0,
                            dl_mb,
                            tot_mb,
                            speed_mb
                        );

                        ui.label(
                            RichText::new(status_line)
                                .size(11.0)
                                .color(palette.accent_blue),
                        );
                    } else {
                        // Check if temporary status text has expired
                        if !self.status_text.is_empty() {
                            if let Some(ts) = self.status_timestamp {
                                let timeout = if self.status_is_error {
                                    std::time::Duration::from_millis(6000)
                                } else {
                                    std::time::Duration::from_millis(3500)
                                };
                                if ts.elapsed() >= timeout {
                                    self.status_text.clear();
                                    self.status_timestamp = None;
                                } else {
                                    ui.ctx().request_repaint_after(
                                        std::time::Duration::from_millis(200),
                                    );
                                }
                            }
                        }

                        if !self.status_text.is_empty() {
                            let text_color = if self.status_is_error {
                                Color32::from_rgb(0xe5, 0x39, 0x35)
                            } else {
                                Color32::from_rgb(0x28, 0xa7, 0x45)
                            };
                            ui.label(
                                RichText::new(&self.status_text)
                                    .size(11.0)
                                    .color(text_color),
                            );
                        } else {
                            let state = *self.connection_state.read().unwrap();
                            Self::render_status_bullet(ui, state);
                            ui.add_space(2.0);
                            let (text, color) = match state {
                                ConnectionState::Connected => (
                                    self.t(
                                        "Connected (127.0.0.1:18492)",
                                        "Подключено (127.0.0.1:18492)",
                                    ),
                                    Color32::from_rgb(0x28, 0xa7, 0x45),
                                ),
                                ConnectionState::AuthRequired => (
                                    self.t(
                                        "Auth Required (Token Missing or Invalid)",
                                        "Требуется авторизация (Токен отсутствует или неверен)",
                                    ),
                                    Color32::from_rgb(0xea, 0x86, 0x00),
                                ),
                                ConnectionState::Offline => (
                                    self.t("Offline (Socket 18492)", "Офлайн (Сокет 18492)"),
                                    palette.text_muted,
                                ),
                            };
                            ui.label(RichText::new(text).size(11.0).color(color));
                        }
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
                            if let Err(e) = self.save_settings() {
                                tracing::error!("Failed to save settings: {e}");
                                let err_prefix =
                                    self.t("Error saving settings", "Ошибка сохранения настроек");
                                self.set_status(format!("{err_prefix}: {e}"), true);
                            }
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
            .frame(
                egui::Frame::new()
                    .fill(palette.bg_fill)
                    .inner_margin(egui::Margin::symmetric(14, 8)),
            )
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
                                ActiveTab::Store => self.render_store_tab(ui, palette),
                                ActiveTab::About => self.render_about_tab(ui, palette),
                            });
                    });
            });

        // Restart confirmation modal for Portable updates
        if self.show_restart_modal {
            let mut close_modal = false;
            let mut trigger_restart = false;

            egui::Window::new(self.t("Confirm Application Restart", "Подтверждение перезапуска"))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
                .min_width(380.0)
                .frame(
                    egui::Frame::new()
                        .fill(palette.card_fill)
                        .stroke(Stroke::new(1.0, palette.border_stroke))
                        .corner_radius(8.0)
                        .inner_margin(16.0),
                )
                .show(ui.ctx(), |ui| {
                    ui.label(
                        RichText::new(self.t(
                            "PhotoCraft Companion needs to close to replace files and restart with the new version. Continue?",
                            "Компаньону необходимо закрыться для замены файлов и перезапуститься с новой версией. Продолжить?",
                        ))
                        .size(12.0)
                        .color(palette.text_primary),
                    );

                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        let ok_btn = egui::Button::new(
                            RichText::new(self.t("Restart Now", "Перезапустить сейчас"))
                                .size(11.0)
                                .color(Color32::WHITE),
                        )
                        .fill(palette.accent_blue)
                        .corner_radius(4.0)
                        .min_size(Vec2::new(110.0, 24.0));

                        if ui.add(ok_btn).clicked() {
                            trigger_restart = true;
                        }

                        let cancel_btn = egui::Button::new(
                            RichText::new(self.t("Cancel", "Отмена"))
                                .size(11.0)
                                .color(palette.text_primary),
                        )
                        .fill(palette.button_secondary)
                        .corner_radius(4.0)
                        .min_size(Vec2::new(70.0, 24.0));

                        if ui.add(cancel_btn).clicked() {
                            close_modal = true;
                        }
                    });
                });

            if close_modal {
                self.show_restart_modal = false;
            }

            if trigger_restart {
                if let Some(ref script) = self.pending_restart_script {
                    let _ = crate::updater::apply_update_and_restart(script);
                }
            }
        }

        // Plugin Documentation & Use Cases modal
        self.render_readme_modal(ui.ctx(), palette);
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

fn detect_default_token(photocraft_path: &str, prefs_path: &std::path::Path) -> Option<String> {
    // 1. Environment variables
    if let Ok(tok) = std::env::var("PHOTOCRAFT_CONTROL_TOKEN") {
        let trimmed = tok.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Ok(path) = std::env::var("PHOTOCRAFT_CONTROL_TOKEN_FILE") {
        if let Ok(s) = std::fs::read_to_string(&path) {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    let mut candidates = Vec::new();

    // 2. User data directories (guaranteed writable across macOS / Windows / Linux)
    let user_data = core_contracts::user_data_dir();
    candidates.push(user_data.join("PhotoCraftData").join("control.token"));
    candidates.push(user_data.join("control.token"));

    // 3. macOS / Linux standard user config
    if let Ok(home) = std::env::var("HOME") {
        candidates.push(
            std::path::PathBuf::from(&home)
                .join("PhotoCraftData")
                .join("control.token"),
        );
        candidates.push(
            std::path::PathBuf::from(&home)
                .join("Library/Application Support/PhotoCraft/control.token"),
        );
        candidates.push(std::path::PathBuf::from(&home).join(".config/photocraft/control.token"));
    }

    // 4. Windows AppData / LocalAppData
    if let Ok(appdata) = std::env::var("APPDATA") {
        candidates.push(
            std::path::PathBuf::from(appdata)
                .join("PhotoCraft")
                .join("control.token"),
        );
    }
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        candidates.push(
            std::path::PathBuf::from(local_appdata)
                .join("PhotoCraft")
                .join("control.token"),
        );
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        candidates.push(
            std::path::PathBuf::from(profile)
                .join("PhotoCraftData")
                .join("control.token"),
        );
    }

    // 5. Configured photocraft_path
    if !photocraft_path.trim().is_empty() {
        let p = std::path::Path::new(photocraft_path.trim());
        let candidate_dir = if p.is_dir() {
            p.to_path_buf()
        } else {
            p.parent().unwrap_or(p).to_path_buf()
        };
        candidates.push(candidate_dir.join("PhotoCraftData").join("control.token"));
        candidates.push(candidate_dir.join("control.token"));
    }

    // 6. Prefs path parent
    if let Some(parent) = prefs_path.parent() {
        candidates.push(parent.join("control.token"));
        candidates.push(parent.join("PhotoCraftData").join("control.token"));
    }

    // 7. Cwd and parent
    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("PhotoCraftData").join("control.token"));
        candidates.push(cwd.join("..").join("PhotoCraftData").join("control.token"));
        candidates.push(cwd.join("control.token"));
    }

    for path in candidates {
        if let Ok(s) = std::fs::read_to_string(&path) {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_app_init() {
        let temp_dir = std::env::temp_dir().join(format!(
            "mgr_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&temp_dir);
        let prefs_file = temp_dir.join("preferences.json");
        std::fs::write(
            &prefs_file,
            r#"{"interface": {"language": "en", "theme": "studioDark"}}"#,
        )
        .unwrap();

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
        let temp_dir = std::env::temp_dir().join(format!(
            "mgr_trans_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
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
        let temp_dir = std::env::temp_dir().join(format!(
            "mgr_save_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&temp_dir);
        let prefs_file = temp_dir.join("preferences.json");
        let is_connected = Arc::new(AtomicBool::new(false));

        let mut app = PluginManagerApp::new(vec![], &prefs_file, is_connected);
        app.config_path = temp_dir.join("companion_config.json");
        app.port = 19999;
        app.current_theme = ThemeMode::Light;
        app.photocraft_path = "D:/apps/photocraft.exe".to_string();

        let res = app.save_settings();
        assert!(res.is_ok());
        assert!(app.config_path.exists());

        let loaded = CompanionConfig::load_or_default(&app.config_path);
        assert_eq!(loaded.port, 19999);
        assert_eq!(loaded.theme, "light");
        assert_eq!(
            loaded.photocraft_path,
            Some(PathBuf::from("D:/apps/photocraft.exe"))
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_plugin_manager_app_status_and_download_events() {
        let temp_dir = std::env::temp_dir().join(format!(
            "mgr_status_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&temp_dir);
        let prefs_file = temp_dir.join("preferences.json");
        let is_connected = Arc::new(AtomicBool::new(false));

        let mut app = PluginManagerApp::new(vec![], &prefs_file, is_connected);
        app.current_language = Language::En;

        // Test set_status
        app.set_status("Operation complete", false);
        assert_eq!(app.status_text, "Operation complete");
        assert!(!app.status_is_error);
        assert!(app.status_timestamp.is_some());

        // Test download events channel
        let (tx, rx) = std::sync::mpsc::channel();
        app.download_event_rx = Some(rx);

        tx.send(ModelDownloadEvent::Progress(ModelDownloadProgress {
            model_name: "test.onnx".to_string(),
            plugin_id: "test_plugin".to_string(),
            downloaded_bytes: 500,
            total_bytes: 1000,
            speed_bytes_sec: 100.0,
        }))
        .unwrap();

        app.poll_download_events();
        assert!(app.active_download.is_some());
        let active = app.active_download.as_ref().unwrap();
        assert_eq!(active.model_name, "test.onnx");
        assert_eq!(active.downloaded_bytes, 500);

        // Finished event
        tx.send(ModelDownloadEvent::Finished {
            model_name: "test.onnx".to_string(),
        })
        .unwrap();

        app.poll_download_events();
        assert!(app.active_download.is_none());
        assert!(app.status_text.contains("test.onnx"));
        assert!(!app.status_is_error);

        // Error event
        tx.send(ModelDownloadEvent::Error {
            model_name: "err.onnx".to_string(),
            error: "Connection timeout".to_string(),
        })
        .unwrap();

        app.poll_download_events();
        assert!(app.active_download.is_none());
        assert!(app.status_text.contains("err.onnx"));
        assert!(app.status_is_error);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_plugin_manager_app_store_events() {
        let temp_dir =
            std::env::temp_dir().join(format!("pc_store_app_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let prefs_path = temp_dir.join("prefs.json");

        let mut app =
            PluginManagerApp::new(Vec::new(), &prefs_path, Arc::new(AtomicBool::new(true)));

        // 1. Registry loaded event
        let sample_reg = crate::store::StoreRegistry {
            generated_at: Some("2026-10-13T12:00:00Z".to_string()),
            total_plugins: 1,
            plugins: vec![crate::store::StorePluginEntry {
                id: "test.plugin".to_string(),
                name: "Test Plugin".to_string(),
                version: "1.0.0".to_string(),
                description: "Test description".to_string(),
                author: "Test author".to_string(),
                plugin_type: "no_code".to_string(),
                download_url: "https://example.com/test.pcpl".to_string(),
                icon_url: None,
                models: Vec::new(),
            }],
        };

        app.store_state
            .event_tx
            .send(crate::store::StoreEvent::RegistryLoaded {
                registry: sample_reg.clone(),
                is_offline: false,
            })
            .unwrap();

        app.poll_store_events();
        assert_eq!(
            app.store_state.status,
            crate::store::StoreLoadStatus::Ready { is_offline: false }
        );
        assert_eq!(app.store_state.registry, Some(sample_reg));

        // 2. Plugin installed event
        let dummy_manifest = PluginManifest {
            id: "test.plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Test".to_string(),
            author: "Tester".to_string(),
            plugin_type: PluginType::NoCode,
            enabled: true,
            icon: None,
            models: Vec::new(),
            commands: Vec::new(),
            post_commands: Vec::new(),
            sub_actions: Vec::new(),
            input: core_contracts::PluginInputSource::ActiveLayer,
            output_mode: core_contracts::PluginOutputMode::InPlace,
            action_id: None,
            entrypoint: None,
            ai: None,
            algorithm: None,
            interactive_ui: None,
            plugin_dir: None,
        };

        app.store_state
            .installing_plugins
            .push("test.plugin".to_string());
        app.store_state
            .event_tx
            .send(crate::store::StoreEvent::PluginInstalled {
                plugin_id: "test.plugin".to_string(),
                manifest: Box::new(dummy_manifest.clone()),
            })
            .unwrap();

        app.poll_store_events();
        assert!(!app
            .store_state
            .installing_plugins
            .contains(&"test.plugin".to_string()));
        assert_eq!(app.manifests.len(), 1);
        assert_eq!(app.manifests[0].id, "test.plugin");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_extract_readme_lines() {
        let sample_md = "# Sample Plugin\n\n[![Badge](url)](link)\n\n**English:**\nEnglish description of the plugin.\n\n### Use Cases:\n- Case 1: Testing something.\n\n---\n\n**Русский:**\nРусское описание плагина.\n\n### Практическое применение:\n- Сценарий 1: Тестирование плагина.\n";

        let en_lines = PluginManagerApp::extract_readme_lines(sample_md, Language::En);
        assert!(en_lines.iter().any(|l| l.contains("# Sample Plugin")));
        assert!(en_lines.iter().any(|l| l.contains("English description")));
        assert!(en_lines
            .iter()
            .any(|l| l.contains("Case 1: Testing something.")));
        assert!(!en_lines.iter().any(|l| l.contains("Русское описание")));

        let ru_lines = PluginManagerApp::extract_readme_lines(sample_md, Language::Ru);
        assert!(ru_lines.iter().any(|l| l.contains("# Sample Plugin")));
        assert!(ru_lines.iter().any(|l| l.contains("Русское описание")));
        assert!(ru_lines.iter().any(|l| l.contains("Сценарий 1")));
        assert!(!ru_lines.iter().any(|l| l.contains("English description")));
    }
}
