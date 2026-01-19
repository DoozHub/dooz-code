//! Pattern Detection
//!
//! Detects coding patterns, conventions, and structural patterns in the repository.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use crate::types::Language;
use super::files::FileAnalysis;

/// Pattern analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternAnalysis {
    /// Detected patterns
    pub detected: Vec<DetectedPattern>,
    
    /// Detected conventions
    pub conventions: ConventionAnalysis,
    
    /// Confidence in analysis
    pub confidence: f32,
}

impl PatternAnalysis {
    /// Check if a pattern was detected
    pub fn has_pattern(&self, name: &str) -> bool {
        self.detected.iter().any(|p| p.name == name)
    }

    /// Get pattern by name
    pub fn get_pattern(&self, name: &str) -> Option<&DetectedPattern> {
        self.detected.iter().find(|p| p.name == name)
    }
}

/// A detected coding pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    /// Pattern name
    pub name: String,
    
    /// Pattern category
    pub category: PatternCategory,
    
    /// Description
    pub description: String,
    
    /// Example files
    pub examples: Vec<String>,
    
    /// Detection confidence (0.0 - 1.0)
    pub confidence: f32,
}

impl DetectedPattern {
    /// Create new detected pattern
    pub fn new(name: impl Into<String>, category: PatternCategory) -> Self {
        Self {
            name: name.into(),
            category,
            description: String::new(),
            examples: Vec::new(),
            confidence: 1.0,
        }
    }

    /// With description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
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

/// Pattern category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternCategory {
    /// Structural pattern (how code is organized)
    Structural,
    
    /// Naming pattern (how things are named)
    Naming,
    
    /// Error handling pattern
    ErrorHandling,
    
    /// Testing pattern
    Testing,
    
    /// Architecture pattern
    Architecture,
    
    /// Code style pattern
    Style,
}

/// Detected conventions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConventionAnalysis {
    /// Indentation style
    pub indent: IndentStyle,
    
    /// Quote style for strings
    pub quotes: QuoteStyle,
    
    /// Line endings
    pub line_endings: LineEndingStyle,
    
    /// Naming conventions by context
    pub naming: NamingConventions,
    
    /// Trailing commas usage
    pub trailing_commas: TrailingCommaStyle,
}

/// Indentation style
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IndentStyle {
    #[default]
    Spaces2,
    Spaces4,
    Tabs,
    Mixed,
}

/// String quote style
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QuoteStyle {
    #[default]
    Single,
    Double,
    Mixed,
}

/// Line ending style
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineEndingStyle {
    #[default]
    Unix,
    Windows,
    Mixed,
}

/// Trailing comma style
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrailingCommaStyle {
    #[default]
    Always,
    Never,
    Mixed,
}

/// Naming conventions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NamingConventions {
    /// Variable naming
    pub variables: NamingStyle,
    
    /// Function naming
    pub functions: NamingStyle,
    
    /// Type/class naming
    pub types: NamingStyle,
    
    /// File naming
    pub files: NamingStyle,
    
    /// Constant naming
    pub constants: NamingStyle,
}

/// Naming style
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NamingStyle {
    #[default]
    CamelCase,
    PascalCase,
    SnakeCase,
    KebabCase,
    ScreamingSnakeCase,
    Mixed,
}

/// Pattern detector
pub struct PatternDetector {
    /// Minimum confidence threshold
    min_confidence: f32,
}

impl PatternDetector {
    /// Create new detector
    pub fn new() -> Self {
        Self {
            min_confidence: 0.3,
        }
    }

    /// Set minimum confidence threshold
    pub fn with_min_confidence(mut self, confidence: f32) -> Self {
        self.min_confidence = confidence;
        self
    }

    /// Detect patterns in file analysis
    pub fn detect(&self, file_analysis: &FileAnalysis) -> PatternAnalysis {
        let mut patterns = Vec::new();
        let mut total_confidence = 0.0;
        let mut pattern_count = 0;

        // Detect structural patterns
        patterns.extend(self.detect_structural_patterns(file_analysis));
        
        // Detect architecture patterns
        patterns.extend(self.detect_architecture_patterns(file_analysis));
        
        // Detect testing patterns
        patterns.extend(self.detect_testing_patterns(file_analysis));

        // Calculate overall confidence
        for pattern in &patterns {
            total_confidence += pattern.confidence;
            pattern_count += 1;
        }

        let confidence = if pattern_count > 0 {
            total_confidence / pattern_count as f32
        } else {
            0.0
        };

        // Detect conventions
        let conventions = self.detect_conventions(file_analysis);

        PatternAnalysis {
            detected: patterns.into_iter().filter(|p| p.confidence >= self.min_confidence).collect(),
            conventions,
            confidence,
        }
    }

