use super::resolver::{resolve_value, ExecutionContext};
use core_contracts::{
    is_translatable_param, to_host_locale, BridgeOperations, Language, PluginCommandStep, Result,
};
use tracing::{debug, info, warn};

/// Evaluates execution condition for a step against context state
fn evaluate_condition(when: &str, ctx: &ExecutionContext) -> bool {
    let cond = when.trim();
    if cond == "has_selection" {
        ctx.has_selection
    } else if cond == "!has_selection" {
        !ctx.has_selection
    } else if let Some(var) = cond.strip_prefix('!') {
        let var_clean = var.trim().trim_start_matches('$');
        match ctx.custom_vars.get(var_clean) {
            Some(serde_json::Value::Bool(b)) => !b,
            Some(serde_json::Value::Null) | None => true,
            _ => false,
        }
    } else {
        let var_clean = cond.trim_start_matches('$');
        match ctx.custom_vars.get(var_clean) {
            Some(serde_json::Value::Bool(b)) => *b,
            Some(serde_json::Value::Null) | None => false,
            Some(_) => true,
        }
    }
}

/// Recursively traverses a JSON value and replaces known canonical terminology with localized terms for `host_lang`.
/// Returns the new JSON value and a boolean indicating whether any replacement took place.
pub fn translate_value_to_host(
    val: &serde_json::Value,
    host_lang: &str,
) -> (serde_json::Value, bool) {
    match val {
        serde_json::Value::Object(map) => {
            let mut changed = false;
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                if is_translatable_param(k) {
                    if let serde_json::Value::String(s) = v {
                        if let Some(trans) = to_host_locale(s, host_lang) {
                            if trans != s {
                                new_map.insert(
                                    k.clone(),
                                    serde_json::Value::String(trans.to_string()),
                                );
                                changed = true;
                                continue;
                            }
                        }
                    }
                }
                let (trans_v, sub_changed) = translate_value_to_host(v, host_lang);
                if sub_changed {
                    changed = true;
                }
                new_map.insert(k.clone(), trans_v);
            }
            (serde_json::Value::Object(new_map), changed)
        }
        serde_json::Value::Array(arr) => {
            let mut changed = false;
            let mut new_arr = Vec::with_capacity(arr.len());
            for item in arr {
                let (trans_item, item_changed) = translate_value_to_host(item, host_lang);
                if item_changed {
                    changed = true;
                }
                new_arr.push(trans_item);
            }
            (serde_json::Value::Array(new_arr), changed)
        }
        _ => (val.clone(), false),
    }
}

