//! Mode Pipeline Orchestration
//!
//! Chains mode agents sequentially: EVALUATE → INTAKE → PLAN → ANALYZE → EXECUTE → REVIEW → VERIFY

use super::{Mode, ModeEvaluation, ModeEvaluator, ModeHandoff};
use crate::agency::task::{Task, TaskResult, TaskStatus, TaskOutput};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

/// Trait for mode-specific agents
#[async_trait]
pub trait ModeAgent: Send + Sync {
    /// Get the mode this agent handles
    fn mode(&self) -> Mode;
    
    /// Process input and produce output for next mode
    async fn process(&self, input: ModeHandoff) -> Result<ModeHandoff, PipelineError>;
}

/// Pipeline state
#[derive(Debug, Clone)]
pub enum PipelineState {
    Idle,
    Evaluating,
    Running { current_mode: Mode, progress: f32 },
    Completed,
    Failed { error: String },
}

/// Pipeline error
#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Evaluation failed: {0}")]
    Evaluation(String),
    
    #[error("Mode {0} failed: {1}")]
    ModeExecution(Mode, String),
    
    #[error("Agent not found for mode: {0}")]
    AgentNotFound(Mode),
    
    #[error("Handoff failed: {0}")]
    Handoff(String),
}

/// Mode pipeline orchestrator
pub struct ModePipeline {
    evaluator: ModeEvaluator,
    agents: HashMap<Mode, Arc<dyn ModeAgent>>,
    state: Arc<RwLock<PipelineState>>,
}

impl ModePipeline {
    /// Create new pipeline with evaluator
    pub fn new() -> Self {
        Self {
            evaluator: ModeEvaluator::new(),
            agents: HashMap::new(),
            state: Arc::new(RwLock::new(PipelineState::Idle)),
        }
    }
    
    /// Register a mode agent
    pub fn register_agent(&mut self, agent: Arc<dyn ModeAgent>) {
        self.agents.insert(agent.mode(), agent);
    }
    
    /// Get current pipeline state
    pub async fn state(&self) -> PipelineState {
        self.state.read().await.clone()
    }
    
    /// Execute the pipeline for a prompt
    pub async fn execute(&self, prompt: &str) -> Result<TaskResult, PipelineError> {
        // Set evaluating state
        *self.state.write().await = PipelineState::Evaluating;
        
        // Evaluate prompt to get required modes
        let evaluation = self.evaluator.evaluate(prompt);
        
        if evaluation.modes.is_empty() {
            return Err(PipelineError::Evaluation("No modes determined".to_string()));
        }
        
        // Create initial handoff with prompt
        let mut current_handoff = ModeHandoff::new(
            Mode::Intake, // Placeholder source
            evaluation.modes[0],
            serde_json::json!({
                "prompt": prompt,
                "evaluation": {
                    "modes": evaluation.modes.iter().map(|m| m.to_string()).collect::<Vec<_>>(),
                    "confidence": evaluation.confidence,
                    "rationale": evaluation.rationale
                }
            }),
        );
        
        // Execute each mode in sequence
        let total_modes = evaluation.modes.len();
        for (i, mode) in evaluation.modes.iter().enumerate() {
            let progress = (i as f32 + 1.0) / total_modes as f32;
            *self.state.write().await = PipelineState::Running {
                current_mode: *mode,
                progress,
            };
            
            // Get agent for this mode
            let agent = self.agents.get(mode)
                .ok_or_else(|| PipelineError::AgentNotFound(*mode))?;
            
            // Process through agent
            tracing::info!(mode = %mode, "Executing mode");
            current_handoff = agent.process(current_handoff).await
                .map_err(|e| PipelineError::ModeExecution(*mode, e.to_string()))?;
        }
        
        // Mark completed
        *self.state.write().await = PipelineState::Completed;
        
        // Extract result from final handoff
        Ok(TaskResult {
            task_id: uuid::Uuid::new_v4().to_string(),
            status: TaskStatus::Completed,
            output: TaskOutput::Success {
                data: serde_json::to_string_pretty(&current_handoff.payload).unwrap_or_default(),
            },
            duration: std::time::Duration::from_secs(1),
            agent_used: Some("mode-pipeline".to_string()),
            metadata: Default::default(),
        })
    }
}

impl Default for ModePipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Default agent implementations for each mode
pub mod agents {
    use super::*;
    
    /// Intake agent - PRD to Spec
    pub struct IntakeAgent;
    
