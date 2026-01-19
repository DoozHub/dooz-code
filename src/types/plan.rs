//! Implementation Plan Types
//!
//! Defines the structure of implementation plans that guide execution.
//! Plans are generated from work packages and repository context.

use serde::{Deserialize, Serialize};
use super::identifiers::PackageId;
use super::step::Step;

/// Complete implementation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    /// Package this plan implements
    pub package_id: PackageId,
    
    /// Ordered list of implementation steps
    pub steps: Vec<Step>,
    
    /// Plan for file modifications
    pub file_plan: FilePlan,
    
    /// Plan for test generation
    pub test_plan: TestPlan,
    
    /// Rollback strategy
    pub rollback: RollbackPlan,
    
    /// Estimated complexity score
    pub complexity: ComplexityScore,
}

impl Plan {
    /// Create a new empty plan
    pub fn new(package_id: PackageId) -> Self {
        Self {
            package_id,
            steps: Vec::new(),
            file_plan: FilePlan::default(),
            test_plan: TestPlan::default(),
            rollback: RollbackPlan::default(),
            complexity: ComplexityScore::default(),
        }
    }

    /// Add a step to the plan
    pub fn add_step(&mut self, step: Step) {
        self.steps.push(step);
    }

    /// Get step count
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Check if plan is empty
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Get steps that can be executed in parallel
    pub fn parallel_groups(&self) -> Vec<Vec<&Step>> {
        // For now, return sequential execution
        // TODO: Implement dependency-based parallelization
        self.steps.iter().map(|s| vec![s]).collect()
    }
}

/// File modification plan
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FilePlan {
    /// Files to create
    pub create: Vec<FileMod>,
    
    /// Files to modify
    pub modify: Vec<FileMod>,
    
    /// Files to delete
    pub delete: Vec<FileMod>,
}

impl FilePlan {
    /// Total affected files
    pub fn total_files(&self) -> usize {
        self.create.len() + self.modify.len() + self.delete.len()
    }

    /// Add file to create
    pub fn add_create(&mut self, file: FileMod) {
        self.create.push(file);
    }

    /// Add file to modify
    pub fn add_modify(&mut self, file: FileMod) {
        self.modify.push(file);
    }

    /// Add file to delete
    pub fn add_delete(&mut self, file: FileMod) {
        self.delete.push(file);
    }
}

/// File modification entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMod {
    /// Relative path
    pub path: String,
    
    /// Reason for modification
    pub reason: String,
    
    /// Estimated lines affected
    pub lines_affected: Option<u32>,
}

impl FileMod {
    /// Create new file modification
    pub fn new(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            reason: reason.into(),
            lines_affected: None,
        }
    }

    /// With lines affected
    pub fn with_lines(mut self, lines: u32) -> Self {
        self.lines_affected = Some(lines);
        self
    }
}

/// Test generation plan
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TestPlan {
    /// Unit tests to generate
    pub unit_tests: Vec<TestSpec>,
    
    /// Integration tests to generate
    pub integration_tests: Vec<TestSpec>,
    
    /// Target coverage percentage
    pub target_coverage: Option<u8>,
}

impl TestPlan {
    /// Create new test plan
    pub fn new() -> Self {
        Self::default()
    }

    /// Total test count
    pub fn total_tests(&self) -> usize {
        self.unit_tests.len() + self.integration_tests.len()
    }

    /// Add unit test
    pub fn add_unit_test(&mut self, test: TestSpec) {
        self.unit_tests.push(test);
    }

    /// Add integration test
    pub fn add_integration_test(&mut self, test: TestSpec) {
        self.integration_tests.push(test);
    }

    /// Add test file (creates a TestSpec from path)
    pub fn add_test_file(&mut self, path: &str) {
        if !path.is_empty() {
            self.unit_tests.push(TestSpec::new(
                path,
                format!("Tests for {}", path),
                path,
            ));
        }
    }

    /// Add criterion-based test
    pub fn add_criterion_test(&mut self, criterion_id: &str, description: &str) {
        self.unit_tests.push(TestSpec::new(
            format!("test_{}", criterion_id),
            description,
            criterion_id,
        ));
    }
}

