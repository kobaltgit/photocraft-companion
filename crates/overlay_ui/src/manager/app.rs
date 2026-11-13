use super::*;
use eframe::egui::{self, Color32, RichText, Stroke, Ui, Vec2};

impl PluginManagerApp {
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
        if !self.icons_calibrated
            && crate::win32_window::apply_crisp_window_icons_by_title(
                "PhotoCraft - Plugins Manager",
            )
        {
            self.icons_calibrated = true;
        }
        let palette = ThemePalette::for_mode(self.current_theme);
        ui.ctx().set_visuals(palette.to_egui_visuals());

        let mut is_open = true;
        self.render_settings_window(ui, &palette, &mut is_open);
    }
}

pub(crate) fn detect_default_token(
    photocraft_path: &str,
    prefs_path: &std::path::Path,
) -> Option<String> {
    // 1. Configured photocraft_path (HIGHEST priority: if user explicitly pointed to a PhotoCraft instance,
    // use its token to resolve multi-version and global env var conflicts from launchers like ArtCraft Launcher)
    if !photocraft_path.trim().is_empty() {
        let p = std::path::Path::new(photocraft_path.trim());
        let candidate_dir = if p.is_dir() {
            p.to_path_buf()
        } else {
            p.parent().unwrap_or(p).to_path_buf()
        };
        let paths = [
            candidate_dir.join("PhotoCraftData").join("control.token"),
            candidate_dir.join("control.token"),
        ];
        for path in &paths {
            if let Ok(s) = std::fs::read_to_string(path) {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }

    // 2. Prefs path parent
    if let Some(parent) = prefs_path.parent() {
        let paths = [
            parent.join("control.token"),
            parent.join("PhotoCraftData").join("control.token"),
        ];
        for path in &paths {
            if let Ok(s) = std::fs::read_to_string(path) {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }

    // 3. Environment variables (fallback if not resolved from configured photocraft_path or prefs)
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

    // 4. User data directories (guaranteed writable across macOS / Windows / Linux)
    let user_data = core_contracts::user_data_dir();
    candidates.push(user_data.join("PhotoCraftData").join("control.token"));
    candidates.push(user_data.join("control.token"));

    // 5. macOS / Linux standard user config
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

    // 6. Windows AppData / LocalAppData / USERPROFILE
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

static RUNTIME_TOOL_CACHE: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<String, (bool, std::time::Instant)>>,
> = std::sync::OnceLock::new();

pub fn clear_runtime_tool_cache() {
    if let Some(mutex) = RUNTIME_TOOL_CACHE.get() {
        if let Ok(mut map) = mutex.lock() {
            map.clear();
        }
    }
}

pub fn is_runtime_tool_installed(tool_name: &str) -> bool {
    let mutex =
        RUNTIME_TOOL_CACHE.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    let now = std::time::Instant::now();
    if let Ok(guard) = mutex.lock() {
        if let Some((installed, timestamp)) = guard.get(tool_name) {
            if now.duration_since(*timestamp) < std::time::Duration::from_secs(10) {
                return *installed;
            }
        }
    }

    let installed = is_runtime_tool_installed_uncached(tool_name);
    if let Ok(mut guard) = mutex.lock() {
        guard.insert(tool_name.to_string(), (installed, now));
    }
    installed
}

fn is_runtime_tool_installed_uncached(tool_name: &str) -> bool {
    let check_candidate = |p: PathBuf| -> bool {
        if p.is_file() {
            return true;
        }
        #[cfg(target_os = "windows")]
        {
            if !tool_name.ends_with(".exe") && p.with_extension("exe").is_file() {
                return true;
            }
        }
        false
    };

    // 1. Check local & user PhotoCraftData/bin/
    let local_bin = PathBuf::from("PhotoCraftData").join("bin").join(tool_name);
    if check_candidate(local_bin) {
        return true;
    }
    let user_bin = core_contracts::user_data_dir()
        .join("PhotoCraftData")
        .join("bin")
        .join(tool_name);
    if check_candidate(user_bin) {
        return true;
    }
    let direct_bin = PathBuf::from("bin").join(tool_name);
    if check_candidate(direct_bin) {
        return true;
    }

    // 2. Check system PATH
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join(tool_name);
            if check_candidate(candidate) {
                return true;
            }
        }
    }

    false
}

static MODELS_INSTALLED_CACHE: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<String, (bool, std::time::Instant)>>,
> = std::sync::OnceLock::new();

pub fn is_plugin_models_installed_cached(manifest: &PluginManifest) -> bool {
    let mutex = MODELS_INSTALLED_CACHE
        .get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()));
    let now = std::time::Instant::now();
    if let Ok(guard) = mutex.lock() {
        if let Some((installed, timestamp)) = guard.get(&manifest.id) {
            if now.duration_since(*timestamp) < std::time::Duration::from_secs(5) {
                return *installed;
            }
        }
    }

    let installed = manifest.all_models_installed();
    if let Ok(mut guard) = mutex.lock() {
        guard.insert(manifest.id.clone(), (installed, now));
    }
    installed
}

pub fn invalidate_models_installed_cache(plugin_id: Option<&str>) {
    if let Some(mutex) = MODELS_INSTALLED_CACHE.get() {
        if let Ok(mut guard) = mutex.lock() {
            if let Some(id) = plugin_id {
                guard.remove(id);
            } else {
                guard.clear();
            }
        }
    }
}
