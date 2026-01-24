//! Execution Modes for dooz-code Pipeline
//!
//! Each task prompt is evaluated to determine which modes are needed,
//! then agents work sequentially through the required modes.

pub mod pipeline;

use serde::{Deserialize, Serialize};
use std::fmt;

pub use pipeline::*;

/// Execution mode in the dooz-code pipeline.
/// Each mode has a specialized agent that processes the task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Mode {
    /// PRD/prompt → Technical Spec conversion
    Intake,
    /// Spec → Task DAG generation
    Plan,
    /// Repository context extraction per task
    Analyze,
    /// Code generation/modification
    Execute,
    /// Self-validation and iteration
    Review,
    /// Compliance verification
    Verify,
    /// Sub-work dispatch to external agents
    Contract,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Intake => write!(f, "INTAKE"),
            Self::Plan => write!(f, "PLAN"),
            Self::Analyze => write!(f, "ANALYZE"),
            Self::Execute => write!(f, "EXECUTE"),
            Self::Review => write!(f, "REVIEW"),
            Self::Verify => write!(f, "VERIFY"),
            Self::Contract => write!(f, "CONTRACT"),
        }
    }
}

impl Mode {
    /// Returns all modes in standard pipeline order
    pub fn all_ordered() -> Vec<Mode> {
        vec![
            Mode::Intake,
            Mode::Plan,
            Mode::Analyze,
            Mode::Execute,
            Mode::Review,
            Mode::Verify,
        ]
    }

    /// Returns the next mode in the pipeline, if any
    pub fn next(&self) -> Option<Mode> {
        match self {
            Mode::Intake => Some(Mode::Plan),
            Mode::Plan => Some(Mode::Analyze),
            Mode::Analyze => Some(Mode::Execute),
            Mode::Execute => Some(Mode::Review),
            Mode::Review => Some(Mode::Verify),
            Mode::Verify => None,
            Mode::Contract => None, // Contract is a branch, not a sequence
        }
    }

    /// Check if this mode requires a previous mode to complete first
    pub fn requires(&self) -> Option<Mode> {
        match self {
            Mode::Intake => None,
            Mode::Plan => Some(Mode::Intake),
            Mode::Analyze => Some(Mode::Plan),
            Mode::Execute => Some(Mode::Analyze),
            Mode::Review => Some(Mode::Execute),
            Mode::Verify => Some(Mode::Review),
            Mode::Contract => Some(Mode::Plan), // Contracts branch from planning
        }
    }
}

/// Configuration for a specific mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    pub mode: Mode,
    pub enabled: bool,
    pub timeout_seconds: u32,
    pub max_retries: u32,
    pub agent_name: String,
}

impl Default for ModeConfig {
    fn default() -> Self {
        Self {
            mode: Mode::Execute,
            enabled: true,
            timeout_seconds: 300,
            max_retries: 3,
            agent_name: "default".to_string(),
        }
    }
}

/// Result of mode evaluation - determines which modes are needed for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeEvaluation {
    /// Ordered list of modes to execute
    pub modes: Vec<Mode>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Explanation of why these modes were selected
    pub rationale: String,
}

impl ModeEvaluation {
    /// Create evaluation for full pipeline (all modes)
    pub fn full_pipeline() -> Self {
        Self {
            modes: Mode::all_ordered(),
            confidence: 1.0,
            rationale: "Full pipeline: PRD to production code".to_string(),
        }
    }

    /// Create evaluation for execution only (skip intake/planning)
    pub fn execution_only() -> Self {
        Self {
            modes: vec![Mode::Analyze, Mode::Execute, Mode::Review, Mode::Verify],
            confidence: 1.0,
            rationale: "Direct execution: work package already defined".to_string(),
        }
    }

    /// Create evaluation for review/verify only
    pub fn verification_only() -> Self {
        Self {
            modes: vec![Mode::Review, Mode::Verify],
            confidence: 1.0,
            rationale: "Verification: review existing artifacts".to_string(),
        }
    }
}

/// Evaluator that determines required modes for a task
pub struct ModeEvaluator;

impl ModeEvaluator {
    pub fn new() -> Self {
        Self
    }

    /// Evaluate a task prompt to determine required modes
    pub fn evaluate(&self, prompt: &str) -> ModeEvaluation {
        // Simple heuristic evaluation
        let prompt_lower = prompt.to_lowercase();
        
        // Check for PRD indicators (needs full pipeline)
        if prompt_lower.contains("prd") 
            || prompt_lower.contains("requirement")
            || prompt_lower.contains("feature request")
            || prompt_lower.contains("user story")
        {
            return ModeEvaluation::full_pipeline();
        }

        // Check for structured work package (skip intake)
        if prompt_lower.contains("task-id")
            || prompt_lower.contains("work package")
            || prompt_lower.contains("acceptance criteria")
        {
            return ModeEvaluation::execution_only();
        }

        // Check for verification request
        if prompt_lower.contains("review")
            || prompt_lower.contains("verify")
            || prompt_lower.contains("check")
        {
            return ModeEvaluation::verification_only();
        }

        // Default: full pipeline
        ModeEvaluation::full_pipeline()
    }
}

impl Default for ModeEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

/// Input/output contract for mode handoffs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeHandoff {
    /// Source mode that produced this handoff
    pub from_mode: Mode,
    /// Target mode to receive this handoff
    pub to_mode: Mode,
    /// Payload data (JSON serialized)
    pub payload: serde_json::Value,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ModeHandoff {
    pub fn new(from: Mode, to: Mode, payload: serde_json::Value) -> Self {
        Self {
            from_mode: from,
            to_mode: to,
            payload,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_display() {
        assert_eq!(Mode::Intake.to_string(), "INTAKE");
        assert_eq!(Mode::Execute.to_string(), "EXECUTE");
    }

    #[test]
    fn mode_sequencing() {
        assert_eq!(Mode::Intake.next(), Some(Mode::Plan));
        assert_eq!(Mode::Plan.next(), Some(Mode::Analyze));
        assert_eq!(Mode::Verify.next(), None);
    }

    #[test]
    fn mode_requirements() {
        assert_eq!(Mode::Intake.requires(), None);
        assert_eq!(Mode::Execute.requires(), Some(Mode::Analyze));
    }

    #[test]
    fn evaluator_prd_detection() {
        let evaluator = ModeEvaluator::new();
        let eval = evaluator.evaluate("I have a PRD for user authentication");
        assert_eq!(eval.modes.len(), 6); // Full pipeline
        assert_eq!(eval.modes[0], Mode::Intake);
    }

    #[test]
    fn evaluator_work_package_detection() {
        let evaluator = ModeEvaluator::new();
        let eval = evaluator.evaluate("TASK-ID: 123. Implement login function");
        assert_eq!(eval.modes.len(), 4); // Skip intake/plan
        assert_eq!(eval.modes[0], Mode::Analyze);
    }
}
