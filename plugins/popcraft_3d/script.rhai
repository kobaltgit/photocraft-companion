// PopCraft 3D Reactive Controller Script (TOBIDASE-aligned)

fn on_state_changed(state) {
    // 1. Preset Handlers
    if state["action_preset_youtube"] == true {
        state["action_preset_youtube"] = false;
        state["extrude_pad_angle"] = 120.0;
        state["extrude_pad_distance"] = 28.0;
        state["perspective"] = 0.25;
        state["body_start_color"] = "#f59e0b";
        state["body_end_color"] = "#78350f";
        state["light_angle"] = 45.0;
        state["bevel_highlight"] = 0.7;
        state["bevel_radius"] = 2.0;
        state["shadow_enabled"] = true;
        state["shadow_blur"] = 12.0;
    }

    if state["action_preset_retro"] == true {
        state["action_preset_retro"] = false;
        state["extrude_pad_angle"] = 135.0;
        state["extrude_pad_distance"] = 34.0;
        state["perspective"] = 0.0;
        state["body_start_color"] = "#ef4444";
        state["body_end_color"] = "#1e1b4b";
        state["light_angle"] = 90.0;
        state["bevel_highlight"] = 0.0;
        state["bevel_radius"] = 0.0;
        state["shadow_enabled"] = true;
        state["shadow_blur"] = 4.0;
    }

    if state["action_preset_neon"] == true {
        state["action_preset_neon"] = false;
        state["extrude_pad_angle"] = 110.0;
        state["extrude_pad_distance"] = 24.0;
        state["perspective"] = 0.35;
        state["body_start_color"] = "#ec4899";
        state["body_end_color"] = "#3b82f6";
        state["light_angle"] = 60.0;
        state["bevel_highlight"] = 0.8;
        state["bevel_radius"] = 2.5;
        state["shadow_enabled"] = true;
        state["shadow_blur"] = 16.0;
    }

    if state["action_preset_gold"] == true {
        state["action_preset_gold"] = false;
        state["extrude_pad_angle"] = 135.0;
        state["extrude_pad_distance"] = 22.0;
        state["perspective"] = 0.15;
        state["body_start_color"] = "#fbbf24";
        state["body_end_color"] = "#78350f";
        state["light_angle"] = 45.0;
        state["bevel_highlight"] = 0.9;
        state["bevel_radius"] = 3.0;
        state["shadow_enabled"] = true;
        state["shadow_blur"] = 8.0;
    }

    // 2. Apply to PhotoCraft Handler
    if state["action_apply"] == true {
        state["action_apply"] = false;

        let out_mode = "merged";
        if state["output_mode"] == 1 {
            out_mode = "split";
        }

        let dist = state["extrude_pad_distance"];
        let light = state["light_angle"];
        if light == () {
            light = 45.0;
        }
        let highlight = state["bevel_highlight"];
        if highlight == () {
            highlight = 0.6;
        }

        let persp = state["perspective"];
        if persp == () {
            persp = 0.0;
        }

        let shadow_op = 0.5;
        if state["shadow_enabled"] == false {
            shadow_op = 0.0;
        }

        let params_map = #{
            angle_deg: state["extrude_pad_angle"],
            distance: dist,
            perspective: persp,
            body_start_color: state["body_start_color"],
            body_end_color: state["body_end_color"],
            light_angle_deg: light,
            bevel_highlight: highlight,
            bevel_radius: state["bevel_radius"],
            shadow_blur: state["shadow_blur"],
            shadow_opacity: shadow_op,
            shadow_distance: dist * 0.4,
            output_mode: out_mode
        };

        execute_command("raster.process", #{
            task: "extrude_2_5d",
            params: params_map
        });
    }

    return state;
}