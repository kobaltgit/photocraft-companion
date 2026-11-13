use anyhow::{anyhow, Result};
use core_contracts::CompanionConfig;
use overlay_ui::dynamic_tool::state::DynamicValue;
use overlay_ui::dynamic_tool::DynamicToolEngine;
use photocraft_bridge::PhotoCraftClient;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{error, info, warn};

/// Decodes base64 string (with or without data URI prefix) into RgbaImage
fn base64_to_rgba_image(b64: &str) -> Result<image::RgbaImage> {
    use base64::Engine;
    let raw = if let Some(idx) = b64.find(',') {
        &b64[idx + 1..]
    } else {
        b64
    };
    let clean = raw.trim();
    let decoded = base64::engine::general_purpose::STANDARD.decode(clean)?;
    let dynamic_img = image::load_from_memory(&decoded)?;
    Ok(dynamic_img.to_rgba8())
}

/// Encodes an RgbaImage into a standard base64 PNG string
fn rgba_to_base64_png(img: &image::RgbaImage) -> Result<String> {
    use base64::Engine;
    let mut bytes = std::io::Cursor::new(Vec::new());
    img.write_to(&mut bytes, image::ImageFormat::Png)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes.into_inner()))
}

/// Encodes a GrayImage into a standard base64 PNG string
fn gray_to_base64_png(img: &image::GrayImage) -> Result<String> {
    use base64::Engine;
    let mut bytes = std::io::Cursor::new(Vec::new());
    img.write_to(&mut bytes, image::ImageFormat::Png)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes.into_inner()))
}

/// Helper to set nested JSON fields (e.g. "controlnet.lineart")
fn set_nested_json_value(target: &mut Value, path: &str, value: Value) {
    if !path.contains('.') {
        target[path] = value;
        return;
    }

    let parts: Vec<&str> = path.split('.').collect();
    let mut curr = target;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            curr[*part] = value;
            return;
        }
        if !curr[*part].is_object() {
            curr[*part] = json!({});
        }
        curr = &mut curr[*part];
    }
}

/// Helper to extract value from nested JSON via dot path
fn get_nested_json_value<'a>(source: &'a Value, path: &str) -> Option<&'a Value> {
    if !path.contains('.') {
        return source.get(path);
    }
    let parts: Vec<&str> = path.split('.').collect();
    let mut curr = source;
    for part in parts {
        curr = curr.get(part)?;
    }
    Some(curr)
}

/// Finds the bounding box (min_x, min_y, max_x, max_y) of non-zero pixels in a mask
pub(crate) fn compute_mask_bbox(mask: &image::GrayImage) -> Option<(u32, u32, u32, u32)> {
    let (w, h) = (mask.width(), mask.height());
    let mut min_x = w;
    let mut min_y = h;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut has_pixels = false;

    for y in 0..h {
        for x in 0..w {
            if mask.get_pixel(x, y)[0] > 0 {
                has_pixels = true;
                if x < min_x {
                    min_x = x;
                }
                if x > max_x {
                    max_x = x;
                }
                if y < min_y {
                    min_y = y;
                }
                if y > max_y {
                    max_y = y;
                }
            }
        }
    }

    if has_pixels {
        Some((min_x, min_y, max_x, max_y))
    } else {
        None
    }
}

/// Expands a bounding box by padding_pct (e.g. 0.35) while clamping within canvas bounds
pub(crate) fn calculate_context_crop_bounds(
    (min_x, min_y, max_x, max_y): (u32, u32, u32, u32),
    orig_w: u32,
    orig_h: u32,
    padding_pct: f32,
) -> (u32, u32, u32, u32) {
    let bbox_w = (max_x - min_x + 1) as f32;
    let bbox_h = (max_y - min_y + 1) as f32;

    let pad_x = (bbox_w * padding_pct).round() as u32;
    let pad_y = (bbox_h * padding_pct).round() as u32;

    let crop_x = min_x.saturating_sub(pad_x);
    let crop_y = min_y.saturating_sub(pad_y);
    let crop_x2 = (max_x + 1 + pad_x).min(orig_w);
    let crop_y2 = (max_y + 1 + pad_y).min(orig_h);

    let crop_w = crop_x2.saturating_sub(crop_x);
    let crop_h = crop_y2.saturating_sub(crop_y);

    (crop_x, crop_y, crop_w, crop_h)
}

