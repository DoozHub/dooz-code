//! External Agent Registry

use super::{Contract, ContractError, AgentCapability};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// External agent information
#[derive(Debug, Clone)]
pub struct ExternalAgent {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub capabilities: Vec<AgentCapability>,
    pub available: bool,
    pub rating: f32,
}

impl ExternalAgent {
    pub fn new(id: &str, name: &str, endpoint: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            endpoint: endpoint.to_string(),
            capabilities: vec![],
            available: true,
            rating: 0.0,
        }
    }
    
    pub fn supports(&self, task_type: &str) -> bool {
        self.capabilities.iter().any(|c| 
            c.supported_tasks.iter().any(|t| t == task_type)
        )
    }
}

/// Registry of external agents
pub struct ExternalAgentRegistry {
    agents: Arc<RwLock<HashMap<String, ExternalAgent>>>,
}

impl ExternalAgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register an external agent
    pub async fn register(&self, agent: ExternalAgent) {
        let mut agents = self.agents.write().await;
        agents.insert(agent.id.clone(), agent);
    }
    
    /// Unregister an agent
    pub async fn unregister(&self, agent_id: &str) {
        let mut agents = self.agents.write().await;
        agents.remove(agent_id);
    }
    
    /// Get agent by ID
    pub async fn get(&self, agent_id: &str) -> Option<ExternalAgent> {
        let agents = self.agents.read().await;
        agents.get(agent_id).cloned()
    }
    
    /// Find agents that support a task type
    pub async fn find_by_capability(&self, task_type: &str) -> Vec<ExternalAgent> {
        let agents = self.agents.read().await;
        agents.values()
            .filter(|a| a.available && a.supports(task_type))
            .cloned()
            .collect()
    }
    
    /// Get best agent for a task (highest rating)
    pub async fn find_best(&self, task_type: &str) -> Option<ExternalAgent> {
        let candidates = self.find_by_capability(task_type).await;
        candidates.into_iter()
            .max_by(|a, b| a.rating.partial_cmp(&b.rating).unwrap())
    }
    
    /// List all registered agents
    pub async fn list_all(&self) -> Vec<ExternalAgent> {
        let agents = self.agents.read().await;
        agents.values().cloned().collect()
    }
    
    /// Update agent availability
    pub async fn set_availability(&self, agent_id: &str, available: bool) {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(agent_id) {
            agent.available = available;
        }
    }
    
    /// Update agent rating
    pub async fn update_rating(&self, agent_id: &str, new_rating: f32) {
        let mut agents = self.agents.write().await;
        if let Some(agent) = agents.get_mut(agent_id) {
            // Exponential moving average
            agent.rating = agent.rating * 0.9 + new_rating * 0.1;
        }
    }
}

impl Default for ExternalAgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
