//! Execution Result Types
//!
//! Represents the outcome of execution including success, failure, and errors.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use super::artifact::Artifact;
use super::plan::Plan;
use super::identifiers::PackageId;

/// Result of executing a work package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Execution status
    pub status: ExecutionStatus,
    
    /// Generated artifacts
    pub artifacts: Vec<Artifact>,
    
    /// Execution plan that was followed
    pub plan: Plan,
    
    /// Total execution duration
    #[serde(with = "duration_serde")]
    pub duration: Duration,
    
    /// Number of iterations (0 if first attempt succeeded)
    pub iterations: u32,
    
    /// Execution log entries
    pub log: Vec<LogEntry>,
}

impl ExecutionResult {
    /// Create successful result
    pub fn success(artifacts: Vec<Artifact>, plan: Plan) -> Self {
        Self {
            status: ExecutionStatus::Success,
            artifacts,
            plan,
            duration: Duration::ZERO,
            iterations: 0,
            log: Vec::new(),
        }
    }

    /// Create successful result after iterations
    pub fn success_after_iteration(artifacts: Vec<Artifact>, plan: Plan, iterations: u32) -> Self {
        Self {
            status: ExecutionStatus::Success,
            artifacts,
            plan,
            duration: Duration::ZERO,
            iterations,
            log: Vec::new(),
        }
    }

    /// Set duration
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Add log entry
    pub fn with_log(mut self, entry: LogEntry) -> Self {
        self.log.push(entry);
        self
    }

    /// Check if successful
    pub fn is_success(&self) -> bool {
        matches!(self.status, ExecutionStatus::Success)
    }

    /// Get artifact count
    pub fn artifact_count(&self) -> usize {
        self.artifacts.len()
    }
}

/// Status of execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    /// Execution completed successfully
    Success,
    
    /// Execution completed with warnings
    SuccessWithWarnings,
    
    /// Execution failed
    Failed,
    
    /// Execution was cancelled
    Cancelled,
}

/// Log entry during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Log level
    pub level: LogLevel,
    
    /// Message
    pub message: String,
    
    /// Component that generated this log
    pub component: String,
    
    /// Step ID if applicable
    pub step_id: Option<String>,
    
    /// Timestamp (seconds since start)
    pub timestamp_secs: f64,
}

impl LogEntry {
    /// Create info log
    pub fn info(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: LogLevel::Info,
            message: message.into(),
            component: component.into(),
            step_id: None,
            timestamp_secs: 0.0,
        }
    }

    /// Create warning log
    pub fn warn(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: LogLevel::Warn,
            message: message.into(),
            component: component.into(),
            step_id: None,
            timestamp_secs: 0.0,
        }
    }

    /// Create error log
    pub fn error(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level: LogLevel::Error,
            message: message.into(),
            component: component.into(),
            step_id: None,
            timestamp_secs: 0.0,
        }
    }

    /// Set step ID
    pub fn for_step(mut self, step_id: impl Into<String>) -> Self {
        self.step_id = Some(step_id.into());
        self
    }

    /// Set timestamp
    pub fn at(mut self, timestamp_secs: f64) -> Self {
        self.timestamp_secs = timestamp_secs;
        self
    }
}

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Validation result from reviewer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Validation status
    pub status: ValidationStatus,
    
    /// Criteria that passed
    pub passed: Vec<String>,
    
    /// Criteria that failed
    pub failed: Vec<String>,
    
    /// Coverage percentage
    pub coverage: Option<f32>,
}

impl ValidationResult {
    /// Create passing validation
    pub fn pass(passed: Vec<String>) -> Self {
        Self {
            status: ValidationStatus::Pass,
            passed,
            failed: Vec::new(),
            coverage: None,
        }
    }

    /// Create failing validation
    pub fn fail(passed: Vec<String>, failed: Vec<String>, issues: Vec<ValidationIssue>) -> Self {
        Self {
            status: ValidationStatus::Fail { issues },
            passed,
            failed,
            coverage: None,
        }
    }

    /// Set coverage
    pub fn with_coverage(mut self, coverage: f32) -> Self {
        self.coverage = Some(coverage);
        self
    }

    /// Check if validation passed
    pub fn is_pass(&self) -> bool {
        matches!(self.status, ValidationStatus::Pass)
    }

