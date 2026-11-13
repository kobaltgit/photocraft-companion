use core_contracts::{CompanionError, Result};
use image::RgbaImage;

/// Read image from system clipboard with automatic retry policy to handle OS clipboard locking
pub fn read_clipboard_image() -> Result<RgbaImage> {
    read_clipboard_image_retry(8, std::time::Duration::from_millis(40))
}

/// Read image from system clipboard with custom retry count and backoff delay
pub fn read_clipboard_image_retry(
    max_attempts: usize,
    delay: std::time::Duration,
) -> Result<RgbaImage> {
    let mut last_err = None;
    for attempt in 0..max_attempts {
        if attempt > 0 {
            std::thread::sleep(delay);
        }
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => match clipboard.get_image() {
                Ok(img_data) => {
                    return RgbaImage::from_raw(
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
                    if let Some(img) = read_clipboard_image_platform_fallback() {
                        return Ok(img);
                    }
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
                if let Some(img) = read_clipboard_image_platform_fallback() {
                    return Ok(img);
                }
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
pub fn write_clipboard_image(img: &RgbaImage) -> Result<()> {
    let mut last_err = None;
    let img_data = arboard::ImageData {
        width: img.width() as usize,
        height: img.height() as usize,
        bytes: std::borrow::Cow::Borrowed(img.as_raw()),
    };
    for attempt in 0..6 {
        if attempt > 0 {
            std::thread::sleep(std::time::Duration::from_millis(30));
        }
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => match clipboard.set_image(img_data.clone()) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    if write_clipboard_image_platform_fallback(img) {
                        return Ok(());
                    }
                    tracing::debug!("Clipboard write attempt {}/6 failed: {}", attempt + 1, e);
                    last_err = Some(CompanionError::Bridge(format!(
                        "Failed to write image to clipboard: {e}"
                    )));
                }
            },
            Err(e) => {
                if write_clipboard_image_platform_fallback(img) {
                    return Ok(());
                }
                tracing::debug!(
                    "Clipboard open for write attempt {}/6 failed: {}",
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

/// Read image from system clipboard asynchronously via spawn_blocking to prevent Tokio thread blockage
pub async fn read_clipboard_image_retry_async(
    max_attempts: usize,
    delay: std::time::Duration,
) -> Result<RgbaImage> {
    let mut last_err = None;
    for attempt in 0..max_attempts {
        if attempt > 0 {
            tokio::time::sleep(delay).await;
        }
        let res = tokio::task::spawn_blocking(|| {
            match arboard::Clipboard::new().and_then(|mut c| c.get_image()) {
                Ok(img_data) => RgbaImage::from_raw(
                    img_data.width as u32,
                    img_data.height as u32,
                    img_data.bytes.into_owned(),
                )
                .ok_or_else(|| {
                    CompanionError::Bridge(
                        "Failed to construct RgbaImage from clipboard bytes".into(),
                    )
                }),
                Err(e) => {
                    if let Some(img) = read_clipboard_image_platform_fallback() {
                        Ok(img)
                    } else {
                        Err(CompanionError::Bridge(format!(
                            "Failed to get image from clipboard: {e}"
                        )))
                    }
                }
            }
        })
        .await;

        match res {
            Ok(Ok(img_data)) => return Ok(img_data),
            Ok(Err(e)) => {
                tracing::debug!(
                    "Async clipboard read attempt {}/{} failed: {}",
                    attempt + 1,
                    max_attempts,
                    e
                );
                last_err = Some(e.to_string());
            }
            Err(join_err) => {
                last_err = Some(join_err.to_string());
            }
        }
    }
    Err(CompanionError::Bridge(format!(
        "Failed to read image from clipboard after {max_attempts} attempts (last error: {})",
        last_err.unwrap_or_else(|| "unknown".into())
    )))
}

/// Write image to system clipboard asynchronously via spawn_blocking
pub async fn write_clipboard_image_retry_async(
    img: &RgbaImage,
    max_attempts: usize,
    delay: std::time::Duration,
) -> Result<()> {
    let mut last_err = None;
    let raw_bytes = img.as_raw().clone();
    let (w, h) = (img.width() as usize, img.height() as usize);
    for attempt in 0..max_attempts {
        if attempt > 0 {
            tokio::time::sleep(delay).await;
        }
        let bytes_clone = raw_bytes.clone();
        let res = tokio::task::spawn_blocking(move || {
            let img_data = arboard::ImageData {
                width: w,
                height: h,
                bytes: std::borrow::Cow::Borrowed(&bytes_clone),
            };
            match arboard::Clipboard::new().and_then(|mut c| c.set_image(img_data)) {
                Ok(()) => Ok(()),
                Err(e) => {
                    if let Some(r_img) = RgbaImage::from_raw(w as u32, h as u32, bytes_clone) {
                        if write_clipboard_image_platform_fallback(&r_img) {
                            return Ok(());
                        }
                    }
                    Err(CompanionError::Bridge(format!(
                        "Failed to write image to clipboard: {e}"
                    )))
                }
            }
        })
        .await;

        match res {
            Ok(Ok(())) => return Ok(()),
            Ok(Err(e)) => {
                tracing::debug!(
                    "Async clipboard write attempt {}/{} failed: {}",
                    attempt + 1,
                    max_attempts,
                    e
                );
                last_err = Some(e.to_string());
            }
            Err(join_err) => {
                last_err = Some(join_err.to_string());
            }
        }
    }
    Err(CompanionError::Bridge(format!(
        "Failed to write image to clipboard after {max_attempts} attempts (last error: {})",
        last_err.unwrap_or_else(|| "unknown".into())
    )))
}

/// Platform-specific fallback to read PNG image from system clipboard when arboard fails.
/// On Linux, supports Wayland (wl-paste) and X11 (xclip).
#[cfg(target_os = "linux")]
fn read_clipboard_image_platform_fallback() -> Option<RgbaImage> {
    // 1. Wayland clipboard: wl-paste --type image/png
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        if let Ok(output) = std::process::Command::new("wl-paste")
            .args(["--type", "image/png"])
            .output()
        {
            if output.status.success() && !output.stdout.is_empty() {
                if let Ok(img) = image::load_from_memory(&output.stdout) {
                    return Some(img.to_rgba8());
                }
            }
        }
    }

    // 2. X11 clipboard: xclip -selection clipboard -t image/png -o
    if std::env::var_os("DISPLAY").is_some() {
        if let Ok(output) = std::process::Command::new("xclip")
            .args(["-selection", "clipboard", "-t", "image/png", "-o"])
            .output()
        {
            if output.status.success() && !output.stdout.is_empty() {
                if let Ok(img) = image::load_from_memory(&output.stdout) {
                    return Some(img.to_rgba8());
                }
            }
        }
    }

    None
}

#[cfg(not(target_os = "linux"))]
fn read_clipboard_image_platform_fallback() -> Option<RgbaImage> {
    None
}

/// Platform-specific fallback to write PNG image into system clipboard when arboard fails.
/// On Linux, supports Wayland (wl-copy) and X11 (xclip).
#[cfg(target_os = "linux")]
fn write_clipboard_image_platform_fallback(img: &RgbaImage) -> bool {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut png_bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_bytes);
    if image::ImageEncoder::write_image(
        encoder,
        img.as_raw(),
        img.width(),
        img.height(),
        image::ExtendedColorType::Rgba8,
    )
    .is_err()
    {
        return false;
    }

    // 1. Wayland clipboard: wl-copy --type image/png
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        if let Ok(mut child) = Command::new("wl-copy")
            .args(["--type", "image/png"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(&png_bytes);
                drop(stdin); // Explicitly drop stdin to signal EOF so wl-copy can daemonize / copy
            }
            if let Ok(status) = child.wait() {
                if status.success() {
                    return true;
                }
            }
        }
    }

    // 2. X11 clipboard: xclip -selection clipboard -t image/png -i
    if std::env::var_os("DISPLAY").is_some() {
        if let Ok(mut child) = Command::new("xclip")
            .args(["-selection", "clipboard", "-t", "image/png", "-i"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(&png_bytes);
                drop(stdin);
            }
            if let Ok(status) = child.wait() {
                if status.success() {
                    return true;
                }
            }
        }
    }

    false
}

#[cfg(not(target_os = "linux"))]
fn write_clipboard_image_platform_fallback(_img: &RgbaImage) -> bool {
    false
}
