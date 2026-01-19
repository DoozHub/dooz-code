//! File System Analysis
//!
//! Analyzes file structure and content using walkdir for traversal.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use sha2::{Sha256, Digest};
use walkdir::WalkDir;
use crate::types::{FileType, Language, FileInfo, ContextError};

/// File analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileAnalysis {
    /// Total files analyzed
    pub total_files: usize,
    
    /// Total lines of code
    pub total_lines: u64,
    
    /// Total bytes
    pub total_bytes: u64,
    
    /// Files by type
    pub by_type: FileTypeBreakdown,
    
    /// Files by language
    pub by_language: LanguageBreakdown,
    
    /// Large files (over threshold)
    pub large_files: Vec<String>,
    
    /// All analyzed files
    pub files: HashMap<String, FileInfo>,
}

impl FileAnalysis {
    /// Get file info by path
    pub fn get(&self, path: &str) -> Option<&FileInfo> {
        self.files.get(path)
    }

    /// Get all source files
    pub fn source_files(&self) -> Vec<&FileInfo> {
        self.files.values().filter(|f| f.is_source()).collect()
    }

    /// Get all test files
    pub fn test_files(&self) -> Vec<&FileInfo> {
        self.files.values().filter(|f| f.is_test()).collect()
    }
}

/// Breakdown by file type
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileTypeBreakdown {
    pub source: usize,
    pub test: usize,
    pub config: usize,
    pub documentation: usize,
    pub build: usize,
    pub other: usize,
}

impl FileTypeBreakdown {
    fn increment(&mut self, file_type: FileType) {
        match file_type {
            FileType::Source => self.source += 1,
            FileType::Test => self.test += 1,
            FileType::Config => self.config += 1,
            FileType::Documentation => self.documentation += 1,
            FileType::Build => self.build += 1,
            FileType::Asset | FileType::Other => self.other += 1,
        }
    }
}

/// Breakdown by language
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LanguageBreakdown {
    pub rust: usize,
    pub typescript: usize,
    pub javascript: usize,
    pub python: usize,
    pub go: usize,
    pub java: usize,
    pub php: usize,
    pub ruby: usize,
    pub c: usize,
    pub cpp: usize,
    pub other: usize,
}

impl LanguageBreakdown {
    fn increment(&mut self, language: Option<Language>) {
        match language {
            Some(Language::Rust) => self.rust += 1,
            Some(Language::TypeScript) => self.typescript += 1,
            Some(Language::JavaScript) => self.javascript += 1,
            Some(Language::Python) => self.python += 1,
            Some(Language::Go) => self.go += 1,
            Some(Language::Java) => self.java += 1,
            Some(Language::PHP) => self.php += 1,
            Some(Language::Ruby) => self.ruby += 1,
            Some(Language::C) => self.c += 1,
            Some(Language::Cpp) => self.cpp += 1,
            Some(Language::Other) | None => self.other += 1,
        }
    }
}

/// File system scanner
pub struct FileScanner {
    /// Extensions to include
    include_extensions: Vec<String>,
    
    /// Directories to exclude
    exclude_dirs: Vec<String>,
    
    /// Maximum files to scan
    max_files: usize,
    
    /// Large file threshold (bytes)
    large_file_threshold: u64,
}

impl FileScanner {
    /// Create new scanner with defaults
    pub fn new() -> Self {
        Self {
            include_extensions: vec![
                "rs".into(), "ts".into(), "tsx".into(), "js".into(), "jsx".into(),
                "py".into(), "go".into(), "java".into(), "php".into(), "rb".into(),
                "c".into(), "h".into(), "cpp".into(), "hpp".into(), "cc".into(),
                "json".into(), "yaml".into(), "yml".into(), "toml".into(),
                "md".into(), "txt".into(), "sh".into(),
            ],
            exclude_dirs: vec![
                "node_modules".into(), "target".into(), ".git".into(),
                "vendor".into(), "dist".into(), "build".into(), "__pycache__".into(),
                ".next".into(), ".nuxt".into(), "coverage".into(),
            ],
            max_files: 10000,
            large_file_threshold: 100_000, // 100KB
        }
    }

