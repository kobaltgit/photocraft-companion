use super::*;
use core_contracts::{decode_base64, ConnectionState};
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

#[test]
fn test_default_token_env_resolution() {
    // 1. Test direct token env var
    std::env::remove_var("PHOTOCRAFT_CONTROL_TOKEN_FILE");
    std::env::set_var("PHOTOCRAFT_CONTROL_TOKEN", "mock_token_12345");
    assert_eq!(
        PhotoCraftClient::default_token(),
        Some("mock_token_12345".to_string())
    );
    std::env::remove_var("PHOTOCRAFT_CONTROL_TOKEN");

    // 2. Test token file env var
    let temp_dir = std::env::temp_dir().join(format!(
        "token_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let token_path = temp_dir.join("control.token");
    std::fs::write(&token_path, "  file_token_abc  \n").unwrap();

    std::env::set_var(
        "PHOTOCRAFT_CONTROL_TOKEN_FILE",
        token_path.to_str().unwrap(),
    );
    assert_eq!(
        PhotoCraftClient::default_token(),
        Some("file_token_abc".to_string())
    );

    // 3. Clean up
    std::env::remove_var("PHOTOCRAFT_CONTROL_TOKEN_FILE");
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_mock_server_connect_auth_and_commands() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    // Spawn mock PhotoCraft server
    let server_handle = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let (read_half, mut write_half) = stream.into_split();
                let mut reader = BufReader::new(read_half);
                let mut line = String::new();

                // Read first line (auth or probe)
                if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                    return; // Connection closed (e.g. is_alive probe)
                }

                if let Ok(auth_req) = serde_json::from_str::<Value>(&line) {
                    if auth_req.get("method").and_then(|m| m.as_str()) == Some("auth") {
                        let auth_resp = json!({
                            "id": auth_req["id"],
                            "ok": true,
                            "result": null
                        });
                        let _ = write_half
                            .write_all(format!("{}\n", auth_resp).as_bytes())
                            .await;
                        let _ = write_half.flush().await;
                    }
                }

                // Loop to handle commands
                loop {
                    line.clear();
                    if reader.read_line(&mut line).await.unwrap_or(0) == 0 {
                        break;
                    }
                    if let Ok(req) = serde_json::from_str::<Value>(&line) {
                        let method = req.get("method").and_then(|m| m.as_str()).unwrap_or("");
                        let resp = match method {
                            "engine.execute" => {
                                let cmd = req["params"]["command"].as_str().unwrap_or("");
                                match cmd {
                                    "document.inspect" => json!({
                                        "id": req["id"],
                                        "ok": true,
                                        "result": {
                                            "name": "Design.psd",
                                            "width": 1920,
                                            "height": 1080,
                                            "activeLayer": 42,
                                            "layers": [
                                                { "id": 42, "name": "Subject" },
                                                { "id": 1, "name": "Background" }
                                            ]
                                        }
                                    }),
                                    "layer.renameLayer" => json!({
                                        "id": req["id"],
                                        "ok": true,
                                        "result": { "renamed": true }
                                    }),
                                    "edit.paste" => json!({
                                        "id": req["id"],
                                        "ok": true,
                                        "result": { "layer": 99 }
                                    }),
                                    "session.inspect" => json!({
                                        "id": req["id"],
                                        "ok": true,
                                        "result": {
                                            "foreground": [0.13, 0.77, 0.36, 1.0],
                                            "background": [1.0, 1.0, 1.0, 1.0]
                                        }
                                    }),
                                    _ => json!({ "id": req["id"], "ok": true, "result": null }),
                                }
                            }
                            "ui.inspect" => json!({
                                "id": req["id"],
                                "ok": true,
                                "result": { "window": "main", "active": true }
                            }),
                            "ui.screenshot" => json!({
                                "id": req["id"],
                                "ok": true,
                                "result": { "base64": "dummy_img" }
                            }),
                            _ => {
                                json!({ "id": req["id"], "ok": false, "error": "Unknown method" })
                            }
                        };

                        let _ = write_half.write_all(format!("{}\n", resp).as_bytes()).await;
                        let _ = write_half.flush().await;
                    }
                }
            });
        }
    });

    let client = PhotoCraftClient::new(port, Some("secret_token_1".to_string()));
    assert!(client.is_alive().await);

    client.connect().await.unwrap();

    // Test get_document_info
    let doc = client.get_document_info().await.unwrap();
    assert_eq!(doc.name, "Design.psd");
    assert_eq!(doc.width, 1920);
    assert_eq!(doc.height, 1080);
    assert_eq!(doc.active_layer, Some(42));
    assert_eq!(doc.active_layer_name, Some("Subject".to_string()));

    // Test rename_layer
    let rename_res = client.rename_layer(42, "Subject (No BG)").await.unwrap();
    assert_eq!(rename_res["renamed"], true);

    // Test paste
    let paste_res = client.paste().await.unwrap();
    assert_eq!(paste_res["layer"], 99);

    // Test ui_inspect
    let ui_info = client.ui_inspect().await.unwrap();
    assert_eq!(ui_info["window"], "main");

    // Test ui_screenshot
    let ss = client.ui_screenshot().await.unwrap();
    assert_eq!(ss["base64"], "dummy_img");

    // Test get_session_colors
    let (fg, bg) = client.get_session_colors().await.unwrap();
    assert_eq!(fg, "#21c45c");
    assert_eq!(bg, "#ffffff");

    server_handle.abort();
}

