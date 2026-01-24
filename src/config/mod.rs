//! Configuration Module
//!
//! Provides configuration file support for dooz-code.
//! Supports YAML and JSON configuration files.
//! Configuration can be overridden by environment variables.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;

use crate::executor::{ExecutorConfig, ComputerUseConfig};

/// Dooz-Code configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DoozCodeConfig {
    /// LLM provider configuration
    #[serde(default)]
    pub llm: LlmConfig,

    /// Executor configuration
    #[serde(default)]
    pub executor: ExecutorConfigSpec,

    /// Analyzer configuration
    #[serde(default)]
    pub analyzer: AnalyzerConfigSpec,

    /// Reviewer configuration
    #[serde(default)]
    pub reviewer: ReviewerConfigSpec,

    /// General settings
    #[serde(default)]
    pub general: GeneralConfigSpec,
}

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Provider type (stub, computer-use, openai, anthropic)
    #[serde(default = "default_provider")]
    pub provider: String,

    /// API endpoint URL
    #[serde(default)]
    pub api_url: Option<String>,

    /// API authentication key
    #[serde(default)]
    pub api_key: Option<String>,

    /// Model ID to use
    #[serde(default)]
    pub model: Option<String>,

    /// Maximum tokens to generate
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Temperature for generation (0.0 - 1.0)
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Number of retry attempts on failure
    #[serde(default = "default_retries")]
    pub retries: u32,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,

    /// Fallback models (tried in order if primary fails)
    #[serde(default)]
    pub fallback_models: Vec<String>,
}

fn default_provider() -> String {
    "stub".to_string()
}

fn default_max_tokens() -> u32 {
    4096
}

fn default_temperature() -> f32 {
    0.2
}

fn default_retries() -> u32 {
    3
}

fn default_timeout() -> u32 {
    60
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: default_provider(),
            api_url: None,
            api_key: None,
            model: None,
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
            retries: default_retries(),
            timeout_seconds: default_timeout(),
            fallback_models: Vec::new(),
        }
    }
}

/// Executor configuration specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorConfigSpec {
    /// Maximum artifacts to generate
    #[serde(default = "default_max_artifacts")]
    pub max_artifacts: usize,

    /// Maximum lines per file
    #[serde(default = "default_max_lines")]
    pub max_lines_per_file: usize,

    /// Dry run mode (no file writes)
    #[serde(default)]
    pub dry_run: bool,

    /// Follow detected patterns
    #[serde(default = "default_true")]
    pub follow_patterns: bool,

    /// Enable correction attempts
    #[serde(default = "default_true")]
    pub enable_correction: bool,

    /// Maximum correction iterations
    #[serde(default = "default_max_corrections")]
    pub max_corrections: usize,
}

fn default_max_artifacts() -> usize {
    100
}

fn default_max_lines() -> usize {
    1000
}

fn default_true() -> bool {
    true
}

fn default_max_corrections() -> usize {
    3
}

impl Default for ExecutorConfigSpec {
    fn default() -> Self {
        Self {
            max_artifacts: default_max_artifacts(),
            max_lines_per_file: default_max_lines(),
            dry_run: false,
            follow_patterns: default_true(),
            enable_correction: default_true(),
            max_corrections: default_max_corrections(),
        }
    }
}

/// Analyzer configuration specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerConfigSpec {
    /// File extensions to include
    #[serde(default)]
    pub include_extensions: Vec<String>,

    /// File patterns to exclude
    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    /// Directories to exclude
    #[serde(default = "default_exclude_dirs")]
    pub exclude_dirs: Vec<String>,

    /// Maximum file size to analyze (bytes)
    #[serde(default = "default_max_file_size")]
    pub max_file_size: u64,

    /// Parallel analysis threads
    #[serde(default = "default_threads")]
    pub threads: usize,
}

fn default_exclude_dirs() -> Vec<String> {
    vec!["target".to_string(), "node_modules".to_string(), ".git".to_string()]
}

fn default_max_file_size() -> u64 {
    1024 * 1024 // 1MB
}

fn default_threads() -> usize {
    num_cpus::get()
}

impl Default for AnalyzerConfigSpec {
    fn default() -> Self {
        Self {
            include_extensions: Vec::new(),
            exclude_patterns: Vec::new(),
            exclude_dirs: default_exclude_dirs(),
            max_file_size: default_max_file_size(),
            threads: default_threads(),
        }
    }
}

/// Reviewer configuration specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewerConfigSpec {
    /// Enable security checks
    #[serde(default = "default_true")]
    pub security_checks: bool,

    /// Enable performance checks
    #[serde(default = "default_true")]
    pub performance_checks: bool,

    /// Enable style checks
    #[serde(default = "default_true")]
    pub style_checks: bool,

    /// Require test coverage threshold (%)
    #[serde(default)]
    pub min_test_coverage: Option<f32>,

    /// Custom linting rules
    #[serde(default)]
    pub custom_rules: HashMap<String, String>,
}