    /// Set include extensions
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.include_extensions = extensions;
        self
    }

    /// Set exclude directories
    pub fn with_excludes(mut self, excludes: Vec<String>) -> Self {
        self.exclude_dirs = excludes;
        self
    }

    /// Set max files
    pub fn with_max_files(mut self, max: usize) -> Self {
        self.max_files = max;
        self
    }

    /// Scan directory and return file analysis
    pub fn scan(&self, root: &Path) -> Result<FileAnalysis, ContextError> {
        let mut analysis = FileAnalysis::default();
        let mut file_count = 0;

        for entry in WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !self.should_exclude(e))
        {
            let entry = entry.map_err(|e| ContextError::ReadError(e.to_string()))?;
            
            if !entry.file_type().is_file() {
                continue;
            }

            if file_count >= self.max_files {
                break;
            }

            let path = entry.path();
            
            // Check extension filter
            if !self.should_include(path) {
                continue;
            }

            // Get relative path
            let rel_path = path.strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            // Analyze file
            let file_info = self.analyze_file(path, &rel_path)?;
            
            // Update stats
            analysis.total_bytes += file_info.size;
            if let Some(lines) = file_info.line_count {
                analysis.total_lines += lines as u64;
            }
            
            // Track large files
            if file_info.size > self.large_file_threshold {
                analysis.large_files.push(rel_path.clone());
            }

            // Update breakdowns
            analysis.by_type.increment(file_info.file_type);
            analysis.by_language.increment(file_info.language);

            // Store file info
            analysis.files.insert(rel_path, file_info);
            file_count += 1;
        }

        analysis.total_files = file_count;
        Ok(analysis)
    }

    /// Check if entry should be excluded
    fn should_exclude(&self, entry: &walkdir::DirEntry) -> bool {
        if entry.file_type().is_dir() {
            let name = entry.file_name().to_string_lossy();
            return self.exclude_dirs.iter().any(|d| d == name.as_ref());
        }
        false
    }

    /// Check if path should be included based on extension
    fn should_include(&self, path: &Path) -> bool {
        if self.include_extensions.is_empty() {
            return true;
        }
        
        path.extension()
            .and_then(|e| e.to_str())
            .map(|e| self.include_extensions.iter().any(|inc| inc == e))
            .unwrap_or(false)
    }

    /// Analyze a single file
    fn analyze_file(&self, path: &Path, rel_path: &str) -> Result<FileInfo, ContextError> {
        let metadata = std::fs::metadata(path)
            .map_err(|e| ContextError::ReadError(format!("Cannot read {}: {}", rel_path, e)))?;
        
        let size = metadata.len();
        let file_type = detect_file_type(rel_path);
        let language = detect_language(rel_path);

        // Read content for hash and line count
        let (content_hash, line_count, imports) = if size < self.large_file_threshold {
            match std::fs::read_to_string(path) {
                Ok(content) => {
                    let hash = compute_hash(&content);
                    let lines = content.lines().count() as u32;
                    let imports = extract_imports(&content, language);
                    (hash, Some(lines), imports)
                }
                Err(_) => {
                    // Binary file or encoding issue
                    (String::new(), None, Vec::new())
                }
            }
        } else {
            (String::new(), None, Vec::new())
        };

        Ok(FileInfo {
            path: rel_path.to_string(),
            file_type,
            content_hash,
            size,
            line_count,
            language,
            imports,
            exports: Vec::new(), // TODO: Extract exports
        })
    }
}

