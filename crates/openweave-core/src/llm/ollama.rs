use super::{LLMProvider, Message, Role, ToolCall};
use crate::error::{Result, WeaveError};
use async_trait::async_trait;
use futures::stream::Stream;
use reqwest::Client;
use serde_json::json;

pub struct OllamaProvider {
    client: Client,
    model: String,
    base_url: String,
}

impl OllamaProvider {
    pub fn new(model: impl Into<String>) -> Self {
        let base_url = std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".into());
        Self {
            client: Client::new(),
            model: model.into(),
            base_url,
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}

#[async_trait]
impl LLMProvider for OllamaProvider {
    async fn complete(&self, messages: &[Message], tools: &[serde_json::Value]) -> Result<Message> {
        let mut api_messages = Vec::new();
        
        for m in messages {
            let role = match m.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::Tool => "tool",
            };
            
            let mut msg = json!({
                "role": role,
                "content": m.content,
            });
            
            if let Some(calls) = &m.tool_calls {
                let mut tool_calls = Vec::new();
                for c in calls {
                    let parsed_args = serde_json::from_str::<serde_json::Value>(&c.arguments)
                        .map_err(|e| WeaveError::LlmError(format!("Invalid tool arguments JSON: {}", e)))?;
                        
                    tool_calls.push(json!({
                        "function": {
                            "name": c.name,
                            "arguments": parsed_args
                        }
                    }));
                }
                msg["tool_calls"] = json!(tool_calls);
            }
            api_messages.push(msg);
        }

        let mut body = json!({
            "model": self.model,
            "stream": false,
            "messages": api_messages
        });

        if !tools.is_empty() {
            body["tools"] = json!(tools.iter().map(|t| {
                json!({
                    "type": "function",
                    "function": t
                })
            }).collect::<Vec<_>>());
        }

        let res = self.client.post(format!("{}/api/chat", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| WeaveError::LlmError(e.to_string()))?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(WeaveError::LlmError(format!("Ollama API error: {}", err_text)));
        }

        let json_res: serde_json::Value = res.json().await
            .map_err(|e| WeaveError::LlmError(e.to_string()))?;

        let message = &json_res["message"];
        let content = message["content"].as_str().unwrap_or("").to_string();
        
        let tool_calls = if let Some(calls) = message["tool_calls"].as_array() {
            let mut result = Vec::new();
            for (i, c) in calls.iter().enumerate() {
                result.push(ToolCall {
                    id: format!("call_{}", i), // Ollama might not return an id, so generate one
                    name: c["function"]["name"].as_str().unwrap_or("").to_string(),
                    arguments: c["function"]["arguments"].to_string(),
                });
            }
            if result.is_empty() { None } else { Some(result) }
        } else {
            None
        };

        Ok(Message {
            role: Role::Assistant,
            content,
            tool_calls,
        })
    }

    async fn stream(
        &self,
        _messages: &[Message],
        _tools: &[serde_json::Value],
    ) -> Result<std::pin::Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        Err(WeaveError::LlmError("Stream not implemented".into()))
    }
}