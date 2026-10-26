use image::{Rgba, RgbaImage};
use raster_engine::{
    calculate_content_bounds, calculate_preview_scale, render_extrude_2_5d, render_extrude_preview,
    render_extrude_split, ExtrudeOutputMode, ExtrudeParams,
};

fn create_test_text_badge(width: u32, height: u32) -> RgbaImage {
    let mut img = RgbaImage::new(width, height);
    // Draw a 20x20 solid badge in center
    for y in (height / 4)..(height * 3 / 4) {
        for x in (width / 4)..(width * 3 / 4) {
            img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }
    img
}

#[test]
fn test_calculate_preview_scale() {
    assert_eq!(calculate_preview_scale(256, 256, 512), 1.0);
    assert_eq!(calculate_preview_scale(1024, 512, 512), 0.5);
    assert_eq!(calculate_preview_scale(500, 2000, 500), 0.25);
}

#[test]
fn test_extrude_params_scaling() {
    let mut params = ExtrudeParams::default();
    params.distance = 20.0;
    params.bevel_radius = 4.0;
    params.shadow_blur = 10.0;
    params.shadow_distance = 6.0;

    let scaled = params.scaled(0.5);
    assert_eq!(scaled.distance, 10.0);
    assert_eq!(scaled.bevel_radius, 2.0);
    assert_eq!(scaled.shadow_blur, 5.0);
    assert_eq!(scaled.shadow_distance, 3.0);
}

#[test]
fn test_render_extrude_2_5d_merged() {
    let src = create_test_text_badge(64, 64);
    let mut params = ExtrudeParams::default();
    params.distance = 8.0;
    params.angle_deg = 90.0; // straight down

    let out = render_extrude_2_5d(&src, &params);
    assert_eq!(out.dimensions(), (64, 64));

    // Check that pixels below original badge now have extruded color
    let original_bottom = 64 * 3 / 4;
    let extruded_y = original_bottom + 2;
    if extruded_y < 64 {
        let px = out.get_pixel(32, extruded_y);
        assert!(
            px[3] > 0,
            "Extruded pixel at (32, {extruded_y}) should have alpha > 0"
        );
    }
}

#[test]
fn test_render_extrude_split_layers() {
    let src = create_test_text_badge(64, 64);
    let mut params = ExtrudeParams::default();
    params.distance = 6.0;
    params.angle_deg = 45.0;
    params.output_mode = ExtrudeOutputMode::SplitLayers;

    let (front, body, shadow) = render_extrude_split(&src, &params);

    assert_eq!(front.dimensions(), (64, 64));
    assert_eq!(body.dimensions(), (64, 64));
    assert!(shadow.is_some());
    let sh = shadow.unwrap();
    assert_eq!(sh.dimensions(), (64, 64));

    // Front should have same center badge
    assert_eq!(front.get_pixel(32, 32)[3], 255);
}

#[test]
fn test_calculate_content_bounds_and_render_preview() {
    let mut img = RgbaImage::new(100, 100);
    // Draw 10x10 square at (20, 30)
    for y in 30..40 {
        for x in 20..30 {
            img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
        }
    }

    let bounds = calculate_content_bounds(&img).expect("Should find bounds");
    assert_eq!(bounds, (20, 30, 29, 39));

    let mut params = ExtrudeParams::default();
    params.distance = 15.0;
    params.angle_deg = 45.0;

    let preview = render_extrude_preview(&img, &params);
    // Preview canvas should be expanded to include content + extrusion padding
    let (pw, ph) = preview.dimensions();
    assert!(pw > 10, "Preview width should be expanded");
    assert!(ph > 10, "Preview height should be expanded");

    // Non-transparent pixels must exist in preview
    let has_alpha = preview.pixels().any(|p| p[3] > 0);
    assert!(has_alpha, "Preview must contain rendered pixels");
}

#[test]
fn test_shadow_no_accumulator_leak_and_disabled_bypass() {
    let mut img = RgbaImage::new(100, 100);
    // Draw 10x10 badge in center (45..55, 45..55)
    for y in 45..55 {
        for x in 45..55 {
            img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }

    // 1. With shadow disabled (opacity = 0.0), render_drop_shadow must return None
    let sh_disabled = raster_engine::shadow::render_drop_shadow(&img, 10.0, 10.0, 12.0, 0.0);
    assert!(
        sh_disabled.is_none(),
        "Disabled shadow must return None without calculating"
    );

    // 2. With shadow enabled, distant corners must remain strictly 0 (no accumulator drift)
    let sh = raster_engine::shadow::render_drop_shadow(&img, 5.0, 5.0, 6.0, 0.5)
        .expect("Shadow should be computed");
    assert_eq!(
        sh.get_pixel(0, 0)[3],
        0,
        "Corner (0,0) must have alpha 0, no box blur bleed"
    );
    assert_eq!(sh.get_pixel(99, 0)[3], 0, "Corner (99,0) must have alpha 0");
    assert_eq!(sh.get_pixel(0, 99)[3], 0, "Corner (0,99) must have alpha 0");
    assert_eq!(
        sh.get_pixel(99, 99)[3],
        0,
        "Corner (99,99) must have alpha 0"
    );

    // Pixels near center + offset (55, 55) must have positive alpha
    assert!(
        sh.get_pixel(52, 52)[3] > 0,
        "Shadow near center must have positive alpha"
    );
}

#[test]
fn test_perspective_extrusion_tapering() {
    let mut img = RgbaImage::new(100, 100);
    // Draw 40x40 square centered at (30..70, 30..70)
    for y in 30..70 {
        for x in 30..70 {
            img.put_pixel(x, y, Rgba([255, 255, 255, 255]));
        }
    }

    let mut params_iso = ExtrudeParams::default();
    params_iso.distance = 20.0;
    params_iso.angle_deg = 90.0; // straight down
    params_iso.perspective = 0.0;
    let (_, body_iso, _) = render_extrude_split(&img, &params_iso);

    let mut params_persp = ExtrudeParams::default();
    params_persp.distance = 20.0;
    params_persp.angle_deg = 90.0;
    params_persp.perspective = 0.5; // 50% taper
    let (_, body_persp, _) = render_extrude_split(&img, &params_persp);

    // At the bottom of extrusion (y = 85), isometric extrusion will have width = 40 (x = 30..70)
    // while perspective extrusion with 50% taper will have narrower width!
    let iso_has_outer_px = body_iso.get_pixel(31, 85)[3] > 0;
    let persp_has_outer_px = body_persp.get_pixel(31, 85)[3] > 0;

    assert!(
        iso_has_outer_px,
        "Isometric body must reach outer x=31 at y=85"
    );
    assert!(
        !persp_has_outer_px,
        "Perspective body must taper inward and not reach outer x=31 at y=85"
    );
}
