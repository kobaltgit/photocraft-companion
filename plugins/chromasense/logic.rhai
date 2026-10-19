// ChromaSense A11y - Dynamic Color Harmony & Vision Simulation Engine (Rhai)

fn calculate_relative_luminance(r, g, b) {
    let rs = r / 255.0;
    let gs = g / 255.0;
    let bs = b / 255.0;
    return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs;
}

// Common diagnostic and accessibility recalculation (semantic name, CVD preview swatch, WCAG collision)
fn recalculate_diagnostics(state) {
    let target = state["edit_target"];
    if target == () || target == "" {
        target = "color_fg";
        state["edit_target"] = target;
    }

    let fg_hex = state["color_fg"];
    let bg_hex = state["color_bg"];
    if fg_hex == () || fg_hex == "" {
        fg_hex = "#2563eb";
        state["color_fg"] = fg_hex;
    }
    if bg_hex == () || bg_hex == "" {
        bg_hex = "#ffffff";
        state["color_bg"] = bg_hex;
    }

    let fg_rgb = hex_to_rgb(fg_hex);
    let bg_rgb = hex_to_rgb(bg_hex);

    // Active color components for semantic naming & CVD simulation
    let active_hex = state[target];
    if active_hex == () || active_hex == "" {
        active_hex = fg_hex;
        state[target] = active_hex;
    }
    let active_rgb = hex_to_rgb(active_hex);
    let r_active = active_rgb["r"];
    let g_active = active_rgb["g"];
    let b_active = active_rgb["b"];

    // 1. Determine semantic color name from 140+ shade database
    let color_name = get_named_color(r_active, g_active, b_active);
    state["semantic_color_name"] = color_name;

    // 2. Vision deficiency profile (0: Normal, 1: Deuteranopia, 2: Protanopia, 3: Tritanopia)
    let profile_idx = state["vision_profile"];
    if profile_idx == () {
        profile_idx = 0;
    }

    // 3. Color vision deficiency simulation preview swatch for active color
    state["color_sim"] = simulate_cvd(r_active, g_active, b_active, profile_idx);

    // 4. Relative luminance & contrast ratio calculation between FG and BG
    let contrast_ratio = calculate_contrast(fg_rgb["r"], fg_rgb["g"], fg_rgb["b"], bg_rgb["r"], bg_rgb["g"], bg_rgb["b"]);
    let ratio_str = ((contrast_ratio * 10.0).round() / 10.0).to_string();

    // 5. Concise accessibility and collision analysis
    // Colors are evaluated symmetrically: compare simulated perception of FG against simulated perception of BG
    if profile_idx == 0 {
        // Normal Vision (WCAG 2.1 criteria)
        if contrast_ratio >= 4.5 {
            state["collision_warning"] = t("WCAG AAA: High contrast [" + ratio_str + ":1]", "Доступность WCAG AAA: Высокий контраст [" + ratio_str + ":1]");
            state["collision_style"] = "success";
        } else if contrast_ratio >= 3.0 {
            state["collision_warning"] = t("WCAG AA: Adequate contrast [" + ratio_str + ":1]", "Доступность WCAG AA: Базовый контраст [" + ratio_str + ":1]");
            state["collision_style"] = "success";
        } else {
            state["collision_warning"] = t("WCAG Notice: Low contrast [" + ratio_str + ":1] - indistinguishable on background", "Внимание WCAG: Низкий контраст [" + ratio_str + ":1] - неразличимо на фоне");
            state["collision_style"] = "warning";
        }
    } else {
        // Simulated Color Vision Deficiency perception
        let fg_sim = hex_to_rgb(simulate_cvd(fg_rgb["r"], fg_rgb["g"], fg_rgb["b"], profile_idx));
        let bg_sim = hex_to_rgb(simulate_cvd(bg_rgb["r"], bg_rgb["g"], bg_rgb["b"], profile_idx));

        let sim_ratio = calculate_contrast(fg_sim["r"], fg_sim["g"], fg_sim["b"], bg_sim["r"], bg_sim["g"], bg_sim["b"]);
        let sim_ratio_str = ((sim_ratio * 10.0).round() / 10.0).to_string();

        let profile_name_en = "Deuteranopia";
        let profile_name_ru = "Дейтеранопия";
        if profile_idx == 2 {
            profile_name_en = "Protanopia";
            profile_name_ru = "Протанопия";
        } else if profile_idx == 3 {
            profile_name_en = "Tritanopia";
            profile_name_ru = "Тританопия";
        }

        if sim_ratio < 3.0 {
            // Contrast below WCAG 2.1 minimum threshold
            if contrast_ratio >= 3.0 {
                // Was adequate for normal vision, but deficiency causes severe color collision
                state["collision_warning"] = t(profile_name_en + " Alert: Color collision [" + sim_ratio_str + ":1] - colors blend together", profile_name_ru + ": Коллизия оттенков [" + sim_ratio_str + ":1] - цвета сливаются для зрителя");
                state["collision_style"] = "warning";
            } else {
                // Low contrast even in normal vision
                state["collision_warning"] = t(profile_name_en + ": Low contrast [" + sim_ratio_str + ":1] - indistinguishable on background", profile_name_ru + ": Низкий контраст [" + sim_ratio_str + ":1] - неразличимо на фоне");
                state["collision_style"] = "warning";
            }
        } else if sim_ratio < 4.5 {
            state["collision_warning"] = t(profile_name_en + " OK: WCAG AA Adequate contrast [" + sim_ratio_str + ":1]", profile_name_ru + ": WCAG AA Базовый контраст [" + sim_ratio_str + ":1]");
            state["collision_style"] = "success";
        } else {
            state["collision_warning"] = t(profile_name_en + " OK: WCAG AAA High contrast [" + sim_ratio_str + ":1]", profile_name_ru + ": WCAG AAA Высокий контраст [" + sim_ratio_str + ":1]");
            state["collision_style"] = "success";
        }
    }

    return state;
}

