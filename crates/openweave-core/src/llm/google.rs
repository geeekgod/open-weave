use super::{LLMProvider, Message, Role, ToolCall};
use crate::error::{Result, WeaveError};
use async_trait::async_trait;
use futures::stream::Stream;
use reqwest::Client;
use serde_json::json;

pub struct GoogleProvider {
    client: Client,
    model: String,
    api_key: String,
}

impl GoogleProvider {
    pub fn new(model: impl Into<String>) -> Self {
        let api_key = std::env::var("GOOGLE_API_KEY").unwrap_or_default();
        Self {
            client: Client::new(),
            model: model.into(),
            api_key,
        }
    }

    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = api_key.into();
        self
    }
}

#[async_trait]
impl LLMProvider for GoogleProvider {
    async fn complete(&self, messages: &[Message], tools: &[serde_json::Value]) -> Result<Message> {
        let mut contents = Vec::new();
        let mut system_instruction = None;

        for m in messages {
            match m.role {
                Role::System => {
                    system_instruction = Some(json!({
                        "parts": [{ "text": m.content }]
                    }));
                }
                Role::User => {
                    contents.push(json!({
                        "role": "user",
                        "parts": [{ "text": m.content }]
                    }));
                }
                Role::Assistant => {
                    let mut parts = Vec::new();
                    if !m.content.is_empty() {
                        parts.push(json!({ "text": m.content }));
                    }
                    if let Some(calls) = &m.tool_calls {
                        for c in calls {
                            parts.push(json!({
                                "functionCall": {
                                    "name": c.name,
                                    "args": serde_json::from_str::<serde_json::Value>(&c.arguments).unwrap_or(json!({}))
                                }
                            }));
                        }
                    }
                    contents.push(json!({
                        "role": "model",
                        "parts": parts
                    }));
                }
                Role::Tool => {
                    let tool_name = m.tool_calls.as_ref()
                        .and_then(|calls| calls.first())
                        .map(|c| c.name.clone())
                        .unwrap_or_else(|| "unknown".to_string());
                        
                    contents.push(json!({
                        "role": "function",
                        "parts": [{
                            "functionResponse": {
                                "name": tool_name,
                                "response": {
                                    "result": m.content
                                }
                            }
                        }]
                    }));
                }
            }
        }

        let mut body = json!({
            "contents": contents
        });

        if let Some(sys) = system_instruction {
            body["systemInstruction"] = sys;
        }

        if !tools.is_empty() {
            body["tools"] = json!([{
                "functionDeclarations": tools
            }]);
        }

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let res = self.client.post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| WeaveError::LlmError(e.to_string()))?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(WeaveError::LlmError(format!("Google API error: {}", err_text)));
        }

        let json_res: serde_json::Value = res.json().await
            .map_err(|e| WeaveError::LlmError(e.to_string()))?;

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        if let Some(candidates) = json_res["candidates"].as_array() {
            if let Some(candidate) = candidates.first() {
                if let Some(parts) = candidate["content"]["parts"].as_array() {
                    for (i, part) in parts.iter().enumerate() {
                        if let Some(text) = part["text"].as_str() {
                            content.push_str(text);
                        } else if let Some(func_call) = part.get("functionCall") {
                            tool_calls.push(ToolCall {
                                id: format!("call_{}", i), // Google doesn't use IDs like OpenAI
                                name: func_call["name"].as_str().unwrap_or("").to_string(),
                                arguments: func_call["args"].to_string(),
                            });
                        }
                    }
                }
            }
        }

        Ok(Message {
            role: Role::Assistant,
            content,
            tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
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