use super::*;
use crate::plugin_adapter::bake_layer_offset_onto_canvas;
use core_contracts::{
    AiPipelineConfig, AiTask, BridgeOperations, DocumentInfo, ModelRequirement, NormalizationMode,
    Plugin, PluginManifest, PluginType,
};
use image::{GrayImage, Rgba, RgbaImage};
use serde_json::json;
use std::sync::Arc;

#[test]
fn test_generic_ai_plugin_manifest_binding() {
    let engine = Arc::new(AiEngine::new("test_models"));
    let manifest = PluginManifest {
        id: "ai.photocraft.plugins.upscale".to_string(),
        name: "AI Super-Resolution (2x)".to_string(),
        version: "1.0.0".to_string(),
        description: "2x Upscale".to_string(),
        author: "kobaltGIT".to_string(),
        plugin_type: PluginType::NativeAi,
        models: vec![ModelRequirement {
            role: Some("super_resolution".to_string()),
            name: "realesrgan-x2.onnx".to_string(),
            required: true,
            size_mb: Some(68),
            url: None,
            sha256: None,
        }],
        ai: Some(AiPipelineConfig {
            task: AiTask::SuperResolution,
            scale: Some(2),
            tile_size: Some(512),
            tile_pad: Some(16),
            normalize: Some(NormalizationMode::ZeroToOne),
            input_size: None,
            output_type: None,
        }),
        ..Default::default()
    };

    let plugin = GenericAiPlugin::new(manifest.clone(), engine);
    assert_eq!(plugin.id(), "ai.photocraft.plugins.upscale");
    assert_eq!(plugin.manifest().name, "AI Super-Resolution (2x)");
    assert_eq!(plugin.manifest().models[0].name, "realesrgan-x2.onnx");
}

