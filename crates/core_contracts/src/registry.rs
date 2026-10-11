use crate::{
    CompanionError, Plugin, PluginContext, PluginManifest, PluginResult, PluginType, Result,
};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Generic No-Code Plugin executing macro steps declared in `plugin.json`
pub struct NoCodePlugin {
    manifest: PluginManifest,
}

impl NoCodePlugin {
    pub fn new(manifest: PluginManifest) -> Self {
        Self { manifest }
    }
}

#[async_trait::async_trait]
impl Plugin for NoCodePlugin {
    fn id(&self) -> &str {
        &self.manifest.id
    }

    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    async fn execute(&self, ctx: &PluginContext) -> Result<PluginResult> {
        info!(
            "Executing No-Code plugin '{}' ({} steps)",
            self.manifest.name,
            self.manifest.commands.len()
        );

        let mut last_output = serde_json::Value::Null;
        for (idx, step) in self.manifest.commands.iter().enumerate() {
            if let Some(desc) = &step.description {
                debug!(
                    "  Step [{}/{}]: {} ({})",
                    idx + 1,
                    self.manifest.commands.len(),
                    step.command,
                    desc
                );
            } else {
                debug!(
                    "  Step [{}/{}]: {}",
                    idx + 1,
                    self.manifest.commands.len(),
                    step.command
                );
            }

            match ctx
                .client
                .execute_command(&step.command, step.params.clone())
                .await
            {
                Ok(val) => {
                    last_output = val;
                }
                Err(e) => {
                    error!("Step failed at command '{}': {}", step.command, e);
                    return Err(e);
                }
            }
        }

        Ok(PluginResult::ok_with_data(
            format!(
                "No-Code plugin '{}' completed successfully",
                self.manifest.name
            ),
            last_output,
        ))
    }
}

/// Registry that scans, loads, and manages all plugins in the system
pub struct PluginRegistry {
    plugins: HashMap<String, Arc<dyn Plugin>>,
    manifests: Vec<PluginManifest>,
    plugins_dir: PathBuf,
}

impl PluginRegistry {
    pub fn new(plugins_dir: impl AsRef<Path>) -> Self {
        Self {
            plugins: HashMap::new(),
            manifests: Vec::new(),
            plugins_dir: plugins_dir.as_ref().to_path_buf(),
        }
    }

    /// Register a native compiled plugin (e.g. RemoveBgPlugin)
    pub fn register_native(&mut self, plugin: Arc<dyn Plugin>) {
        let manifest = plugin.manifest().clone();
        self.plugins.insert(manifest.id.clone(), plugin);
        self.manifests.push(manifest);
    }

    /// Scan directory for `plugin.json` manifests and instantiate plugins
    pub fn scan_and_load(&mut self) -> Result<usize> {
        if !self.plugins_dir.exists() {
            debug!(
                "Plugins directory does not exist: {}",
                self.plugins_dir.display()
            );
            return Ok(0);
        }

        let mut loaded_count = 0;
        let entries = fs::read_dir(&self.plugins_dir)?;

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let manifest_file = path.join("plugin.json");
                if manifest_file.exists() {
                    match fs::read_to_string(&manifest_file) {
                        Ok(content) => match serde_json::from_str::<PluginManifest>(&content) {
                            Ok(mut manifest) => {
                                manifest.plugin_dir = Some(path.clone());
                                if !manifest.enabled {
                                    info!(
                                        "Skipping disabled plugin: {} ({})",
                                        manifest.name, manifest.id
                                    );
                                    continue;
                                }

                                match manifest.plugin_type {
                                    PluginType::NoCode => {
                                        info!(
                                            "Loaded No-Code plugin: {} [{}]",
                                            manifest.name, manifest.id
                                        );
                                        let plugin = Arc::new(NoCodePlugin::new(manifest.clone()));
                                        self.plugins.insert(manifest.id.clone(), plugin);
                                        self.manifests.push(manifest);
                                        loaded_count += 1;
                                    }
                                    PluginType::NativeAi => {
                                        info!(
                                            "Found Native AI plugin manifest: {} [{}]",
                                            manifest.name, manifest.id
                                        );
                                        // Manifest is tracked; instance registered via register_native
                                        if !self.plugins.contains_key(&manifest.id) {
                                            self.manifests.push(manifest);
                                        }
                                    }
                                    PluginType::Script => {
                                        info!(
                                            "Found Script plugin manifest: {} [{}]",
                                            manifest.name, manifest.id
                                        );
                                        self.manifests.push(manifest);
                                    }
                                    PluginType::ExternalCli => {
                                        info!(
                                            "Found External CLI plugin manifest: {} [{}]",
                                            manifest.name, manifest.id
                                        );
                                        self.manifests.push(manifest);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!(
                                    "Failed to parse plugin manifest {}: {}",
                                    manifest_file.display(),
                                    e
                                );
                            }
                        },
                        Err(e) => {
                            warn!(
                                "Failed to read plugin manifest {}: {}",
                                manifest_file.display(),
                                e
                            );
                        }
                    }
                }
            }
        }

        info!(
            "Plugin registry loaded {} plugins from {}",
            loaded_count,
            self.plugins_dir.display()
        );
        Ok(loaded_count)
    }

