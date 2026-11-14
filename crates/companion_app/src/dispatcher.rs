use crate::cloud_engine;
use crate::raster::execute_raster_process_action;
use crate::workflow_engine;
use core_contracts::{ExecutionState, PluginContext, PluginType, SharedExecutionStatus};
use overlay_ui::DynamicToolEngine;
use photocraft_bridge::PhotoCraftClient;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tracing::info;

#[allow(clippy::too_many_arguments)]
pub fn spawn_action_dispatcher(
    rt: &tokio::runtime::Runtime,
    mut action_rx: tokio::sync::mpsc::UnboundedReceiver<String>,
    client_exec: Arc<PhotoCraftClient>,
    registry_exec: Arc<tokio::sync::Mutex<core_contracts::PluginRegistry>>,
    data_dir_exec: PathBuf,
    plugins_dir_exec: PathBuf,
    config_path_exec: PathBuf,
    prefs_path_exec: PathBuf,
    exec_status_worker: SharedExecutionStatus,
) {
    rt.spawn(async move {
        while let Some(raw_action) = action_rx.recv().await {
            let req = core_contracts::ActionRequest::from(raw_action);
            info!(
                "Dispatching action request '{}' (action: {}, sub: {:?})...",
                req, req.action_id, req.sub_action_id
            );

            let action_name = {
                let reg = registry_exec.lock().await;
                reg.get(&req.action_id)
                    .map(|p| p.manifest().name.clone())
                    .unwrap_or_else(|| req.action_id.clone())
            };

            let t_start = Instant::now();
            *exec_status_worker.lock().unwrap() = ExecutionState::Running {
                action_id: req.to_string(),
                action_name: action_name.clone(),
                started_at: t_start,
            };

            if let Some(cmd) = req.action_id.strip_prefix("engine:") {
                info!("Executing PhotoCraft engine command '{}'...", cmd);
                match client_exec
                    .execute_command(cmd, serde_json::json!({}))
                    .await
                {
                    Ok(res) => {
                        let elapsed = t_start.elapsed().as_secs_f32();
                        info!("Engine command '{}' executed successfully: {:?}", cmd, res);
                        *exec_status_worker.lock().unwrap() = ExecutionState::Success {
                            action_name,
                            message: format!("Command '{cmd}' completed"),
                            duration_secs: elapsed,
                            finished_at: Instant::now(),
                        };
                    }
                    Err(e) => {
                        tracing::error!("Engine command '{}' failed: {}", cmd, e);
                        *exec_status_worker.lock().unwrap() = ExecutionState::Error {
                            action_name,
                            error: e.to_string(),
                            finished_at: Instant::now(),
                        };
                    }
                }
            } else if let Ok(doc) = client_exec.get_document_info().await {
                let reg = registry_exec.lock().await;
                if let Some(plugin) = reg.get(&req.action_id) {
                    let manifest = plugin.manifest().clone();
                    let plugin_dir = manifest
                        .plugin_dir
                        .clone()
                        .unwrap_or_else(|| plugins_dir_exec.join(&manifest.id));

                    let current_host_lang = core_contracts::Language::Auto
                        .resolve(Some(&prefs_path_exec))
                        .code()
                        .to_string();

                    let mut exec_ctx = workflow_engine::ExecutionContext {
                        source_layer_id: doc.active_layer,
                        result_layer_id: None,
                        active_layer_id: doc.active_layer,
                        document_id: None,
                        document_width: doc.width,
                        document_height: doc.height,
                        has_selection: doc.has_selection,
                        fg_color: None,
                        bg_color: None,
                        custom_vars: std::collections::HashMap::new(),
                        host_language: Some(current_host_lang),
                    };

                    // 1. Pre-commands (only for non-NoCode plugins, as NoCode commands ARE the plugin body)
                    if manifest.plugin_type != PluginType::NoCode && !manifest.commands.is_empty() {
                        if let Err(e) = workflow_engine::execute_steps(
                            &manifest.commands,
                            &mut exec_ctx,
                            client_exec.as_ref(),
                        )
                        .await
                        {
                            *exec_status_worker.lock().unwrap() = ExecutionState::Error {
                                action_name,
                                error: format!("Pre-commands failed: {e}"),
                                finished_at: Instant::now(),
                            };
                            continue;
                        }
                    }

                    // 2. Main plugin execution
                    let plugin_ctx = PluginContext {
                        document: doc,
                        models_dir: data_dir_exec.join("models"),
                        plugin_dir,
                        client: client_exec.clone(),
                        sub_action_id: req.sub_action_id.clone(),
                    };

                    match plugin.execute(&plugin_ctx).await {
                        Ok(res) => {
                            if let Some(ref data) = res.data {
                                if let Some(src) =
                                    data.get("source_layer_id").and_then(|v| v.as_u64())
                                {
                                    exec_ctx.source_layer_id = Some(src);
                                }
                                if let Some(out_id) =
                                    data.get("result_layer_id").and_then(|v| v.as_u64())
                                {
                                    exec_ctx.result_layer_id = Some(out_id);
                                }
                            }

                            // 3. Post-commands (sub_action variant or manifest fallback)
                            let chosen_sub = req.sub_action_id.as_ref().and_then(|sub_id| {
                                manifest.sub_actions.iter().find(|s| &s.id == sub_id)
                            });
                            let post_steps = if let Some(sub) = chosen_sub {
                                &sub.post_commands
                            } else {
                                &manifest.post_commands
                            };

                            if !post_steps.is_empty() {
                                if let Err(e) = workflow_engine::execute_steps(
                                    post_steps,
                                    &mut exec_ctx,
                                    client_exec.as_ref(),
                                )
                                .await
                                {
                                    *exec_status_worker.lock().unwrap() = ExecutionState::Error {
                                        action_name,
                                        error: format!("Post-commands failed: {e}"),
                                        finished_at: Instant::now(),
                                    };
                                    continue;
                                }
                            }

                            // 4. Persist active sub-action to config
                            if let Some(ref sub_id) = req.sub_action_id {
                                let mut cfg = core_contracts::CompanionConfig::load_or_default(
                                    &config_path_exec,
                                );
                                cfg.active_sub_actions
                                    .insert(manifest.id.clone(), sub_id.clone());
                                let _ = cfg.save(&config_path_exec);
                            }

                            let elapsed = t_start.elapsed().as_secs_f32();
                            info!("Action '{}' finished successfully: {}", req, res.message);
                            *exec_status_worker.lock().unwrap() = ExecutionState::Success {
                                action_name,
                                message: res.message,
                                duration_secs: elapsed,
                                finished_at: Instant::now(),
                            };
                        }
                        Err(e) => {
                            tracing::error!("Action '{}' failed: {}", req, e);
                            *exec_status_worker.lock().unwrap() = ExecutionState::Error {
                                action_name,
                                error: e.to_string(),
                                finished_at: Instant::now(),
                            };
                        }
                    }
                } else {
                    tracing::warn!("Action '{}' not found in registry", req.action_id);
                    *exec_status_worker.lock().unwrap() = ExecutionState::Error {
                        action_name,
                        error: format!("Action '{}' not found in registry", req.action_id),
                        finished_at: Instant::now(),
                    };
                }
            } else {
                let conn_st = client_exec.verify_connection().await;
                let err_msg = match conn_st {
                    core_contracts::ConnectionState::AuthRequired => {
                        "PhotoCraft authentication required. Please set valid token in Settings."
                            .to_string()
                    }
                    core_contracts::ConnectionState::Offline => {
                        "PhotoCraft is offline or unreachable on port 18492.".to_string()
                    }
                    core_contracts::ConnectionState::Connected => {
                        "No active document open in PhotoCraft.".to_string()
                    }
                };
                tracing::warn!("Failed to execute action '{}': {}", req, err_msg);
                *exec_status_worker.lock().unwrap() = ExecutionState::Error {
                    action_name,
                    error: err_msg,
                    finished_at: Instant::now(),
                };
            }
        }
    });
}

