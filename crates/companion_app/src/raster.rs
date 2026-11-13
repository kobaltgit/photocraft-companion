use overlay_ui::DynamicToolEngine;
use std::sync::Arc;

pub fn parse_hex_rgba(hex: &str) -> std::result::Result<image::Rgba<u8>, ()> {
    let clean = hex.trim().trim_start_matches('#');
    match clean.len() {
        6 => {
            let r = u8::from_str_radix(&clean[0..2], 16).map_err(|_| ())?;
            let g = u8::from_str_radix(&clean[2..4], 16).map_err(|_| ())?;
            let b = u8::from_str_radix(&clean[4..6], 16).map_err(|_| ())?;
            Ok(image::Rgba([r, g, b, 255]))
        }
        8 => {
            let r = u8::from_str_radix(&clean[0..2], 16).map_err(|_| ())?;
            let g = u8::from_str_radix(&clean[2..4], 16).map_err(|_| ())?;
            let b = u8::from_str_radix(&clean[4..6], 16).map_err(|_| ())?;
            let a = u8::from_str_radix(&clean[6..8], 16).map_err(|_| ())?;
            Ok(image::Rgba([r, g, b, a]))
        }
        _ => Err(()),
    }
}

pub async fn execute_raster_process_action(
    client: &photocraft_bridge::PhotoCraftClient,
    params: &serde_json::Value,
    dynamic_engine: &Arc<std::sync::Mutex<DynamicToolEngine>>,
    egui_ctx_holder: &Arc<std::sync::Mutex<Option<overlay_ui::egui::Context>>>,
) -> core_contracts::Result<()> {
    let task = params.get("task").and_then(|v| v.as_str()).unwrap_or("");
    match task {
        "capture_canvas_frame" => {
            let p_val = params.get("params").unwrap_or(params);
            let path_str = p_val.get("path").and_then(|v| v.as_str()).unwrap_or("");
            if !path_str.is_empty() {
                let target_path = std::path::PathBuf::from(path_str);
                match client
                    .call("ui.screenshot", serde_json::json!({ "focus": false }))
                    .await
                {
                    Ok(res) => {
                        let b64_opt = res
                            .get("image_base64")
                            .or_else(|| res.get("data"))
                            .or_else(|| res.get("base64"))
                            .and_then(|v| v.as_str());
                        if let Some(b64) = b64_opt {
                            let clean = if let Some(idx) = b64.find(',') {
                                if b64.starts_with("data:") {
                                    &b64[idx + 1..]
                                } else {
                                    b64
                                }
                            } else {
                                b64
                            };
                            if let Ok(bytes) = core_contracts::decode_base64(clean) {
                                if let Some(parent) = target_path.parent() {
                                    let _ = std::fs::create_dir_all(parent);
                                }
                                if let Err(e) = std::fs::write(&target_path, bytes) {
                                    tracing::error!(
                                        "Failed to save captured frame to {}: {e}",
                                        target_path.display()
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to capture ui.screenshot: {e}");
                    }
                }
            }
            Ok(())
        }
        "refresh_preview" => {
            if let Some(plugin_id) = params.get("plugin_id").and_then(|v| v.as_str()) {
                if let Ok(mut engine) = dynamic_engine.lock() {
                    engine.clear_preview_and_role_mask(plugin_id);
                }
                if let Ok(guard) = egui_ctx_holder.lock() {
                    if let Some(ctx) = guard.as_ref() {
                        ctx.request_repaint();
                    }
                }
            }
            Ok(())
        }
        "lens_blur" => {
            let plugin_id = params
                .get("plugin_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if !plugin_id.is_empty() {
                if let Ok(mut engine) = dynamic_engine.lock() {
                    engine.set_busy(plugin_id, true);
                }
                if let Ok(guard) = egui_ctx_holder.lock() {
                    if let Some(ctx) = guard.as_ref() {
                        ctx.request_repaint();
                    }
                }
            }

            let result = async {
                let doc = client.get_document_info().await.ok();
                let source_name = doc
                    .as_ref()
                    .and_then(|d| d.active_layer_name.clone())
                    .unwrap_or_else(|| "Layer".to_string());
                let lang = core_contracts::Language::Auto.resolve(None);

                let src = client.copy_active_layer_image().await?;
                if src.width() == 0 || src.height() == 0 {
                    return Err(core_contracts::CompanionError::Plugin(
                        "Active layer has empty dimensions".to_string(),
                    ));
                }

                let p_val = params.get("params").unwrap_or(params);
                let mut lens_params = raster_engine::LensBlurParams::default();

                if let Some(b) = p_val.get("fog_enabled").and_then(|v| v.as_bool()) {
                    lens_params.fog_enabled = b;
                }
                if let Some(hex) = p_val.get("fog_color").and_then(|v| v.as_str()) {
                    if let Ok(c) = parse_hex_rgba(hex) {
                        lens_params.fog_color = [c.0[0], c.0[1], c.0[2], c.0[3]];
                    }
                }
                if let Some(n) = p_val.get("fog_density").and_then(|v| v.as_f64()) {
                    lens_params.fog_density = n as f32;
                }
                if let Some(n) = p_val.get("fog_start").and_then(|v| v.as_f64()) {
                    lens_params.fog_start = n as f32;
                }
                if let Some(n) = p_val.get("fog_falloff").and_then(|v| v.as_f64()) {
                    lens_params.fog_falloff = n as f32;
                }
                if let Some(idx) = p_val.get("fog_blend_mode").and_then(|v| v.as_u64()) {
                    lens_params.fog_blend_mode = idx as u8;
                }
                if let Some(b) = p_val.get("bokeh_enabled").and_then(|v| v.as_bool()) {
                    lens_params.bokeh_enabled = b;
                }
                if let Some(n) = p_val.get("focus_distance").and_then(|v| v.as_f64()) {
                    lens_params.focus_distance = n as f32;
                }
                if let Some(n) = p_val.get("dof_width").and_then(|v| v.as_f64()) {
                    lens_params.dof_width = n as f32;
                }
                if let Some(n) = p_val.get("blur_radius").and_then(|v| v.as_f64()) {
                    lens_params.blur_radius = n as f32;
                }
                if let Some(n) = p_val.get("specular_bloom").and_then(|v| v.as_f64()) {
                    lens_params.specular_bloom = n as f32;
                }

                let cached_mask = {
                    if let Ok(engine) = dynamic_engine.lock() {
                        engine.get_role_mask(plugin_id).cloned()
                    } else {
                        None
                    }
                };

                let depth_map = cached_mask.unwrap_or_else(|| {
                    image::GrayImage::from_pixel(src.width(), src.height(), image::Luma([128]))
                });

                let depth_map =
                    if depth_map.width() != src.width() || depth_map.height() != src.height() {
                        image::imageops::resize(
                            &depth_map,
                            src.width(),
                            src.height(),
                            image::imageops::FilterType::Triangle,
                        )
                    } else {
                        depth_map
                    };

                let fog_as_separate = params
                    .get("fog_as_separate_layer")
                    .and_then(|v| v.as_bool())
                    .or_else(|| {
                        params
                            .get("output_mode")
                            .and_then(|v| v.as_str())
                            .map(|m| m == "split" || m == "split_layers")
                    })
                    .unwrap_or(true);

                let bokeh_name = lang.format_bokeh_name(&source_name);

                if fog_as_separate {
                    let (defocused, fog_layer_opt) =
                        raster_engine::render_lens_blur_split(&src, &depth_map, &lens_params);
                    let paste_res = client.paste_image_in_place(&defocused).await?;
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

                    if let Some(new_layer_id) = result_layer_id {
                        let _ = client.rename_layer(new_layer_id, &bokeh_name).await;
                    }

                    if let Some(fog_layer) = fog_layer_opt {
                        tokio::time::sleep(std::time::Duration::from_millis(40)).await;
                        let fog_res = client.paste_image_in_place(&fog_layer).await?;
                        let fallback_fog = client
                            .get_document_info()
                            .await
                            .ok()
                            .and_then(|i| i.active_layer);
                        let fog_layer_id = fog_res
                            .get("layer")
                            .or_else(|| fog_res.get("layerId"))
                            .or_else(|| fog_res.get("id"))
                            .and_then(|l| l.as_u64())
                            .or(fallback_fog);

                        if let Some(fog_id) = fog_layer_id {
                            let fog_name = lang.fog_layer_name();
                            let _ = client.rename_layer(fog_id, fog_name).await;
                        }
                    }
                } else {
                    let merged = raster_engine::render_lens_blur(&src, &depth_map, &lens_params);
                    let paste_res = client.paste_image_in_place(&merged).await?;
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

                    if let Some(new_layer_id) = result_layer_id {
                        let _ = client.rename_layer(new_layer_id, &bokeh_name).await;
                    }
                }

                Ok(())
            }
            .await;

            if !plugin_id.is_empty() {
                if let Ok(mut engine) = dynamic_engine.lock() {
                    engine.set_busy(plugin_id, false);
                }
                if let Ok(guard) = egui_ctx_holder.lock() {
                    if let Some(ctx) = guard.as_ref() {
                        ctx.request_repaint();
                    }
                }
            }

            result
        }
        "extrude_2_5d" => {
            // 1. Fetch active layer image
            let src = client.copy_active_layer_image().await?;
            if src.width() == 0 || src.height() == 0 {
                return Err(core_contracts::CompanionError::Plugin(
                    "Active layer has empty dimensions".to_string(),
                ));
            }

            // 2. Parse extrusion params
            let p_val = params.get("params").unwrap_or(params);
            let mut extrude_params = raster_engine::ExtrudeParams::default();

            if let Some(angle) = p_val.get("angle_deg").and_then(|v| v.as_f64()) {
                extrude_params.angle_deg = angle as f32;
            }
            if let Some(dist) = p_val.get("distance").and_then(|v| v.as_f64()) {
                extrude_params.distance = dist as f32;
            }
            if let Some(front_hex) = p_val.get("front_color").and_then(|v| v.as_str()) {
                if let Ok(c) = parse_hex_rgba(front_hex) {
                    extrude_params.front_color = Some(c);
                }
            }
            if let Some(start_hex) = p_val.get("body_start_color").and_then(|v| v.as_str()) {
                if let Ok(c) = parse_hex_rgba(start_hex) {
                    extrude_params.body_gradient_start = c;
                }
            }
            if let Some(end_hex) = p_val.get("body_end_color").and_then(|v| v.as_str()) {
                if let Ok(c) = parse_hex_rgba(end_hex) {
                    extrude_params.body_gradient_end = c;
                }
            }
            if let Some(bevel) = p_val.get("bevel_radius").and_then(|v| v.as_f64()) {
                extrude_params.bevel_radius = bevel as f32;
            }
            if let Some(highlight) = p_val.get("bevel_highlight").and_then(|v| v.as_f64()) {
                extrude_params.bevel_highlight = highlight as f32;
            }
            if let Some(shadow_blur) = p_val.get("shadow_blur").and_then(|v| v.as_f64()) {
                extrude_params.shadow_blur = shadow_blur as f32;
            }
            if let Some(shadow_opacity) = p_val.get("shadow_opacity").and_then(|v| v.as_f64()) {
                extrude_params.shadow_opacity = shadow_opacity as f32;
            }
            if let Some(shadow_dist) = p_val.get("shadow_distance").and_then(|v| v.as_f64()) {
                extrude_params.shadow_distance = shadow_dist as f32;
            }
            if let Some(light_angle) = p_val
                .get("light_angle_deg")
                .or_else(|| p_val.get("light_angle"))
                .and_then(|v| v.as_f64())
            {
                extrude_params.light_angle_deg = light_angle as f32;
            }
            if let Some(light_int) = p_val.get("light_intensity").and_then(|v| v.as_f64()) {
                extrude_params.light_intensity = light_int as f32;
            }
            if let Some(persp) = p_val.get("perspective").and_then(|v| v.as_f64()) {
                extrude_params.perspective = persp as f32;
            }

            // 3. Composite source layer onto full document canvas or dynamic padding to eliminate clipping
            let doc_info = client.get_document_info().await.ok();
            let render_canvas = if let Some(ref doc) = doc_info {
                if let (true, true, Some(b)) =
                    (doc.width > 0, doc.height > 0, doc.active_layer_bounds)
                {
                    let left = b[0].max(0) as u32;
                    let top = b[1].max(0) as u32;
                    let canvas_w = doc.width.max(left + src.width());
                    let canvas_h = doc.height.max(top + src.height());
                    let mut canvas = image::RgbaImage::new(canvas_w, canvas_h);
                    image::imageops::overlay(&mut canvas, &src, left as i64, top as i64);
                    canvas
                } else {
                    pad_source_for_extrusion(&src, &extrude_params)
                }
            } else {
                pad_source_for_extrusion(&src, &extrude_params)
            };

            let is_split = p_val
                .get("output_mode")
                .and_then(|v| v.as_str())
                .map(|m| m == "split" || m == "split_layers")
                .unwrap_or(false);

            if is_split {
                extrude_params.output_mode = raster_engine::ExtrudeOutputMode::SplitLayers;
                let (front, body, shadow) =
                    raster_engine::render_extrude_split(&render_canvas, &extrude_params);

                // Insert in bottom-to-top Z-order with 40ms micro-pause
                if let Some(sh) = shadow {
                    client.paste_image_in_place(&sh).await?;
                    tokio::time::sleep(std::time::Duration::from_millis(40)).await;
                }
                client.paste_image_in_place(&body).await?;
                tokio::time::sleep(std::time::Duration::from_millis(40)).await;
                client.paste_image_in_place(&front).await?;
            } else {
                let merged = raster_engine::render_extrude_2_5d(&render_canvas, &extrude_params);
                client.paste_image_in_place(&merged).await?;
            }

            Ok(())
        }
        unknown => Err(core_contracts::CompanionError::Plugin(format!(
            "Unsupported raster task: '{unknown}'"
        ))),
    }
}

pub fn pad_source_for_extrusion(
    src: &image::RgbaImage,
    params: &raster_engine::ExtrudeParams,
) -> image::RgbaImage {
    let rad = params.angle_deg.to_radians();
    let total_dist = params.distance + params.shadow_distance;
    let sx = rad.cos() * total_dist;
    let sy = rad.sin() * total_dist;
    let blur_pad = (params.shadow_blur * 2.0 + params.bevel_radius + 20.0).ceil() as u32;
    let pad_left = blur_pad + (if sx < 0.0 { sx.abs().ceil() as u32 } else { 0 });
    let pad_right = blur_pad + (if sx > 0.0 { sx.ceil() as u32 } else { 0 });
    let pad_top = blur_pad + (if sy < 0.0 { sy.abs().ceil() as u32 } else { 0 });
    let pad_bottom = blur_pad + (if sy > 0.0 { sy.ceil() as u32 } else { 0 });

    let total_w = src.width() + pad_left + pad_right;
    let total_h = src.height() + pad_top + pad_bottom;
    let mut padded = image::RgbaImage::new(total_w, total_h);
    image::imageops::overlay(&mut padded, src, pad_left as i64, pad_top as i64);
    padded
}
