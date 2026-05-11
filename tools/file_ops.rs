use openweave_core::error::{Result, WeaveError};
use openweave_core::tools::Tool;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::fs;

pub struct FileOpsTool;

impl FileOpsTool {
    /// Create a new FileOpsTool for reading and writing files on the local disk.
    ///
    /// # Examples
    ///
    /// ```
    /// let tool = FileOpsTool::new();
    /// assert_eq!(tool.name(), "file_ops");
    /// ```
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileOpsTool {
    /// Creates a FileOpsTool initialized with the type's standard default configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// let tool = FileOpsTool::default();
    /// assert_eq!(tool.name(), "file_ops");
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for FileOpsTool {
    /// Tool identifier used to reference this tool.
    ///
    /// # Examples
    ///
    /// ```
    /// let tool = FileOpsTool::new();
    /// assert_eq!(tool.name(), "file_ops");
    /// ```
    fn name(&self) -> &str {
        "file_ops"
    }

    /// A short human-readable description of the tool's purpose.
    ///
    /// # Examples
    ///
    /// ```
    /// let tool = FileOpsTool::new();
    /// assert_eq!(tool.description(), "Read or write files to the local disk");
    /// ```
    fn description(&self) -> &str {
        "Read or write files to the local disk"
    }

    /// JSON schema describing the `file_ops` tool's expected input.
    ///
    /// The returned `serde_json::Value` is an object with `name`, `description`, and `parameters` describing:
    /// - `action`: string enum `"read"` or `"write"` (required)
    /// - `path`: file path string (required)
    /// - `content`: string to write when `action` is `"write"` (optional)
    ///
    /// # Examples
    ///
    /// ```
    /// let schema = FileOpsTool::new().schema();
    /// let params = &schema["parameters"]["properties"];
    /// assert_eq!(schema["name"], "file_ops");
    /// assert_eq!(params["action"]["enum"][0], "read");
    /// assert_eq!(params["action"]["enum"][1], "write");
    /// assert!(schema["parameters"]["required"].as_array().unwrap().iter().any(|v| v == "path"));
    /// ```
    fn schema(&self) -> Value {
        json!({
            "name": "file_ops",
            "description": "Read or write files to the local disk",
            "parameters": {
                "type": "object",
                "properties": {
                    "action": { "type": "string", "enum": ["read", "write"], "description": "The action to perform: read or write" },
                    "path": { "type": "string", "description": "The path to the file" },
                    "content": { "type": "string", "description": "The content to write to the file (only for write action)" }
                },
                "required": ["action", "path"]
            }
        })
    }

    /// Execute a file operation described by a JSON `input`.
    ///
    /// The `input` must be a JSON object with:
    /// - `"action"`: either `"read"` or `"write"`.
    /// - `"path"`: file system path to read from or write to.
    /// - `"content"`: (for `"write"`) the string to write to `path`.
    ///
    /// On `"read"`, returns the file contents. On `"write"`, writes `content` to `path` and
    /// returns a success message of the form `"Successfully wrote to <path>"`.
    ///
    /// I/O errors from reading or writing are propagated. If `action` is anything other than
    /// `"read"` or `"write"`, returns `WeaveError::ToolNotFound` containing the invalid action.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::json;
    ///
    /// // Read example (assuming "foo.txt" exists)
    /// let input = json!({ "action": "read", "path": "foo.txt" });
    /// // In an async runtime call: let result = file_ops.execute(input).await.unwrap();
    ///
    /// // Write example
    /// let input = json!({ "action": "write", "path": "bar.txt", "content": "hello" });
    /// // In an async runtime call: let message = file_ops.execute(input).await.unwrap();
    /// // assert_eq!(message, "Successfully wrote to bar.txt");
    /// ```
    async fn execute(&self, input: Value) -> Result<String> {
        let action = input.get("action").and_then(|v| v.as_str()).unwrap_or("");
        let path = input.get("path").and_then(|v| v.as_str()).unwrap_or("");

        match action {
            "read" => {
                let content = fs::read_to_string(path)?;
                Ok(content)
            }
            "write" => {
                let content = input.get("content").and_then(|v| v.as_str()).unwrap_or("");
                fs::write(path, content)?;
                Ok(format!("Successfully wrote to {}", path))
            }
            _ => Err(WeaveError::ToolNotFound(format!("Invalid action for file_ops: {}", action))),
        }
    }
}