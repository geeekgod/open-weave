pub mod anthropic;
pub mod google;
pub mod groq;
pub mod ollama;
pub mod openai;

use crate::error::Result;
use async_trait::async_trait;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, messages: &[Message], tools: &[serde_json::Value]) -> Result<Message>;
    async fn stream(
        &self,
        messages: &[Message],
        tools: &[serde_json::Value],
    ) -> Result<std::pin::Pin<Box<dyn Stream<Item = Result<String>> + Send>>>;
}