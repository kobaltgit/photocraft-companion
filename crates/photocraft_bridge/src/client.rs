use crate::protocol::{parse_session_colors, JsonRpcRequest, JsonRpcResponse};
use core_contracts::{CompanionError, ConnectionState, DocumentInfo, Result};
use image::RgbaImage;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::{debug, info};

pub const DEFAULT_CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);
pub const DEFAULT_RPC_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

pub struct PhotoCraftClient {
    port: u16,
    token: Arc<tokio::sync::RwLock<Option<String>>>,
    next_id: AtomicU64,
    writer: Arc<Mutex<Option<OwnedWriteHalf>>>,
    reader: Arc<Mutex<Option<BufReader<OwnedReadHalf>>>>,
}

impl PhotoCraftClient {
    pub fn new(port: u16, token: Option<String>) -> Self {
        Self {
            port,
            token: Arc::new(tokio::sync::RwLock::new(token)),
            next_id: AtomicU64::new(1),
            writer: Arc::new(Mutex::new(None)),
            reader: Arc::new(Mutex::new(None)),
        }
    }

    /// Try to locate running photocraft process directory
    pub fn find_running_photocraft_dir() -> Option<PathBuf> {
        crate::process::find_running_photocraft_dir()
    }

    /// Check whether a running PhotoCraft process is detected on the system (cached with 1500ms TTL)
    pub fn is_photocraft_process_running() -> bool {
        crate::process::is_photocraft_process_running()
    }

    /// Reset process running TTL cache to immediately re-check on user trigger
    pub fn invalidate_process_running_cache() {
        crate::process::invalidate_process_running_cache();
    }

    /// Inspect a candidate path (handling Linux .desktop entries, macOS .app bundles or plain directories)
    pub fn resolve_candidate(p: &Path) -> Option<PathBuf> {
        crate::launcher::resolve_candidate(p)
    }

    /// Locate PhotoCraft executable: configured path -> running process -> relative candidates -> standard install paths -> PATH
    pub fn find_photocraft_executable(configured_path: Option<&Path>) -> Option<PathBuf> {
        crate::launcher::find_photocraft_executable(configured_path)
    }

    /// Launch PhotoCraft instance with control port and token arguments
    pub fn launch_photocraft(
        exe_path: &Path,
        port: u16,
        token_file: Option<&Path>,
    ) -> Result<std::process::Child> {
        crate::launcher::launch_photocraft(exe_path, port, token_file)
    }

    /// Try to locate the token automatically from environment, process, or standard candidate paths
    pub fn default_token() -> Option<String> {
        crate::token::default_token()
    }

    /// Read image from system clipboard with automatic retry policy to handle OS clipboard locking
    pub fn read_clipboard_image() -> Result<RgbaImage> {
        crate::clipboard::read_clipboard_image()
    }

    /// Read image from system clipboard with custom retry count and backoff delay
    pub fn read_clipboard_image_retry(
        max_attempts: usize,
        delay: std::time::Duration,
    ) -> Result<RgbaImage> {
        crate::clipboard::read_clipboard_image_retry(max_attempts, delay)
    }

    /// Write image to system clipboard with automatic retry policy
    pub fn write_clipboard_image(img: &RgbaImage) -> Result<()> {
        crate::clipboard::write_clipboard_image(img)
    }

    /// Read image from system clipboard asynchronously via spawn_blocking to prevent Tokio thread blockage
    pub async fn read_clipboard_image_retry_async(
        max_attempts: usize,
        delay: std::time::Duration,
    ) -> Result<RgbaImage> {
        crate::clipboard::read_clipboard_image_retry_async(max_attempts, delay).await
    }

    /// Write image to system clipboard asynchronously via spawn_blocking
    pub async fn write_clipboard_image_retry_async(
        img: &RgbaImage,
        max_attempts: usize,
        delay: std::time::Duration,
    ) -> Result<()> {
        crate::clipboard::write_clipboard_image_retry_async(img, max_attempts, delay).await
    }

    /// Dynamically update token, disconnect existing stream, and prepare for new auth
    pub async fn set_token(&self, new_token: Option<String>) {
        *self.token.write().await = new_token;
        *self.writer.lock().await = None;
        *self.reader.lock().await = None;
    }

