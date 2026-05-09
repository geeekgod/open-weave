use super::registry::ToolRegistry;
use crate::error::{Result, WeaveError};
use crate::llm::ToolCall;
use futures::future::join_all;
use std::sync::Arc;
use tokio::time::{timeout, Duration};

pub struct ToolExecutor {
    registry: Arc<ToolRegistry>,
}

impl ToolExecutor {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self { registry }
    }

    pub async fn execute_all(&self, calls: Vec<ToolCall>) -> Vec<Result<String>> {
        let futures = calls.into_iter().map(|call| {
            let registry = Arc::clone(&self.registry);
            async move {
                let input = serde_json::from_str(&call.arguments)
                    .map_err(|e| WeaveError::SerdeError(e))?;
                
                // 30s timeout per tool execution
                match timeout(Duration::from_secs(30), registry.execute(&call.name, input)).await {
                    Ok(res) => res,
                    Err(_) => Err(WeaveError::ToolTimeout),
                }
            }
        });

        join_all(futures).await
    }
}