use thiserror::Error;

#[derive(Error, Debug)]
pub enum WeaveError {
    #[error("LLM Provider error: {0}")]
    LlmError(String),
    
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("Tool execution timed out")]
    ToolTimeout,
    
    #[error("Maximum iterations reached")]
    MaxIterationsReached,
    
    #[error("Memory error: {0}")]
    MemoryError(String),
    
    #[error("Sandbox error: {0}")]
    SandboxError(String),
    
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, WeaveError>;