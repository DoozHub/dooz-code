//! Artifact Reviewer
//!
//! Validates generated artifacts against acceptance criteria.
//! Review is deterministic and scope-aware.

mod validate;
mod iterate;

pub use validate::*;
pub use iterate::*;

use crate::types::{Artifact, AcceptanceCriterion, ValidationResult, ValidationStatus, ContextError};

/// Artifact reviewer component
pub struct ArtifactReviewer {
    /// Reviewer configuration
    config: ReviewerConfig,
    
    /// Validator
    validator: CriteriaValidator,
    
    /// Iteration handler
    iteration_handler: IterationHandler,
}

impl ArtifactReviewer {
    /// Create new reviewer
    pub fn new() -> Self {
        Self {
            config: ReviewerConfig::default(),
            validator: CriteriaValidator::new(),
            iteration_handler: IterationHandler::new(3),
        }
    }

    /// Create reviewer with config
    pub fn with_config(config: ReviewerConfig) -> Self {
        let validator_config = ValidatorConfig {
            strict: config.strict,
            check_quality: config.check_quality,
            check_patterns: config.check_patterns,
        };
        Self { 
            config: config.clone(),
            validator: CriteriaValidator::with_config(validator_config),
            iteration_handler: IterationHandler::new(config.max_iterations),
        }
    }

    /// Validate artifacts against criteria
    pub fn validate(
        &self,
        artifacts: &[Artifact],
        criteria: &[AcceptanceCriterion],
    ) -> Result<ValidationResult, ContextError> {
        self.validator.validate(artifacts, criteria)
    }

    /// Run full review with iteration support
    pub fn review(
        &mut self,
        artifacts: &[Artifact],
        criteria: &[AcceptanceCriterion],
    ) -> Result<ReviewResult, ContextError> {
        let mut current_result = self.validate(artifacts, criteria)?;
        let mut all_issues = Vec::new();

        // Initial validation
        if current_result.is_pass() {
            return Ok(ReviewResult {
                final_status: ReviewStatus::Approved,
                validation: current_result,
                iterations: 0,
                summary: self.iteration_handler.summary(),
            });
        }

        // Collect initial issues
        all_issues.extend(current_result.issues());

        // Track iterations needed
        let mut iteration_count = 0;

        while self.iteration_handler.can_iterate() && !current_result.is_pass() {
            if let Some(iter_num) = self.iteration_handler.next() {
                iteration_count = iter_num;

                // Plan corrections
                let corrections = self.iteration_handler.plan_corrections(&current_result.issues());
                
                // Record iteration
                self.iteration_handler.record(&current_result.issues(), &corrections);

                // Check if we're making progress
                if !self.iteration_handler.is_improving() && iteration_count > 1 {
                    break; // Stop if not improving
                }

                // In a real implementation, corrections would be applied here
                // For now, we just track that corrections were planned
            }
        }

        let final_status = if current_result.is_pass() {
            ReviewStatus::Approved
        } else if iteration_count >= self.config.max_iterations {
            ReviewStatus::Rejected {
                reason: format!("Max iterations ({}) reached", self.config.max_iterations),
            }
        } else {
            ReviewStatus::NeedsWork {
                issues: current_result.issues().len(),
            }
        };

        Ok(ReviewResult {
            final_status,
            validation: current_result,
            iterations: iteration_count,
            summary: self.iteration_handler.summary(),
        })
    }

    /// Get iteration handler for external iteration control
    pub fn iteration_handler(&self) -> &IterationHandler {
        &self.iteration_handler
    }

    /// Get mutable iteration handler
    pub fn iteration_handler_mut(&mut self) -> &mut IterationHandler {
        &mut self.iteration_handler
    }

    /// Add custom validation rule
    pub fn add_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.validator.add_rule(rule);
    }
}

impl Default for ArtifactReviewer {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of full review process
#[derive(Debug)]
pub struct ReviewResult {
    /// Final review status
    pub final_status: ReviewStatus,
    
    /// Last validation result
    pub validation: ValidationResult,
    
    /// Number of iterations performed
    pub iterations: u32,
    
