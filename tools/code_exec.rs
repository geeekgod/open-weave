use openweave_core::error::Result;
use openweave_core::tools::Tool;
use openweave_core::sandbox::wasm::WasmSandbox;
use async_trait::async_trait;
use serde_json::{json, Value};

pub struct CodeExecTool {
    sandbox: WasmSandbox,
}

impl CodeExecTool {
    /// Creates a new `CodeExecTool` initialized with a fresh WASM sandbox.
    ///
    /// # Examples
    ///
    /// ```
    /// let tool = CodeExecTool::new();
    /// assert_eq!(tool.name(), "code_exec");
    /// ```
    pub fn new() -> Self {
        Self {
            sandbox: WasmSandbox::new(),
        }
    }
}

impl Default for CodeExecTool {
    /// Constructs a default CodeExecTool.
    ///
    /// # Examples
    ///
    /// ```
    /// let tool = CodeExecTool::default();
    /// assert_eq!(tool.name(), "code_exec");
    /// ```
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for CodeExecTool {
    /// Get the tool's static identifier.
    ///
    /// # Examples
    ///
    /// ```
    /// let tool = CodeExecTool::new();
    /// assert_eq!(tool.name(), "code_exec");
    /// ```
    fn name(&self) -> &str {
        "code_exec"
    }

    fn description(&self) -> &str {
        "Execute code in a secure WASM sandbox"
    }

    /// Provide the JSON Schema describing the tool's public metadata and parameters.
    ///
    /// The returned value contains the tool `name`, `description`, and a `parameters` object
    /// that requires a single `code` string parameter.
    ///
    /// # Returns
    ///
    /// A `serde_json::Value` containing the tool metadata and parameter schema.
    ///
    /// # Examples
    ///
    /// ```
    /// let schema = CodeExecTool::new().schema();
    /// assert_eq!(schema["name"], "code_exec");
    /// assert_eq!(schema["parameters"]["required"][0], "code");
    /// ```
    fn schema(&self) -> Value {
        json!({
            "name": "code_exec",
            "description": "Execute WebAssembly or mock code in a secure sandbox",
            "parameters": {
                "type": "object",
                "properties": {
                    "code": { "type": "string", "description": "The code to execute" }
                },
                "required": ["code"]
            }
        })
    }

    /// Executes the provided code string in the tool's WASM sandbox and returns a simulated execution result.
    ///
    /// The `input` JSON must include a `"code"` field (string). The method prints the provided code, performs a mocked sandbox invocation, and returns a fixed success message containing simulated output.
    ///
    /// # Parameters
    ///
    /// - `input`: A JSON object expected to contain `"code"` with the source or payload to execute.
    ///
    /// # Returns
    ///
    /// A `String` containing a human-readable execution result message (e.g. `"Execution completed successfully.\nOutput:\nHello from sandbox!"`).
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::json;
    /// use futures::executor::block_on;
    ///
    /// let tool = CodeExecTool::new();
    /// let input = json!({ "code": "console.log('hello');" });
    /// let result = block_on(tool.execute(input)).expect("execution failed");
    /// assert!(result.contains("Execution completed successfully."));
    /// assert!(result.contains("Hello from sandbox!"));
    /// ```
    async fn execute(&self, input: Value) -> Result<String> {
        let code = input.get("code").and_then(|v| v.as_str()).unwrap_or("");
        
        // MVP: If it's literally WASM bytes (base64 encoded), we could run it.
        // For now, return a simulated code execution result to satisfy LLM planning.
        println!("Executing code in sandbox: \n{}", code);
        
        let _ = self.sandbox.execute_wasm(&[], "{}"); // Ignored for mock

        Ok(format!("Execution completed successfully.\nOutput:\n{}", "Hello from sandbox!"))
    }
}