/// Executes a declarative sequence of plugin command steps sequentially
pub async fn execute_steps(
    steps: &[PluginCommandStep],
    ctx: &mut ExecutionContext,
    client: &(dyn BridgeOperations + Send + Sync),
) -> Result<()> {
    for (idx, step) in steps.iter().enumerate() {
        if let Some(ref when_cond) = step.when {
            if !evaluate_condition(when_cond, ctx) {
                debug!(
                    "Step [{}/{}] '{}' skipped because condition '{}' was not met",
                    idx + 1,
                    steps.len(),
                    step.command,
                    when_cond
                );
                continue;
            }
        }

        let mut resolved_params = resolve_value(&step.params, ctx);
        if resolved_params.is_null() {
            resolved_params = serde_json::json!({});
        }
        if let Some(ref desc) = step.description {
            info!(
                "Running step [{}/{}]: {} ({}) with params: {}",
                idx + 1,
                steps.len(),
                step.command,
                desc,
                resolved_params
            );
        } else {
            info!(
                "Running step [{}/{}]: {} with params: {}",
                idx + 1,
                steps.len(),
                step.command,
                resolved_params
            );
        }

        // Canonical-first + Smart Retry:
        // 1. Try canonical English parameters first (zero overhead for English hosts and when PhotoCraft updates)
        // 2. If it fails and localized translations exist for host locale, retry with translated parameters
        let first_try = client
            .execute_command(&step.command, resolved_params.clone())
            .await;
        let command_result = match first_try {
            Ok(output) => Ok(output),
            Err(original_err) => {
                let target_locale = ctx.host_language.as_deref().unwrap_or("auto");
                let effective_locale = if target_locale == "auto" {
                    Language::Auto.resolve(None).code()
                } else {
                    target_locale
                };

                let (translated_params, changed) =
                    translate_value_to_host(&resolved_params, effective_locale);

                if changed {
                    info!(
                        "Step [{}/{}] '{}' failed with canonical params: {}. Retrying with host locale '{}' params: {}",
                        idx + 1,
                        steps.len(),
                        step.command,
                        original_err,
                        effective_locale,
                        translated_params
                    );
                    match client
                        .execute_command(&step.command, translated_params)
                        .await
                    {
                        Ok(retry_output) => Ok(retry_output),
                        Err(_) => Err(original_err),
                    }
                } else {
                    Err(original_err)
                }
            }
        };

        match command_result {
            Ok(output) => {
                if let Some(ref save_var) = step.save_as {
                    if let Some(layer_id) = output
                        .get("id")
                        .or_else(|| output.get("layer_id"))
                        .or_else(|| output.get("layer"))
                        .and_then(|v| v.as_u64())
                    {
                        if save_var == "result_layer" {
                            ctx.result_layer_id = Some(layer_id);
                        } else if save_var == "source_layer" {
                            ctx.source_layer_id = Some(layer_id);
                        }
                    }
                    ctx.custom_vars.insert(save_var.clone(), output);
                }
            }
            Err(err) => {
                if step.ignore_errors {
                    warn!(
                        "Step [{}/{}] '{}' failed with error, but ignore_errors=true: {}",
                        idx + 1,
                        steps.len(),
                        step.command,
                        err
                    );
                } else {
                    return Err(err);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use core_contracts::{CompanionError, DocumentInfo};
    use image::RgbaImage;
    use serde_json::json;
    use std::sync::Mutex;

    struct TestBridge {
        pub calls: Mutex<Vec<(String, serde_json::Value)>>,
        pub fail_command: Option<String>,
    }

    #[async_trait]
    impl BridgeOperations for TestBridge {
        async fn execute_command(
            &self,
            command: &str,
            params: serde_json::Value,
        ) -> Result<serde_json::Value> {
            self.calls
                .lock()
                .unwrap()
                .push((command.to_string(), params));
            if let Some(ref f) = self.fail_command {
                if f == command {
                    return Err(CompanionError::Bridge(format!("Failed {command}")));
                }
            }
            Ok(json!({ "status": "ok", "id": 77 }))
        }

        async fn copy_active_layer_image(&self) -> Result<RgbaImage> {
            Ok(RgbaImage::new(1, 1))
        }

        async fn paste_image_in_place(&self, _image: &RgbaImage) -> Result<serde_json::Value> {
            Ok(json!({ "id": 88 }))
        }

        async fn rename_layer(&self, _layer_id: u64, _new_name: &str) -> Result<serde_json::Value> {
            Ok(json!({ "status": "renamed" }))
        }

        async fn get_document_info(&self) -> Result<DocumentInfo> {
            Ok(DocumentInfo::default())
        }
    }

    #[tokio::test]
    async fn test_execute_steps_sequential_and_save_as() {
        let bridge = TestBridge {
            calls: Mutex::new(Vec::new()),
            fail_command: None,
        };

        let steps = vec![
            PluginCommandStep {
                command: "layer.duplicate".to_string(),
                params: json!({ "layer": "$source_layer" }),
                save_as: Some("result_layer".to_string()),
                ignore_errors: false,
                when: None,
                description: None,
            },
            PluginCommandStep {
                command: "layer.hide".to_string(),
                params: json!({ "target": "$result_layer" }),
                save_as: None,
                ignore_errors: false,
                when: None,
                description: None,
            },
        ];

        let mut ctx = ExecutionContext {
            source_layer_id: Some(10),
            ..Default::default()
        };

        execute_steps(&steps, &mut ctx, &bridge).await.unwrap();

        assert_eq!(ctx.result_layer_id, Some(77));
        let calls = bridge.calls.lock().unwrap();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].1["layer"], 10);
        assert_eq!(calls[1].1["target"], 77);
    }

    #[tokio::test]
    async fn test_execute_steps_condition_when() {
        let bridge = TestBridge {
            calls: Mutex::new(Vec::new()),
            fail_command: None,
        };

        let steps = vec![PluginCommandStep {
            command: "select.invert".to_string(),
            params: json!({}),
            save_as: None,
            ignore_errors: false,
            when: Some("has_selection".to_string()),
            description: None,
        }];

        let mut ctx = ExecutionContext {
            has_selection: false,
            ..Default::default()
        };

        execute_steps(&steps, &mut ctx, &bridge).await.unwrap();
        assert_eq!(bridge.calls.lock().unwrap().len(), 0);

        ctx.has_selection = true;
        execute_steps(&steps, &mut ctx, &bridge).await.unwrap();
        assert_eq!(bridge.calls.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_execute_steps_ignore_errors() {
        let bridge = TestBridge {
            calls: Mutex::new(Vec::new()),
            fail_command: Some("layer.delete".to_string()),
        };

        let steps = vec![PluginCommandStep {
            command: "layer.delete".to_string(),
            params: json!({}),
            save_as: None,
            ignore_errors: true,
            when: None,
            description: None,
        }];

        let mut ctx = ExecutionContext::default();
        let res = execute_steps(&steps, &mut ctx, &bridge).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_smart_retry_with_localized_blend_mode() {
        struct MockLocalizedBridge;
        #[async_trait]
        impl BridgeOperations for MockLocalizedBridge {
            async fn execute_command(
                &self,
                command: &str,
                params: serde_json::Value,
            ) -> Result<serde_json::Value> {
                if command == "layer.setProps" {
                    if let Some(b) = params.get("blend").and_then(|v| v.as_str()) {
                        if b == "Soft Light" {
                            return Err(CompanionError::Bridge(
                                "unknown blend mode `Soft Light`".to_string(),
                            ));
                        } else if b == "Weiches Licht" {
                            return Ok(json!({ "status": "ok", "applied": "Weiches Licht" }));
                        }
                    }
                }
                Ok(json!({ "status": "ok" }))
            }
            async fn copy_active_layer_image(&self) -> Result<RgbaImage> {
                Ok(RgbaImage::new(1, 1))
            }
            async fn paste_image_in_place(&self, _image: &RgbaImage) -> Result<serde_json::Value> {
                Ok(json!({}))
            }
            async fn rename_layer(
                &self,
                _layer_id: u64,
                _new_name: &str,
            ) -> Result<serde_json::Value> {
                Ok(json!({}))
            }
            async fn get_document_info(&self) -> Result<DocumentInfo> {
                Ok(DocumentInfo::default())
            }
        }

        let bridge = MockLocalizedBridge;
        let steps = vec![PluginCommandStep {
            command: "layer.setProps".to_string(),
            params: json!({ "blend": "Soft Light" }),
            save_as: Some("result".to_string()),
            ignore_errors: false,
            when: None,
            description: None,
        }];

        let mut ctx = ExecutionContext {
            host_language: Some("de".to_string()),
            ..Default::default()
        };

        let res = execute_steps(&steps, &mut ctx, &bridge).await;
        assert!(
            res.is_ok(),
            "Expected retry to succeed with German blend mode"
        );
        assert_eq!(ctx.custom_vars["result"]["applied"], "Weiches Licht");
    }

    #[tokio::test]
    async fn test_translate_value_to_host() {
        let input = json!({
            "blend": "Soft Light",
            "opacity": 0.85,
            "nested": {
                "mode": "Linear Light"
            }
        });

        let (translated, changed) = translate_value_to_host(&input, "de");
        assert!(changed);
        assert_eq!(translated["blend"], "Weiches Licht");
        assert_eq!(translated["opacity"], 0.85);
        assert_eq!(translated["nested"]["mode"], "Lineares Licht");
    }
}
