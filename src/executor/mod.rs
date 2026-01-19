//! Code Executor
//!
//! Executes implementation plans by generating code and applying changes.
//! Execution is deterministic and respects scope boundaries.

mod step;
mod generate;
mod apply;
mod llm;

pub use step::*;
pub use generate::*;
pub use apply::*;
pub use llm::*;

use crate::types::{Plan, Artifact, ValidationIssue, ContextError};
use crate::analyzer::AnalyzedContext;
use std::sync::Arc;

/// Code executor component
pub struct CodeExecutor {
    /// Executor configuration
    config: ExecutorConfig,
    
    /// LLM provider
    llm: Arc<dyn LlmProvider>,
}

impl CodeExecutor {
    /// Create new executor with stub provider
    pub fn new() -> Self {
        Self {
            config: ExecutorConfig::default(),
            llm: Arc::new(StubLlmProvider::new()),
        }
    }

    /// Create executor with config
    pub fn with_config(config: ExecutorConfig) -> Self {
        Self { 
            config,
            llm: Arc::new(StubLlmProvider::new()),
        }
    }

    /// Create executor with custom LLM provider
    pub fn with_provider(llm: Arc<dyn LlmProvider>) -> Self {
        Self {
            config: ExecutorConfig::default(),
            llm,
        }
    }

    /// Create executor with config and provider
    pub fn with_config_and_provider(config: ExecutorConfig, llm: Arc<dyn LlmProvider>) -> Self {
        Self { config, llm }
    }

    /// Execute plan and generate artifacts
    pub fn execute(
        &self,
        plan: &Plan,
        context: &AnalyzedContext,
    ) -> Result<Vec<Artifact>, ContextError> {
        let mut artifacts = Vec::new();
        let mut step_executor = StepExecutor::with_provider(self.llm.clone());
        
        // Validate plan
        if plan.steps.len() > self.config.max_artifacts {
            return Err(ContextError::ValidationError(format!(
                "Plan has {} steps, exceeds max of {}",
                plan.steps.len(),
                self.config.max_artifacts
            )));
        }

        // Execute each step
        for step in &plan.steps {
            let step_artifacts = step_executor.execute(step, context)?;
            
            // Check line limits
            for artifact in &step_artifacts {
                if artifact.line_count > self.config.max_lines_per_file as u32 {
                    return Err(ContextError::ValidationError(format!(
                        "Artifact {} exceeds max lines: {} > {}",
                        artifact.path,
                        artifact.line_count,
                        self.config.max_lines_per_file
                    )));
                }
            }
            
            artifacts.extend(step_artifacts);
            step_executor.mark_executed(&step.id.to_string());
        }

        Ok(artifacts)
    }

    /// Attempt to correct artifacts based on issues
    pub fn correct(
        &self,
        artifacts: &[Artifact],
        issues: &[ValidationIssue],
        _context: &AnalyzedContext,
    ) -> Result<Vec<Artifact>, ContextError> {
        if issues.is_empty() {
            return Ok(artifacts.to_vec());
        }

        let mut corrected = Vec::new();

        for artifact in artifacts {
            // Find issues for this artifact
            let artifact_issues: Vec<_> = issues.iter()
                .filter(|i| i.file.as_deref() == Some(&artifact.path))
                .collect();

            if artifact_issues.is_empty() {
                corrected.push(artifact.clone());
                continue;
            }

            // Build correction request
            let language = detect_language_from_path(&artifact.path);
            let mut correction = CorrectionRequest::new(&artifact.content, language);
            
            for issue in artifact_issues {
                correction = correction.with_issue(&issue.description);
            }

            // Attempt correction
            match self.llm.correct_code(&correction) {
                Ok(response) => {
                    let mut fixed_artifact = artifact.clone();
                    fixed_artifact.content = response.code;
                    fixed_artifact.metadata.corrected = true;
                    corrected.push(fixed_artifact);
                }
                Err(_) => {
                    // Keep original if correction fails
                    corrected.push(artifact.clone());
                }
            }
        }

        Ok(corrected)
    }

    /// Apply artifacts to file system
    pub fn apply(&self, artifacts: &[Artifact], root: &std::path::Path) -> Result<ApplyResult, ContextError> {
        if self.config.dry_run {
            return Ok(ApplyResult::dry_run(artifacts.len()));
        }

        let applier = ChangeApplicator::new();
        applier.apply_all(artifacts, root)
    }

    /// Get provider name
    pub fn provider_name(&self) -> &str {
        self.llm.name()
    }
}

impl Default for CodeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Executor configuration
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Maximum artifacts to generate
    pub max_artifacts: usize,
    
    /// Maximum lines per file
    pub max_lines_per_file: usize,
    
    /// Dry run mode (no file writes)
    pub dry_run: bool,
    
    /// Follow detected patterns
    pub follow_patterns: bool,
    
    /// Enable correction attempts
    pub enable_correction: bool,
    
    /// Maximum correction iterations
    pub max_corrections: usize,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_artifacts: 100,
            max_lines_per_file: 1000,
            dry_run: false,
            follow_patterns: true,
            enable_correction: true,
            max_corrections: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PackageId, RepoContext};
    use crate::analyzer::RepoAnalyzer;
    use tempfile::tempdir;

    #[test]
    fn executor_creation() {
        let executor = CodeExecutor::new();
        assert_eq!(executor.config.max_artifacts, 100);
        assert!(!executor.config.dry_run);
        assert_eq!(executor.provider_name(), "stub");
    }

    #[test]
    fn execute_empty_plan() {
        let dir = tempdir().unwrap();
        let context = RepoContext::from_path(dir.path()).unwrap();
        let analyzed = RepoAnalyzer::new().analyze(&context).unwrap();
        let plan = Plan::new(PackageId::new("TEST-001"));

        let executor = CodeExecutor::new();
        let result = executor.execute(&plan, &analyzed);

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn executor_respects_dry_run() {
        let config = ExecutorConfig {
            dry_run: true,
            ..Default::default()
        };
        let executor = CodeExecutor::with_config(config);
        
        assert!(executor.config.dry_run);
    }

    #[test]
    fn executor_with_custom_provider() {
        let provider = Arc::new(StubLlmProvider::new());
        let executor = CodeExecutor::with_provider(provider);
        
        assert_eq!(executor.provider_name(), "stub");
    }
}
