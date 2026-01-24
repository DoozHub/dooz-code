//! Git Worktree Task Execution
//!
//! Provides parallel task execution using git worktrees for isolation.
//! Each task runs in its own worktree branch, preventing conflicts.

pub mod pool;
pub mod executor;
pub mod queue;

pub use pool::*;
pub use executor::*;
pub use queue::*;

use std::path::PathBuf;
use thiserror::Error;

/// Worktree execution error
#[derive(Debug, Error)]
pub enum WorktreeError {
    #[error("Git error: {0}")]
    Git(String),
    
    #[error("Worktree creation failed: {0}")]
    Creation(String),
    
    #[error("Worktree cleanup failed: {0}")]
    Cleanup(String),
    
    #[error("Task execution failed: {0}")]
    Execution(String),
    
    #[error("Queue error: {0}")]
    Queue(String),
}

/// Worktree configuration
#[derive(Debug, Clone)]
pub struct WorktreeConfig {
    /// Base repository path
    pub repo_path: PathBuf,
    
    /// Directory for worktrees
    pub worktree_dir: PathBuf,
    
    /// Maximum concurrent worktrees
    pub max_worktrees: usize,
    
    /// Branch prefix for task branches
    pub branch_prefix: String,
    
    /// Auto-cleanup after task completion
    pub auto_cleanup: bool,
}

impl Default for WorktreeConfig {
    fn default() -> Self {
        Self {
            repo_path: PathBuf::from("."),
            worktree_dir: PathBuf::from(".dooz-worktrees"),
            max_worktrees: 4,
            branch_prefix: "dooz-task".to_string(),
            auto_cleanup: true,
        }
    }
}

impl WorktreeConfig {
    pub fn new(repo_path: PathBuf) -> Self {
        Self {
            repo_path: repo_path.clone(),
            worktree_dir: repo_path.join(".dooz-worktrees"),
            ..Default::default()
        }
    }
    
    pub fn with_max_worktrees(mut self, max: usize) -> Self {
        self.max_worktrees = max;
        self
    }
}
