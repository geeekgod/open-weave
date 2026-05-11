use super::{LLMProvider, Message, Role, ToolCall};
use crate::error::{Result, WeaveError};
use async_trait::async_trait;
use futures::stream::Stream;
use reqwest::Client;
use serde_json::json;

pub struct GroqProvider {
    client: Client,
    model: String,
    api_key: String,
}

impl GroqProvider {
    /// Create a new `GroqProvider` for the specified model.
    ///
    /// Reads the `GROQ_API_KEY` environment variable and stores its value (defaults to an empty
    /// string if the variable is not set), initializes an HTTP client, and sets the provider's model.
    ///
    /// # Examples
    ///
    /// ```
    /// let _provider = openweave_core::llm::groq::GroqProvider::new("gpt-model");
    /// ```
    pub fn new(model: impl Into<String>) -> Self {
        let api_key = std::env::var("GROQ_API_KEY").unwrap_or_default();
        Self {
            client: Client::new(),
            model: model.into(),
            api_key,
        }
    }

    /// Replace the provider's API key and return the updated provider.
    ///
    /// Sets the provider's stored API key to the provided value and returns the modified `GroqProvider`.
    ///
    /// # Returns
    ///
    /// `Self` with its `api_key` replaced by the provided value.
    ///
    /// # Examples
    ///
    /// ```
    /// let p = GroqProvider::new("gpt-model").with_api_key("my-key");
    /// let _ = p;
    /// ```
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = api_key.into();
        self
    }
}

#[async_trait]
impl LLMProvider for GroqProvider {
    /// Send the given chat messages to Groq's chat-completions endpoint and return the assistant's reply.
    ///
    /// The returned `Message` holds role `Role::Assistant`, the assistant `content`, and any parsed `tool_calls` if present in the response.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use openweave_core::llm::groq::GroqProvider;
    /// # use openweave_core::llm::{Message, Role};
    /// # use serde_json::json;
    /// # tokio_test::block_on(async {
    /// let provider = GroqProvider::new("groq-model");
    /// let messages = vec![Message { role: Role::User, content: "Hello".into(), tool_calls: None }];
    /// let tools: Vec<serde_json::Value> = Vec::new();
    /// let resp = provider.complete(&messages, &tools).await.unwrap();
    /// assert_eq!(resp.role, Role::Assistant);
    /// # });
    /// ```
    async fn complete(&self, messages: &[Message], tools: &[serde_json::Value]) -> Result<Message> {
        let mut body = json!({
            "model": self.model,
            "messages": messages.iter().map(|m| {
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
                
                if m.role == Role::Tool {
                    // Groq/OpenAI tool response needs `tool_call_id` and `name` at root level, not inside `tool_calls`
                    if let Some(calls) = &m.tool_calls {
                        if let Some(call) = calls.first() {
                            msg["tool_call_id"] = json!(call.id);
                            msg["name"] = json!(call.name);
                        }
                    } else {
                        // Fallback id if lost
                        msg["tool_call_id"] = json!("unknown");
                    }
                } else if let Some(calls) = &m.tool_calls {
                    let tool_calls: Vec<_> = calls.iter().map(|c| {
                        json!({
                            "id": c.id,
                            "type": "function",
                            "function": {
                                "name": c.name,
                                "arguments": c.arguments
                            }
                        })
                    }).collect();
                    msg["tool_calls"] = json!(tool_calls);
                }
                msg
            }).collect::<Vec<_>>()
        });

        if !tools.is_empty() {
            body["tools"] = json!(tools.iter().map(|t| {
                json!({
                    "type": "function",
                    "function": t
                })
            }).collect::<Vec<_>>());
        }

        let res = self.client.post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| WeaveError::LlmError(e.to_string()))?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(WeaveError::LlmError(format!("Groq API error: {}", err_text)));
        }

        let json_res: serde_json::Value = res.json().await
            .map_err(|e| WeaveError::LlmError(e.to_string()))?;

        let message = &json_res["choices"][0]["message"];
        let content = message["content"].as_str().unwrap_or("").to_string();
        
        let tool_calls = if let Some(calls) = message["tool_calls"].as_array() {
            let mut result = Vec::new();
            for c in calls {
                result.push(ToolCall {
                    id: c["id"].as_str().unwrap_or("").to_string(),
                    name: c["function"]["name"].as_str().unwrap_or("").to_string(),
                    arguments: c["function"]["arguments"].as_str().unwrap_or("").to_string(),
                });
            }
            Some(result)
        } else {
            None
        };

        Ok(Message {
            role: Role::Assistant,
            content,
            tool_calls,
        })
    }

    /// Indicates that streaming completions are not supported by this provider.
    ///
    /// This method always returns an error indicating streaming is not implemented.
    ///
    /// # Returns
    ///
    /// An `Err(WeaveError::LlmError)` with the message `"Stream not implemented"`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use openweave_core::llm::groq::GroqProvider;
    /// # use openweave_core::error::WeaveError;
    /// # tokio_test::block_on(async {
    /// let provider = GroqProvider::new("test-model");
    /// let res = provider.stream(&[], &[]).await;
    /// assert!(matches!(res, Err(WeaveError::LlmError(msg)) if msg.contains("Stream not implemented")));
    /// # });
    /// ```
    async fn stream(
        &self,
        _messages: &[Message],
        _tools: &[serde_json::Value],
    ) -> Result<std::pin::Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        Err(WeaveError::LlmError("Stream not implemented".into()))
    }
}