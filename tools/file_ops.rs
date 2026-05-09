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

#[async_trait]
impl Tool for FileOpsTool {
    fn name(&self) -> &str {
        "file_ops"
    }

    fn description(&self) -> &str {
        "Read or write files"
    }

    fn schema(&self) -> Value {
        json!({
            "name": "file_ops",
            "parameters": {
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["read", "write"] },
                    "path": { "type": "string" },
                    "content": { "type": "string" }
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
                Ok("Success".into())
            }
            _ => Err(WeaveError::ToolNotFound("Invalid action".into())),
        }
    }
}