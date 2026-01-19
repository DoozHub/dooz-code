//! Implementation Step Types
//!
//! Defines individual steps within an implementation plan.
//! Steps are atomic units of execution.

use serde::{Deserialize, Serialize};
use super::identifiers::StepId;

/// A single implementation step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// Unique step identifier
    pub id: StepId,
    
    /// Human-readable description
    pub description: String,
    
    /// Type of step
    pub step_type: StepType,
    
    /// Target file or resource
    pub target: String,
    
    /// Dependencies on other steps
    pub depends_on: Vec<StepId>,
    
    /// Changes to apply
    pub changes: Vec<Change>,
    
    /// Status of this step
    pub status: StepStatus,
}

impl Step {
    /// Create a new step
    pub fn new(id: StepId, description: impl Into<String>, step_type: StepType) -> Self {
        Self {
            id,
            description: description.into(),
            step_type,
            target: String::new(),
            depends_on: Vec::new(),
            changes: Vec::new(),
            status: StepStatus::Pending,
        }
    }

    /// Set target
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = target.into();
        self
    }

    /// Add dependency
    pub fn depends_on(mut self, step_id: StepId) -> Self {
        self.depends_on.push(step_id);
        self
    }

    /// Add change
    pub fn with_change(mut self, change: Change) -> Self {
        self.changes.push(change);
        self
    }

    /// Check if step can execute (all dependencies complete)
    pub fn can_execute(&self, completed: &[StepId]) -> bool {
        self.depends_on.iter().all(|dep| completed.contains(dep))
    }

    /// Mark as executing
    pub fn start(&mut self) {
        self.status = StepStatus::Executing;
    }

    /// Mark as complete
    pub fn complete(&mut self) {
        self.status = StepStatus::Complete;
    }

    /// Mark as failed
    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = StepStatus::Failed(error.into());
    }
}

/// Type of implementation step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepType {
    /// Create a new file
    CreateFile,
    
    /// Modify an existing file
    ModifyFile,
    
    /// Delete a file
    DeleteFile,
    
    /// Add content to a file
    AddContent,
    
    /// Remove content from a file
    RemoveContent,
    
    /// Replace content in a file
    ReplaceContent,
    
    /// Create a test file
    CreateTest,
    
    /// Update configuration
    UpdateConfig,
    
    /// Run a verification
    Verify,
}

/// Status of a step
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepStatus {
    /// Not yet started
    Pending,
    
    /// Currently executing
    Executing,
    
    /// Successfully completed
    Complete,
    
    /// Failed with error
    Failed(String),
    
    /// Skipped (dependency failed)
    Skipped,
}

impl StepStatus {
    /// Check if step is complete
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Complete)
    }

    /// Check if step failed
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed(_))
    }

    /// Check if step is pending
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }
}

/// A change to apply to a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    /// Type of change
    pub change_type: ChangeType,
    
    /// Content to add/replace
    pub content: Option<String>,
    
    /// Location specification
    pub location: ChangeLocation,
    
    /// Reason for this change
    pub reason: String,
}

impl Change {
    /// Create a new change with type and reason
    pub fn new(change_type: ChangeType, reason: impl Into<String>) -> Self {
        Self {
            change_type,
            content: None,
            location: ChangeLocation::WholeFile,
            reason: reason.into(),
        }
    }

    /// With content
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// With location
    pub fn with_location(mut self, location: ChangeLocation) -> Self {
        self.location = location;
        self
    }

    /// Create a content addition
    pub fn add(content: impl Into<String>, location: ChangeLocation, reason: impl Into<String>) -> Self {
        Self {
            change_type: ChangeType::Add,
            content: Some(content.into()),
            location,
            reason: reason.into(),
        }
    }

    /// Create a content removal
    pub fn remove(location: ChangeLocation, reason: impl Into<String>) -> Self {
        Self {
            change_type: ChangeType::Remove,
            content: None,
            location,
            reason: reason.into(),
        }
    }

    /// Create a content replacement
    pub fn replace(content: impl Into<String>, location: ChangeLocation, reason: impl Into<String>) -> Self {
        Self {
            change_type: ChangeType::Replace,
            content: Some(content.into()),
            location,
            reason: reason.into(),
        }
    }

    /// Create file creation
    pub fn create_file(content: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            change_type: ChangeType::CreateFile,
            content: Some(content.into()),
            location: ChangeLocation::WholeFile,
            reason: reason.into(),
        }
    }

    /// Create file deletion
    pub fn delete_file(reason: impl Into<String>) -> Self {
        Self {
            change_type: ChangeType::DeleteFile,
            content: None,
            location: ChangeLocation::WholeFile,
            reason: reason.into(),
        }
    }
}

/// Type of change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// Add new content
    Add,
    
    /// Remove existing content
    Remove,
    
    /// Replace existing content
    Replace,
    
    /// Refactor existing content
    Refactor,
    
    /// Create entire file
    Create,
    
    /// Create entire file (alias)
    CreateFile,
    
    /// Modify existing file
    Modify,
    
    /// Delete entire file
    Delete,
    
    /// Delete entire file (alias)
    DeleteFile,
}

/// Location specification for a change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeLocation {
    /// Entire file
    WholeFile,
    
    /// Specific line range
    LineRange { start: u32, end: u32 },
    
    /// After a specific pattern
    AfterPattern { pattern: String },
    
    /// Before a specific pattern
    BeforePattern { pattern: String },
    
    /// Replace a specific pattern
    PatternMatch { pattern: String },
    
    /// At end of file
    EndOfFile,
    
    /// At start of file
    StartOfFile,
    
    /// Inside a specific block (function, class, etc.)
    InsideBlock { block_type: String, name: String },
}

impl ChangeLocation {
    /// Create line range location
    pub fn lines(start: u32, end: u32) -> Self {
        Self::LineRange { start, end }
    }

    /// Create after pattern location
    pub fn after(pattern: impl Into<String>) -> Self {
        Self::AfterPattern { pattern: pattern.into() }
    }

    /// Create before pattern location
    pub fn before(pattern: impl Into<String>) -> Self {
        Self::BeforePattern { pattern: pattern.into() }
    }

    /// Create pattern match location
    pub fn matching(pattern: impl Into<String>) -> Self {
        Self::PatternMatch { pattern: pattern.into() }
    }

    /// Create inside block location
    pub fn inside(block_type: impl Into<String>, name: impl Into<String>) -> Self {
        Self::InsideBlock {
            block_type: block_type.into(),
            name: name.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PackageId;

    #[test]
    fn step_creation() {
        let step = Step::new(
            StepId::new(&PackageId::new("TEST-001"), 1),
            "Create main file",
            StepType::CreateFile,
        )
        .with_target("src/main.rs");

        assert_eq!(step.target, "src/main.rs");
        assert!(step.status.is_pending());
    }

    #[test]
    fn step_dependency_check() {
        let pkg_id = PackageId::new("TEST-001");
        let step1_id = StepId::new(&pkg_id, 1);
        let step2_id = StepId::new(&pkg_id, 2);

        let step = Step::new(step2_id, "Step 2", StepType::ModifyFile)
            .depends_on(step1_id.clone());

        assert!(!step.can_execute(&[]));
        assert!(step.can_execute(&[step1_id]));
    }

    #[test]
    fn change_locations() {
        let loc1 = ChangeLocation::lines(10, 20);
        assert!(matches!(loc1, ChangeLocation::LineRange { start: 10, end: 20 }));

        let loc2 = ChangeLocation::after("import");
        assert!(matches!(loc2, ChangeLocation::AfterPattern { .. }));
    }
}
