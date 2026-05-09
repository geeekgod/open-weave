use super::{LLMProvider, Message};
use crate::error::{Result, WeaveError};
use async_trait::async_trait;
use futures::stream::Stream;
use reqwest::Client;

pub struct OllamaProvider {
    client: Client,
    model: String,
    base_url: String,
}

impl OllamaProvider {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            model: model.into(),
            base_url: "http://localhost:11434".into(),
        }
    }
}

#[async_trait]
impl LLMProvider for OllamaProvider {
    async fn complete(&self, _messages: &[Message], _tools: &[serde_json::Value]) -> Result<Message> {
        Err(WeaveError::LlmError("Not implemented".into()))
    }

    async fn stream(
        &self,
        _messages: &[Message],
        _tools: &[serde_json::Value],
    ) -> Result<std::pin::Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        Err(WeaveError::LlmError("Not implemented".into()))
    }
}