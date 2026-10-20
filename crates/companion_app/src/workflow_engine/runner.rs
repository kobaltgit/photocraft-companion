use super::resolver::{resolve_value, ExecutionContext};
use core_contracts::{BridgeOperations, PluginCommandStep, Result};
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

        let resolved_params = resolve_value(&step.params, ctx);
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

        match client.execute_command(&step.command, resolved_params).await {
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

        let mut ctx = ExecutionContext::default();
        ctx.source_layer_id = Some(10);

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

        let mut ctx = ExecutionContext::default();
        ctx.has_selection = false;

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
}