pub fn spawn_ipc_action_dispatcher(
    rt_handle: &tokio::runtime::Handle,
    mut ipc_action_rx: tokio::sync::mpsc::UnboundedReceiver<core_contracts::PluginIpcAction>,
    client_ipc: Arc<PhotoCraftClient>,
    dynamic_engine_ipc: Arc<std::sync::Mutex<DynamicToolEngine>>,
    egui_ctx_holder_ipc: Arc<std::sync::Mutex<Option<overlay_ui::egui::Context>>>,
    config_path_ipc: PathBuf,
) {
    rt_handle.spawn(async move {
        while let Some(action) = ipc_action_rx.recv().await {
            info!(
                "Executing IPC action '{}' with params: {}",
                action.command, action.params
            );
            if action.command == "tools.setColors" {
                let fg = action.params.get("foreground").and_then(|v| v.as_str());
                let bg = action.params.get("background").and_then(|v| v.as_str());
                if let Err(e) = client_ipc.set_colors(fg, bg).await {
                    tracing::warn!("Failed to set colors via IPC: {e}");
                }
            } else if action.command == "raster.process" {
                if let Err(e) = execute_raster_process_action(
                    &client_ipc,
                    &action.params,
                    &dynamic_engine_ipc,
                    &egui_ctx_holder_ipc,
                )
                .await
                {
                    tracing::error!("Failed to execute raster.process: {e}");
                }
            } else if action.command == "cloud.process" {
                let client_cloud = client_ipc.clone();
                let dynamic_engine_cloud = dynamic_engine_ipc.clone();
                let egui_ctx_holder_cloud = egui_ctx_holder_ipc.clone();
                let action_params = action.params.clone();
                let config_path_cloud = config_path_ipc.clone();
                tokio::spawn(async move {
                    if let Err(e) = cloud_engine::execute_cloud_process_action(
                        &client_cloud,
                        &action_params,
                        &dynamic_engine_cloud,
                        &egui_ctx_holder_cloud,
                        &config_path_cloud,
                    )
                    .await
                    {
                        tracing::error!("Failed to execute cloud.process: {e}");
                    }
                });
            } else {
                let first_try = client_ipc
                    .execute_command(&action.command, action.params.clone())
                    .await;
                if let Err(e) = first_try {
                    let host_lang = core_contracts::Language::Auto.resolve(None).code();
                    let (translated, changed) =
                        workflow_engine::runner::translate_value_to_host(&action.params, host_lang);
                    if changed {
                        if let Err(retry_err) = client_ipc
                            .execute_command(&action.command, translated)
                            .await
                        {
                            tracing::warn!(
                                "Failed to execute IPC action '{}' (retry failed: {retry_err}, original: {e})",
                                action.command
                            );
                        }
                    } else {
                        tracing::warn!("Failed to execute IPC action '{}': {e}", action.command);
                    }
                }
            }
        }
    });
}
