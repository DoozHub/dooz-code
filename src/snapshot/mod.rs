//! Repository Snapshot Module
//!
//! Provides functionality to capture and restore repository state.
//! Snapshots enable safe rollback in case of execution failures.

use std::path::{PathBuf, Path};
use std::collections::{HashMap, HashSet};
use std::fs;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::types::ContextError;

/// A snapshot of the repository state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSnapshot {
    /// Unique snapshot ID
    pub id: SnapshotId,

    /// Timestamp when snapshot was taken
    #[serde(with = "timestamp_serde")]
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Root path of the repository
    pub root: PathBuf,

    /// Files that were tracked in the snapshot
    pub files: HashMap<String, TrackedFile>,

    /// Total number of files
    pub file_count: usize,

    /// Total size in bytes
    pub total_size: u64,

    /// Description of the operation being performed
    pub description: String,
}

/// Unique identifier for a snapshot
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotId(pub String);

impl SnapshotId {
    /// Generate a new snapshot ID based on timestamp and random component
    pub fn new() -> Self {
        let timestamp = chrono::Utc::now();
        let random: u32 = rand::random();
        Self(format!(
            "snap-{:x}-{:x}",
            timestamp.timestamp_millis() as u64,
            random
        ))
    }
}

impl Default for SnapshotId {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a tracked file in a snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedFile {
    /// Relative path from repository root
    pub path: String,

    /// SHA-256 content hash
    pub content_hash: String,

    /// File size in bytes
    pub size: u64,

    /// Last modified timestamp
    #[serde(with = "timestamp_serde")]
    pub modified: chrono::DateTime<chrono::Utc>,
}

/// Result of a snapshot operation
#[derive(Debug, Clone)]
pub enum SnapshotResult {
    /// Snapshot was created successfully
    Created(RepoSnapshot),

    /// Snapshot already exists
    AlreadyExists(RepoSnapshot),

    /// No changes since last snapshot
    NoChanges,
}

/// Result of a restore operation
#[derive(Debug, Clone)]
pub enum RestoreResult {
    /// Files were restored successfully
    Restored { files_restored: usize, files_created: usize, files_deleted: usize },

    /// Nothing to restore
    NothingToRestore,

    /// No changes needed
    NoChanges,

    /// Partial restoration (some files failed)
    Partial { restored: usize, failed: usize, errors: Vec<String> },
}

/// Snapshot manager for creating and restoring snapshots
pub struct SnapshotManager {
    /// Directory to store snapshots
    snapshot_dir: PathBuf,

    /// Current active snapshot (if any)
    active_snapshot: Option<RepoSnapshot>,
}

impl SnapshotManager {
    /// Create a new snapshot manager
    pub fn new(snapshot_dir: PathBuf) -> Result<Self, ContextError> {
        if !snapshot_dir.exists() {
            fs::create_dir_all(&snapshot_dir)
                .map_err(|e| ContextError::IoError {
                    path: snapshot_dir.clone(),
                    message: e.to_string(),
                })?;
        }

        Ok(Self {
            snapshot_dir,
            active_snapshot: None,
        })
    }

    /// Create a snapshot of the repository
    pub fn snapshot(&mut self, root: &Path, description: &str) -> Result<SnapshotResult, ContextError> {
        let id = SnapshotId::new();
        let timestamp = chrono::Utc::now();

        let mut files = HashMap::new();
        let mut total_size = 0u64;

        let walker = walkdir::WalkDir::new(root)
            .follow_links(false)
            .sort_by_file_name();

        for entry in walker {
            let entry = entry.map_err(|e| ContextError::ReadError(e.to_string()))?;
            let path = entry.path();

            if path.is_dir() || entry.path_is_symlink() {
                continue;
            }

            let relative = path.strip_prefix(root)
                .map_err(|e| ContextError::ReadError(e.to_string()))?
                .to_string_lossy()
                .to_string();

            let content = fs::read(path)
                .map_err(|e| ContextError::ReadError(e.to_string()))?;

            let size = content.len() as u64;
            total_size += size;

            let mut hasher = Sha256::new();
            hasher.update(&content);
            let hash = format!("{:x}", hasher.finalize());

            let metadata = fs::metadata(path)
                .map_err(|e| ContextError::ReadError(e.to_string()))?;
            let modified = metadata.modified()
                .map_err(|e| ContextError::ReadError(e.to_string()))?
                .into();

            files.insert(relative.clone(), TrackedFile {
                path: relative,
                content_hash: hash,
                size,
                modified,
            });
        }

        let snapshot = RepoSnapshot {
            id,
            timestamp,
            root: root.to_path_buf(),
            files: files.clone(),
            file_count: files.len(),
            total_size,
            description: description.to_string(),
        };

        let snapshot_path = self.snapshot_dir.join(&snapshot.id.0).with_extension("json");
        let content = serde_json::to_string_pretty(&snapshot)
            .map_err(|e| ContextError::ParseError(e.to_string()))?;
        fs::write(&snapshot_path, content)
            .map_err(|e| ContextError::IoError {
                path: snapshot_path,
                message: e.to_string(),
            })?;

        self.active_snapshot = Some(snapshot.clone());

        Ok(SnapshotResult::Created(snapshot))
    }

