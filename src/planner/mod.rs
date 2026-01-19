//! Implementation Planner
//!
//! Decomposes work packages into executable implementation steps.
//! Planning is deterministic and respects scope boundaries.

mod decompose;
mod order;
mod rollback;
mod file_plan;

pub use decompose::*;
pub use order::*;
pub use rollback::*;
pub use file_plan::*;

use crate::types::{WorkPackage, Plan, FilePlan, TestPlan, ComplexityScore, ContextError};
use crate::analyzer::AnalyzedContext;

/// Implementation planner component
pub struct ImplementationPlanner {
    /// Planner configuration
    config: PlannerConfig,
}

impl ImplementationPlanner {
    /// Create new planner
    pub fn new() -> Self {
        Self {
            config: PlannerConfig::default(),
        }
    }

    /// Create planner with config
    pub fn with_config(config: PlannerConfig) -> Self {
        Self { config }
    }

    /// Plan implementation for work package
    pub fn plan(
        &self,
        package: &WorkPackage,
        context: &AnalyzedContext,
    ) -> Result<Plan, ContextError> {
        // Validate scope against constraints
        self.validate_scope(package, context)?;

        // Decompose into steps
        let decomposer_config = DecomposerConfig {
            generate_tests: self.config.generate_tests,
            generate_docs: self.config.generate_docs,
            follow_patterns: self.config.follow_patterns,
            max_steps_per_item: self.config.max_steps_per_item,
        };
        let mut decomposer = StepDecomposer::with_config(decomposer_config);
        let steps = decomposer.decompose(package, context)?;

        // Check step limit
        if steps.len() > self.config.max_steps {
            return Err(ContextError::ParseError(format!(
                "Plan exceeds max steps: {} > {}",
                steps.len(),
                self.config.max_steps
            )));
        }

        // Order steps by dependencies
        let orderer = StepOrderer::new();
        let ordered_steps = orderer.order(steps)?;

        // Generate rollback plan
        let rollback = if self.config.always_rollback {
            let rollback_gen = RollbackGenerator::new();
            rollback_gen.generate(&ordered_steps)?
        } else {
            crate::types::RollbackPlan::new()
        };

        // Build plan
        let mut plan = Plan::new(package.id.clone());
        for step in ordered_steps {
            plan.add_step(step);
        }
        plan.rollback = rollback;

        // Generate file plan
        plan.file_plan = self.generate_file_plan(&plan, context)?;

        // Generate test plan
        plan.test_plan = self.generate_test_plan(&plan, package)?;

        // Calculate complexity
        plan.complexity = self.calculate_complexity(&plan, context);

        Ok(plan)
    }

    /// Validate scope against package constraints
    fn validate_scope(&self, package: &WorkPackage, context: &AnalyzedContext) -> Result<(), ContextError> {
        // Check if package is approved
        if !package.is_approved() {
            return Err(ContextError::ValidationError(
                "Work package must be approved before planning".into()
            ));
        }

        // Check max new files constraint
        if let Some(max_files) = package.scope.max_new_files {
            let scope_items = package.scope.includes.len();
            if scope_items > max_files as usize {
                return Err(ContextError::ValidationError(format!(
                    "Scope items ({}) exceed max new files constraint ({})",
                    scope_items, max_files
                )));
            }
        }

        // Check allowed file patterns
        if !package.scope.allowed_files.is_empty() {
            // Would verify all target files match allowed patterns
            // For now, pass through
        }

        Ok(())
    }

    /// Generate file plan from steps
    fn generate_file_plan(&self, plan: &Plan, _context: &AnalyzedContext) -> Result<FilePlan, ContextError> {
        let planner = FilePlanner::new();
        planner.plan_from_steps(&plan.steps)
    }

    /// Generate test plan from steps and criteria
    fn generate_test_plan(&self, plan: &Plan, package: &WorkPackage) -> Result<TestPlan, ContextError> {
        let mut test_plan = TestPlan::new();

        // Map steps to test requirements
        for step in &plan.steps {
            if step.step_type == crate::types::StepType::CreateTest {
                test_plan.add_test_file(&step.target);
            }
        }

        // Add criteria-based tests
        for criterion in &package.criteria {
            test_plan.add_criterion_test(&criterion.id, &criterion.description);
        }

        Ok(test_plan)
    }

