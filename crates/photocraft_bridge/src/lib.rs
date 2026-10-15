use core_contracts::{CompanionError, DocumentInfo, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::{debug, info};

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    id: u64,
    method: String,
    params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    id: Option<u64>,
    #[serde(default)]
    ok: bool,
    #[serde(default)]
    result: Value,
    #[serde(default)]
    error: Option<Value>,
}

use core_contracts::ConnectionState;

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
    pub fn find_running_photocraft_dir() -> Option<std::path::PathBuf> {
        #[cfg(target_os = "windows")]
        {
            if let Some(dir) = find_running_photocraft_dir_win32() {
                return Some(dir);
            }

            // Fallback: PowerShell query with CREATE_NO_WINDOW
            let mut cmd = std::process::Command::new("powershell");
            cmd.args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "(Get-Process -Name photocraft -ErrorAction SilentlyContinue).Path",
            ]);
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x08000000);
            }
            if let Ok(out) = cmd.output() {
                if out.status.success() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    for line in text.lines() {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            let p = std::path::PathBuf::from(trimmed);
                            if let Some(parent) = p.parent() {
                                if parent.is_dir() {
                                    return Some(parent.to_path_buf());
                                }
                            }
                        }
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Check /proc for running photocraft binary
            if let Ok(entries) = std::fs::read_dir("/proc") {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    if let Some(pid_str) = file_name.to_str() {
                        if pid_str.chars().all(|c| c.is_ascii_digit()) {
                            let exe_link = entry.path().join("exe");
                            if let Ok(target) = std::fs::read_link(&exe_link) {
                                if target.file_name().and_then(|n| n.to_str()) == Some("photocraft")
                                {
                                    if let Some(parent) = target.parent() {
                                        return Some(parent.to_path_buf());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }
}

#[cfg(target_os = "windows")]
fn find_running_photocraft_dir_win32() -> Option<std::path::PathBuf> {
    #[allow(clippy::upper_case_acronyms)]
    type HANDLE = *mut std::ffi::c_void;
    const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
    const TH32CS_SNAPPROCESS: u32 = 0x00000002;
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
    const MAX_PATH: usize = 260;

    #[repr(C)]
    #[allow(non_snake_case, clippy::upper_case_acronyms)]
    struct PROCESSENTRY32W {
        dwSize: u32,
        cntUsage: u32,
        th32ProcessID: u32,
        th32DefaultHeapID: usize,
        th32ModuleID: u32,
        cntThreads: u32,
        th32ParentProcessID: u32,
        pcPriClassBase: i32,
        dwFlags: u32,
        szExeFile: [u16; MAX_PATH],
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateToolhelp32Snapshot(dwFlags: u32, th32ProcessID: u32) -> HANDLE;
        fn Process32FirstW(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> i32;
        fn Process32NextW(hSnapshot: HANDLE, lppe: *mut PROCESSENTRY32W) -> i32;
        fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> HANDLE;
        fn QueryFullProcessImageNameW(
            hProcess: HANDLE,
            dwFlags: u32,
            lpExeName: *mut u16,
            lpdwSize: *mut u32,
        ) -> i32;
        fn CloseHandle(hObject: HANDLE) -> i32;
    }

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return None;
        }

        let mut entry = std::mem::zeroed::<PROCESSENTRY32W>();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        let mut found_dir = None;

        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                let null_pos = entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                let exe_name = String::from_utf16_lossy(&entry.szExeFile[..null_pos]);

                if exe_name.eq_ignore_ascii_case("photocraft.exe") {
                    let h_proc =
                        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, entry.th32ProcessID);
                    if !h_proc.is_null() && h_proc != INVALID_HANDLE_VALUE {
                        let mut path_buf = [0u16; 1024];
                        let mut size = path_buf.len() as u32;
                        if QueryFullProcessImageNameW(h_proc, 0, path_buf.as_mut_ptr(), &mut size)
                            != 0
                        {
                            let full_path = String::from_utf16_lossy(&path_buf[..size as usize]);
                            let pb = std::path::PathBuf::from(full_path);
                            if let Some(parent) = pb.parent() {
                                if parent.is_dir() {
                                    found_dir = Some(parent.to_path_buf());
                                    CloseHandle(h_proc);
                                    break;
                                }
                            }
                        }
                        CloseHandle(h_proc);
                    }
                }

                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }
        }

        CloseHandle(snapshot);
        found_dir
    }
}

/// Helper to generate a 64-character hex token (32 random bytes)
pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];
    let mut filled = false;

    #[cfg(target_os = "windows")]
    {
        #[link(name = "bcrypt")]
        extern "system" {
            fn BCryptGenRandom(
                h_algorithm: *mut std::ffi::c_void,
                pb_buffer: *mut u8,
                cb_buffer: u32,
                dw_flags: u32,
            ) -> i32;
        }
        const BCRYPT_USE_SYSTEM_PREFERRED_RNG: u32 = 0x00000002;
        unsafe {
            if BCryptGenRandom(
                std::ptr::null_mut(),
                bytes.as_mut_ptr(),
                bytes.len() as u32,
                BCRYPT_USE_SYSTEM_PREFERRED_RNG,
            ) == 0
            {
                filled = true;
            }
        }
    }

    #[cfg(unix)]
    {
        if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
            use std::io::Read;
            if f.read_exact(&mut bytes).is_ok() {
                filled = true;
            }
        }
    }

    if !filled {
        // Fallback PRNG seeded with nanoseconds and process ID
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(123456789);
        let pid = std::process::id() as u128;
        let mut state = seed ^ (pid << 64);
        for byte in bytes.iter_mut() {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            *byte = (state >> 56) as u8;
        }
    }

    let mut hex = String::with_capacity(64);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(&mut hex, "{:02x}", b);
    }
    hex
}