impl Default for ReviewerConfigSpec {
    fn default() -> Self {
        Self {
            security_checks: default_true(),
            performance_checks: default_true(),
            style_checks: default_true(),
            min_test_coverage: None,
            custom_rules: HashMap::new(),
        }
    }
}

/// General configuration specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfigSpec {
    /// Working directory
    #[serde(default)]
    pub work_dir: Option<PathBuf>,

    /// Output format (json, yaml, summary)
    #[serde(default = "default_output_format")]
    pub output_format: String,

    /// Verbose logging
    #[serde(default)]
    pub verbose: bool,

    /// Log level (debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Colors in output
    #[serde(default = "default_true")]
    pub colors: bool,
}

fn default_output_format() -> String {
    "summary".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for GeneralConfigSpec {
    fn default() -> Self {
        Self {
            work_dir: None,
            output_format: default_output_format(),
            verbose: false,
            log_level: default_log_level(),
            colors: default_true(),
        }
    }
}

impl DoozCodeConfig {
    /// Load configuration from file (YAML or JSON)
    pub fn from_file(path: &PathBuf) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::IoError {
                path: path.clone(),
                message: e.to_string(),
            })?;

        // Try YAML first, then JSON
        if path.extension().map(|e| e == "yaml" || e == "yml").unwrap_or(false) {
            serde_yaml::from_str(&content)
                .map_err(|e| ConfigError::ParseError {
                    path: path.clone(),
                    message: e.to_string(),
                })
        } else {
            serde_json::from_str(&content)
                .map_err(|e| ConfigError::ParseError {
                    path: path.clone(),
                    message: e.to_string(),
                })
        }
    }

    /// Load configuration from default locations
    /// Searches in: ./dooz-code.yaml, ./dooz-code.json, ~/.config/dooz-code.yaml
    pub fn load_default() -> Result<Self, ConfigError> {
        // Current directory
        if let Ok(config) = Self::from_file(&PathBuf::from("dooz-code.yaml")) {
            return Ok(config);
        }
        if let Ok(config) = Self::from_file(&PathBuf::from("dooz-code.yml")) {
            return Ok(config);
        }
        if let Ok(config) = Self::from_file(&PathBuf::from("dooz-code.json")) {
            return Ok(config);
        }

        // Home directory
        let home_config = home::home_dir()
            .map(|p| p.join(".config/dooz-code.yaml"));

        if let Some(path) = home_config {
            if let Ok(config) = Self::from_file(&path) {
                return Ok(config);
            }
        }

        // Return default config if no file found
        Ok(Self::default())
    }

    /// Merge with environment variables
    pub fn with_env_overrides(mut self) -> Self {
        // LLM overrides
        if let Ok(url) = std::env::var("DOOZ_LLM_API_URL") {
            self.llm.api_url = Some(url);
        }
        if let Ok(key) = std::env::var("DOOZ_LLM_API_KEY") {
            self.llm.api_key = Some(key);
        }
        if let Ok(model) = std::env::var("DOOZ_LLM_MODEL") {
            self.llm.model = Some(model);
        }

        // General overrides
        if let Ok(format) = std::env::var("DOOZ_OUTPUT_FORMAT") {
            self.general.output_format = format;
        }
        if let Ok(level) = std::env::var("DOOZ_LOG_LEVEL") {
            self.general.log_level = level;
        }
        if std::env::var("DOOZ_VERBOSE").is_ok() {
            self.general.verbose = true;
        }

        self
    }

    /// Convert to executor config
    pub fn to_executor_config(&self) -> ExecutorConfig {
        ExecutorConfig {
            max_artifacts: self.executor.max_artifacts,
            max_lines_per_file: self.executor.max_lines_per_file,
            dry_run: self.executor.dry_run,
            follow_patterns: self.executor.follow_patterns,
            enable_correction: self.executor.enable_correction,
            max_corrections: self.executor.max_corrections,
        }
    }

    /// Convert to LLM config
    pub fn to_llm_config(&self) -> ComputerUseConfig {
        ComputerUseConfig {
            api_url: self.llm.api_url.clone().unwrap_or_else(|| "http://127.0.0.1:8315".to_string()),
            api_key: self.llm.api_key.clone().unwrap_or_default(),
            model_id: self.llm.model.clone().unwrap_or_else(|| "gemini-2.5-computer-use-preview-10-2025".to_string()),
            max_tokens: self.llm.max_tokens,
            temperature: self.llm.temperature,
        }
    }

    /// Save configuration to file
    pub fn save(&self, path: &PathBuf) -> Result<(), ConfigError> {
        let content = serde_yaml::to_string(self)
            .map_err(|e| ConfigError::SerializationError {
                message: e.to_string(),
            })?;

        std::fs::write(path, content)
            .map_err(|e| ConfigError::IoError {
                path: path.clone(),
                message: e.to_string(),
            })
    }
}

