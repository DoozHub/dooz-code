//! Contract Dispatcher

use super::{
    Contract, ContractWork, ContractTerms, ContractResult, ContractStatus,
    ContractError, ExternalAgent, ExternalAgentRegistry, ContractArtifact,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Dispatches contracts to external agents
pub struct ContractDispatcher {
    registry: Arc<ExternalAgentRegistry>,
    http_client: reqwest::Client,
}

impl ContractDispatcher {
    pub fn new(registry: Arc<ExternalAgentRegistry>) -> Self {
        Self {
            registry,
            http_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(300))
                .build()
                .unwrap(),
        }
    }
    
    /// Create and dispatch a contract to the best available agent
    pub async fn dispatch(&self, work: ContractWork, terms: ContractTerms) -> Result<ContractResult, ContractError> {
        // Find best agent
        let agent = self.registry
            .find_best(&work.task_type)
            .await
            .ok_or_else(|| ContractError::AgentNotFound(work.task_type.clone()))?;
        
        // Create contract
        let mut contract = Contract::new(&agent.id, work.clone(), terms);
        
        // Negotiate
        contract.status = ContractStatus::Negotiating;
        let accepted = self.negotiate(&agent, &contract).await?;
        
        if !accepted {
            return Err(ContractError::Rejected("Agent declined contract".to_string()));
        }
        
        contract.status = ContractStatus::Accepted;
        
        // Execute
        contract.status = ContractStatus::InProgress;
        let result = self.execute(&agent, &contract).await?;
        
        // Update agent rating based on result
        self.registry.update_rating(&agent.id, result.quality_score).await;
        
        Ok(result)
    }
    
    /// Dispatch to a specific agent
    pub async fn dispatch_to(&self, agent_id: &str, work: ContractWork, terms: ContractTerms) -> Result<ContractResult, ContractError> {
        let agent = self.registry
            .get(agent_id)
            .await
            .ok_or_else(|| ContractError::AgentNotFound(agent_id.to_string()))?;
        
        let contract = Contract::new(agent_id, work, terms);
        self.execute(&agent, &contract).await
    }
    
    async fn negotiate(&self, agent: &ExternalAgent, contract: &Contract) -> Result<bool, ContractError> {
        // In a real implementation, this would call the agent's negotiate endpoint
        // For now, always accept
        tracing::info!(agent_id = %agent.id, "Negotiating contract");
        Ok(true)
    }
    
    async fn execute(&self, agent: &ExternalAgent, contract: &Contract) -> Result<ContractResult, ContractError> {
        let start = Instant::now();
        
        tracing::info!(
            contract_id = %contract.id,
            agent_id = %agent.id,
            task_type = %contract.work.task_type,
            "Executing contract"
        );
        
        // In a real implementation, this would call the agent's execute endpoint
        // For now, return mock result
        let result = ContractResult {
            contract_id: contract.id.clone(),
            success: true,
            outputs: serde_json::json!({
                "message": format!("Executed by agent: {}", agent.name),
                "task_type": contract.work.task_type,
            }),
            quality_score: 0.85,
            cost: 0.0,
            duration_secs: start.elapsed().as_secs(),
            artifacts: vec![
                ContractArtifact {
                    path: "contract_output.txt".to_string(),
                    content: "Contract execution complete".to_string(),
                    artifact_type: "text".to_string(),
                }
            ],
        };
        
        Ok(result)
    }
}

/// Helper to create common contract work types
impl ContractWork {
    pub fn code_review(path: &str) -> Self {
        Self::new("code_review", &format!("Review code at: {}", path))
    }
    
    pub fn security_audit() -> Self {
        Self::new("security_audit", "Perform security audit")
    }
    
    pub fn performance_optimization() -> Self {
        Self::new("performance", "Optimize performance")
    }
    
    pub fn documentation(target: &str) -> Self {
        Self::new("documentation", &format!("Generate documentation for: {}", target))
    }
}
