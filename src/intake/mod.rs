//! Intake Module - PRD to Technical Spec Conversion
//!
//! Converts natural language requirements (PRDs, user stories, feature requests)
//! into structured Technical Specifications.

mod parser;
mod spec;

pub use parser::*;
pub use spec::*;

use crate::orchestrator::{MultiProviderOrchestrator, UnifiedRequest, ProviderType};
use crate::types::Language;
use crate::executor::GenerationIntent;

/// Intake processor that converts PRDs to specs
pub struct IntakeProcessor {
    orchestrator: Option<MultiProviderOrchestrator>,
}

impl IntakeProcessor {
    pub fn new() -> Self {
        Self { orchestrator: None }
    }

    pub fn with_claude(api_key: &str) -> Result<Self, crate::ContextError> {
        Ok(Self {
            orchestrator: Some(MultiProviderOrchestrator::with_claude(api_key)?),
        })
    }

    /// Process a PRD and generate a technical spec
    pub async fn process(&self, prd: &str) -> Result<TechnicalSpec, IntakeError> {
        // If we have an LLM, use it for extraction
        if let Some(ref orchestrator) = self.orchestrator {
            self.process_with_llm(orchestrator, prd).await
        } else {
            // Fallback to heuristic parsing
            self.process_heuristic(prd)
        }
    }

    async fn process_with_llm(&self, _orchestrator: &MultiProviderOrchestrator, prd: &str) -> Result<TechnicalSpec, IntakeError> {
        // LLM-based extraction would go here
        // For now, fall back to heuristic
        self.process_heuristic(prd)
    }

    fn process_heuristic(&self, prd: &str) -> Result<TechnicalSpec, IntakeError> {
        let parser = PrdParser::new();
        parser.parse(prd)
    }
}

impl Default for IntakeProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Intake processing error
#[derive(Debug, thiserror::Error)]
pub enum IntakeError {
    #[error("Parsing failed: {0}")]
    Parse(String),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Invalid PRD: {0}")]
    InvalidPrd(String),
}