    /// Detect structural patterns (Controllers, Services, Repositories)
    fn detect_structural_patterns(&self, analysis: &FileAnalysis) -> Vec<DetectedPattern> {
        let mut patterns = Vec::new();
        let paths: Vec<&str> = analysis.files.keys().map(|s| s.as_str()).collect();

        // Check for Controller pattern
        let controller_files: Vec<&str> = paths.iter()
            .filter(|p| p.to_lowercase().contains("controller"))
            .copied()
            .collect();
        if !controller_files.is_empty() {
            let mut pattern = DetectedPattern::new("Controller", PatternCategory::Structural)
                .with_description("Controller pattern for request handling")
                .with_confidence(controller_files.len().min(5) as f32 / 5.0);
            for file in controller_files.iter().take(3) {
                pattern = pattern.with_example(*file);
            }
            patterns.push(pattern);
        }

        // Check for Service pattern
        let service_files: Vec<&str> = paths.iter()
            .filter(|p| {
                let lower = p.to_lowercase();
                lower.contains("service") && !lower.contains("serviceaccount")
            })
            .copied()
            .collect();
        if !service_files.is_empty() {
            let mut pattern = DetectedPattern::new("Service Layer", PatternCategory::Structural)
                .with_description("Service layer for business logic")
                .with_confidence(service_files.len().min(5) as f32 / 5.0);
            for file in service_files.iter().take(3) {
                pattern = pattern.with_example(*file);
            }
            patterns.push(pattern);
        }

        // Check for Repository pattern
        let repo_files: Vec<&str> = paths.iter()
            .filter(|p| p.to_lowercase().contains("repository") || p.to_lowercase().contains("repo"))
            .copied()
            .collect();
        if !repo_files.is_empty() {
            let mut pattern = DetectedPattern::new("Repository", PatternCategory::Structural)
                .with_description("Repository pattern for data access")
                .with_confidence(repo_files.len().min(5) as f32 / 5.0);
            for file in repo_files.iter().take(3) {
                pattern = pattern.with_example(*file);
            }
            patterns.push(pattern);
        }

        // Check for Middleware pattern
        let middleware_files: Vec<&str> = paths.iter()
            .filter(|p| p.to_lowercase().contains("middleware"))
            .copied()
            .collect();
        if !middleware_files.is_empty() {
            let mut pattern = DetectedPattern::new("Middleware", PatternCategory::Structural)
                .with_description("Middleware for request/response processing")
                .with_confidence(middleware_files.len().min(3) as f32 / 3.0);
            for file in middleware_files.iter().take(3) {
                pattern = pattern.with_example(*file);
            }
            patterns.push(pattern);
        }

        // Check for Model pattern
        let model_files: Vec<&str> = paths.iter()
            .filter(|p| {
                let lower = p.to_lowercase();
                (lower.contains("model") || lower.contains("/models/") || lower.contains("\\models\\"))
                    && !lower.contains("node_modules")
            })
            .copied()
            .collect();
        if !model_files.is_empty() {
            let mut pattern = DetectedPattern::new("Model", PatternCategory::Structural)
                .with_description("Model pattern for data structures")
                .with_confidence(model_files.len().min(5) as f32 / 5.0);
            for file in model_files.iter().take(3) {
                pattern = pattern.with_example(*file);
            }
            patterns.push(pattern);
        }

        patterns
    }

