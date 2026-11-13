#[cfg(target_os = "macos")]
use std::path::Path;
use std::path::PathBuf;

static CACHED_PROCESS_RUNNING: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static LAST_PROCESS_CHECK_MS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static PROCESS_CHECK_START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

/// Try to locate running photocraft process directory
pub fn find_running_photocraft_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Some(dir) = find_running_photocraft_dir_win32() {
            return Some(dir);
        }
    }

    #[cfg(target_os = "macos")]
    {
        let mut cmd = std::process::Command::new("/bin/ps");
        cmd.args(["-A", "-o", "comm"]);
        if let Ok(out) = cmd.output() {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let path = Path::new(trimmed);
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        let file_lower = file_name.to_lowercase();
                        if !file_lower.contains("companion") && file_lower == "photocraft" {
                            if let Some(parent) = path.parent() {
                                if parent.is_dir() {
                                    return Some(parent.to_path_buf());
                                }
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
                        let proc_dir = entry.path();
                        let exe_link = proc_dir.join("exe");

                        let mut is_match = false;
                        let mut target_path: Option<PathBuf> = None;

                        if let Ok(target) = std::fs::read_link(&exe_link) {
                            let target_str = target.to_string_lossy();
                            let target_lower = target_str.to_lowercase();
                            if target_lower.contains("photocraft")
                                && !target_lower.contains("companion")
                            {
                                is_match = true;
                                target_path = Some(target);
                            }
                        }

                        if !is_match {
                            let cmdline_file = proc_dir.join("cmdline");
                            if let Ok(cmd_bytes) = std::fs::read(&cmdline_file) {
                                let cmd_str = String::from_utf8_lossy(&cmd_bytes).to_lowercase();
                                if cmd_str.contains("photocraft") && !cmd_str.contains("companion")
                                {
                                    is_match = true;
                                }
                            }
                        }

                        if is_match {
                            if let Some(tp) = target_path {
                                if let Some(parent) = tp.parent() {
                                    if parent.is_dir() {
                                        return Some(parent.to_path_buf());
                                    }
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

/// Check if photocraft executable or process is currently active/running (cached with 1000ms TTL)
pub fn is_photocraft_process_running() -> bool {
    let start = PROCESS_CHECK_START.get_or_init(std::time::Instant::now);
    let now_ms = start.elapsed().as_millis() as u64;
    let last_check = LAST_PROCESS_CHECK_MS.load(std::sync::atomic::Ordering::Relaxed);

    if now_ms >= last_check && (now_ms - last_check) < 1000 {
        return CACHED_PROCESS_RUNNING.load(std::sync::atomic::Ordering::Relaxed);
    }

    let is_running = find_running_photocraft_dir().is_some();
    CACHED_PROCESS_RUNNING.store(is_running, std::sync::atomic::Ordering::Relaxed);
    LAST_PROCESS_CHECK_MS.store(now_ms, std::sync::atomic::Ordering::Relaxed);
    is_running
}

/// Invalidate cached process state
pub fn invalidate_process_running_cache() {
    LAST_PROCESS_CHECK_MS.store(0, std::sync::atomic::Ordering::Relaxed);
    CACHED_PROCESS_RUNNING.store(false, std::sync::atomic::Ordering::Relaxed);
}

#[cfg(target_os = "windows")]
fn find_running_photocraft_dir_win32() -> Option<PathBuf> {
    #[repr(C)]
    struct PROCESSENTRY32W {
        dw_size: u32,
        cnt_usage: u32,
        th32_process_id: u32,
        th32_default_heap_id: usize,
        th32_module_id: u32,
        cnt_threads: u32,
        th32_parent_process_id: u32,
        pc_pri_class_base: i32,
        dw_flags: u32,
        sz_exe_file: [u16; 260],
    }

    const TH32CS_SNAPPROCESS: u32 = 0x00000002;
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateToolhelp32Snapshot(dw_flags: u32, th32_process_id: u32) -> *mut std::ffi::c_void;
        fn Process32FirstW(h_snapshot: *mut std::ffi::c_void, lppe: *mut PROCESSENTRY32W) -> i32;
        fn Process32NextW(h_snapshot: *mut std::ffi::c_void, lppe: *mut PROCESSENTRY32W) -> i32;
        fn CloseHandle(h_object: *mut std::ffi::c_void) -> i32;
        fn OpenProcess(
            dw_desired_access: u32,
            b_inherit_handle: i32,
            dw_process_id: u32,
        ) -> *mut std::ffi::c_void;
        fn QueryFullProcessImageNameW(
            h_process: *mut std::ffi::c_void,
            dw_flags: u32,
            lp_exe_name: *mut u16,
            lpdw_size: *mut u32,
        ) -> i32;
    }

    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot.is_null() || snapshot == -1isize as *mut std::ffi::c_void {
            return None;
        }

        let mut entry = std::mem::zeroed::<PROCESSENTRY32W>();
        entry.dw_size = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        let mut found_dir = None;
        if Process32FirstW(snapshot, &mut entry) != 0 {
            loop {
                let len = entry
                    .sz_exe_file
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.sz_exe_file.len());
                let exe_name = String::from_utf16_lossy(&entry.sz_exe_file[..len]);
                let exe_name_lower = exe_name.to_lowercase();

                if (exe_name_lower == "photocraft.exe" || exe_name_lower.contains("photocraft"))
                    && !exe_name_lower.contains("companion")
                {
                    let h_proc =
                        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, entry.th32_process_id);
                    if !h_proc.is_null() {
                        let mut buf = [0u16; 1024];
                        let mut size = buf.len() as u32;
                        if QueryFullProcessImageNameW(h_proc, 0, buf.as_mut_ptr(), &mut size) != 0
                            && size > 0
                        {
                            let full_path_str = String::from_utf16_lossy(&buf[..size as usize]);
                            let full_path = PathBuf::from(full_path_str);
                            if let Some(parent) = full_path.parent() {
                                found_dir = Some(parent.to_path_buf());
                            }
                        }
                        CloseHandle(h_proc);
                    }
                    if found_dir.is_some() {
                        break;
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
