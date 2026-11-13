mod app;
mod modals;
mod tabs;
mod types;

#[cfg(test)]
mod tests;

pub(crate) use app::detect_default_token;
pub use app::{
    clear_runtime_tool_cache, invalidate_models_installed_cache, is_plugin_models_installed_cached,
    is_runtime_tool_installed,
};
pub use types::*;

use crate::theme::{ThemeMode, ThemePalette};
use core_contracts::{CompanionConfig, ConnectionState, Language, PluginManifest, PluginType};
use eframe::egui::{self, Color32, RichText, Ui, Vec2};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::info;

pub struct PluginManagerApp {
    pub active_tab: ActiveTab,
    pub manifests: Vec<PluginManifest>,
    pub current_language: Language,
    pub resolved_auto_language: Language,
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
    pub icons_calibrated: bool,
    pub is_process_running: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
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
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let lang = Language::from_str_code(&cfg.language);
        let resolved = if let Some(ref p) = cfg.photocraft_path {
            let exe_prefs = p
                .parent()
                .map(|dir| dir.join("PhotoCraftData").join("preferences.json"));
            if let Some(ref ep) = exe_prefs {
                if ep.exists() {
                    Language::Auto.resolve(Some(ep))
                } else {
                    Language::Auto.resolve(Some(&prefs_path))
                }
            } else {
                Language::Auto.resolve(Some(&prefs_path))
            }
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

        let candidate = if PathBuf::from("plugins").is_dir() {
            PathBuf::from("plugins")
        } else {
            data_dir.parent().unwrap_or(&data_dir).join("plugins")
        };
        let plugins_dir = if core_contracts::is_dir_writable(&candidate) {
            candidate
        } else {
            core_contracts::user_data_dir().join("plugins")
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
            resolved_auto_language: resolved,
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
            icons_calibrated: false,
            is_process_running: None,
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

    pub fn with_process_detector(mut self, callback: Arc<dyn Fn() -> bool + Send + Sync>) -> Self {
        self.is_process_running = Some(callback);
        self
    }

    pub fn with_plugins_dir(mut self, plugins_dir: impl Into<PathBuf>) -> Self {
        self.plugins_dir = plugins_dir.into();
        self
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
                    invalidate_models_installed_cache(None);
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
        cfg.language = self.current_language.code().to_string();
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
}