    /// Get validation issues
    pub fn issues(&self) -> Vec<ValidationIssue> {
        match &self.status {
            ValidationStatus::Fail { issues } => issues.clone(),
            ValidationStatus::Pass => Vec::new(),
        }
    }
}

/// Validation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStatus {
    /// All criteria passed
    Pass,
    
    /// Some criteria failed
    Fail { issues: Vec<ValidationIssue> },
}

/// A validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Issue type
    pub issue_type: IssueType,
    
    /// Description
    pub description: String,
    
    /// Affected file
    pub file: Option<String>,
    
    /// Line number
    pub line: Option<u32>,
    
    /// Severity
    pub severity: IssueSeverity,
}

impl ValidationIssue {
    /// Create new issue
    pub fn new(issue_type: IssueType, description: impl Into<String>) -> Self {
        Self {
            issue_type,
            description: description.into(),
            file: None,
            line: None,
            severity: IssueSeverity::Error,
        }
    }

    /// Set file location with line
    pub fn in_file(mut self, file: impl Into<String>, line: u32) -> Self {
        self.file = Some(file.into());
        self.line = Some(line);
        self
    }

    /// Set file only (no line)
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Set severity
    pub fn with_severity(mut self, severity: IssueSeverity) -> Self {
        self.severity = severity;
        self
    }
}

/// Type of validation issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueType {
    /// Criterion not met
    CriterionNotMet,
    
    /// Test failure
    TestFailure,
    
    /// Pattern violation
    PatternViolation,
    
    /// Missing test
    MissingTest,
    
    /// Security issue
    SecurityIssue,
    
    /// Performance issue
    PerformanceIssue,
}

/// Severity of issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Execution error
#[derive(Debug, Clone)]
pub enum ExecutionError {
    /// Work package not approved
    NotApproved { package_id: PackageId },
    
    /// Context extraction failed
    ContextFailed { message: String },
    
    /// Planning failed
    PlanningFailed { message: String },
    
    /// Execution of step failed
    StepFailed { step_id: String, message: String },
    
    /// Validation failed after max iterations
    ValidationFailed { issues: Vec<ValidationIssue>, iterations: u32 },
    
    /// Scope violation detected
    ScopeViolation { message: String },
    
    /// IO error
    IoError { message: String },
}

impl std::fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotApproved { package_id } => {
                write!(f, "Work package {} not approved", package_id)
            }
            Self::ContextFailed { message } => {
                write!(f, "Context extraction failed: {}", message)
            }
            Self::PlanningFailed { message } => {
                write!(f, "Planning failed: {}", message)
            }
            Self::StepFailed { step_id, message } => {
                write!(f, "Step {} failed: {}", step_id, message)
            }
            Self::ValidationFailed { issues, iterations } => {
                write!(f, "Validation failed after {} iterations ({} issues)", iterations, issues.len())
            }
            Self::ScopeViolation { message } => {
                write!(f, "Scope violation: {}", message)
            }
            Self::IoError { message } => {
                write!(f, "IO error: {}", message)
            }
        }
    }
}

impl std::error::Error for ExecutionError {}

impl From<super::context::ContextError> for ExecutionError {
    fn from(err: super::context::ContextError) -> Self {
        ExecutionError::ContextFailed {
            message: err.to_string(),
        }
    }
}

/// Duration serialization helper
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_secs_f64().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = f64::deserialize(deserializer)?;
        Ok(Duration::from_secs_f64(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_result_success() {
        let plan = Plan::new(PackageId::new("TEST-001"));
        let result = ExecutionResult::success(vec![], plan);

        assert!(result.is_success());
        assert_eq!(result.iterations, 0);
    }

    #[test]
    fn validation_result() {
        let result = ValidationResult::pass(vec!["AC1".to_string(), "AC2".to_string()]);
        assert!(matches!(result.status, ValidationStatus::Pass));
        assert_eq!(result.passed.len(), 2);
    }

    #[test]
    fn log_entry_creation() {
        let entry = LogEntry::info("executor", "Starting execution")
            .for_step("step-001")
            .at(1.5);

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.step_id, Some("step-001".to_string()));
        assert_eq!(entry.timestamp_secs, 1.5);
    }
}