    /// Iteration summary
    pub summary: IterationSummary,
}

impl ReviewResult {
    /// Check if approved
    pub fn is_approved(&self) -> bool {
        matches!(self.final_status, ReviewStatus::Approved)
    }

    /// Get remaining issues
    pub fn issue_count(&self) -> usize {
        self.validation.issues().len()
    }
}

/// Review status
#[derive(Debug, Clone)]
pub enum ReviewStatus {
    /// All criteria passed
    Approved,
    
    /// Some issues remain but can be fixed
    NeedsWork { issues: usize },
    
    /// Cannot be approved
    Rejected { reason: String },
}

/// Reviewer configuration
#[derive(Debug, Clone)]
pub struct ReviewerConfig {
    /// Strict mode (all criteria must pass)
    pub strict: bool,
    
    /// Require tests
    pub require_tests: bool,
    
    /// Minimum coverage percentage
    pub min_coverage: Option<f32>,
    
    /// Maximum correction iterations
    pub max_iterations: u32,
    
    /// Check code quality
    pub check_quality: bool,
    
    /// Check patterns
    pub check_patterns: bool,
}

impl Default for ReviewerConfig {
    fn default() -> Self {
        Self {
            strict: true,
            require_tests: true,
            min_coverage: None,
            max_iterations: 3,
            check_quality: true,
            check_patterns: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ArtifactType, CriterionType};

    #[test]
    fn reviewer_creation() {
        let reviewer = ArtifactReviewer::new();
        assert!(reviewer.config.strict);
    }

    #[test]
    fn validate_empty() {
        let reviewer = ArtifactReviewer::new();
        let result = reviewer.validate(&[], &[]).unwrap();
        assert!(matches!(result.status, ValidationStatus::Pass));
    }

    #[test]
    fn validate_with_criteria() {
        let reviewer = ArtifactReviewer::new();
        
        // Need enough lines to pass functional checks (min 5 lines)
        let artifacts = vec![
            Artifact::new("src/main.rs", 
                "//! Main module\n\
                 pub fn main() {\n\
                     println!(\"Hello\");\n\
                     run();\n\
                 }\n\
                 pub fn run() { }",
                ArtifactType::Source),
            Artifact::new("tests/test.rs", 
                "//! Tests\n\
                 #[test]\n\
                 fn test_main() {\n\
                     assert!(true);\n\
                 }",
                ArtifactType::Test),
        ];
        
        let criteria = vec![
            AcceptanceCriterion::required("AC1", "Code must work")
                .of_type(CriterionType::Functional),
            AcceptanceCriterion::required("AC2", "Must have tests")
                .of_type(CriterionType::Testing),
        ];

        let result = reviewer.validate(&artifacts, &criteria).unwrap();
        assert!(result.is_pass());
    }

    #[test]
    fn full_review_pass() {
        let mut reviewer = ArtifactReviewer::new();
        
        // Need enough lines to pass functional checks
        let artifacts = vec![
            Artifact::new("src/main.rs", 
                "//! Main module\n\
                 pub fn main() {\n\
                     println!(\"Hello\");\n\
                     run();\n\
                 }\n\
                 pub fn run() { }",
                ArtifactType::Source),
            Artifact::new("tests/test.rs", 
                "//! Tests\n\
                 #[test]\n\
                 fn test_main() {\n\
                     assert!(true);\n\
                 }",
                ArtifactType::Test),
        ];
        
        let criteria = vec![
            AcceptanceCriterion::required("AC1", "Code must work")
                .of_type(CriterionType::Functional),
        ];

        let result = reviewer.review(&artifacts, &criteria).unwrap();
        assert!(result.is_approved());
        assert_eq!(result.iterations, 0);
    }

    #[test]
    fn review_config() {
        let config = ReviewerConfig {
            strict: false,
            max_iterations: 5,
            ..Default::default()
        };
        let reviewer = ArtifactReviewer::with_config(config);
        
        assert!(!reviewer.config.strict);
        assert_eq!(reviewer.config.max_iterations, 5);
    }
}
