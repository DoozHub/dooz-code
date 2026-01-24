//! Verification Report

use super::checks::{CheckResult, CheckStatus};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Complete verification report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationReport {
    pub spec_id: String,
    pub status: VerificationStatus,
    pub checks: Vec<CheckResult>,
    pub spec_coverage: f32,
    pub quality_score: f32,
    pub blocking_issues: Vec<String>,
    pub recommendations: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerificationStatus {
    Pass,
    Warn,
    Fail,
}

impl VerificationReport {
    pub fn new(spec_id: &str) -> Self {
        Self {
            spec_id: spec_id.to_string(),
            status: VerificationStatus::Pass,
            checks: Vec::new(),
            spec_coverage: 0.0,
            quality_score: 0.0,
            blocking_issues: Vec::new(),
            recommendations: Vec::new(),
            created_at: Utc::now(),
        }
    }

    pub fn add_check(&mut self, result: CheckResult) {
        self.checks.push(result);
    }

    pub fn finalize(&mut self) {
        // Count results
        let total = self.checks.len() as f32;
        let passed = self.checks.iter().filter(|c| c.status == CheckStatus::Pass).count() as f32;
        let failed = self.checks.iter().filter(|c| c.status == CheckStatus::Fail).count();
        let warned = self.checks.iter().filter(|c| c.status == CheckStatus::Warn).count();

        // Calculate scores
        self.spec_coverage = if total > 0.0 { passed / total } else { 0.0 };
        self.quality_score = if total > 0.0 {
            (passed + (warned as f32 * 0.5)) / total
        } else {
            0.0
        };

        // Collect blocking issues
        for check in &self.checks {
            if check.status == CheckStatus::Fail {
                self.blocking_issues.push(format!("{}: {}", check.check_name, check.message));
            }
        }

        // Collect recommendations
        for check in &self.checks {
            if check.status == CheckStatus::Warn {
                self.recommendations.push(format!("{}: {}", check.check_name, check.message));
            }
        }

        // Determine overall status
        self.status = if failed > 0 {
            VerificationStatus::Fail
        } else if warned > 0 {
            VerificationStatus::Warn
        } else {
            VerificationStatus::Pass
        };
    }

    /// Get a summary string
    pub fn summary(&self) -> String {
        format!(
            "Verification {}: {} checks, {:.0}% coverage, {:.0}% quality",
            match self.status {
                VerificationStatus::Pass => "PASSED",
                VerificationStatus::Warn => "WARNING",
                VerificationStatus::Fail => "FAILED",
            },
            self.checks.len(),
            self.spec_coverage * 100.0,
            self.quality_score * 100.0
        )
    }

    /// Check if verification passed
    pub fn is_pass(&self) -> bool {
        self.status == VerificationStatus::Pass
    }

    /// Check if there are blocking issues
    pub fn has_blockers(&self) -> bool {
        !self.blocking_issues.is_empty()
    }
}

impl std::fmt::Display for VerificationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Verification Report ===")?;
        writeln!(f, "Spec: {}", self.spec_id)?;
        writeln!(f, "Status: {:?}", self.status)?;
        writeln!(f, "Coverage: {:.0}%", self.spec_coverage * 100.0)?;
        writeln!(f, "Quality: {:.0}%", self.quality_score * 100.0)?;
        writeln!(f)?;
        
        writeln!(f, "Checks:")?;
        for check in &self.checks {
            let icon = match check.status {
                CheckStatus::Pass => "✓",
                CheckStatus::Warn => "⚠",
                CheckStatus::Fail => "✗",
            };
            writeln!(f, "  {} {}: {}", icon, check.check_name, check.message)?;
        }
        
        if !self.blocking_issues.is_empty() {
            writeln!(f)?;
            writeln!(f, "Blocking Issues:")?;
            for issue in &self.blocking_issues {
                writeln!(f, "  - {}", issue)?;
            }
        }
        
        if !self.recommendations.is_empty() {
            writeln!(f)?;
            writeln!(f, "Recommendations:")?;
            for rec in &self.recommendations {
                writeln!(f, "  - {}", rec)?;
            }
        }
        
        Ok(())
    }
}
