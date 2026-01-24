//! Worktree Pool Management
//!
//! Manages a pool of git worktrees for parallel task execution.

use super::{WorktreeConfig, WorktreeError};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};

/// A single worktree instance
#[derive(Debug, Clone)]
pub struct Worktree {
    pub id: String,
    pub path: PathBuf,
    pub branch: String,
    pub in_use: bool,
}

impl Worktree {
    /// Create a new worktree
    pub fn create(repo_path: &PathBuf, worktree_dir: &PathBuf, task_id: &str, branch_prefix: &str) -> Result<Self, WorktreeError> {
        let branch = format!("{}/{}", branch_prefix, task_id);
        let worktree_path = worktree_dir.join(task_id);
        
        // Ensure worktree directory exists
        std::fs::create_dir_all(worktree_dir)
            .map_err(|e| WorktreeError::Creation(e.to_string()))?;
        
        // Create new branch and worktree
        let output = Command::new("git")
            .current_dir(repo_path)
            .args(["worktree", "add", "-b", &branch, worktree_path.to_str().unwrap()])
            .output()
            .map_err(|e| WorktreeError::Git(e.to_string()))?;
        
        if !output.status.success() {
            // Try without -b if branch exists
            let output2 = Command::new("git")
                .current_dir(repo_path)
                .args(["worktree", "add", worktree_path.to_str().unwrap(), &branch])
                .output()
                .map_err(|e| WorktreeError::Git(e.to_string()))?;
            
            if !output2.status.success() {
                return Err(WorktreeError::Creation(
                    String::from_utf8_lossy(&output2.stderr).to_string()
                ));
            }
        }
        
        Ok(Self {
            id: task_id.to_string(),
            path: worktree_path,
            branch,
            in_use: true,
        })
    }
    
    /// Remove the worktree
    pub fn remove(&self, repo_path: &PathBuf) -> Result<(), WorktreeError> {
        // Remove worktree
        let output = Command::new("git")
            .current_dir(repo_path)
            .args(["worktree", "remove", "--force", self.path.to_str().unwrap()])
            .output()
            .map_err(|e| WorktreeError::Git(e.to_string()))?;
        
        if !output.status.success() {
            // Force remove directory if git fails
            if self.path.exists() {
                std::fs::remove_dir_all(&self.path)
                    .map_err(|e| WorktreeError::Cleanup(e.to_string()))?;
            }
        }
        
        // Optionally delete the branch
        let _ = Command::new("git")
            .current_dir(repo_path)
            .args(["branch", "-D", &self.branch])
            .output();
        
        Ok(())
    }
}

/// Pool of worktrees for parallel execution
pub struct WorktreePool {
    config: WorktreeConfig,
    worktrees: Arc<Mutex<HashMap<String, Worktree>>>,
    semaphore: Arc<Semaphore>,
}

impl WorktreePool {
    /// Create a new worktree pool
    pub fn new(config: WorktreeConfig) -> Self {
        let max = config.max_worktrees;
        Self {
            config,
            worktrees: Arc::new(Mutex::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(max)),
        }
    }
    
    /// Acquire a worktree for a task
    pub async fn acquire(&self, task_id: &str) -> Result<Worktree, WorktreeError> {
        // Wait for available slot
        let _permit = self.semaphore.acquire().await
            .map_err(|e| WorktreeError::Queue(e.to_string()))?;
        
        // Create worktree
        let worktree = Worktree::create(
            &self.config.repo_path,
            &self.config.worktree_dir,
            task_id,
            &self.config.branch_prefix,
        )?;
        
        // Track it
        let mut worktrees = self.worktrees.lock().await;
        worktrees.insert(task_id.to_string(), worktree.clone());
        
        Ok(worktree)
    }
    
    /// Release a worktree after task completion
    pub async fn release(&self, task_id: &str) -> Result<(), WorktreeError> {
        let mut worktrees = self.worktrees.lock().await;
        
        if let Some(worktree) = worktrees.remove(task_id) {
            if self.config.auto_cleanup {
                worktree.remove(&self.config.repo_path)?;
            }
        }
        
        Ok(())
    }
    
    /// Get active worktree count
    pub async fn active_count(&self) -> usize {
        self.worktrees.lock().await.len()
    }
    
    /// Cleanup all worktrees
    pub async fn cleanup_all(&self) -> Result<(), WorktreeError> {
        let mut worktrees = self.worktrees.lock().await;
        
        for (_, worktree) in worktrees.drain() {
            worktree.remove(&self.config.repo_path)?;
        }
        
        Ok(())
    }
}
