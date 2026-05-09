use super::Tool;
use crate::error::{Result, WeaveError};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: impl Tool + 'static) {
        self.tools.insert(tool.name().to_string(), Arc::new(tool));
    }

    pub fn get_schemas(&self) -> Vec<Value> {
        self.tools.values().map(|t| t.schema()).collect()
    }

    pub async fn execute(&self, name: &str, input: Value) -> Result<String> {
        if let Some(tool) = self.tools.get(name) {
            tool.execute(input).await
        } else {
            Err(WeaveError::ToolNotFound(name.into()))
        }
    }
}