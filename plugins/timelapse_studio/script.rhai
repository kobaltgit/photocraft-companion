// Timelapse Studio Reactive Controller
// Manages drawing session recording, snapshot frames, and FFmpeg video export

fn format_frame_path(dir, idx) {
    let safe_dir = if dir == () || dir == "" {
        "PhotoCraftData/sessions/timelapse_current"
    } else {
        dir
    };
    let int_idx = idx.to_int();
    let s = int_idx.to_string();
    while s.len() < 6 {
        s = "0" + s;
    }
    safe_dir + "/frame_" + s + ".png"
}

fn on_revision(revision, state) {
    if state["is_recording"] == true {
        let count = if state["frame_count"] != () { state["frame_count"].to_int() + 1 } else { 1 };
        state["frame_count"] = count;
        let frame_path = format_frame_path(state["session_dir"], count);
        execute_raster_process("capture_canvas_frame", #{
            "path": frame_path
        });
        state = update_localized_labels(state);
    }
    return state;
}

fn on_state_changed(state) {
    if state["initialized"] != true {
        state["initialized"] = true;
        state["is_recording"] = false;
        state["frame_count"] = 0;
        state["session_dir"] = "PhotoCraftData/sessions/timelapse_current";
        state["last_exported_path"] = "";
        state["banner_message"] = t("Studio ready. Click Start Recording to begin.", "Студия готова. Нажмите Начать запись для старта.");
        state["banner_style"] = "info";
        state["session_info"] = t("Status: Idle | Frames: 0", "Статус: Ожидание | Кадров: 0");
        state["stats_info"] = t("Recorded frames: 0 | Video length: ~0.0s", "Записано кадров: 0 | Длина видео: ~0.0с");
        return state;
    }

    if state["action_btn_start_record"] == true {
        state["action_btn_start_record"] = false;
        let count = state["frame_count"].to_int();
        if count == 0 {
            // Fresh recording: clean directory and capture frame 1
            clean_session_dir(state["session_dir"]);
            count = 1;
            state["frame_count"] = count;
            let frame_path = format_frame_path(state["session_dir"], count);
            execute_raster_process("capture_canvas_frame", #{
                "path": frame_path
            });
            state["banner_message"] = t("Recording in progress. Drawing on canvas is being recorded.", "Идет запись. Процесс рисования на холсте сохраняется.");
        } else {
            // Resuming from pause/stop: preserve captured frames
            state["banner_message"] = t("Recording resumed.", "Запись возобновлена.");
        }
        state["is_recording"] = true;
        state["is_open"] = false; // Auto-hide window to free canvas for painting
        state.set_badge(true, "REC", [239, 68, 68, 255]); // Pulsing REC badge on toolstrip button
        state["banner_style"] = "info";

        let fps = 30;
        if state["framerate"] != () {
            let f = state["framerate"].to_int();
            if f > 0 { fps = f; }
        }
        let hold = 2.0;
        if state["hold_duration"] != () {
            hold = state["hold_duration"].to_float();
        }
        let sec = (count * 1.0) / (fps * 1.0) + hold;
        state["session_info"] = t("Status: Recording Active | Frames: ", "Статус: Идет запись | Кадров: ") + count;
        state["stats_info"] = t("Recorded frames: ", "Записано кадров: ") + count + t(" | Estimated length: ~", " | Примерная длина: ~") + sec + t("s", "с");
        return state;
    } else if state["action_btn_pause_record"] == true {
        state["action_btn_pause_record"] = false;
        if state["is_recording"] == true {
            state["is_recording"] = false;
            state.clear_badge();
            state["banner_message"] = t("Recording paused. Click Record to resume.", "Запись приостановлена. Нажмите Запись для продолжения.");
            state["banner_style"] = "warning";
            state["session_info"] = t("Status: Paused | Frames: ", "Статус: На паузе | Кадров: ") + state["frame_count"];
        } else if state["frame_count"].to_int() > 0 {
            state["is_recording"] = true;
            state["is_open"] = false;
            state.set_badge(true, "REC", [239, 68, 68, 255]);
            state["banner_message"] = t("Recording resumed.", "Запись возобновлена.");
            state["banner_style"] = "info";
            state["session_info"] = t("Status: Recording Active | Frames: ", "Статус: Идет запись | Кадров: ") + state["frame_count"];
        }
        return state;
    } else if state["action_btn_stop_record"] == true {
        state["action_btn_stop_record"] = false;
        state["is_recording"] = false;
        state.clear_badge();
        let count = state["frame_count"].to_int();
        state["banner_message"] = t("Recording stopped. Ready for export.", "Запись остановлена. Готово к экспорту.");
        state["banner_style"] = "info";
        state["session_info"] = t("Status: Stopped | Frames: ", "Статус: Остановлено | Кадров: ") + count;
        return state;
    } else if state["action_btn_reset_session"] == true {
        state["action_btn_reset_session"] = false;
        state["is_recording"] = false;
        state.clear_badge();
        clean_session_dir(state["session_dir"]);
        state["frame_count"] = 0;
        state["banner_message"] = t("Session reset. Studio ready for a new recording.", "Сессия сброшена. Студия готова к новой записи.");
        state["banner_style"] = "info";
        state["session_info"] = t("Status: Idle | Frames: 0", "Статус: Ожидание | Кадров: 0");
        state["stats_info"] = t("Recorded frames: 0 | Video length: ~0.0s", "Записано кадров: 0 | Длина видео: ~0.0с");
        return state;
    } else if state["action_btn_capture_frame"] == true {
        state["action_btn_capture_frame"] = false;
        let count = state["frame_count"].to_int() + 1;
        state["frame_count"] = count;

        let frame_path = format_frame_path(state["session_dir"], count);
        execute_raster_process("capture_canvas_frame", #{
            "path": frame_path
        });

        let fps = 30;
        if state["framerate"] != () {
            let f = state["framerate"].to_int();
            if f > 0 { fps = f; }
        }
        let hold = 2.0;
        if state["hold_duration"] != () {
            hold = state["hold_duration"].to_float();
        }
        let sec = (count * 1.0) / (fps * 1.0) + hold;
        state["banner_message"] = t("Captured canvas frame #", "Сохранен кадр холста #") + count;
        state["banner_style"] = "success";
        state["session_info"] = t("Status: Recording Active | Frames: ", "Статус: Идет запись | Кадров: ") + count;
        state["stats_info"] = t("Recorded frames: ", "Записано кадров: ") + count + t(" | Estimated length: ~", " | Примерная длина: ~") + sec + t("s", "с");
        return state;
    } else if state["action_btn_export_video"] == true {
        state["action_btn_export_video"] = false;
        let count = state["frame_count"].to_int();
        if count == 0 {
            state["banner_message"] = t("No frames recorded yet. Draw on canvas first!", "Кадры еще не записаны. Сделайте несколько мазков на холсте!");
            state["banner_style"] = "warning";
            return state;
        }

        if is_tool_installed("ffmpeg") != true {
            state["banner_message"] = t("FFmpeg is not installed! Click 'Download FFmpeg' below.", "FFmpeg не установлен! Нажмите 'Скачать FFmpeg' ниже.");
            state["banner_style"] = "error";
            return state;
        }

        let default_name = "photocraft_timelapse.mp4";
        let out_path = save_file_dialog(
            t("Export Timelapse Video", "Экспорт видео таймлапса"),
            default_name,
            "mp4"
        );

        if out_path.len() > 0 {
            state["banner_message"] = t("Encoding MP4 video with FFmpeg...", "Кодирование MP4 видео через FFmpeg...");
            state["banner_style"] = "info";

            let fps = 30;
            if state["framerate"] != () {
                let f = state["framerate"].to_int();
                if f > 0 { fps = f; }
            }

            let hold = 2.0;
            if state["hold_duration"] != () {
                hold = state["hold_duration"].to_float();
            }

            let preset_idx = 0;
            if state["export_preset"] != () {
                preset_idx = state["export_preset"].to_int();
            }

            let session_dir = state["session_dir"];
            let input_pattern = session_dir + "/frame_%06d.png";

            let args = [
                "-y",
                "-framerate", fps.to_string(),
                "-i", input_pattern
            ];

            if preset_idx == 0 {
                // 9:16 Social Reels / TikTok (Blur-Fill)
                let filter_str = "[0:v]scale=1080:1920:force_original_aspect_ratio=increase,crop=1080:1920,boxblur=20:5[bg];[0:v]scale=1080:1920:force_original_aspect_ratio=decrease[fg];[bg][fg]overlay=(W-w)/2:(H-h)/2";
                if hold > 0.0 {
                    filter_str = filter_str + ",tpad=stop_mode=clone:stop_duration=" + hold.to_string();
                }
                args.push("-filter_complex");
                args.push(filter_str);
            } else if preset_idx == 2 {
                // 16:9 Widescreen (Blur-Fill)
                let filter_str = "[0:v]scale=1920:1080:force_original_aspect_ratio=increase,crop=1920:1080,boxblur=20:5[bg];[0:v]scale=1920:1080:force_original_aspect_ratio=decrease[fg];[bg][fg]overlay=(W-w)/2:(H-h)/2";
                if hold > 0.0 {
                    filter_str = filter_str + ",tpad=stop_mode=clone:stop_duration=" + hold.to_string();
                }
                args.push("-filter_complex");
                args.push(filter_str);
            } else {
                // Original Canvas Ratio (ensure even dimensions for yuv420p)
                let filter_str = "pad=ceil(iw/2)*2:ceil(ih/2)*2";
                if hold > 0.0 {
                    filter_str = filter_str + ",tpad=stop_mode=clone:stop_duration=" + hold.to_string();
                }
                args.push("-vf");
                args.push(filter_str);
            }

            args.push("-c:v");
            args.push("libx264");
            args.push("-pix_fmt");
            args.push("yuv420p");
            args.push(out_path);

            let result = execute_tool("ffmpeg", args);
            if result["success"] == true || result["exit_code"] == 0 {
                state["last_exported_path"] = out_path;
                state["is_recording"] = false;
                state.clear_badge();
                state["banner_message"] = t("Export successful! Video saved.", "Экспорт завершен! Видео успешно создано.");
                state["banner_style"] = "success";

                if state["auto_clean"] == true {
                    clean_session_dir(session_dir);
                    state["frame_count"] = 0;
                    state["session_info"] = t("Status: Idle | Frames: 0", "Статус: Ожидание | Кадров: 0");
                    state["stats_info"] = t("Recorded frames: 0 | Video length: ~0.0s", "Записано кадров: 0 | Длина видео: ~0.0с");
                }
            } else {
                let err_msg = if result["error"] != () && result["error"] != "" {
                    result["error"]
                } else if result["stderr"] != () && result["stderr"] != "" {
                    result["stderr"]
                } else {
                    t("FFmpeg encoding failed.", "Ошибка кодирования FFmpeg.")
                };
                state["banner_message"] = t("Export failed: ", "Ошибка экспорта: ") + err_msg;
                state["banner_style"] = "error";
            }
        }
        return state;
    } else if state["action_btn_reveal_file"] == true {
        state["action_btn_reveal_file"] = false;
        let last_path = state["last_exported_path"];
        if last_path.len() > 0 {
            reveal_in_file_manager(last_path);
        } else {
            state["banner_message"] = t("No exported file to show yet.", "Файл еще не экспортирован.");
            state["banner_style"] = "warning";
        }
        return state;
    } else if state["action_btn_download_ffmpeg"] == true {
        state["action_btn_download_ffmpeg"] = false;
        if is_tool_installed("ffmpeg") == true {
            state["banner_message"] = t("FFmpeg is already installed and ready!", "FFmpeg уже установлен и готов к работе!");
            state["banner_style"] = "success";
            return state;
        }

        state["banner_message"] = t("Downloading FFmpeg, please wait...", "Скачивание FFmpeg, пожалуйста подождите...");
        state["banner_style"] = "info";

        let ps_cmd = "$ErrorActionPreference = 'Stop'; [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; $zip = 'PhotoCraftData/sessions/ffmpeg_setup.zip'; $extractDir = 'PhotoCraftData/sessions/ffmpeg_temp'; $binDir = 'PhotoCraftData/bin'; if (!(Test-Path $binDir)) { New-Item -ItemType Directory -Force -Path $binDir | Out-Null }; if (Test-Path $extractDir) { Remove-Item -Recurse -Force $extractDir }; Invoke-WebRequest -Uri 'https://github.com/GyanD/codexffmpeg/releases/download/7.0/ffmpeg-7.0-essentials_build.zip' -OutFile $zip; Expand-Archive -Path $zip -DestinationPath $extractDir -Force; $exe = Get-ChildItem -Path $extractDir -Filter 'ffmpeg.exe' -Recurse | Select-Object -First 1; if ($exe) { Copy-Item -Path $exe.FullName -Destination (Join-Path $binDir 'ffmpeg.exe') -Force; } else { throw 'ffmpeg.exe not found in archive' }; Remove-Item -Path $zip -Force -ErrorAction SilentlyContinue; Remove-Item -Path $extractDir -Recurse -Force -ErrorAction SilentlyContinue;";

        let res = execute_tool("powershell", [
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            ps_cmd
        ]);

        if res["success"] == true || is_tool_installed("ffmpeg") == true {
            state["banner_message"] = t("FFmpeg installed successfully! Ready to export.", "FFmpeg успешно установлен! Готово к экспорту.");
            state["banner_style"] = "success";
        } else {
            let err = if res["error"] != () && res["error"] != "" {
                res["error"]
            } else if res["stderr"] != () && res["stderr"] != "" {
                res["stderr"]
            } else {
                t("Download failed.", "Ошибка загрузки.")
            };
            state["banner_message"] = t("Failed to setup FFmpeg: ", "Не удалось установить FFmpeg: ") + err;
            state["banner_style"] = "error";
        }
        return state;
    }

    state = update_localized_labels(state);
    return state;
}

