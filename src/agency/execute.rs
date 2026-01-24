//! Parallel Execution Engine
//!
//! Executes multiple tasks concurrently across agents.

use super::{Task, TaskResult, TaskStatus, AgencyError};
use crate::agency::task::{TaskOutput, TaskPayload};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::time;

/// Parallel execution plan
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub tasks: Vec<Task>,
    pub strategy: ExecutionStrategy,
    pub concurrency_limit: usize,
}

impl Default for ExecutionPlan {
    fn default() -> Self {
        Self {
            tasks: Vec::new(),
            strategy: ExecutionStrategy::Balanced,
            concurrency_limit: 10,
        }
    }
}

/// Strategy for parallel execution
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ExecutionStrategy {
    Parallel,
    Sequential,
    Grouped,
    Priority,
    Balanced,
}

/// Parallel executor
pub struct ParallelExecutor {
    task_tx: mpsc::Sender<Task>,
    concurrency_limit: usize,
}

impl ParallelExecutor {
    /// Create a new executor
    pub fn new(task_tx: mpsc::Sender<Task>, concurrency_limit: usize) -> Self {
        Self {
            task_tx,
            concurrency_limit,
        }
    }

    /// Execute all tasks and wait for results
    pub async fn execute_all(&self, tasks: Vec<Task>) -> Vec<TaskResult> {
        let mut results = Vec::new();
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(self.concurrency_limit));

        let handles: Vec<_> = tasks
            .into_iter()
            .map(|task| {
                let tx = self.task_tx.clone();
                let permit = semaphore.clone().acquire_owned();
                async move {
                    let _permit = permit.await;
                    Self::execute_single(tx, task).await
                }
            })
            .collect();

        for handle in handles {
            results.push(handle.await);
        }

        results
    }

    /// Execute a single task with timeout
    async fn execute_single(task_tx: mpsc::Sender<Task>, task: Task) -> TaskResult {
        let (tx, rx) = oneshot::channel();
        let task_id = task.id.clone();

        if let Err(e) = task_tx.send(task).await {
            return TaskResult {
                task_id,
                status: TaskStatus::Failed,
                output: TaskOutput::Error {
                    message: format!("Failed to send task: {}", e),
                    recoverable: false,
                },
                duration: Duration::ZERO,
                agent_used: None,
                metadata: Default::default(),
            };
        }

        match time::timeout(Duration::from_secs(300), rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => TaskResult {
                task_id,
                status: TaskStatus::Failed,
                output: TaskOutput::Error {
                    message: "Result channel closed".to_string(),
                    recoverable: true,
                },
                duration: Duration::ZERO,
                agent_used: None,
                metadata: Default::default(),
            },
            Err(_) => TaskResult {
                task_id,
                status: TaskStatus::TimedOut,
                output: TaskOutput::Error {
                    message: "Task execution timed out".to_string(),
                    recoverable: true,
                },
                duration: Duration::from_secs(300),
                agent_used: None,
                metadata: Default::default(),
            },
        }
    }

    /// Split a large task into subtasks
    pub fn split_task(task: &Task, _chunk_size: usize) -> Vec<Task> {
        match &task.payload {
            TaskPayload::CodeGen { path: _, spec: _ } => {
                vec![task.clone()]
            }
            _ => vec![task.clone()],
        }
    }

    /// Aggregate multiple results
    pub fn aggregate_results(results: Vec<TaskResult>) -> TaskResult {
        let successful: Vec<_> = results.iter().filter(|r| r.status == TaskStatus::Completed).collect();
        let failed: Vec<_> = results.iter().filter(|r| r.status == TaskStatus::Failed).collect();

        let overall_status = if failed.is_empty() {
            TaskStatus::Completed
        } else if !successful.is_empty() {
            TaskStatus::Failed
        } else {
            TaskStatus::Failed
        };

        let total_duration: Duration = results.iter().map(|r| r.duration).sum();

        let output = if !successful.is_empty() && failed.is_empty() {
            TaskOutput::Success { data: format!("All {} tasks completed", successful.len()) }
        } else if !successful.is_empty() {
            TaskOutput::Partial {
                data: format!("{}/{} tasks completed", successful.len(), results.len()),
                warnings: failed.iter().map(|r| format!("Task failed: {}", r.task_id)).collect(),
            }
        } else {
            TaskOutput::Error {
                message: "All tasks failed".to_string(),
                recoverable: true,
            }
        };

        TaskResult {
            task_id: format!("aggregate-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("agg")),
            status: overall_status,
            output,
            duration: total_duration,
            agent_used: Some("parallel-executor".to_string()),
            metadata: Default::default(),
        }
    }
}

/// Progress of batch execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProgress {
    pub total: usize,
    pub completed: usize,
    pub current_task: String,
    pub status: TaskStatus,
}

impl BatchProgress {
    /// Calculate percentage complete
    pub fn percent(&self) -> f32 {
        if self.total == 0 { 0.0 } else { (self.completed as f32 / self.total as f32) * 100.0 }
    }
}