    /// Calculate plan complexity
    fn calculate_complexity(&self, plan: &Plan, context: &AnalyzedContext) -> ComplexityScore {
        let step_count = plan.steps.len();
        let file_count = plan.file_plan.create.len() + plan.file_plan.modify.len();
        let has_tests = plan.steps.iter().any(|s| s.step_type == crate::types::StepType::CreateTest);
        let has_refactoring = plan.steps.iter().any(|s| s.step_type == crate::types::StepType::ModifyFile);
        let existing_file_count = context.file_count();

        // Base score from step count
        let base: u8 = match step_count {
            0..=3 => 1,
            4..=7 => 2,
            8..=12 => 3,
            13..=20 => 4,
            _ => 5,
        };

        // Adjust based on context
        let mut score: u8 = base;
        
        // More files = more complex
        if file_count > 5 {
            score += 1;
        }

        // Has tests = slightly easier (better safety net)
        if has_tests {
            score = score.saturating_sub(1).max(1);
        }

        // Refactoring in large codebase = more complex
        if has_refactoring && existing_file_count > 50 {
            score += 1;
        }

        // Patterns detected = easier to follow
        if context.patterns().len() > 2 {
            score = score.saturating_sub(1).max(1);
        }

        ComplexityScore::new(score.min(5))
    }
}

impl Default for ImplementationPlanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Planner configuration
#[derive(Debug, Clone)]
pub struct PlannerConfig {
    /// Maximum steps in a plan
    pub max_steps: usize,
    
    /// Enable parallel step detection
    pub detect_parallel: bool,
    
    /// Always generate rollback
    pub always_rollback: bool,
    
    /// Generate test steps
    pub generate_tests: bool,
    
    /// Generate documentation steps
    pub generate_docs: bool,
    
    /// Follow detected patterns
    pub follow_patterns: bool,
    
    /// Max steps per scope item
    pub max_steps_per_item: usize,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            max_steps: 100,
            detect_parallel: true,
            always_rollback: true,
            generate_tests: true,
            generate_docs: false,
            follow_patterns: true,
            max_steps_per_item: 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PackageId, RepoContext, ApprovalRef, Scope, ScopeItem, AcceptanceCriterion};
    use tempfile::tempdir;

    #[test]
    fn planner_creation() {
        let planner = ImplementationPlanner::new();
        assert_eq!(planner.config.max_steps, 100);
    }

    #[test]
    fn plan_empty_package() {
        let dir = tempdir().unwrap();
        let context = RepoContext::from_path(dir.path()).unwrap();
        let analyzed = crate::analyzer::RepoAnalyzer::new().analyze(&context).unwrap();

        let package = WorkPackage::new(PackageId::new("TEST-001"), "Test")
            .approve(ApprovalRef::new("VETO-1", "now"));

        let planner = ImplementationPlanner::new();
        let plan = planner.plan(&package, &analyzed);

        assert!(plan.is_ok());
    }

    #[test]
    fn plan_with_scope_and_criteria() {
        let dir = tempdir().unwrap();
        let context = RepoContext::from_path(dir.path()).unwrap();
        let analyzed = crate::analyzer::RepoAnalyzer::new().analyze(&context).unwrap();

        let package = WorkPackage::new(PackageId::new("TEST-001"), "Test Feature")
            .with_scope(
                Scope::new()
                    .include(ScopeItem::feature("User authentication"))
                    .include(ScopeItem::feature("Session management"))
            )
            .with_criterion(AcceptanceCriterion::required("AC1", "Users can login"))
            .approve(ApprovalRef::new("VETO-1", "now"));

        let planner = ImplementationPlanner::new();
        let plan = planner.plan(&package, &analyzed).unwrap();

        // Should have steps for features, tests, and verify
        assert!(!plan.steps.is_empty());
        assert!(plan.steps.iter().any(|s| s.step_type == crate::types::StepType::CreateFile));
        assert!(plan.steps.iter().any(|s| s.step_type == crate::types::StepType::Verify));
    }

    #[test]
    fn plan_rejects_unapproved_package() {
        let dir = tempdir().unwrap();
        let context = RepoContext::from_path(dir.path()).unwrap();
        let analyzed = crate::analyzer::RepoAnalyzer::new().analyze(&context).unwrap();

        let package = WorkPackage::new(PackageId::new("TEST-001"), "Test");
        // Not approved!

        let planner = ImplementationPlanner::new();
        let result = planner.plan(&package, &analyzed);

        assert!(result.is_err());
    }

    #[test]
    fn complexity_scoring() {
        let dir = tempdir().unwrap();
        let context = RepoContext::from_path(dir.path()).unwrap();
        let analyzed = crate::analyzer::RepoAnalyzer::new().analyze(&context).unwrap();

        let package = WorkPackage::new(PackageId::new("TEST-001"), "Simple")
            .with_scope(Scope::new().include(ScopeItem::feature("One feature")))
            .approve(ApprovalRef::new("VETO-1", "now"));

        let planner = ImplementationPlanner::new();
        let plan = planner.plan(&package, &analyzed).unwrap();

        // Small plan = low complexity
        assert!(plan.complexity.value() <= 3);
    }
}
