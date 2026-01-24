//! # Dooz-Code
//!
//! An autonomous coder that belongs inside a company.
//!
//! ## Identity
//!
//! dooz-code is:
//! - An **autonomous execution engine** for approved work packages
//! - A **context-aware coder** that respects existing patterns
//! - A **scope-bound implementer** that cannot expand features
//! - A **deterministic artifact generator** for production code
//!
//! dooz-code is NOT:
//! - A code completion tool
//! - A chat-based assistant
//! - An IDE plugin
//! - A productivity enhancer
//! - A copilot
//! - A decision maker
//!
//! ## Core Components
//!
//! - [`types`] - Core data structures (WorkPackage, Context, Plan, Result)
//! - [`analyzer`] - Repository context extraction
//! - [`planner`] - Implementation step decomposition
//! - [`executor`] - Code generation and application
//! - [`reviewer`] - Self-validation against acceptance criteria
//! - [`signals`] - Status emission and audit trail
//!
//! ## Constitutional Laws (Immutable)
//!
//! 1. **Scope is immutable during execution** - Cannot expand, reduce, or reinterpret
//! 2. **Execution is deterministic** - Same input → same output, no randomness
//! 3. **Governance must precede execution** - Approval required before any action
//! 4. **Patterns are respected** - Existing conventions take precedence
//! 5. **Context is required** - Cannot execute without full repository analysis
//! 6. **Audit is complete** - Every decision is logged

pub mod types;
pub mod analyzer;
pub mod planner;
pub mod executor;
pub mod reviewer;
pub mod signals;
pub mod config;
pub mod snapshot;
pub mod orchestrator;
pub mod modes;
pub mod worktree;
pub mod intake;
pub mod verifier;
pub mod errors;
pub mod logging;
pub mod contracts;
pub mod agency;

pub use types::*;
pub use analyzer::RepoAnalyzer;
pub use planner::ImplementationPlanner;
pub use executor::CodeExecutor;
pub use reviewer::ArtifactReviewer;
pub use signals::StatusEmitter;
pub use config::*;
pub use snapshot::*;
pub use orchestrator::*;
pub use modes::*;
pub use worktree::*;
pub use agency::*;

// Re-export agency submodules
pub use agency::task::{Task, TaskResult, TaskStatus, TaskType, TaskPayload, TaskOutput};
pub use agency::registry::{AgentRegistry, RegisteredAgent, AgentCapability, AgentStatus};
pub use agency::execute::{ParallelExecutor, ExecutionPlan, ExecutionStrategy, BatchProgress};

// Re-export worktree submodules
pub use worktree::{WorktreePool, WorktreeExecutor, TaskQueue, TaskLoop, WorktreeConfig};

/// Dooz-Code version
pub const VERSION: &str = "0.4.0-worktree";

/// Primary execution entry point
/// 
/// Takes an approved work package and repository context,
/// returns execution result with generated artifacts.
pub fn execute(
    package: &WorkPackage,
    context: &RepoContext,
) -> Result<ExecutionResult, ExecutionError> {
    // Validate approval
    if !package.is_approved() {
        return Err(ExecutionError::NotApproved {
            package_id: package.id.clone(),
        });
    }

    // Analyze repository
    let analyzer = RepoAnalyzer::new();
    let analyzed_context = analyzer.analyze(context)?;

    // Plan implementation
    let planner = ImplementationPlanner::new();
    let plan = planner.plan(package, &analyzed_context)?;

    // Execute plan
    let executor = CodeExecutor::new();
    let artifacts = executor.execute(&plan, &analyzed_context)?;

    // Review artifacts
    let reviewer = ArtifactReviewer::new();
    let validation = reviewer.validate(&artifacts, &package.criteria)?;

    // Handle validation result
    match validation.status {
        ValidationStatus::Pass => {
            Ok(ExecutionResult::success(artifacts, plan))
        }
        ValidationStatus::Fail { issues } => {
            // Attempt iteration
            let max_iterations = 3;
            let mut current_artifacts = artifacts;
            
            for iteration in 1..=max_iterations {
                let corrected = executor.correct(&current_artifacts, &issues, &analyzed_context)?;
                let revalidation = reviewer.validate(&corrected, &package.criteria)?;
                
                match revalidation.status {
                    ValidationStatus::Pass => {
                        return Ok(ExecutionResult::success_after_iteration(
                            corrected,
                            plan.clone(),
                            iteration,
                        ));
                    }
                    ValidationStatus::Fail { issues: new_issues } => {
                        current_artifacts = corrected;
                        if iteration == max_iterations {
                            return Err(ExecutionError::ValidationFailed {
                                issues: new_issues,
                                iterations: iteration,
                            });
                        }
                    }
                }
            }
            
            unreachable!()
        }
    }
}

/// Check if dooz-code is operational
pub fn is_alive() -> bool {
    true // All components are stateless
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_exists() {
        assert!(!VERSION.is_empty());
        assert!(VERSION.starts_with("0.3"));
    }

    #[test]
    fn is_alive_returns_true() {
        assert!(is_alive());
    }
}
