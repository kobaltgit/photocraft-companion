use core_contracts::{CompanionError, Result};
use rhai::{Engine, Scope, AST};

/// Sandboxed Rhai scripting engine interface for PhotoCraft companion automation
pub struct RhaiBridgeEngine {
    engine: Engine,
}

impl Default for RhaiBridgeEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl RhaiBridgeEngine {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        engine.set_max_expr_depths(64, 64);
        engine.set_max_operations(50_000);

        Self { engine }
    }

    #[allow(dead_code)]
    pub fn compile(&self, script: &str) -> Result<AST> {
        self.engine
            .compile(script)
            .map_err(|e| CompanionError::Plugin(format!("Rhai compile error: {e}")))
    }

    #[allow(dead_code)]
    pub fn eval<T: Clone + Send + Sync + 'static>(&self, script: &str) -> Result<T> {
        self.engine
            .eval::<T>(script)
            .map_err(|e| CompanionError::Plugin(format!("Rhai eval error: {e}")))
    }

    #[allow(dead_code)]
    pub fn eval_with_scope<T: Clone + Send + Sync + 'static>(
        &self,
        scope: &mut Scope,
        script: &str,
    ) -> Result<T> {
        self.engine
            .eval_with_scope::<T>(scope, script)
            .map_err(|e| CompanionError::Plugin(format!("Rhai eval error: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rhai_bridge_basic_eval() {
        let bridge = RhaiBridgeEngine::new();
        let val: i64 = bridge.eval("40 + 2").unwrap();
        assert_eq!(val, 42);
    }
}
