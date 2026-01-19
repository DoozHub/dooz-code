//! Dependency Analysis
//!
//! Analyzes dependencies between files, modules, and external packages.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use crate::types::Language;
use super::files::FileAnalysis;

/// Dependency analysis results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    /// Internal dependency graph
    pub graph: DependencyGraph,
    
    /// Circular dependencies detected
    pub circular: Vec<CircularDep>,
    
    /// External dependencies
    pub external: Vec<ExternalDep>,
    
    /// Dependency statistics
    pub stats: DependencyStats,
}

impl DependencyAnalysis {
    /// Check for circular dependencies
    pub fn has_cycles(&self) -> bool {
        !self.circular.is_empty()
    }

    /// Get external dependency count
    pub fn external_count(&self) -> usize {
        self.external.len()
    }

    /// Get most depended-on files
    pub fn most_depended(&self) -> Vec<(&str, usize)> {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        
        for edge in &self.graph.edges {
            *counts.entry(edge.to.as_str()).or_insert(0) += 1;
        }

        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1));
        result.truncate(10);
        result
    }
}

/// Dependency graph
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// Nodes (files/modules)
    pub nodes: Vec<String>,
    
    /// Edges (dependencies)
    pub edges: Vec<DepEdge>,
}

impl DependencyGraph {
    /// Create new graph
    pub fn new() -> Self {
        Self::default()
    }

    /// Add node
    pub fn add_node(&mut self, node: impl Into<String>) {
        let node = node.into();
        if !self.nodes.contains(&node) {
            self.nodes.push(node);
        }
    }

    /// Add edge
    pub fn add_edge(&mut self, from: impl Into<String>, to: impl Into<String>, dep_type: DepType) {
        let from = from.into();
        let to = to.into();
        
        self.add_node(from.clone());
        self.add_node(to.clone());
        
        let edge = DepEdge { from, to, dep_type };
        if !self.edges.contains(&edge) {
            self.edges.push(edge);
        }
    }

    /// Get dependencies of a node
    pub fn dependencies_of(&self, node: &str) -> Vec<&str> {
        self.edges
            .iter()
            .filter(|e| e.from == node)
            .map(|e| e.to.as_str())
            .collect()
    }

    /// Get dependents of a node (what depends on it)
    pub fn dependents_of(&self, node: &str) -> Vec<&str> {
        self.edges
            .iter()
            .filter(|e| e.to == node)
            .map(|e| e.from.as_str())
            .collect()
    }

    /// Get node degree (in + out edges)
    pub fn degree(&self, node: &str) -> usize {
        self.edges.iter()
            .filter(|e| e.from == node || e.to == node)
            .count()
    }

    /// Node count
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Edge count
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Find all cycles in the graph
    pub fn find_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node in &self.nodes {
            if !visited.contains(node) {
                self.dfs_cycles(node, &mut visited, &mut rec_stack, &mut path, &mut cycles);
            }
        }

        cycles
    }

    fn dfs_cycles(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        for dep in self.dependencies_of(node) {
            if !visited.contains(dep) {
                self.dfs_cycles(dep, visited, rec_stack, path, cycles);
            } else if rec_stack.contains(dep) {
                // Found a cycle
                let cycle_start = path.iter().position(|n| n == dep).unwrap_or(0);
                let cycle: Vec<String> = path[cycle_start..].to_vec();
                if !cycle.is_empty() && !cycles.contains(&cycle) {
                    cycles.push(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
    }
}

/// Dependency edge
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepEdge {
    /// Source node
    pub from: String,
    
    /// Target node
    pub to: String,
    
    /// Type of dependency
    pub dep_type: DepType,
}

/// Type of dependency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DepType {
    /// Import dependency
    Import,
    
    /// Re-export
    ReExport,
    
    /// Inheritance
    Inheritance,
    
    /// Implementation/Trait
    Implementation,
    
    /// Type usage
    TypeUsage,
}

/// Circular dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDep {
    /// Nodes in the cycle
    pub cycle: Vec<String>,
    
    /// Severity (higher = worse)
    pub severity: u8,
}

impl CircularDep {
    /// Create new circular dependency
    pub fn new(cycle: Vec<String>) -> Self {
        let severity = cycle.len().min(10) as u8;
        Self { cycle, severity }
    }

    /// Get cycle length
    pub fn len(&self) -> usize {
        self.cycle.len()
    }

    /// Check if cycle is empty
    pub fn is_empty(&self) -> bool {
        self.cycle.is_empty()
    }
}

/// External dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDep {
    /// Package name
    pub name: String,
    
    /// Version constraint (if known)
    pub version: Option<String>,
    
    /// Source registry
    pub source: DepSource,
    
    /// Files that use this dependency
    pub used_by: Vec<String>,
    
    /// Import count
    pub import_count: usize,
}