/// Composes a full-canvas transparent layer with generated patch pixels placed strictly by mask
pub(crate) fn compose_patch_layer(
    orig_w: u32,
    orig_h: u32,
    crop_x: u32,
    crop_y: u32,
    crop_mask: &image::GrayImage,
    inpaint_patch: &image::RgbaImage,
) -> image::RgbaImage {
    let mut patch_layer = image::RgbaImage::new(orig_w, orig_h);
    let patch_w = inpaint_patch.width().min(crop_mask.width());
    let patch_h = inpaint_patch.height().min(crop_mask.height());

    for py in 0..patch_h {
        for px in 0..patch_w {
            let m_val = crop_mask.get_pixel(px, py)[0];
            if m_val > 0 {
                let canvas_x = crop_x + px;
                let canvas_y = crop_y + py;
                if canvas_x < orig_w && canvas_y < orig_h {
                    let mut p = *inpaint_patch.get_pixel(px, py);
                    if m_val < 255 {
                        let a = (p[3] as u32 * m_val as u32 / 255) as u8;
                        p[3] = a;
                    }
                    patch_layer.put_pixel(canvas_x, canvas_y, p);
                }
            }
        }
    }

    patch_layer
}

/// Internal metadata tracking context crop for accurate pixel-to-pixel compositing
struct ContextCropMeta {
    crop_x: u32,
    crop_y: u32,
    crop_w: u32,
    crop_h: u32,
    orig_w: u32,
    orig_h: u32,
    crop_mask: image::GrayImage,
}