#[tokio::test]
async fn test_mock_server_auth_failure() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);
        let mut line = String::new();

        reader.read_line(&mut line).await.unwrap();
        let req: Value = serde_json::from_str(&line).unwrap();

        let auth_resp = json!({
            "id": req["id"],
            "ok": false,
            "error": "Access denied: invalid token"
        });
        write_half
            .write_all(format!("{}\n", auth_resp).as_bytes())
            .await
            .unwrap();
        write_half.flush().await.unwrap();
    });

    let client = PhotoCraftClient::new(port, Some("wrong_token".to_string()));
    let res = client.connect().await;
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("Access denied"));
}

#[tokio::test]
async fn test_mock_server_auto_reconnect() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        // First connection: auth, then close abruptly
        if let Ok((stream, _)) = listener.accept().await {
            let (read_half, mut write_half) = stream.into_split();
            let mut reader = BufReader::new(read_half);
            let mut line = String::new();

            let _ = reader.read_line(&mut line).await;
            let req: Value = serde_json::from_str(&line).unwrap();
            let auth_resp = json!({ "id": req["id"], "ok": true, "result": null });
            let _ = write_half
                .write_all(format!("{}\n", auth_resp).as_bytes())
                .await;
            let _ = write_half.flush().await;

            // Close socket
            drop(write_half);
            drop(reader);
        }

        // Second connection (reconnect): auth, then answer command
        if let Ok((stream, _)) = listener.accept().await {
            let (read_half, mut write_half) = stream.into_split();
            let mut reader = BufReader::new(read_half);
            let mut line = String::new();

            let _ = reader.read_line(&mut line).await;
            let req: Value = serde_json::from_str(&line).unwrap();
            let auth_resp = json!({ "id": req["id"], "ok": true, "result": null });
            let _ = write_half
                .write_all(format!("{}\n", auth_resp).as_bytes())
                .await;
            let _ = write_half.flush().await;

            // Next command
            line.clear();
            let _ = reader.read_line(&mut line).await;
            let cmd_req: Value = serde_json::from_str(&line).unwrap();
            let cmd_resp =
                json!({ "id": cmd_req["id"], "ok": true, "result": { "reconnected": true } });
            let _ = write_half
                .write_all(format!("{}\n", cmd_resp).as_bytes())
                .await;
            let _ = write_half.flush().await;
        }
    });

    let client = PhotoCraftClient::new(port, Some("token".to_string()));
    client.connect().await.unwrap();

    // The next call will detect socket EOF/failure, auto-reconnect, and succeed!
    let res = client.call("test.cmd", json!({})).await.unwrap();
    assert_eq!(res["reconnected"], true);
}

#[tokio::test]
async fn test_client_dynamic_token_update() {
    let client = PhotoCraftClient::new(18492, Some("initial_token".to_string()));
    assert_eq!(client.get_token().await, Some("initial_token".to_string()));

    client.set_token(Some("updated_token".to_string())).await;
    assert_eq!(client.get_token().await, Some("updated_token".to_string()));

    client.set_token(None).await;
    assert_eq!(client.get_token().await, None);
}