impl ExternalDep {
    /// Create new external dependency
    pub fn new(name: impl Into<String>, source: DepSource) -> Self {
        Self {
            name: name.into(),
            version: None,
            source,
            used_by: Vec::new(),
            import_count: 0,
        }
    }

    /// With version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Add usage
    pub fn used_in(mut self, file: impl Into<String>) -> Self {
        let file = file.into();
        if !self.used_by.contains(&file) {
            self.used_by.push(file);
        }
        self.import_count += 1;
        self
    }
}

/// Dependency source registry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DepSource {
    /// Rust crates.io
    Crates,
    
    /// npm
    Npm,
    
    /// Python pip
    Pip,
    
    /// Go modules
    GoMod,
    
    /// PHP Composer
    Composer,
    
    /// Local/Internal
    Local,
    
    /// Unknown
    Unknown,
}

/// Dependency statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyStats {
    /// Total internal dependencies
    pub internal_count: usize,
    
    /// Total external dependencies
    pub external_count: usize,
    
    /// Average dependencies per file
    pub avg_deps_per_file: f32,
    
    /// Max dependencies for a single file
    pub max_deps: usize,
    
    /// Files with most dependencies
    pub highest_coupling: Vec<String>,
}

/// Dependency analyzer
pub struct DependencyAnalyzer {
    /// Include local dependencies
    include_local: bool,
}

impl DependencyAnalyzer {
    /// Create new analyzer
    pub fn new() -> Self {
        Self { include_local: true }
    }

    /// Analyze dependencies from file analysis
    pub fn analyze(&self, file_analysis: &FileAnalysis) -> DependencyAnalysis {
        let mut graph = DependencyGraph::new();
        let mut external_deps: HashMap<String, ExternalDep> = HashMap::new();
        
        // Build graph from file imports
        for (path, file_info) in &file_analysis.files {
            graph.add_node(path.clone());
            
            for import in &file_info.imports {
                if self.is_external_import(import, file_info.language) {
                    // External dependency
                    let source = self.detect_source(file_info.language);
                    let dep = external_deps.entry(import.clone())
                        .or_insert_with(|| ExternalDep::new(import.clone(), source));
                    if !dep.used_by.contains(path) {
                        dep.used_by.push(path.clone());
                    }
                    dep.import_count += 1;
                } else if self.include_local {
                    // Internal dependency - try to resolve to a file
                    if let Some(resolved) = self.resolve_import(import, path, file_analysis) {
                        graph.add_edge(path.clone(), resolved, DepType::Import);
                    }
                }
            }
        }

        // Find circular dependencies
        let cycles = graph.find_cycles();
        let circular: Vec<CircularDep> = cycles.into_iter()
            .map(CircularDep::new)
            .collect();

        // Convert external deps to vec
        let external: Vec<ExternalDep> = external_deps.into_values().collect();

        // Calculate stats
        let stats = self.calculate_stats(&graph, &external, file_analysis);

        DependencyAnalysis {
            graph,
            circular,
            external,
            stats,
        }
    }

    /// Check if import is external (not local file)
    fn is_external_import(&self, import: &str, language: Option<Language>) -> bool {
        match language {
            Some(Language::Rust) => {
                // Rust: external if not starting with crate, self, super, or known module
                !import.starts_with("crate") 
                    && !import.starts_with("self") 
                    && !import.starts_with("super")
                    && import != "std"
            }
            Some(Language::TypeScript) | Some(Language::JavaScript) => {
                // JS/TS: external if not starting with . or /
                !import.starts_with('.') && !import.starts_with('/')
            }
            Some(Language::Python) => {
                // Python: external if not a relative import and not in common stdlib
                !import.starts_with('.')
                    && !["os", "sys", "json", "re", "typing", "collections", "functools", 
                         "itertools", "pathlib", "datetime", "math", "random", "time",
                         "logging", "unittest", "dataclasses", "enum", "abc"].contains(&import)
            }
            Some(Language::Go) => {
                // Go: external if contains a domain-like path
                import.contains('.') && import.contains('/')
            }
            Some(Language::PHP) => {
                // PHP: external if not App\ or local namespace
                !import.starts_with("App") && import.contains('\\')
            }
            _ => true,
        }
    }

    /// Detect dependency source from language
    fn detect_source(&self, language: Option<Language>) -> DepSource {
        match language {
            Some(Language::Rust) => DepSource::Crates,
            Some(Language::TypeScript) | Some(Language::JavaScript) => DepSource::Npm,
            Some(Language::Python) => DepSource::Pip,
            Some(Language::Go) => DepSource::GoMod,
            Some(Language::PHP) => DepSource::Composer,
            _ => DepSource::Unknown,
        }
    }

