//! Work Package Types
//!
//! Defines the structure of approved work packages that drive execution.
//! Work packages are immutable once created and define the exact scope.

use serde::{Deserialize, Serialize};
use super::identifiers::{PackageId, ApprovalRef};

/// A structured work package defining scope for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkPackage {
    /// Unique identifier
    pub id: PackageId,
    
    /// Human-readable title
    pub title: String,
    
    /// Detailed description of what should be implemented
    pub description: String,
    
    /// Acceptance criteria that must be satisfied
    pub criteria: Vec<AcceptanceCriterion>,
    
    /// Technical and business constraints
    pub constraints: Vec<Constraint>,
    
    /// Explicit scope boundaries
    pub scope: Scope,
    
    /// Approval reference from governance layer
    pub approval: Option<ApprovalRef>,
}

impl WorkPackage {
    /// Create a new work package (unapproved)
    pub fn new(id: PackageId, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            description: String::new(),
            criteria: Vec::new(),
            constraints: Vec::new(),
            scope: Scope::default(),
            approval: None,
        }
    }

    /// Check if work package has been approved
    pub fn is_approved(&self) -> bool {
        self.approval.is_some()
    }

    /// Add description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add acceptance criterion
    pub fn with_criterion(mut self, criterion: AcceptanceCriterion) -> Self {
        self.criteria.push(criterion);
        self
    }

    /// Add constraint
    pub fn with_constraint(mut self, constraint: Constraint) -> Self {
        self.constraints.push(constraint);
        self
    }

    /// Set scope
    pub fn with_scope(mut self, scope: Scope) -> Self {
        self.scope = scope;
        self
    }

    /// Approve work package
    pub fn approve(mut self, approval: ApprovalRef) -> Self {
        self.approval = Some(approval);
        self
    }
}

/// Acceptance criterion that must be satisfied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    /// Unique identifier within work package
    pub id: String,
    
    /// Description of what must be true
    pub description: String,
    
    /// Type of criterion
    pub criterion_type: CriterionType,
    
    /// Whether this is required for completion
    pub required: bool,
}

impl AcceptanceCriterion {
    /// Create a new required criterion
    pub fn required(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            criterion_type: CriterionType::Functional,
            required: true,
        }
    }

    /// Create an optional criterion
    pub fn optional(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            criterion_type: CriterionType::Functional,
            required: false,
        }
    }

    /// Set criterion type
    pub fn of_type(mut self, criterion_type: CriterionType) -> Self {
        self.criterion_type = criterion_type;
        self
    }
}

/// Type of acceptance criterion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CriterionType {
    /// Functional requirement (feature works)
    Functional,
    /// Security requirement
    Security,
    /// Performance requirement
    Performance,
    /// Testing requirement
    Testing,
    /// Documentation requirement
    Documentation,
}

/// Constraint on implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    /// Constraint identifier
    pub id: String,
    
    /// Description of constraint
    pub description: String,
    
    /// Type of constraint
    pub constraint_type: ConstraintType,
    
    /// Whether violation is a hard failure
    pub hard: bool,
}

impl Constraint {
    /// Create a hard constraint (must not be violated)
    pub fn hard(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            constraint_type: ConstraintType::Technical,
            hard: true,
        }
    }

    /// Create a soft constraint (prefer to follow)
    pub fn soft(id: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            constraint_type: ConstraintType::Technical,
            hard: false,
        }
    }

    /// Set constraint type
    pub fn of_type(mut self, constraint_type: ConstraintType) -> Self {
        self.constraint_type = constraint_type;
        self
    }
}

/// Type of constraint
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Technical constraint (architecture, patterns)
    Technical,
    /// Business constraint (compliance, policy)
    Business,
    /// Resource constraint (time, size)
    Resource,
    /// Governance constraint (from veto)
    Governance,
}

/// Explicit scope definition
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Scope {
    /// What is included in scope
    pub includes: Vec<ScopeItem>,
    
    /// What is explicitly excluded
    pub excludes: Vec<ScopeItem>,
    
    /// Files that may be modified
    pub allowed_files: Vec<String>,
    
    /// Maximum new files allowed
    pub max_new_files: Option<u32>,
    
    /// Maximum lines of code
    pub max_lines: Option<u32>,
}

impl Scope {
    /// Create a new scope
    pub fn new() -> Self {
        Self::default()
    }

    /// Add item to includes
    pub fn include(mut self, item: ScopeItem) -> Self {
        self.includes.push(item);
        self
    }

    /// Add item to excludes
    pub fn exclude(mut self, item: ScopeItem) -> Self {
        self.excludes.push(item);
        self
    }

    /// Set allowed files pattern
    pub fn allow_files(mut self, patterns: Vec<String>) -> Self {
        self.allowed_files = patterns;
        self
    }

    /// Set max new files
    pub fn with_max_files(mut self, max: u32) -> Self {
        self.max_new_files = Some(max);
        self
    }
}

/// Item in scope definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeItem {
    /// Description of scope item
    pub description: String,
    
    /// Category
    pub category: ScopeCategory,
}

impl ScopeItem {
    /// Create a feature scope item
    pub fn feature(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            category: ScopeCategory::Feature,
        }
    }

    /// Create a bugfix scope item
    pub fn bugfix(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            category: ScopeCategory::Bugfix,
        }
    }

    /// Create a refactor scope item
    pub fn refactor(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            category: ScopeCategory::Refactor,
        }
    }

    /// Create a test scope item
    pub fn test(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            category: ScopeCategory::Test,
        }
    }

    /// Create a documentation scope item
    pub fn documentation(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            category: ScopeCategory::Documentation,
        }
    }

    /// Create a file scope item
    pub fn file(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            category: ScopeCategory::File,
        }
    }

    /// Create a dependency scope item
    pub fn dependency(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            category: ScopeCategory::Dependency,
        }
    }
}

/// Category of scope item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeCategory {
    Feature,
    Bugfix,
    Refactor,
    Test,
    Documentation,
    File,
    Dependency,
    Integration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_package_builder() {
        let pkg = WorkPackage::new(PackageId::new("TEST-001"), "Test Package")
            .with_description("A test work package")
            .with_criterion(AcceptanceCriterion::required("AC1", "Must work"))
            .with_constraint(Constraint::hard("C1", "No external deps"));

        assert_eq!(pkg.title, "Test Package");
        assert_eq!(pkg.criteria.len(), 1);
        assert_eq!(pkg.constraints.len(), 1);
        assert!(!pkg.is_approved());
    }

    #[test]
    fn work_package_approval() {
        let pkg = WorkPackage::new(PackageId::new("TEST-001"), "Test")
            .approve(ApprovalRef::new("VETO-123", "2024-01-01T00:00:00Z"));

        assert!(pkg.is_approved());
    }

    #[test]
    fn scope_builder() {
        let scope = Scope::new()
            .include(ScopeItem::feature("User login"))
            .exclude(ScopeItem::feature("Password reset"))
            .with_max_files(5);

        assert_eq!(scope.includes.len(), 1);
        assert_eq!(scope.excludes.len(), 1);
        assert_eq!(scope.max_new_files, Some(5));
    }
}
