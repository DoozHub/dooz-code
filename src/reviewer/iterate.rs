//! Iteration Logic
//!
//! Handles correction iterations when validation fails.

use crate::types::{Artifact, ValidationIssue, ValidationResult, IssueType, IssueSeverity};

/// Iteration handler for correction loops
pub struct IterationHandler {
    /// Maximum iterations allowed
    max_iterations: u32,
    
    /// Current iteration
    current: u32,
    
    /// History of validation results
    history: Vec<IterationRecord>,
}

/// Record of an iteration
#[derive(Debug, Clone)]
pub struct IterationRecord {
    /// Iteration number
    pub iteration: u32,
    
    /// Issues at this iteration
    pub issues: Vec<ValidationIssue>,
    
    /// Corrections applied
    pub corrections: Vec<Correction>,
    
    /// Whether iteration improved
    pub improved: bool,
}

impl IterationHandler {
    /// Create new handler
    pub fn new(max_iterations: u32) -> Self {
        Self {
            max_iterations,
            current: 0,
            history: Vec::new(),
        }
    }

    /// Check if more iterations are allowed
    pub fn can_iterate(&self) -> bool {
        self.current < self.max_iterations
    }

    /// Start next iteration
    pub fn next(&mut self) -> Option<u32> {
        if self.can_iterate() {
            self.current += 1;
            Some(self.current)
        } else {
            None
        }
    }

    /// Get current iteration
    pub fn current(&self) -> u32 {
        self.current
    }

    /// Get remaining iterations
    pub fn remaining(&self) -> u32 {
        self.max_iterations.saturating_sub(self.current)
    }

    /// Record iteration result
    pub fn record(&mut self, issues: &[ValidationIssue], corrections: &[Correction]) {
        let improved = self.history.last()
            .map(|prev| issues.len() < prev.issues.len())
            .unwrap_or(true);

        self.history.push(IterationRecord {
            iteration: self.current,
            issues: issues.to_vec(),
            corrections: corrections.to_vec(),
            improved,
        });
    }

    /// Check if iterations are making progress
    pub fn is_improving(&self) -> bool {
        if self.history.len() < 2 {
            return true;
        }

        // Check if last 2 iterations improved
        self.history.iter().rev().take(2).all(|r| r.improved)
    }

    /// Get iteration history
    pub fn history(&self) -> &[IterationRecord] {
        &self.history
    }

    /// Generate correction plan from issues
    pub fn plan_corrections(&self, issues: &[ValidationIssue]) -> Vec<Correction> {
        let mut corrections: Vec<Correction> = issues.iter().map(|issue| {
            Correction {
                issue_type: issue.issue_type,
                target: issue.file.clone(),
                description: issue.description.clone(),
                strategy: CorrectionStrategy::from_issue(issue),
                priority: priority_from_severity(issue.severity),
            }
        }).collect();

        // Sort by priority (highest first)
        corrections.sort_by(|a, b| b.priority.cmp(&a.priority));

        corrections
    }

    /// Generate summary report
    pub fn summary(&self) -> IterationSummary {
        IterationSummary {
            total_iterations: self.current,
            max_iterations: self.max_iterations,
            total_issues_found: self.history.iter().map(|r| r.issues.len()).sum(),
            total_corrections: self.history.iter().map(|r| r.corrections.len()).sum(),
            improvements: self.history.iter().filter(|r| r.improved).count(),
            final_issue_count: self.history.last().map(|r| r.issues.len()).unwrap_or(0),
        }
    }
}

impl Default for IterationHandler {
    fn default() -> Self {
        Self::new(3)
    }
}

/// Summary of iteration process
#[derive(Debug, Clone)]
pub struct IterationSummary {
    pub total_iterations: u32,
    pub max_iterations: u32,
    pub total_issues_found: usize,
    pub total_corrections: usize,
    pub improvements: usize,
    pub final_issue_count: usize,
}

impl IterationSummary {
    /// Check if fully resolved
    pub fn is_resolved(&self) -> bool {
        self.final_issue_count == 0
    }

    /// Check if max iterations hit
    pub fn hit_limit(&self) -> bool {
        self.total_iterations >= self.max_iterations
    }
}

/// Correction to apply
#[derive(Debug, Clone)]
pub struct Correction {
    /// Issue type being corrected
    pub issue_type: IssueType,
    
    /// Target file
    pub target: Option<String>,
    
    /// Description
    pub description: String,
    
    /// Correction strategy
    pub strategy: CorrectionStrategy,
    
    /// Priority (higher = more urgent)
    pub priority: u8,
}

impl Correction {
    /// Create new correction
    pub fn new(issue_type: IssueType, description: impl Into<String>) -> Self {
        Self {
            issue_type,
            target: None,
            description: description.into(),
            strategy: CorrectionStrategy::FixExisting,
            priority: 5,
        }
    }

