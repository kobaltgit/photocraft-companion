use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::Sender;
use std::thread;
use tracing::{error, info};

/// GitHub repository owner and name for companion updates
pub const COMPANION_REPO: &str = "kobaltgit/photocraft-companion";
pub const COMPANION_GITHUB_URL: &str = "https://github.com/kobaltgit/photocraft-companion";
pub const COMPANION_ISSUES_URL: &str = "https://github.com/kobaltgit/photocraft-companion/issues";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseAsset {
    pub name: String,
    pub download_url: String,
    pub size_bytes: u64,
    pub is_installer: bool,
    pub is_portable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateCheckResult {
    pub tag_name: String,
    pub version: String,
    pub release_notes: String,
    pub html_url: String,
    pub published_at: String,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateState {
    Idle,
    Checking,
    UpToDate {
        checked_version: String,
    },
    UpdateAvailable {
        result: UpdateCheckResult,
    },
    Downloading {
        asset_name: String,
        target_path: PathBuf,
        is_portable: bool,
    },
    Downloaded {
        target_path: PathBuf,
        is_portable: bool,
    },
    ApplyingUpdate,
    Error(String),
}

/// Parses a SemVer-like version string and compares whether `latest` is strictly newer than `current`.
/// Supports optional leading 'v' or 'V' (e.g. "v1.2.3" vs "1.2.0").
pub fn is_version_newer(latest: &str, current: &str) -> bool {
    let parse_nums = |v: &str| -> Vec<u64> {
        let cleaned = v.trim().trim_start_matches(['v', 'V']);
        // Cut prerelease/build suffix if any (e.g. 1.0.0-beta.1)
        let main_part = cleaned.split('-').next().unwrap_or(cleaned);
        main_part
            .split('.')
            .filter_map(|p| p.parse::<u64>().ok())
            .collect()
    };

    let l_parts = parse_nums(latest);
    let c_parts = parse_nums(current);

    if l_parts.is_empty() || c_parts.is_empty() {
        return false;
    }

    let max_len = l_parts.len().max(c_parts.len());
    for i in 0..max_len {
        let l_val = *l_parts.get(i).unwrap_or(&0);
        let c_val = *c_parts.get(i).unwrap_or(&0);
        if l_val > c_val {
            return true;
        } else if l_val < c_val {
            return false;
        }
    }
    false
}

/// Determines whether a release asset matches the current operating system and architecture.
pub fn is_asset_matching_os(asset_name: &str, os: &str, arch: &str) -> bool {
    let lower = asset_name.to_lowercase();

    // Architectural check: if asset explicitly names arch, must match
    if lower.contains("arm64") || lower.contains("aarch64") {
        if arch != "aarch64" {
            return false;
        }
    } else if (lower.contains("x86_64") || lower.contains("x64") || lower.contains("amd64"))
        && arch != "x86_64"
    {
        return false;
    }

    match os {
        "windows" => {
            lower.ends_with(".exe")
                || lower.ends_with(".msi")
                || (lower.ends_with(".zip")
                    && (lower.contains("win")
                        || lower.contains("windows")
                        || !lower.contains("linux")))
        }
        "linux" => {
            lower.ends_with(".appimage")
                || lower.ends_with(".deb")
                || lower.ends_with(".rpm")
                || (lower.ends_with(".tar.gz") && lower.contains("linux"))
        }
        "macos" => {
            lower.ends_with(".dmg")
                || lower.ends_with(".pkg")
                || (lower.ends_with(".zip")
                    && (lower.contains("mac") || lower.contains("darwin") || lower.contains("osx")))
        }
        _ => true,
    }
}

/// Categorizes whether the asset is an installer or portable archive
pub fn categorize_asset(name: &str) -> (bool, bool) {
    let lower = name.to_lowercase();
    let is_installer = lower.ends_with(".exe")
        || lower.ends_with(".msi")
        || lower.ends_with(".pkg")
        || lower.ends_with(".deb")
        || lower.ends_with(".rpm")
        || lower.ends_with(".dmg")
        || lower.ends_with(".appimage");

    let is_portable = lower.ends_with(".zip") || lower.ends_with(".tar.gz");
    (is_installer, is_portable)
}

/// Resolves standard Downloads directory across platforms
pub fn get_downloads_dir() -> PathBuf {
    if let Some(user_dirs) = std::env::var_os("USERPROFILE") {
        let p = PathBuf::from(user_dirs).join("Downloads");
        if p.exists() {
            return p;
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        let p = PathBuf::from(home).join("Downloads");
        if p.exists() {
            return p;
        }
    }
    std::env::temp_dir()
}

/// Queries GitHub releases API using system curl or powershell without extra heavy dependencies
pub fn fetch_latest_release(repo: &str) -> Result<UpdateCheckResult, String> {
    let api_url = format!("https://api.github.com/repos/{}/releases/latest", repo);
    info!("Fetching latest release from {}", api_url);

    // Try curl first (available on modern Windows 10/11, macOS, Linux)
    let mut curl_cmd = Command::new("curl");
    curl_cmd
        .arg("-s")
        .arg("-L")
        .arg("-H")
        .arg("User-Agent: PhotoCraft-Companion-Updater")
        .arg("-H")
        .arg("Accept: application/vnd.github.v3+json")
        .arg(&api_url);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        curl_cmd.creation_flags(0x08000000);
    }
    let output = curl_cmd.output();

    let json_text = match output {
        Ok(out) if out.status.success() && !out.stdout.is_empty() => {
            String::from_utf8_lossy(&out.stdout).to_string()
        }
        _ => {
            // Fallback for Windows using PowerShell Invoke-RestMethod
            if cfg!(target_os = "windows") {
                let ps_cmd = format!(
                    "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; (Invoke-RestMethod -Uri '{}' -Headers @{{'User-Agent'='PhotoCraft-Companion-Updater'}}) | ConvertTo-Json -Depth 5",
                    api_url
                );
                let mut ps = Command::new("powershell");
                ps.arg("-NoProfile").arg("-Command").arg(&ps_cmd);
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    ps.creation_flags(0x08000000);
                }
                let ps_out = ps.output().map_err(|e| format!("PowerShell error: {e}"))?;

                if !ps_out.status.success() {
                    return Err(format!(
                        "GitHub API request failed: {}",
                        String::from_utf8_lossy(&ps_out.stderr)
                    ));
                }
                String::from_utf8_lossy(&ps_out.stdout).to_string()
            } else {
                return Err("Failed to execute curl or query GitHub Releases API".to_string());
            }
        }
    };

    let parsed: serde_json::Value = serde_json::from_str(&json_text)
        .map_err(|e| format!("Invalid JSON from GitHub API: {e}"))?;

    if let Some(msg) = parsed.get("message").and_then(|m| m.as_str()) {
        if msg.to_lowercase().contains("rate limit") {
            return Err("GitHub API rate limit exceeded. Please try again later.".to_string());
        }
        if msg.to_lowercase().contains("not found") {
            return Err("No releases found on GitHub repository.".to_string());
        }
    }

    let tag_name = parsed
        .get("tag_name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing tag_name in GitHub release".to_string())?
        .to_string();

    let clean_version = tag_name.trim_start_matches(['v', 'V']).to_string();
    let release_notes = parsed
        .get("body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let html_url = parsed
        .get("html_url")
        .and_then(|v| v.as_str())
        .unwrap_or(COMPANION_GITHUB_URL)
        .to_string();
    let published_at = parsed
        .get("published_at")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let mut assets = Vec::new();
    if let Some(asset_arr) = parsed.get("assets").and_then(|a| a.as_array()) {
        for a in asset_arr {
            let name = match a.get("name").and_then(|n| n.as_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            if !is_asset_matching_os(&name, os, arch) {
                continue;
            }

            let dl_url = match a.get("browser_download_url").and_then(|u| u.as_str()) {
                Some(u) => u.to_string(),
                None => continue,
            };

            let size = a.get("size").and_then(|s| s.as_u64()).unwrap_or(0);
            let (is_installer, is_portable) = categorize_asset(&name);

            assets.push(ReleaseAsset {
                name,
                download_url: dl_url,
                size_bytes: size,
                is_installer,
                is_portable,
            });
        }
    }

    Ok(UpdateCheckResult {
        tag_name,
        version: clean_version,
        release_notes,
        html_url,
        published_at,
        assets,
    })
}

/// Spawns a background thread that executes curl to download the given file URL to destination path
pub fn download_file_background(
    url: String,
    target_path: PathBuf,
    is_portable: bool,
    tx: Sender<Result<(PathBuf, bool), String>>,
) {
    thread::spawn(move || {
        info!("Starting download {} to {}", url, target_path.display());
        let mut curl_cmd = Command::new("curl");
        curl_cmd
            .arg("-L")
            .arg("-f")
            .arg("-o")
            .arg(&target_path)
            .arg(&url);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            curl_cmd.creation_flags(0x08000000);
        }
        let res = curl_cmd.status();

        match res {
            Ok(status) if status.success() => {
                info!("Downloaded successfully: {}", target_path.display());
                let _ = tx.send(Ok((target_path, is_portable)));
            }
            Ok(status) => {
                let err = format!("curl failed with exit code: {:?}", status.code());
                error!("{}", err);
                let _ = tx.send(Err(err));
            }
            Err(e) => {
                // Fallback via PowerShell on Windows if curl is somehow missing
                if cfg!(target_os = "windows") {
                    let ps_cmd = format!(
                        "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; (New-Object System.Net.WebClient).DownloadFile('{}', '{}')",
                        url,
                        target_path.to_string_lossy().replace('\\', "\\\\")
                    );
                    let mut ps = Command::new("powershell");
                    ps.arg("-NoProfile").arg("-Command").arg(&ps_cmd);
                    #[cfg(windows)]
                    {
                        use std::os::windows::process::CommandExt;
                        ps.creation_flags(0x08000000);
                    }
                    let ps_res = ps.status();

                    match ps_res {
                        Ok(st) if st.success() => {
                            info!("PowerShell download succeeded: {}", target_path.display());
                            let _ = tx.send(Ok((target_path, is_portable)));
                        }
                        _ => {
                            let _ = tx.send(Err(format!("Download failed: {e}")));
                        }
                    }
                } else {
                    let _ = tx.send(Err(format!("Failed to start curl: {e}")));
                }
            }
        }
    });
}

/// Unpacks a portable .zip update to a temporary folder and generates a platform update script
pub fn prepare_portable_update(zip_path: &Path) -> Result<PathBuf, String> {
    let file =
        std::fs::File::open(zip_path).map_err(|e| format!("Failed to open zip file: {e}"))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| format!("Failed to parse zip archive: {e}"))?;

    let temp_dir = std::env::temp_dir().join(format!("photocraft_update_{}", std::process::id()));
    if temp_dir.exists() {
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp extraction dir: {e}"))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Zip entry read error: {e}"))?;
        let outpath = match file.enclosed_name() {
            Some(path) => temp_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            let _ = std::fs::create_dir_all(&outpath);
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    let _ = std::fs::create_dir_all(p);
                }
            }
            let mut outfile = std::fs::File::create(&outpath)
                .map_err(|e| format!("Failed to extract file: {e}"))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("Failed to write extracted file: {e}"))?;
        }
    }

    // Determine target installation directory (the directory where our current executable lives)
    let current_exe =
        std::env::current_exe().map_err(|e| format!("Cannot get current exe path: {e}"))?;
    let install_dir = current_exe
        .parent()
        .ok_or_else(|| "Failed to get exe directory".to_string())?;

    let pid = std::process::id();

    if cfg!(target_os = "windows") {
        let script_path = std::env::temp_dir().join("photocraft_apply_update.cmd");
        // Script waits for current process to terminate, copies files with robocopy or xcopy, and restarts
        let script_content = format!(
            r#"@echo off
setlocal
echo Waiting for PhotoCraft Companion (PID: {pid}) to terminate...
:waitloop
tasklist /fi "PID eq {pid}" | find "{pid}" >nul
if not errorlevel 1 (
    timeout /t 1 /nobreak >nul
    goto waitloop
)
echo Updating files...
xcopy /E /Y /I "{src}\*" "{dest}\" >nul
echo Restarting PhotoCraft Companion...
start "" "{exe}"
del "%~f0"
exit
"#,
            pid = pid,
            src = temp_dir.display(),
            dest = install_dir.display(),
            exe = current_exe.display()
        );

        std::fs::write(&script_path, script_content)
            .map_err(|e| format!("Failed to write update script: {e}"))?;
        Ok(script_path)
    } else {
        let script_path = std::env::temp_dir().join("photocraft_apply_update.sh");
        let script_content = format!(
            r#"#!/bin/bash
while kill -0 {pid} 2>/dev/null; do
    sleep 1
done
cp -r "{src}/"* "{dest}/"
nohup "{exe}" >/dev/null 2>&1 &
rm "$0"
"#,
            pid = pid,
            src = temp_dir.display(),
            dest = install_dir.display(),
            exe = current_exe.display()
        );
        std::fs::write(&script_path, script_content)
            .map_err(|e| format!("Failed to write shell script: {e}"))?;
        Ok(script_path)
    }
}