impl PhotoCraftClient {
    /// Locate PhotoCraft executable: configured path -> running process -> relative candidates -> standard install paths -> PATH
    pub fn find_photocraft_executable(
        configured_path: Option<&std::path::Path>,
    ) -> Option<std::path::PathBuf> {
        let exe_name = if cfg!(windows) {
            "photocraft.exe"
        } else {
            "photocraft"
        };

        // 1. Explicitly configured path
        if let Some(path) = configured_path {
            if path.is_file() {
                return Some(path.to_path_buf());
            }
            if path.is_dir() {
                let candidate = path.join(exe_name);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }

        // 2. Running photocraft process directory
        if let Some(proc_dir) = Self::find_running_photocraft_dir() {
            let candidate = proc_dir.join(exe_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }

        // 3. Neighbor directories
        let mut candidate_paths = Vec::new();
        if let Ok(cwd) = std::env::current_dir() {
            candidate_paths.push(cwd.join("..").join(exe_name));
            candidate_paths.push(cwd.join("..").join("..").join(exe_name));
            candidate_paths.push(cwd.join("..").join("PhotoCraft").join(exe_name));
            candidate_paths.push(cwd.join("..").join("..").join("PhotoCraft").join(exe_name));
            candidate_paths.push(cwd.join(exe_name));
        }

        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(dir) = current_exe.parent() {
                candidate_paths.push(dir.join(exe_name));
                candidate_paths.push(dir.join("..").join(exe_name));
                candidate_paths.push(dir.join("..").join("..").join(exe_name));
                candidate_paths.push(dir.join("..").join("PhotoCraft").join(exe_name));
                candidate_paths.push(dir.join("..").join("..").join("PhotoCraft").join(exe_name));
            }
        }

        // 4. Windows standard install paths
        #[cfg(target_os = "windows")]
        {
            if let Ok(pf) = std::env::var("ProgramFiles") {
                candidate_paths.push(
                    std::path::PathBuf::from(pf)
                        .join("PhotoCraft")
                        .join(exe_name),
                );
            }
            if let Ok(pf86) = std::env::var("ProgramFiles(x86)") {
                candidate_paths.push(
                    std::path::PathBuf::from(pf86)
                        .join("PhotoCraft")
                        .join(exe_name),
                );
            }
            if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
                candidate_paths.push(
                    std::path::PathBuf::from(local_appdata)
                        .join("Programs")
                        .join("PhotoCraft")
                        .join(exe_name),
                );
            }
        }

        // Unix standard paths
        #[cfg(not(target_os = "windows"))]
        {
            candidate_paths.push(std::path::PathBuf::from("/usr/local/bin").join(exe_name));
            candidate_paths.push(std::path::PathBuf::from("/usr/bin").join(exe_name));
            candidate_paths.push(std::path::PathBuf::from("/opt/photocraft").join(exe_name));
            if let Ok(home) = std::env::var("HOME") {
                candidate_paths.push(
                    std::path::PathBuf::from(home)
                        .join(".local/bin")
                        .join(exe_name),
                );
            }
        }

        for path in candidate_paths {
            if path.is_file() {
                return Some(path);
            }
        }

        // 5. System PATH
        if let Some(path_var) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path_var) {
                let candidate = dir.join(exe_name);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }

