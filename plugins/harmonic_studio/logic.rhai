// Harmonic Color Studio — Rhai Color Harmony Engine

// Convert RGB (0..255) to HSL (h: 0..360, s: 0..100, l: 0..100)
fn rgb_to_hsl(r, g, b) {
    let rf = r.to_float() / 255.0;
    let gf = g.to_float() / 255.0;
    let bf = b.to_float() / 255.0;

    let max_v = rf;
    if gf > max_v { max_v = gf; }
    if bf > max_v { max_v = bf; }

    let min_v = rf;
    if gf < min_v { min_v = gf; }
    if bf < min_v { min_v = bf; }

    let delta = max_v - min_v;
    let l = (max_v + min_v) / 2.0;

    let h = 0.0;
    let s = 0.0;

    if delta > 0.00001 {
        if l > 0.5 {
            s = delta / (2.0 - max_v - min_v);
        } else {
            s = delta / (max_v + min_v);
        }

        if (max_v - rf).abs() < 0.00001 {
            let offset = 0.0;
            if gf < bf { offset = 6.0; }
            h = (gf - bf) / delta + offset;
        } else if (max_v - gf).abs() < 0.00001 {
            h = (bf - rf) / delta + 2.0;
        } else {
            h = (rf - gf) / delta + 4.0;
        }
        h = h * 60.0;
    }

    if h < 0.0 { h += 360.0; }
    if h >= 360.0 { h -= 360.0; }

    return #{
        h: h,
        s: s * 100.0,
        l: l * 100.0
    };
}

// Convert HSL to HEX color string using native rgb_to_hex
fn hsl_to_hex(h, s, l) {
    let norm_h = (h % 360.0 + 360.0) % 360.0;
    let norm_s = s / 100.0;
    if norm_s < 0.0 { norm_s = 0.0; }
    if norm_s > 1.0 { norm_s = 1.0; }

    let norm_l = l / 100.0;
    if norm_l < 0.0 { norm_l = 0.0; }
    if norm_l > 1.0 { norm_l = 1.0; }

    let diff = 2.0 * norm_l - 1.0;
    let abs_diff = diff;
    if abs_diff < 0.0 { abs_diff = -abs_diff; }
    let c = (1.0 - abs_diff) * norm_s;

    let hp = norm_h / 60.0;
    let mod2 = hp % 2.0;
    let sub = mod2 - 1.0;
    if sub < 0.0 { sub = -sub; }
    let x = c * (1.0 - sub);

    let m = norm_l - c / 2.0;

    let r1 = 0.0;
    let g1 = 0.0;
    let b1 = 0.0;

    if hp >= 0.0 && hp < 1.0 {
        r1 = c; g1 = x; b1 = 0.0;
    } else if hp >= 1.0 && hp < 2.0 {
        r1 = x; g1 = c; b1 = 0.0;
    } else if hp >= 2.0 && hp < 3.0 {
        r1 = 0.0; g1 = c; b1 = x;
    } else if hp >= 3.0 && hp < 4.0 {
        r1 = 0.0; g1 = x; b1 = c;
    } else if hp >= 4.0 && hp < 5.0 {
        r1 = x; g1 = 0.0; b1 = c;
    } else {
        r1 = c; g1 = 0.0; b1 = x;
    }

    let r_byte = (r1 + m) * 255.0;
    let g_byte = (g1 + m) * 255.0;
    let b_byte = (b1 + m) * 255.0;

    return rgb_to_hex(r_byte, g_byte, b_byte);
}

// Clamp helper
fn clamp_val(val, min_v, max_v) {
    if val < min_v { return min_v; }
    if val > max_v { return max_v; }
    return val;
}

