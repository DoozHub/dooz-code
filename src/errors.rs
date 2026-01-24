//! Error handling module for dooz-code

use thiserror::Error;
use std::fmt;

/// Unified error type for dooz-code
#[derive(Debug, Error)]
pub enum DoozError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("LLM error: {0}")]
    Llm(String),
    
    #[error("Pipeline error: {0}")]
    Pipeline(String),
    
    #[error("Worktree error: {0}")]
    Worktree(String),
    
    #[error("Intake error: {0}")]
    Intake(String),
    
    #[error("Verification error: {0}")]
    Verification(String),
    
    #[error("Task error: {0}")]
    Task(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Rate limited")]
    RateLimited,
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl DoozError {
    /// Create a config error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }
    
    /// Create a pipeline error
    pub fn pipeline(msg: impl Into<String>) -> Self {
        Self::Pipeline(msg.into())
    }
    
    /// Create an LLM error
    pub fn llm(msg: impl Into<String>) -> Self {
        Self::Llm(msg.into())
    }
    
    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(self, 
            Self::RateLimited | 
            Self::Worktree(_) |
            Self::Io(_)
        )
    }
    
    /// Get HTTP status code equivalent
    pub fn status_code(&self) -> u16 {
        match self {
            Self::NotFound(_) => 404,
            Self::Unauthorized(_) => 401,
            Self::RateLimited => 429,
            Self::Config(_) | Self::Serialization(_) => 400,
            _ => 500,
        }
    }
}

/// Result type alias
pub type DoozResult<T> = Result<T, DoozError>;

/// Error context extension trait
pub trait ErrorContext<T> {
    fn context(self, ctx: &str) -> DoozResult<T>;
}

impl<T, E: std::error::Error> ErrorContext<T> for Result<T, E> {
    fn context(self, ctx: &str) -> DoozResult<T> {
        self.map_err(|e| DoozError::Internal(format!("{}: {}", ctx, e)))
    }
}

/// Log and return error
#[macro_export]
macro_rules! log_error {
    ($err:expr) => {{
        let e = $err;
        tracing::error!(error = %e, "Operation failed");
        e
    }};
}

/// Log and return error with context
#[macro_export]
macro_rules! log_error_ctx {
    ($err:expr, $ctx:expr) => {{
        let e = $err;
        tracing::error!(error = %e, context = $ctx, "Operation failed");
        e
    }};
}
