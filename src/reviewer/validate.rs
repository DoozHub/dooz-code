//! Criteria Validation
//!
//! Validates artifacts against acceptance criteria with pluggable rules.

use crate::types::{
    Artifact, ArtifactType, AcceptanceCriterion, CriterionType,
    ValidationResult, ValidationIssue, IssueType, IssueSeverity,
    ContextError,
};

/// Criteria validator with pluggable rules
pub struct CriteriaValidator {
    /// Validation rules
    rules: Vec<Box<dyn ValidationRule>>,
    
    /// Config
    config: ValidatorConfig,
}

/// Validator configuration
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    /// Require all criteria to pass
    pub strict: bool,
    
    /// Check code quality
    pub check_quality: bool,
    
    /// Check patterns
    pub check_patterns: bool,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            strict: true,
            check_quality: true,
            check_patterns: true,
        }
    }
}

impl CriteriaValidator {
    /// Create new validator with default rules
    pub fn new() -> Self {
        Self {
            rules: Self::default_rules(),
            config: ValidatorConfig::default(),
        }
    }

    /// Create with config
    pub fn with_config(config: ValidatorConfig) -> Self {
        Self {
            rules: Self::default_rules(),
            config,
        }
    }

    /// Default validation rules
    fn default_rules() -> Vec<Box<dyn ValidationRule>> {
        vec![
            Box::new(TestPresenceRule),
            Box::new(FunctionalRule),
            Box::new(SecurityRule),
            Box::new(PerformanceRule),
            Box::new(DocumentationRule),
        ]
    }

    /// Add custom rule
    pub fn add_rule(&mut self, rule: Box<dyn ValidationRule>) {
        self.rules.push(rule);
    }

    /// Validate artifacts against criteria
    pub fn validate(
        &self,
        artifacts: &[Artifact],
        criteria: &[AcceptanceCriterion],
    ) -> Result<ValidationResult, ContextError> {
        let mut passed = Vec::new();
        let mut failed = Vec::new();
        let mut issues = Vec::new();

        // Check each criterion
        for criterion in criteria {
            let check_result = self.check_criterion(criterion, artifacts);
            
            if check_result.passed {
                passed.push(criterion.id.clone());
            } else {
                if criterion.required || self.config.strict {
                    failed.push(criterion.id.clone());
                }
                issues.extend(check_result.issues);
            }
        }

        // Additional quality checks
        if self.config.check_quality {
            issues.extend(self.check_code_quality(artifacts));
        }

        // Determine final result
        if failed.is_empty() {
            Ok(ValidationResult::pass(passed))
        } else {
            Ok(ValidationResult::fail(passed, failed, issues))
        }
    }

    /// Check single criterion
    fn check_criterion(&self, criterion: &AcceptanceCriterion, artifacts: &[Artifact]) -> CriterionCheck {
        // Apply relevant rules based on criterion type
        for rule in &self.rules {
            if rule.applies_to(criterion) {
                return rule.check(criterion, artifacts);
            }
        }

        // Default: pass if criterion type not specifically handled
        CriterionCheck::pass()
    }

    /// Run code quality checks on all artifacts
    fn check_code_quality(&self, artifacts: &[Artifact]) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        for artifact in artifacts {
            // Check for TODO markers left in code
            if artifact.content.contains("TODO:") || artifact.content.contains("todo!") {
                issues.push(ValidationIssue::new(
                    IssueType::CriterionNotMet,
                    format!("File {} contains TODO markers", artifact.path),
                ).with_file(&artifact.path).with_severity(IssueSeverity::Warning));
            }

            // Check for empty implementations
            if artifact.content.contains("todo!(\"Implementation pending\")") {
                issues.push(ValidationIssue::new(
                    IssueType::CriterionNotMet,
                    format!("File {} has incomplete implementation", artifact.path),
                ).with_file(&artifact.path).with_severity(IssueSeverity::Error));
            }

            // Check for panic!/unwrap in production code
            if artifact.is_source() && !artifact.is_test() {
                if artifact.content.contains("panic!") {
                    issues.push(ValidationIssue::new(
                        IssueType::SecurityIssue,
                        format!("File {} uses panic! - consider proper error handling", artifact.path),
                    ).with_file(&artifact.path).with_severity(IssueSeverity::Warning));
                }

                if artifact.content.contains(".unwrap()") {
                    issues.push(ValidationIssue::new(
                        IssueType::SecurityIssue,
                        format!("File {} uses unwrap() - consider ? operator or expect()", artifact.path),
                    ).with_file(&artifact.path).with_severity(IssueSeverity::Warning));
                }
            }

            // Check for basic test structure
            if artifact.is_test() {
                if !artifact.content.contains("#[test]") && !artifact.content.contains("test(") {
                    issues.push(ValidationIssue::new(
                        IssueType::MissingTest,
                        format!("Test file {} has no test markers", artifact.path),
                    ).with_file(&artifact.path).with_severity(IssueSeverity::Error));
                }
            }
        }

