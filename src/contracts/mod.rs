//! Contracts Module - External Agent Dispatch
//!
//! Enables sub-work dispatch to specialized external agents.

mod types;
mod registry;
mod dispatcher;

pub use types::*;
pub use registry::*;
pub use dispatcher::*;

/// Contract negotiation error
#[derive(Debug, thiserror::Error)]
pub enum ContractError {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    
    #[error("Contract rejected: {0}")]
    Rejected(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Timeout")]
    Timeout,
    
    #[error("Network error: {0}")]
    Network(String),
}
