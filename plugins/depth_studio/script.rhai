// Depth Studio Reactive Controller (Atmospheric Fog & Lens Bokeh)

fn on_state_changed(state) {
    // Обработка пресетов тумана: автоматически включает туман
    if state["action_preset_mist"] == true {
        state["action_preset_mist"] = false;
        state["fog_enabled"] = true;
        state["fog_color"] = "#c8d6e5";
        state["fog_density"] = 55.0;
        state["fog_start"] = 10.0;
        state["fog_falloff"] = 1.4;
        state["fog_blend_mode"] = 0;
    } else if state["action_preset_sunset"] == true {
        state["action_preset_sunset"] = false;
        state["fog_enabled"] = true;
        state["fog_color"] = "#ffb142";
        state["fog_density"] = 65.0;
        state["fog_start"] = 20.0;
        state["fog_falloff"] = 1.2;
        state["fog_blend_mode"] = 1;
    } else if state["action_preset_silent_hill"] == true {
        state["action_preset_silent_hill"] = false;
        state["fog_enabled"] = true;
        state["fog_color"] = "#a4b0be";
        state["fog_density"] = 90.0;
        state["fog_start"] = 5.0;
        state["fog_falloff"] = 1.8;
        state["fog_blend_mode"] = 0;
    // Обработка пресетов боке: автоматически включает боке
    } else if state["action_preset_portrait"] == true {
        state["action_preset_portrait"] = false;
        state["bokeh_enabled"] = true;
        state["dof_width"] = 20.0;
        state["blur_radius"] = 18.0;
        state["specular_bloom"] = 35.0;
    } else if state["action_preset_street"] == true {
        state["action_preset_street"] = false;
        state["bokeh_enabled"] = true;
        state["dof_width"] = 40.0;
        state["blur_radius"] = 10.0;
        state["specular_bloom"] = 20.0;
    } else if state["action_preset_macro"] == true {
        state["action_preset_macro"] = false;
        state["bokeh_enabled"] = true;
        state["dof_width"] = 10.0;
        state["blur_radius"] = 28.0;
        state["specular_bloom"] = 50.0;
    // Обработка действий
    } else if state["action_refresh_scene"] == true {
        state["action_refresh_scene"] = false;
        execute_command("raster.process", #{
            "task": "refresh_preview",
            "plugin_id": "ui.photocraft.plugins.depth_studio"
        });
    } else if state["action_apply_to_canvas"] == true {
        state["action_apply_to_canvas"] = false;
        let params = #{
            "fog_enabled": state["fog_enabled"],
            "fog_color": state["fog_color"],
            "fog_density": state["fog_density"],
            "fog_start": state["fog_start"],
            "fog_falloff": state["fog_falloff"],
            "fog_blend_mode": state["fog_blend_mode"],
            "bokeh_enabled": state["bokeh_enabled"],
            "focus_distance": state["focus_distance"],
            "dof_width": state["dof_width"],
            "blur_radius": state["blur_radius"],
            "specular_bloom": state["specular_bloom"]
        };
        let fog_sep = if state["fog_as_separate_layer"] == false { false } else { true };
        params["fog_as_separate_layer"] = fog_sep;
        execute_command("raster.process", #{
            "task": "lens_blur",
            "plugin_id": "ui.photocraft.plugins.depth_studio",
            "fog_as_separate_layer": fog_sep,
            "params": params
        });
        if fog_sep && state["fog_enabled"] == true {
            let blend_mode = state["fog_blend_mode"];
            let blend_name = if blend_mode == 1 {
                "Screen"
            } else if blend_mode == 2 {
                "Soft Light"
            } else {
                "Normal"
            };
            if blend_name != "Normal" {
                execute_command("layer.setProps", #{ "blend": blend_name });
            }
        }
    }
    return state;
}

fn on_action(action_id) {
    if action_id == "preset_mist" {
        state["fog_enabled"] = true;
        state["fog_color"] = "#c8d6e5";
        state["fog_density"] = 55.0;
        state["fog_start"] = 10.0;
        state["fog_falloff"] = 1.4;
        state["fog_blend_mode"] = 0;
    } else if action_id == "preset_sunset" {
        state["fog_enabled"] = true;
        state["fog_color"] = "#ffb142";
        state["fog_density"] = 65.0;
        state["fog_start"] = 20.0;
        state["fog_falloff"] = 1.2;
        state["fog_blend_mode"] = 1;
    } else if action_id == "preset_silent_hill" {
        state["fog_enabled"] = true;
        state["fog_color"] = "#a4b0be";
        state["fog_density"] = 90.0;
        state["fog_start"] = 5.0;
        state["fog_falloff"] = 1.8;
        state["fog_blend_mode"] = 0;
    } else if action_id == "preset_portrait" {
        state["bokeh_enabled"] = true;
        state["dof_width"] = 20.0;
        state["blur_radius"] = 18.0;
        state["specular_bloom"] = 35.0;
    } else if action_id == "preset_street" {
        state["bokeh_enabled"] = true;
        state["dof_width"] = 40.0;
        state["blur_radius"] = 10.0;
        state["specular_bloom"] = 20.0;
    } else if action_id == "preset_macro" {
        state["bokeh_enabled"] = true;
        state["dof_width"] = 10.0;
        state["blur_radius"] = 28.0;
        state["specular_bloom"] = 50.0;
    } else if action_id == "refresh_scene" {
        execute_command("raster.process", #{
            "task": "refresh_preview",
            "plugin_id": "ui.photocraft.plugins.depth_studio"
        });
    } else if action_id == "apply_to_canvas" {
        let params = #{
            "fog_enabled": state["fog_enabled"],
            "fog_color": state["fog_color"],
            "fog_density": state["fog_density"],
            "fog_start": state["fog_start"],
            "fog_falloff": state["fog_falloff"],
            "fog_blend_mode": state["fog_blend_mode"],
            "bokeh_enabled": state["bokeh_enabled"],
            "focus_distance": state["focus_distance"],
            "dof_width": state["dof_width"],
            "blur_radius": state["blur_radius"],
            "specular_bloom": state["specular_bloom"]
        };
        let fog_sep = if state["fog_as_separate_layer"] == false { false } else { true };
        params["fog_as_separate_layer"] = fog_sep;
        execute_command("raster.process", #{
            "task": "lens_blur",
            "plugin_id": "ui.photocraft.plugins.depth_studio",
            "fog_as_separate_layer": fog_sep,
            "params": params
        });
        if fog_sep && state["fog_enabled"] == true {
            let blend_mode = state["fog_blend_mode"];
            let blend_name = if blend_mode == 1 {
                "Screen"
            } else if blend_mode == 2 {
                "Soft Light"
            } else {
                "Normal"
            };
            if blend_name != "Normal" {
                execute_command("layer.setProps", #{ "blend": blend_name });
            }
        }
    }
}
