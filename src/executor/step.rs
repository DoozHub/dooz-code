//! Step Execution
//!
//! Executes individual implementation steps using LLM provider.

use crate::types::{Step, StepType, Artifact, ArtifactType, ContextError, Language};
use crate::analyzer::AnalyzedContext;
use super::llm::{LlmProvider, CodeRequest, GenerationIntent, ContextSummary, StubLlmProvider};
use std::sync::Arc;

/// Step executor with LLM integration
pub struct StepExecutor {
    /// Track execution state
    executed: Vec<String>,
    
    /// LLM provider
    llm: Arc<dyn LlmProvider>,
}

impl StepExecutor {
    /// Create new executor with stub provider
    pub fn new() -> Self {
        Self {
            executed: Vec::new(),
            llm: Arc::new(StubLlmProvider::new()),
        }
    }

    /// Create executor with custom LLM provider
    pub fn with_provider(llm: Arc<dyn LlmProvider>) -> Self {
        Self {
            executed: Vec::new(),
            llm,
        }
    }

    /// Execute a single step
    pub fn execute(
        &self,
        step: &Step,
        context: &AnalyzedContext,
    ) -> Result<Vec<Artifact>, ContextError> {
        let mut artifacts = Vec::new();

        match step.step_type {
            StepType::CreateFile => {
                let artifact = self.execute_create_file(step, context)?;
                artifacts.push(artifact);
            }
            StepType::CreateTest => {
                let artifact = self.execute_create_test(step, context)?;
                artifacts.push(artifact);
            }
            StepType::ModifyFile => {
                if let Some(artifact) = self.execute_modify_file(step, context)? {
                    artifacts.push(artifact);
                }
            }
            StepType::AddContent => {
                if let Some(artifact) = self.execute_add_content(step, context)? {
                    artifacts.push(artifact);
                }
            }
            StepType::ReplaceContent => {
                if let Some(artifact) = self.execute_replace_content(step, context)? {
                    artifacts.push(artifact);
                }
            }
            StepType::RemoveContent => {
                // Removal doesn't produce new artifacts, just modification
                if let Some(artifact) = self.execute_remove_content(step, context)? {
                    artifacts.push(artifact);
                }
            }
            StepType::DeleteFile => {
                // Deletion produces a deletion marker artifact
                artifacts.push(self.execute_delete_file(step)?);
            }
            StepType::UpdateConfig => {
                let artifact = self.execute_update_config(step, context)?;
                artifacts.push(artifact);
            }
            StepType::Verify => {
                // Verification doesn't produce artifacts
                self.execute_verify(step, context)?;
            }
        }

        Ok(artifacts)
    }

    /// Execute file creation
    fn execute_create_file(&self, step: &Step, context: &AnalyzedContext) -> Result<Artifact, ContextError> {
        let language = detect_language_from_path(&step.target);
        let context_summary = ContextSummary::from_context(context);
        
        let request = CodeRequest::new(&step.description, &step.target, language)
            .with_intent(GenerationIntent::Implementation)
            .with_context(context_summary);
        
        let response = self.llm.generate_code(&request)?;
        
        let mut artifact = Artifact::new(&step.target, &response.code, ArtifactType::Source);
        artifact.metadata.generator = self.llm.name().to_string();
        artifact.metadata.confidence = response.confidence;
        
        Ok(artifact)
    }

    /// Execute test creation
    fn execute_create_test(&self, step: &Step, context: &AnalyzedContext) -> Result<Artifact, ContextError> {
        let language = detect_language_from_path(&step.target);
        let context_summary = ContextSummary::from_context(context);
        
        let request = CodeRequest::new(&step.description, &step.target, language)
            .with_intent(GenerationIntent::Test)
            .with_context(context_summary);
        
        let response = self.llm.generate_code(&request)?;
        
        let mut artifact = Artifact::new(&step.target, &response.code, ArtifactType::Test);
        artifact.metadata.generator = self.llm.name().to_string();
        artifact.metadata.confidence = response.confidence;
        
        Ok(artifact)
    }

