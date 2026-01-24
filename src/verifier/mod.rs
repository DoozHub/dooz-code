//! Verifier Module - Compliance Verification
//!
//! Verifies generated artifacts against specifications for:
//! - Spec mismatch detection
//! - Missing feature detection
//! - Hallucination detection
//! - Edge case coverage

mod checks;
mod report;

pub use checks::*;
pub use report::*;

use crate::intake::TechnicalSpec;
use std::path::Path;

/// Verifier that checks artifacts against specs
pub struct Verifier {
    checks: Vec<Box<dyn VerificationCheck>>,
}

impl Verifier {
    pub fn new() -> Self {
        Self {
            checks: vec![
                Box::new(SpecMismatchCheck),
                Box::new(MissingFeatureCheck),
                Box::new(HallucinationCheck),
                Box::new(EdgeCaseCheck),
                Box::new(TestCoverageCheck),
            ],
        }
    }

    /// Verify artifacts against a specification
    pub fn verify(&self, spec: &TechnicalSpec, artifacts_path: &Path) -> VerificationReport {
        let mut report = VerificationReport::new(&spec.id);
        
        // Collect artifact files
        let artifacts = self.collect_artifacts(artifacts_path);
        
        // Run all checks
        for check in &self.checks {
            let result = check.run(spec, &artifacts);
            report.add_check(result);
        }
        
        // Calculate overall status
        report.finalize();
        
        report
    }

    fn collect_artifacts(&self, path: &Path) -> Vec<Artifact> {
        let mut artifacts = Vec::new();
        
        if path.is_file() {
            if let Ok(content) = std::fs::read_to_string(path) {
                artifacts.push(Artifact {
                    path: path.to_string_lossy().to_string(),
                    content,
                });
            }
        } else if path.is_dir() {
            for entry in walkdir::WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let file_path = entry.path();
                // Skip hidden files and common non-code files
                if file_path.to_string_lossy().contains("/.") {
                    continue;
                }
                
                if let Ok(content) = std::fs::read_to_string(file_path) {
                    artifacts.push(Artifact {
                        path: file_path.to_string_lossy().to_string(),
                        content,
                    });
                }
            }
        }
        
        artifacts
    }
}

impl Default for Verifier {
    fn default() -> Self {
        Self::new()
    }
}

/// An artifact to verify
#[derive(Debug, Clone)]
pub struct Artifact {
    pub path: String,
    pub content: String,
}

/// Verification error
#[derive(Debug, thiserror::Error)]
pub enum VerifyError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("Spec error: {0}")]
    Spec(String),
}
