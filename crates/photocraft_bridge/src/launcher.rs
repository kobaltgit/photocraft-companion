use crate::desktop_entry;
use crate::process::find_running_photocraft_dir;
use crate::token::{generate_token, is_token_dir_writable, user_data_token_path};
use core_contracts::{CompanionError, Result};
use std::path::{Path, PathBuf};

/// Inspect a candidate path (handling Linux .desktop entries, macOS .app bundles or plain directories)
pub fn resolve_candidate(p: &Path) -> Option<PathBuf> {
    let exe_name = if cfg!(windows) {
        "photocraft.exe"
    } else {
        "photocraft"
    };

    if p.is_file() {
        // If it's a Linux FreeDesktop .desktop entry file
        if p.extension()
            .and_then(|e| e.to_str())
            .map(|e| e.eq_ignore_ascii_case("desktop"))
            .unwrap_or(false)
        {
            if let Some(resolved) = desktop_entry::parse_desktop_file(p) {
                return Some(resolved);
            }
            return None;
        }
        return Some(p.to_path_buf());
    }
    if p.is_dir() {
        // If it's a macOS .app bundle
        let mac_candidates = [
            p.join("Contents").join("MacOS").join("photocraft"),
            p.join("Contents").join("MacOS").join("PhotoCraft"),
        ];
        for mc in mac_candidates {
            if mc.is_file() {
                return Some(mc);
            }
        }
        // Check if any file inside Contents/MacOS exists
        let mac_dir = p.join("Contents").join("MacOS");
        if mac_dir.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&mac_dir) {
                for entry in entries.flatten() {
                    let ep = entry.path();
                    if ep.is_file() {
                        return Some(ep);
                    }
                }
            }
        }

        // Check standard binary names in directory
        let sub_candidates = [
            p.join(exe_name),
            p.join("PhotoCraft.exe"),
            p.join("photocraft.exe"),
            p.join("PhotoCraft"),
            p.join("photocraft"),
        ];
        for sc in sub_candidates {
            if sc.is_file() {
                return Some(sc);
            }
        }
    }
    None
}