#[test]
fn test_ai_engine_model_path_resolution() {
    let temp_dir = std::env::temp_dir().join(format!(
        "ai_resolve_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let models_dir = temp_dir.join("models");
    std::fs::create_dir_all(&models_dir).unwrap();
    let fake_model = models_dir.join("test_model.onnx");
    std::fs::write(&fake_model, b"fake onnx").unwrap();

    let engine = AiEngine::new(&temp_dir);

    // 1. Resolve inside engine models_dir
    let resolved = engine.resolve_model_path("test_model.onnx", None);
    assert_eq!(resolved, Some(fake_model.clone()));

    // 2. Resolve inside plugin directory
    let plugin_dir = temp_dir.join("test_plugin");
    let plugin_models = plugin_dir.join("models");
    std::fs::create_dir_all(&plugin_models).unwrap();
    let plugin_fake_model = plugin_models.join("plugin_model.onnx");
    std::fs::write(&plugin_fake_model, b"plugin onnx").unwrap();

    let resolved_p = engine.resolve_model_path("plugin_model.onnx", Some(&plugin_dir));
    assert_eq!(resolved_p, Some(plugin_fake_model.clone()));

    // 3. Resolve flat inside plugin directory
    let plugin_flat_model = plugin_dir.join("flat_model.onnx");
    std::fs::write(&plugin_flat_model, b"flat onnx").unwrap();
    let resolved_flat = engine.resolve_model_path("flat_model.onnx", Some(&plugin_dir));
    assert_eq!(resolved_flat, Some(plugin_flat_model));

    // 4. Fallback to PhotoCraftData/models in CWD
    let cwd_portable = std::path::PathBuf::from("PhotoCraftData").join("models");
    let _ = std::fs::create_dir_all(&cwd_portable);
    let portable_fake = cwd_portable.join("portable_test.onnx");
    let _ = std::fs::write(&portable_fake, b"portable onnx");

    let resolved_portable = engine.resolve_model_path("portable_test.onnx", None);
    assert_eq!(resolved_portable, Some(portable_fake.clone()));
    let _ = std::fs::remove_file(portable_fake);

    // 5. Check directory traversal / non-existent
    assert_eq!(engine.resolve_model_path("../../../etc/passwd", None), None);
    assert_eq!(
        engine.resolve_model_path("totally_nonexistent.onnx", None),
        None
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[allow(dead_code)]
#[derive(Default)]
struct MockPluginClient {
    active_image: std::sync::Mutex<Option<RgbaImage>>,
    pasted_image: std::sync::Mutex<Option<RgbaImage>>,
}

#[async_trait::async_trait]
impl BridgeOperations for MockPluginClient {
    async fn copy_active_layer_image(&self) -> core_contracts::Result<RgbaImage> {
        let lock = self.active_image.lock().unwrap();
        lock.clone()
            .ok_or_else(|| core_contracts::CompanionError::Bridge("No mock image".into()))
    }

    async fn paste_image_in_place(
        &self,
        img: &RgbaImage,
    ) -> core_contracts::Result<serde_json::Value> {
        let mut lock = self.pasted_image.lock().unwrap();
        *lock = Some(img.clone());
        Ok(json!({ "ok": true, "layer_id": 42 }))
    }

    async fn rename_layer(
        &self,
        _layer_id: u64,
        _new_name: &str,
    ) -> core_contracts::Result<serde_json::Value> {
        Ok(json!({ "ok": true }))
    }

    async fn execute_command(
        &self,
        _command: &str,
        _params: serde_json::Value,
    ) -> core_contracts::Result<serde_json::Value> {
        Ok(json!({ "ok": true }))
    }

    async fn get_document_info(&self) -> core_contracts::Result<DocumentInfo> {
        Ok(DocumentInfo::default())
    }
}

#[test]
fn test_compute_mask_bbox() {
    let mut mask = GrayImage::new(100, 100);
    assert_eq!(AiEngine::compute_mask_bbox(&mask), None);

    mask.put_pixel(10, 20, image::Luma([255]));
    mask.put_pixel(30, 40, image::Luma([255]));
    mask.put_pixel(15, 25, image::Luma([128]));

    assert_eq!(AiEngine::compute_mask_bbox(&mask), Some((10, 20, 30, 40)));
}

#[test]
fn test_tiled_boundary_clamping_coordinates() {
    let (w, h) = (1000u32, 800u32);
    let tile_size = 512u32;
    let tile_pad = 16u32;
    let step = tile_size.saturating_sub(tile_pad * 2).max(1);

    let mut visited_x = Vec::new();
    let mut x = 0;
    loop {
        let tile_x = if x + tile_size > w {
            w.saturating_sub(tile_size)
        } else {
            x
        };
        visited_x.push(tile_x);
        if tile_x + tile_size >= w {
            break;
        }
        x += step;
    }

    // Must reach the right boundary exactly
    assert_eq!(*visited_x.last().unwrap() + tile_size, w);
    // All coordinate origins must be valid
    for &tx in &visited_x {
        assert!(tx + tile_size <= w);
    }

    let mut visited_y = Vec::new();
    let mut y = 0;
    loop {
        let tile_y = if y + tile_size > h {
            h.saturating_sub(tile_size)
        } else {
            y
        };
        visited_y.push(tile_y);
        if tile_y + tile_size >= h {
            break;
        }
        y += step;
    }

    // Must reach bottom boundary exactly
    assert_eq!(*visited_y.last().unwrap() + tile_size, h);
    for &ty in &visited_y {
        assert!(ty + tile_size <= h);
    }
}

#[test]
fn test_depth_map_plugin_manifest_binding() {
    let engine = Arc::new(AiEngine::new("test_models"));
    let manifest = PluginManifest {
        id: "ai.photocraft.plugins.depth".to_string(),
        name: "Depth Anything V2".to_string(),
        version: "1.0.0".to_string(),
        description: "Monocular Depth Estimation".to_string(),
        author: "kobaltGIT".to_string(),
        plugin_type: PluginType::NativeAi,
        models: vec![ModelRequirement {
            role: Some("depth_estimation".to_string()),
            name: "depth_anything_v2_vits.onnx".to_string(),
            required: true,
            size_mb: Some(99),
            url: None,
            sha256: None,
        }],
        ai: Some(AiPipelineConfig {
            task: AiTask::SegmentationMask,
            scale: None,
            tile_size: None,
            tile_pad: None,
            normalize: Some(NormalizationMode::Centered),
            input_size: Some(518),
            output_type: Some(core_contracts::AiOutputType::GrayscaleLayer),
        }),
        ..Default::default()
    };

    let plugin = GenericAiPlugin::new(manifest.clone(), engine);
    assert_eq!(plugin.id(), "ai.photocraft.plugins.depth");
    assert_eq!(plugin.manifest().name, "Depth Anything V2");
    assert_eq!(
        plugin.manifest().models[0].role.as_deref(),
        Some("depth_estimation")
    );
    assert_eq!(
        plugin.manifest().ai.as_ref().unwrap().output_type,
        Some(core_contracts::AiOutputType::GrayscaleLayer)
    );
}

#[test]
fn test_session_cache_capacity_and_invalidation() {
    let temp_dir = std::env::temp_dir().join(format!("ai_cache_test_{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let engine = AiEngine::new(&temp_dir);

    // Initial cache should be empty
    {
        let cache = engine.session_cache.lock().unwrap();
        assert_eq!(cache.entries.len(), 0);
        assert_eq!(cache.max_capacity, 3);
    }

    // Invalidation on non-existent session is a safe no-op
    let fake_path = temp_dir.join("nonexistent.onnx");
    engine.invalidate_session(&fake_path, false);
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_bake_layer_offset_zero_offset_passthrough() {
    let img = RgbaImage::from_pixel(100, 100, Rgba([255, 0, 0, 255]));
    let baked = bake_layer_offset_onto_canvas(&img, Some([0, 0, 100, 100]), 500, 500);
    assert_eq!(baked.width(), 100);
    assert_eq!(baked.height(), 100);
}

#[test]
fn test_bake_layer_offset_expands_to_canvas_at_original_position() {
    let mut img = RgbaImage::new(10, 10);
    img.put_pixel(0, 0, Rgba([200, 100, 50, 255]));
    img.put_pixel(9, 9, Rgba([10, 20, 30, 255]));

    let baked = bake_layer_offset_onto_canvas(&img, Some([50, 80, 10, 10]), 200, 300);
    assert_eq!(baked.width(), 200);
    assert_eq!(baked.height(), 300);

    // Top-left corner must remain transparent outside the offset
    assert_eq!(baked.get_pixel(0, 0)[3], 0);
    // Shifted pixels must match the source at the layer offset
    assert_eq!(baked.get_pixel(50, 80), &Rgba([200, 100, 50, 255]));
    assert_eq!(baked.get_pixel(59, 89), &Rgba([10, 20, 30, 255]));
}

#[test]
fn test_bake_layer_offset_size_mismatch_passthrough() {
    // If output is scaled (e.g. 2x super-res 200×200 vs 100×100 bound), do not bake
    let img = RgbaImage::new(200, 200);
    let baked = bake_layer_offset_onto_canvas(&img, Some([10, 10, 100, 100]), 500, 500);
    assert_eq!(baked.width(), 200);
    assert_eq!(baked.height(), 200);
}

#[test]
fn test_bake_layer_offset_missing_bounds_passthrough() {
    let img = RgbaImage::new(50, 50);
    let baked = bake_layer_offset_onto_canvas(&img, None, 500, 500);
    assert_eq!(baked.width(), 50);
    assert_eq!(baked.height(), 50);
}

#[test]
fn test_bake_layer_offset_negative_offset_clips_safely() {
    let mut img = RgbaImage::new(20, 20);
    img.put_pixel(15, 15, Rgba([77, 88, 99, 255]));

    let baked = bake_layer_offset_onto_canvas(&img, Some([-5, -5, 20, 20]), 100, 100);
    assert_eq!(baked.width(), 100);
    assert_eq!(baked.height(), 100);
    assert_eq!(baked.get_pixel(10, 10), &Rgba([77, 88, 99, 255]));
}
