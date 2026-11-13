use super::super::*;
use eframe::egui::{self, Color32, RichText, Stroke, Ui, Vec2};

impl PluginManagerApp {
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
        let msg_tool_ready = self.t("Tool ready", "Утилита готова");
        let msg_tool_missing = self.t("Tool missing", "Утилита не установлена");
        let msg_view_doc = self.t(
            "View Documentation & Use Cases",
            "Документация и сценарии использования",
        );
        let msg_no_doc = self.t(
            "No documentation found for this plugin.",
            "Документация для этого плагина не найдена.",
        );
        let plugins_dir = self.plugins_dir.clone();
        let cur_lang = self.effective_language();
        let mut download_notice = None;
        let mut open_readme = None;

        for manifest in &mut self.manifests {
            let all_models_ok = is_plugin_models_installed_cached(manifest);

            egui::Frame::new()
                .fill(palette.card_inner)
                .stroke(Stroke::new(1.0, palette.border_stroke))
                .corner_radius(6.0)
                .inner_margin(8.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // 1. Vector SVG Icon (24x24)
                        let is_dark = self.current_theme.is_dark();
                        let local_icon = manifest.resolve_icon_path(Some(&plugins_dir));
                        let cached_icon = self.store_state.get_cached_icon_path(&manifest.id);
                        let icon_path = local_icon.or_else(|| {
                            if cached_icon.exists() {
                                Some(cached_icon)
                            } else {
                                None
                            }
                        });

                        let texture = self.store_icon_cache.get_or_load(
                            ui.ctx(),
                            &manifest.id,
                            icon_path.as_deref(),
                            is_dark,
                        );

                        if let Some(tex) = texture {
                            ui.image((tex.id(), Vec2::new(24.0, 24.0)));
                        } else {
                            // Vector fallback badge if icon not available
                            let badge_text = match manifest.plugin_type {
                                PluginType::NativeAi => "AI",
                                PluginType::NativeAlgorithm => "ALGO",
                                PluginType::NoCode => "FX",
                                PluginType::InteractiveUi => "UI",
                                PluginType::Script => "RH",
                                PluginType::ExternalCli => "CLI",
                            };
                            let (badge_rect, _) =
                                ui.allocate_exact_size(Vec2::new(24.0, 24.0), egui::Sense::hover());
                            ui.painter().rect_filled(badge_rect, 4.0, palette.card_fill);
                            ui.painter().text(
                                badge_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                badge_text,
                                egui::FontId::proportional(9.0),
                                palette.accent_blue,
                            );
                        }

                        ui.add_space(4.0);

                        // 2. Center: Content (Header, Description, Footer)
                        ui.vertical(|ui| {
                            let max_content_width = (ui.available_width() - 40.0).max(100.0);
                            ui.set_max_width(max_content_width);

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
                            RichText::new(manifest.localized_name(cur_lang.code()))
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

                    });

                    ui.add_space(2.0);
                    ui.label(
                        RichText::new(manifest.localized_description(cur_lang.code()))
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

                        // Runtime tools status badge (clean vector circle dot + label)
                        if !manifest.runtime_tools.is_empty() {
                            let all_tools_ok = manifest
                                .runtime_tools
                                .iter()
                                .all(|tool| is_runtime_tool_installed(&tool.name));

                            let (dot_rect, _) =
                                ui.allocate_exact_size(Vec2::new(8.0, 10.0), egui::Sense::hover());
                            let dot_color = if all_tools_ok {
                                Color32::from_rgb(0x28, 0xa7, 0x45)
                            } else {
                                Color32::from_rgb(0xea, 0x86, 0x00)
                            };
                            ui.painter()
                                .circle_filled(dot_rect.center(), 2.8, dot_color);

                            if all_tools_ok {
                                ui.label(
                                    RichText::new(msg_tool_ready)
                                        .size(10.0)
                                        .color(Color32::from_rgb(0x28, 0xa7, 0x45)),
                                );
                            } else {
                                ui.label(
                                    RichText::new(msg_tool_missing)
                                        .size(10.0)
                                        .color(Color32::from_rgb(0xea, 0x86, 0x00)),
                                );
                            }
                        }

                // 3. Right: Enabled checkbox
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut manifest.enabled, "");
                });
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
}
