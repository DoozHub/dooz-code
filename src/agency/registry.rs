//! Agent Registry
//!
//! Manages registration and discovery of dooz-* agents.

use super::AgentMetadata;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Capability tags for agent matching
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentCapability {
    CodeGeneration,
    CodeReview,
    Testing,
    Documentation,
    FileOperations,
    Governance,
    RiskAssessment,
    ScopeValidation,
    Architecture,
    Debugging,
    Refactoring,
    Exploration,
    Database,
    ApiIntegration,
    Frontend,
    Backend,
    DevOps,
    Security,
}

/// Agent registration entry (not serializable due to Instant)
#[derive(Debug, Clone)]
pub struct RegisteredAgent {
    pub name: String,
    pub path: std::path::PathBuf,
    pub capabilities: Vec<AgentCapability>,
    pub status: AgentStatus,
    pub last_heartbeat: Instant,
    pub metadata: AgentMetadata,
}

/// Agent availability status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AgentStatus {
    Available,
    Busy,
    NotFound,
    Unavailable,
    Unknown,
}

/// In-memory agent registry
#[derive(Debug, Clone, Default)]
pub struct AgentRegistry {
    agents: HashMap<String, RegisteredAgent>,
    capability_index: HashMap<AgentCapability, Vec<String>>,
}

impl AgentRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
            capability_index: HashMap::new(),
        }
    }

    /// Register an agent
    pub fn register(&mut self, agent: RegisteredAgent) {
        self.agents.insert(agent.name.clone(), agent.clone());

        for capability in &agent.capabilities {
            self.capability_index
                .entry(capability.clone())
                .or_insert_with(Vec::new)
                .push(agent.name.clone());
        }
    }

    /// Unregister an agent
    pub fn unregister(&mut self, name: &str) -> Option<RegisteredAgent> {
        if let Some(agent) = self.agents.remove(name) {
            for capability in &agent.capabilities {
                if let Some(agents) = self.capability_index.get_mut(capability) {
                    agents.retain(|n| n != name);
                }
            }
            Some(agent)
        } else {
            None
        }
    }

    /// Get an agent by name
    pub fn get(&self, name: &str) -> Option<&RegisteredAgent> {
        self.agents.get(name)
    }

    /// Find agents with specific capability
    pub fn find_by_capability(&self, capability: AgentCapability) -> Vec<RegisteredAgent> {
        self.capability_index
            .get(&capability)
            .map(|names| {
                names.iter()
                    .filter_map(|n| self.agents.get(n))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find agents matching multiple capabilities
    pub fn find_by_capabilities(&self, capabilities: Vec<AgentCapability>) -> Vec<RegisteredAgent> {
        let mut candidates: Vec<(String, usize)> = Vec::new();

        for capability in &capabilities {
            if let Some(agents) = self.capability_index.get(capability) {
                for name in agents {
                    if let Some((_, count)) = candidates.iter_mut().find(|(n, _)| n == name) {
                        *count += 1;
                    } else {
                        candidates.push((name.clone(), 1));
                    }
                }
            }
        }

        candidates.sort_by(|(_, a), (_, b)| b.cmp(a));
        candidates
            .into_iter()
            .filter_map(|(name, _)| self.agents.get(&name).cloned())
            .collect()
    }

    /// List all registered agents
    pub fn list(&self) -> Vec<RegisteredAgent> {
        self.agents.values().cloned().collect()
    }

    /// Count total agents
    pub fn len(&self) -> usize {
        self.agents.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }

    /// Count available agents
    pub fn available_count(&self) -> usize {
        self.agents.values().filter(|a| a.status == AgentStatus::Available).count()
    }

    /// Update agent status
    pub fn update_status(&mut self, name: &str, status: AgentStatus) {
        if let Some(agent) = self.agents.get_mut(name) {
            agent.status = status;
            agent.last_heartbeat = Instant::now();
        }
    }
}
