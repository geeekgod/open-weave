use super::{LLMProvider, Message, Role, ToolCall};
use crate::error::{Result, WeaveError};
use async_trait::async_trait;
use futures::stream::Stream;
use reqwest::Client;
use serde_json::json;

pub struct OpenAIProvider {
    client: Client,
    model: String,
    api_key: String,
}

impl OpenAIProvider {
    /// Create a new OpenAIProvider configured for the specified model.
    ///
    /// The returned provider is initialized with a fresh HTTP client, the given model identifier,
    /// and the API key read from the `OPENAI_API_KEY` environment variable (empty string if unset).
    ///
    /// # Examples
    ///
    /// ```
    /// let provider = OpenAIProvider::new("gpt-4");
    /// // provider can now be used to perform completions (e.g., provider.complete(...))
    /// ```
    pub fn new(model: impl Into<String>) -> Self {
        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
        Self {
            client: Client::new(),
            model: model.into(),
            api_key,
        }
    }

    /// Sets the OpenAI API key on the provider and returns the modified instance for method chaining.
    ///
    /// Allows overriding the API key that was read from the `OPENAI_API_KEY` environment variable when
    /// the provider was created.
    ///
    /// # Examples
    ///
    /// ```
    /// let provider = OpenAIProvider::new("gpt-4").with_api_key("sk-xxxxxxxx");
    /// // provider can now be used with the provided API key
    /// ```
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = api_key.into();
        self
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    /// Send the provided conversation and optional tool definitions to OpenAI's Chat Completions API and return the assistant's reply as a `Message`.
    ///
    /// The request body includes `model`, the supplied `messages` converted to the OpenAI chat format, and `tools` when provided. If the API response contains `tool_calls`, they are parsed into the returned `Message`'s `tool_calls` field.
    ///
    /// # Parameters
    ///
    /// - `messages`: The conversation history to send; each `Message` will be translated to the corresponding OpenAI chat message and may include function call hints.
    /// - `tools`: Optional JSON values describing functions to expose to the model; each entry is sent as a `type: "function"` with the provided `function` object.
    ///
    /// # Returns
    ///
    /// A `Message` containing the assistant's content and, when present, a vector of parsed `ToolCall`s in `tool_calls`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use openweave_core::llm::{OpenAIProvider, Message, Role};
    /// # async fn doc_example() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider = OpenAIProvider::new("gpt-4").with_api_key("sk-test");
    /// let messages = [Message { role: Role::User, content: "Say hello".into(), tool_calls: None }];
    /// let reply = provider.complete(&messages, &[]).await?;
    /// assert!(reply.content.len() > 0);
    /// # Ok(())
    /// # }
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
                
                if role == "tool" {
                    if let Some(calls) = &m.tool_calls {
                        if let Some(call) = calls.first() {
                            msg["tool_call_id"] = json!(call.id);
                        }
                    } else {
                        msg["tool_call_id"] = json!("unknown");
                    }
                }
                
                if let Some(calls) = &m.tool_calls {
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

        let res = self.client.post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| WeaveError::LlmError(e.to_string()))?;

        if !res.status().is_success() {
            let err_text = res.text().await.unwrap_or_default();
            return Err(WeaveError::LlmError(format!("OpenAI API error: {}", err_text)));
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

    /// Provides a streaming completion interface for the provider; currently unimplemented.
    ///
    /// # Examples
    ///
    /// ```
    /// # use openweave_core::llm::openai::OpenAIProvider;
    /// # use openweave_core::llm::{Message, Role};
    /// # tokio_test::block_on(async {
    /// let provider = OpenAIProvider::new("gpt-4");
    /// let messages: Vec<Message> = vec![];
    /// let tools: Vec<serde_json::Value> = vec![];
    /// let res = provider.stream(&messages, &tools).await;
    /// assert!(res.is_err());
    /// # });
    /// ```
    ///
    /// # Returns
    ///
    /// An error `WeaveError::LlmError` with the message `"Stream not implemented"`.
    async fn stream(
        &self,
        _messages: &[Message],
        _tools: &[serde_json::Value],
    ) -> Result<std::pin::Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        Err(WeaveError::LlmError("Stream not implemented".into()))
    }
}