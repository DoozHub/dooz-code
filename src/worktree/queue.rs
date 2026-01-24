//! Task Queue with Processing Loop
//!
//! Local task queue that feeds tasks to the worktree executor.

use super::{WorktreeExecutor, WorktreeError};
use crate::agency::task::{Task, TaskResult, TaskStatus};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc, broadcast};

/// Task queue for local processing
pub struct TaskQueue {
    queue: Arc<Mutex<VecDeque<Task>>>,
    result_tx: broadcast::Sender<TaskResult>,
}

impl TaskQueue {
    /// Create a new task queue
    pub fn new() -> (Self, broadcast::Receiver<TaskResult>) {
        let (result_tx, result_rx) = broadcast::channel(100);
        (
            Self {
                queue: Arc::new(Mutex::new(VecDeque::new())),
                result_tx,
            },
            result_rx,
        )
    }
    
    /// Add a task to the queue
    pub async fn push(&self, task: Task) {
        let mut queue = self.queue.lock().await;
        queue.push_back(task);
    }
    
    /// Get next task from queue
    pub async fn pop(&self) -> Option<Task> {
        let mut queue = self.queue.lock().await;
        queue.pop_front()
    }
    
    /// Check if queue is empty
    pub async fn is_empty(&self) -> bool {
        self.queue.lock().await.is_empty()
    }
    
    /// Get queue length
    pub async fn len(&self) -> usize {
        self.queue.lock().await.len()
    }
    
    /// Send result notification
    pub fn notify_result(&self, result: TaskResult) {
        let _ = self.result_tx.send(result);
    }
    
    /// Get result receiver
    pub fn subscribe(&self) -> broadcast::Receiver<TaskResult> {
        self.result_tx.subscribe()
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new().0
    }
}

/// Task processing loop
pub struct TaskLoop {
    queue: Arc<TaskQueue>,
    executor: Arc<WorktreeExecutor>,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl TaskLoop {
    /// Create a new task loop
    pub fn new(queue: Arc<TaskQueue>, executor: Arc<WorktreeExecutor>) -> Self {
        Self {
            queue,
            executor,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
    
    /// Start the processing loop
    pub async fn start(&self) {
        self.running.store(true, std::sync::atomic::Ordering::SeqCst);
        tracing::info!("Task loop started");
        
        while self.running.load(std::sync::atomic::Ordering::SeqCst) {
            match self.queue.pop().await {
                Some(task) => {
                    let task_id = task.id.clone();
                    tracing::info!(task_id = %task_id, "Processing task");
                    
                    // Execute task
                    let result = self.executor.execute(task).await;
                    
                    // Notify result
                    self.queue.notify_result(result);
                }
                None => {
                    // No tasks, wait before checking again
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                }
            }
        }
        
        tracing::info!("Task loop stopped");
    }
    
    /// Stop the processing loop
    pub fn stop(&self) {
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
    }
    
    /// Check if loop is running
    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// Create and run a task loop with given configuration
pub async fn run_task_loop(
    queue: Arc<TaskQueue>,
    executor: Arc<WorktreeExecutor>,
) -> TaskLoop {
    let task_loop = TaskLoop::new(queue, executor);
    task_loop.start().await;
    task_loop
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agency::task::{TaskType, TaskPayload};
    
    #[tokio::test]
    async fn test_queue_operations() {
        let (queue, _rx) = TaskQueue::new();
        
        assert!(queue.is_empty().await);
        
        let task = Task::new(
            TaskType::CodeGeneration,
            TaskPayload::CodeGen {
                path: "test.rs".to_string(),
                spec: "Add function".to_string(),
            },
        );
        
        queue.push(task.clone()).await;
        assert_eq!(queue.len().await, 1);
        
        let popped = queue.pop().await;
        assert!(popped.is_some());
        assert!(queue.is_empty().await);
    }
}
