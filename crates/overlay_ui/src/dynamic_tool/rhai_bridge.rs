use crate::dynamic_tool::state::{DynamicValue, DynamicWindowState};
use crate::dynamic_tool::widgets::hue_wheel::{
    calculate_wcag_contrast, find_nearest_named_color, parse_hex_color, simulate_cvd_hex,
};
use core_contracts::{Language, PluginIpcAction};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc::UnboundedSender;

/// Creates and configures the standard Rhai scripting engine with all PhotoCraft plugins API bindings
pub fn create_rhai_engine(
    action_tx: Option<UnboundedSender<PluginIpcAction>>,
    lang_arc: Arc<RwLock<Language>>,
    echo_guard: Arc<std::sync::Mutex<Option<(String, String)>>>,
) -> rhai::Engine {
    let mut engine = rhai::Engine::new();
    engine.set_max_expr_depths(64, 64);
    engine.set_max_operations(50_000);

    if let Some(tx) = action_tx {
        let tx_cmd = tx.clone();
        let echo_guard_cmd = echo_guard.clone();
        engine.register_fn(
            "execute_command",
            move |command: rhai::ImmutableString, params: rhai::Map| {
                if command.as_str() == "tools.setColors" {
                    let fg = params
                        .get("foreground")
                        .and_then(|v| v.clone().into_string().ok());
                    let bg = params
                        .get("background")
                        .and_then(|v| v.clone().into_string().ok());
                    if let (Some(f), Some(b)) = (fg, bg) {
                        if let Ok(mut g) = echo_guard_cmd.lock() {
                            *g = Some((f.to_lowercase(), b.to_lowercase()));
                        }
                    }
                }

                let json_val =
                    rhai::serde::from_dynamic::<serde_json::Value>(&rhai::Dynamic::from(params))
                        .unwrap_or(serde_json::Value::Null);

                let action = PluginIpcAction {
                    command: command.to_string(),
                    params: json_val,
                };
                let _ = tx_cmd.send(action);
            },
        );

        let tx_bare = tx.clone();
        engine.register_fn("execute_command", move |command: rhai::ImmutableString| {
            let action = PluginIpcAction {
                command: command.to_string(),
                params: serde_json::Value::Null,
            };
            let _ = tx_bare.send(action);
        });

        let tx_colors = tx.clone();
        let echo_guard_colors = echo_guard.clone();
        engine.register_fn(
            "set_colors",
            move |foreground: rhai::ImmutableString, background: rhai::ImmutableString| {
                if let Ok(mut g) = echo_guard_colors.lock() {
                    *g = Some((
                        foreground.to_string().to_lowercase(),
                        background.to_string().to_lowercase(),
                    ));
                }
                let action = PluginIpcAction {
                    command: "tools.setColors".to_string(),
                    params: serde_json::json!({
                        "foreground": foreground.to_string(),
                        "background": background.to_string()
                    }),
                };
                let _ = tx_colors.send(action);
            },
        );

        let tx_raster = tx;
        engine.register_fn(
            "execute_raster_process",
            move |task: rhai::ImmutableString, params: rhai::Map| {
                let json_params =
                    rhai::serde::from_dynamic::<serde_json::Value>(&rhai::Dynamic::from(params))
                        .unwrap_or(serde_json::Value::Null);

                let action = PluginIpcAction::new(
                    "raster.process",
                    serde_json::json!({
                        "task": task.to_string(),
                        "params": json_params,
                    }),
                );
                let _ = tx_raster.send(action);
            },
        );
    }

    // I18n helpers
    let lang_t = lang_arc.clone();
    engine.register_fn(
        "t",
        move |en: rhai::ImmutableString, ru: rhai::ImmutableString| -> String {
            let lang = *lang_t.read().unwrap();
            if matches!(lang, Language::Ru) {
                ru.to_string()
            } else {
                en.to_string()
            }
        },
    );

    let lang_curr = lang_arc.clone();
    engine.register_fn("current_language", move || -> String {
        let lang = *lang_curr.read().unwrap();
        if matches!(lang, Language::Ru) {
            "ru".to_string()
        } else {
            "en".to_string()
        }
    });

    // Color formatting helpers
    engine.register_fn("rgb_to_hex", |r: i64, g: i64, b: i64| -> String {
        let r_clamped = r.clamp(0, 255) as u8;
        let g_clamped = g.clamp(0, 255) as u8;
        let b_clamped = b.clamp(0, 255) as u8;
        format!("#{r_clamped:02x}{g_clamped:02x}{b_clamped:02x}")
    });
    engine.register_fn("rgb_to_hex", |r: f64, g: f64, b: f64| -> String {
        let r_clamped = r.round().clamp(0.0, 255.0) as u8;
        let g_clamped = g.round().clamp(0.0, 255.0) as u8;
        let b_clamped = b.round().clamp(0.0, 255.0) as u8;
        format!("#{r_clamped:02x}{g_clamped:02x}{b_clamped:02x}")
    });

    engine.register_fn("hex_to_rgb", |hex: rhai::ImmutableString| -> rhai::Map {
        let mut map = rhai::Map::new();
        if let Some(color) = parse_hex_color(&hex) {
            map.insert("r".into(), rhai::Dynamic::from(color.r() as i64));
            map.insert("g".into(), rhai::Dynamic::from(color.g() as i64));
            map.insert("b".into(), rhai::Dynamic::from(color.b() as i64));
        } else {
            map.insert("r".into(), rhai::Dynamic::from(0_i64));
            map.insert("g".into(), rhai::Dynamic::from(0_i64));
            map.insert("b".into(), rhai::Dynamic::from(0_i64));
        }
        map
    });

    let lang_named1 = lang_arc.clone();
    engine.register_fn("get_named_color", move |r: i64, g: i64, b: i64| -> String {
        let lang = *lang_named1.read().unwrap();
        find_nearest_named_color(r, g, b, lang)
    });
    let lang_named2 = lang_arc.clone();
    engine.register_fn("get_named_color", move |r: f64, g: f64, b: f64| -> String {
        let lang = *lang_named2.read().unwrap();
        find_nearest_named_color(r.round() as i64, g.round() as i64, b.round() as i64, lang)
    });

    engine.register_fn(
        "simulate_cvd",
        |r: i64, g: i64, b: i64, profile_index: i64| -> String {
            simulate_cvd_hex(r as f64, g as f64, b as f64, profile_index)
        },
    );
    engine.register_fn(
        "simulate_cvd",
        |r: f64, g: f64, b: f64, profile_index: i64| -> String {
            simulate_cvd_hex(r, g, b, profile_index)
        },
    );
    engine.register_fn(
        "simulate_cvd",
        |r: f64, g: f64, b: f64, profile_index: f64| -> String {
            simulate_cvd_hex(r, g, b, profile_index.round() as i64)
        },
    );

    engine.register_fn(
        "calculate_contrast",
        |r1: i64, g1: i64, b1: i64, r2: i64, g2: i64, b2: i64| -> f64 {
            calculate_wcag_contrast(
                r1 as f64, g1 as f64, b1 as f64, r2 as f64, g2 as f64, b2 as f64,
            )
        },
    );
    engine.register_fn(
        "calculate_contrast",
        |r1: f64, g1: f64, b1: f64, r2: f64, g2: f64, b2: f64| -> f64 {
            calculate_wcag_contrast(r1, g1, b1, r2, g2, b2)
        },
    );

    engine
}