    /// With target file
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// With strategy
    pub fn with_strategy(mut self, strategy: CorrectionStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// With priority
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
}

/// Strategy for correction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrectionStrategy {
    /// Add missing content
    AddMissing,
    
    /// Fix existing content
    FixExisting,
    
    /// Replace content entirely
    Replace,
    
    /// Generate new file/content
    GenerateNew,
    
    /// Delete problematic content
    Delete,
    
    /// Refactor code structure
    Refactor,
}

impl CorrectionStrategy {
    /// Determine strategy from issue
    pub fn from_issue(issue: &ValidationIssue) -> Self {
        match issue.issue_type {
            IssueType::MissingTest => Self::GenerateNew,
            IssueType::CriterionNotMet => Self::AddMissing,
            IssueType::PatternViolation => Self::Refactor,
            IssueType::TestFailure => Self::FixExisting,
            IssueType::SecurityIssue => Self::Replace,
            IssueType::PerformanceIssue => Self::Refactor,
        }
    }

    /// Get description
    pub fn description(&self) -> &str {
        match self {
            Self::AddMissing => "Add missing functionality",
            Self::FixExisting => "Fix existing implementation",
            Self::Replace => "Replace problematic code",
            Self::GenerateNew => "Generate new code",
            Self::Delete => "Remove problematic code",
            Self::Refactor => "Refactor code structure",
        }
    }
}

/// Convert severity to priority
fn priority_from_severity(severity: IssueSeverity) -> u8 {
    match severity {
        IssueSeverity::Critical => 15,
        IssueSeverity::Error => 10,
        IssueSeverity::Warning => 5,
        IssueSeverity::Info => 2,
    }
}

/// Correction applicator
pub struct CorrectionApplicator;

impl CorrectionApplicator {
    /// Apply corrections to artifacts (placeholder for LLM integration)
    pub fn apply(
        artifacts: &[Artifact],
        corrections: &[Correction],
    ) -> Vec<CorrectionAction> {
        corrections.iter().map(|c| {
            CorrectionAction {
                correction: c.clone(),
                target_artifact: c.target.as_ref().and_then(|t| {
                    artifacts.iter().position(|a| &a.path == t)
                }),
                action_type: match c.strategy {
                    CorrectionStrategy::GenerateNew => ActionType::Create,
                    CorrectionStrategy::Delete => ActionType::Delete,
                    _ => ActionType::Modify,
                },
            }
        }).collect()
    }
}

/// Action to take for correction
#[derive(Debug)]
pub struct CorrectionAction {
    pub correction: Correction,
    pub target_artifact: Option<usize>,
    pub action_type: ActionType,
}

/// Type of action
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Create,
    Modify,
    Delete,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iteration_limits() {
        let mut handler = IterationHandler::new(3);
        
        assert!(handler.can_iterate());
        assert_eq!(handler.remaining(), 3);
        
        handler.next();
        handler.next();
        handler.next();
        
        assert!(!handler.can_iterate());
        assert_eq!(handler.remaining(), 0);
    }

    #[test]
    fn correction_strategy() {
        let issue = ValidationIssue::new(IssueType::MissingTest, "No test");
        let strategy = CorrectionStrategy::from_issue(&issue);
        
        assert_eq!(strategy, CorrectionStrategy::GenerateNew);
    }

    #[test]
    fn iteration_tracking() {
        let mut handler = IterationHandler::new(5);
        
        handler.next();
        handler.record(&[
            ValidationIssue::new(IssueType::MissingTest, "Test 1"),
            ValidationIssue::new(IssueType::MissingTest, "Test 2"),
        ], &[]);

        handler.next();
        handler.record(&[
            ValidationIssue::new(IssueType::MissingTest, "Test 1"),
        ], &[]);

        assert_eq!(handler.history().len(), 2);
        assert!(handler.is_improving());
        
        let summary = handler.summary();
        assert_eq!(summary.total_iterations, 2);
        assert_eq!(summary.final_issue_count, 1);
    }

    #[test]
    fn correction_priority() {
        let handler = IterationHandler::new(3);
        let issues = vec![
            ValidationIssue::new(IssueType::MissingTest, "Low")
                .with_severity(IssueSeverity::Info),
            ValidationIssue::new(IssueType::SecurityIssue, "High")
                .with_severity(IssueSeverity::Error),
            ValidationIssue::new(IssueType::PatternViolation, "Medium")
                .with_severity(IssueSeverity::Warning),
        ];

        let corrections = handler.plan_corrections(&issues);
        
        // Should be sorted by priority (Error first)
        assert_eq!(corrections[0].priority, 10); // Error
        assert_eq!(corrections[1].priority, 5);  // Warning
        assert_eq!(corrections[2].priority, 2);  // Info
    }
}
