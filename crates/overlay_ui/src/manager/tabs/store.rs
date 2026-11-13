use super::super::*;
use eframe::egui::{self, Color32, RichText, Stroke, Ui, Vec2};

impl PluginManagerApp {
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
                            let action_btn_area_width = 110.0;
                            let max_content_width =
                                (ui.available_width() - action_btn_area_width).max(100.0);
                            ui.set_max_width(max_content_width);
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
}