    /// Detect architecture patterns (MVC, Clean Architecture, etc.)
    fn detect_architecture_patterns(&self, analysis: &FileAnalysis) -> Vec<DetectedPattern> {
        let mut patterns = Vec::new();
        let paths: Vec<&str> = analysis.files.keys().map(|s| s.as_str()).collect();

        // MVC Detection
        let has_models = paths.iter().any(|p| p.contains("/models/") || p.contains("\\models\\"));
        let has_views = paths.iter().any(|p| p.contains("/views/") || p.contains("\\views\\") || p.contains("/templates/"));
        let has_controllers = paths.iter().any(|p| p.contains("controller"));
        
        if has_models && has_views && has_controllers {
            patterns.push(
                DetectedPattern::new("MVC", PatternCategory::Architecture)
                    .with_description("Model-View-Controller architecture")
                    .with_confidence(0.9)
            );
        }

        // Clean Architecture Detection
        let has_domain = paths.iter().any(|p| p.contains("/domain/") || p.contains("\\domain\\"));
        let has_usecase = paths.iter().any(|p| p.contains("usecase") || p.contains("use_case") || p.contains("use-case"));
        let has_infra = paths.iter().any(|p| p.contains("/infrastructure/") || p.contains("/infra/"));
        
        if has_domain && has_usecase && has_infra {
            patterns.push(
                DetectedPattern::new("Clean Architecture", PatternCategory::Architecture)
                    .with_description("Clean/Hexagonal architecture with domain separation")
                    .with_confidence(0.85)
            );
        }

        // Layered Architecture Detection
        let has_api = paths.iter().any(|p| p.contains("/api/") || p.contains("\\api\\"));
        let has_business = paths.iter().any(|p| p.contains("/business/") || p.contains("/logic/") || p.contains("/services/"));
        let has_data = paths.iter().any(|p| p.contains("/data/") || p.contains("/repositories/") || p.contains("/dal/"));
        
        if has_api && has_business && has_data {
            patterns.push(
                DetectedPattern::new("Layered Architecture", PatternCategory::Architecture)
                    .with_description("Traditional layered architecture (API/Business/Data)")
                    .with_confidence(0.8)
            );
        }

        // Component-Based Architecture (React/Vue/Angular)
        let has_components = paths.iter().any(|p| p.contains("/components/") || p.contains("\\components\\"));
        let has_jsx_tsx = paths.iter().any(|p| p.ends_with(".jsx") || p.ends_with(".tsx") || p.ends_with(".vue"));
        
        if has_components && has_jsx_tsx {
            patterns.push(
                DetectedPattern::new("Component-Based", PatternCategory::Architecture)
                    .with_description("Component-based UI architecture")
                    .with_confidence(0.9)
            );
        }

        patterns
    }

    /// Detect testing patterns
    fn detect_testing_patterns(&self, analysis: &FileAnalysis) -> Vec<DetectedPattern> {
        let mut patterns = Vec::new();
        let test_files = analysis.test_files();

        if test_files.is_empty() {
            return patterns;
        }

        // Check test file organization
        let tests_in_tests_dir = test_files.iter()
            .filter(|f| f.path.starts_with("tests/") || f.path.contains("/tests/"))
            .count();
        let tests_colocated = test_files.iter()
            .filter(|f| f.path.contains(".test.") || f.path.contains("_test.") || f.path.contains(".spec."))
            .count();

        if tests_in_tests_dir > tests_colocated {
            patterns.push(
                DetectedPattern::new("Separate Test Directory", PatternCategory::Testing)
                    .with_description("Tests organized in separate tests/ directory")
                    .with_confidence(0.8)
            );
        } else if tests_colocated > 0 {
            patterns.push(
                DetectedPattern::new("Colocated Tests", PatternCategory::Testing)
                    .with_description("Tests colocated with source files")
                    .with_confidence(0.8)
            );
        }

        // Test coverage estimate
        let source_count = analysis.source_files().len();
        if source_count > 0 {
            let test_ratio = test_files.len() as f32 / source_count as f32;
            if test_ratio > 0.5 {
                patterns.push(
                    DetectedPattern::new("High Test Coverage", PatternCategory::Testing)
                        .with_description(format!("Test to source ratio: {:.0}%", test_ratio * 100.0))
                        .with_confidence(test_ratio.min(1.0))
                );
            }
        }

        patterns
    }

    /// Detect code conventions from file content
    fn detect_conventions(&self, analysis: &FileAnalysis) -> ConventionAnalysis {
        let mut indent_counts: HashMap<IndentStyle, usize> = HashMap::new();
        let mut quote_counts: HashMap<QuoteStyle, usize> = HashMap::new();
        let mut naming_samples: Vec<NamingStyle> = Vec::new();

        // Sample files for convention detection
        for file_info in analysis.files.values().take(50) {
            if let Some(Language::TypeScript) | Some(Language::JavaScript) = file_info.language {
                // Would need file content for detailed analysis
                // For now, default to common conventions
                *indent_counts.entry(IndentStyle::Spaces2).or_insert(0) += 1;
                *quote_counts.entry(QuoteStyle::Single).or_insert(0) += 1;
            } else if let Some(Language::Rust) = file_info.language {
                *indent_counts.entry(IndentStyle::Spaces4).or_insert(0) += 1;
            } else if let Some(Language::Python) = file_info.language {
                *indent_counts.entry(IndentStyle::Spaces4).or_insert(0) += 1;
                naming_samples.push(NamingStyle::SnakeCase);
            } else if let Some(Language::Go) = file_info.language {
                *indent_counts.entry(IndentStyle::Tabs).or_insert(0) += 1;
            }
        }

        // Determine dominant patterns
        let indent = indent_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(style, _)| style)
            .unwrap_or_default();

