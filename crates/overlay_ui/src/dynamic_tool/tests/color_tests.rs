use super::*;

#[test]
fn test_find_nearest_named_color_perceptual_accuracy() {
    // Pure red
    assert_eq!(find_nearest_named_color(255, 0, 0, Language::En), "Red");
    assert_eq!(find_nearest_named_color(255, 0, 0, Language::Ru), "Красный");

    // Classical Ultramarine pigment
    assert_eq!(
        find_nearest_named_color(18, 10, 143, Language::En),
        "Ultramarine"
    );
    assert_eq!(
        find_nearest_named_color(18, 10, 143, Language::Ru),
        "Ультрамарин"
    );

    // Pure white
    assert_eq!(
        find_nearest_named_color(255, 255, 255, Language::En),
        "White"
    );
    assert_eq!(
        find_nearest_named_color(255, 255, 255, Language::Ru),
        "Белый"
    );

    // Pure black
    assert_eq!(find_nearest_named_color(0, 0, 0, Language::En), "Black");
    assert_eq!(find_nearest_named_color(0, 0, 0, Language::Ru), "Чёрный");
}

#[test]
fn test_simulate_cvd_color_and_hex() {
    // Normal vision
    assert_eq!(simulate_cvd_hex(255.0, 0.0, 0.0, 0), "#ff0000");

    // Deuteranopia (green blindness)
    let (dr, dg, db) = simulate_cvd_color(0.0, 255.0, 0.0, 1);
    assert!(dr > 80 && dr < 110);
    assert!(dg > 60 && dg < 90);
    assert!(db > 60 && db < 90);
    let d_hex = simulate_cvd_hex(0.0, 255.0, 0.0, 1);
    assert_eq!(d_hex.len(), 7);
    assert!(d_hex.starts_with('#'));

    // Protanopia (red blindness)
    let p_hex = simulate_cvd_hex(255.0, 0.0, 0.0, 2);
    assert_eq!(p_hex.len(), 7);
    assert!(p_hex.starts_with('#'));

    // Tritanopia (blue blindness)
    let t_hex = simulate_cvd_hex(0.0, 0.0, 255.0, 3);
    assert_eq!(t_hex.len(), 7);
    assert!(t_hex.starts_with('#'));
}

#[test]

fn test_chromasense_wcag_and_cvd_standard_compliance() {
    // Light pink #ffb4eb (255, 180, 235) on pure white #ffffff (255, 255, 255)
    let ratio_normal = calculate_wcag_contrast(255.0, 180.0, 235.0, 255.0, 255.0, 255.0);
    assert!(ratio_normal < 3.0);
    assert!((ratio_normal - 1.58).abs() < 0.1);

    // Deuteranopia (profile 1): simulated contrast must also be strictly < 3.0:1 (~1.2:1)
    let (dr1, dg1, db1) = simulate_cvd_color(255.0, 180.0, 235.0, 1);
    let (dr2, dg2, db2) = simulate_cvd_color(255.0, 255.0, 255.0, 1);
    let ratio_deut = calculate_wcag_contrast(
        dr1 as f64, dg1 as f64, db1 as f64, dr2 as f64, dg2 as f64, db2 as f64,
    );
    assert!(ratio_deut < 3.0);
    assert!((ratio_deut - 1.2).abs() < 0.2);

    // Protanopia (profile 2): simulated contrast < 3.0:1 (~1.3:1)
    let (pr1, pg1, pb1) = simulate_cvd_color(255.0, 180.0, 235.0, 2);
    let (pr2, pg2, pb2) = simulate_cvd_color(255.0, 255.0, 255.0, 2);
    let ratio_prot = calculate_wcag_contrast(
        pr1 as f64, pg1 as f64, pb1 as f64, pr2 as f64, pg2 as f64, pb2 as f64,
    );
    assert!(ratio_prot < 3.0);
    assert!((ratio_prot - 1.3).abs() < 0.2);

    // Tritanopia (profile 3): simulated contrast < 3.0:1 (~1.4:1)
    let (tr1, tg1, tb1) = simulate_cvd_color(255.0, 180.0, 235.0, 3);
    let (tr2, tg2, tb2) = simulate_cvd_color(255.0, 255.0, 255.0, 3);
    let ratio_trit = calculate_wcag_contrast(
        tr1 as f64, tg1 as f64, tb1 as f64, tr2 as f64, tg2 as f64, tb2 as f64,
    );
    assert!(ratio_trit < 3.0);
    assert!((ratio_trit - 1.4).abs() < 0.2);
}
