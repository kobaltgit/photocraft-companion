use super::super::*;
use eframe::egui::{self, Color32, RichText, Stroke, Ui, Vec2};

impl PluginManagerApp {
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
}
