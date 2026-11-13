use std::path::{Path, PathBuf};

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

/// Returns the user-writable path for control.token appropriate for the current operating system
pub fn user_data_token_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let app_support = PathBuf::from(&home)
                .join("Library")
                .join("Application Support")
                .join("PhotoCraft")
                .join("control.token");
            return app_support;
        }
    }
    #[cfg(target_os = "linux")]
    {
        if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
            return PathBuf::from(xdg).join("photocraft").join("control.token");
        }
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(&home).join(".local/share/photocraft/control.token");
        }
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            return PathBuf::from(local)
                .join("PhotoCraft")
                .join("control.token");
        }
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata)
                .join("PhotoCraft")
                .join("control.token");
        }
    }
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        return PathBuf::from(home)
            .join("PhotoCraftData")
            .join("control.token");
    }
    PathBuf::from("PhotoCraftData").join("control.token")
}

/// Checks if directory is safe and writable for placing control.token (not a read-only or system volume)
pub fn is_token_dir_writable(path: &Path) -> bool {
    let parent = match path.parent() {
        Some(p) => p,
        None => return false,
    };
    // Semantic protection: never allow writing tokens inside .app bundles (preserves code signature on macOS)
    let is_inside_bundle = parent
        .ancestors()
        .any(|p| p.extension().and_then(|e| e.to_str()) == Some("app"));
    if is_inside_bundle {
        return false;
    }

    let parent_str = parent.to_string_lossy();
    #[cfg(target_os = "macos")]
    {
        if parent_str.starts_with("/Applications")
            || parent_str.starts_with("/Volumes")
            || parent_str.starts_with("/usr")
            || parent_str.starts_with("/System")
        {
            return false;
        }
    }
    #[cfg(target_os = "linux")]
    {
        if parent_str.starts_with("/usr")
            || parent_str.starts_with("/opt")
            || parent_str.starts_with("/var")
            || parent_str.starts_with("/etc")
            || parent_str.starts_with("/tmp/.mount_")
        {
            return false;
        }
    }
    #[cfg(target_os = "windows")]
    {
        let upper = parent_str.to_uppercase();
        if upper.contains("PROGRAM FILES") || upper.contains("WINDOWS\\SYSTEM") {
            return false;
        }
    }

    if parent.exists() {
        let test_file = parent.join(format!(".token_test_{}", std::process::id()));
        if std::fs::write(&test_file, b"ok").is_ok() {
            let _ = std::fs::remove_file(&test_file);
            true
        } else {
            false
        }
    } else {
        match std::fs::create_dir_all(parent) {
            Ok(_) => {
                let test_file = parent.join(format!(".token_test_{}", std::process::id()));
                if std::fs::write(&test_file, b"ok").is_ok() {
                    let _ = std::fs::remove_file(&test_file);
                    true
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }
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
    if let Some(proc_dir) = crate::process::find_running_photocraft_dir() {
        let candidate = proc_dir.join("PhotoCraftData").join("control.token");
        if let Ok(content) = std::fs::read_to_string(&candidate) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }

    // 2b. Candidate next to find_photocraft_executable(None) -> PhotoCraftData/control.token
    if let Some(exe_path) = crate::launcher::find_photocraft_executable(None) {
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

    // 3. Candidate search directories: user data, relative, and AppData / user config
    let mut candidate_paths = Vec::new();
    candidate_paths.push(user_data_token_path());

    if let Ok(home) = std::env::var("HOME") {
        candidate_paths.push(
            PathBuf::from(&home)
                .join("PhotoCraftData")
                .join("control.token"),
        );
        candidate_paths.push(
            PathBuf::from(&home).join("Library/Application Support/PhotoCraft/control.token"),
        );
        candidate_paths.push(PathBuf::from(&home).join(".config/photocraft/control.token"));
    }

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
    candidate_paths.push(PathBuf::from("PhotoCraftData").join("control.token"));
    candidate_paths.push(PathBuf::from("../PhotoCraftData").join("control.token"));

    if let Ok(appdata) = std::env::var("APPDATA") {
        candidate_paths.push(
            PathBuf::from(appdata)
                .join("PhotoCraft")
                .join("control.token"),
        );
    }
    if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
        candidate_paths.push(
            PathBuf::from(local_appdata)
                .join("PhotoCraft")
                .join("control.token"),
        );
    }
    if let Ok(profile) = std::env::var("USERPROFILE") {
        candidate_paths.push(
            PathBuf::from(&profile)
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
