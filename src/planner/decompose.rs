//! Step Decomposition
//!
//! Breaks work packages into individual implementation steps using context analysis.

use crate::types::{
    WorkPackage, Step, StepId, StepType, Change, ChangeType,
    ContextError, ScopeItem, ScopeCategory, CriterionType,
};
use crate::analyzer::{AnalyzedContext, DetectedPattern, PatternCategory};

/// Step decomposer - context-aware
pub struct StepDecomposer {
    /// Counter for step IDs
    step_counter: u32,
    
    /// Configuration
    config: DecomposerConfig,
}

/// Decomposer configuration
#[derive(Debug, Clone)]
pub struct DecomposerConfig {
    /// Generate test steps
    pub generate_tests: bool,
    
    /// Generate documentation steps
    pub generate_docs: bool,
    
    /// Follow detected patterns
    pub follow_patterns: bool,
    
    /// Max steps per scope item
    pub max_steps_per_item: usize,
}

impl Default for DecomposerConfig {
    fn default() -> Self {
        Self {
            generate_tests: true,
            generate_docs: false,
            follow_patterns: true,
            max_steps_per_item: 5,
        }
    }
}

impl StepDecomposer {
    /// Create new decomposer
    pub fn new() -> Self {
        Self { 
            step_counter: 0,
            config: DecomposerConfig::default(),
        }
    }

    /// Create with config
    pub fn with_config(config: DecomposerConfig) -> Self {
        Self {
            step_counter: 0,
            config,
        }
    }

    /// Decompose work package into steps using context
    pub fn decompose(
        &mut self,
        package: &WorkPackage,
        context: &AnalyzedContext,
    ) -> Result<Vec<Step>, ContextError> {
        let mut steps = Vec::new();

        // Detect patterns to follow
        let patterns = if self.config.follow_patterns {
            self.relevant_patterns(package, context)
        } else {
            Vec::new()
        };

        // Decompose each scope item
        for item in &package.scope.includes {
            let item_steps = self.decompose_scope_item(package, item, context, &patterns)?;
            steps.extend(item_steps);
        }

        // Generate test steps for acceptance criteria
        if self.config.generate_tests {
            let test_steps = self.generate_test_steps(package, context)?;
            steps.extend(test_steps);
        }

        // Generate documentation steps if needed
        if self.config.generate_docs {
            let doc_steps = self.generate_doc_steps(package)?;
            steps.extend(doc_steps);
        }

        // Add verification step
        if !steps.is_empty() {
            self.step_counter += 1;
            steps.push(Step::new(
                StepId::new(&package.id, self.step_counter),
                "Verify all changes compile and tests pass",
                StepType::Verify,
            ));
        }

        Ok(steps)
    }