/// 100% agnostic cloud task executor for external REST APIs
pub async fn execute_cloud_process_action(
    client: &PhotoCraftClient,
    params: &Value,
    dynamic_engine: &Arc<Mutex<DynamicToolEngine>>,
    egui_ctx_holder: &Arc<Mutex<Option<overlay_ui::egui::Context>>>,
    config_path: &Path,
) -> Result<()> {
    let plugin_id = params
        .get("plugin_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let task_type = params
        .get("task_type")
        .and_then(|v| v.as_str())
        .unwrap_or("async_generation");

    let url = params
        .get("url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'url' parameter in cloud.process"))?
        .to_string();

    let method = params
        .get("method")
        .and_then(|v| v.as_str())
        .unwrap_or("POST")
        .to_uppercase();

    // 1. Resolve and persist API keys if provided
    let mut config = CompanionConfig::load_or_default(config_path);
    let mut resolved_api_key = String::new();

    if let Some(service) = params.get("api_key_service").and_then(|v| v.as_str()) {
        if let Some(key_val) = params.get("api_key_value").and_then(|v| v.as_str()) {
            let trimmed = key_val.trim();
            if !trimmed.is_empty() {
                config
                    .api_keys
                    .insert(service.to_string(), trimmed.to_string());
                let _ = config.save(config_path);
                resolved_api_key = trimmed.to_string();
            }
        }
        if resolved_api_key.is_empty() {
            if let Some(k) = config.api_keys.get(service) {
                resolved_api_key = k.clone();
            }
        }
    }

    // 2. Prepare headers map
    let mut headers: HashMap<String, String> = HashMap::new();
    if let Some(h_obj) = params.get("headers").and_then(|v| v.as_object()) {
        for (k, v) in h_obj {
            if let Some(v_str) = v.as_str() {
                let resolved_val = if v_str == "$api_key" {
                    resolved_api_key.clone()
                } else if v_str.starts_with("$api_key:") {
                    let svc = &v_str[9..];
                    config.api_keys.get(svc).cloned().unwrap_or_default()
                } else {
                    v_str.to_string()
                };
                headers.insert(k.clone(), resolved_val);
            }
        }
    }

    // 3. Prepare body and incorporate input images and masks if requested
    let mut body = params.get("body").cloned().unwrap_or(json!({}));
    let input_mode = params
        .get("input_image_mode")
        .and_then(|v| v.as_str())
        .unwrap_or("none");
    let input_mask_mode = params
        .get("input_mask_mode")
        .and_then(|v| v.as_str())
        .unwrap_or("none");
    let input_mask_field = params
        .get("input_mask_field")
        .and_then(|v| v.as_str())
        .unwrap_or("mask");
    let require_selection = params
        .get("require_selection")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let input_image_field = params
        .get("input_image_field")
        .and_then(|v| v.as_str())
        .unwrap_or("image");

    let context_crop = params
        .get("context_crop")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let mut crop_meta: Option<ContextCropMeta> = None;

    if input_mask_mode == "selection" {
        info!("Fetching active layer and selection mask from PhotoCraft for cloud payload...");
        let (full_img, mask_opt) = client.copy_active_layer_and_selection_mask().await?;

        if let Some(mask) = mask_opt {
            let (img_to_send, mask_to_send) = if context_crop {
                if let Some(bbox) = compute_mask_bbox(&mask) {
                    let orig_w = full_img.width();
                    let orig_h = full_img.height();
                    let (cx, cy, cw, ch) =
                        calculate_context_crop_bounds(bbox, orig_w, orig_h, 0.35);
                    if cw > 0 && ch > 0 {
                        let cropped_img =
                            image::imageops::crop_imm(&full_img, cx, cy, cw, ch).to_image();
                        let cropped_mask =
                            image::imageops::crop_imm(&mask, cx, cy, cw, ch).to_image();

                        let (final_crop_img, final_crop_mask) = if cw > 1536 || ch > 1536 {
                            info!(
                                "Crop size {cw}x{ch} exceeds 1536 px; applying Catmull-Rom downscaling before cloud dispatch"
                            );
                            let scale = 1536.0 / (cw.max(ch) as f32);
                            let target_w = ((cw as f32 * scale).round() as u32).max(1);
                            let target_h = ((ch as f32 * scale).round() as u32).max(1);
                            let scaled_img = image::imageops::resize(
                                &cropped_img,
                                target_w,
                                target_h,
                                image::imageops::FilterType::CatmullRom,
                            );
                            let scaled_mask = image::imageops::resize(
                                &cropped_mask,
                                target_w,
                                target_h,
                                image::imageops::FilterType::CatmullRom,
                            );
                            (scaled_img, scaled_mask)
                        } else {
                            (cropped_img, cropped_mask.clone())
                        };

                        crop_meta = Some(ContextCropMeta {
                            crop_x: cx,
                            crop_y: cy,
                            crop_w: cw,
                            crop_h: ch,
                            orig_w,
                            orig_h,
                            crop_mask: cropped_mask,
                        });
                        (final_crop_img, final_crop_mask)
                    } else {
                        (full_img, mask)
                    }
                } else {
                    (full_img, mask)
                }
            } else {
                (full_img, mask)
            };

            let img_b64 = rgba_to_base64_png(&img_to_send)?;
            let mask_b64 = gray_to_base64_png(&mask_to_send)?;

            if input_image_field == "images" {
                set_nested_json_value(&mut body, input_image_field, json!([img_b64]));
            } else {
                set_nested_json_value(&mut body, input_image_field, json!(img_b64));
            }
            set_nested_json_value(&mut body, input_mask_field, json!(mask_b64));
        } else if require_selection {
            return Err(anyhow!("Active selection is required for this operation"));
        } else {
            let img_b64 = rgba_to_base64_png(&full_img)?;
            if input_image_field == "images" {
                set_nested_json_value(&mut body, input_image_field, json!([img_b64]));
            } else {
                set_nested_json_value(&mut body, input_image_field, json!(img_b64));
            }
        }
    } else if input_mode == "active_layer" {
        info!("Fetching active layer image from PhotoCraft for cloud payload...");
        match client.copy_active_layer_image().await {
            Ok(img) => {
                let b64 = rgba_to_base64_png(&img)?;
                if input_image_field == "images" {
                    set_nested_json_value(&mut body, input_image_field, json!([b64]));
                } else {
                    set_nested_json_value(&mut body, input_image_field, json!(b64));
                }
            }
            Err(e) => {
                warn!("Failed to fetch active layer image: {e}");
            }
        }
    }

    // 4. Handle HTTP Query (one-shot, non-polling request, e.g. /api/balance)
    if task_type == "http_query" {
        let headers_clone = headers.clone();
        let url_clone = url.clone();
        let method_clone = method.clone();
        let body_clone = body.clone();

        let query_result = tokio::task::spawn_blocking(move || -> Result<Value, String> {
            let res = if method_clone == "GET" {
                let mut req = ureq::get(&url_clone);
                for (k, v) in &headers_clone {
                    req = req.header(k, v);
                }
                req.call()
            } else {
                let mut req = ureq::post(&url_clone);
                for (k, v) in &headers_clone {
                    req = req.header(k, v);
                }
                req.send_json(body_clone)
            };

            match res {
                Ok(resp) => {
                    let json_resp: Value =
                        resp.into_body().read_json().map_err(|e| e.to_string())?;
                    Ok(json_resp)
                }
                Err(e) => Err(e.to_string()),
            }
        })
        .await?;

        match query_result {
            Ok(resp_json) => {
                if let Some(bindings) = params.get("response_bindings").and_then(|v| v.as_object())
                {
                    if let Ok(mut engine) = dynamic_engine.lock() {
                        for (state_key, json_path) in bindings {
                            if let Some(path_str) = json_path.as_str() {
                                if let Some(val) = get_nested_json_value(&resp_json, path_str) {
                                    let dyn_val = if let Some(n) = val.as_f64() {
                                        DynamicValue::Number(n)
                                    } else if let Some(b) = val.as_bool() {
                                        DynamicValue::Bool(b)
                                    } else if let Some(s) = val.as_str() {
                                        DynamicValue::String(s.to_string())
                                    } else {
                                        DynamicValue::String(val.to_string())
                                    };
                                    engine.set_widget_value(&plugin_id, state_key, dyn_val);
                                }
                            }
                        }
                        engine.set_widget_value(
                            &plugin_id,
                            "query_status",
                            DynamicValue::String("success".to_string()),
                        );
                        engine.execute_script_state_change(&plugin_id);
                    }
                } else if let Ok(mut engine) = dynamic_engine.lock() {
                    engine.set_widget_value(
                        &plugin_id,
                        "query_status",
                        DynamicValue::String("success".to_string()),
                    );
                    engine.execute_script_state_change(&plugin_id);
                }
            }
            Err(err_msg) => {
                error!("HTTP query failed: {err_msg}");
                let is_auth_error = err_msg.contains("401")
                    || err_msg.contains("403")
                    || err_msg.to_lowercase().contains("api-key is required");

                if let Ok(mut engine) = dynamic_engine.lock() {
                    engine.set_widget_value(
                        &plugin_id,
                        "query_status",
                        DynamicValue::String("error".to_string()),
                    );
                    engine.set_widget_value(
                        &plugin_id,
                        "query_error_message",
                        DynamicValue::String(err_msg),
                    );
                    engine.set_widget_value(
                        &plugin_id,
                        "is_auth_error",
                        DynamicValue::Bool(is_auth_error),
                    );
                    engine.execute_script_state_change(&plugin_id);
                }
            }
        }
        if let Ok(guard) = egui_ctx_holder.lock() {
            if let Some(ctx) = guard.as_ref() {
                ctx.request_repaint();
            }
        }

        return Ok(());
    }

    // 5. Handle Async Generation with Polling
    if !plugin_id.is_empty() {
        let start_msg = "Starting...";
        if let Ok(mut engine) = dynamic_engine.lock() {
            engine.set_busy(&plugin_id, true);
            engine.set_widget_value(&plugin_id, "progress", DynamicValue::Number(0.0));
            engine.set_widget_value(
                &plugin_id,
                "progress_text",
                DynamicValue::String(start_msg.to_string()),
            );
        }
        if let Ok(guard) = egui_ctx_holder.lock() {
            if let Some(ctx) = guard.as_ref() {
                ctx.request_repaint();
            }
        }
    }

    let polling_cfg = params.get("polling_config").cloned().unwrap_or(json!({}));
    let task_id_path = polling_cfg
        .get("task_id_json_path")
        .and_then(|v| v.as_str())
        .unwrap_or("task_id");
    let status_template = polling_cfg
        .get("status_url_template")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'status_url_template' in polling_config"))?
        .to_string();
    let interval_ms = polling_cfg
        .get("interval_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(1000);
    let progress_path = polling_cfg
        .get("progress_json_path")
        .and_then(|v| v.as_str())
        .unwrap_or("progress");
    let preview_path = polling_cfg
        .get("preview_json_path")
        .and_then(|v| v.as_str())
        .unwrap_or("preview");
    let status_path = polling_cfg
        .get("status_json_path")
        .and_then(|v| v.as_str())
        .unwrap_or("status");
    let completed_val = polling_cfg
        .get("completed_status_value")
        .and_then(|v| v.as_str())
        .unwrap_or("completed");
    let images_path = polling_cfg
        .get("result_images_json_path")
        .and_then(|v| v.as_str())
        .unwrap_or("images");

    // Initiate task
    let headers_clone = headers.clone();
    let url_clone = url.clone();
    let body_clone = body.clone();

    let init_resp: Value = match tokio::task::spawn_blocking(move || -> Result<Value> {
        let mut req = ureq::post(&url_clone);
        for (k, v) in &headers_clone {
            req = req.header(k, v);
        }
        let resp = req.send_json(body_clone)?;
        let json_resp: Value = resp.into_body().read_json()?;
        Ok(json_resp)
    })
    .await?
    {
        Ok(v) => v,
        Err(e) => {
            error!("Cloud initiation request failed: {e}");
            let err_formatted = format!("Error: {e}");
            if !plugin_id.is_empty() {
                if let Ok(mut engine) = dynamic_engine.lock() {
                    engine.set_busy(&plugin_id, false);
                    engine.set_widget_value(
                        &plugin_id,
                        "progress_text",
                        DynamicValue::String(err_formatted),
                    );
                }
            }
            return Err(e);
        }
    };

    let task_id = get_nested_json_value(&init_resp, task_id_path)
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Failed to extract task_id from cloud initiation response"))?
        .to_string();

    info!("Cloud task initiated successfully. Task ID: {task_id}");

    let max_polls = 180;
    let mut poll_count = 0;
    let mut final_images: Vec<String> = Vec::new();

    while poll_count < max_polls {
        tokio::time::sleep(Duration::from_millis(interval_ms)).await;
        poll_count += 1;

        let poll_url = status_template.replace("{task_id}", &task_id);
        let headers_loop = headers.clone();

        let status_resp: Value = match tokio::task::spawn_blocking(move || -> Result<Value> {
            let mut req = ureq::get(&poll_url);
            for (k, v) in &headers_loop {
                req = req.header(k, v);
            }
            let resp = req.call()?;
            let json_resp: Value = resp.into_body().read_json()?;
            Ok(json_resp)
        })
        .await?
        {
            Ok(v) => v,
            Err(e) => {
                warn!("Polling status attempt {poll_count} failed: {e}");
                continue;
            }
        };

        let current_status = get_nested_json_value(&status_resp, status_path)
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if let Some(prog_val) = get_nested_json_value(&status_resp, progress_path) {
            let prog_num = prog_val.as_f64().unwrap_or(0.0);
            if let Ok(mut engine) = dynamic_engine.lock() {
                engine.set_widget_value(&plugin_id, "progress", DynamicValue::Number(prog_num));
                engine.set_widget_value(
                    &plugin_id,
                    "progress_text",
                    DynamicValue::String(format!("{current_status} ({:.0}%)", prog_num)),
                );
            }
        }

        // Live preview if available
        if let Some(prev_val) = get_nested_json_value(&status_resp, preview_path) {
            if let Some(prev_str) = prev_val.as_str() {
                if !prev_str.is_empty() {
                    if let Ok(prev_img) = base64_to_rgba_image(prev_str) {
                        if let Ok(mut engine) = dynamic_engine.lock() {
                            engine.set_preview_image(&plugin_id, prev_img, 1.0);
                        }
                    }
                }
            }
        }

        if let Ok(guard) = egui_ctx_holder.lock() {
            if let Some(ctx) = guard.as_ref() {
                ctx.request_repaint();
            }
        }

        if current_status == completed_val {
            if let Some(img_val) = get_nested_json_value(&status_resp, images_path) {
                if let Some(arr) = img_val.as_array() {
                    for it in arr {
                        if let Some(s) = it.as_str() {
                            final_images.push(s.to_string());
                        }
                    }
                } else if let Some(s) = img_val.as_str() {
                    final_images.push(s.to_string());
                }
            }
            break;
        } else if current_status == "error" || current_status == "failed" {
            let err_msg = status_resp
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("Task failed on remote server");
            let err_formatted = format!("Error: {err_msg}");
            if let Ok(mut engine) = dynamic_engine.lock() {
                engine.set_busy(&plugin_id, false);
                engine.set_widget_value(
                    &plugin_id,
                    "progress_text",
                    DynamicValue::String(err_formatted),
                );
            }
            return Err(anyhow!("{err_msg}"));
        }
    }

    if final_images.is_empty() {
        let timeout_msg = "Timeout or no images returned";
        if let Ok(mut engine) = dynamic_engine.lock() {
            engine.set_busy(&plugin_id, false);
            engine.set_widget_value(
                &plugin_id,
                "progress_text",
                DynamicValue::String(timeout_msg.to_string()),
            );
        }
        return Err(anyhow!("No images received from cloud task"));
    }

    // 6. Paste resulting images into PhotoCraft active canvas
    let default_layer_name = "Cloud Generation";

    let layer_name = params
        .get("output_config")
        .and_then(|o| o.get("layer_name_template"))
        .and_then(|v| v.as_str())
        .unwrap_or(default_layer_name);

    let paste_mode = params
        .get("output_config")
        .and_then(|o| o.get("paste_mode"))
        .and_then(|v| v.as_str())
        .unwrap_or("in_place");

    let merge_down = params
        .get("output_config")
        .and_then(|o| o.get("merge_down"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    for (idx, b64_img) in final_images.iter().enumerate() {
        let mut rgba = base64_to_rgba_image(b64_img)?;

        let img_to_paste = if let Some(meta) = &crop_meta {
            if rgba.width() != meta.crop_w || rgba.height() != meta.crop_h {
                rgba = image::imageops::resize(
                    &rgba,
                    meta.crop_w,
                    meta.crop_h,
                    image::imageops::FilterType::CatmullRom,
                );
            }
            compose_patch_layer(
                meta.orig_w,
                meta.orig_h,
                meta.crop_x,
                meta.crop_y,
                &meta.crop_mask,
                &rgba,
            )
        } else {
            rgba
        };

        if crop_meta.is_some() {
            let _ = client.deselect().await;
        }

        let paste_res = if paste_mode == "center" && crop_meta.is_none() {
            client.paste_image_centered(&img_to_paste).await?
        } else {
            client.paste_image_in_place(&img_to_paste).await?
        };

        let fallback_id = client
            .get_document_info()
            .await
            .ok()
            .and_then(|i| i.active_layer);
        let result_layer_id = paste_res
            .get("layer")
            .or_else(|| paste_res.get("layerId"))
            .or_else(|| paste_res.get("id"))
            .and_then(|l| l.as_u64())
            .or(fallback_id);

        if merge_down {
            info!("Merging generated patch layer into active layer (In-Place)...");
            let _ = client.merge_down().await;
        } else if let Some(lid) = result_layer_id {
            let name = if final_images.len() > 1 {
                format!("{layer_name} {}", idx + 1)
            } else {
                layer_name.to_string()
            };
            let _ = client.rename_layer(lid, &name).await;
        }
    }

    let done_msg = "Completed!";

    if let Ok(mut engine) = dynamic_engine.lock() {
        engine.set_busy(&plugin_id, false);
        engine.set_widget_value(&plugin_id, "progress", DynamicValue::Number(100.0));
        engine.set_widget_value(
            &plugin_id,
            "progress_text",
            DynamicValue::String(done_msg.to_string()),
        );
        engine.set_widget_value(&plugin_id, "task_completed", DynamicValue::Bool(true));
        engine.execute_script_state_change(&plugin_id);
    }
    if let Ok(guard) = egui_ctx_holder.lock() {
        if let Some(ctx) = guard.as_ref() {
            ctx.request_repaint();
        }
    }

    info!("Cloud process completed successfully and pasted into PhotoCraft.");
    Ok(())
}