/// Locate PhotoCraft executable: configured path -> running process -> relative candidates -> standard install paths -> PATH
pub fn find_photocraft_executable(configured_path: Option<&Path>) -> Option<PathBuf> {
    let exe_name = if cfg!(windows) {
        "photocraft.exe"
    } else {
        "photocraft"
    };

    // 1. Explicitly configured path
    if let Some(path) = configured_path {
        if let Some(resolved) = resolve_candidate(path) {
            return Some(resolved);
        }
    }

    // 2. Running photocraft process directory
    if let Some(proc_dir) = find_running_photocraft_dir() {
        if let Some(resolved) = resolve_candidate(&proc_dir) {
            return Some(resolved);
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

            // Sibling portable directories in parent folder (e.g. ~/Programs/photocraft-.../bin/photocraft)
            if let Some(grandparent) = dir.parent() {
                if let Ok(entries) = std::fs::read_dir(grandparent) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_dir() {
                            let name_lower = p
                                .file_name()
                                .and_then(|n| n.to_str())
                                .map(|s| s.to_lowercase())
                                .unwrap_or_default();
                            if name_lower.contains("photocraft")
                                && !name_lower.contains("companion")
                            {
                                candidate_paths.push(p.join("bin").join(exe_name));
                                candidate_paths.push(p.join(exe_name));
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. Windows standard install paths
    #[cfg(target_os = "windows")]
    {
        if let Ok(pf) = std::env::var("ProgramFiles") {
            candidate_paths.push(PathBuf::from(pf).join("PhotoCraft").join(exe_name));
        }
        if let Ok(pf86) = std::env::var("ProgramFiles(x86)") {
            candidate_paths.push(PathBuf::from(pf86).join("PhotoCraft").join(exe_name));
        }
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            candidate_paths.push(
                PathBuf::from(local_appdata)
                    .join("Programs")
                    .join("PhotoCraft")
                    .join(exe_name),
            );
        }
    }

    // 4b. macOS standard install paths
    #[cfg(target_os = "macos")]
    {
        candidate_paths.push(PathBuf::from("/Applications/PhotoCraft.app"));
        candidate_paths.push(PathBuf::from("/Applications/photocraft.app"));
        candidate_paths.push(PathBuf::from(
            "/Applications/PhotoCraft.app/Contents/MacOS/photocraft",
        ));
        candidate_paths.push(PathBuf::from(
            "/Applications/PhotoCraft.app/Contents/MacOS/PhotoCraft",
        ));
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(&home);
            candidate_paths.push(home_path.join("Applications/PhotoCraft.app"));
            candidate_paths.push(home_path.join("Applications/photocraft.app"));
            candidate_paths
                .push(home_path.join("Applications/PhotoCraft.app/Contents/MacOS/photocraft"));
            candidate_paths
                .push(home_path.join("Applications/PhotoCraft.app/Contents/MacOS/PhotoCraft"));
            candidate_paths.push(home_path.join("Downloads/PhotoCraft.app"));
            candidate_paths.push(home_path.join("Downloads/photocraft.app"));
            candidate_paths.push(home_path.join("Downloads/PhotoCraft"));
            candidate_paths.push(home_path.join("Downloads/photocraft"));
        }
        candidate_paths.push(PathBuf::from("/opt/homebrew/bin/photocraft"));
        candidate_paths.push(PathBuf::from("/usr/local/bin/photocraft"));
    }

    // 4c. Linux standard install paths & Desktop entries
    #[cfg(target_os = "linux")]
    {
        candidate_paths.push(PathBuf::from("/usr/bin/photocraft"));
        candidate_paths.push(PathBuf::from("/usr/bin/PhotoCraft"));
        candidate_paths.push(PathBuf::from("/usr/local/bin/photocraft"));
        candidate_paths.push(PathBuf::from("/usr/local/bin/PhotoCraft"));
        candidate_paths.push(PathBuf::from("/opt/photocraft/photocraft"));
        candidate_paths.push(PathBuf::from("/opt/PhotoCraft/PhotoCraft"));
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(&home);
            candidate_paths.push(home_path.join(".local/bin/photocraft"));
            candidate_paths.push(home_path.join(".local/bin/PhotoCraft"));
            candidate_paths.push(home_path.join("bin/photocraft"));
            candidate_paths.push(home_path.join("bin/PhotoCraft"));
            candidate_paths.push(home_path.join("Programs/photocraft/bin/photocraft"));
            candidate_paths.push(home_path.join("Programs/photocraft/photocraft"));
            candidate_paths.push(home_path.join("Programs/PhotoCraft/bin/PhotoCraft"));
            candidate_paths.push(home_path.join("Programs/PhotoCraft/PhotoCraft"));
            candidate_paths.push(home_path.join("Applications/photocraft/bin/photocraft"));
            candidate_paths.push(home_path.join("Applications/photocraft/photocraft"));
            candidate_paths.push(home_path.join("Applications/PhotoCraft/bin/PhotoCraft"));
            candidate_paths.push(home_path.join("Applications/PhotoCraft/PhotoCraft"));

            for base in [home_path.join("Programs"), home_path.join("Applications")] {
                if base.is_dir() {
                    if let Ok(entries) = std::fs::read_dir(&base) {
                        for entry in entries.flatten() {
                            let path = entry.path();
                            if path.is_dir() {
                                let name_lower = path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .map(|s| s.to_lowercase())
                                    .unwrap_or_default();
                                if name_lower.contains("photocraft")
                                    && !name_lower.contains("companion")
                                {
                                    candidate_paths.push(path.join("bin/photocraft"));
                                    candidate_paths.push(path.join("bin/PhotoCraft"));
                                    candidate_paths.push(path.join("photocraft"));
                                    candidate_paths.push(path.join("PhotoCraft"));
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut desktop_dirs = Vec::new();
        if let Ok(home) = std::env::var("HOME") {
            let home_path = PathBuf::from(&home);
            desktop_dirs.push(home_path.join(".local/share/applications"));
            desktop_dirs.push(home_path.join(".local/share/flatpak/exports/share/applications"));
        }
        desktop_dirs.push(PathBuf::from("/usr/local/share/applications"));
        desktop_dirs.push(PathBuf::from("/usr/share/applications"));
        desktop_dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));

        for d_dir in desktop_dirs {
            if !d_dir.is_dir() {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(&d_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.eq_ignore_ascii_case("desktop"))
                        .unwrap_or(false)
                    {
                        let file_name_lower = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .map(|s| s.to_lowercase())
                            .unwrap_or_default();
                        let is_candidate_name = file_name_lower.contains("photocraft")
                            && !file_name_lower.contains("companion");

                        if is_candidate_name {
                            candidate_paths.push(path.clone());
                        } else if let Ok(content) = std::fs::read_to_string(&path) {
                            if let Some(de) = desktop_entry::parse_desktop_entry(&content) {
                                let name_matches = de
                                    .name
                                    .as_deref()
                                    .map(|n| {
                                        let nl = n.to_lowercase();
                                        nl.contains("photocraft") && !nl.contains("companion")
                                    })
                                    .unwrap_or(false);
                                if name_matches {
                                    candidate_paths.push(path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for path in candidate_paths {
        if let Some(resolved) = resolve_candidate(&path) {
            return Some(resolved);
        }
    }

    // 5. System PATH
    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            for name in [exe_name, "PhotoCraft", "photocraft"] {
                let candidate = dir.join(name);
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

/// Launch PhotoCraft instance with control port and token arguments
pub fn launch_photocraft(
    exe_path: &Path,
    port: u16,
    token_file: Option<&Path>,
) -> Result<std::process::Child> {
    let is_desktop_entry = exe_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("desktop"))
        .unwrap_or(false);

    let actual_exe = if is_desktop_entry {
        desktop_entry::parse_desktop_file(exe_path).ok_or_else(|| {
            CompanionError::Bridge(format!(
                "Failed to resolve executable binary from desktop entry: {}",
                exe_path.display()
            ))
        })?
    } else if exe_path.is_dir() {
        resolve_candidate(exe_path).unwrap_or_else(|| exe_path.to_path_buf())
    } else {
        exe_path.to_path_buf()
    };

    #[cfg(target_os = "macos")]
    let app_bundle = if actual_exe.extension().and_then(|e| e.to_str()) == Some("app") {
        Some(actual_exe.clone())
    } else {
        actual_exe
            .ancestors()
            .find(|p| p.extension().and_then(|e| e.to_str()) == Some("app"))
            .map(|p| p.to_path_buf())
    };
    #[cfg(not(target_os = "macos"))]
    let app_bundle: Option<PathBuf> = None;

    let is_valid_target = if app_bundle.is_some() {
        actual_exe.is_file() || actual_exe.is_dir()
    } else {
        actual_exe.is_file()
    };

    if !is_valid_target {
        return Err(CompanionError::Bridge(format!(
            "PhotoCraft executable not found at: {}",
            actual_exe.display()
        )));
    }

    // Determine token file path:
    let token_path = match token_file {
        Some(p) => p.to_path_buf(),
        None => {
            #[cfg(target_os = "macos")]
            if app_bundle.is_some() {
                user_data_token_path()
            } else {
                let base_dir = actual_exe.parent().unwrap_or_else(|| Path::new("."));
                let candidate = base_dir.join("PhotoCraftData").join("control.token");
                if is_token_dir_writable(&candidate) {
                    candidate
                } else {
                    user_data_token_path()
                }
            }
            #[cfg(not(target_os = "macos"))]
            {
                let base_dir = actual_exe.parent().unwrap_or_else(|| Path::new("."));
                let candidate = base_dir.join("PhotoCraftData").join("control.token");
                if is_token_dir_writable(&candidate) {
                    candidate
                } else {
                    user_data_token_path()
                }
            }
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
        #[cfg(unix)]
        {
            use std::io::Write;
            use std::os::unix::fs::OpenOptionsExt;
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .mode(0o600)
                .open(&token_path)
                .map_err(|e| {
                    CompanionError::Bridge(format!(
                        "Failed to write token file {}: {e}",
                        token_path.display()
                    ))
                })?;
            file.write_all(token.as_bytes()).map_err(|e| {
                CompanionError::Bridge(format!(
                    "Failed to write token data {}: {e}",
                    token_path.display()
                ))
            })?;
        }
        #[cfg(not(unix))]
        {
            std::fs::write(&token_path, token).map_err(|e| {
                CompanionError::Bridge(format!(
                    "Failed to write token file {}: {e}",
                    token_path.display()
                ))
            })?;
        }
    }

    // Repair / enforce 0600 permissions on Unix (PhotoCraft 0.18+ requirement, Issue #10)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&token_path, std::fs::Permissions::from_mode(0o600));
    }

    if let Some(bundle_path) = app_bundle {
        let mut cmd = std::process::Command::new("/usr/bin/open");
        cmd.arg("-n")
            .arg("-a")
            .arg(&bundle_path)
            .arg("--args")
            .arg("--control")
            .arg(port.to_string())
            .arg("--control-token-file")
            .arg(&token_path);
        return cmd.spawn().map_err(|e| {
            CompanionError::Bridge(format!("Failed to launch PhotoCraft bundle via open: {e}"))
        });
    }

    let mut cmd = std::process::Command::new(&actual_exe);
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
            actual_exe.display()
        ))
    })?;

    Ok(child)
}