    /// Execute file modification
    fn execute_modify_file(&self, step: &Step, context: &AnalyzedContext) -> Result<Option<Artifact>, ContextError> {
        if step.target.is_empty() {
            return Ok(None);
        }
        
        let language = detect_language_from_path(&step.target);
        let context_summary = ContextSummary::from_context(context);
        
        let request = CodeRequest::new(&step.description, &step.target, language)
            .with_intent(GenerationIntent::Modification)
            .with_context(context_summary);
        
        let response = self.llm.generate_code(&request)?;
        
        let mut artifact = Artifact::new(&step.target, &response.code, ArtifactType::Modification);
        artifact.metadata.generator = self.llm.name().to_string();
        artifact.metadata.is_partial = true;
        
        Ok(Some(artifact))
    }

    /// Execute content addition
    fn execute_add_content(&self, step: &Step, context: &AnalyzedContext) -> Result<Option<Artifact>, ContextError> {
        if step.target.is_empty() {
            return Ok(None);
        }
        
        let language = detect_language_from_path(&step.target);
        let context_summary = ContextSummary::from_context(context);
        
        let request = CodeRequest::new(&step.description, &step.target, language)
            .with_intent(GenerationIntent::Modification)
            .with_context(context_summary)
            .with_constraint("Add new content without modifying existing code");
        
        let response = self.llm.generate_code(&request)?;
        
        let mut artifact = Artifact::new(&step.target, &response.code, ArtifactType::Modification);
        artifact.metadata.generator = self.llm.name().to_string();
        artifact.metadata.is_partial = true;
        artifact.metadata.operation = "add".to_string();
        
        Ok(Some(artifact))
    }

    /// Execute content replacement
    fn execute_replace_content(&self, step: &Step, context: &AnalyzedContext) -> Result<Option<Artifact>, ContextError> {
        if step.target.is_empty() {
            return Ok(None);
        }
        
        let language = detect_language_from_path(&step.target);
        let context_summary = ContextSummary::from_context(context);
        
        let request = CodeRequest::new(&step.description, &step.target, language)
            .with_intent(GenerationIntent::Modification)
            .with_context(context_summary)
            .with_constraint("Replace existing content with new implementation");
        
        let response = self.llm.generate_code(&request)?;
        
        let mut artifact = Artifact::new(&step.target, &response.code, ArtifactType::Modification);
        artifact.metadata.generator = self.llm.name().to_string();
        artifact.metadata.is_partial = true;
        artifact.metadata.operation = "replace".to_string();
        
        Ok(Some(artifact))
    }

    /// Execute content removal
    fn execute_remove_content(&self, step: &Step, _context: &AnalyzedContext) -> Result<Option<Artifact>, ContextError> {
        if step.target.is_empty() {
            return Ok(None);
        }
        
        // Removal doesn't generate new content, just marks for deletion
        let mut artifact = Artifact::new(&step.target, "", ArtifactType::Deletion);
        artifact.metadata.operation = "remove".to_string();
        
        Ok(Some(artifact))
    }

    /// Execute file deletion
    fn execute_delete_file(&self, step: &Step) -> Result<Artifact, ContextError> {
        let mut artifact = Artifact::new(&step.target, "", ArtifactType::Deletion);
        artifact.metadata.operation = "delete".to_string();
        
        Ok(artifact)
    }

    /// Execute config update
    fn execute_update_config(&self, step: &Step, context: &AnalyzedContext) -> Result<Artifact, ContextError> {
        let language = detect_language_from_path(&step.target);
        let context_summary = ContextSummary::from_context(context);
        
        let request = CodeRequest::new(&step.description, &step.target, language)
            .with_intent(GenerationIntent::Modification)
            .with_context(context_summary)
            .with_constraint("Update configuration only, preserve existing structure");
        
        let response = self.llm.generate_code(&request)?;
        
        let mut artifact = Artifact::new(&step.target, &response.code, ArtifactType::Config);
        artifact.metadata.generator = self.llm.name().to_string();
        
        Ok(artifact)
    }

