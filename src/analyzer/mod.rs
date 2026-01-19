//! Repository Analyzer
//!
//! Extracts context from a repository for informed implementation decisions.
//! Analysis is deterministic and produces content-addressable results.

mod files;
mod patterns;
mod dependencies;

pub use files::*;
pub use patterns::*;
pub use dependencies::*;

use crate::types::{RepoContext, ContextError, ContextHash};
use std::path::Path;

/// Repository analyzer component
pub struct RepoAnalyzer {
    /// Configuration for analysis
    config: AnalyzerConfig,
}

impl RepoAnalyzer {
    /// Create new analyzer with default config
    pub fn new() -> Self {
        Self {
            config: AnalyzerConfig::default(),
        }
    }

    /// Create analyzer with custom config
    pub fn with_config(config: AnalyzerConfig) -> Self {
        Self { config }
    }

    /// Analyze repository and enrich context
    pub fn analyze(&self, context: &RepoContext) -> Result<AnalyzedContext, ContextError> {
        // Scan files
        let scanner = FileScanner::new()
            .with_extensions(self.config.include_extensions.clone())
            .with_excludes(self.config.exclude_dirs.clone())
            .with_max_files(self.config.max_files);
        
        let file_analysis = scanner.scan(&context.root)?;

        // Detect patterns
        let pattern_analysis = if self.config.detect_patterns {
            let detector = PatternDetector::new();
            detector.detect(&file_analysis)
        } else {
            PatternAnalysis::default()
        };

        // Analyze dependencies
        let dependency_analysis = if self.config.analyze_dependencies {
            let analyzer = DependencyAnalyzer::new();
            analyzer.analyze(&file_analysis)
        } else {
            DependencyAnalysis::default()
        };

        // Compute context hash from file hashes
        let file_hashes: Vec<String> = file_analysis.files.values()
            .map(|f| f.content_hash.clone())
            .filter(|h| !h.is_empty())
            .collect();
        let hash = ContextHash::compute(&file_hashes);

        // Update base context with analysis results
        let mut enriched_context = context.clone();
        enriched_context.hash = hash;
        enriched_context.file_count = file_analysis.total_files;

        Ok(AnalyzedContext {
            base: enriched_context,
            file_analysis,
            pattern_analysis,
            dependency_analysis,
        })
    }

    /// Analyze from path directly
    pub fn analyze_path(&self, path: &Path) -> Result<AnalyzedContext, ContextError> {
        let context = RepoContext::from_path(path)?;
        self.analyze(&context)
    }
}

impl Default for RepoAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Analyzer configuration
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// Maximum files to analyze
    pub max_files: usize,
    
    /// File extensions to include
    pub include_extensions: Vec<String>,
    
    /// Directories to exclude
    pub exclude_dirs: Vec<String>,
    
    /// Enable dependency analysis
    pub analyze_dependencies: bool,
    
    /// Enable pattern detection
    pub detect_patterns: bool,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            max_files: 10000,
            include_extensions: vec![
                "rs".into(), "ts".into(), "tsx".into(), "js".into(), "jsx".into(),
                "py".into(), "go".into(), "java".into(), "php".into(), "rb".into(),
                "json".into(), "yaml".into(), "yml".into(), "toml".into(),
                "md".into(), "sh".into(),
            ],
            exclude_dirs: vec![
                "node_modules".into(), "target".into(), ".git".into(),
                "vendor".into(), "dist".into(), "build".into(),
                "__pycache__".into(), ".next".into(), "coverage".into(),
            ],
            analyze_dependencies: true,
            detect_patterns: true,
        }
    }
}

/// Enriched repository context
#[derive(Debug, Clone)]
pub struct AnalyzedContext {
    /// Base context
    pub base: RepoContext,
    
    /// File analysis results
    pub file_analysis: FileAnalysis,
    
    /// Pattern analysis results
    pub pattern_analysis: PatternAnalysis,
    
    /// Dependency analysis results
    pub dependency_analysis: DependencyAnalysis,
}

impl AnalyzedContext {
    /// Get base context
    pub fn context(&self) -> &RepoContext {
        &self.base
    }

    /// Check if pattern exists
    pub fn has_pattern(&self, name: &str) -> bool {
        self.pattern_analysis.has_pattern(name)
    }

    /// Get file count
    pub fn file_count(&self) -> usize {
        self.file_analysis.total_files
    }

    /// Get total lines of code
    pub fn total_lines(&self) -> u64 {
        self.file_analysis.total_lines
    }

    /// Get external dependency count
    pub fn external_dep_count(&self) -> usize {
        self.dependency_analysis.external_count()
    }

    /// Check for circular dependencies
    pub fn has_cycles(&self) -> bool {
        self.dependency_analysis.has_cycles()
    }

    /// Get detected patterns
    pub fn patterns(&self) -> &[DetectedPattern] {
        &self.pattern_analysis.detected
    }

    /// Get file info
    pub fn get_file(&self, path: &str) -> Option<&crate::types::FileInfo> {
        self.file_analysis.get(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn analyzer_creation() {
        let analyzer = RepoAnalyzer::new();
        assert_eq!(analyzer.config.max_files, 10000);
    }

    #[test]
    fn analyze_empty_repo() {
        let dir = tempdir().unwrap();
        let context = RepoContext::from_path(dir.path()).unwrap();
        
        let analyzer = RepoAnalyzer::new();
        let result = analyzer.analyze(&context);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().file_count(), 0);
    }

    #[test]
    fn analyze_repo_with_files() {
        let dir = tempdir().unwrap();
        
        // Create test structure
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("main.rs"), "use crate::lib;\nfn main() {}").unwrap();
        fs::write(src.join("lib.rs"), "pub mod utils;\npub fn hello() {}").unwrap();
        
        let tests = dir.path().join("tests");
        fs::create_dir_all(&tests).unwrap();
        fs::write(tests.join("test_main.rs"), "#[test]\nfn test() {}").unwrap();
        
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
        
        let analyzer = RepoAnalyzer::new();
        let result = analyzer.analyze_path(dir.path()).unwrap();
        
        assert_eq!(result.file_count(), 4);
        assert!(result.total_lines() > 0);
        assert_eq!(result.file_analysis.by_type.source, 2);
        assert_eq!(result.file_analysis.by_type.test, 1);
        assert_eq!(result.file_analysis.by_type.config, 1);
    }

    #[test]
    fn analyze_with_patterns() {
        let dir = tempdir().unwrap();
        
        // Create MVC-like structure
        let models = dir.path().join("app/models");
        let views = dir.path().join("app/views");
        let controllers = dir.path().join("app/controllers");
        
        fs::create_dir_all(&models).unwrap();
        fs::create_dir_all(&views).unwrap();
        fs::create_dir_all(&controllers).unwrap();
        
        fs::write(models.join("User.php"), "<?php class User {}").unwrap();
        fs::write(views.join("user.blade.php"), "<h1>User</h1>").unwrap();
        fs::write(controllers.join("UserController.php"), "<?php class UserController {}").unwrap();
        
        let analyzer = RepoAnalyzer::new();
        let result = analyzer.analyze_path(dir.path()).unwrap();
        
        // Should detect MVC and Controller patterns
        assert!(result.has_pattern("MVC") || result.has_pattern("Controller"));
    }
}