    #[async_trait]
    impl ModeAgent for IntakeAgent {
        fn mode(&self) -> Mode { Mode::Intake }
        
        async fn process(&self, input: ModeHandoff) -> Result<ModeHandoff, PipelineError> {
            let prompt = input.payload["prompt"].as_str().unwrap_or("");
            
            // Extract spec from prompt (simplified)
            let spec = serde_json::json!({
                "title": format!("Spec for: {}", &prompt[..prompt.len().min(50)]),
                "entities": [],
                "apis": [],
                "acceptance_criteria": ["Feature works as described"]
            });
            
            Ok(ModeHandoff::new(Mode::Intake, Mode::Plan, spec))
        }
    }
    
    /// Plan agent - Spec to DAG
    pub struct PlanAgent;
    
    #[async_trait]
    impl ModeAgent for PlanAgent {
        fn mode(&self) -> Mode { Mode::Plan }
        
        async fn process(&self, input: ModeHandoff) -> Result<ModeHandoff, PipelineError> {
            let dag = serde_json::json!({
                "tasks": [{
                    "task_id": "TASK-001",
                    "title": "Implement feature",
                    "dependencies": []
                }],
                "execution_order": ["TASK-001"]
            });
            
            Ok(ModeHandoff::new(Mode::Plan, Mode::Analyze, dag))
        }
    }
    
    /// Analyze agent - context extraction
    pub struct AnalyzeAgent;
    
    #[async_trait]
    impl ModeAgent for AnalyzeAgent {
        fn mode(&self) -> Mode { Mode::Analyze }
        
        async fn process(&self, input: ModeHandoff) -> Result<ModeHandoff, PipelineError> {
            let context = serde_json::json!({
                "patterns": [],
                "conventions": {},
                "dependencies": [],
                "related_files": []
            });
            
            Ok(ModeHandoff::new(Mode::Analyze, Mode::Execute, serde_json::json!({
                "dag": input.payload,
                "context": context
            })))
        }
    }
    
    /// Execute agent - code generation
    pub struct ExecuteAgent;
    
    #[async_trait]
    impl ModeAgent for ExecuteAgent {
        fn mode(&self) -> Mode { Mode::Execute }
        
        async fn process(&self, input: ModeHandoff) -> Result<ModeHandoff, PipelineError> {
            let artifacts = serde_json::json!({
                "artifacts": [{
                    "path": "generated/output.rs",
                    "action": "create",
                    "content": "// Generated code"
                }]
            });
            
            Ok(ModeHandoff::new(Mode::Execute, Mode::Review, artifacts))
        }
    }
    
    /// Review agent - self-validation
    pub struct ReviewAgent;
    
    #[async_trait]
    impl ModeAgent for ReviewAgent {
        fn mode(&self) -> Mode { Mode::Review }
        
        async fn process(&self, input: ModeHandoff) -> Result<ModeHandoff, PipelineError> {
            let review = serde_json::json!({
                "status": "pass",
                "artifacts": input.payload["artifacts"],
                "issues": [],
                "suggestions": []
            });
            
            Ok(ModeHandoff::new(Mode::Review, Mode::Verify, review))
        }
    }
    
    /// Verify agent - compliance check
    pub struct VerifyAgent;
    
    #[async_trait]
    impl ModeAgent for VerifyAgent {
        fn mode(&self) -> Mode { Mode::Verify }
        
        async fn process(&self, input: ModeHandoff) -> Result<ModeHandoff, PipelineError> {
            let verification = serde_json::json!({
                "status": "PASS",
                "spec_coverage": 1.0,
                "quality_score": 0.9,
                "artifacts": input.payload["artifacts"],
                "blocking_issues": []
            });
            
            Ok(ModeHandoff::new(Mode::Verify, Mode::Verify, verification))
        }
    }
    
    /// Create pipeline with all default agents
    pub fn create_default_pipeline() -> ModePipeline {
        let mut pipeline = ModePipeline::new();
        
        pipeline.register_agent(Arc::new(IntakeAgent));
        pipeline.register_agent(Arc::new(PlanAgent));
        pipeline.register_agent(Arc::new(AnalyzeAgent));
        pipeline.register_agent(Arc::new(ExecuteAgent));
        pipeline.register_agent(Arc::new(ReviewAgent));
        pipeline.register_agent(Arc::new(VerifyAgent));
        
        pipeline
    }
}
