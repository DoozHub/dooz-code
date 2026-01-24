//! Contract Types

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A contract for external work dispatch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub id: ContractId,
    pub agent_id: String,
    pub work: ContractWork,
    pub terms: ContractTerms,
    pub status: ContractStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub type ContractId = String;

impl Contract {
    pub fn new(agent_id: &str, work: ContractWork, terms: ContractTerms) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            work,
            terms,
            status: ContractStatus::Pending,
            created_at: Utc::now(),
            completed_at: None,
        }
    }
}

/// Work to be performed under contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractWork {
    pub task_type: String,
    pub description: String,
    pub inputs: serde_json::Value,
    pub expected_outputs: Vec<String>,
}

impl ContractWork {
    pub fn new(task_type: &str, description: &str) -> Self {
        Self {
            task_type: task_type.to_string(),
            description: description.to_string(),
            inputs: serde_json::json!({}),
            expected_outputs: vec![],
        }
    }
    
    pub fn with_inputs(mut self, inputs: serde_json::Value) -> Self {
        self.inputs = inputs;
        self
    }
}

/// Contract terms and constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTerms {
    pub max_cost: Option<f64>,
    pub max_duration_secs: u64,
    pub retry_allowed: bool,
    pub quality_threshold: f32,
}

impl Default for ContractTerms {
    fn default() -> Self {
        Self {
            max_cost: None,
            max_duration_secs: 300,
            retry_allowed: true,
            quality_threshold: 0.8,
        }
    }
}

/// Contract execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContractStatus {
    Pending,
    Negotiating,
    Accepted,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Result of contract execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractResult {
    pub contract_id: ContractId,
    pub success: bool,
    pub outputs: serde_json::Value,
    pub quality_score: f32,
    pub cost: f64,
    pub duration_secs: u64,
    pub artifacts: Vec<ContractArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractArtifact {
    pub path: String,
    pub content: String,
    pub artifact_type: String,
}

/// External agent capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    pub name: String,
    pub description: String,
    pub supported_tasks: Vec<String>,
    pub avg_quality: f32,
    pub avg_cost: f64,
}
