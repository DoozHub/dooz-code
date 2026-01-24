//! Dooz-AI Agency Orchestration
//!
//! Multi-agent coordination system for the Dooz-AI Agency.
//! Enables agent registration, task delegation, and parallel execution.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub mod registry;
pub mod task;
pub mod message;
pub mod execute;

pub use registry::{AgentRegistry, RegisteredAgent, AgentCapability, AgentStatus};
pub use task::{Task, TaskResult, TaskStatus, TaskType};
pub use message::{AgentMessage, MessageType, EventBus, AgencyEvent};
pub use execute::{ParallelExecutor, ExecutionPlan, BatchProgress};

/// Agency orchestrator that manages all dooz-* agents
#[derive(Debug, Clone)]
pub struct AgencyOrchestrator {
    registry: AgentRegistry,
    event_bus: EventBus,
}

impl AgencyOrchestrator {
    /// Create a new agency orchestrator
    pub fn new() -> Self {
        Self {
            registry: AgentRegistry::new(),
            event_bus: EventBus::new(),
        }
    }

    /// Initialize with default dooz-* agents
    pub async fn initialize(&mut self) -> Result<(), AgencyError> {
        self.discover_agents().await
    }

    /// Auto-discover available dooz-* agents
    pub async fn discover_agents(&mut self) -> Result<(), AgencyError> {
        let agents = vec![
            RegisteredAgent {
                name: "dooz-code".to_string(),
                path: std::env::current_exe().unwrap_or_else(|_| PathBuf::from("dooz-code")),
                capabilities: vec![
                    AgentCapability::CodeGeneration,
                    AgentCapability::FileOperations,
                    AgentCapability::Testing,
                ],
                status: AgentStatus::Available,
                last_heartbeat: Instant::now(),
                metadata: AgentMetadata {
                    version: "0.3.0".to_string(),
                    description: "Primary code execution agent".to_string(),
                },
            },
            RegisteredAgent {
                name: "dooz-veto".to_string(),
                path: PathBuf::from("../dooz-veto/target/release/dooz-veto"),
                capabilities: vec![
                    AgentCapability::Governance,
                    AgentCapability::RiskAssessment,
                    AgentCapability::ScopeValidation,
                ],
                status: AgentStatus::Unknown,
                last_heartbeat: Instant::now(),
                metadata: AgentMetadata {
                    version: "0.2.0".to_string(),
                    description: "Governance and entropy gate agent".to_string(),
                },
            },
        ];

        for agent in agents {
            let health = Self::check_agent_health(&agent.path).await;
            let mut agent = agent;
            agent.status = health;
            self.registry.register(agent);
        }

        Ok(())
    }

    /// Check if an agent binary is available and healthy
    async fn check_agent_health(path: &PathBuf) -> AgentStatus {
        if !path.exists() {
            return AgentStatus::NotFound;
        }

        let output = Command::new(path)
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .output();

        match output {
            Ok(o) if o.status.success() => AgentStatus::Available,
            _ => AgentStatus::Unavailable,
        }
    }

    /// Register a new agent
    pub fn register_agent(&mut self, agent: RegisteredAgent) {
        self.registry.register(agent);
    }

    /// Get agents with specific capability
    pub fn get_agents_by_capability(&self, capability: AgentCapability) -> Vec<RegisteredAgent> {
        self.registry.find_by_capability(capability)
    }

    /// Get registry status
    pub fn status(&self) -> AgencyStatus {
        AgencyStatus {
            total_agents: self.registry.len(),
            available_agents: self.registry.available_count(),
        }
    }

    /// Get event bus for publishing events
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    /// Get the agent registry
    pub fn registry(&self) -> &AgentRegistry {
        &self.registry
    }
}

/// Agency-wide status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgencyStatus {
    pub total_agents: usize,
    pub available_agents: usize,
}

/// Agent metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub version: String,
    pub description: String,
}

/// Agency errors
#[derive(Debug, thiserror::Error)]
pub enum AgencyError {
    #[error("Task execution failed")]
    TaskFailed,
    #[error("Agent not found")]
    AgentNotFound,
    #[error("Capability not supported")]
    CapabilityNotSupported,
}
