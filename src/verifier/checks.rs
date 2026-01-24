//! Verification Checks

use crate::intake::TechnicalSpec;
use crate::verifier::Artifact;
use serde::{Deserialize, Serialize};

/// Trait for verification checks
pub trait VerificationCheck: Send + Sync {
    fn name(&self) -> &str;
    fn run(&self, spec: &TechnicalSpec, artifacts: &[Artifact]) -> CheckResult;
}

/// Result of a single check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub check_name: String,
    pub status: CheckStatus,
    pub message: String,
    pub details: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

// === Check Implementations ===

/// Check for spec mismatches
pub struct SpecMismatchCheck;

impl VerificationCheck for SpecMismatchCheck {
    fn name(&self) -> &str { "Spec Mismatch" }
    
    fn run(&self, spec: &TechnicalSpec, artifacts: &[Artifact]) -> CheckResult {
        let mut mismatches = Vec::new();
        
        // Check if entities are implemented
        for entity in &spec.entities {
            let entity_found = artifacts.iter().any(|a| {
                a.content.to_lowercase().contains(&entity.name.to_lowercase())
            });
            
            if !entity_found {
                mismatches.push(format!("Entity '{}' not found in artifacts", entity.name));
            }
        }
        
        // Check if APIs are implemented
        for api in &spec.apis {
            let api_found = artifacts.iter().any(|a| {
                a.content.contains(&api.path) || 
                a.content.to_lowercase().contains(&api.description.to_lowercase())
            });
            
            if !api_found {
                mismatches.push(format!("API '{}' not found in artifacts", api.path));
            }
        }
        
        if mismatches.is_empty() {
            CheckResult {
                check_name: self.name().to_string(),
                status: CheckStatus::Pass,
                message: "All spec items found in artifacts".to_string(),
                details: vec![],
            }
        } else {
            CheckResult {
                check_name: self.name().to_string(),
                status: CheckStatus::Fail,
                message: format!("{} spec mismatches found", mismatches.len()),
                details: mismatches,
            }
        }
    }
}

/// Check for missing features
pub struct MissingFeatureCheck;

impl VerificationCheck for MissingFeatureCheck {
    fn name(&self) -> &str { "Missing Features" }
    
    fn run(&self, spec: &TechnicalSpec, artifacts: &[Artifact]) -> CheckResult {
        let mut missing = Vec::new();
        
        // Check acceptance criteria
        for criterion in &spec.acceptance_criteria {
            // Simple keyword check
            let keywords: Vec<&str> = criterion.split_whitespace()
                .filter(|w| w.len() > 4)
                .collect();
            
            let found = artifacts.iter().any(|a| {
                keywords.iter().any(|k| a.content.to_lowercase().contains(&k.to_lowercase()))
            });
            
            if !found && !criterion.contains("test") {
                missing.push(format!("Criterion may not be implemented: {}", criterion));
            }
        }
        
        if missing.is_empty() {
            CheckResult {
                check_name: self.name().to_string(),
                status: CheckStatus::Pass,
                message: "All features appear to be implemented".to_string(),
                details: vec![],
            }
        } else {
            CheckResult {
                check_name: self.name().to_string(),
                status: CheckStatus::Warn,
                message: format!("{} potential missing features", missing.len()),
                details: missing,
            }
        }
    }
}

/// Check for hallucinations (features not in spec)
pub struct HallucinationCheck;

impl VerificationCheck for HallucinationCheck {
    fn name(&self) -> &str { "Hallucination Detection" }
    
    fn run(&self, spec: &TechnicalSpec, artifacts: &[Artifact]) -> CheckResult {
        // Look for common hallucination patterns
        let suspicious_patterns = [
            "TODO: implement", "placeholder", "example only",
            "not implemented", "stub", "mock data"
        ];
        
        let mut issues = Vec::new();
        
        for artifact in artifacts {
            for pattern in &suspicious_patterns {
                if artifact.content.to_lowercase().contains(pattern) {
                    issues.push(format!("Suspicious pattern '{}' in {}", pattern, artifact.path));
                }
            }
        }
        
        if issues.is_empty() {
            CheckResult {
                check_name: self.name().to_string(),
                status: CheckStatus::Pass,
                message: "No obvious hallucinations detected".to_string(),
                details: vec![],
            }
        } else {
            CheckResult {
                check_name: self.name().to_string(),
                status: CheckStatus::Warn,
                message: format!("{} potential issues", issues.len()),
                details: issues,
            }
        }
    }
}

/// Check edge case coverage
pub struct EdgeCaseCheck;

impl VerificationCheck for EdgeCaseCheck {
    fn name(&self) -> &str { "Edge Case Coverage" }
    
    fn run(&self, spec: &TechnicalSpec, artifacts: &[Artifact]) -> CheckResult {
        let mut uncovered = Vec::new();
        
        for edge_case in &spec.edge_cases {
            // Look for error handling patterns
            let has_handling = artifacts.iter().any(|a| {
                a.content.contains("error") || 
                a.content.contains("Error") ||
                a.content.contains("catch") ||
                a.content.contains("try") ||
                a.content.contains("Result<")
            });
            
            if !has_handling {
                uncovered.push(format!("Edge case may not be handled: {}", edge_case));
            }
        }
        
        // We only fail if there are no error handling patterns at all
        if uncovered.len() == spec.edge_cases.len() && !spec.edge_cases.is_empty() {
            CheckResult {
                check_name: self.name().to_string(),
                status: CheckStatus::Warn,
                message: "Limited error handling detected".to_string(),
                details: uncovered,
            }
        } else {
            CheckResult {
                check_name: self.name().to_string(),
                status: CheckStatus::Pass,
                message: "Error handling patterns found".to_string(),
                details: vec![],
            }
        }
    }
}

/// Check test coverage
pub struct TestCoverageCheck;

impl VerificationCheck for TestCoverageCheck {
    fn name(&self) -> &str { "Test Coverage" }
    
    fn run(&self, _spec: &TechnicalSpec, artifacts: &[Artifact]) -> CheckResult {
        let test_patterns = ["#[test]", "#[cfg(test)]", "fn test_", "it(\"", "describe(\"", "test(\""];
        
        let has_tests = artifacts.iter().any(|a| {
            test_patterns.iter().any(|p| a.content.contains(p))
        });
        
        let test_files = artifacts.iter()
            .filter(|a| a.path.contains("test") || a.path.contains("spec"))
            .count();
        
        if has_tests || test_files > 0 {
            CheckResult {
                check_name: self.name().to_string(),
                status: CheckStatus::Pass,
                message: format!("Tests found ({} test files)", test_files),
                details: vec![],
            }
        } else {
            CheckResult {
                check_name: self.name().to_string(),
                status: CheckStatus::Warn,
                message: "No tests detected".to_string(),
                details: vec!["Consider adding unit tests".to_string()],
            }
        }
    }
}
