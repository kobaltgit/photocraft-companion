// Selection Modifiers Reactive Controller Script
// Author: CharlieGoesMax & kobaltGIT

fn on_state_changed(state) {
    let order = state["modifiers_order"];
    if order == () {
        order = ["border", "smooth", "expand", "contract", "feather"];
    }

    // 1. Initial snapshot of base selection when window opens
    if state["is_open"] == true && state["base_saved"] != true {
        execute_command("select.saveSelection", #{
            channel: "__pc_companion_base_sel",
            name: "__pc_companion_base_sel",
            operation: "replace"
        });
        for mod_id in order {
            state[mod_id + "_prev_enabled"] = state[mod_id + "_enabled"];
        }
        state["base_saved"] = true;
    }

    // 2. Window closed cleanup: delete temporary alpha channel
    if state["is_open"] == false {
        if state["base_saved"] == true {
            execute_command("channel.delete", #{
                channel: "__pc_companion_base_sel"
            });
            state["base_saved"] = false;
            state["last_bounds"] = ();
        }
    }

    // 3. Reset Handler (Zero Data-Loss UX)
    // Resets UI modifiers without altering or discarding user selection on canvas.
    // Syncs current canvas selection as the new base contour.
    if state["action_reset"] == true {
        state["action_reset"] = false;

        for mod_id in order {
            state[mod_id + "_enabled"] = false;
            state[mod_id + "_prev_enabled"] = false;
            state[mod_id + "_value"] = 0.0;
        }

        // Commit current intact canvas contour as base
        execute_command("select.saveSelection", #{
            channel: "__pc_companion_base_sel",
            name: "__pc_companion_base_sel",
            operation: "replace"
        });
        state["base_saved"] = true;
    }

    // 4. Apply Modifier Stack (1-Step Undo) Handler
    if state["action_apply"] == true {
        state["action_apply"] = false;

        // If base snapshot was invalidated due to external changes, snapshot current canvas
        if state["base_saved"] != true {
            execute_command("select.saveSelection", #{
                channel: "__pc_companion_base_sel",
                name: "__pc_companion_base_sel",
                operation: "replace"
            });
            state["base_saved"] = true;
        }

        // 1. Restore base contour
        execute_command("select.loadSelection", #{
            channel: "__pc_companion_base_sel",
            operation: "new"
        });

        // 2. Sequentially apply active modifiers in configured list order
        for mod_id in order {
            let is_enabled = state[mod_id + "_enabled"];
            let val = state[mod_id + "_value"];
            if is_enabled == true && val != () && val > 0.0 {
                if mod_id == "border" {
                    execute_command("select.modify.border", #{
                        radius: val
                    });
                } else {
                    execute_command("select.modify." + mod_id, #{
                        radius: val,
                        applyAtCanvasBounds: false
                    });
                }
            }
        }
    }

    // 5. Unchecking a modifier commits current canvas as new working base and zeroes its radius
    let uncheck_detected = false;
    for mod_id in order {
        let is_enabled = state[mod_id + "_enabled"];
        let was_enabled = state[mod_id + "_prev_enabled"];
        if was_enabled == true && (is_enabled == false || is_enabled == ()) {
            state[mod_id + "_value"] = 0.0;
            uncheck_detected = true;
        }
        state[mod_id + "_prev_enabled"] = is_enabled;
    }

    if uncheck_detected && state["is_open"] == true {
        execute_command("select.saveSelection", #{
            channel: "__pc_companion_base_sel",
            name: "__pc_companion_base_sel",
            operation: "replace"
        });
        state["base_saved"] = true;
    }

    return state;
}

fn on_selection_state(has_selection, bounds, state) {
    if !has_selection {
        state["base_saved"] = false;
        state["last_bounds"] = ();
    } else {
        let prev_bounds = state["last_bounds"];
        // External contour alteration detected in PhotoCraft
        if prev_bounds != () && bounds != () && prev_bounds != bounds {
            state["base_saved"] = false;
        }
        state["last_bounds"] = bounds;
    }
    return state;
}

fn on_selection_state(has_selection, state) {
    if !has_selection {
        state["base_saved"] = false;
        state["last_bounds"] = ();
    }
    return state;
}