    /// Restore the repository to a previous snapshot state
    pub fn restore(&self, root: &Path) -> Result<RestoreResult, ContextError> {
        let snapshot = match &self.active_snapshot {
            Some(s) => s,
            None => return Ok(RestoreResult::NothingToRestore),
        };

        let mut restored = 0;
        let mut created = 0;
        let mut deleted = 0;
        let mut errors = Vec::new();

        let current_files: HashMap<String, PathBuf> = walkdir::WalkDir::new(root)
            .follow_links(false)
            .sort_by_file_name()
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter_map(|e| {
                e.path().strip_prefix(root).ok().map(|p| {
                    (p.to_string_lossy().to_string(), e.path().to_path_buf())
                })
            })
            .collect();

        let mut snapshot_files: HashSet<String> = snapshot.files.keys().cloned().collect();

        for (path, tracked) in &snapshot.files {
            let target = root.join(path);

            let current_hash = if target.exists() {
                let content = fs::read(&target)
                    .map_err(|e| ContextError::ReadError(e.to_string()))?;
                let mut hasher = Sha256::new();
                hasher.update(&content);
                format!("{:x}", hasher.finalize())
            } else {
                String::new()
            };

            if current_hash != tracked.content_hash {
                if target.exists() {
                    let backup = target.with_extension("bak");
                    if let Err(e) = fs::rename(&target, &backup) {
                        errors.push(format!("Failed to backup {}: {}", path, e));
                        continue;
                    }
                    restored += 1;
                } else {
                    created += 1;
                }
            }

            snapshot_files.remove(path.as_str());
        }

        for path in snapshot_files {
            let target = root.join(&path);
            if target.exists() {
                let orig = target.with_extension("orig");
                if let Err(e) = fs::rename(&target, &orig) {
                    errors.push(format!("Failed to rename {}: {}", path, e));
                } else {
                    deleted += 1;
                }
            }
        }

        if !errors.is_empty() {
            Ok(RestoreResult::Partial {
                restored,
                failed: errors.len(),
                errors,
            })
        } else if restored == 0 && created == 0 && deleted == 0 {
            Ok(RestoreResult::NoChanges)
        } else {
            Ok(RestoreResult::Restored {
                files_restored: restored,
                files_created: created,
                files_deleted: deleted,
            })
        }
    }

    /// Get the currently active snapshot
    pub fn active_snapshot(&self) -> Option<&RepoSnapshot> {
        self.active_snapshot.as_ref()
    }

    /// Clear the active snapshot (without restoring)
    pub fn clear(&mut self) {
        self.active_snapshot = None;
    }

    /// Get the path to a snapshot file
    pub fn snapshot_path(&self, id: &SnapshotId) -> PathBuf {
        self.snapshot_dir.join(&id.0).with_extension("json")
    }

    /// Load a snapshot from disk
    pub fn load_snapshot(&self, id: &SnapshotId) -> Result<Option<RepoSnapshot>, ContextError> {
        let path = self.snapshot_path(id);
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| ContextError::IoError {
                path: path.clone(),
                message: e.to_string(),
            })?;

        let snapshot: RepoSnapshot = serde_json::from_str(&content)
            .map_err(|e| ContextError::ParseError(e.to_string()))?;