        issues
    }
}

impl Default for CriteriaValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of checking a criterion
#[derive(Debug)]
pub struct CriterionCheck {
    pub passed: bool,
    pub issues: Vec<ValidationIssue>,
    pub suggestions: Vec<String>,
}

impl CriterionCheck {
    /// Create passing check
    pub fn pass() -> Self {
        Self {
            passed: true,
            issues: Vec::new(),
            suggestions: Vec::new(),
        }
    }

    /// Create failing check
    pub fn fail(issue: ValidationIssue) -> Self {
        Self {
            passed: false,
            issues: vec![issue],
            suggestions: Vec::new(),
        }
    }

    /// Fail with suggestion
    pub fn fail_with_suggestion(issue: ValidationIssue, suggestion: impl Into<String>) -> Self {
        Self {
            passed: false,
            issues: vec![issue],
            suggestions: vec![suggestion.into()],
        }
    }

    /// Add suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }
}

/// Validation rule trait
pub trait ValidationRule: Send + Sync {
    /// Check if rule applies to criterion
    fn applies_to(&self, criterion: &AcceptanceCriterion) -> bool;
    
    /// Check criterion against artifacts
    fn check(&self, criterion: &AcceptanceCriterion, artifacts: &[Artifact]) -> CriterionCheck;

    /// Get rule name
    fn name(&self) -> &str;
}

// ============================================================================
// Built-in validation rules
// ============================================================================

/// Rule: Tests must be present for testing criteria
pub struct TestPresenceRule;

impl ValidationRule for TestPresenceRule {
    fn applies_to(&self, criterion: &AcceptanceCriterion) -> bool {
        matches!(criterion.criterion_type, CriterionType::Testing)
    }

    fn check(&self, criterion: &AcceptanceCriterion, artifacts: &[Artifact]) -> CriterionCheck {
        let has_tests = artifacts.iter().any(|a| a.is_test());
        
        if has_tests {
            // Check test content quality
            let test_count: usize = artifacts.iter()
                .filter(|a| a.is_test())
                .map(|a| a.content.matches("#[test]").count() + a.content.matches("test(").count())
                .sum();

            if test_count == 0 {
                CriterionCheck::fail_with_suggestion(
                    ValidationIssue::new(
                        IssueType::MissingTest,
                        format!("Test files exist but no test functions found: {}", criterion.description),
                    ),
                    "Add #[test] annotations to test functions",
                )
            } else {
                CriterionCheck::pass()
            }
        } else {
            CriterionCheck::fail_with_suggestion(
                ValidationIssue::new(
                    IssueType::MissingTest,
                    format!("No tests found for criterion: {}", criterion.description),
                ),
                "Create test file with #[test] functions",
            )
        }
    }

    fn name(&self) -> &str {
        "TestPresence"
    }
}

/// Rule: Functional criteria require relevant source code
pub struct FunctionalRule;

impl ValidationRule for FunctionalRule {
    fn applies_to(&self, criterion: &AcceptanceCriterion) -> bool {
        matches!(criterion.criterion_type, CriterionType::Functional)
    }

    fn check(&self, criterion: &AcceptanceCriterion, artifacts: &[Artifact]) -> CriterionCheck {
        let source_files: Vec<_> = artifacts.iter()
            .filter(|a| a.is_source())
            .collect();

        if source_files.is_empty() {
            return CriterionCheck::fail(ValidationIssue::new(
                IssueType::CriterionNotMet,
                "No source code found",
            ));
        }

        // Check that source code has substance (not just stubs)
        let total_lines: u32 = source_files.iter().map(|a| a.line_count).sum();
        let has_implementation = source_files.iter()
            .any(|a| !a.content.contains("todo!(\"Implementation pending\")"));

        if !has_implementation {
            return CriterionCheck::fail_with_suggestion(
                ValidationIssue::new(
                    IssueType::CriterionNotMet,
                    format!("Criterion not implemented: {}", criterion.description),
                ),
                "Replace todo!() stubs with actual implementation",
            );
        }

        if total_lines < 5 {
            return CriterionCheck::fail_with_suggestion(
                ValidationIssue::new(
                    IssueType::CriterionNotMet,
                    format!("Implementation too minimal for: {}", criterion.description),
                ),
                "Add more complete implementation",
            );
        }

        CriterionCheck::pass()
    }

