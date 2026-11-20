use core_contracts::{CompanionError, Result};
use std::path::PathBuf;
use std::sync::OnceLock;

static ORT_INIT_ERROR: OnceLock<Option<String>> = OnceLock::new();

/// Detect whether the host CPU supports AVX2 vector instructions
pub fn is_avx2_supported() -> bool {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        is_x86_feature_detected!("avx2")
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        false
    }
}

/// Detect whether the host CPU supports baseline AVX (AVX1) instructions
pub fn is_avx_supported() -> bool {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        is_x86_feature_detected!("avx")
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        false
    }
}

/// Find onnxruntime shared library across common search paths
pub fn find_onnxruntime_dll() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    let lib_name = "onnxruntime.dll";
    #[cfg(target_os = "linux")]
    let lib_name = "libonnxruntime.so";
    #[cfg(target_os = "macos")]
    let lib_name = "libonnxruntime.dylib";

    // 1. Next to the current executable
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let candidate = parent.join(lib_name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    // 2. Relative to current working directory
    let cwd_candidate = PathBuf::from(lib_name);
    if cwd_candidate.exists() {
        return Some(cwd_candidate);
    }

    // 3. In PhotoCraftData directory
    let data_candidate = PathBuf::from("PhotoCraftData").join(lib_name);
    if data_candidate.exists() {
        return Some(data_candidate);
    }

    // 4. In target/release or target/debug for tests and local dev
    let target_rel = PathBuf::from("target").join("release").join(lib_name);
    if target_rel.exists() {
        return Some(target_rel);
    }
    let target_deb = PathBuf::from("target").join("debug").join(lib_name);
    if target_deb.exists() {
        return Some(target_deb);
    }

    None
}

/// Thread-safe one-time dynamic initialization of ONNX Runtime
pub fn ensure_ort_initialized() -> Result<()> {
    let err_opt = ORT_INIT_ERROR.get_or_init(|| {
        let avx2 = is_avx2_supported();
        let avx = is_avx_supported();
        tracing::info!("Host CPU capabilities: AVX={}, AVX2={}", avx, avx2);

        if let Some(dll_path) = find_onnxruntime_dll() {
            tracing::info!(
                "Loading ONNX Runtime dynamically from: {}",
                dll_path.display()
            );
            match ort::init_from(&dll_path) {
                Ok(builder) => {
                    let _ = builder.with_name("photocraft_companion").commit();
                    None
                }
                Err(e) => Some(format!(
                    "Failed to load ONNX dynamic library from {}: {e}",
                    dll_path.display()
                )),
            }
        } else {
            tracing::info!(
                "No local ONNX Runtime library found on disk, falling back to system loader"
            );
            let _ = ort::init().with_name("photocraft_companion").commit();
            None
        }
    });

    if let Some(err) = err_opt {
        Err(CompanionError::AiEngine(err.clone()))
    } else {
        Ok(())
    }
}
