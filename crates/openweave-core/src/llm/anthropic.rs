use super::{LLMProvider, Message, Role, ToolCall};
use crate::error::{Result, WeaveError};
use async_trait::async_trait;
use futures::stream::Stream;
use reqwest::Client;
use serde_json::json;

pub struct AnthropicProvider {
    client: Client,
    model: String,
    api_key: String,
}

impl AnthropicProvider {
    pub fn new(model: impl Into<String>) -> Self {
        let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
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
impl LLMProvider for AnthropicProvider {
    async fn complete(&self, messages: &[Message], tools: &[serde_json::Value]) -> Result<Message> {
        let mut system_prompts = Vec::new();
        let mut anthropic_messages = Vec::new();

        for m in messages {
            match m.role {
                Role::System => {
                    system_prompts.push(m.content.clone());
                }
                Role::User => {
                    anthropic_messages.push(json!({
                        "role": "user",
                        "content": m.content
                    }));
                }
                Role::Assistant => {
                    let mut content_array = Vec::new();
                    if !m.content.is_empty() {
                        content_array.push(json!({
                            "type": "text",
                            "text": m.content
                        }));
                    }
                    if let Some(calls) = &m.tool_calls {
                        for c in calls {
                            content_array.push(json!({
                                "type": "tool_use",
                                "id": c.id,
                                "name": c.name,
                                "input": serde_json::from_str::<serde_json::Value>(&c.arguments).unwrap_or(json!({}))
                            }));
                        }
                    }
                    anthropic_messages.push(json!({
                        "role": "assistant",
                        "content": content_array
                    }));
                }
                Role::Tool => {
                    let tool_use_id = m.tool_calls.as_ref()
                        .and_then(|calls| calls.first())
                        .map(|c| c.id.clone())
                        .unwrap_or_else(|| "unknown".to_string());

                    anthropic_messages.push(json!({
                        "role": "user",
                        "content": [
                            {
                                "type": "tool_result",
                                "tool_use_id": tool_use_id,
                                "content": m.content
                            }
                        ]
                    }));
                }
            }
        }

        let mut body = json!({
            "model": self.model,
            "max_tokens": 4096,
            "messages": anthropic_messages,
        });

        if !system_prompts.is_empty() {
            body["system"] = json!(system_prompts.join("\n"));
        }

        if !tools.is_empty() {
            let mut mapped_tools = Vec::new();
            for t in tools {
                let mut tool_schema = t.clone();
                if let Some(params) = tool_schema.get("parameters").cloned() {
                    let obj = tool_schema.as_object_mut().ok_or_else(|| {
                        WeaveError::LlmError("Invalid tool schema: not an object".into())
                    })?;
                    obj.remove("parameters");
                    obj.insert("input_schema".to_string(), params);
                }
                mapped_tools.push(tool_schema);
            }
            body["tools"] = json!(mapped_tools);
        }

        let res = self.client.post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| WeaveError::LlmError(e.to_string()))?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(WeaveError::LlmError(format!("Anthropic API error: {}", err_text)));
        }

        let json_res: serde_json::Value = res.json().await
            .map_err(|e| WeaveError::LlmError(e.to_string()))?;

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        if let Some(content_blocks) = json_res["content"].as_array() {
            for block in content_blocks {
                if block["type"] == "text" {
                    content.push_str(block["text"].as_str().unwrap_or(""));
                } else if block["type"] == "tool_use" {
                    tool_calls.push(ToolCall {
                        id: block["id"].as_str().unwrap_or("").to_string(),
                        name: block["name"].as_str().unwrap_or("").to_string(),
                        arguments: block["input"].to_string(),
                    });
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