//! Core Identifier Types
//!
//! Unique identifiers for all entities in the execution engine.
//! All identifiers are content-addressable where possible.

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

/// Unique identifier for a work package
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PackageId(String);

impl PackageId {
    /// Create a new package ID from a string
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Generate a package ID from content hash
    pub fn from_content(content: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        Self(format!("pkg-{}", hex::encode(&result[..8])))
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PackageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for an implementation step
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StepId(String);

impl StepId {
    /// Create a new step ID
    pub fn new(package_id: &PackageId, step_number: u32) -> Self {
        Self(format!("{}-step-{:03}", package_id.as_str(), step_number))
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for StepId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for a generated artifact
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtifactId(String);

impl ArtifactId {
    /// Create artifact ID from file path and content hash
    pub fn new(path: &str, content_hash: &str) -> Self {
        Self(format!("art-{}:{}", 
            path.replace('/', "_").replace('.', "_"),
            &content_hash[..8]
        ))
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for ArtifactId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Content-addressable hash for repository context
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContextHash(String);

impl ContextHash {
    /// Compute hash from repository state
    pub fn compute(file_hashes: &[String]) -> Self {
        let mut hasher = Sha256::new();
        for hash in file_hashes {
            hasher.update(hash.as_bytes());
        }
        let result = hasher.finalize();
        Self(hex::encode(result))
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get shortened hash for display
    pub fn short(&self) -> &str {
        &self.0[..16]
    }
}

impl std::fmt::Display for ContextHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.short())
    }
}

/// Approval reference linking to governance decision
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRef {
    /// Veto decision ID that approved this work
    pub veto_id: String,
    /// Timestamp of approval (ISO 8601)
    pub approved_at: String,
    /// Constraints imposed by veto
    pub constraints: Vec<String>,
}

impl ApprovalRef {
    /// Create a new approval reference
    pub fn new(veto_id: impl Into<String>, approved_at: impl Into<String>) -> Self {
        Self {
            veto_id: veto_id.into(),
            approved_at: approved_at.into(),
            constraints: Vec::new(),
        }
    }

    /// Add a constraint from veto
    pub fn with_constraint(mut self, constraint: impl Into<String>) -> Self {
        self.constraints.push(constraint.into());
        self
    }
}

// Hex encoding helper (to avoid external dependency)
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_id_from_content() {
        let id1 = PackageId::from_content("test content");
        let id2 = PackageId::from_content("test content");
        let id3 = PackageId::from_content("different content");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn step_id_format() {
        let pkg_id = PackageId::new("AUTH-001");
        let step_id = StepId::new(&pkg_id, 1);
        
        assert_eq!(step_id.as_str(), "AUTH-001-step-001");
    }

    #[test]
    fn context_hash_deterministic() {
        let hashes = vec!["hash1".to_string(), "hash2".to_string()];
        let ctx1 = ContextHash::compute(&hashes);
        let ctx2 = ContextHash::compute(&hashes);

        assert_eq!(ctx1, ctx2);
    }
}