fn update_localized_labels(state) {
    let count = if state["frame_count"] != () { state["frame_count"].to_int() } else { 0 };
    let is_rec = if state["is_recording"] != () { state["is_recording"] == true } else { false };

    let fps = 30;
    if state["framerate"] != () {
        let f = state["framerate"].to_int();
        if f > 0 { fps = f; }
    }
    let hold = 2.0;
    if state["hold_duration"] != () {
        hold = state["hold_duration"].to_float();
    }
    let sec = (count * 1.0) / (fps * 1.0) + hold;

    if count == 0 {
        state["session_info"] = t("Status: Idle | Frames: 0", "Статус: Ожидание | Кадров: 0");
        state["stats_info"] = t("Recorded frames: 0 | Video length: ~0.0s", "Записано кадров: 0 | Длина видео: ~0.0с");
        if state["banner_style"] == "info" {
            state["banner_message"] = t("Studio ready. Click Start Recording to begin.", "Студия готова. Нажмите Начать запись для старта.");
        }
    } else if is_rec {
        state["session_info"] = t("Status: Recording Active | Frames: ", "Статус: Идет запись | Кадров: ") + count;
        state["stats_info"] = t("Recorded frames: ", "Записано кадров: ") + count + t(" | Estimated length: ~", " | Примерная длина: ~") + sec + t("s", "с");
    } else {
        state["session_info"] = t("Status: Stopped | Frames: ", "Статус: Остановлено | Кадров: ") + count;
        state["stats_info"] = t("Recorded frames: ", "Записано кадров: ") + count + t(" | Estimated length: ~", " | Примерная длина: ~") + sec + t("s", "с");
    }
    return state;
}