        let quotes = quote_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(style, _)| style)
            .unwrap_or_default();

        // Detect file naming from paths
        let file_naming = self.detect_file_naming_convention(analysis);

        ConventionAnalysis {
            indent,
            quotes,
            line_endings: LineEndingStyle::Unix,
            naming: NamingConventions {
                files: file_naming,
                ..Default::default()
            },
            trailing_commas: TrailingCommaStyle::Always,
        }
    }

    /// Detect file naming convention from paths
    fn detect_file_naming_convention(&self, analysis: &FileAnalysis) -> NamingStyle {
        let mut style_counts: HashMap<NamingStyle, usize> = HashMap::new();

        for path in analysis.files.keys() {
            if let Some(file_name) = path.rsplit('/').next() {
                if let Some(name) = file_name.rsplit('.').last() {
                    let style = detect_naming_style(name);
                    *style_counts.entry(style).or_insert(0) += 1;
                }
            }
        }

        style_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(style, _)| style)
            .unwrap_or_default()
    }
}

impl Default for PatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect naming style of a string
fn detect_naming_style(name: &str) -> NamingStyle {
    if name.contains('-') {
        return NamingStyle::KebabCase;
    }
    if name.contains('_') {
        if name.chars().all(|c| c.is_uppercase() || c == '_' || c.is_ascii_digit()) {
            return NamingStyle::ScreamingSnakeCase;
        }
        return NamingStyle::SnakeCase;
    }
    if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
        return NamingStyle::PascalCase;
    }
    if name.chars().any(|c| c.is_uppercase()) {
        return NamingStyle::CamelCase;
    }
    NamingStyle::SnakeCase // Default for all lowercase
}

/// Common patterns to detect
pub mod common {
    use super::*;

    /// MVC pattern
    pub fn mvc() -> DetectedPattern {
        DetectedPattern::new("MVC", PatternCategory::Architecture)
            .with_description("Model-View-Controller architecture")
    }

    /// Repository pattern
    pub fn repository() -> DetectedPattern {
        DetectedPattern::new("Repository", PatternCategory::Structural)
            .with_description("Repository pattern for data access")
    }

    /// Service layer pattern
    pub fn service_layer() -> DetectedPattern {
        DetectedPattern::new("Service Layer", PatternCategory::Structural)
            .with_description("Service layer for business logic")
    }

    /// Controller pattern
    pub fn controller() -> DetectedPattern {
        DetectedPattern::new("Controller", PatternCategory::Structural)
            .with_description("Controller pattern for request handling")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FileInfo;

    fn create_test_analysis(paths: Vec<&str>) -> FileAnalysis {
        let mut analysis = FileAnalysis::default();
        for path in paths {
            analysis.files.insert(
                path.to_string(),
                FileInfo::new(path, crate::types::FileType::Source),
            );
        }
        analysis.total_files = analysis.files.len();
        analysis
    }

    #[test]
    fn pattern_creation() {
        let pattern = DetectedPattern::new("MVC", PatternCategory::Architecture)
            .with_description("Model-View-Controller")
            .with_example("app/controllers/UserController.php")
            .with_confidence(0.9);

        assert_eq!(pattern.name, "MVC");
        assert_eq!(pattern.confidence, 0.9);
        assert_eq!(pattern.examples.len(), 1);
    }

    #[test]
    fn detect_controller_pattern() {
        let analysis = create_test_analysis(vec![
            "src/controllers/UserController.ts",
            "src/controllers/AuthController.ts",
            "src/services/UserService.ts",
        ]);

        let detector = PatternDetector::new();
        let result = detector.detect(&analysis);

        assert!(result.has_pattern("Controller"));
    }

    #[test]
    fn detect_mvc_pattern() {
        let analysis = create_test_analysis(vec![
            "app/models/User.php",
            "app/views/user/show.blade.php",
            "app/controllers/UserController.php",
        ]);

        let detector = PatternDetector::new();
        let result = detector.detect(&analysis);

        assert!(result.has_pattern("MVC"));
    }

    #[test]
    fn naming_style_detection() {
        assert_eq!(detect_naming_style("my-component"), NamingStyle::KebabCase);
        assert_eq!(detect_naming_style("my_function"), NamingStyle::SnakeCase);
        assert_eq!(detect_naming_style("MAX_VALUE"), NamingStyle::ScreamingSnakeCase);
        assert_eq!(detect_naming_style("MyClass"), NamingStyle::PascalCase);
        assert_eq!(detect_naming_style("myVariable"), NamingStyle::CamelCase);
    }
}