impl Default for FileScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute SHA-256 hash of content
fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Extract imports from file content based on language
fn extract_imports(content: &str, language: Option<Language>) -> Vec<String> {
    let mut imports = Vec::new();
    
    match language {
        Some(Language::Rust) => {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("use ") {
                    // Extract module path: use crate::foo::bar;
                    if let Some(path) = line.strip_prefix("use ").and_then(|s| s.strip_suffix(';')) {
                        imports.push(path.split("::").next().unwrap_or(path).to_string());
                    }
                } else if line.starts_with("mod ") {
                    if let Some(name) = line.strip_prefix("mod ").and_then(|s| s.strip_suffix(';')) {
                        imports.push(name.to_string());
                    }
                }
            }
        }
        Some(Language::TypeScript) | Some(Language::JavaScript) => {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("import ") || line.starts_with("export ") {
                    // Extract from 'from "..."' or 'from '...''
                    if let Some(from_idx) = line.find(" from ") {
                        let rest = &line[from_idx + 6..];
                        let quote_char = if rest.starts_with('"') { '"' } else { '\'' };
                        if let Some(start) = rest.find(quote_char) {
                            if let Some(end) = rest[start + 1..].find(quote_char) {
                                imports.push(rest[start + 1..start + 1 + end].to_string());
                            }
                        }
                    }
                } else if line.starts_with("require(") || line.contains("require(") {
                    // CommonJS require
                    if let Some(start) = line.find("require(") {
                        let rest = &line[start + 8..];
                        let quote_char = if rest.starts_with('"') { '"' } else { '\'' };
                        if let Some(start) = rest.find(quote_char) {
                            if let Some(end) = rest[start + 1..].find(quote_char) {
                                imports.push(rest[start + 1..start + 1 + end].to_string());
                            }
                        }
                    }
                }
            }
        }
        Some(Language::Python) => {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("import ") {
                    if let Some(module) = line.strip_prefix("import ") {
                        let module = module.split_whitespace().next().unwrap_or(module);
                        imports.push(module.split('.').next().unwrap_or(module).to_string());
                    }
                } else if line.starts_with("from ") {
                    if let Some(rest) = line.strip_prefix("from ") {
                        let module = rest.split_whitespace().next().unwrap_or(rest);
                        imports.push(module.split('.').next().unwrap_or(module).to_string());
                    }
                }
            }
        }
        Some(Language::Go) => {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("import ") {
                    if let Some(start) = line.find('"') {
                        if let Some(end) = line[start + 1..].find('"') {
                            imports.push(line[start + 1..start + 1 + end].to_string());
                        }
                    }
                }
            }
        }
        Some(Language::PHP) => {
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("use ") {
                    if let Some(ns) = line.strip_prefix("use ").and_then(|s| s.strip_suffix(';')) {
                        imports.push(ns.split('\\').next().unwrap_or(ns).to_string());
                    }
                } else if line.contains("require") || line.contains("include") {
                    // Extract file path from require/include
                    for keyword in ["require_once", "include_once", "require", "include"] {
                        if line.contains(keyword) {
                            if let Some(start) = line.find('\'').or_else(|| line.find('"')) {
                                let quote = line.chars().nth(start).unwrap();
                                if let Some(end) = line[start + 1..].find(quote) {
                                    imports.push(line[start + 1..start + 1 + end].to_string());
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }

    imports
}

/// Detect language from file extension
pub fn detect_language(path: &str) -> Option<Language> {
    let ext = path.rsplit('.').next()?;
    
    match ext.to_lowercase().as_str() {
        "rs" => Some(Language::Rust),
        "ts" | "tsx" => Some(Language::TypeScript),
        "js" | "jsx" | "mjs" | "cjs" => Some(Language::JavaScript),
        "py" | "pyw" => Some(Language::Python),
        "go" => Some(Language::Go),
        "java" => Some(Language::Java),
        "php" => Some(Language::PHP),
        "rb" => Some(Language::Ruby),
        "c" | "h" => Some(Language::C),
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => Some(Language::Cpp),
        _ => None,
    }
}

/// Detect file type from path
pub fn detect_file_type(path: &str) -> FileType {
    let path_lower = path.to_lowercase();
    
    // Test files
    if path_lower.contains("test") 
        || path_lower.contains("spec")
        || path_lower.contains("_test.")
        || path_lower.contains(".test.")
        || path_lower.starts_with("tests/")
        || path_lower.contains("/tests/")
    {
        return FileType::Test;
    }
    
    // Config files
    if path_lower.ends_with(".json")
        || path_lower.ends_with(".yaml")
        || path_lower.ends_with(".yml")
        || path_lower.ends_with(".toml")
        || path_lower.ends_with(".ini")
        || path_lower.ends_with(".env")
        || path_lower.ends_with(".config.js")
        || path_lower.ends_with(".config.ts")
        || path_lower.contains("config")
        || path_lower == "package.json"
        || path_lower == "cargo.toml"
        || path_lower == "go.mod"
        || path_lower == "composer.json"
        || path_lower == "gemfile"
        || path_lower == "requirements.txt"
    {
        return FileType::Config;
    }
    
    // Build files - check BEFORE documentation to catch Dockerfile
    if path_lower.contains("makefile")
        || path_lower.contains("dockerfile")
        || path_lower.ends_with(".sh")
        || path_lower.ends_with(".bash")
        || path_lower.ends_with(".cmake")
        || path_lower.ends_with(".mk")
        || path_lower == "justfile"
    {
        return FileType::Build;
    }
    
    // Documentation
    if path_lower.ends_with(".md")
        || path_lower.ends_with(".rst")
        || path_lower.ends_with(".txt")
        || path_lower.contains("doc/")
        || path_lower.contains("/doc/")
        || path_lower.contains("readme")
        || path_lower.contains("changelog")
        || path_lower.contains("license")
    {
        return FileType::Documentation;
    }
    
    // Source code
    if detect_language(path).is_some() {
        return FileType::Source;
    }
    
    FileType::Other
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn language_detection() {
        assert_eq!(detect_language("main.rs"), Some(Language::Rust));
        assert_eq!(detect_language("app.ts"), Some(Language::TypeScript));
        assert_eq!(detect_language("app.tsx"), Some(Language::TypeScript));
        assert_eq!(detect_language("script.py"), Some(Language::Python));
        assert_eq!(detect_language("main.go"), Some(Language::Go));
        assert_eq!(detect_language("App.java"), Some(Language::Java));
        assert_eq!(detect_language("unknown.xyz"), None);
    }

    #[test]
    fn file_type_detection() {
        assert_eq!(detect_file_type("src/main.rs"), FileType::Source);
        assert_eq!(detect_file_type("tests/test_main.rs"), FileType::Test);
        assert_eq!(detect_file_type("src/main_test.go"), FileType::Test);
        assert_eq!(detect_file_type("config.json"), FileType::Config);
        assert_eq!(detect_file_type("README.md"), FileType::Documentation);
        assert_eq!(detect_file_type("Dockerfile"), FileType::Build);
    }

    #[test]
    fn scan_directory() {
        let dir = tempdir().unwrap();
        
        // Create test files
        let src = dir.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(src.join("lib.rs"), "pub mod foo;\n").unwrap();
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
        
        let scanner = FileScanner::new();
        let analysis = scanner.scan(dir.path()).unwrap();

        assert_eq!(analysis.total_files, 3);
        assert_eq!(analysis.by_type.source, 2);
        assert_eq!(analysis.by_type.config, 1);
        assert_eq!(analysis.by_language.rust, 2);
    }

    #[test]
    fn extract_rust_imports() {
        let content = r#"
use std::io;
use crate::types::Foo;
mod bar;
"#;
        let imports = extract_imports(content, Some(Language::Rust));
        assert!(imports.contains(&"std".to_string()));
        assert!(imports.contains(&"crate".to_string()));
        assert!(imports.contains(&"bar".to_string()));
    }

    #[test]
    fn extract_js_imports() {
        let content = r#"
import React from 'react';
import { useState } from "react";
const fs = require('fs');
"#;
        let imports = extract_imports(content, Some(Language::JavaScript));
        assert!(imports.contains(&"react".to_string()));
        assert!(imports.contains(&"fs".to_string()));
    }

    #[test]
    fn extract_python_imports() {
        let content = r#"
import os
from pathlib import Path
import json as js
"#;
        let imports = extract_imports(content, Some(Language::Python));
        assert!(imports.contains(&"os".to_string()));
        assert!(imports.contains(&"pathlib".to_string()));
        assert!(imports.contains(&"json".to_string()));
    }
}
