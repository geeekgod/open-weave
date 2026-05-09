use openweave_core::error::{Result, WeaveError};
use openweave_core::tools::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::fs;

pub struct FileOpsTool;

impl FileOpsTool {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileOpsTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileOpsTool {
    fn name(&self) -> &str {
        "file_ops"
    }

    fn description(&self) -> &str {
        "Read or write files to the local disk"
    }

    fn schema(&self) -> Value {
        json!({
            "name": "file_ops",
            "description": "Read or write files to the local disk",
            "parameters": {
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["read", "write"], "description": "The action to perform: read or write" },
                    "path": { "type": "string", "description": "The path to the file" },
                    "content": { "type": "string", "description": "The content to write to the file (only for write action)" }
                },
                "required": ["action", "path"]
            }
        })
    }

    async fn execute(&self, input: Value) -> Result<String> {
        let action = input.get("action").and_then(|v| v.as_str()).unwrap_or("");
        let path = input.get("path").and_then(|v| v.as_str()).unwrap_or("");

        match action {
            "read" => {
                let content = fs::read_to_string(path)?;
                Ok(content)
            }
            "write" => {
                let content = input.get("content").and_then(|v| v.as_str()).unwrap_or("");
                fs::write(path, content)?;
                Ok(format!("Successfully wrote to {}", path))
            }
            _ => Err(WeaveError::ToolNotFound(format!("Invalid action for file_ops: {}", action))),
        }
    }
}