        None
    }

    /// Launch PhotoCraft instance with control port and token arguments
    pub fn launch_photocraft(
        exe_path: &std::path::Path,
        port: u16,
        token_file: Option<&std::path::Path>,
    ) -> Result<std::process::Child> {
        if !exe_path.is_file() {
            return Err(CompanionError::Bridge(format!(
                "PhotoCraft executable not found at: {}",
                exe_path.display()
            )));
        }

        // Determine token file path:
        // If token_file is None, use PhotoCraftData/control.token next to exe_path (or fallback to cwd)
        let token_path = match token_file {
            Some(p) => p.to_path_buf(),
            None => {
                let base_dir = exe_path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("."));
                base_dir.join("PhotoCraftData").join("control.token")
            }
        };

        // If token file does not exist, generate 64-char hex token and write to file
        if !token_path.exists() {
            if let Some(parent) = token_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    CompanionError::Bridge(format!(
                        "Failed to create token directory {}: {e}",
                        parent.display()
                    ))
                })?;
            }
            let token = generate_token();
            std::fs::write(&token_path, token).map_err(|e| {
                CompanionError::Bridge(format!(
                    "Failed to write token file {}: {e}",
                    token_path.display()
                ))
            })?;
        }

        let mut cmd = std::process::Command::new(exe_path);
        let port_str = port.to_string();
        let token_path_str = token_path
            .to_str()
            .ok_or_else(|| CompanionError::Bridge("Invalid UTF-8 in token file path".into()))?;

        cmd.args([
            "--control",
            &port_str,
            "--control-token-file",
            token_path_str,
        ]);

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW (BUG-053)
        }

        let child = cmd.spawn().map_err(|e| {
            CompanionError::Bridge(format!(
                "Failed to launch PhotoCraft process ({}): {e}",
                exe_path.display()
            ))
        })?;

        Ok(child)
    }

    /// Try to locate the token automatically from environment, process, or standard candidate paths
    pub fn default_token() -> Option<String> {
        // 1. Environment variables
        if let Ok(token) = std::env::var("PHOTOCRAFT_CONTROL_TOKEN") {
            let trimmed = token.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
        if let Ok(path) = std::env::var("PHOTOCRAFT_CONTROL_TOKEN_FILE") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }

        // 2. Running photocraft process executable directory -> PhotoCraftData/control.token
        if let Some(proc_dir) = Self::find_running_photocraft_dir() {
            let candidate = proc_dir.join("PhotoCraftData").join("control.token");
            if let Ok(content) = std::fs::read_to_string(&candidate) {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }

        // 2b. Candidate next to find_photocraft_executable(None) -> PhotoCraftData/control.token
        if let Some(exe_path) = Self::find_photocraft_executable(None) {
            if let Some(parent) = exe_path.parent() {
                let candidate = parent.join("PhotoCraftData").join("control.token");
                if let Ok(content) = std::fs::read_to_string(&candidate) {
                    let trimmed = content.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }

        // 3. Candidate search directories: relative and AppData / user config
        let mut candidate_paths = Vec::new();
        if let Ok(cwd) = std::env::current_dir() {
            candidate_paths.push(cwd.join("PhotoCraftData").join("control.token"));
            candidate_paths.push(cwd.join("..").join("PhotoCraftData").join("control.token"));
            candidate_paths.push(
                cwd.join("..")
                    .join("..")
                    .join("PhotoCraftData")
                    .join("control.token"),
            );
        }
        candidate_paths.push(std::path::PathBuf::from("PhotoCraftData").join("control.token"));
        candidate_paths.push(std::path::PathBuf::from("../PhotoCraftData").join("control.token"));

        if let Ok(appdata) = std::env::var("APPDATA") {
            candidate_paths.push(
                std::path::PathBuf::from(appdata)
                    .join("PhotoCraft")
                    .join("control.token"),
            );
        }
        if let Ok(home) = std::env::var("USERPROFILE").or_else(|_| std::env::var("HOME")) {
            candidate_paths.push(
                std::path::PathBuf::from(&home)
                    .join(".config")
                    .join("photocraft")
                    .join("control.token"),
            );
            candidate_paths.push(
                std::path::PathBuf::from(&home)
                    .join("PhotoCraftData")
                    .join("control.token"),
            );
        }

        for path in candidate_paths {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }

        None
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
        let stream = TcpStream::connect(("127.0.0.1", self.port))
            .await
            .map_err(|e| CompanionError::Bridge(format!("Failed to connect to PhotoCraft: {e}")))?;

        let (read_half, write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);
        let mut writer = write_half;

        // Perform authentication
        let current_token = self.token.read().await.clone();
        let token = current_token.or_else(Self::default_token);
        if let Some(token_val) = token {
            let auth_req = json!({
                "id": self.next_id.fetch_add(1, Ordering::SeqCst),
                "method": "auth",
                "params": {
                    "token": token_val
                }
            });
            let mut line = serde_json::to_string(&auth_req)?;
            line.push('\n');

            writer
                .write_all(line.as_bytes())
                .await
                .map_err(|e| CompanionError::Bridge(format!("Failed to write auth: {e}")))?;
            writer
                .flush()
                .await
                .map_err(|e| CompanionError::Bridge(e.to_string()))?;

            let mut resp_line = String::new();
            reader
                .read_line(&mut resp_line)
                .await
                .map_err(|e| CompanionError::Bridge(format!("Failed to read auth resp: {e}")))?;

            let resp: JsonRpcResponse = serde_json::from_str(&resp_line)
                .map_err(|e| CompanionError::Bridge(format!("Invalid auth response JSON: {e}")))?;

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
        let stream = match TcpStream::connect(("127.0.0.1", self.port)).await {
            Ok(s) => s,
            Err(_) => return ConnectionState::Offline,
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

        if write_half.write_all(line.as_bytes()).await.is_err() || write_half.flush().await.is_err()
        {
            return ConnectionState::AuthRequired;
        }

        let mut resp_line = String::new();
        match reader.read_line(&mut resp_line).await {
            Ok(n) if n > 0 => {
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

        writer
            .write_all(line.as_bytes())
            .await
            .map_err(|e| map_socket_err(e, "write"))?;
        writer
            .flush()
            .await
            .map_err(|e| map_socket_err(e, "flush"))?;

        let mut resp_line = String::new();
        reader
            .read_line(&mut resp_line)
            .await
            .map_err(|e| map_socket_err(e, "read"))?;

        let resp: JsonRpcResponse = serde_json::from_str(&resp_line).map_err(|e| {
            CompanionError::Bridge(format!("Invalid response JSON: {e} (raw: {resp_line})"))
        })?;

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

        Ok(DocumentInfo {
            name,
            width,
            height,
            active_layer,
            active_layer_name,
            has_selection,
            selection_bounds,
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

    /// Paste from system clipboard into the active document
    pub async fn paste(&self) -> Result<Value> {
        self.execute_command("edit.paste", json!({})).await
    }

    /// Read image from system clipboard with automatic retry policy to handle OS clipboard locking
    pub fn read_clipboard_image() -> Result<image::RgbaImage> {
        Self::read_clipboard_image_retry(6, std::time::Duration::from_millis(35))
    }

    /// Read image from system clipboard with custom retry count and backoff delay
    pub fn read_clipboard_image_retry(
        max_attempts: usize,
        delay: std::time::Duration,
    ) -> Result<image::RgbaImage> {
        let mut last_err = None;
        for attempt in 0..max_attempts {
            if attempt > 0 {
                std::thread::sleep(delay);
            }
            match arboard::Clipboard::new() {
                Ok(mut clipboard) => match clipboard.get_image() {
                    Ok(img_data) => {
                        return image::RgbaImage::from_raw(
                            img_data.width as u32,
                            img_data.height as u32,
                            img_data.bytes.into_owned(),
                        )
                        .ok_or_else(|| {
                            CompanionError::Bridge(
                                "Failed to construct RgbaImage from clipboard bytes".into(),
                            )
                        });
                    }
                    Err(e) => {
                        tracing::debug!(
                            "Clipboard read attempt {}/{} failed: {}",
                            attempt + 1,
                            max_attempts,
                            e
                        );
                        last_err = Some(CompanionError::Bridge(format!(
                            "Failed to get image from clipboard: {e}"
                        )));
                    }
                },
                Err(e) => {
                    tracing::debug!(
                        "Clipboard open attempt {}/{} failed: {}",
                        attempt + 1,
                        max_attempts,
                        e
                    );
                    last_err = Some(CompanionError::Bridge(format!(
                        "Failed to open clipboard: {e}"
                    )));
                }
            }
        }
        Err(last_err.unwrap_or_else(|| {
            CompanionError::Bridge("Failed to read image from clipboard after retries".into())
        }))
    }

    /// Write image to system clipboard with automatic retry policy
    pub fn write_clipboard_image(img: &image::RgbaImage) -> Result<()> {
        let mut last_err = None;
        let img_data = arboard::ImageData {
            width: img.width() as usize,
            height: img.height() as usize,
            bytes: std::borrow::Cow::Borrowed(img.as_raw()),
        };
        for attempt in 0..5 {
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
            match arboard::Clipboard::new() {
                Ok(mut clipboard) => match clipboard.set_image(img_data.clone()) {
                    Ok(()) => return Ok(()),
                    Err(e) => {
                        tracing::debug!("Clipboard write attempt {}/5 failed: {}", attempt + 1, e);
                        last_err = Some(CompanionError::Bridge(format!(
                            "Failed to write image to clipboard: {e}"
                        )));
                    }
                },
                Err(e) => {
                    tracing::debug!(
                        "Clipboard open for write attempt {}/5 failed: {}",
                        attempt + 1,
                        e
                    );
                    last_err = Some(CompanionError::Bridge(format!(
                        "Failed to open clipboard: {e}"
                    )));
                }
            }
        }
        Err(last_err.unwrap_or_else(|| {
            CompanionError::Bridge("Failed to write image to clipboard after retries".into())
        }))
    }

    /// Copy active layer to clipboard and read it as RgbaImage with settling delay
    pub async fn copy_active_layer_image(&self) -> Result<image::RgbaImage> {
        self.execute_command("edit.copy", json!({})).await?;
        // Micro-pause (40ms) allows PhotoCraft to finalize writing DIB/PNG payload and release Windows clipboard lock
        tokio::time::sleep(std::time::Duration::from_millis(40)).await;
        Self::read_clipboard_image()
    }

    /// Write image to clipboard and paste it in place in PhotoCraft
    pub async fn paste_image_in_place(&self, img: &image::RgbaImage) -> Result<Value> {
        Self::write_clipboard_image(img)?;
        // Micro-pause (30ms) ensures Windows clipboard lock is released before PhotoCraft paste command reads it
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        self.execute_command("edit.pasteSpecial.pasteInPlace", json!({}))
            .await
    }
}

#[async_trait::async_trait]
impl core_contracts::BridgeOperations for PhotoCraftClient {
    async fn execute_command(&self, command: &str, params: Value) -> Result<Value> {
        self.execute_command(command, params).await
    }

    async fn copy_active_layer_image(&self) -> Result<image::RgbaImage> {
        self.copy_active_layer_image().await
    }

    async fn paste_image_in_place(&self, image: &image::RgbaImage) -> Result<Value> {
        self.paste_image_in_place(image).await
    }

    async fn rename_layer(&self, layer_id: u64, new_name: &str) -> Result<Value> {
        self.rename_layer(layer_id, new_name).await
    }

    async fn get_document_info(&self) -> Result<DocumentInfo> {
        self.get_document_info().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
                                let _ =
                                    write_half.write_all(format!("{}\n", resp).as_bytes()).await;
                                let _ = write_half.flush().await;
                            } else {
                                let resp = json!({ "id": req["id"], "ok": false, "error": "Invalid token" });
                                let _ =
                                    write_half.write_all(format!("{}\n", resp).as_bytes()).await;
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
            if let Ok(read_back) = PhotoCraftClient::read_clipboard_image_retry(
                3,
                std::time::Duration::from_millis(10),
            ) {
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

        let _ = child.wait();
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_generate_token_format() {
        let token = generate_token();
        assert_eq!(token.len(), 64);
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
