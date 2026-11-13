// Flexy Studio Reactive Rhai Controller
// Agnostic orchestration of Flexy AI generative and editing REST endpoints

fn on_revision(revision, state) {
    return state;
}

fn on_selection_state(has_selection, bounds, state) {
    state["has_selection"] = has_selection;
    if has_selection && state["banner_style"] == "warning" {
        state["banner_message"] = "";
    }
    return state;
}

fn on_state_changed(state) {
    if state["initialized"] != true {
        state["initialized"] = true;
        state["banner_message"] = t(
            "Welcome to Flexy Studio. Enter your API key and choose an action.",
            "Студия Flexy AI готова к работе. Введите API-ключ и выберите действие."
        );
        state["banner_style"] = "info";
        state["balance_text"] = t("Balance: Not checked (click Check)", "Баланс: Не проверен (нажмите «Проверить»)");
        state["progress"] = 0.0;
        state["progress_text"] = t("Ready", "Готов к работе");
        return state;
    }

    let plugin_id = "ui.photocraft.plugins.flexy_studio";
    let api_key = if state["api_key"] != () { state["api_key"].to_string() } else { "" };

    // --- Automatic Trigger: Task Completed ---
    if state["task_completed"] == true {
        state["task_completed"] = false;
        state["action_check_balance"] = true;
    }

    // --- Handling HTTP Query Status & Error Messages ---
    if state["query_status"] == "success" {
        state["query_status"] = ();
        if state["raw_balance"] != () {
            let bal = state["raw_balance"];
            state["balance_text"] = t("Balance: " + bal + " credits", "Баланс: " + bal + " кредитов");
            state["banner_message"] = t("Balance verified: " + bal + " credits available", "Баланс проверен: " + bal + " кредитов доступно");
            state["banner_style"] = "success";
        } else {
            state["banner_message"] = t("Query successful", "Запрос выполнен успешно");
            state["banner_style"] = "success";
        }
    } else if state["query_status"] == "error" {
        state["query_status"] = ();
        let is_auth = state["is_auth_error"] == true;
        if is_auth {
            state["banner_message"] = t("Error: Invalid or unauthorized API key (403)", "Ошибка: Неверный или заблокированный API-ключ (403)");
        } else {
            let err = if state["query_error_message"] != () { state["query_error_message"].to_string() } else { "" };
            state["banner_message"] = t("Network error: " + err, "Ошибка сети: " + err);
        }
        state["banner_style"] = "error";
        state["balance_text"] = t("Balance: Check error", "Баланс: Ошибка проверки");
    }

    // --- Ratio Presets Synchronization ---
    if state["last_gen_ratio"] != state["gen_ratio"] {
        state["last_gen_ratio"] = state["gen_ratio"];
        let ratio_idx = if state["gen_ratio"] != () { state["gen_ratio"].to_int() } else { 0 };
        if ratio_idx == 0 {
            state["gen_width"] = 1024.0;
            state["gen_height"] = 1024.0;
        } else if ratio_idx == 1 {
            state["gen_width"] = 1344.0;
            state["gen_height"] = 768.0;
        } else if ratio_idx == 2 {
            state["gen_width"] = 768.0;
            state["gen_height"] = 1344.0;
        } else if ratio_idx == 3 {
            state["gen_width"] = 896.0;
            state["gen_height"] = 1152.0;
        } else if ratio_idx == 4 {
            state["gen_width"] = 1536.0;
            state["gen_height"] = 640.0;
        }
    }

    // --- Action: Check Balance ---
    if state["action_check_balance"] == true {
        state["action_check_balance"] = false;

        if api_key == "" {
            state["banner_message"] = t("Please enter API key first.", "Сначала введите API-ключ.");
            state["banner_style"] = "warning";
            return state;
        }

        state["banner_message"] = t("Checking account balance...", "Проверка баланса аккаунта...");
        state["banner_style"] = "info";

        execute_command("cloud.process", #{
            "plugin_id": plugin_id,
            "task_type": "http_query",
            "url": "https://flexy-ai.com/api/balance",
            "method": "GET",
            "api_key_service": "flexy_ai",
            "api_key_value": api_key,
            "headers": #{
                "X-API-Key": "$api_key"
            },
            "response_bindings": #{
                "raw_balance": "balance"
            }
        });
        return state;
    }

    // Polling configuration template for async tasks
    let polling_cfg = #{
        "task_id_json_path": "task_id",
        "status_url_template": "https://flexy-ai.com/api/status/{task_id}",
        "interval_ms": 1200,
        "progress_json_path": "progress",
        "preview_json_path": "preview",
        "status_json_path": "status",
        "completed_status_value": "completed",
        "result_images_json_path": "images"
    };

    // --- Action: Generation (Txt2Img / ControlNet) ---
    if state["action_btn_generate"] == true {
        state["action_btn_generate"] = false;

        let prompt = if state["gen_prompt"] != () { state["gen_prompt"].to_string() } else { "" };
        if prompt == "" {
            state["banner_message"] = t("Please enter a text prompt for generation.", "Пожалуйста, введите текстовый промпт для генерации.");
            state["banner_style"] = "warning";
            return state;
        }

        let width = if state["gen_width"] != () { state["gen_width"].to_int() } else { 1024 };
        let height = if state["gen_height"] != () { state["gen_height"].to_int() } else { 1024 };

        let model_idx = if state["gen_model"] != () { state["gen_model"].to_int() } else { 0 };
        let model = if model_idx == 1 { "Turbo" } else { "Flexy" };

        let style_idx = if state["gen_style"] != () { state["gen_style"].to_int() } else { 0 };
        let style = "";
        if style_idx == 1 { style = "Flexy Cinematic"; }
        else if style_idx == 2 { style = "Flexy Professional Photo"; }
        else if style_idx == 3 { style = "Flexy Dramatic Photo"; }
        else if style_idx == 4 { style = "Flexy Vintage Photo"; }
        else if style_idx == 5 { style = "Flexy Fujifilm"; }
        else if style_idx == 6 { style = "Flexy Neo Retro"; }
        else if style_idx == 7 { style = "Flexy Gloomy"; }
        else if style_idx == 8 { style = "Flexy Art (Painting)"; }
        else if style_idx == 9 { style = "Flexy Art (Watercolor)"; }
        else if style_idx == 10 { style = "Flexy 3D Movie"; }
        else if style_idx == 11 { style = "Flexy Line Art"; }
        else if style_idx == 12 { style = "Flexy Anime"; }
        else if style_idx == 13 { style = "Flexy Comic book"; }
        else if style_idx == 14 { style = "Flexy Logo"; }
        else if style_idx == 15 { style = "Flexy Sticker"; }

        let tiling = if state["gen_tiling"] == true { true } else { false };

        let cn_idx = if state["controlnet_mode"] != () { state["controlnet_mode"].to_int() } else { 0 };
        let controlnet_mode = "none";
        if cn_idx == 1 { controlnet_mode = "lineart"; }
        else if cn_idx == 2 { controlnet_mode = "canny"; }
        else if cn_idx == 3 { controlnet_mode = "depth"; }
        else if cn_idx == 4 { controlnet_mode = "scribble"; }
        else if cn_idx == 5 { controlnet_mode = "pose"; }

        let body = #{
            "prompt": prompt,
            "width": width,
            "height": height,
            "model": model,
            "tiling": tiling,
            "sfw": false
        };

        if style != "" {
            body["style"] = style;
        }

        let input_mode = "none";
        let input_field = "image";

        if controlnet_mode != "none" {
            input_mode = "active_layer";
            input_field = "controlnet." + controlnet_mode;
        }

        state["banner_message"] = t("Starting generation via Flexy AI...", "Запуск генерации через Flexy AI...");
        state["banner_style"] = "info";

        execute_command("cloud.process", #{
            "plugin_id": plugin_id,
            "task_type": "async_generation",
            "url": "https://flexy-ai.com/api/txt2img",
            "method": "POST",
            "api_key_service": "flexy_ai",
            "api_key_value": api_key,
            "headers": #{
                "X-API-Key": "$api_key",
                "Content-Type": "application/json"
            },
            "body": body,
            "input_image_mode": input_mode,
            "input_image_field": input_field,
            "polling_config": polling_cfg,
            "output_config": #{
                "layer_name_template": "Flexy Generation",
                "paste_mode": "center"
            }
        });
        return state;
    }

    // --- Action: Inpaint & Eraser ---
    if state["action_btn_inpaint"] == true {
        state["action_btn_inpaint"] = false;

        let task_idx = if state["inpaint_task"] != () { state["inpaint_task"].to_int() } else { 0 };

        if (task_idx == 0 || task_idx == 1) && state["has_selection"] != true {
            state["banner_message"] = t(
                "Please create a selection on the canvas for eraser/inpaint.",
                "Пожалуйста, выделите область на холсте для ластика или заливки."
            );
            state["banner_style"] = "warning";
            return state;
        }

        let url = "https://flexy-ai.com/api/removeobject";
        let layer_name = "Flexy Smart Eraser";
        let body = #{};

        if task_idx == 1 {
            url = "https://flexy-ai.com/api/inpaint";
            layer_name = "Flexy Inpaint";
            let prompt = if state["inpaint_prompt"] != () { state["inpaint_prompt"].to_string() } else { "" };
            let denoise = if state["inpaint_denoise"] != () { state["inpaint_denoise"].to_float() } else { 0.75 };
            let ctx_mode = if state["inpaint_context"] == true { true } else { false };
            body["prompt"] = prompt;
            body["denoise"] = denoise;
            body["context_mode"] = ctx_mode;
        } else if task_idx == 2 {
            url = "https://flexy-ai.com/api/watermarkremover";
            layer_name = "Flexy Watermark Remover";
        }

        let input_mask_mode = if task_idx == 0 || task_idx == 1 { "selection" } else { "none" };
        let require_sel = if task_idx == 0 || task_idx == 1 { true } else { false };
        let in_place = if state["inpaint_in_place"] != () { state["inpaint_in_place"] } else { true };
        let output_layer_name = if in_place { layer_name } else { layer_name + " Patch" };

        state["banner_message"] = t("Processing inpaint/eraser on active layer...", "Обработка ластиком/заливкой активного слоя...");
        state["banner_style"] = "info";

        execute_command("cloud.process", #{
            "plugin_id": plugin_id,
            "task_type": "async_generation",
            "url": url,
            "method": "POST",
            "api_key_service": "flexy_ai",
            "api_key_value": api_key,
            "headers": #{
                "X-API-Key": "$api_key",
                "Content-Type": "application/json"
            },
            "body": body,
            "input_image_mode": "active_layer",
            "input_image_field": "image",
            "input_mask_mode": input_mask_mode,
            "input_mask_field": "mask",
            "context_crop": true,
            "require_selection": require_sel,
            "polling_config": polling_cfg,
            "output_config": #{
                "layer_name_template": output_layer_name,
                "merge_down": in_place
            }
        });
        return state;
    }

    // --- Action: Portrait & Face ---
    if state["action_btn_portrait"] == true {
        state["action_btn_portrait"] = false;

        let task_idx = if state["portrait_task"] != () { state["portrait_task"].to_int() } else { 0 };
        let url = "https://flexy-ai.com/api/retouch";
        let layer_name = "Flexy Retouch";
        let body = #{};
        let input_field = "images";

        let strength_slider = if state["retouch_strength"] != () { (state["retouch_strength"].to_float() * 100.0).to_int() } else { 80 };

        if task_idx == 1 {
            url = "https://flexy-ai.com/api/retouch";
            layer_name = "Flexy Studio Retouch Pro";
            body["strength"] = strength_slider.max(88);
        } else if task_idx == 2 {
            url = "https://flexy-ai.com/api/oldifyphoto";
            layer_name = "Flexy Oldify Photo";
            input_field = "image";
        } else {
            body["strength"] = strength_slider.min(75);
        }

        state["banner_message"] = t("Processing portrait...", "Обработка портрета...");
        state["banner_style"] = "info";

        execute_command("cloud.process", #{
            "plugin_id": plugin_id,
            "task_type": "async_generation",
            "url": url,
            "method": "POST",
            "api_key_service": "flexy_ai",
            "api_key_value": api_key,
            "headers": #{
                "X-API-Key": "$api_key",
                "Content-Type": "application/json"
            },
            "body": body,
            "input_image_mode": "active_layer",
            "input_image_field": input_field,
            "polling_config": polling_cfg,
            "output_config": #{
                "layer_name_template": layer_name
            }
        });
        return state;
    }

    // --- Action: Canvas & Background ---
    if state["action_btn_canvas"] == true {
        state["action_btn_canvas"] = false;

        let task_idx = if state["canvas_task"] != () { state["canvas_task"].to_int() } else { 0 };
        let url = "https://flexy-ai.com/api/outpaint";
        let layer_name = "Flexy Outpaint";
        let body = #{};

        if task_idx == 1 {
            url = "https://flexy-ai.com/api/bgchanger";
            layer_name = "Flexy Background Changer";
            let prompt = if state["outpaint_prompt"] != () { state["outpaint_prompt"].to_string() } else { "" };
            body["prompt"] = prompt;
            body["mode"] = "person";
        } else if task_idx == 2 {
            url = "https://flexy-ai.com/api/restoration";
            layer_name = "Flexy Colorize";
            body["colorization"] = true;
            body["global_restoration"] = false;
        } else {
            let prompt = if state["outpaint_prompt"] != () { state["outpaint_prompt"].to_string() } else { "" };
            body["prompt"] = prompt;
            body["top"] = 128;
            body["bottom"] = 128;
            body["left"] = 128;
            body["right"] = 128;
        }

        state["banner_message"] = t("Executing canvas operation...", "Выполнение операции с холстом...");
        state["banner_style"] = "info";

        execute_command("cloud.process", #{
            "plugin_id": plugin_id,
            "task_type": "async_generation",
            "url": url,
            "method": "POST",
            "api_key_service": "flexy_ai",
            "api_key_value": api_key,
            "headers": #{
                "X-API-Key": "$api_key",
                "Content-Type": "application/json"
            },
            "body": body,
            "input_image_mode": "active_layer",
            "input_image_field": "image",
            "polling_config": polling_cfg,
            "output_config": #{
                "layer_name_template": layer_name
            }
        });
        return state;
    }

    return state;
}