#[tokio::test]
async fn test_connection_state_transitions() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let (read_half, mut write_half) = stream.into_split();
                let mut reader = BufReader::new(read_half);
                let mut line = String::new();

                if reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                    if let Ok(req) = serde_json::from_str::<Value>(&line) {
                        let token = req["params"]["token"].as_str().unwrap_or("");
                        if token == "valid_token" {
                            let resp = json!({ "id": req["id"], "ok": true, "result": null });
                            let _ = write_half.write_all(format!("{}\n", resp).as_bytes()).await;
                            let _ = write_half.flush().await;
                        } else {
                            let resp =
                                json!({ "id": req["id"], "ok": false, "error": "Invalid token" });
                            let _ = write_half.write_all(format!("{}\n", resp).as_bytes()).await;
                            let _ = write_half.flush().await;
                        }
                    }
                }
            });
        }
    });

    // 1. Valid token -> Connected
    let client_valid = PhotoCraftClient::new(port, Some("valid_token".to_string()));
    assert_eq!(
        client_valid.verify_connection().await,
        ConnectionState::Connected
    );

    // 2. Wrong token -> AuthRequired
    let client_invalid = PhotoCraftClient::new(port, Some("wrong_token".to_string()));
    assert_eq!(
        client_invalid.verify_connection().await,
        ConnectionState::AuthRequired
    );

    // 3. Unopened port -> Offline
    let client_offline = PhotoCraftClient::new(65530, Some("any".to_string()));
    assert_eq!(
        client_offline.verify_connection().await,
        ConnectionState::Offline
    );
}

#[test]
fn test_clipboard_image_rw() {
    let test_img = image::RgbaImage::from_pixel(4, 4, image::Rgba([200, 100, 50, 255]));
    // In some CI/headless/restricted environments, opening the OS clipboard may fail;
    // if writing succeeds, verify reading matches the dimensions.
    if PhotoCraftClient::write_clipboard_image(&test_img).is_ok() {
        if let Ok(read_back) =
            PhotoCraftClient::read_clipboard_image_retry(3, std::time::Duration::from_millis(10))
        {
            assert_eq!(read_back.width(), 4);
            assert_eq!(read_back.height(), 4);
        }
    }
}

