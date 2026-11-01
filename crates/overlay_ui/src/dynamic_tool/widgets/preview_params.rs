use crate::dynamic_tool::state::DynamicValue;
use crate::dynamic_tool::widgets::hue_wheel::parse_hex_color;
use raster_engine::{ExtrudeParams, LensBlurParams};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

pub fn extract_lens_blur_params(values: &HashMap<String, DynamicValue>) -> Option<LensBlurParams> {
    if !values.contains_key("blur_radius")
        && !values.contains_key("focus_distance")
        && !values.contains_key("fog_density")
        && !values.contains_key("bokeh_enabled")
        && !values.contains_key("fog_enabled")
    {
        return None;
    }

    let mut p = LensBlurParams::default();
    if let Some(b) = values.get("fog_enabled").and_then(|v| v.as_bool()) {
        p.fog_enabled = b;
    }
    if let Some(s) = values.get("fog_color").and_then(|v| v.as_str()) {
        if let Some(c) = parse_hex_color(s) {
            p.fog_color = [c.r(), c.g(), c.b(), 255];
        }
    }
    if let Some(n) = values.get("fog_density").and_then(|v| v.as_f64()) {
        p.fog_density = n as f32;
    }
    if let Some(n) = values.get("fog_start").and_then(|v| v.as_f64()) {
        p.fog_start = n as f32;
    }
    if let Some(n) = values.get("fog_falloff").and_then(|v| v.as_f64()) {
        p.fog_falloff = n as f32;
    }
    if let Some(idx) = values.get("fog_blend_mode").and_then(|v| v.as_index()) {
        p.fog_blend_mode = idx as u8;
    }
    if let Some(b) = values.get("bokeh_enabled").and_then(|v| v.as_bool()) {
        p.bokeh_enabled = b;
    }
    if let Some(n) = values.get("focus_distance").and_then(|v| v.as_f64()) {
        p.focus_distance = n as f32;
    }
    if let Some(n) = values.get("dof_width").and_then(|v| v.as_f64()) {
        p.dof_width = n as f32;
    }
    if let Some(n) = values.get("blur_radius").and_then(|v| v.as_f64()) {
        p.blur_radius = n as f32;
    }
    if let Some(n) = values.get("specular_bloom").and_then(|v| v.as_f64()) {
        p.specular_bloom = n as f32;
    }
    Some(p)
}

pub fn hash_lens_blur_params(p: &LensBlurParams, scale: f32) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    p.fog_enabled.hash(&mut hasher);
    p.fog_color.hash(&mut hasher);
    p.fog_density.to_bits().hash(&mut hasher);
    p.fog_start.to_bits().hash(&mut hasher);
    p.fog_falloff.to_bits().hash(&mut hasher);
    p.fog_blend_mode.hash(&mut hasher);
    p.bokeh_enabled.hash(&mut hasher);
    p.focus_distance.to_bits().hash(&mut hasher);
    p.dof_width.to_bits().hash(&mut hasher);
    p.blur_radius.to_bits().hash(&mut hasher);
    p.specular_bloom.to_bits().hash(&mut hasher);
    scale.to_bits().hash(&mut hasher);
    hasher.finish()
}

pub fn extract_extrude_params(values: &HashMap<String, DynamicValue>) -> ExtrudeParams {
    let mut params = ExtrudeParams::default();

    for (k, v) in values {
        let k_lower = k.to_lowercase();
        if let Some(num) = v.as_f64() {
            if k_lower.ends_with("_light_angle")
                || k_lower == "light_angle"
                || k_lower == "light_angle_deg"
            {
                params.light_angle_deg = num as f32;
            } else if k_lower.ends_with("_light_intensity") || k_lower == "light_intensity" {
                params.light_intensity = num as f32;
            } else if k_lower.ends_with("_angle") || k_lower == "angle" || k_lower == "angle_deg" {
                params.angle_deg = num as f32;
            } else if k_lower.ends_with("_distance") || k_lower == "distance" {
                params.distance = num as f32;
            } else if k_lower.ends_with("_bevel_radius") || k_lower == "bevel_radius" {
                params.bevel_radius = num as f32;
            } else if k_lower.ends_with("_bevel_highlight")
                || k_lower == "bevel_highlight"
                || k_lower == "highlight"
            {
                params.bevel_highlight = num as f32;
            } else if k_lower.ends_with("_shadow_blur") || k_lower == "shadow_blur" {
                params.shadow_blur = num as f32;
            } else if k_lower.ends_with("_shadow_opacity") || k_lower == "shadow_opacity" {
                params.shadow_opacity = num as f32;
            } else if k_lower.ends_with("_shadow_distance") || k_lower == "shadow_distance" {
                params.shadow_distance = num as f32;
            } else if k_lower.ends_with("_perspective") || k_lower == "perspective" {
                params.perspective = num as f32;
            }
        } else if let Some(s) = v.as_str() {
            if (k_lower.ends_with("_start_color")
                || k_lower.ends_with("body_start")
                || k_lower.ends_with("_start"))
                && s.starts_with('#')
            {
                if let Some(c) = parse_hex_color(s) {
                    params.body_gradient_start = image::Rgba([c.r(), c.g(), c.b(), 255]);
                }
            } else if (k_lower.ends_with("_end_color")
                || k_lower.ends_with("body_end")
                || k_lower.ends_with("_end"))
                && s.starts_with('#')
            {
                if let Some(c) = parse_hex_color(s) {
                    params.body_gradient_end = image::Rgba([c.r(), c.g(), c.b(), 255]);
                }
            } else if (k_lower.ends_with("front_color") || k_lower == "front") && s.starts_with('#')
            {
                if let Some(c) = parse_hex_color(s) {
                    params.front_color = Some(image::Rgba([c.r(), c.g(), c.b(), 255]));
                }
            }
        } else if let Some(b) = v.as_bool() {
            if (k_lower.ends_with("shadow_enabled") || k_lower == "shadow_enabled") && !b {
                params.shadow_opacity = 0.0;
            }
        }
    }

    params
}

pub fn hash_extrude_params(p: &ExtrudeParams, scale: f32) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    p.angle_deg.to_bits().hash(&mut hasher);
    p.distance.to_bits().hash(&mut hasher);
    if let Some(fc) = p.front_color {
        fc.0.hash(&mut hasher);
    }
    p.body_gradient_start.0.hash(&mut hasher);
    p.body_gradient_end.0.hash(&mut hasher);
    p.bevel_radius.to_bits().hash(&mut hasher);
    p.bevel_highlight.to_bits().hash(&mut hasher);
    p.light_angle_deg.to_bits().hash(&mut hasher);
    p.light_intensity.to_bits().hash(&mut hasher);
    p.shadow_blur.to_bits().hash(&mut hasher);
    p.shadow_opacity.to_bits().hash(&mut hasher);
    p.shadow_distance.to_bits().hash(&mut hasher);
    p.perspective.to_bits().hash(&mut hasher);
    scale.to_bits().hash(&mut hasher);
    hasher.finish()
}
