//! Rollback Generation
//!
//! Generates rollback plans for reverting changes.

use crate::types::{Step, StepType, RollbackPlan, RollbackStep, RollbackAction, ContextError};

/// Rollback plan generator
pub struct RollbackGenerator {
    /// Always generate full rollback
    full_rollback: bool,
}

impl RollbackGenerator {
    /// Create new generator
    pub fn new() -> Self {
        Self { full_rollback: true }
    }

    /// Generate rollback plan for steps
    pub fn generate(&self, steps: &[Step]) -> Result<RollbackPlan, ContextError> {
        let mut plan = RollbackPlan::new();
        
        // Generate rollback steps in reverse order
        for (idx, step) in steps.iter().enumerate().rev() {
            let rollback_step = self.generate_for_step(step, (steps.len() - idx) as u32)?;
            if let Some(rs) = rollback_step {
                plan.add_step(rs);
            }
        }

        // Check if full rollback is possible
        plan.full_rollback_possible = self.can_fully_rollback(steps);

        Ok(plan)
    }

    /// Generate rollback step for a single step
    fn generate_for_step(&self, step: &Step, order: u32) -> Result<Option<RollbackStep>, ContextError> {
        let rollback = match step.step_type {
            StepType::CreateFile | StepType::CreateTest => {
                // Rollback: delete the created file
                Some(RollbackStep::delete(order, &step.target))
            }
            StepType::ModifyFile | StepType::AddContent | StepType::ReplaceContent => {
                // Rollback: restore original content
                // Note: This requires storing original content hash
                Some(RollbackStep::restore(order, &step.target, "TODO-original-hash"))
            }
            StepType::DeleteFile => {
                // Rollback: recreate the deleted file
                Some(RollbackStep::recreate(order, &step.target, "TODO-original-hash"))
            }
            StepType::RemoveContent => {
                // Rollback: add back removed content
                Some(RollbackStep::restore(order, &step.target, "TODO-original-hash"))
            }
            StepType::UpdateConfig => {
                // Rollback: restore original config
                Some(RollbackStep::restore(order, &step.target, "TODO-original-hash"))
            }
            StepType::Verify => {
                // Verification steps don't need rollback
                None
            }
        };

        Ok(rollback)
    }

    /// Check if all steps can be fully rolled back
    fn can_fully_rollback(&self, steps: &[Step]) -> bool {
        // For now, assume all steps are rollbackable
        // In practice, some operations (like external API calls) cannot be rolled back
        steps.iter().all(|s| !matches!(s.step_type, StepType::Verify))
    }
}

impl Default for RollbackGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PackageId, StepId};

    #[test]
    fn generate_for_create() {
        let step = Step::new(
            StepId::new(&PackageId::new("TEST-001"), 1),
            "Create file",
            StepType::CreateFile,
        )
        .with_target("src/new.rs");

        let generator = RollbackGenerator::new();
        let plan = generator.generate(&[step]).unwrap();

        assert_eq!(plan.steps.len(), 1);
        assert!(matches!(plan.steps[0].action, RollbackAction::Delete));
    }

    #[test]
    fn generate_for_modify() {
        let step = Step::new(
            StepId::new(&PackageId::new("TEST-001"), 1),
            "Modify file",
            StepType::ModifyFile,
        )
        .with_target("src/existing.rs");

        let generator = RollbackGenerator::new();
        let plan = generator.generate(&[step]).unwrap();

        assert_eq!(plan.steps.len(), 1);
        assert!(matches!(plan.steps[0].action, RollbackAction::Restore));
    }

    #[test]
    fn skip_verify_steps() {
        let step = Step::new(
            StepId::new(&PackageId::new("TEST-001"), 1),
            "Verify",
            StepType::Verify,
        );

        let generator = RollbackGenerator::new();
        let plan = generator.generate(&[step]).unwrap();

        assert!(plan.steps.is_empty());
    }
}