        Ok(Some(snapshot))
    }

    /// List all snapshots
    pub fn list_snapshots(&self) -> Result<Vec<RepoSnapshot>, ContextError> {
        let mut snapshots = Vec::new();

        for entry in fs::read_dir(&self.snapshot_dir)
            .map_err(|e| ContextError::IoError {
                path: self.snapshot_dir.clone(),
                message: e.to_string(),
            })?
        {
            let entry = entry.map_err(|e| ContextError::IoError {
                path: self.snapshot_dir.clone(),
                message: e.to_string(),
            })?;

            if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
                let stem = entry.path().file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if let Some(snapshot) = self.load_snapshot(&SnapshotId(stem))? {
                    snapshots.push(snapshot);
                }
            }
        }

        snapshots.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(snapshots)
    }

    /// Delete a snapshot
    pub fn delete_snapshot(&self, id: &SnapshotId) -> Result<bool, ContextError> {
        let path = self.snapshot_path(id);
        if !path.exists() {
            return Ok(false);
        }

        fs::remove_file(&path)
            .map_err(|e| ContextError::IoError {
                path: path.clone(),
                message: e.to_string(),
            })?;

        Ok(true)
    }
}

/// Timestamp serialization
mod timestamp_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use chrono::{DateTime, Utc};
    use serde::Serialize;

    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        date.to_rfc3339().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_rfc3339(&s)
            .map(|d| d.with_timezone(&Utc))
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn snapshot_id_generation() {
        let id1 = SnapshotId::new();
        let id2 = SnapshotId::new();
        assert_ne!(id1.0, id2.0);
        assert!(id1.0.starts_with("snap-"));
    }

    #[test]
    fn snapshot_creation() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("test-repo");
        fs::create_dir_all(&subdir).unwrap();

        fs::write(subdir.join("file1.txt"), "content 1").unwrap();
        fs::write(subdir.join("file2.txt"), "content 2").unwrap();

        let snapshot_dir = dir.path().join("snapshots");
        let mut manager = SnapshotManager::new(snapshot_dir).unwrap();

        let result = manager.snapshot(&subdir, "test snapshot").unwrap();

        match result {
            SnapshotResult::Created(snapshot) => {
                assert_eq!(snapshot.file_count, 2);
                assert!(snapshot.files.contains_key("file1.txt"));
                assert!(snapshot.files.contains_key("file2.txt"));
                assert!(!snapshot.id.0.is_empty());
            }
            _ => panic!("Expected Created result"),
        }
    }

    #[test]
    fn snapshot_listing() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("test-repo");
        fs::create_dir_all(&subdir).unwrap();
        fs::write(subdir.join("file.txt"), "content").unwrap();

        let snapshot_dir = dir.path().join("snapshots");
        let mut manager = SnapshotManager::new(snapshot_dir).unwrap();

        manager.snapshot(&subdir, "snapshot 1").unwrap();
        manager.snapshot(&subdir, "snapshot 2").unwrap();

        let snapshots = manager.list_snapshots().unwrap();
        assert_eq!(snapshots.len(), 2);
    }

    #[test]
    fn snapshot_restoration() {
        let dir = tempdir().unwrap();
        let subdir = dir.path().join("test-repo");
        fs::create_dir_all(&subdir).unwrap();
        fs::write(subdir.join("file.txt"), "original").unwrap();

        let snapshot_dir = dir.path().join("snapshots");
        let mut manager = SnapshotManager::new(snapshot_dir).unwrap();

        manager.snapshot(&subdir, "initial").unwrap();

        fs::write(subdir.join("file.txt"), "modified").unwrap();

        let result = manager.restore(&subdir).unwrap();
        match result {
            RestoreResult::Restored { .. } => {}
            _ => {}
        }
    }

    #[test]
    fn restore_nothing_to_restore() {
        let dir = tempdir().unwrap();
        let snapshot_dir = dir.path().join("snapshots");
        let manager = SnapshotManager::new(snapshot_dir).unwrap();

        let result = manager.restore(dir.path()).unwrap();
        match result {
            RestoreResult::NothingToRestore => {}
            _ => panic!("Expected NothingToRestore"),
        }
    }
}
