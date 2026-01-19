//! Change Application
//!
//! Applies generated changes to the file system.

use std::path::Path;
use crate::types::{Artifact, ContextError};

/// Change applicator
pub struct ChangeApplicator {
    /// Dry run mode
    dry_run: bool,
    
    /// Applied changes
    applied: Vec<AppliedChange>,
}

impl ChangeApplicator {
    /// Create new applicator
    pub fn new() -> Self {
        Self {
            dry_run: false,
            applied: Vec::new(),
        }
    }

    /// Create in dry run mode
    pub fn dry_run() -> Self {
        Self {
            dry_run: true,
            applied: Vec::new(),
        }
    }

    /// Apply artifacts to file system
    pub fn apply(
        &mut self,
        root: &Path,
        artifacts: &[Artifact],
    ) -> Result<Vec<AppliedChange>, ContextError> {
        let mut changes = Vec::new();

        for artifact in artifacts {
            let change = self.apply_artifact(root, artifact)?;
            changes.push(change.clone());
            self.applied.push(change);
        }

        Ok(changes)
    }

    /// Apply single artifact
    fn apply_artifact(
        &self,
        root: &Path,
        artifact: &Artifact,
    ) -> Result<AppliedChange, ContextError> {
        let full_path = root.join(&artifact.path);
        let existed = full_path.exists();

        if self.dry_run {
            return Ok(AppliedChange {
                path: artifact.path.clone(),
                change_type: if existed { ChangeType::Modified } else { ChangeType::Created },
                dry_run: true,
                bytes_written: artifact.size,
            });
        }

        // Create parent directories
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ContextError::ReadError(format!("Failed to create directory: {}", e))
            })?;
        }

        // Write file
        std::fs::write(&full_path, &artifact.content).map_err(|e| {
            ContextError::ReadError(format!("Failed to write file: {}", e))
        })?;

        Ok(AppliedChange {
            path: artifact.path.clone(),
            change_type: if existed { ChangeType::Modified } else { ChangeType::Created },
            dry_run: false,
            bytes_written: artifact.size,
        })
    }

    /// Get applied changes
    pub fn applied(&self) -> &[AppliedChange] {
        &self.applied
    }

    /// Get total bytes written
    pub fn total_bytes(&self) -> usize {
        self.applied.iter().map(|c| c.bytes_written).sum()
    }
}

impl Default for ChangeApplicator {
    fn default() -> Self {
        Self::new()
    }
}

impl ChangeApplicator {
    /// Apply all artifacts and return result summary
    pub fn apply_all(&self, artifacts: &[Artifact], root: &Path) -> Result<ApplyResult, ContextError> {
        let mut result = ApplyResult::default();
        
        for artifact in artifacts {
            let full_path = root.join(&artifact.path);
            let existed = full_path.exists();

            if self.dry_run {
                result.processed += 1;
                result.dry_run = true;
                continue;
            }

            // Create parent directories
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    ContextError::ReadError(format!("Failed to create directory: {}", e))
                })?;
            }

            // Write file
            std::fs::write(&full_path, &artifact.content).map_err(|e| {
                ContextError::ReadError(format!("Failed to write file: {}", e))
            })?;

            if existed {
                result.modified += 1;
            } else {
                result.created += 1;
            }
            result.processed += 1;
            result.bytes_written += artifact.size;
        }

        result.success = true;
        Ok(result)
    }
}

/// Result of applying changes
#[derive(Debug, Clone, Default)]
pub struct ApplyResult {
    /// Whether apply succeeded
    pub success: bool,
    
    /// Was this a dry run
    pub dry_run: bool,
    
    /// Files created
    pub created: usize,
    
    /// Files modified
    pub modified: usize,
    
    /// Files deleted
    pub deleted: usize,
    
    /// Total files processed
    pub processed: usize,
    
    /// Bytes written
    pub bytes_written: usize,
}

impl ApplyResult {
    /// Create dry run result
    pub fn dry_run(count: usize) -> Self {
        Self {
            dry_run: true,
            processed: count,
            success: true,
            ..Default::default()
        }
    }

    /// Total changes
    pub fn total(&self) -> usize {
        self.created + self.modified + self.deleted
    }
}

/// Applied change record
#[derive(Debug, Clone)]
pub struct AppliedChange {
    /// File path
    pub path: String,
    
    /// Type of change
    pub change_type: ChangeType,
    
    /// Was this a dry run
    pub dry_run: bool,
    
    /// Bytes written
    pub bytes_written: usize,
}

/// Type of applied change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Created,
    Modified,
    Deleted,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ArtifactType;
    use tempfile::tempdir;

    #[test]
    fn dry_run_apply() {
        let artifact = Artifact::new(
            "src/test.rs",
            "fn test() {}",
            ArtifactType::Source,
        );

        let mut applicator = ChangeApplicator::dry_run();
        let dir = tempdir().unwrap();

        let changes = applicator.apply(dir.path(), &[artifact]).unwrap();

        assert_eq!(changes.len(), 1);
        assert!(changes[0].dry_run);
        assert!(!dir.path().join("src/test.rs").exists());
    }

    #[test]
    fn actual_apply() {
        let artifact = Artifact::new(
            "src/test.rs",
            "fn test() {}",
            ArtifactType::Source,
        );

        let mut applicator = ChangeApplicator::new();
        let dir = tempdir().unwrap();

        let changes = applicator.apply(dir.path(), &[artifact]).unwrap();

        assert_eq!(changes.len(), 1);
        assert!(!changes[0].dry_run);
        assert!(dir.path().join("src/test.rs").exists());
    }
}