    /// Execute verification
    fn execute_verify(&self, _step: &Step, _context: &AnalyzedContext) -> Result<(), ContextError> {
        // Verification would run tests, linters, etc.
        // For now, just succeed
        Ok(())
    }

    /// Check if step has been executed
    pub fn is_executed(&self, step_id: &str) -> bool {
        self.executed.contains(&step_id.to_string())
    }

    /// Mark step as executed
    pub fn mark_executed(&mut self, step_id: &str) {
        self.executed.push(step_id.to_string());
    }

    /// Get execution history
    pub fn history(&self) -> &[String] {
        &self.executed
    }
}

impl Default for StepExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect language from file path
pub fn detect_language_from_path(path: &str) -> Language {
    let path_lower = path.to_lowercase();
    
    if path_lower.ends_with(".rs") {
        Language::Rust
    } else if path_lower.ends_with(".ts") || path_lower.ends_with(".tsx") {
        Language::TypeScript
    } else if path_lower.ends_with(".js") || path_lower.ends_with(".jsx") {
        Language::JavaScript
    } else if path_lower.ends_with(".py") {
        Language::Python
    } else if path_lower.ends_with(".go") {
        Language::Go
    } else if path_lower.ends_with(".java") {
        Language::Java
    } else if path_lower.ends_with(".php") {
        Language::PHP
    } else if path_lower.ends_with(".rb") {
        Language::Ruby
    } else if path_lower.ends_with(".c") || path_lower.ends_with(".h") {
        Language::C
    } else if path_lower.ends_with(".cpp") || path_lower.ends_with(".cc") || path_lower.ends_with(".hpp") {
        Language::Cpp
    } else {
        Language::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PackageId, StepId, RepoContext};
    use crate::analyzer::RepoAnalyzer;
    use tempfile::tempdir;

    #[test]
    fn execute_create_file() {
        let dir = tempdir().unwrap();
        let context = RepoContext::from_path(dir.path()).unwrap();
        let analyzed = RepoAnalyzer::new().analyze(&context).unwrap();

        let step = Step::new(
            StepId::new(&PackageId::new("TEST-001"), 1),
            "Create main file",
            StepType::CreateFile,
        )
        .with_target("src/main.rs");

        let executor = StepExecutor::new();
        let artifacts = executor.execute(&step, &analyzed).unwrap();

        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].path, "src/main.rs");
        assert!(!artifacts[0].content.is_empty());
    }

    #[test]
    fn execute_create_test() {
        let dir = tempdir().unwrap();
        let context = RepoContext::from_path(dir.path()).unwrap();
        let analyzed = RepoAnalyzer::new().analyze(&context).unwrap();

        let step = Step::new(
            StepId::new(&PackageId::new("TEST-001"), 1),
            "Create test file",
            StepType::CreateTest,
        )
        .with_target("tests/test_main.rs");

        let executor = StepExecutor::new();
        let artifacts = executor.execute(&step, &analyzed).unwrap();

        assert_eq!(artifacts.len(), 1);
        assert!(artifacts[0].is_test());
    }

    #[test]
    fn detect_languages() {
        assert_eq!(detect_language_from_path("src/main.rs"), Language::Rust);
        assert_eq!(detect_language_from_path("src/app.ts"), Language::TypeScript);
        assert_eq!(detect_language_from_path("src/app.tsx"), Language::TypeScript);
        assert_eq!(detect_language_from_path("main.py"), Language::Python);
        assert_eq!(detect_language_from_path("main.go"), Language::Go);
        assert_eq!(detect_language_from_path("Unknown.xyz"), Language::Other);
    }
    
    #[test]
    fn execution_tracking() {
        let mut executor = StepExecutor::new();
        
        assert!(!executor.is_executed("step-1"));
        executor.mark_executed("step-1");
        assert!(executor.is_executed("step-1"));
        assert_eq!(executor.history().len(), 1);
    }
}
