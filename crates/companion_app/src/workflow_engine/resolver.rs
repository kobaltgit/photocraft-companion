use serde_json::{json, Value};
use std::collections::HashMap;

/// Runtime context holding state variables during workflow and command step execution.
#[derive(Debug, Clone, Default)]
pub struct ExecutionContext {
    pub source_layer_id: Option<u64>,
    pub result_layer_id: Option<u64>,
    pub active_layer_id: Option<u64>,
    pub document_id: Option<u64>,
    pub document_width: u32,
    pub document_height: u32,
    pub has_selection: bool,
    pub fg_color: Option<String>,
    pub bg_color: Option<String>,
    pub custom_vars: HashMap<String, Value>,
    pub host_language: Option<String>,
}

impl ExecutionContext {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Recursively resolves template variables within command parameters.
/// Supports both exact typed substitutions (e.g. `"$source_layer"` -> `123`)
/// and embedded string interpolations (e.g. `"Layer $source_layer copy"` -> `"Layer 123 copy"`).
pub fn resolve_value(val: &Value, ctx: &ExecutionContext) -> Value {
    match val {
        Value::Object(map) => {
            let mut resolved = serde_json::Map::new();
            for (k, v) in map {
                resolved.insert(k.clone(), resolve_value(v, ctx));
            }
            Value::Object(resolved)
        }
        Value::Array(arr) => {
            let resolved: Vec<Value> = arr.iter().map(|item| resolve_value(item, ctx)).collect();
            Value::Array(resolved)
        }
        Value::String(s) => {
            // 1. Exact match replacements (preserves numeric/boolean types)
            match s.as_str() {
                "$source_layer" => {
                    return ctx
                        .source_layer_id
                        .map(|id| json!(id))
                        .unwrap_or(Value::Null)
                }
                "$result_layer" => {
                    return ctx
                        .result_layer_id
                        .map(|id| json!(id))
                        .unwrap_or(Value::Null)
                }
                "$active_layer" => {
                    return ctx
                        .active_layer_id
                        .map(|id| json!(id))
                        .unwrap_or(Value::Null)
                }
                "$document.id" => {
                    return ctx.document_id.map(|id| json!(id)).unwrap_or(Value::Null)
                }
                "$document.width" => return json!(ctx.document_width),
                "$document.height" => return json!(ctx.document_height),
                "$document.has_selection" | "$has_selection" => return json!(ctx.has_selection),
                "$session.fg_color" => {
                    return ctx
                        .fg_color
                        .as_ref()
                        .map(|c| json!(c))
                        .unwrap_or(Value::Null)
                }
                "$session.bg_color" => {
                    return ctx
                        .bg_color
                        .as_ref()
                        .map(|c| json!(c))
                        .unwrap_or(Value::Null)
                }
                custom if custom.starts_with('$') => {
                    let var_name = &custom[1..];
                    if let Some(custom_val) = ctx.custom_vars.get(var_name) {
                        return custom_val.clone();
                    }
                }
                _ => {}
            }

            // 2. String interpolation if string contains '$'
            if s.contains('$') {
                let mut interpolated = s.clone();
                let src_str = ctx
                    .source_layer_id
                    .map(|id| id.to_string())
                    .unwrap_or_default();
                let res_str = ctx
                    .result_layer_id
                    .map(|id| id.to_string())
                    .unwrap_or_default();
                let act_str = ctx
                    .active_layer_id
                    .map(|id| id.to_string())
                    .unwrap_or_default();
                let doc_str = ctx.document_id.map(|id| id.to_string()).unwrap_or_default();
                let w_str = ctx.document_width.to_string();
                let h_str = ctx.document_height.to_string();
                let fg_str = ctx.fg_color.as_deref().unwrap_or_default();
                let bg_str = ctx.bg_color.as_deref().unwrap_or_default();

                interpolated = interpolated.replace("$source_layer", &src_str);
                interpolated = interpolated.replace("$result_layer", &res_str);
                interpolated = interpolated.replace("$active_layer", &act_str);
                interpolated = interpolated.replace("$document.id", &doc_str);
                interpolated = interpolated.replace("$document.width", &w_str);
                interpolated = interpolated.replace("$document.height", &h_str);
                interpolated = interpolated.replace("$session.fg_color", fg_str);
                interpolated = interpolated.replace("$session.bg_color", bg_str);

                for (var_k, var_v) in &ctx.custom_vars {
                    let needle = format!("${var_k}");
                    let repl = match var_v {
                        Value::String(inner_s) => inner_s.clone(),
                        other => other.to_string(),
                    };
                    interpolated = interpolated.replace(&needle, &repl);
                }

                Value::String(interpolated)
            } else {
                Value::String(s.clone())
            }
        }
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_variable_resolution() {
        let ctx = ExecutionContext {
            source_layer_id: Some(42),
            result_layer_id: Some(108),
            document_width: 1920,
            has_selection: true,
            ..Default::default()
        };

        assert_eq!(resolve_value(&json!("$source_layer"), &ctx), json!(42));
        assert_eq!(resolve_value(&json!("$result_layer"), &ctx), json!(108));
        assert_eq!(resolve_value(&json!("$document.width"), &ctx), json!(1920));
        assert_eq!(resolve_value(&json!("$has_selection"), &ctx), json!(true));
        assert_eq!(resolve_value(&json!("$active_layer"), &ctx), Value::Null);
    }

    #[test]
    fn test_string_interpolation() {
        let mut custom_vars = std::collections::HashMap::new();
        custom_vars.insert("suffix".to_string(), json!("backup"));
        let ctx = ExecutionContext {
            source_layer_id: Some(5),
            custom_vars,
            ..Default::default()
        };

        let input = json!({
            "name": "Layer_$source_layer_$suffix",
            "layer": "$source_layer"
        });

        let resolved = resolve_value(&input, &ctx);
        assert_eq!(resolved["name"], "Layer_5_backup");
        assert_eq!(resolved["layer"], 5);
    }

    #[test]
    fn test_nested_objects_and_arrays() {
        let ctx = ExecutionContext {
            result_layer_id: Some(99),
            ..Default::default()
        };

        let input = json!({
            "ops": [
                { "target": "$result_layer" },
                { "mode": "normal" }
            ]
        });

        let resolved = resolve_value(&input, &ctx);
        assert_eq!(resolved["ops"][0]["target"], 99);
        assert_eq!(resolved["ops"][1]["mode"], "normal");
    }
}