// Recalculates all 5 harmony slots based on base_color, scheme_type, spread_angle, tone_depth
fn recalculate_palette(state) {
    let base_hex = state["base_color"];
    if base_hex == () || base_hex == "" {
        base_hex = "#2563eb";
        state["base_color"] = base_hex;
    }

    let rgb = hex_to_rgb(base_hex);
    let hsl = rgb_to_hsl(rgb["r"], rgb["g"], rgb["b"]);
    let h0 = hsl["h"];
    let s0 = hsl["s"];
    let l0 = hsl["l"];

    let scheme = state["scheme_type"];
    if scheme == () { scheme = 0; }

    let spread = state["spread_angle"];
    if spread == () { spread = 30.0; }

    let delta_l = state["tone_depth"];
    if delta_l == () { delta_l = 35.0; }

    // Dynamic slider enabling: Spread Angle is only applicable for Split-Complementary (1) and Analogous (3)
    state["spread_angle_enabled"] = (scheme == 1 || scheme == 3);

    // Dynamic label for nuance slot: 4th tone in Tetradic, otherwise highlight tint
    if scheme == 4 {
        state["slot_nuance_label"] = t("Tetradic Nuance (4th Tone)", "Тетраидный нюанс (4-й тон)");
    } else {
        state["slot_nuance_label"] = t("Highlight Tint", "Блик / Светлый тон");
    }

    let h_harm = h0;
    let s_harm = s0;
    let l_harm = l0;

    let h_acc = h0;
    let s_acc = s0;
    let l_acc = l0;

    let h_nuance = h0;
    let s_nuance = s0;
    let l_nuance = clamp_val(l0 + delta_l, 10.0, 95.0);

    let h_shadow = h0;
    let s_shadow = s0;
    let l_shadow = clamp_val(l0 - delta_l, 5.0, 90.0);

    let connect_mode = "none";

    if scheme == 0 {
        // Complementary: base, opposite (180 deg), opposite accent, lighter tint, deeper shade
        // Contrast & vibrant accent: crisp, bright opposite
        h_harm = h0 + 180.0;
        h_acc = h0 + 180.0;
        s_acc = clamp_val(s0 + 20.0, 20.0, 100.0);
        l_acc = clamp_val(l0 + 15.0, 20.0, 95.0);
        connect_mode = "diameter";
    } else if scheme == 1 {
        // Split-Complementary: base, opposite - spread, opposite + spread, tint, shade
        h_harm = h0 + 180.0 - spread;
        h_acc = h0 + 180.0 + spread;
        connect_mode = "spokes";
    } else if scheme == 2 {
        // Triadic: base, +120 deg, +240 deg, tint, shade
        h_harm = h0 + 120.0;
        h_acc = h0 + 240.0;
        connect_mode = "polygon";
    } else if scheme == 3 {
        // Analogous: base, -spread, +spread, tint, shade
        h_harm = h0 - spread;
        h_acc = h0 + spread;
        connect_mode = "spokes";
    } else if scheme == 4 {
        // Tetradic: balanced 4 tones (+0, +90, +180, +270), shade
        h_harm = h0 + 90.0;
        s_harm = clamp_val(s0 - 5.0, 15.0, 100.0);
        l_harm = clamp_val(l0 - 5.0, 15.0, 90.0);

        h_acc = h0 + 180.0;
        s_acc = clamp_val(s0 + 15.0, 25.0, 100.0);
        l_acc = clamp_val(l0 + 10.0, 20.0, 95.0);

        h_nuance = h0 + 270.0;
        s_nuance = clamp_val(s0, 15.0, 100.0);
        l_nuance = clamp_val(l0 + 5.0, 15.0, 92.0);

        connect_mode = "polygon";
    } else if scheme == 5 {
        // Monochromatic: same hue with rich perceptual lightness and saturation shifts
        h_harm = h0;
        s_harm = clamp_val(s0 - 20.0, 10.0, 100.0);
        l_harm = clamp_val(l0 + 15.0, 10.0, 90.0);

        h_acc = h0;
        s_acc = clamp_val(s0 + 20.0, 10.0, 100.0);
        l_acc = clamp_val(l0 - 15.0, 10.0, 90.0);

        h_nuance = h0;
        s_nuance = clamp_val(s0 - 40.0, 5.0, 100.0);
        l_nuance = clamp_val(l0 + delta_l, 10.0, 96.0);

        h_shadow = h0;
        s_shadow = clamp_val(s0 + 10.0, 10.0, 100.0);
        l_shadow = clamp_val(l0 - delta_l, 4.0, 85.0);
        connect_mode = "none";
    }

    let hex_base = hsl_to_hex(h0, s0, l0);
    let hex_harm = hsl_to_hex(h_harm, s_harm, l_harm);
    let hex_acc = hsl_to_hex(h_acc, s_acc, l_acc);
    let hex_nuance = hsl_to_hex(h_nuance, s_nuance, l_nuance);
    let hex_shadow = hsl_to_hex(h_shadow, s_shadow, l_shadow);

    state["slot_base"] = hex_base;
    state["slot_harmony1"] = hex_harm;
    state["slot_accent"] = hex_acc;
    state["slot_nuance"] = hex_nuance;
    state["slot_shadow"] = hex_shadow;

    // Generate wheel markers
    let markers = [];
    if scheme == 0 {
        markers = [hex_base, hex_harm];
    } else if scheme == 1 || scheme == 3 {
        markers = [hex_base, hex_harm, hex_acc];
    } else if scheme == 2 {
        markers = [hex_base, hex_harm, hex_acc];
    } else if scheme == 4 {
        markers = [hex_base, hex_harm, hex_acc, hex_nuance];
    } else {
        markers = [hex_base];
    }

    state["wheel_markers"] = markers;
    state["wheel_connect_mode"] = connect_mode;

    state["palette_header_label"] = t("Select swatch to activate brush:", "Выберите образец для смены кисти:");
    return state;
}

