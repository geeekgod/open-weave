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
    /// Creates a new OllamaProvider configured for the given model.
    ///
    /// The provider will use the URL from the `OLLAMA_BASE_URL` environment variable if set;
    /// otherwise it defaults to `http://localhost:11434`. A new HTTP client is created and
    /// the given model name is stored for future requests.
    ///
    /// # Parameters
    ///
    /// - `model`: The name of the Ollama model to use.
    ///
    /// # Examples
    ///
    /// ```
    /// let _provider = OllamaProvider::new("ggml-mpt");
    /// ```
    pub fn new(model: impl Into<String>) -> Self {
        let base_url = std::env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".into());
        Self {
            client: Client::new(),
            model: model.into(),
            base_url,
        }
    }

    /// Sets the provider's base URL and returns the updated instance.
    ///
    /// # Examples
    ///
    /// ```
    /// let provider = OllamaProvider::new("my-model").with_base_url("http://localhost:11434");
    /// ```
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }
}

#[async_trait]
impl LLMProvider for OllamaProvider {
    /// Send the provided conversation to Ollama's /api/chat endpoint and return the assistant's reply.
    ///
    /// Builds an Ollama-compatible request from `messages` and optional `tools`, posts it to the provider's
    /// configured base URL, and converts the API response into a `Message` containing the assistant's
    /// content and any returned tool call metadata.
    ///
    /// # Arguments
    ///
    /// * `messages` - Slice of conversation `Message` values to send to the model (roles and any tool calls are translated to Ollama's expected shape).
    /// * `tools` - Slice of JSON function descriptors (each entry should be a function descriptor object); when non-empty these are attached to the request as callable tools.
    ///
    /// # Returns
    ///
    /// `Message` containing `role: Role::Assistant`, the assistant `content` extracted from the response, and `tool_calls` when the response includes tool invocation data.
    ///
    /// # Examples
    ///
    /// ```
    /// # use serde_json::json;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider = OllamaProvider::new("gpt-ollama");
    /// let messages = vec![Message { role: Role::User, content: "Hello".into(), tool_calls: None }];
    /// let tools = vec![json!({"name": "echo", "parameters": {}})];
    /// let reply = provider.complete(&messages, &tools).await?;
    /// assert_eq!(reply.role, Role::Assistant);
    /// # Ok(()) }
    /// ```
    async fn complete(&self, messages: &[Message], tools: &[serde_json::Value]) -> Result<Message> {
        let mut body = json!({
            "model": self.model,
            "stream": false,
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
                
                if let Some(calls) = &m.tool_calls {
                    let tool_calls: Vec<_> = calls.iter().map(|c| {
                        json!({
                            "function": {
                                "name": c.name,
                                "arguments": serde_json::from_str::<serde_json::Value>(&c.arguments).unwrap_or(json!({}))
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

    /// Exposes a streaming completion interface for the provider; currently not implemented.
    ///
    /// This method is intended to return a stream of incremental completion chunks from the LLM,
    /// but it currently always returns an error indicating streaming is unsupported.
    ///
    /// # Returns
    ///
    /// `Err(WeaveError::LlmError)` indicating that streaming is not implemented.
    ///
    /// # Examples
    ///
    /// ```
    /// use futures::executor::block_on;
    /// // assume OllamaProvider and related types are in scope
    /// let provider = OllamaProvider::new("model");
    /// let res = block_on(provider.stream(&[], &[]));
    /// assert!(res.is_err());
    /// ```
    async fn stream(
        &self,
        _messages: &[Message],
        _tools: &[serde_json::Value],
    ) -> Result<std::pin::Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        Err(WeaveError::LlmError("Stream not implemented".into()))
    }
}