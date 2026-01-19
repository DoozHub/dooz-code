//! File Planning
//!
//! Generates file-level plans from implementation steps.

use crate::types::{Step, StepType, FilePlan, FileMod, ContextError};

/// File planner - generates file modification plans
pub struct FilePlanner {
    // No state needed
}

impl FilePlanner {
    /// Create new planner
    pub fn new() -> Self {
        Self {}
    }

    /// Plan files from steps
    pub fn plan_from_steps(&self, steps: &[Step]) -> Result<FilePlan, ContextError> {
        let mut plan = FilePlan::default();
        
        for step in steps {
            match step.step_type {
                StepType::CreateFile | StepType::CreateTest => {
                    if !step.target.is_empty() {
                        plan.add_create(FileMod::new(&step.target, &step.description));
                    }
                }
                StepType::ModifyFile | StepType::AddContent | StepType::ReplaceContent | StepType::RemoveContent => {
                    if !step.target.is_empty() {
                        plan.add_modify(FileMod::new(&step.target, &step.description));
                    }
                }
                StepType::DeleteFile => {
                    if !step.target.is_empty() {
                        plan.add_delete(FileMod::new(&step.target, &step.description));
                    }
                }
                StepType::UpdateConfig => {
                    if !step.target.is_empty() {
                        plan.add_modify(FileMod::new(&step.target, &step.description));
                    }
                }
                StepType::Verify => {
                    // Verification doesn't modify files
                }
            }
        }

        // Deduplicate
        plan.create = dedupe_file_mods(plan.create);
        plan.modify = dedupe_file_mods(plan.modify);
        plan.delete = dedupe_file_mods(plan.delete);

        Ok(plan)
    }

    /// Estimate lines to add/remove
    pub fn estimate_line_changes(&self, steps: &[Step]) -> LineEstimate {
        let mut added = 0;
        let mut removed = 0;

        for step in steps {
            match step.step_type {
                StepType::CreateFile | StepType::CreateTest => {
                    added += 50; // Average file size
                }
                StepType::AddContent => {
                    added += 20;
                }
                StepType::RemoveContent => {
                    removed += 10;
                }
                StepType::ModifyFile | StepType::ReplaceContent => {
                    added += 15;
                    removed += 10;
                }
                StepType::DeleteFile => {
                    removed += 50;
                }
                _ => {}
            }
        }

        LineEstimate { added, removed }
    }
}

impl Default for FilePlanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Estimate of line changes
#[derive(Debug, Clone, Copy)]
pub struct LineEstimate {
    /// Lines to add
    pub added: usize,
    /// Lines to remove
    pub removed: usize,
}

impl LineEstimate {
    /// Net change
    pub fn net(&self) -> isize {
        self.added as isize - self.removed as isize
    }

    /// Total churn
    pub fn churn(&self) -> usize {
        self.added + self.removed
    }
}

/// Deduplicate file modifications
fn dedupe_file_mods(mods: Vec<FileMod>) -> Vec<FileMod> {
    let mut seen = std::collections::HashSet::new();
    mods.into_iter()
        .filter(|m| seen.insert(m.path.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PackageId, StepId};

    #[test]
    fn plan_new_files() {
        let steps = vec![
            Step::new(
                StepId::new(&PackageId::new("TEST-001"), 1),
                "Create main file",
                StepType::CreateFile,
            ).with_target("src/main.rs"),
            Step::new(
                StepId::new(&PackageId::new("TEST-001"), 2),
                "Create lib file",
                StepType::CreateFile,
            ).with_target("src/lib.rs"),
        ];

        let planner = FilePlanner::new();
        let plan = planner.plan_from_steps(&steps).unwrap();

        assert_eq!(plan.create.len(), 2);
    }

    #[test]
    fn plan_modified_files() {
        let steps = vec![
            Step::new(
                StepId::new(&PackageId::new("TEST-001"), 1),
                "Modify existing",
                StepType::ModifyFile,
            ).with_target("src/lib.rs"),
        ];

        let planner = FilePlanner::new();
        let plan = planner.plan_from_steps(&steps).unwrap();

        assert_eq!(plan.modify.len(), 1);
    }

    #[test]
    fn deduplicate_files() {
        let steps = vec![
            Step::new(
                StepId::new(&PackageId::new("TEST-001"), 1),
                "First change",
                StepType::ModifyFile,
            ).with_target("src/lib.rs"),
            Step::new(
                StepId::new(&PackageId::new("TEST-001"), 2),
                "Second change",
                StepType::ModifyFile,
            ).with_target("src/lib.rs"),
        ];

        let planner = FilePlanner::new();
        let plan = planner.plan_from_steps(&steps).unwrap();

        // Should be deduped
        assert_eq!(plan.modify.len(), 1);
    }

    #[test]
    fn line_estimates() {
        let steps = vec![
            Step::new(
                StepId::new(&PackageId::new("TEST-001"), 1),
                "Create file",
                StepType::CreateFile,
            ),
            Step::new(
                StepId::new(&PackageId::new("TEST-001"), 2),
                "Add content",
                StepType::AddContent,
            ),
        ];

        let planner = FilePlanner::new();
        let estimate = planner.estimate_line_changes(&steps);

        assert!(estimate.added > 0);
        assert_eq!(estimate.removed, 0);
    }
}
