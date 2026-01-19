//! Status Types
//!
//! Defines status messages for execution progress.

use serde::{Deserialize, Serialize};

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    /// Status type
    pub status_type: StatusType,
    
    /// Package ID
    pub package_id: String,
    
    /// Message
    pub message: String,
    
    /// Progress percentage (0-100)
    pub progress: Option<u8>,
    
    /// Current step
    pub step: Option<String>,
    
    /// Timestamp (seconds since start)
    pub timestamp: f64,
}

impl Status {
    /// Create started status
    pub fn started(package_id: impl Into<String>) -> Self {
        Self {
            status_type: StatusType::Started,
            package_id: package_id.into(),
            message: "Execution started".into(),
            progress: Some(0),
            step: None,
            timestamp: 0.0,
        }
    }

    /// Create progress status
    pub fn progress(
        package_id: impl Into<String>,
        step: impl Into<String>,
        progress: u8,
    ) -> Self {
        let step_str = step.into();
        Self {
            status_type: StatusType::Progress,
            package_id: package_id.into(),
            message: format!("Executing step: {}", step_str),
            progress: Some(progress.min(100)),
            step: Some(step_str),
            timestamp: 0.0,
        }
    }

    /// Create completed status
    pub fn completed(package_id: impl Into<String>, artifacts: usize) -> Self {
        Self {
            status_type: StatusType::Completed,
            package_id: package_id.into(),
            message: format!("Execution completed. {} artifacts generated.", artifacts),
            progress: Some(100),
            step: None,
            timestamp: 0.0,
        }
    }

    /// Create failed status
    pub fn failed(package_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            status_type: StatusType::Failed,
            package_id: package_id.into(),
            message: error.into(),
            progress: None,
            step: None,
            timestamp: 0.0,
        }
    }

    /// Create iterating status
    pub fn iterating(package_id: impl Into<String>, iteration: u32) -> Self {
        Self {
            status_type: StatusType::Iterating,
            package_id: package_id.into(),
            message: format!("Iteration {} - Correcting issues", iteration),
            progress: None,
            step: None,
            timestamp: 0.0,
        }
    }

    /// Set timestamp
    pub fn at(mut self, timestamp: f64) -> Self {
        self.timestamp = timestamp;
        self
    }
}

/// Type of status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusType {
    /// Execution started
    Started,
    
    /// Execution in progress
    Progress,
    
    /// Iterating to fix issues
    Iterating,
    
    /// Execution completed successfully
    Completed,
    
    /// Execution failed
    Failed,
    
    /// Execution cancelled
    Cancelled,
}

impl StatusType {
    /// Check if terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    /// Check if error state
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Failed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_creation() {
        let status = Status::started("pkg-001");
        assert_eq!(status.status_type, StatusType::Started);
        assert_eq!(status.progress, Some(0));
    }

    #[test]
    fn status_progress() {
        let status = Status::progress("pkg-001", "step-001", 50);
        assert_eq!(status.status_type, StatusType::Progress);
        assert_eq!(status.progress, Some(50));
    }

    #[test]
    fn terminal_states() {
        assert!(StatusType::Completed.is_terminal());
        assert!(StatusType::Failed.is_terminal());
        assert!(!StatusType::Progress.is_terminal());
    }
}