    /// Get current token if set
    pub async fn get_token(&self) -> Option<String> {
        self.token.read().await.clone()
    }

    pub async fn connect(&self) -> Result<()> {
        info!("Connecting to PhotoCraft at 127.0.0.1:{}...", self.port);
        let stream = tokio::time::timeout(
            DEFAULT_CONNECT_TIMEOUT,
            TcpStream::connect(("127.0.0.1", self.port)),
        )
        .await
        .map_err(|_| {
            CompanionError::Bridge("Connection to PhotoCraft timed out (exceeded 5s)".into())
        })?
        .map_err(|e| CompanionError::Bridge(format!("Failed to connect to PhotoCraft: {e}")))?;

        let (read_half, write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);
        let mut writer = write_half;

        // Perform authentication
        let current_token = self.token.read().await.clone();
        let token = current_token.or_else(Self::default_token);
        if let Some(token_val) = token {
            let auth_id = self.next_id.fetch_add(1, Ordering::SeqCst);
            let auth_req = json!({
                "id": auth_id,
                "method": "auth",
                "params": {
                    "token": token_val
                }
            });
            let mut line = serde_json::to_string(&auth_req)?;
            line.push('\n');

            tokio::time::timeout(DEFAULT_RPC_TIMEOUT, async {
                writer.write_all(line.as_bytes()).await?;
                writer.flush().await
            })
            .await
            .map_err(|_| CompanionError::Bridge("Auth write timeout (exceeded 30s)".into()))?
            .map_err(|e| CompanionError::Bridge(format!("Failed to write auth: {e}")))?;

            let mut resp_line = String::new();
            tokio::time::timeout(DEFAULT_RPC_TIMEOUT, reader.read_line(&mut resp_line))
                .await
                .map_err(|_| CompanionError::Bridge("Auth read timeout (exceeded 30s)".into()))?
                .map_err(|e| CompanionError::Bridge(format!("Failed to read auth resp: {e}")))?;

            let resp: JsonRpcResponse = serde_json::from_str(&resp_line)
                .map_err(|e| CompanionError::Bridge(format!("Invalid auth response JSON: {e}")))?;

            if resp.id != Some(auth_id) {
                return Err(CompanionError::Bridge(format!(
                    "Mismatched auth response ID: expected {auth_id}, got {:?}",
                    resp.id
                )));
            }

            if !resp.ok {
                let err_msg = resp
                    .error
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "Unknown auth error".into());
                return Err(CompanionError::Bridge(format!(
                    "PhotoCraft auth failed: {err_msg}"
                )));
            }
            info!("Authenticated with PhotoCraft successfully.");
        } else {
            debug!("Connecting without control token.");
        }

        *self.writer.lock().await = Some(writer);
        *self.reader.lock().await = Some(reader);