// Universal hook invoked when PhotoCraft active session colors change externally
fn on_session_colors(fg_hex, bg_hex, state) {
    // 1. Update color slots
    state["color_fg"] = fg_hex;
    state["color_bg"] = bg_hex;

    // 2. Synchronize RGB sliders to current edit_target
    let target = state["edit_target"];
    if target == () || target == "" {
        target = "color_fg";
        state["edit_target"] = target;
    }
    let active_hex = state[target];
    let rgb = hex_to_rgb(active_hex);
    state["red"] = rgb["r"];
    state["green"] = rgb["g"];
    state["blue"] = rgb["b"];
    state["last_synced_hex"] = active_hex;

    // 3. Recalculate diagnostics & accessibility
    state = recalculate_diagnostics(state);

    // Note: We DO NOT call execute_command("tools.setColors") here to avoid echo loops!
    return state;
}

// Reactive hook invoked whenever a widget value or action is modified in the UI
fn on_state_changed(state) {
    // 1. Resolve current active editing target (default to color_fg)
    let target = state["edit_target"];
    if target == () || target == "" {
        target = "color_fg";
        state["edit_target"] = target;
    }

    // Ensure valid color_fg and color_bg initial values
    if state["color_fg"] == () || state["color_fg"] == "" {
        state["color_fg"] = "#2563eb";
    }
    if state["color_bg"] == () || state["color_bg"] == "" {
        state["color_bg"] = "#ffffff";
    }

    // 2. Process Swap Colors action
    if state["action_swap_colors"] == true {
        state["action_swap_colors"] = false;
        let temp = state["color_fg"];
        state["color_fg"] = state["color_bg"];
        state["color_bg"] = temp;

        // Force synchronization of RGB sliders to current active target
        let target_hex = state[target];
        let target_rgb = hex_to_rgb(target_hex);
        state["red"] = target_rgb["r"];
        state["green"] = target_rgb["g"];
        state["blue"] = target_rgb["b"];
        state["last_edit_target"] = target;
        state["last_synced_hex"] = target_hex;
    } else if state["last_edit_target"] != target {
        // Target slot was switched (e.g. clicked on Active Color or Background)
        let target_hex = state[target];
        let target_rgb = hex_to_rgb(target_hex);
        state["red"] = target_rgb["r"];
        state["green"] = target_rgb["g"];
        state["blue"] = target_rgb["b"];
        state["last_edit_target"] = target;
        state["last_synced_hex"] = target_hex;
    } else {
        // Sliders or direct text edit: check if sliders changed
        let r_slider = state["red"].round();
        let g_slider = state["green"].round();
        let b_slider = state["blue"].round();
        let current_slider_hex = rgb_to_hex(r_slider, g_slider, b_slider);

        if current_slider_hex != state["last_synced_hex"] {
            // Slider moved -> update active target slot
            state[target] = current_slider_hex;
            state["last_synced_hex"] = current_slider_hex;
        } else if state[target] != state["last_synced_hex"] {
            // Target slot text edit -> update sliders
            let rgb_parsed = hex_to_rgb(state[target]);
            state["red"] = rgb_parsed["r"];
            state["green"] = rgb_parsed["g"];
            state["blue"] = rgb_parsed["b"];
            state["last_synced_hex"] = state[target];
        }
    }

    // 3. Recalculate diagnostics & accessibility
    state = recalculate_diagnostics(state);

    // 4. Synchronize foreground and background brush colors in active PhotoCraft document
    let fg_hex = state["color_fg"];
    let bg_hex = state["color_bg"];
    execute_command("tools.setColors", #{
        foreground: fg_hex,
        background: bg_hex
    });

    return state;
}
