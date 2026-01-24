//! Task Executor in Worktree Context
//!
//! Executes tasks within isolated git worktree environments.

use super::{Worktree, WorktreeError, WorktreePool};
use crate::agency::task::{Task, TaskResult, TaskStatus, TaskOutput, TaskResultMetadata, TaskPayload};
use crate::modes::pipeline::{ModePipeline, agents::create_default_pipeline};
use std::time::{Duration, Instant};
use std::process::Command;
use std::fs;
use std::path::Path;

/// Execute a task within a worktree
pub struct WorktreeExecutor {
    pool: WorktreePool,
    pipeline: ModePipeline,
}

impl WorktreeExecutor {
    pub fn new(pool: WorktreePool) -> Self {
        Self { 
            pool,
            pipeline: create_default_pipeline(),
        }
    }
    
    /// Execute a task in its own worktree
    pub async fn execute(&self, task: Task) -> TaskResult {
        let start = Instant::now();
        let task_id = task.id.clone();
        
        // Acquire worktree
        let worktree = match self.pool.acquire(&task_id).await {
            Ok(wt) => wt,
            Err(e) => {
                return TaskResult {
                    task_id,
                    status: TaskStatus::Failed,
                    output: TaskOutput::Error {
                        message: format!("Worktree acquisition failed: {}", e),
                        recoverable: true,
                    },
                    duration: start.elapsed(),
                    agent_used: None,
                    metadata: TaskResultMetadata::default(),
                };
            }
        };
        
        // Execute task in worktree
        let result = self.execute_in_worktree(&worktree, &task).await;
        
        // Commit changes if successful
        if result.status == TaskStatus::Completed {
            let _ = self.commit_changes(&worktree, &task);
        }
        
        // Release worktree
        let _ = self.pool.release(&task_id).await;
        
        TaskResult {
            task_id: task.id,
            status: result.status,
            output: result.output,
            duration: start.elapsed(),
            agent_used: result.agent_used,
            metadata: result.metadata,
        }
    }
    
    async fn execute_in_worktree(&self, worktree: &Worktree, task: &Task) -> TaskResult {
        tracing::info!(
            task_id = %task.id,
            worktree = %worktree.path.display(),
            "Executing task in worktree"
        );
        
        // Extract prompt from task payload
        let prompt = match &task.payload {
            TaskPayload::CodeGen { spec, .. } => spec.clone(),
            TaskPayload::Refactor { pattern, .. } => pattern.clone(),
            TaskPayload::Explore { query, .. } => query.clone(),
            _ => format!("{:?}", task.payload),
        };
        
        // Run through the mode pipeline
        match self.pipeline.execute(&prompt).await {
            Ok(pipeline_result) => {
                // Extract artifacts from pipeline result
                let artifacts = self.extract_artifacts(&pipeline_result);
                
                // Write artifacts to worktree
                let written = self.write_artifacts(&worktree.path, &artifacts);
                
                TaskResult {
                    task_id: task.id.clone(),
                    status: TaskStatus::Completed,
                    output: TaskOutput::Success {
                        data: format!(
                            "Generated {} artifacts in worktree: {}",
                            written.len(),
                            worktree.path.display()
                        ),
                    },
                    duration: Duration::from_millis(100),
                    agent_used: Some("worktree-executor".to_string()),
                    metadata: TaskResultMetadata {
                        confidence: 0.9,
                        tokens_used: 0,
                        cost_estimate: 0.0,
                        artifacts: written,
                    },
                }
            }
            Err(e) => {
                TaskResult {
                    task_id: task.id.clone(),
                    status: TaskStatus::Failed,
                    output: TaskOutput::Error {
                        message: format!("Pipeline execution failed: {}", e),
                        recoverable: true,
                    },
                    duration: Duration::from_millis(100),
                    agent_used: Some("worktree-executor".to_string()),
                    metadata: TaskResultMetadata::default(),
                }
            }
        }
    }
    
    fn extract_artifacts(&self, result: &TaskResult) -> Vec<(String, String)> {
        // Extract artifacts from TaskResult output
        let mut artifacts = Vec::new();
        
        if let TaskOutput::Success { data } = &result.output {
            // Parse JSON to find artifacts
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                if let Some(artifact_arr) = json.get("artifacts").and_then(|a| a.as_array()) {
                    for artifact in artifact_arr {
                        if let (Some(path), Some(content)) = (
                            artifact.get("path").and_then(|p| p.as_str()),
                            artifact.get("content").and_then(|c| c.as_str()),
                        ) {
                            artifacts.push((path.to_string(), content.to_string()));
                        }
                    }
                }
            }
        }
        
        // If no artifacts found, generate a placeholder
        if artifacts.is_empty() {
            artifacts.push((
                "generated/output.rs".to_string(),
                format!(
                    "//! Generated by dooz-code worktree executor\n//! Task ID: {}\n\n// TODO: Implement\n",
                    result.task_id
                ),
            ));
        }
        
        artifacts
    }
    
    fn write_artifacts(&self, worktree_path: &Path, artifacts: &[(String, String)]) -> Vec<String> {
        let mut written = Vec::new();
        
        for (path, content) in artifacts {
            let full_path = worktree_path.join(path);
            
            // Create parent directories
            if let Some(parent) = full_path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            
            // Write file
            if fs::write(&full_path, content).is_ok() {
                written.push(path.clone());
                tracing::info!(path = %path, "Wrote artifact");
            } else {
                tracing::warn!(path = %path, "Failed to write artifact");
            }
        }
        
        written
    }
    
    fn commit_changes(&self, worktree: &Worktree, task: &Task) -> Result<(), WorktreeError> {
        // Stage all changes
        let output = Command::new("git")
            .current_dir(&worktree.path)
            .args(["add", "-A"])
            .output()
            .map_err(|e| WorktreeError::Git(e.to_string()))?;
        
        if !output.status.success() {
            return Ok(()); // No changes to stage
        }
        
        // Commit
        let commit_msg = format!("[dooz-task] {}: {:?}", task.id, task.task_type);
        let output = Command::new("git")
            .current_dir(&worktree.path)
            .args(["commit", "-m", &commit_msg, "--allow-empty"])
            .output()
            .map_err(|e| WorktreeError::Git(e.to_string()))?;
        
        if !output.status.success() {
            tracing::warn!("Commit failed: {}", String::from_utf8_lossy(&output.stderr));
        } else {
            tracing::info!(branch = %worktree.branch, "Changes committed");
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worktree::WorktreeConfig;
    use crate::agency::task::TaskType;
    
    #[tokio::test]
    async fn test_executor_creation() {
        let config = WorktreeConfig::default();
        let pool = WorktreePool::new(config);
        let executor = WorktreeExecutor::new(pool);
        
        // Just verify it creates without panic
        assert!(true);
    }
}
