use core_contracts::{AiPipelineConfig, CompanionError, NormalizationMode, Result};
use image::{GrayImage, RgbaImage};
use ort::session::Session;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub(crate) type SessionKey = (PathBuf, std::time::SystemTime, bool);

pub(crate) struct SessionCache {
    pub(crate) entries:
        std::collections::HashMap<SessionKey, (Arc<Mutex<Session>>, std::time::Instant)>,
    pub(crate) max_capacity: usize,
}

/// Thread-safe runtime for ONNX sessions with adaptive DirectML/CPU fallback and LRU session cache
pub struct AiEngine {
    pub model_dir: PathBuf,
    pub(crate) cpu_fallback_models: Mutex<Vec<PathBuf>>,
    pub(crate) session_cache: Mutex<SessionCache>,
}

impl AiEngine {
    pub fn new(model_dir: impl AsRef<Path>) -> Self {
        Self {
            model_dir: model_dir.as_ref().to_path_buf(),
            cpu_fallback_models: Mutex::new(Vec::new()),
            session_cache: Mutex::new(SessionCache {
                entries: std::collections::HashMap::new(),
                max_capacity: 3,
            }),
        }
    }

    /// Resolve absolute or relative path to a model file across plugin dirs, app cache, and global dirs
    pub fn resolve_model_path(
        &self,
        model_name: &str,
        plugin_dir: Option<&Path>,
    ) -> Option<PathBuf> {
        // Enforce strict model isolation: model name must be a pure filename without path traversal
        if model_name.contains("..") || model_name.contains('/') || model_name.contains('\\') {
            return None;
        }

        // 1. Check inside plugin directory: plugin_dir/models/<name> or plugin_dir/<name>
        if let Some(p_dir) = plugin_dir {
            let direct = p_dir.join("models").join(model_name);
            if direct.exists() {
                return Some(direct);
            }
            let flat = p_dir.join(model_name);
            if flat.exists() {
                return Some(flat);
            }
        }

        // 2. Direct path inside configured model_dir / PhotoCraftData
        let in_models = self.model_dir.join("models").join(model_name);
        if in_models.exists() {
            return Some(in_models);
        }
        let direct = self.model_dir.join(model_name);
        if direct.exists() {
            return Some(direct);
        }

        // 3. Check relative to plugins/ directory in CWD or exe location
        let in_plugins = PathBuf::from("plugins").join("models").join(model_name);
        if in_plugins.exists() {
            return Some(in_plugins);
        }

        if let Ok(exe) = std::env::current_exe() {
            if let Some(parent) = exe.parent() {
                let candidate = parent.join("models").join(model_name);
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }

        // 4. Fallback to PhotoCraft portable models directory
        let portable = PathBuf::from("PhotoCraftData")
            .join("models")
            .join(model_name);
        if portable.exists() {
            return Some(portable);
        }

        None
    }

    /// Retrieve an existing session from LRU cache or create and cache a new one
    pub fn get_or_create_session(
        &self,
        model_path: &Path,
        force_cpu: bool,
    ) -> Result<Arc<Mutex<Session>>> {
        let modified = std::fs::metadata(model_path)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let key = (model_path.to_path_buf(), modified, force_cpu);

        {
            let mut cache = self.session_cache.lock().unwrap_or_else(|e| e.into_inner());
            if let Some((sess, touch)) = cache.entries.get_mut(&key) {
                *touch = std::time::Instant::now();
                return Ok(sess.clone());
            }
        }

        // Create new session outside of cache lock
        let new_session = self.create_session(model_path, force_cpu)?;
        let sess_arc = Arc::new(Mutex::new(new_session));

        let mut cache = self.session_cache.lock().unwrap_or_else(|e| e.into_inner());
        if cache.entries.len() >= cache.max_capacity {
            if let Some((oldest_key, _)) = cache
                .entries
                .iter()
                .min_by_key(|(_, (_, touch))| *touch)
                .map(|(k, _)| (k.clone(), ()))
            {
                cache.entries.remove(&oldest_key);
            }
        }
        cache
            .entries
            .insert(key, (sess_arc.clone(), std::time::Instant::now()));

        Ok(sess_arc)
    }

    /// Remove a model session from LRU cache upon failure or fallback
    pub fn invalidate_session(&self, model_path: &Path, force_cpu: bool) {
        let modified = std::fs::metadata(model_path)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        let key = (model_path.to_path_buf(), modified, force_cpu);
        let mut cache = self.session_cache.lock().unwrap_or_else(|e| e.into_inner());
        cache.entries.remove(&key);
    }

    /// Load ONNX session with DirectML or CPU execution providers
    pub fn create_session(&self, model_path: &Path, force_cpu: bool) -> Result<Session> {
        let _is_cpu_only = force_cpu
            || self
                .cpu_fallback_models
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .contains(&model_path.to_path_buf());

        let builder = Session::builder().map_err(|e| {
            CompanionError::AiEngine(format!("Failed to init Session builder: {e}"))
        })?;

        #[cfg(windows)]
        let mut builder = if !_is_cpu_only {
            match builder.with_execution_providers([
                ort::ep::DirectML::default().build(),
                ort::ep::CPU::default().build(),
            ]) {
                Ok(b) => {
                    tracing::debug!("Configured DirectML + CPU execution provider");
                    b
                }
                Err(e) => {
                    tracing::warn!("DirectML provider setup failed ({e}), falling back to CPU");
                    Session::builder()
                        .map_err(|e| {
                            CompanionError::AiEngine(format!("Failed to re-init builder: {e}"))
                        })?
                        .with_execution_providers([ort::ep::CPU::default().build()])
                        .map_err(|e| CompanionError::AiEngine(format!("CPU provider error: {e}")))?
                }
            }
        } else {
            builder
                .with_execution_providers([ort::ep::CPU::default().build()])
                .map_err(|e| CompanionError::AiEngine(format!("CPU provider error: {e}")))?
        };

        #[cfg(not(windows))]
        let mut builder = builder
            .with_execution_providers([ort::ep::CPU::default().build()])
            .map_err(|e| CompanionError::AiEngine(format!("CPU provider error: {e}")))?;

        builder.commit_from_file(model_path).map_err(|e| {
            CompanionError::AiEngine(format!(
                "Failed to load ONNX model from {}: {e}",
                model_path.display()
            ))
        })
    }

    /// Paradigm 1: Image-to-Image (Denoise, Deblur, Colorization, 1:1 Filters)
    pub fn execute_image_to_image(
        &self,
        model_path: &Path,
        img: &RgbaImage,
        config: &AiPipelineConfig,
    ) -> Result<RgbaImage> {
        let tile_size = config.tile_size.unwrap_or(512);
        let tile_pad = config.tile_pad.unwrap_or(16);

        self.run_tiled_rgb_inference(
            model_path,
            img,
            1,
            tile_size,
            tile_pad,
            config
                .normalize
                .as_ref()
                .unwrap_or(&NormalizationMode::ZeroToOne),
        )
    }

    /// Paradigm 2: Super-Resolution (2x, 4x Upscaling via Real-ESRGAN etc.)
    pub fn execute_super_resolution(
        &self,
        model_path: &Path,
        img: &RgbaImage,
        config: &AiPipelineConfig,
    ) -> Result<RgbaImage> {
        let scale = config.scale.unwrap_or(2);
        let tile_size = config.tile_size.unwrap_or(512);
        let tile_pad = config.tile_pad.unwrap_or(16);

        self.run_tiled_rgb_inference(
            model_path,
            img,
            scale,
            tile_size,
            tile_pad,
            config
                .normalize
                .as_ref()
                .unwrap_or(&NormalizationMode::ZeroToOne),
        )
    }

    /// Paradigm 3: Inpainting (LaMa Generative Fill / Object Eraser)
    pub fn execute_inpainting(
        &self,
        model_path: &Path,
        img: &RgbaImage,
        mask: &GrayImage,
        config: &AiPipelineConfig,
    ) -> Result<RgbaImage> {
        let norm_mode = config
            .normalize
            .as_ref()
            .unwrap_or(&NormalizationMode::ZeroToOne);
        let tile_size = config.tile_size.unwrap_or(512);
        let tile_pad = config.tile_pad.unwrap_or(32);

        self.run_adaptive_inpaint(model_path, img, mask, tile_size, tile_pad, norm_mode)
    }

    /// Paradigm 4: Segmentation Mask / Depth Map (RMBG, Depth Anything V2)
    pub fn execute_segmentation(
        &self,
        model_path: &Path,
        img: &RgbaImage,
        config: &AiPipelineConfig,
    ) -> Result<GrayImage> {
        let norm_mode = config
            .normalize
            .as_ref()
            .unwrap_or(&NormalizationMode::Centered);
        let input_size = config.input_size.unwrap_or(1024);
        let is_grayscale_layer = config
            .output_type
            .as_ref()
            .map(|t| *t == core_contracts::AiOutputType::GrayscaleLayer)
            .unwrap_or(false);
        self.run_segmentation_inference(model_path, img, norm_mode, input_size, is_grayscale_layer)
    }
}