/// Executes script callbacks for state changes and button actions
pub fn execute_script_state_change(
    engine: &rhai::Engine,
    ast: &rhai::AST,
    state: &mut DynamicWindowState,
) {
    // 1. Pass current state WITH all active action_* flags to on_state_changed
    let mut scope = rhai::Scope::new();
    let state_map = state.to_rhai_map();
    scope.push("state", state_map.clone());

    let res =
        engine.call_fn::<rhai::Dynamic>(&mut scope, ast, "on_state_changed", (state_map.clone(),));
    let final_res = match res {
        Ok(v) => Ok(v),
        Err(_) => engine.call_fn::<rhai::Dynamic>(&mut scope, ast, "on_state_changed", ()),
    };

    if let Ok(dyn_val) = final_res {
        if let Some(mutated_map) = dyn_val.try_cast::<rhai::Map>() {
            state.update_from_rhai_map(mutated_map);
        }
    } else if let Some(mutated_map) = scope.get_value::<rhai::Map>("state") {
        state.update_from_rhai_map(mutated_map);
    }

    // 2. Dispatch on_action(action_id) / on_action(action_id, state)
    let action_keys: Vec<String> = state
        .values
        .iter()
        .filter_map(|(k, v)| {
            if k.starts_with("action_") && v.as_bool() == Some(true) {
                Some(k.clone())
            } else {
                None
            }
        })
        .collect();

    for key in &action_keys {
        let action_id = key.strip_prefix("action_").unwrap_or(key);
        let mut action_scope = rhai::Scope::new();
        let cur_state_map = state.to_rhai_map();
        action_scope.push("state", cur_state_map.clone());

        let res = engine.call_fn::<rhai::Dynamic>(
            &mut action_scope,
            ast,
            "on_action",
            (action_id.to_string(), cur_state_map),
        );
        let final_res = match res {
            Ok(v) => Ok(v),
            Err(_) => engine.call_fn::<rhai::Dynamic>(
                &mut action_scope,
                ast,
                "on_action",
                (action_id.to_string(),),
            ),
        };

        if let Ok(dyn_val) = final_res {
            if let Some(mutated_map) = dyn_val.try_cast::<rhai::Map>() {
                state.update_from_rhai_map(mutated_map);
            }
        } else if let Some(mutated_map) = action_scope.get_value::<rhai::Map>("state") {
            state.update_from_rhai_map(mutated_map);
        }
    }

    // 3. Reset all action_* flags in state so they do not re-trigger on next tick
    for (k, v) in &mut state.values {
        if k.starts_with("action_") {
            *v = DynamicValue::Bool(false);
        }
    }
}
