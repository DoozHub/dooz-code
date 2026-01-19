//! Audit Trail
//!
//! Append-only audit log for traceability.

use serde::{Deserialize, Serialize};
use super::Status;

/// Audit log (append-only)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditLog {
    /// Log entries
    entries: Vec<AuditEntry>,
}

impl AuditLog {
    /// Create new audit log
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Append status to log
    pub fn append(&mut self, status: Status) {
        let entry = AuditEntry {
            sequence: self.entries.len() as u64,
            status,
            recorded_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0),
        };
        self.entries.push(entry);
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get all entries
    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    /// Get entries for package
    pub fn for_package(&self, package_id: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.status.package_id == package_id)
            .collect()
    }

    /// Get last entry
    pub fn last(&self) -> Option<&AuditEntry> {
        self.entries.last()
    }

    /// Export to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.entries)
    }
}

/// Single audit entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Sequence number
    pub sequence: u64,
    
    /// Status that was recorded
    pub status: Status,
    
    /// When this was recorded (Unix timestamp)
    pub recorded_at: f64,
}

impl AuditEntry {
    /// Get package ID
    pub fn package_id(&self) -> &str {
        &self.status.package_id
    }

    /// Get status type
    pub fn status_type(&self) -> super::StatusType {
        self.status.status_type
    }
}

/// Audit summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    /// Total entries
    pub total_entries: usize,
    
    /// Packages executed
    pub packages: Vec<String>,
    
    /// Successful completions
    pub successes: usize,
    
    /// Failures
    pub failures: usize,
    
    /// Total duration
    pub total_duration_secs: f64,
}

impl AuditLog {
    /// Generate summary
    pub fn summarize(&self) -> AuditSummary {
        use std::collections::HashSet;
        use super::StatusType;

        let packages: HashSet<_> = self.entries
            .iter()
            .map(|e| e.status.package_id.clone())
            .collect();

        let successes = self.entries
            .iter()
            .filter(|e| e.status.status_type == StatusType::Completed)
            .count();

        let failures = self.entries
            .iter()
            .filter(|e| e.status.status_type == StatusType::Failed)
            .count();

        let duration = if let (Some(first), Some(last)) = (self.entries.first(), self.entries.last()) {
            last.recorded_at - first.recorded_at
        } else {
            0.0
        };

        AuditSummary {
            total_entries: self.entries.len(),
            packages: packages.into_iter().collect(),
            successes,
            failures,
            total_duration_secs: duration,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_log_append() {
        let mut log = AuditLog::new();
        log.append(Status::started("pkg-001"));
        log.append(Status::completed("pkg-001", 5));

        assert_eq!(log.len(), 2);
        assert_eq!(log.entries[0].sequence, 0);
        assert_eq!(log.entries[1].sequence, 1);
    }

    #[test]
    fn audit_log_filter() {
        let mut log = AuditLog::new();
        log.append(Status::started("pkg-001"));
        log.append(Status::started("pkg-002"));
        log.append(Status::completed("pkg-001", 5));

        let pkg1_entries = log.for_package("pkg-001");
        assert_eq!(pkg1_entries.len(), 2);
    }

    #[test]
    fn audit_summary() {
        let mut log = AuditLog::new();
        log.append(Status::started("pkg-001"));
        log.append(Status::completed("pkg-001", 5));
        log.append(Status::started("pkg-002"));
        log.append(Status::failed("pkg-002", "Error"));

        let summary = log.summarize();
        assert_eq!(summary.packages.len(), 2);
        assert_eq!(summary.successes, 1);
        assert_eq!(summary.failures, 1);
    }
}