    /// Retrieve plugin by ID
    pub fn get(&self, id: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins.get(id).cloned()
    }

    /// List all plugin manifests
    pub fn list_manifests(&self) -> &[PluginManifest] {
        &self.manifests
    }

    /// Execute plugin by ID
    pub async fn execute(&self, id: &str, ctx: &PluginContext) -> Result<PluginResult> {
        let plugin = self.get(id).ok_or_else(|| {
            CompanionError::Plugin(format!("Plugin with id '{}' not found in registry", id))
        })?;
        plugin.execute(ctx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BridgeOperations, DocumentInfo, PluginCommandStep};
    use async_trait::async_trait;
    use image::RgbaImage;
    use serde_json::json;
    use std::sync::Mutex;

    struct MockBridge {
        executed: Mutex<Vec<(String, serde_json::Value)>>,
        fail_on: Option<String>,
    }

    impl MockBridge {
        fn new() -> Self {
            Self {
                executed: Mutex::new(Vec::new()),
                fail_on: None,
            }
        }

        fn with_fail_on(command: &str) -> Self {
            Self {
                executed: Mutex::new(Vec::new()),
                fail_on: Some(command.to_string()),
            }
        }
    }

    #[async_trait]
    impl BridgeOperations for MockBridge {
        async fn execute_command(
            &self,
            command: &str,
            params: serde_json::Value,
        ) -> Result<serde_json::Value> {
            if let Some(ref fail) = self.fail_on {
                if fail == command {
                    return Err(CompanionError::Bridge(format!("Mock failure on {command}")));
                }
            }
            self.executed
                .lock()
                .unwrap()
                .push((command.to_string(), params.clone()));
            Ok(json!({ "status": "ok", "cmd": command }))
        }

        async fn copy_active_layer_image(&self) -> Result<RgbaImage> {
            Ok(RgbaImage::new(10, 10))
        }

        async fn paste_image_in_place(&self, _image: &RgbaImage) -> Result<serde_json::Value> {
            Ok(json!({ "layer": 101 }))
        }

        async fn rename_layer(&self, layer_id: u64, new_name: &str) -> Result<serde_json::Value> {
            Ok(json!({ "renamed": layer_id, "name": new_name }))
        }

        async fn get_document_info(&self) -> Result<DocumentInfo> {
            Ok(DocumentInfo {
                name: "TestDoc".to_string(),
                width: 800,
                height: 600,
                active_layer: Some(1),
                active_layer_name: Some("Layer 1".to_string()),
            })
        }
    }

    #[tokio::test]
    async fn test_nocode_plugin_execute_success() {
        let manifest = PluginManifest {
            id: "macro.test".to_string(),
            name: "Test Macro".to_string(),
            version: "1.0.0".to_string(),
            description: "Test macro".to_string(),
            author: "Tester".to_string(),
            plugin_type: PluginType::NoCode,
            default_hotkey: None,
            icon: None,
            commands: vec![
                PluginCommandStep {
                    command: "layer.duplicate".to_string(),
                    params: json!({ "name": "Copy" }),
                    description: Some("Duplicate layer".to_string()),
                },
                PluginCommandStep {
                    command: "filter.blur".to_string(),
                    params: json!({ "radius": 5 }),
                    description: None,
                },
            ],
            models: vec![],
            action_id: None,
            entrypoint: None,
            enabled: true,
            plugin_dir: None,
        };

        let plugin = NoCodePlugin::new(manifest);
        let mock_bridge = Arc::new(MockBridge::new());
        let ctx = PluginContext {
            document: DocumentInfo {
                name: "Doc".to_string(),
                width: 100,
                height: 100,
                active_layer: Some(1),
                active_layer_name: Some("L1".to_string()),
            },
            models_dir: PathBuf::from("models"),
            plugin_dir: PathBuf::from("plugins/test"),
            client: mock_bridge.clone(),
        };

        let result = plugin.execute(&ctx).await.unwrap();
        assert!(result.success);
        let executed = mock_bridge.executed.lock().unwrap();
        assert_eq!(executed.len(), 2);
        assert_eq!(executed[0].0, "layer.duplicate");
        assert_eq!(executed[1].0, "filter.blur");
    }

    #[tokio::test]
    async fn test_nocode_plugin_execute_failure() {
        let manifest = PluginManifest {
            id: "macro.fail".to_string(),
            name: "Fail Macro".to_string(),
            version: "1.0.0".to_string(),
            description: "Failing macro".to_string(),
            author: "Tester".to_string(),
            plugin_type: PluginType::NoCode,
            default_hotkey: None,
            icon: None,
            commands: vec![PluginCommandStep {
                command: "fail.cmd".to_string(),
                params: json!({}),
                description: None,
            }],
            models: vec![],
            action_id: None,
            entrypoint: None,
            enabled: true,
            plugin_dir: None,
        };

        let plugin = NoCodePlugin::new(manifest);
        let mock_bridge = Arc::new(MockBridge::with_fail_on("fail.cmd"));
        let ctx = PluginContext {
            document: DocumentInfo {
                name: "Doc".to_string(),
                width: 100,
                height: 100,
                active_layer: None,
                active_layer_name: None,
            },
            models_dir: PathBuf::from("models"),
            plugin_dir: PathBuf::from("plugins/test"),
            client: mock_bridge,
        };

        let result = plugin.execute(&ctx).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_plugin_registry_scan_and_load() {
        let temp_dir = std::env::temp_dir().join(format!(
            "registry_scan_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let p1 = temp_dir.join("p1");
        fs::create_dir_all(&p1).unwrap();
        fs::write(
            p1.join("plugin.json"),
            r#"{
            "id": "p1.id",
            "name": "Plugin One",
            "version": "1.0",
            "description": "desc",
            "plugin_type": "no_code",
            "commands": []
        }"#,
        )
        .unwrap();

        let p2_disabled = temp_dir.join("p2_disabled");
        fs::create_dir_all(&p2_disabled).unwrap();
        fs::write(
            p2_disabled.join("plugin.json"),
            r#"{
            "id": "p2.disabled",
            "name": "Disabled Plugin",
            "version": "1.0",
            "description": "desc",
            "plugin_type": "no_code",
            "enabled": false
        }"#,
        )
        .unwrap();

        let mut registry = PluginRegistry::new(&temp_dir);
        let count = registry.scan_and_load().unwrap();
        assert_eq!(count, 1);
        assert!(registry.get("p1.id").is_some());
        assert!(registry.get("p2.disabled").is_none());

        let manifests = registry.list_manifests();
        assert_eq!(manifests.len(), 1);
        assert_eq!(manifests[0].id, "p1.id");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn test_plugin_registry_execute_missing() {
        let registry = PluginRegistry::new("non_existent");
        let mock_bridge = Arc::new(MockBridge::new());
        let ctx = PluginContext {
            document: DocumentInfo {
                name: "Doc".to_string(),
                width: 10,
                height: 10,
                active_layer: None,
                active_layer_name: None,
            },
            models_dir: PathBuf::from("models"),
            plugin_dir: PathBuf::from("plugins"),
            client: mock_bridge,
        };

        let res = registry.execute("unknown.plugin", &ctx).await;
        assert!(res.is_err());
    }
}
