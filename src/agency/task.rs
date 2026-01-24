//! Task Delegation Protocol
//!
//! Defines how tasks are routed to agents and results are aggregated.

use super::AgentCapability;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A task to be delegated to an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub task_type: TaskType,
    pub payload: TaskPayload,
    pub priority: TaskPriority,
    pub timeout: Duration,
    pub retry_count: u32,
    pub max_retries: u32,
    pub metadata: TaskMetadata,
}

impl Task {
    /// Create a new task
    pub fn new(task_type: TaskType, payload: TaskPayload) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            task_type,
            payload,
            priority: TaskPriority::Normal,
            timeout: Duration::from_secs(300),
            retry_count: 0,
            max_retries: 3,
            metadata: TaskMetadata::default(),
        }
    }

    /// Create a code generation task
    pub fn code_generation(path: String, spec: String) -> Self {
        Self::new(
            TaskType::CodeGeneration,
            TaskPayload::CodeGen { path, spec },
        )
    }

    /// Create a governance check task
    pub fn governance_check(scope: String, constraints: Vec<String>) -> Self {
        Self::new(
            TaskType::Governance,
            TaskPayload::Governance { scope, constraints },
        )
    }

    /// Create a test generation task
    pub fn test_generation(target: String, framework: String) -> Self {
        Self::new(
            TaskType::TestGeneration,
            TaskPayload::TestGen { target, framework },
        )
    }
}

/// Task type for routing
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskType {
    CodeGeneration,
    CodeReview,
    Testing,
    Documentation,
    Governance,
    Refactoring,
    Exploration,
    Analysis,
    TestGeneration,
    Database,
    ApiIntegration,
    Frontend,
    Backend,
    SecurityAudit,
}

/// Task priority levels
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TaskPriority {
    Critical,
    High,
    Normal,
    Low,
}

/// Task payload - the actual work to perform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPayload {
    CodeGen { path: String, spec: String },
    CodeReview { path: String, criteria: Vec<String> },
    TestGen { target: String, framework: String },
    Governance { scope: String, constraints: Vec<String> },
    Refactor { target: String, pattern: String },
    Explore { query: String, depth: u32 },
    Analyze { target: String, focus: String },
    Docs { target: String, style: String },
}

/// Result from task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub status: TaskStatus,
    pub output: TaskOutput,
    pub duration: Duration,
    pub agent_used: Option<String>,
    pub metadata: TaskResultMetadata,
}

/// Task execution status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    TimedOut,
    Cancelled,
}

/// Output from a completed task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskOutput {
    Success { data: String },
    Partial { data: String, warnings: Vec<String> },
    Error { message: String, recoverable: bool },
    None,
}

/// Metadata for results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskResultMetadata {
    pub confidence: f32,
    pub tokens_used: u32,
    pub cost_estimate: f64,
    pub artifacts: Vec<String>,
}

/// Metadata for tasks
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskMetadata {
    pub correlation_id: Option<String>,
    pub parent_task: Option<String>,
    pub user_id: Option<String>,
    pub project_id: Option<String>,
}

/// Convert Task to AgentCapability requirements
impl From<TaskType> for Vec<AgentCapability> {
    fn from(task_type: TaskType) -> Self {
        match task_type {
            TaskType::CodeGeneration => vec![AgentCapability::CodeGeneration],
            TaskType::CodeReview => vec![AgentCapability::CodeReview],
            TaskType::Testing | TaskType::TestGeneration => vec![AgentCapability::Testing],
            TaskType::Documentation => vec![AgentCapability::Documentation],
            TaskType::Governance => vec![AgentCapability::Governance],
            TaskType::Refactoring => vec![AgentCapability::Refactoring],
            TaskType::Exploration => vec![AgentCapability::Exploration],
            TaskType::Analysis => vec![AgentCapability::Architecture],
            TaskType::Database => vec![AgentCapability::Database],
            TaskType::ApiIntegration => vec![AgentCapability::ApiIntegration],
            TaskType::Frontend => vec![AgentCapability::Frontend],
            TaskType::Backend => vec![AgentCapability::Backend],
            TaskType::SecurityAudit => vec![AgentCapability::Security],
        }
    }
}
