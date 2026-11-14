use ai_engine::AiEngine;
use core_contracts::PluginManifest;
use overlay_ui::DynamicToolEngine;
use photocraft_bridge::PhotoCraftClient;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub fn spawn_session_sync_worker(
    rt_handle: &tokio::runtime::Handle,
    client: Arc<PhotoCraftClient>,
    dynamic_engine: Arc<std::sync::Mutex<DynamicToolEngine>>,
    is_connected: Arc<AtomicBool>,
    egui_ctx_holder: Arc<std::sync::Mutex<Option<overlay_ui::egui::Context>>>,
    ai: Arc<AiEngine>,
    manifests: &[PluginManifest],
) {
    let client_colors = client;
    let dynamic_engine_colors = dynamic_engine;
    let is_conn_colors = is_connected;
    let ctx_holder_colors = egui_ctx_holder;
    let ai_colors = ai;
    let manifests_map: Arc<HashMap<String, PluginManifest>> = Arc::new(
        manifests
            .iter()
            .map(|m| (m.id.clone(), m.clone()))
            .collect(),
    );

    rt_handle.spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(400));
        let mut tick_counter: u32 = 0;
        loop {
            interval.tick().await;
            if !is_conn_colors.load(Ordering::Relaxed) {
                continue;
            }
            tick_counter = tick_counter.wrapping_add(1);

            let has_active_sub = {
                if let Ok(engine) = dynamic_engine_colors.lock() {
                    engine.has_open_windows_with_subscription(
                        core_contracts::IPC_SUB_DOCUMENT_REVISION,
                    ) || engine
                        .has_open_windows_with_subscription(core_contracts::IPC_SUB_SELECTION_STATE)
                        || engine.has_open_windows_with_subscription(
                            core_contracts::IPC_SUB_ACTIVE_LAYER_IMAGE,
                        )
                } else {
                    false
                }
            };

            // Only poll get_document_info frequently (400ms) if an open window subscribes to canvas changes.
            // When idle or just floating, poll at calm 2000ms pace (every 5 ticks) to eliminate PhotoCraft GPU frame stalls!
            #[allow(clippy::manual_is_multiple_of)]
            let should_poll_doc = has_active_sub || (tick_counter % 5 == 0);

            if should_poll_doc {
                if let Ok(doc) = client_colors.get_document_info().await {
                    let updated_sel = {
                        if let Ok(mut engine) = dynamic_engine_colors.lock() {
                            let prev = engine.has_selection;
                            engine.has_selection = doc.has_selection;
                            let dispatched = engine
                                .dispatch_selection_state(doc.has_selection, doc.selection_bounds);
                            let rev_dispatched = if doc.revision > 0 {
                                engine.dispatch_document_revision(doc.revision)
                            } else {
                                false
                            };
                            let has_open_rev_windows = engine.has_open_windows_with_subscription(
                                core_contracts::IPC_SUB_DOCUMENT_REVISION,
                            );
                            let needs_rev_repaint = rev_dispatched && has_open_rev_windows;
                            dispatched || needs_rev_repaint || (prev != doc.has_selection)
                        } else {
                            false
                        }
                    };
                    if updated_sel {
                        if let Ok(guard) = ctx_holder_colors.lock() {
                            if let Some(ctx) = guard.as_ref() {
                                ctx.request_repaint();
                            }
                        }
                    }
                }
            }

            let needs_sync = {
                if let Ok(engine) = dynamic_engine_colors.lock() {
                    engine
                        .has_open_windows_with_subscription(core_contracts::IPC_SUB_SESSION_COLORS)
                } else {
                    false
                }
            };

            if needs_sync {
                if let Ok((fg, bg)) = client_colors.get_session_colors().await {
                    let updated = {
                        if let Ok(mut engine) = dynamic_engine_colors.lock() {
                            engine.dispatch_session_colors(&fg, &bg)
                        } else {
                            false
                        }
                    };
                    if updated {
                        if let Ok(guard) = ctx_holder_colors.lock() {
                            if let Some(ctx) = guard.as_ref() {
                                ctx.request_repaint();
                            }
                        }
                    }
                }
            }

            let subscribed_image_plugin = {
                if let Ok(engine) = dynamic_engine_colors.lock() {
                    engine.first_open_window_with_subscription(
                        core_contracts::IPC_SUB_ACTIVE_LAYER_IMAGE,
                    )
                } else {
                    None
                }
            };

            if let Some(plugin_id) = subscribed_image_plugin {
                let needs_thumb = {
                    if let Ok(engine) = dynamic_engine_colors.lock() {
                        engine.get_preview_image(&plugin_id).is_none()
                    } else {
                        false
                    }
                };

                if needs_thumb {
                    if let Ok(img) = client_colors.copy_active_layer_image().await {
                        let orig_w = img.width();
                        let orig_h = img.height();
                        if orig_w > 0 && orig_h > 0 {
                            let scale =
                                raster_engine::calculate_preview_scale(orig_w, orig_h, 1280);
                            let target_w = ((orig_w as f32 * scale).round() as u32).max(1);
                            let target_h = ((orig_h as f32 * scale).round() as u32).max(1);
                            let thumbnail = image::imageops::resize(
                                &img,
                                target_w,
                                target_h,
                                image::imageops::FilterType::Triangle,
                            );
                            if let Ok(mut engine) = dynamic_engine_colors.lock() {
                                engine.set_preview_image(&plugin_id, thumbnail, scale);
                            }
                            if let Ok(guard) = ctx_holder_colors.lock() {
                                if let Some(ctx) = guard.as_ref() {
                                    ctx.request_repaint();
                                }
                            }

                            // If plugin declares auto_run_model_role, run model inference asynchronously
                            if let Some(m) = manifests_map.get(&plugin_id) {
                                if let Some(ref ui_cfg) = m.interactive_ui {
                                    if let Some(ref role) = ui_cfg.auto_run_model_role {
                                        if let Some(model_req) = m.model_by_role(role) {
                                            if let Some(model_path) = ai_colors.resolve_model_path(
                                                &model_req.name,
                                                m.plugin_dir.as_deref(),
                                            ) {
                                                if let Some(ref ai_cfg) = m.ai {
                                                    if let Ok(mut engine) =
                                                        dynamic_engine_colors.lock()
                                                    {
                                                        engine.set_busy(&plugin_id, true);
                                                    }
                                                    if let Ok(guard) = ctx_holder_colors.lock() {
                                                        if let Some(ctx) = guard.as_ref() {
                                                            ctx.request_repaint();
                                                        }
                                                    }

                                                    let ai_worker = ai_colors.clone();
                                                    let ai_cfg_worker = ai_cfg.clone();
                                                    let dyn_engine_worker =
                                                        dynamic_engine_colors.clone();
                                                    let ctx_holder_worker =
                                                        ctx_holder_colors.clone();
                                                    let pid_worker = plugin_id.clone();
                                                    let model_path_worker = model_path.clone();
                                                    let img_worker = img.clone();

                                                    tokio::task::spawn_blocking(move || {
                                                        if let Ok(mask) = ai_worker
                                                            .execute_segmentation(
                                                                &model_path_worker,
                                                                &img_worker,
                                                                &ai_cfg_worker,
                                                            )
                                                        {
                                                            if let Ok(mut engine) =
                                                                dyn_engine_worker.lock()
                                                            {
                                                                engine.set_role_mask(
                                                                    &pid_worker,
                                                                    mask,
                                                                );
                                                                engine.set_busy(&pid_worker, false);
                                                            }
                                                            if let Ok(guard) =
                                                                ctx_holder_worker.lock()
                                                            {
                                                                if let Some(ctx) = guard.as_ref() {
                                                                    ctx.request_repaint();
                                                                }
                                                            }
                                                            return;
                                                        }
                                                        if let Ok(mut engine) =
                                                            dyn_engine_worker.lock()
                                                        {
                                                            engine.set_busy(&pid_worker, false);
                                                        }
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}