        Ok(())
    }

    /// Check honest 3-state connection status: Offline, AuthRequired, or Connected
    pub async fn verify_connection(&self) -> ConnectionState {
        let stream = match tokio::time::timeout(
            DEFAULT_CONNECT_TIMEOUT,
            TcpStream::connect(("127.0.0.1", self.port)),
        )
        .await
        {
            Ok(Ok(s)) => s,
            _ => return ConnectionState::Offline,
        };

        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);

        let current_token = self.token.read().await.clone();
        let token = current_token.or_else(Self::default_token);

        let token_val = match token {
            Some(t) if !t.trim().is_empty() => t.trim().to_string(),
            _ => return ConnectionState::AuthRequired,
        };

        let auth_req = json!({
            "id": self.next_id.fetch_add(1, Ordering::SeqCst),
            "method": "auth",
            "params": {
                "token": token_val
            }
        });
        let mut line = match serde_json::to_string(&auth_req) {
            Ok(l) => l,
            Err(_) => return ConnectionState::AuthRequired,
        };
        line.push('\n');

        let write_op = async {
            write_half.write_all(line.as_bytes()).await?;
            write_half.flush().await
        };
        if tokio::time::timeout(std::time::Duration::from_secs(3), write_op)
            .await
            .is_err()
        {
            return ConnectionState::AuthRequired;
        }

        let mut resp_line = String::new();
        let read_op = reader.read_line(&mut resp_line);
        match tokio::time::timeout(std::time::Duration::from_secs(3), read_op).await {
            Ok(Ok(n)) if n > 0 => {
                if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(&resp_line) {
                    if resp.ok {
                        ConnectionState::Connected
                    } else {
                        ConnectionState::AuthRequired
                    }
                } else {
                    ConnectionState::AuthRequired
                }
            }
            _ => ConnectionState::AuthRequired,
        }
    }

    pub async fn is_alive(&self) -> bool {
        TcpStream::connect(("127.0.0.1", self.port)).await.is_ok()
    }

    async fn call_once(&self, method: &str, params: &Value) -> Result<Value> {
        let mut writer_guard = self.writer.lock().await;
        let mut reader_guard = self.reader.lock().await;

        if writer_guard.is_none() || reader_guard.is_none() {
            drop(writer_guard);
            drop(reader_guard);
            self.connect().await?;
            writer_guard = self.writer.lock().await;
            reader_guard = self.reader.lock().await;
        }

        let writer = writer_guard
            .as_mut()
            .ok_or_else(|| CompanionError::Bridge("Writer not connected".into()))?;
        let reader = reader_guard
            .as_mut()
            .ok_or_else(|| CompanionError::Bridge("Reader not connected".into()))?;

        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let req = JsonRpcRequest {
            id,
            method: method.to_string(),
            params: params.clone(),
        };

        let mut line = serde_json::to_string(&req)?;
        line.push('\n');

        let map_socket_err = |e: std::io::Error, op: &str| -> CompanionError {
            let err_str = e.to_string();
            if err_str.contains("10053")
                || err_str.contains("ConnectionReset")
                || err_str.contains("connection was aborted")
                || err_str.contains("Broken pipe")
            {
                CompanionError::Bridge(format!(
                    "Authentication error: PhotoCraft aborted connection during {op}. Valid control token required."
                ))
            } else {
                CompanionError::Bridge(format!("Socket {op} error: {e}"))
            }
        };

        let write_res = tokio::time::timeout(DEFAULT_RPC_TIMEOUT, async {
            writer.write_all(line.as_bytes()).await?;
            writer.flush().await
        })
        .await;

        match write_res {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                *writer_guard = None;
                *reader_guard = None;
                return Err(map_socket_err(e, "write"));
            }
            Err(_) => {
                *writer_guard = None;
                *reader_guard = None;
                return Err(CompanionError::Bridge(
                    "Socket write timeout (exceeded 30s)".into(),
                ));
            }
        }

        let mut resp_line = String::new();
        let read_res =
            tokio::time::timeout(DEFAULT_RPC_TIMEOUT, reader.read_line(&mut resp_line)).await;

        match read_res {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => {
                *writer_guard = None;
                *reader_guard = None;
                return Err(map_socket_err(e, "read"));
            }
            Err(_) => {
                *writer_guard = None;
                *reader_guard = None;
                return Err(CompanionError::Bridge(
                    "Socket read timeout (exceeded 30s)".into(),
                ));
            }
        }

        let resp: JsonRpcResponse = serde_json::from_str(&resp_line).map_err(|e| {
            CompanionError::Bridge(format!("Invalid response JSON: {e} (raw: {resp_line})"))
        })?;

        if resp.id != Some(id) {
            return Err(CompanionError::Bridge(format!(
                "Mismatched JSON-RPC response ID: expected {id}, got {:?}",
                resp.id
            )));
        }

        if !resp.ok {
            let err_msg = resp
                .error
                .map(|v| v.to_string())
                .unwrap_or_else(|| "Command failed".into());
            return Err(CompanionError::Bridge(format!(
                "PhotoCraft command failed: {err_msg}"
            )));
        }

        Ok(resp.result)
    }

    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        match self.call_once(method, &params).await {
            Ok(res) => Ok(res),
            Err(e) => {
                info!("Bridge call failed ({e}), re-establishing connection to PhotoCraft and retrying...");
                *self.writer.lock().await = None;
                *self.reader.lock().await = None;
                self.connect().await?;
                self.call_once(method, &params).await
            }
        }
    }

    /// Execute an engine command by id with parameters
    pub async fn execute_command(&self, command: &str, params: Value) -> Result<Value> {
        self.call(
            "engine.execute",
            json!({
                "command": command,
                "params": params
            }),
        )
        .await
    }

    /// Inspect current UI and document state
    pub async fn ui_inspect(&self) -> Result<Value> {
        self.call("ui.inspect", json!({})).await
    }

    /// Capture live screenshot
    pub async fn ui_screenshot(&self) -> Result<Value> {
        self.call("ui.screenshot", json!({})).await
    }

    /// Query current active foreground and background colors from PhotoCraft session
    pub async fn get_session_colors(&self) -> Result<(String, String)> {
        let res = self
            .execute_command("session.inspect", serde_json::json!({}))
            .await?;
        Ok(parse_session_colors(&res))
    }

    /// Get current active document info
    pub async fn get_document_info(&self) -> Result<DocumentInfo> {
        let inspect = self.execute_command("document.inspect", json!({})).await?;
        let name = inspect
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled")
            .to_string();
        let width = inspect.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let height = inspect.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
        let active_layer = inspect.get("activeLayer").and_then(|v| v.as_u64());

        let mut active_layer_name = None;
        let mut active_layer_bounds = None;
        if let (Some(active_id), Some(layers)) = (
            active_layer,
            inspect.get("layers").and_then(|l| l.as_array()),
        ) {
            for layer in layers {
                if layer.get("id").and_then(|id| id.as_u64()) == Some(active_id) {
                    active_layer_name = layer
                        .get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string());
                    if let Some(arr) = layer.get("bounds").and_then(|b| b.as_array()) {
                        if arr.len() == 4 {
                            active_layer_bounds = Some([
                                arr[0].as_i64().unwrap_or(0) as i32,
                                arr[1].as_i64().unwrap_or(0) as i32,
                                arr[2].as_i64().unwrap_or(0) as i32,
                                arr[3].as_i64().unwrap_or(0) as i32,
                            ]);
                        }
                    }
                    break;
                }
            }
        }

        let has_selection = inspect
            .get("hasSelection")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let selection_bounds = inspect
            .get("selectionBounds")
            .and_then(|v| v.as_array())
            .and_then(|arr| {
                if arr.len() == 4 {
                    Some([
                        arr[0].as_i64().unwrap_or(0) as i32,
                        arr[1].as_i64().unwrap_or(0) as i32,
                        arr[2].as_i64().unwrap_or(0) as i32,
                        arr[3].as_i64().unwrap_or(0) as i32,
                    ])
                } else {
                    None
                }
            });

        let revision = inspect
            .get("revision")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        Ok(DocumentInfo {
            name,
            width,
            height,
            active_layer,
            active_layer_name,
            has_selection,
            selection_bounds,
            active_layer_bounds,
            revision,
        })
    }

    /// Rename layer by id using layer.renameLayer
    pub async fn rename_layer(&self, layer_id: u64, new_name: &str) -> Result<Value> {
        self.execute_command(
            "layer.renameLayer",
            json!({
                "layer": layer_id,
                "name": new_name
            }),
        )
        .await
    }

    /// Set layer properties (blend mode, opacity, etc.) by id using layer.setProps
    pub async fn set_layer_props(&self, layer_id: u64, mut props: Value) -> Result<Value> {
        if let Some(map) = props.as_object_mut() {
            map.entry("layer").or_insert_with(|| json!(layer_id));
            map.entry("layerId").or_insert_with(|| json!(layer_id));
        } else {
            props = json!({
                "layer": layer_id,
                "layerId": layer_id,
                "props": props
            });
        }
        self.execute_command("layer.setProps", props).await
    }

    /// Paste from system clipboard into the active document
    pub async fn paste(&self) -> Result<Value> {
        self.execute_command("edit.paste", json!({})).await
    }

    /// Copy active layer to clipboard and read it as RgbaImage with settling delay
    pub async fn copy_active_layer_image(&self) -> Result<RgbaImage> {
        self.execute_command("edit.copy", json!({})).await?;
        // Micro-pause (40ms) allows PhotoCraft to finalize writing DIB/PNG payload and release clipboard lock
        tokio::time::sleep(std::time::Duration::from_millis(40)).await;
        Self::read_clipboard_image_retry_async(8, std::time::Duration::from_millis(40)).await
    }

    /// Copy active layer and binary selection mask if active selection exists on canvas
    pub async fn copy_active_layer_and_selection_mask(
        &self,
    ) -> Result<(RgbaImage, Option<image::GrayImage>)> {
        core_contracts::BridgeOperations::copy_active_layer_and_selection_mask(self).await
    }

    /// Deselect active selection in PhotoCraft
    pub async fn deselect(&self) -> Result<Value> {
        self.execute_command("select.deselect", json!({})).await
    }

    /// Merge active layer down into the layer below it in PhotoCraft
    pub async fn merge_down(&self) -> Result<Value> {
        self.execute_command("layer.mergeDown", json!({})).await
    }

    /// Write image to clipboard and paste it in place in PhotoCraft
    pub async fn paste_image_in_place(&self, img: &RgbaImage) -> Result<Value> {
        Self::write_clipboard_image_retry_async(img, 5, std::time::Duration::from_millis(25))
            .await?;
        // Micro-pause (30ms) ensures Windows clipboard lock is released before PhotoCraft paste command reads it
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        self.execute_command("edit.pasteSpecial.pasteInPlace", json!({}))
            .await
    }

    /// Write image to clipboard and paste it centered in PhotoCraft
    pub async fn paste_image_centered(&self, img: &RgbaImage) -> Result<Value> {
        Self::write_clipboard_image_retry_async(img, 5, std::time::Duration::from_millis(25))
            .await?;
        // Micro-pause (30ms) ensures Windows clipboard lock is released before PhotoCraft paste command reads it
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        self.paste().await
    }

    /// Set foreground and/or background colors in PhotoCraft via tools.setColors
    pub async fn set_colors(
        &self,
        foreground: Option<&str>,
        background: Option<&str>,
    ) -> Result<Value> {
        let mut params = serde_json::Map::new();
        if let Some(fg) = foreground {
            params.insert("foreground".to_string(), json!(fg));
        }
        if let Some(bg) = background {
            params.insert("background".to_string(), json!(bg));
        }
        self.execute_command("tools.setColors", Value::Object(params))
            .await
    }

    /// Save active selection to a named alpha channel
    pub async fn save_selection_channel(&self, name: &str) -> Result<Value> {
        self.execute_command("select.saveSelection", json!({ "name": name }))
            .await
    }

    /// Load selection from a named alpha channel
    pub async fn load_selection_channel(&self, name: &str) -> Result<Value> {
        self.execute_command(
            "select.loadSelection",
            json!({ "channel": name, "operation": "new" }),
        )
        .await
    }

    /// Delete a named alpha channel
    pub async fn delete_selection_channel(&self, name: &str) -> Result<Value> {
        self.execute_command("channel.delete", json!({ "channel": name }))
            .await
    }
}

#[async_trait::async_trait]
impl core_contracts::BridgeOperations for PhotoCraftClient {
    async fn execute_command(&self, command: &str, params: Value) -> Result<Value> {
        self.execute_command(command, params).await
    }

    async fn copy_active_layer_image(&self) -> Result<RgbaImage> {
        self.copy_active_layer_image().await
    }

    async fn paste_image_in_place(&self, image: &RgbaImage) -> Result<Value> {
        self.paste_image_in_place(image).await
    }

    async fn rename_layer(&self, layer_id: u64, new_name: &str) -> Result<Value> {
        self.rename_layer(layer_id, new_name).await
    }

    async fn set_layer_props(&self, layer_id: u64, props: Value) -> Result<Value> {
        self.set_layer_props(layer_id, props).await
    }

    async fn get_document_info(&self) -> Result<DocumentInfo> {
        self.get_document_info().await
    }
}
