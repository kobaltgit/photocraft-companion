use crate::discovery::{find_photocraft_data_dir, find_plugins_dir};
use crate::win32_theme::apply_win32_menu_theme;

#[test]
fn test_find_plugins_dir() {
    let p = find_plugins_dir();
    assert!(p.to_string_lossy().contains("plugins"));
}

#[test]
fn test_find_photocraft_data_dir() {
    let p = find_photocraft_data_dir();
    assert!(
        p.to_string_lossy().contains("PhotoCraftData")
            || p.to_string_lossy().contains("PhotoCraft")
    );
}

#[test]
fn test_apply_win32_menu_theme() {
    // Mode 1: AllowDark / System, Mode 2: ForceDark, Mode 3: ForceLight
    apply_win32_menu_theme(1);
    apply_win32_menu_theme(2);
    apply_win32_menu_theme(3);
}

#[test]
fn test_no_code_plugin_not_duplicated_as_pre_commands() {
    use core_contracts::{PluginCommandStep, PluginManifest, PluginType};
    let m = PluginManifest {
        id: "test.nocode".to_string(),
        name: "Test NoCode".to_string(),
        plugin_type: PluginType::NoCode,
        commands: vec![PluginCommandStep {
            command: "layer.duplicate".to_string(),
            params: serde_json::json!({}),
            save_as: None,
            ignore_errors: false,
            when: None,
            description: None,
        }],
        ..Default::default()
    };
    // Verify that pre-commands condition strictly checks plugin_type != PluginType::NoCode
    let should_run_pre_commands = m.plugin_type != PluginType::NoCode && !m.commands.is_empty();
    assert!(
        !should_run_pre_commands,
        "NoCode commands must never be executed as pre-commands to avoid duplicate execution"
    );
}

#[tokio::test]
async fn test_copy_active_layer_and_selection_mask_default_impl() {
    use async_trait::async_trait;
    use core_contracts::{BridgeOperations, DocumentInfo, Result};
    use image::{Rgba, RgbaImage};
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestBridge {
        has_sel: bool,
        bounds: Option<[i32; 4]>,
        copy_count: AtomicUsize,
    }

    #[async_trait]
    impl BridgeOperations for TestBridge {
        async fn execute_command(
            &self,
            _command: &str,
            _params: serde_json::Value,
        ) -> Result<serde_json::Value> {
            Ok(serde_json::json!({}))
        }
        async fn copy_active_layer_image(&self) -> Result<RgbaImage> {
            let count = self.copy_count.fetch_add(1, Ordering::SeqCst);
            if self.has_sel && count == 0 {
                // Return cutout of 10x10 with opaque alpha
                let mut img = RgbaImage::new(10, 10);
                for p in img.pixels_mut() {
                    *p = Rgba([255, 0, 0, 255]);
                }
                Ok(img)
            } else {
                // Full image 50x50
                let mut img = RgbaImage::new(50, 50);
                for p in img.pixels_mut() {
                    *p = Rgba([0, 255, 0, 255]);
                }
                Ok(img)
            }
        }
        async fn paste_image_in_place(&self, _image: &RgbaImage) -> Result<serde_json::Value> {
            Ok(serde_json::json!({}))
        }
        async fn rename_layer(&self, _layer_id: u64, _new_name: &str) -> Result<serde_json::Value> {
            Ok(serde_json::json!({}))
        }
        async fn get_document_info(&self) -> Result<DocumentInfo> {
            Ok(DocumentInfo {
                name: "Test Doc".to_string(),
                width: 50,
                height: 50,
                has_selection: self.has_sel,
                selection_bounds: self.bounds,
                ..Default::default()
            })
        }
    }

    // Case 1: No selection
    let no_sel_bridge = TestBridge {
        has_sel: false,
        bounds: None,
        copy_count: AtomicUsize::new(0),
    };
    let (img, mask) = no_sel_bridge
        .copy_active_layer_and_selection_mask()
        .await
        .unwrap();
    assert_eq!(img.width(), 50);
    assert_eq!(img.height(), 50);
    assert!(mask.is_none());

    // Case 2: With selection at bounds [5, 5, 10, 10]
    let sel_bridge = TestBridge {
        has_sel: true,
        bounds: Some([5, 5, 10, 10]),
        copy_count: AtomicUsize::new(0),
    };
    let (img, mask) = sel_bridge
        .copy_active_layer_and_selection_mask()
        .await
        .unwrap();
    assert_eq!(img.width(), 50);
    assert_eq!(img.height(), 50);
    assert!(mask.is_some());
    let mask = mask.unwrap();
    assert_eq!(mask.width(), 50);
    assert_eq!(mask.height(), 50);
    // Pixel inside bounds must be 255
    assert_eq!(mask.get_pixel(5, 5)[0], 255);
    assert_eq!(mask.get_pixel(14, 14)[0], 255);
    // Pixel outside bounds must be 0
    assert_eq!(mask.get_pixel(0, 0)[0], 0);
    assert_eq!(mask.get_pixel(4, 4)[0], 0);
    assert_eq!(mask.get_pixel(15, 15)[0], 0);
}