/// Test specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSpec {
    /// Test name
    pub name: String,
    
    /// What the test verifies
    pub verifies: String,
    
    /// Target file/function
    pub target: String,
}

impl TestSpec {
    /// Create new test spec
    pub fn new(name: impl Into<String>, verifies: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            verifies: verifies.into(),
            target: target.into(),
        }
    }
}

/// Rollback plan for reverting changes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RollbackPlan {
    /// Steps to reverse changes
    pub steps: Vec<RollbackStep>,
    
    /// Whether full rollback is possible
    pub full_rollback_possible: bool,
    
    /// Estimated rollback time
    pub estimated_duration_seconds: Option<u32>,
}

impl RollbackPlan {
    /// Create new rollback plan
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            full_rollback_possible: true,
            estimated_duration_seconds: None,
        }
    }

    /// Add rollback step
    pub fn add_step(&mut self, step: RollbackStep) {
        self.steps.push(step);
    }

    /// Mark as partial rollback only
    pub fn mark_partial(&mut self) {
        self.full_rollback_possible = false;
    }
}

/// Single rollback step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStep {
    /// Step order (reverse execution order)
    pub order: u32,
    
    /// Action to take
    pub action: RollbackAction,
    
    /// File affected
    pub file: String,
    
    /// Original content hash (for verification)
    pub original_hash: Option<String>,
}

impl RollbackStep {
    /// Create delete rollback (delete created file)
    pub fn delete(order: u32, file: impl Into<String>) -> Self {
        Self {
            order,
            action: RollbackAction::Delete,
            file: file.into(),
            original_hash: None,
        }
    }

    /// Create restore rollback (restore modified file)
    pub fn restore(order: u32, file: impl Into<String>, original_hash: impl Into<String>) -> Self {
        Self {
            order,
            action: RollbackAction::Restore,
            file: file.into(),
            original_hash: Some(original_hash.into()),
        }
    }

    /// Create recreate rollback (recreate deleted file)
    pub fn recreate(order: u32, file: impl Into<String>, original_hash: impl Into<String>) -> Self {
        Self {
            order,
            action: RollbackAction::Recreate,
            file: file.into(),
            original_hash: Some(original_hash.into()),
        }
    }
}

/// Rollback action type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RollbackAction {
    /// Delete a created file
    Delete,
    /// Restore a modified file to original
    Restore,
    /// Recreate a deleted file
    Recreate,
}

/// Complexity score for plan
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplexityScore {
    /// Overall score (1-10)
    pub overall: u8,
    
    /// File complexity component
    pub file_complexity: u8,
    
    /// Logic complexity component
    pub logic_complexity: u8,
    
    /// Integration complexity component
    pub integration_complexity: u8,
}

impl ComplexityScore {
    /// Create with overall score
    pub fn new(overall: u8) -> Self {
        Self {
            overall: overall.clamp(1, 10),
            file_complexity: overall.clamp(1, 10),
            logic_complexity: overall.clamp(1, 10),
            integration_complexity: 1,
        }
    }

    /// Set component scores
    pub fn with_components(mut self, file: u8, logic: u8, integration: u8) -> Self {
        self.file_complexity = file.clamp(1, 10);
        self.logic_complexity = logic.clamp(1, 10);
        self.integration_complexity = integration.clamp(1, 10);
        self.overall = ((file as u16 + logic as u16 + integration as u16) / 3) as u8;
        self
    }

    /// Get overall value
    pub fn value(&self) -> u8 {
        self.overall
    }

    /// Check if high complexity
    pub fn is_high(&self) -> bool {
        self.overall >= 7
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_creation() {
        let plan = Plan::new(PackageId::new("TEST-001"));
        assert!(plan.is_empty());
        assert_eq!(plan.step_count(), 0);
    }

    #[test]
    fn file_plan_tracking() {
        let mut file_plan = FilePlan::default();
        file_plan.add_create(FileMod::new("new.rs", "Create new file"));
        file_plan.add_modify(FileMod::new("existing.rs", "Add function"));

        assert_eq!(file_plan.total_files(), 2);
    }

    #[test]
    fn complexity_score_range() {
        let score = ComplexityScore::new(15); // Should be clamped
        assert!(score.overall <= 10);

        let high_score = ComplexityScore::new(8);
        assert!(high_score.is_high());
    }
}
