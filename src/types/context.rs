//! Repository Context Types
//!
//! Represents the analyzed state of a repository.
//! Context is extracted before execution and used to inform implementation decisions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use super::identifiers::ContextHash;

/// Complete repository context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoContext {
    /// Root path of repository
    pub root: PathBuf,
    
    /// Context hash for determinism verification
    pub hash: ContextHash,
    
    /// Total file count
    pub file_count: usize,
    
    /// File index by path
    pub files: HashMap<String, FileInfo>,
    
    /// Detected patterns
    pub patterns: Vec<Pattern>,
    
    /// Dependency graph
    pub dependencies: Vec<Dependency>,
    
    /// Detected conventions
    pub conventions: Conventions,
}

impl RepoContext {
    /// Create context from repository path
    pub fn from_path(path: &Path) -> Result<Self, ContextError> {
        if !path.exists() {
            return Err(ContextError::PathNotFound(path.to_path_buf()));
        }

        if !path.is_dir() {
            return Err(ContextError::NotADirectory(path.to_path_buf()));
        }

        // For now, create a minimal context
        // Full implementation will use the analyzer component
        Ok(Self {
            root: path.to_path_buf(),
            hash: ContextHash::compute(&[]),
            file_count: 0,
            files: HashMap::new(),
            patterns: Vec::new(),
            dependencies: Vec::new(),
            conventions: Conventions::default(),
        })
    }

    /// Get file info by relative path
    pub fn get_file(&self, path: &str) -> Option<&FileInfo> {
        self.files.get(path)
    }

    /// Check if a pattern exists
    pub fn has_pattern(&self, pattern_type: PatternType) -> bool {
        self.patterns.iter().any(|p| p.pattern_type == pattern_type)
    }

    /// Get all files matching a glob pattern
    pub fn files_matching(&self, _glob: &str) -> Vec<&FileInfo> {
        // TODO: Implement glob matching
        Vec::new()
    }
}

/// Information about a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    /// Relative path from repo root
    pub path: String,
    
    /// File type
    pub file_type: FileType,
    
    /// Content hash (SHA-256)
    pub content_hash: String,
    
    /// Size in bytes
    pub size: u64,
    
    /// Line count (for text files)
    pub line_count: Option<u32>,
    
    /// Detected language
    pub language: Option<Language>,
    
    /// Imports/dependencies within this file
    pub imports: Vec<String>,
    
    /// Exports from this file
    pub exports: Vec<String>,
}

impl FileInfo {
    /// Create new file info
    pub fn new(path: impl Into<String>, file_type: FileType) -> Self {
        Self {
            path: path.into(),
            file_type,
            content_hash: String::new(),
            size: 0,
            line_count: None,
            language: None,
            imports: Vec::new(),
            exports: Vec::new(),
        }
    }

    /// Check if file is source code
    pub fn is_source(&self) -> bool {
        matches!(self.file_type, FileType::Source)
    }

    /// Check if file is test
    pub fn is_test(&self) -> bool {
        matches!(self.file_type, FileType::Test)
    }
}

/// Type of file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    Source,
    Test,
    Config,
    Documentation,
    Asset,
    Build,
    Other,
}

/// Programming language
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Java,
    PHP,
    Ruby,
    C,
    Cpp,
    Other,
}

/// Detected code pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// Pattern type
    pub pattern_type: PatternType,
    
    /// Description
    pub description: String,
    
    /// Example occurrences
    pub examples: Vec<String>,
    
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
}

impl Pattern {
    /// Create a new pattern
    pub fn new(pattern_type: PatternType, description: impl Into<String>) -> Self {
        Self {
            pattern_type,
            description: description.into(),
            examples: Vec::new(),
            confidence: 1.0,
        }
    }

    /// Add example
    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.examples.push(example.into());
        self
    }

    /// Set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }
}

/// Type of detected pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PatternType {
    // Structural patterns
    ModuleStructure,
    RouteHandler,
    Controller,
    Service,
    Repository,
    Model,
    Middleware,
    
    // Code style patterns
    NamingConvention,
    ErrorHandling,
    Logging,
    Testing,
    
    // Architectural patterns
    LayeredArchitecture,
    CleanArchitecture,
    Microservice,
    Monolith,
}

/// Dependency between files/modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Source file/module
    pub from: String,
    
    /// Target file/module
    pub to: String,
    
    /// Type of dependency
    pub dependency_type: DependencyType,
}

impl Dependency {
    /// Create a new dependency
    pub fn new(from: impl Into<String>, to: impl Into<String>, dep_type: DependencyType) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            dependency_type: dep_type,
        }
    }
}

/// Type of dependency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    Import,
    Inheritance,
    Implementation,
    Usage,
    Test,
}

/// Detected coding conventions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Conventions {
    /// Indentation style
    pub indent: IndentStyle,
    
    /// Quote style for strings
    pub quotes: QuoteStyle,
    
    /// Semicolon usage
    pub semicolons: SemicolonStyle,
    
    /// Naming conventions
    pub naming: NamingConventions,
    
    /// File organization
    pub file_organization: FileOrganization,
}

/// Indentation style
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndentStyle {
    #[default]
    Spaces2,
    Spaces4,
    Tabs,
}

/// String quote style
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuoteStyle {
    #[default]
    Single,
    Double,
}

/// Semicolon style
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemicolonStyle {
    #[default]
    Always,
    Never,
    AsNeeded,
}

/// Naming conventions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NamingConventions {
    /// Variable naming (camelCase, snake_case, etc.)
    pub variables: NamingStyle,
    
    /// Function naming
    pub functions: NamingStyle,
    
    /// Class/type naming
    pub types: NamingStyle,
    
    /// File naming
    pub files: NamingStyle,
}

/// Naming style
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NamingStyle {
    #[default]
    CamelCase,
    PascalCase,
    SnakeCase,
    KebabCase,
    ScreamingSnakeCase,
}

/// File organization pattern
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileOrganization {
    /// Source directory
    pub src_dir: Option<String>,
    
    /// Test directory
    pub test_dir: Option<String>,
    
    /// Config directory
    pub config_dir: Option<String>,
    
    /// Docs directory
    pub docs_dir: Option<String>,
}

/// Context extraction errors
#[derive(Debug, Clone)]
pub enum ContextError {
    PathNotFound(PathBuf),
    NotADirectory(PathBuf),
    ReadError(String),
    ParseError(String),
    ValidationError(String),
}

impl std::fmt::Display for ContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PathNotFound(p) => write!(f, "Path not found: {}", p.display()),
            Self::NotADirectory(p) => write!(f, "Not a directory: {}", p.display()),
            Self::ReadError(e) => write!(f, "Read error: {}", e),
            Self::ParseError(e) => write!(f, "Parse error: {}", e),
            Self::ValidationError(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for ContextError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn context_from_valid_path() {
        let dir = tempdir().unwrap();
        let ctx = RepoContext::from_path(dir.path());
        assert!(ctx.is_ok());
    }

    #[test]
    fn context_from_invalid_path() {
        let ctx = RepoContext::from_path(Path::new("/nonexistent/path"));
        assert!(matches!(ctx.unwrap_err(), ContextError::PathNotFound(_)));
    }

    #[test]
    fn file_info_types() {
        let source = FileInfo::new("src/main.rs", FileType::Source);
        let test = FileInfo::new("tests/test.rs", FileType::Test);

        assert!(source.is_source());
        assert!(!source.is_test());
        assert!(test.is_test());
    }
}