    /// Resolve import to a file path
    fn resolve_import(&self, import: &str, from_path: &str, analysis: &FileAnalysis) -> Option<String> {
        // Try to match import to existing files
        let possible_paths = [
            format!("{}.rs", import.replace("::", "/")),
            format!("{}/mod.rs", import.replace("::", "/")),
            format!("{}.ts", import.replace("./", "").replace("../", "")),
            format!("{}/index.ts", import.replace("./", "").replace("../", "")),
            format!("{}.js", import.replace("./", "").replace("../", "")),
            format!("{}/index.js", import.replace("./", "").replace("../", "")),
            format!("{}.py", import.replace(".", "/")),
            format!("{}/__init__.py", import.replace(".", "/")),
        ];

        for path in possible_paths {
            if analysis.files.contains_key(&path) {
                return Some(path);
            }
        }

        // Try relative resolution from current file's directory
        if let Some(dir) = from_path.rsplit_once('/').map(|(d, _)| d) {
            let relative = if import.starts_with("./") {
                format!("{}/{}", dir, &import[2..])
            } else if import.starts_with("../") {
                // Go up one directory
                if let Some(parent) = dir.rsplit_once('/').map(|(d, _)| d) {
                    format!("{}/{}", parent, &import[3..])
                } else {
                    import[3..].to_string()
                }
            } else {
                return None;
            };

            for ext in ["", ".ts", ".js", ".tsx", ".jsx", "/index.ts", "/index.js"] {
                let full_path = format!("{}{}", relative, ext);
                if analysis.files.contains_key(&full_path) {
                    return Some(full_path);
                }
            }
        }

        None
    }

    /// Calculate dependency statistics
    fn calculate_stats(&self, graph: &DependencyGraph, external: &[ExternalDep], analysis: &FileAnalysis) -> DependencyStats {
        let mut deps_per_file: HashMap<&str, usize> = HashMap::new();
        
        for edge in &graph.edges {
            *deps_per_file.entry(edge.from.as_str()).or_insert(0) += 1;
        }

        let max_deps = deps_per_file.values().copied().max().unwrap_or(0);
        let total_deps: usize = deps_per_file.values().sum();
        let file_count = analysis.total_files.max(1);
        let avg_deps = total_deps as f32 / file_count as f32;

        // Find highest coupling files
        let mut sorted: Vec<_> = deps_per_file.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        let highest_coupling: Vec<String> = sorted.into_iter()
            .take(5)
            .map(|(k, _)| k.to_string())
            .collect();

        DependencyStats {
            internal_count: graph.edge_count(),
            external_count: external.len(),
            avg_deps_per_file: avg_deps,
            max_deps,
            highest_coupling,
        }
    }
}

impl Default for DependencyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FileInfo;

    #[test]
    fn graph_operations() {
        let mut graph = DependencyGraph::new();
        graph.add_edge("a.rs", "b.rs", DepType::Import);
        graph.add_edge("a.rs", "c.rs", DepType::Import);
        graph.add_edge("b.rs", "c.rs", DepType::Import);

        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 3);
        
        let deps = graph.dependencies_of("a.rs");
        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn cycle_detection() {
        let mut graph = DependencyGraph::new();
        graph.add_edge("a.rs", "b.rs", DepType::Import);
        graph.add_edge("b.rs", "c.rs", DepType::Import);
        graph.add_edge("c.rs", "a.rs", DepType::Import);

        let cycles = graph.find_cycles();
        assert!(!cycles.is_empty());
    }

    #[test]
    fn external_dep() {
        let dep = ExternalDep::new("serde", DepSource::Crates)
            .with_version("1.0")
            .used_in("src/types/mod.rs")
            .used_in("src/lib.rs");

        assert_eq!(dep.name, "serde");
        assert_eq!(dep.version, Some("1.0".to_string()));
        assert_eq!(dep.used_by.len(), 2);
        assert_eq!(dep.import_count, 2);
    }

    #[test]
    fn is_external_import_js() {
        let analyzer = DependencyAnalyzer::new();
        
        assert!(analyzer.is_external_import("react", Some(Language::JavaScript)));
        assert!(analyzer.is_external_import("lodash", Some(Language::JavaScript)));
        assert!(!analyzer.is_external_import("./utils", Some(Language::JavaScript)));
        assert!(!analyzer.is_external_import("../components/Button", Some(Language::JavaScript)));
    }

    #[test]
    fn is_external_import_rust() {
        let analyzer = DependencyAnalyzer::new();
        
        assert!(analyzer.is_external_import("serde", Some(Language::Rust)));
        assert!(!analyzer.is_external_import("crate", Some(Language::Rust)));
        assert!(!analyzer.is_external_import("self", Some(Language::Rust)));
        assert!(!analyzer.is_external_import("super", Some(Language::Rust)));
    }
}