    /// Get relevant patterns for this package
    fn relevant_patterns<'a>(
        &self,
        _package: &WorkPackage,
        context: &'a AnalyzedContext,
    ) -> Vec<&'a DetectedPattern> {
        context.pattern_analysis.detected.iter()
            .filter(|p| matches!(p.category, 
                PatternCategory::Structural | 
                PatternCategory::Architecture
            ))
            .collect()
    }

    /// Decompose a single scope item
    fn decompose_scope_item(
        &mut self,
        package: &WorkPackage,
        item: &ScopeItem,
        context: &AnalyzedContext,
        patterns: &[&DetectedPattern],
    ) -> Result<Vec<Step>, ContextError> {
        let mut steps = Vec::new();

        match item.category {
            ScopeCategory::Feature => {
                steps.extend(self.decompose_feature(package, item, context, patterns)?);
            }
            ScopeCategory::Bugfix => {
                steps.extend(self.decompose_bugfix(package, item, context)?);
            }
            ScopeCategory::Refactor => {
                steps.extend(self.decompose_refactor(package, item, context)?);
            }
            ScopeCategory::Test => {
                steps.extend(self.decompose_test(package, item, context)?);
            }
            ScopeCategory::Documentation => {
                steps.extend(self.decompose_documentation(package, item)?);
            }
            ScopeCategory::File | ScopeCategory::Dependency | ScopeCategory::Integration => {
                // Generic handling for file/dependency/integration
                steps.push(self.create_implementation_step(package, item)?);
            }
        }

        Ok(steps)
    }

    /// Decompose a feature scope item
    fn decompose_feature(
        &mut self,
        package: &WorkPackage,
        item: &ScopeItem,
        context: &AnalyzedContext,
        patterns: &[&DetectedPattern],
    ) -> Result<Vec<Step>, ContextError> {
        let mut steps = Vec::new();

        // Determine file structure based on patterns
        if patterns.iter().any(|p| p.name == "MVC") {
            // MVC pattern - create model, controller, view
            steps.extend(self.create_mvc_steps(package, item)?);
        } else if patterns.iter().any(|p| p.name == "Service Layer") {
            // Service layer - create service and potentially DTO
            steps.extend(self.create_service_steps(package, item)?);
        } else if patterns.iter().any(|p| p.name == "Component-Based") {
            // Component-based - create component file
            steps.extend(self.create_component_steps(package, item)?);
        } else {
            // Default: single file implementation
            steps.push(self.create_implementation_step(package, item)?);
        }

        // Check for config changes needed
        if item.description.to_lowercase().contains("config") 
            || item.description.to_lowercase().contains("setting") {
            steps.push(self.create_config_step(package, item)?);
        }

        Ok(steps)
    }

    /// Create MVC-style steps
    fn create_mvc_steps(
        &mut self,
        package: &WorkPackage,
        item: &ScopeItem,
    ) -> Result<Vec<Step>, ContextError> {
        let mut steps = Vec::new();
        let name = self.extract_entity_name(&item.description);

        // Model
        self.step_counter += 1;
        let model_step = Step::new(
            StepId::new(&package.id, self.step_counter),
            format!("Create model: {}", name),
            StepType::CreateFile,
        )
        .with_target(&format!("models/{}.rs", name.to_lowercase()))
        .with_change(Change::new(
            ChangeType::Create,
            format!("Model definition for {}", name),
        ));
        steps.push(model_step);

        // Controller
        self.step_counter += 1;
        let controller_step = Step::new(
            StepId::new(&package.id, self.step_counter),
            format!("Create controller: {}Controller", name),
            StepType::CreateFile,
        )
        .with_target(&format!("controllers/{}_controller.rs", name.to_lowercase()))
        .with_change(Change::new(
            ChangeType::Create,
            format!("Controller for {} endpoints", name),
        ))
        .depends_on(StepId::new(&package.id, self.step_counter - 1));
        steps.push(controller_step);

        Ok(steps)
    }

    /// Create service layer steps
    fn create_service_steps(
        &mut self,
        package: &WorkPackage,
        item: &ScopeItem,
    ) -> Result<Vec<Step>, ContextError> {
        let mut steps = Vec::new();
        let name = self.extract_entity_name(&item.description);

        // Service
        self.step_counter += 1;
        let service_step = Step::new(
            StepId::new(&package.id, self.step_counter),
            format!("Create service: {}Service", name),
            StepType::CreateFile,
        )
        .with_target(&format!("services/{}_service.rs", name.to_lowercase()))
        .with_change(Change::new(
            ChangeType::Create,
            format!("Business logic service for {}", name),
        ));
        steps.push(service_step);

        Ok(steps)
    }

    /// Create component-based steps (React/Vue/etc)
    fn create_component_steps(
        &mut self,
        package: &WorkPackage,
        item: &ScopeItem,
    ) -> Result<Vec<Step>, ContextError> {
        let mut steps = Vec::new();
        let name = self.extract_entity_name(&item.description);

        // Component file
        self.step_counter += 1;
        let component_step = Step::new(
            StepId::new(&package.id, self.step_counter),
            format!("Create component: {}", name),
            StepType::CreateFile,
        )
        .with_target(&format!("components/{}/{}.tsx", name, name))
        .with_change(Change::new(
            ChangeType::Create,
            format!("React component for {}", name),
        ));
        steps.push(component_step);

        // Component styles
        self.step_counter += 1;
        let styles_step = Step::new(
            StepId::new(&package.id, self.step_counter),
            format!("Create styles: {}.module.css", name),
            StepType::CreateFile,
        )
        .with_target(&format!("components/{}/{}.module.css", name, name))
        .with_change(Change::new(
            ChangeType::Create,
            format!("Styles for {} component", name),
        ));
        steps.push(styles_step);

        Ok(steps)
    }

    /// Create a simple implementation step
    fn create_implementation_step(
        &mut self,
        package: &WorkPackage,
        item: &ScopeItem,
    ) -> Result<Step, ContextError> {
        self.step_counter += 1;
        Ok(Step::new(
            StepId::new(&package.id, self.step_counter),
            format!("Implement: {}", item.description),
            StepType::CreateFile,
        )
        .with_change(Change::new(
            ChangeType::Create,
            item.description.clone(),
        )))
    }

    /// Create a config modification step
    fn create_config_step(
        &mut self,
        package: &WorkPackage,
        item: &ScopeItem,
    ) -> Result<Step, ContextError> {
        self.step_counter += 1;
        Ok(Step::new(
            StepId::new(&package.id, self.step_counter),
            format!("Update config for: {}", item.description),
            StepType::UpdateConfig,
        )
        .with_change(Change::new(
            ChangeType::Modify,
            format!("Configuration changes for {}", item.description),
        )))
    }

    /// Decompose a bugfix scope item
    fn decompose_bugfix(
        &mut self,
        package: &WorkPackage,
        item: &ScopeItem,
        _context: &AnalyzedContext,
    ) -> Result<Vec<Step>, ContextError> {
        let mut steps = Vec::new();

        // Identify affected file
        self.step_counter += 1;
        let fix_step = Step::new(
            StepId::new(&package.id, self.step_counter),
            format!("Fix: {}", item.description),
            StepType::ModifyFile,
        )
        .with_change(Change::new(
            ChangeType::Modify,
            format!("Bug fix: {}", item.description),
        ));
        steps.push(fix_step);

        // Add regression test
        if self.config.generate_tests {
            self.step_counter += 1;
            let test_step = Step::new(
                StepId::new(&package.id, self.step_counter),
                format!("Add regression test for: {}", item.description),
                StepType::CreateTest,
            )
            .depends_on(StepId::new(&package.id, self.step_counter - 1));
            steps.push(test_step);
        }

        Ok(steps)
    }

    /// Decompose a refactor scope item
    fn decompose_refactor(
        &mut self,
        package: &WorkPackage,
        item: &ScopeItem,
        _context: &AnalyzedContext,
    ) -> Result<Vec<Step>, ContextError> {
        let mut steps = Vec::new();

        // Main refactor step
        self.step_counter += 1;
        let refactor_step = Step::new(
            StepId::new(&package.id, self.step_counter),
            format!("Refactor: {}", item.description),
            StepType::ModifyFile,
        )
        .with_change(Change::new(
            ChangeType::Refactor,
            item.description.clone(),
        ));
        steps.push(refactor_step);

        Ok(steps)
    }

    /// Decompose a test scope item
    fn decompose_test(
        &mut self,
        package: &WorkPackage,
        item: &ScopeItem,
        _context: &AnalyzedContext,
    ) -> Result<Vec<Step>, ContextError> {
        let mut steps = Vec::new();

        self.step_counter += 1;
        let test_step = Step::new(
            StepId::new(&package.id, self.step_counter),
            format!("Create tests: {}", item.description),
            StepType::CreateTest,
        )
        .with_change(Change::new(
            ChangeType::Create,
            format!("Tests for {}", item.description),
        ));
        steps.push(test_step);

        Ok(steps)
    }

    /// Decompose a documentation scope item
    fn decompose_documentation(
        &mut self,
        package: &WorkPackage,
        item: &ScopeItem,
    ) -> Result<Vec<Step>, ContextError> {
        let mut steps = Vec::new();

        self.step_counter += 1;
        let doc_step = Step::new(
            StepId::new(&package.id, self.step_counter),
            format!("Document: {}", item.description),
            StepType::CreateFile,
        )
        .with_target("docs/")
        .with_change(Change::new(
            ChangeType::Create,
            format!("Documentation for {}", item.description),
        ));
        steps.push(doc_step);

        Ok(steps)
    }

    /// Generate test steps from acceptance criteria
    fn generate_test_steps(
        &mut self,
        package: &WorkPackage,
        _context: &AnalyzedContext,
    ) -> Result<Vec<Step>, ContextError> {
        let mut steps = Vec::new();

        // Group criteria by type
        let functional: Vec<_> = package.criteria.iter()
            .filter(|c| matches!(c.criterion_type, CriterionType::Functional))
            .collect();
        let testing: Vec<_> = package.criteria.iter()
            .filter(|c| matches!(c.criterion_type, CriterionType::Testing))
            .collect();

        // Create test file for functional criteria
        if !functional.is_empty() {
            self.step_counter += 1;
            let mut test_step = Step::new(
                StepId::new(&package.id, self.step_counter),
                format!("Create unit tests for {} functional criteria", functional.len()),
                StepType::CreateTest,
            );
            for criterion in &functional {
                test_step = test_step.with_change(Change::new(
                    ChangeType::Create,
                    format!("Test: {}", criterion.description),
                ));
            }
            steps.push(test_step);
        }

        // Create integration tests if specified
        if !testing.is_empty() {
            self.step_counter += 1;
            let mut integration_step = Step::new(
                StepId::new(&package.id, self.step_counter),
                format!("Create integration tests for {} testing criteria", testing.len()),
                StepType::CreateTest,
            );
            for criterion in &testing {
                integration_step = integration_step.with_change(Change::new(
                    ChangeType::Create,
                    format!("Integration test: {}", criterion.description),
                ));
            }
            steps.push(integration_step);
        }

        Ok(steps)
    }

    /// Generate documentation steps
    fn generate_doc_steps(
        &mut self,
        package: &WorkPackage,
    ) -> Result<Vec<Step>, ContextError> {
        let mut steps = Vec::new();

        // README update
        self.step_counter += 1;
        steps.push(Step::new(
            StepId::new(&package.id, self.step_counter),
            "Update README with new features",
            StepType::ModifyFile,
        )
        .with_target("README.md")
        .with_change(Change::new(
            ChangeType::Modify,
            format!("Documentation for {}", package.title),
        )));

        Ok(steps)
    }

    /// Extract entity name from description
    fn extract_entity_name(&self, description: &str) -> String {
        // Try to extract a PascalCase name from the description
        let words: Vec<&str> = description.split_whitespace()
            .filter(|w| {
                w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
            })
            .collect();

        if let Some(first) = words.first() {
            first.to_string()
        } else {
            // Fallback: capitalize first word
            description.split_whitespace()
                .next()
                .map(|w| {
                    let mut c = w.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .unwrap_or_else(|| "Entity".to_string())
        }
    }

    /// Reset counter
    pub fn reset(&mut self) {
        self.step_counter = 0;
    }
}

impl Default for StepDecomposer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PackageId, RepoContext, Scope, ScopeItem, AcceptanceCriterion, ApprovalRef};
    use crate::analyzer::RepoAnalyzer;
    use tempfile::tempdir;

    #[test]
    fn decompose_with_scope() {
        let dir = tempdir().unwrap();
        let context = RepoContext::from_path(dir.path()).unwrap();
        let analyzed = RepoAnalyzer::new().analyze(&context).unwrap();

        let package = WorkPackage::new(PackageId::new("TEST-001"), "Test")
            .with_scope(
                Scope::new()
                    .include(ScopeItem::feature("Feature A"))
                    .include(ScopeItem::feature("Feature B"))
            )
            .approve(ApprovalRef::new("VETO-1", "now"));

        let mut decomposer = StepDecomposer::new();
        let steps = decomposer.decompose(&package, &analyzed).unwrap();

        // 2 features + 1 verify step
        assert_eq!(steps.len(), 3);
    }

    #[test]
    fn decompose_with_criteria() {
        let dir = tempdir().unwrap();
        let context = RepoContext::from_path(dir.path()).unwrap();
        let analyzed = RepoAnalyzer::new().analyze(&context).unwrap();

        let package = WorkPackage::new(PackageId::new("TEST-001"), "Test")
            .with_criterion(AcceptanceCriterion::required("AC1", "Must work"))
            .approve(ApprovalRef::new("VETO-1", "now"));

        let mut decomposer = StepDecomposer::new();
        let steps = decomposer.decompose(&package, &analyzed).unwrap();

        // Should have test generation step + verify step
        assert!(steps.iter().any(|s| matches!(s.step_type, StepType::CreateTest)));
        assert!(steps.iter().any(|s| matches!(s.step_type, StepType::Verify)));
    }

    #[test]
    fn decompose_bugfix() {
        let dir = tempdir().unwrap();
        let context = RepoContext::from_path(dir.path()).unwrap();
        let analyzed = RepoAnalyzer::new().analyze(&context).unwrap();

        let package = WorkPackage::new(PackageId::new("TEST-001"), "Fix bug")
            .with_scope(
                Scope::new()
                    .include(ScopeItem::bugfix("Fix null pointer exception"))
            )
            .approve(ApprovalRef::new("VETO-1", "now"));

        let mut decomposer = StepDecomposer::new();
        let steps = decomposer.decompose(&package, &analyzed).unwrap();

        // Bugfix + regression test + verify
        assert!(steps.iter().any(|s| matches!(s.step_type, StepType::ModifyFile)));
        assert!(steps.iter().any(|s| matches!(s.step_type, StepType::CreateTest)));
    }

    #[test]
    fn extract_entity_names() {
        let decomposer = StepDecomposer::new();
        
        assert_eq!(decomposer.extract_entity_name("User authentication"), "User");
        assert_eq!(decomposer.extract_entity_name("Add UserProfile component"), "Add"); // First uppercase word
        assert_eq!(decomposer.extract_entity_name("UserProfile component"), "UserProfile");
        assert_eq!(decomposer.extract_entity_name("simple feature"), "Simple");
    }
}