/// Executes the generated update script and exits current application
pub fn apply_update_and_restart(script_path: &Path) -> Result<(), String> {
    if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "start", "", &script_path.to_string_lossy()]);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        cmd.spawn()
            .map_err(|e| format!("Failed to spawn update script: {e}"))?;
    } else {
        Command::new("sh")
            .arg(script_path)
            .spawn()
            .map_err(|e| format!("Failed to spawn update script: {e}"))?;
    }
    std::process::exit(0);
}

/// Launches installer executable (.exe / .msi / .pkg)
pub fn launch_installer(path: &Path) -> Result<(), String> {
    if cfg!(target_os = "windows") {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "start", "", &path.to_string_lossy()]);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        cmd.spawn()
            .map_err(|e| format!("Failed to launch installer: {e}"))?;
    } else if cfg!(target_os = "macos") {
        Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open installer: {e}"))?;
    } else {
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to launch installer: {e}"))?;
    }
    Ok(())
}

/// Opens the folder containing the downloaded file and selects it
pub fn reveal_in_folder(path: &Path) {
    if cfg!(target_os = "windows") {
        let _ = Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn();
    } else if cfg!(target_os = "macos") {
        let _ = Command::new("open").arg("-R").arg(path).spawn();
    } else {
        if let Some(parent) = path.parent() {
            let _ = Command::new("xdg-open").arg(parent).spawn();
        }
    }
}

/// Opens a web URL in the user's default browser cross-platform
pub fn open_browser_url(url: &str) {
    if cfg!(target_os = "windows") {
        // "start" is a cmd built-in; first empty string is the window title parameter
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "start", "", url]);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        let _ = cmd.spawn();
    } else if cfg!(target_os = "macos") {
        let _ = Command::new("open").arg(url).spawn();
    } else {
        let _ = Command::new("xdg-open").arg(url).spawn();
    }
}