// Get localized rule title for Swatches group and layer naming
fn get_rule_title(scheme) {
    if scheme == 0 {
        return t("Complementary", "Комплементарная");
    } else if scheme == 1 {
        return t("Split-Complementary", "Сплит-комплементарная");
    } else if scheme == 2 {
        return t("Triadic", "Триада");
    } else if scheme == 3 {
        return t("Analogous", "Аналоговая");
    } else if scheme == 4 {
        return t("Tetradic", "Тетраида");
    } else {
        return t("Monochromatic", "Монохроматическая");
    }
}

// React to session colors picked in PhotoCraft editor
fn on_session_colors(fg_hex, bg_hex, state) {
    if fg_hex != () && fg_hex != "" {
        state["base_color"] = fg_hex;
    }
    if bg_hex != () && bg_hex != "" {
        state["bg_color"] = bg_hex;
    }
    state = recalculate_palette(state);
    return state;
}

// Main reactive UI loop hook
fn on_state_changed(state) {
    let base_color = state["base_color"];
    if base_color == () || base_color == "" {
        base_color = "#2563eb";
        state["base_color"] = base_color;
    }

    if state["spread_angle"] == () { state["spread_angle"] = 30.0; }
    if state["tone_depth"] == () { state["tone_depth"] = 35.0; }
    if state["scheme_type"] == () { state["scheme_type"] = 0; }
    if state["card_layout"] == () { state["card_layout"] = 0; }

    // Check if language, harmony parameters or base color changed
    let cur_lang = current_language();
    let last_lang = state["last_language"];
    let last_base = state["last_base_color"];
    let last_scheme = state["last_scheme_type"];
    let last_spread = state["last_spread_angle"];
    let last_tone = state["last_tone_depth"];

    if cur_lang != last_lang || base_color != last_base || state["scheme_type"] != last_scheme || state["spread_angle"] != last_spread || state["tone_depth"] != last_tone {
        if state["scheme_type"] != last_scheme || state["spread_angle"] != last_spread || state["tone_depth"] != last_tone {
            state["banner_msg"] = "";
        }
        state["last_language"] = cur_lang;
        state["last_base_color"] = base_color;
        state["last_scheme_type"] = state["scheme_type"];
        state["last_spread_angle"] = state["spread_angle"];
        state["last_tone_depth"] = state["tone_depth"];
        state = recalculate_palette(state);
    }

    // Swatch selection: switch active brush in PhotoCraft
    let selected = state["selected_slot"];
    if selected != () && selected != "" && selected != state["last_selected_slot"] {
        let chosen_hex = state[selected];
        if chosen_hex != () && chosen_hex != "" {
            let bg = state["bg_color"];
            if bg == () || bg == "" { bg = "#ffffff"; }
            execute_command("tools.setColors", #{
                foreground: chosen_hex,
                background: bg
            });
            state["base_color"] = chosen_hex;
            state["last_base_color"] = chosen_hex;
            state["last_selected_slot"] = selected;
            state["banner_msg"] = t("Active brush set to " + chosen_hex, "Кисть переключена на " + chosen_hex);
            state["banner_style"] = "info";
            state = recalculate_palette(state);
        }
    }

    // Action: Add swatches to PhotoCraft Swatches library
    if state["action_btn_save_swatches"] == true {
        state["action_btn_save_swatches"] = false;

        let rule_title = get_rule_title(state["scheme_type"]);
        let base_hex = state["slot_base"];
        if base_hex == () || base_hex == "" {
            base_hex = state["base_color"];
        }

        let base_group_name = t("Harmonic: ", "Гармония: ") + rule_title + " (" + base_hex + ")";
        let group_name = base_group_name;

        // Duplicate click guard: if user exports the exact same scheme repeatedly, append counter
        let last_exported = "";
        let repeat_count = 1;
        if state.contains("last_exported_group_base") {
            last_exported = state["last_exported_group_base"];
        }
        if state.contains("last_exported_repeat_count") {
            repeat_count = state["last_exported_repeat_count"];
        }

        if last_exported == base_group_name {
            repeat_count = repeat_count + 1;
            group_name = base_group_name + " " + repeat_count;
        } else {
            repeat_count = 1;
        }

        state["last_exported_group_base"] = base_group_name;
        state["last_exported_repeat_count"] = repeat_count;

        execute_command("swatches.newGroup", #{ name: group_name });
        execute_command("swatches.add", #{ color: state["slot_base"], name: t("Base 60%", "Базовый 60%"), group: group_name });
        execute_command("swatches.add", #{ color: state["slot_harmony1"], name: t("Harmony 30%", "Гармония 30%"), group: group_name });
        execute_command("swatches.add", #{ color: state["slot_accent"], name: t("Accent 10%", "Акцент 10%"), group: group_name });
        execute_command("swatches.add", #{ color: state["slot_nuance"], name: t("Highlight", "Блик / Свет"), group: group_name });
        execute_command("swatches.add", #{ color: state["slot_shadow"], name: t("Shadow", "Глубокая тень"), group: group_name });
        state["banner_msg"] = t("Added to Swatches: " + group_name, "Добавлено в образцы: " + group_name);
        state["banner_style"] = "success";
    }

    // Action: Draw palette chips on a new canvas layer
    if state["action_btn_draw_canvas"] == true {
        state["action_btn_draw_canvas"] = false;

        let rule_title = get_rule_title(state["scheme_type"]);
        let base_hex = state["slot_base"];
        if base_hex == () || base_hex == "" {
            base_hex = state["base_color"];
        }
        let layer_name = t("Harmonic: ", "Гармония: ") + rule_title + " (" + base_hex + ")";
        execute_command("layer.new.layer", #{ name: layer_name });

        let is_vert = state["card_layout"] == 1;
        let chips = [state["slot_base"], state["slot_harmony1"], state["slot_accent"], state["slot_nuance"], state["slot_shadow"]];

        for i in 0..5 {
            let chip_color = chips[i];
            if chip_color == () || chip_color == "" { continue; }

            let pos_x = 30;
            let pos_y = 30;
            if is_vert {
                pos_x = 30;
                pos_y = 30 + i * 52;
            } else {
                pos_x = 30 + i * 52;
                pos_y = 30;
            }

            execute_command("select.rect", #{
                x: pos_x,
                y: pos_y,
                width: 44,
                height: 44,
                mode: "replace"
            });
            execute_command("edit.fill", #{
                contents: "color",
                color: chip_color
            });
        }
        execute_command("select.deselect", #{});
        state["banner_msg"] = t("Swatch card drawn on new canvas layer!", "Карточка палитры создана на новом слое холста!");
        state["banner_style"] = "success";
    }

    return state;
}
