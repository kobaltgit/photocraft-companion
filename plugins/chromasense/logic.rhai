// ChromaSense A11y - Dynamic Color Harmony & Vision Simulation Engine (Rhai)

fn hex_digit(val) {
    if val < 10 {
        return (val).to_string();
    } else if val == 10 {
        return "a";
    } else if val == 11 {
        return "b";
    } else if val == 12 {
        return "c";
    } else if val == 13 {
        return "d";
    } else if val == 14 {
        return "e";
    } else {
        return "f";
    }
}

fn to_hex_byte(num) {
    let clamped = num.max(0).min(255);
    let high = (clamped / 16).round();
    let low = clamped % 16;
    return hex_digit(high) + hex_digit(low);
}

fn format_hex(r, g, b) {
    return "#" + to_hex_byte(r) + to_hex_byte(g) + to_hex_byte(b);
}

fn get_semantic_color_name(r, g, b) {
    let max_val = r.max(g).max(b);
    let min_val = r.min(g).min(b);
    let delta = max_val - min_val;

    if delta < 25 {
        if max_val < 50 {
            return "Obsidian Black (Нейтральный черный)";
        } else if max_val > 210 {
            return "Alabaster White (Чистый белый)";
        } else {
            return "Slate Neutral Gray (Нейтральный серый)";
        }
    }

    if r >= g && r >= b {
        if g > 180 && b < 60 {
            return "Amber Gold / Yellow (Янтарно-золотой)";
        } else if g > 80 && b < 60 {
            return "Tangerine Orange (Мандариновый оранжевый)";
        } else if b > 140 {
            return "Magenta Rose (Пурпурно-розовый)";
        } else {
            return "Crimson Scarlet (Насыщенный алый)";
        }
    } else if g >= r && g >= b {
        if b > 150 {
            return "Cyan Aquamarine (Морская лазурь)";
        } else if r > 150 {
            return "Chartreuse Lime (Лаймовый шартрез)";
        } else {
            return "Emerald Jade (Изумрудный зеленый)";
        }
    } else {
        if r > 150 {
            return "Royal Violet (Королевский фиолетовый)";
        } else if g > 150 {
            return "Electric Azure (Электрический лазурный)";
        } else {
            return "Cobalt Ultramarine (Кобальтовый синий)";
        }
    }
}

fn calculate_relative_luminance(r, g, b) {
    let rs = r / 255.0;
    let gs = g / 255.0;
    let bs = b / 255.0;
    return 0.2126 * rs + 0.7152 * gs + 0.0722 * bs;
}

// Reactive hook invoked whenever a widget value is modified
fn on_state_changed(state) {
    let r = state["red"].round();
    let g = state["green"].round();
    let b = state["blue"].round();

    let fg_hex = format_hex(r, g, b);
    state["color_fg"] = fg_hex;

    let bg_hex = state["color_bg"];
    if bg_hex == () || bg_hex == "" {
        bg_hex = "#ffffff";
        state["color_bg"] = bg_hex;
    }

    // Determine semantic color name
    let color_name = get_semantic_color_name(r, g, b);
    state["semantic_color_name"] = color_name;

    // Vision deficiency profile (0: Normal, 1: Deuteranopia, 2: Protanopia, 3: Tritanopia)
    let profile_idx = state["vision_profile"];
    if profile_idx == () {
        profile_idx = 0;
    }

    // Relative luminance & contrast ratio
    let lum_fg = calculate_relative_luminance(r, g, b);
    let lum_bg = 1.0; // Assume white background default
    let l1 = lum_fg.max(lum_bg);
    let l2 = lum_fg.min(lum_bg);
    let contrast_ratio = (l1 + 0.05) / (l2 + 0.05);

    if profile_idx == 1 {
        // Deuteranopia (Green-blindness simulation)
        if (r - g).abs() < 40 && (r > 100 || g > 100) {
            state["collision_warning"] = "Deuteranopia Alert: Low red/green perceptual distance";
            state["collision_style"] = "warning";
        } else {
            state["collision_warning"] = "Deuteranopia Simulated: Perceptually distinguishable";
            state["collision_style"] = "success";
        }
    } else if profile_idx == 2 {
        // Protanopia (Red-blindness simulation)
        if r > 120 && g > 90 && (r - g) < 50 {
            state["collision_warning"] = "Protanopia Alert: Red hue brightness significantly reduced";
            state["collision_style"] = "warning";
        } else {
            state["collision_warning"] = "Protanopia Simulated: Perceptually safe";
            state["collision_style"] = "success";
        }
    } else if profile_idx == 3 {
        // Tritanopia (Blue-blindness simulation)
        if (b - g).abs() < 50 && (b > 100 || g > 100) {
            state["collision_warning"] = "Tritanopia Alert: Blue/yellow confusion sector";
            state["collision_style"] = "warning";
        } else {
            state["collision_warning"] = "Tritanopia Simulated: Adequate distinction";
            state["collision_style"] = "success";
        }
    } else {
        // Normal Vision
        if contrast_ratio < 3.0 {
            state["collision_warning"] = "WCAG Notice: Low contrast ratio with background (" + contrast_ratio.round().to_string() + ":1)";
            state["collision_style"] = "warning";
        } else {
            state["collision_warning"] = "Accessibility OK: High contrast ratio (" + contrast_ratio.round().to_string() + ":1)";
            state["collision_style"] = "success";
        }
    }

    // Send update command to PhotoCraft
    execute_command("tools.setColors", #{
        foreground: fg_hex,
        background: bg_hex
    });

    return state;
}
