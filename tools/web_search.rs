use openweave_core::error::Result;
use openweave_core::tools::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct WebSearchTool;

impl WebSearchTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web for a query"
    }

    fn schema(&self) -> Value {
        json!({
            "name": "web_search",
            "parameters": {
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                },
                "required": ["query"]
            }
        })
    }

    async fn execute(&self, input: Value) -> Result<String> {
        let _query = input.get("query").and_then(|v| v.as_str()).unwrap_or("");
        Ok("Mock search results...".into())
    }
}