#[test]
fn test_find_photocraft_executable_configured_path() {
    let temp_dir = std::env::temp_dir().join(format!(
        "exe_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);
    let dummy_exe = temp_dir.join("custom_photocraft.exe");
    std::fs::write(&dummy_exe, b"dummy binary").unwrap();

    let found = PhotoCraftClient::find_photocraft_executable(Some(&dummy_exe));
    assert_eq!(found, Some(dummy_exe.clone()));

    // Non-existent configured path should not return the non-existent path
    let non_existent = temp_dir.join("does_not_exist.exe");
    let found_non_existent = PhotoCraftClient::find_photocraft_executable(Some(&non_existent));
    assert_ne!(found_non_existent, Some(non_existent));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_launch_photocraft_token_generation() {
    let temp_dir = std::env::temp_dir().join(format!(
        "launch_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);

    let mock_exe = if cfg!(windows) {
        let p = temp_dir.join("mock.cmd");
        std::fs::write(&p, "@echo off\r\nexit 0\r\n").unwrap();
        p
    } else {
        let p = temp_dir.join("mock.sh");
        std::fs::write(&p, "#!/bin/sh\nexit 0\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755));
        }
        p
    };

    let token_file = temp_dir.join("PhotoCraftData").join("control.token");
    assert!(!token_file.exists());

    let mut child = PhotoCraftClient::launch_photocraft(&mock_exe, 18492, Some(&token_file))
        .expect("Launch should succeed");

    assert!(token_file.exists());
    let token_content = std::fs::read_to_string(&token_file).unwrap();
    assert_eq!(token_content.len(), 64);
    assert!(token_content.chars().all(|c| c.is_ascii_hexdigit()));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = token_file.metadata().unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "Token file mode on Unix must be strictly 0600");
    }

    let _ = child.wait();
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_is_photocraft_process_running_smoke() {
    let first = PhotoCraftClient::is_photocraft_process_running();
    // Second immediate call uses TTL cache (<= 1500ms)
    let cached = PhotoCraftClient::is_photocraft_process_running();
    assert_eq!(first, cached);

    // Invalidation resets the cache
    PhotoCraftClient::invalidate_process_running_cache();
    let after_invalidation = PhotoCraftClient::is_photocraft_process_running();
    assert_eq!(first, after_invalidation);
}

#[test]
fn test_launch_photocraft_token_existing_mode_repair() {
    let temp_dir = std::env::temp_dir().join(format!(
        "launch_repair_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::create_dir_all(&temp_dir);

    let mock_exe = if cfg!(windows) {
        let p = temp_dir.join("mock.cmd");
        std::fs::write(&p, "@echo off\r\nexit 0\r\n").unwrap();
        p
    } else {
        let p = temp_dir.join("mock.sh");
        std::fs::write(&p, "#!/bin/sh\nexit 0\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755));
        }
        p
    };

    let token_dir = temp_dir.join("PhotoCraftData");
    let _ = std::fs::create_dir_all(&token_dir);
    let token_file = token_dir.join("control.token");
    std::fs::write(
        &token_file,
        "existing_token_value_0123456789abcdef0123456789abcdef",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&token_file, std::fs::Permissions::from_mode(0o644));
    }

    let mut child = PhotoCraftClient::launch_photocraft(&mock_exe, 18492, Some(&token_file))
        .expect("Launch should succeed with pre-existing token");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = token_file.metadata().unwrap().permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o600,
            "Existing token permissions must be repaired to 0600 on launch"
        );
    }

    let _ = child.wait();
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_generate_token_format() {
    let token = generate_token();
    assert_eq!(token.len(), 64);
    assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_parse_session_colors_success() {
    let json = json!({
        "foreground": [0.13, 0.77, 0.36, 1.0],
        "background": [1.0, 1.0, 1.0, 1.0]
    });
    let (fg, bg) = parse_session_colors(&json);
    assert_eq!(fg, "#21c45c");
    assert_eq!(bg, "#ffffff");
}

#[test]
fn test_parse_session_colors_fallback() {
    let empty = json!({});
    let (fg, bg) = parse_session_colors(&empty);
    assert_eq!(fg, "#000000");
    assert_eq!(bg, "#ffffff");

    let malformed = json!({
        "foreground": [0.5],
        "background": "invalid"
    });
    let (fg2, bg2) = parse_session_colors(&malformed);
    assert_eq!(fg2, "#000000");
    assert_eq!(bg2, "#ffffff");
}

#[test]
fn test_is_token_dir_writable_app_bundle() {
    let bundle_file =
        std::path::PathBuf::from("/Applications/PhotoCraft.app/Contents/MacOS/control.token");
    let parent = bundle_file.parent().unwrap();
    let is_inside_bundle = parent
        .ancestors()
        .any(|p| p.extension().and_then(|e| e.to_str()) == Some("app"));
    assert!(
        is_inside_bundle,
        "Ancestor check should detect .app extension"
    );

    assert!(!is_token_dir_writable(&bundle_file));

    let user_bundle =
        std::path::PathBuf::from("/Users/developer/Downloads/PhotoCraft.app/control.token");
    assert!(!is_token_dir_writable(&user_bundle));
}

#[test]
fn test_launch_photocraft_non_existent_target() {
    let non_existent = std::path::Path::new("/path/that/does/not/exist/photocraft_fake_bin");
    let result = PhotoCraftClient::launch_photocraft(non_existent, 18492, None);
    assert!(
        result.is_err(),
        "Launch should fail for non-existent target"
    );
    let err_msg = result.err().unwrap().to_string();
    assert!(
        err_msg.contains("PhotoCraft executable not found"),
        "Error message should mention executable not found, got: {err_msg}"
    );
}

#[tokio::test]
async fn test_mock_server_mismatched_response_id() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);
        let mut line = String::new();

        reader.read_line(&mut line).await.unwrap();
        // Auth request: reply with mismatched ID
        let auth_resp = json!({ "id": 999999, "ok": true, "result": null });
        write_half
            .write_all(format!("{}\n", auth_resp).as_bytes())
            .await
            .unwrap();
        write_half.flush().await.unwrap();
    });

    let client = PhotoCraftClient::new(port, Some("test_token".to_string()));
    let res = client.connect().await;
    assert!(res.is_err(), "Mismatched ID must trigger an error!");
    let err_msg = res.unwrap_err().to_string();
    assert!(
        err_msg.contains("Mismatched auth response ID")
            || err_msg.contains("Mismatched JSON-RPC response ID"),
        "Error must specify mismatched ID: {err_msg}"
    );
}

#[tokio::test]
async fn test_clipboard_async_rw() {
    let test_img = image::RgbaImage::from_pixel(4, 4, image::Rgba([120, 80, 240, 255]));
    if PhotoCraftClient::write_clipboard_image_retry_async(
        &test_img,
        2,
        std::time::Duration::from_millis(10),
    )
    .await
    .is_ok()
    {
        if let Ok(read_back) = PhotoCraftClient::read_clipboard_image_retry_async(
            2,
            std::time::Duration::from_millis(10),
        )
        .await
        {
            assert_eq!(read_back.width(), 4);
            assert_eq!(read_back.height(), 4);
        }
    }
}

#[test]
fn test_launch_photocraft_unresolvable_desktop_entry() {
    let temp_dir = std::env::temp_dir();
    let invalid_desktop = temp_dir.join(format!("unresolvable_{}.desktop", std::process::id()));
    std::fs::write(
        &invalid_desktop,
        "[Desktop Entry]\nType=Application\nName=PhotoCraft\nExec=/nonexistent/binary/path %F\n",
    )
    .unwrap();

    let res = PhotoCraftClient::launch_photocraft(&invalid_desktop, 18492, None);
    assert!(res.is_err());
    let err_str = res.err().unwrap().to_string();
    assert!(
        err_str.contains("Failed to resolve executable binary from desktop entry")
            || err_str.contains("PhotoCraft executable not found"),
        "Error should explain failed executable resolution: {err_str}"
    );

    let _ = std::fs::remove_file(&invalid_desktop);
}

#[test]
fn test_resolve_candidate_desktop_file() {
    let temp_dir = std::env::temp_dir();
    let test_bin = temp_dir.join(format!("test_target_bin_{}.tmp", std::process::id()));
    std::fs::write(&test_bin, b"binary").unwrap();

    let valid_desktop = temp_dir.join(format!("valid_candidate_{}.desktop", std::process::id()));
    std::fs::write(
        &valid_desktop,
        format!(
            "[Desktop Entry]\nType=Application\nName=PhotoCraft\nExec=\"{}\" %U\n",
            test_bin.display()
        ),
    )
    .unwrap();

    let resolved = PhotoCraftClient::resolve_candidate(&valid_desktop);
    assert_eq!(resolved, Some(test_bin.clone()));

    let _ = std::fs::remove_file(&test_bin);
    let _ = std::fs::remove_file(&valid_desktop);
}

#[test]
fn test_read_clipboard_fallback_decode() {
    let original = image::RgbaImage::from_pixel(6, 6, image::Rgba([11, 22, 33, 255]));
    let mut png_bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    image::ImageEncoder::write_image(
        encoder,
        original.as_raw(),
        original.width(),
        original.height(),
        image::ExtendedColorType::Rgba8,
    )
    .unwrap();

    assert!(!png_bytes.is_empty());
    let decoded = image::load_from_memory(&png_bytes).unwrap().to_rgba8();
    assert_eq!(decoded.width(), 6);
    assert_eq!(decoded.height(), 6);
    assert_eq!(decoded.get_pixel(0, 0), &image::Rgba([11, 22, 33, 255]));
}

#[test]
fn test_find_photocraft_executable_programs_directory() {
    let temp_dir = std::env::temp_dir().join(format!("pc_test_prog_{}", std::process::id()));
    let bin_dir = temp_dir.join("photocraft-0.20.1-linux-x86_64").join("bin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let exe_name = if cfg!(windows) {
        "photocraft.exe"
    } else {
        "photocraft"
    };
    let fake_bin = bin_dir.join(exe_name);
    std::fs::write(&fake_bin, b"fake binary").unwrap();

    let resolved = PhotoCraftClient::resolve_candidate(&fake_bin);
    assert_eq!(resolved, Some(fake_bin.clone()));

    let resolved_dir = PhotoCraftClient::resolve_candidate(&bin_dir);
    assert_eq!(resolved_dir, Some(fake_bin.clone()));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[cfg(test)]
fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);

        s.push(TABLE[(b0 >> 2) as usize] as char);
        s.push(TABLE[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);

        if chunk.len() > 1 {
            s.push(TABLE[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            s.push('=');
        }

        if chunk.len() > 2 {
            s.push(TABLE[(b2 & 0x3f) as usize] as char);
        } else {
            s.push('=');
        }
    }
    s
}

#[test]
fn test_base64_decode_roundtrip() {
    let payload = b"Hello PhotoCraft Companion preview engine!";
    let encoded = encode_base64(payload);
    let decoded = decode_base64(&encoded).unwrap();
    assert_eq!(decoded, payload);

    let data_uri = format!("data:application/octet-stream;base64,{encoded}");
    let decoded_uri = decode_base64(&data_uri).unwrap();
    assert_eq!(decoded_uri, payload);
}