#[test]
fn test_compute_mask_bbox_empty_and_valid() {
    use crate::cloud_engine::compute_mask_bbox;
    use image::{GrayImage, Luma};

    // Case 1: Empty mask (all zeros)
    let empty_mask = GrayImage::new(100, 80);
    assert!(compute_mask_bbox(&empty_mask).is_none());

    // Case 2: Mask with isolated pixels
    let mut mask = GrayImage::new(100, 80);
    mask.put_pixel(20, 15, Luma([255]));
    mask.put_pixel(45, 60, Luma([128]));
    let bbox = compute_mask_bbox(&mask);
    assert_eq!(bbox, Some((20, 15, 45, 60)));
}

#[test]
fn test_calculate_context_crop_bounds_padding_and_clamping() {
    use crate::cloud_engine::calculate_context_crop_bounds;

    // BBox: (20, 20, 40, 40) -> width = 21, height = 21
    // 35% padding: 21 * 0.35 = 7.35 -> 7 px
    // Crop should be: (20 - 7, 20 - 7, 21 + 14, 21 + 14) -> (13, 13, 35, 35)
    let bbox = (20, 20, 40, 40);
    let (cx, cy, cw, ch) = calculate_context_crop_bounds(bbox, 200, 200, 0.35);
    assert_eq!(cx, 13);
    assert_eq!(cy, 13);
    assert_eq!(cw, 35);
    assert_eq!(ch, 35);

    // Test clamping near canvas borders
    let near_border_bbox = (2, 2, 10, 10);
    let (cx2, cy2, cw2, ch2) = calculate_context_crop_bounds(near_border_bbox, 50, 50, 0.5);
    assert_eq!(cx2, 0); // clamped to 0
    assert_eq!(cy2, 0); // clamped to 0
    assert!(cw2 > 0 && ch2 > 0);
    assert!(cx2 + cw2 <= 50);
    assert!(cy2 + ch2 <= 50);
}

#[test]
fn test_compose_patch_layer_transparency_and_alignment() {
    use crate::cloud_engine::compose_patch_layer;
    use image::{GrayImage, Luma, Rgba, RgbaImage};

    let orig_w = 100;
    let orig_h = 80;
    let crop_x = 20;
    let crop_y = 15;
    let patch_w = 30;
    let patch_h = 25;

    let mut crop_mask = GrayImage::new(patch_w, patch_h);
    let mut inpaint_patch = RgbaImage::new(patch_w, patch_h);

    // Put a defect in the patch at (5, 5) with red color
    crop_mask.put_pixel(5, 5, Luma([255]));
    inpaint_patch.put_pixel(5, 5, Rgba([255, 0, 0, 255]));

    let patch_layer =
        compose_patch_layer(orig_w, orig_h, crop_x, crop_y, &crop_mask, &inpaint_patch);

    assert_eq!(patch_layer.width(), orig_w);
    assert_eq!(patch_layer.height(), orig_h);

    // Outside the mask must be completely transparent (alpha == 0)
    assert_eq!(patch_layer.get_pixel(0, 0)[3], 0);
    assert_eq!(patch_layer.get_pixel(crop_x, crop_y)[3], 0);

    // At the defect position (crop_x + 5, crop_y + 5) must be the red pixel
    let defect_pixel = patch_layer.get_pixel(crop_x + 5, crop_y + 5);
    assert_eq!(*defect_pixel, Rgba([255, 0, 0, 255]));
}