/// Configuration errors
#[derive(Debug, Clone)]
pub enum ConfigError {
    IoError { path: PathBuf, message: String },
    ParseError { path: PathBuf, message: String },
    SerializationError { message: String },
    ValidationError { message: String },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IoError { path, message } => {
                write!(f, "IO error reading {}: {}", path.display(), message)
            }
            Self::ParseError { path, message } => {
                write!(f, "Parse error in {}: {}", path.display(), message)
            }
            Self::SerializationError { message } => {
                write!(f, "Serialization error: {}", message)
            }
            Self::ValidationError { message } => {
                write!(f, "Validation error: {}", message)
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Configuration file template
pub fn generate_config_template() -> &'static str {
    r#"# Dooz-Code Configuration File
# Generated by dooz-code v0.2.0

# LLM Provider Configuration
llm:
  provider: "stub"  # stub, computer-use, openai, anthropic
  api_url: "http://127.0.0.1:8315"
  api_key: ""  # Set via DOOZ_LLM_API_KEY environment variable
  model: "gemini-2.5-computer-use-preview-10-2025"
  max_tokens: 4096
  temperature: 0.2
  retries: 3
  timeout_seconds: 60
  fallback_models: []

# Executor Configuration
executor:
  max_artifacts: 100
  max_lines_per_file: 1000
  dry_run: false
  follow_patterns: true
  enable_correction: true
  max_corrections: 3

# Analyzer Configuration
analyzer:
  include_extensions: []
  exclude_patterns: []
  exclude_dirs:
    - target
    - node_modules
    - .git
  max_file_size: 1048576
  threads: 4

# Reviewer Configuration
reviewer:
  security_checks: true
  performance_checks: true
  style_checks: true
  min_test_coverage: null
  custom_rules: {}

# General Configuration
general:
  work_dir: null
  output_format: "summary"
  verbose: false
  log_level: "info"
  colors: true
"#
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn config_default() {
        let config = DoozCodeConfig::default();
        assert_eq!(config.llm.provider, "stub");
        assert_eq!(config.executor.max_artifacts, 100);
    }

    #[test]
    fn config_from_yaml() {
        let yaml = r#"
llm:
  provider: "computer-use"
  api_url: "http://localhost:8080"
  model: "test-model"
executor:
  max_artifacts: 50
"#;
        let config: DoozCodeConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.llm.provider, "computer-use");
        assert_eq!(config.llm.api_url, Some("http://localhost:8080".to_string()));
        assert_eq!(config.executor.max_artifacts, 50);
    }

    #[test]
    fn config_from_json() {
        let json = r#"{
  "llm": {
    "provider": "computer-use",
    "api_url": "http://localhost:8080",
    "model": "test-model"
  },
  "executor": {
    "max_artifacts": 50
  }
}"#;
        let config: DoozCodeConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.llm.provider, "computer-use");
        assert_eq!(config.executor.max_artifacts, 50);
    }

    #[test]
    fn config_save_and_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.yaml");

        let config = DoozCodeConfig {
            llm: LlmConfig {
                provider: "computer-use".to_string(),
                api_url: Some("http://test:8080".to_string()),
                ..Default::default()
            },
            ..Default::default()
        };

        config.save(&path).unwrap();
        let loaded = DoozCodeConfig::from_file(&path).unwrap();

        assert_eq!(loaded.llm.provider, "computer-use");
        assert_eq!(loaded.llm.api_url, Some("http://test:8080".to_string()));
    }

    #[test]
    fn config_env_overrides() {
        std::env::set_var("DOOZ_LLM_API_KEY", "test-key-123");
        std::env::set_var("DOOZ_VERBOSE", "true");

        let config = DoozCodeConfig::default().with_env_overrides();

        assert_eq!(config.llm.api_key, Some("test-key-123".to_string()));
        assert!(config.general.verbose);

        std::env::remove_var("DOOZ_LLM_API_KEY");
        std::env::remove_var("DOOZ_VERBOSE");
    }

    #[test]
    fn config_to_executor_config() {
        let config = DoozCodeConfig {
            executor: ExecutorConfigSpec {
                max_artifacts: 200,
                max_lines_per_file: 500,
                dry_run: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let exec_config = config.to_executor_config();
        assert_eq!(exec_config.max_artifacts, 200);
        assert!(exec_config.dry_run);
    }

    #[test]
    fn generate_config_template_is_valid_yaml() {
        let template = generate_config_template();
        let config: DoozCodeConfig = serde_yaml::from_str(template).unwrap();
        assert_eq!(config.llm.provider, "stub");
        assert!(config.analyzer.exclude_dirs.contains(&"target".to_string()));
    }
}
