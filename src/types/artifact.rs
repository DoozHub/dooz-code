//! Generated Artifact Types
//!
//! Represents files and resources generated during execution.

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use super::identifiers::ArtifactId;

/// A generated artifact (file, config, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Unique artifact identifier
    pub id: ArtifactId,
    
    /// Relative path from repo root
    pub path: String,
    
    /// Type of artifact
    pub artifact_type: ArtifactType,
    
    /// Generated content
    pub content: String,
    
    /// Content hash (SHA-256)
    pub content_hash: String,
    
    /// Line count
    pub line_count: u32,
    
    /// Byte size
    pub size: usize,
    
    /// Generation metadata
    pub metadata: ArtifactMetadata,
}

impl Artifact {
    /// Create a new artifact
    pub fn new(path: impl Into<String>, content: impl Into<String>, artifact_type: ArtifactType) -> Self {
        let path = path.into();
        let content = content.into();
        
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        
        let line_count = content.lines().count() as u32;
        let size = content.len();
        
        let id = ArtifactId::new(&path, &hash);
        
        Self {
            id,
            path,
            artifact_type,
            content,
            content_hash: hash,
            line_count,
            size,
            metadata: ArtifactMetadata::default(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: ArtifactMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Check if artifact is source code
    pub fn is_source(&self) -> bool {
        matches!(self.artifact_type, ArtifactType::Source)
    }

    /// Check if artifact is a test
    pub fn is_test(&self) -> bool {
        matches!(self.artifact_type, ArtifactType::Test)
    }

    /// Check if artifact is new (not modifying existing)
    pub fn is_new(&self) -> bool {
        self.metadata.is_new
    }

    /// Get file extension
    pub fn extension(&self) -> Option<&str> {
        self.path.rsplit('.').next()
    }
}

/// Type of artifact
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactType {
    /// Source code file
    Source,
    
    /// Test file
    Test,
    
    /// Configuration file
    Config,
    
    /// Documentation
    Documentation,
    
    /// Migration script
    Migration,
    
    /// File modification (partial change)
    Modification,
    
    /// File deletion marker
    Deletion,
    
    /// Other
    Other,
}

impl ArtifactType {
    /// Infer type from file path
    pub fn from_path(path: &str) -> Self {
        let path_lower = path.to_lowercase();
        
        if path_lower.contains("test") || path_lower.contains("spec") {
            return Self::Test;
        }
        
        if path_lower.ends_with(".md") || path_lower.contains("doc") {
            return Self::Documentation;
        }
        
        if path_lower.contains("config") 
            || path_lower.ends_with(".json")
            || path_lower.ends_with(".yaml")
            || path_lower.ends_with(".yml")
            || path_lower.ends_with(".toml")
        {
            return Self::Config;
        }
        
        if path_lower.contains("migration") {
            return Self::Migration;
        }
        
        if path_lower.ends_with(".rs")
            || path_lower.ends_with(".ts")
            || path_lower.ends_with(".js")
            || path_lower.ends_with(".py")
            || path_lower.ends_with(".go")
            || path_lower.ends_with(".java")
            || path_lower.ends_with(".php")
        {
            return Self::Source;
        }
        
        Self::Other
    }
}

/// Metadata about artifact generation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    /// Whether this is a new file
    pub is_new: bool,
    
    /// Step ID that generated this artifact
    pub generated_by: Option<String>,
    
    /// Generator name (e.g., LLM provider)
    pub generator: String,
    
    /// Generation confidence score (0.0 - 1.0)
    pub confidence: f32,
    
    /// Whether this is a partial change
    pub is_partial: bool,
    
    /// Operation type (add, replace, remove, delete)
    pub operation: String,
    
    /// Whether artifact was corrected
    pub corrected: bool,
    
    /// Reason for generation
    pub reason: Option<String>,
    
    /// Original content hash (if modifying)
    pub original_hash: Option<String>,
    
    /// Patterns followed
    pub patterns_followed: Vec<String>,
}

impl ArtifactMetadata {
    /// Create metadata for new file
    pub fn new_file() -> Self {
        Self {
            is_new: true,
            ..Default::default()
        }
    }

    /// Create metadata for modification
    pub fn modification(original_hash: impl Into<String>) -> Self {
        Self {
            is_new: false,
            original_hash: Some(original_hash.into()),
            ..Default::default()
        }
    }

    /// Set generator
    pub fn with_generator(mut self, step_id: impl Into<String>) -> Self {
        self.generated_by = Some(step_id.into());
        self
    }

    /// Set reason
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Add pattern followed
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.patterns_followed.push(pattern.into());
        self
    }
}

/// Collection of artifacts from execution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArtifactCollection {
    /// All artifacts
    pub artifacts: Vec<Artifact>,
}

impl ArtifactCollection {
    /// Create new collection
    pub fn new() -> Self {
        Self {
            artifacts: Vec::new(),
        }
    }

    /// Add artifact
    pub fn add(&mut self, artifact: Artifact) {
        self.artifacts.push(artifact);
    }

    /// Get artifact count
    pub fn len(&self) -> usize {
        self.artifacts.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.artifacts.is_empty()
    }

    /// Get all source artifacts
    pub fn sources(&self) -> Vec<&Artifact> {
        self.artifacts.iter().filter(|a| a.is_source()).collect()
    }

    /// Get all test artifacts
    pub fn tests(&self) -> Vec<&Artifact> {
        self.artifacts.iter().filter(|a| a.is_test()).collect()
    }

    /// Get artifact by path
    pub fn get_by_path(&self, path: &str) -> Option<&Artifact> {
        self.artifacts.iter().find(|a| a.path == path)
    }

    /// Total lines of code
    pub fn total_lines(&self) -> u32 {
        self.artifacts.iter().map(|a| a.line_count).sum()
    }

    /// Total bytes
    pub fn total_bytes(&self) -> usize {
        self.artifacts.iter().map(|a| a.size).sum()
    }
}

impl IntoIterator for ArtifactCollection {
    type Item = Artifact;
    type IntoIter = std::vec::IntoIter<Artifact>;

    fn into_iter(self) -> Self::IntoIter {
        self.artifacts.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_creation() {
        let artifact = Artifact::new(
            "src/main.rs",
            "fn main() { println!(\"Hello\"); }",
            ArtifactType::Source,
        );

        assert!(artifact.is_source());
        assert!(!artifact.is_test());
        assert!(!artifact.content_hash.is_empty());
        assert_eq!(artifact.line_count, 1);
    }

    #[test]
    fn artifact_type_inference() {
        assert_eq!(ArtifactType::from_path("src/main.rs"), ArtifactType::Source);
        assert_eq!(ArtifactType::from_path("tests/test_main.rs"), ArtifactType::Test);
        assert_eq!(ArtifactType::from_path("config.json"), ArtifactType::Config);
        assert_eq!(ArtifactType::from_path("README.md"), ArtifactType::Documentation);
    }

    #[test]
    fn artifact_collection() {
        let mut collection = ArtifactCollection::new();
        collection.add(Artifact::new("src/main.rs", "code", ArtifactType::Source));
        collection.add(Artifact::new("tests/test.rs", "test", ArtifactType::Test));

        assert_eq!(collection.len(), 2);
        assert_eq!(collection.sources().len(), 1);
        assert_eq!(collection.tests().len(), 1);
    }
}