    fn name(&self) -> &str {
        "Functional"
    }
}

/// Rule: Security criteria check for common vulnerabilities
pub struct SecurityRule;

impl ValidationRule for SecurityRule {
    fn applies_to(&self, criterion: &AcceptanceCriterion) -> bool {
        matches!(criterion.criterion_type, CriterionType::Security)
    }

    fn check(&self, criterion: &AcceptanceCriterion, artifacts: &[Artifact]) -> CriterionCheck {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        for artifact in artifacts.iter().filter(|a| a.is_source()) {
            // Check for hardcoded credentials
            if artifact.content.to_lowercase().contains("password = \"") ||
               artifact.content.to_lowercase().contains("secret = \"") ||
               artifact.content.to_lowercase().contains("api_key = \"") {
                issues.push(ValidationIssue::new(
                    IssueType::SecurityIssue,
                    format!("Possible hardcoded credentials in {}", artifact.path),
                ).with_file(&artifact.path).with_severity(IssueSeverity::Error));
                suggestions.push("Use environment variables for secrets".to_string());
            }

            // Check for unsafe Rust
            if artifact.content.contains("unsafe {") {
                issues.push(ValidationIssue::new(
                    IssueType::SecurityIssue,
                    format!("Unsafe block in {}", artifact.path),
                ).with_file(&artifact.path).with_severity(IssueSeverity::Warning));
                suggestions.push("Document safety invariants for unsafe code".to_string());
            }

            // Check for SQL injection patterns (basic)
            if artifact.content.contains("format!(") && 
               (artifact.content.contains("SELECT") || artifact.content.contains("INSERT")) {
                issues.push(ValidationIssue::new(
                    IssueType::SecurityIssue,
                    format!("Possible SQL injection in {}", artifact.path),
                ).with_file(&artifact.path).with_severity(IssueSeverity::Error));
                suggestions.push("Use parameterized queries".to_string());
            }
        }

        if issues.is_empty() {
            CriterionCheck::pass()
        } else {
            CriterionCheck {
                passed: false,
                issues,
                suggestions,
            }
        }
    }

    fn name(&self) -> &str {
        "Security"
    }
}

/// Rule: Performance criteria check for obvious issues
pub struct PerformanceRule;

impl ValidationRule for PerformanceRule {
    fn applies_to(&self, criterion: &AcceptanceCriterion) -> bool {
        matches!(criterion.criterion_type, CriterionType::Performance)
    }

    fn check(&self, criterion: &AcceptanceCriterion, artifacts: &[Artifact]) -> CriterionCheck {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        for artifact in artifacts.iter().filter(|a| a.is_source()) {
            // Check for nested loops (O(n^2) potential)
            let loop_count = artifact.content.matches("for ").count() + 
                           artifact.content.matches("while ").count();
            
            // Very basic O(n²) detection
            if loop_count >= 2 && artifact.content.contains("for ") {
                // Check if loops appear nested (simplified heuristic)
                let lines: Vec<&str> = artifact.content.lines().collect();
                let mut in_loop = false;
                let mut nested = false;
                
                for line in &lines {
                    let trimmed = line.trim();
                    if trimmed.starts_with("for ") || trimmed.starts_with("while ") {
                        if in_loop {
                            nested = true;
                            break;
                        }
                        in_loop = true;
                    }
                    if trimmed.contains("}") {
                        in_loop = false;
                    }
                }
                
                if nested {
                    issues.push(ValidationIssue::new(
                        IssueType::PerformanceIssue,
                        format!("Potential O(n²) complexity in {}", artifact.path),
                    ).with_file(&artifact.path).with_severity(IssueSeverity::Warning));
                    suggestions.push("Consider using HashMap or more efficient algorithm".to_string());
                }
            }

            // Check for Clone in hot paths (basic)
            if artifact.content.matches(".clone()").count() > 5 {
                issues.push(ValidationIssue::new(
                    IssueType::PerformanceIssue,
                    format!("Many clone() calls in {} - consider borrowing", artifact.path),
                ).with_file(&artifact.path).with_severity(IssueSeverity::Warning));
                suggestions.push("Use references instead of cloning where possible".to_string());
            }
        }

        if issues.is_empty() {
            CriterionCheck::pass()
        } else {
            CriterionCheck {
                passed: false,
                issues,
                suggestions,
            }
        }
    }

    fn name(&self) -> &str {
        "Performance"
    }
}

/// Rule: Documentation criteria require docs
pub struct DocumentationRule;

impl ValidationRule for DocumentationRule {
    fn applies_to(&self, criterion: &AcceptanceCriterion) -> bool {
        matches!(criterion.criterion_type, CriterionType::Documentation)
    }

    fn check(&self, criterion: &AcceptanceCriterion, artifacts: &[Artifact]) -> CriterionCheck {
        // Check for documentation artifacts
        let has_docs = artifacts.iter().any(|a| {
            matches!(a.artifact_type, ArtifactType::Documentation) ||
            a.path.ends_with(".md") ||
            a.path.to_lowercase().contains("doc")
        });

        // Check for inline documentation
        let source_with_docs = artifacts.iter()
            .filter(|a| a.is_source())
            .any(|a| {
                a.content.contains("///") || 
                a.content.contains("//!") ||
                a.content.contains("/**")
            });

        if has_docs || source_with_docs {
            CriterionCheck::pass()
        } else {
            CriterionCheck::fail_with_suggestion(
                ValidationIssue::new(
                    IssueType::CriterionNotMet,
                    format!("No documentation found for: {}", criterion.description),
                ),
                "Add README.md or inline doc comments (/// for functions, //! for modules)",
            )
        }
    }

    fn name(&self) -> &str {
        "Documentation"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_with_tests() {
        let validator = CriteriaValidator::new();
        
        let artifacts = vec![
            Artifact::new("src/main.rs", "fn main() { println!(\"Hello\"); }", ArtifactType::Source),
            Artifact::new("tests/test.rs", "#[test]\nfn test() { assert!(true); }", ArtifactType::Test),
        ];
        
        let criteria = vec![
            AcceptanceCriterion::required("AC1", "Must have tests")
                .of_type(CriterionType::Testing),
        ];

        let result = validator.validate(&artifacts, &criteria).unwrap();
        assert!(result.is_pass());
    }

    #[test]
    fn validate_missing_tests() {
        let validator = CriteriaValidator::new();
        
        let artifacts = vec![
            Artifact::new("src/main.rs", "fn main() {}", ArtifactType::Source),
        ];
        
        let criteria = vec![
            AcceptanceCriterion::required("AC1", "Must have tests")
                .of_type(CriterionType::Testing),
        ];

        let result = validator.validate(&artifacts, &criteria).unwrap();
        assert!(!result.is_pass());
    }

    #[test]
    fn security_check_hardcoded_secret() {
        let rule = SecurityRule;
        let criterion = AcceptanceCriterion::required("SEC1", "No hardcoded secrets")
            .of_type(CriterionType::Security);
        
        let artifacts = vec![
            Artifact::new("src/config.rs", "let password = \"secret123\";", ArtifactType::Source),
        ];

        let result = rule.check(&criterion, &artifacts);
        assert!(!result.passed);
        assert!(result.issues.iter().any(|i| i.description.contains("hardcoded")));
    }

    #[test]
    fn documentation_check() {
        let rule = DocumentationRule;
        let criterion = AcceptanceCriterion::required("DOC1", "Must have docs")
            .of_type(CriterionType::Documentation);
        
        let with_docs = vec![
            Artifact::new("src/lib.rs", "/// Main function\npub fn main() {}", ArtifactType::Source),
        ];
        let without_docs = vec![
            Artifact::new("src/lib.rs", "pub fn main() {}", ArtifactType::Source),
        ];

        assert!(rule.check(&criterion, &with_docs).passed);
        assert!(!rule.check(&criterion, &without_docs).passed);
    }

    #[test]
    fn code_quality_todo_detection() {
        let validator = CriteriaValidator::new();
        let issues = validator.check_code_quality(&[
            Artifact::new("src/lib.rs", "// TODO: implement this", ArtifactType::Source),
        ]);

        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.description.contains("TODO")));
    }
}
