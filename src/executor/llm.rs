//! LLM Provider Abstraction
//!
//! Defines traits and implementations for LLM integration.
//! Supports stub providers for testing and actual LLM providers for production.

use crate::types::{ContextError, Language};
use crate::analyzer::AnalyzedContext;
use std::collections::HashMap;

/// LLM provider trait - implemented by external orchestrators
pub trait LlmProvider: Send + Sync {
    /// Generate code based on prompt and context
    fn generate_code(&self, request: &CodeRequest) -> Result<CodeResponse, ContextError>;
    
    /// Correct code based on feedback
    fn correct_code(&self, request: &CorrectionRequest) -> Result<CodeResponse, ContextError>;
    
    /// Provider name
    fn name(&self) -> &str;
}

/// Request for code generation
#[derive(Debug, Clone)]
pub struct CodeRequest {
    /// Description of what to generate
    pub description: String,
    
    /// Target file path
    pub target_path: String,
    
    /// Target language
    pub language: Language,
    
    /// Generation type
    pub gen_type: GenerationIntent,
    
    /// Context from analysis
    pub context_summary: ContextSummary,
    
    /// Constraints to follow
    pub constraints: Vec<String>,
    
    /// Patterns to follow
    pub patterns: Vec<String>,
}

impl CodeRequest {
    /// Create new request
    pub fn new(description: impl Into<String>, target_path: impl Into<String>, language: Language) -> Self {
        Self {
            description: description.into(),
            target_path: target_path.into(),
            language,
            gen_type: GenerationIntent::Implementation,
            context_summary: ContextSummary::default(),
            constraints: Vec::new(),
            patterns: Vec::new(),
        }
    }

    /// With generation intent
    pub fn with_intent(mut self, intent: GenerationIntent) -> Self {
        self.gen_type = intent;
        self
    }

    /// With context summary
    pub fn with_context(mut self, context: ContextSummary) -> Self {
        self.context_summary = context;
        self
    }

    /// Add constraint
    pub fn with_constraint(mut self, constraint: impl Into<String>) -> Self {
        self.constraints.push(constraint.into());
        self
    }

    /// Add pattern
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.patterns.push(pattern.into());
        self
    }

    /// Build prompt for LLM
    pub fn to_prompt(&self) -> String {
        let mut prompt = String::new();
        
        prompt.push_str(&format!("Generate {} code for: {}\n\n", 
            format!("{:?}", self.language).to_lowercase(),
            self.description
        ));
        
        prompt.push_str(&format!("Target file: {}\n", self.target_path));
        prompt.push_str(&format!("Intent: {:?}\n\n", self.gen_type));
        
        if !self.patterns.is_empty() {
            prompt.push_str("Follow these patterns:\n");
            for pattern in &self.patterns {
                prompt.push_str(&format!("- {}\n", pattern));
            }
            prompt.push('\n');
        }
        
        if !self.constraints.is_empty() {
            prompt.push_str("Constraints:\n");
            for constraint in &self.constraints {
                prompt.push_str(&format!("- {}\n", constraint));
            }
            prompt.push('\n');
        }
        
        if !self.context_summary.imports.is_empty() {
            prompt.push_str("Available imports:\n");
            for import in &self.context_summary.imports {
                prompt.push_str(&format!("- {}\n", import));
            }
            prompt.push('\n');
        }
        
        prompt.push_str("Generate complete, production-ready code.\n");
        
        prompt
    }
}

/// Generation intent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenerationIntent {
    /// Create new implementation
    Implementation,
    /// Create tests
    Test,
    /// Modify existing code
    Modification,
    /// Refactor code
    Refactor,
    /// Fix bug
    Bugfix,
    /// Add documentation
    Documentation,
}

/// Summary of context for LLM
#[derive(Debug, Clone, Default)]
pub struct ContextSummary {
    /// Available imports
    pub imports: Vec<String>,
    
    /// Detected patterns
    pub patterns: Vec<String>,
    
    /// Conventions
    pub conventions: HashMap<String, String>,
    
    /// Related files
    pub related_files: Vec<String>,
}

impl ContextSummary {
    /// Build from analyzed context
    pub fn from_context(context: &AnalyzedContext) -> Self {
        let mut summary = Self::default();
        
        // Extract patterns
        for pattern in context.patterns() {
            summary.patterns.push(pattern.name.clone());
        }
        
        // Extract conventions
        summary.conventions.insert(
            "indent".to_string(),
            format!("{:?}", context.pattern_analysis.conventions.indent),
        );
        summary.conventions.insert(
            "quotes".to_string(),
            format!("{:?}", context.pattern_analysis.conventions.quotes),
        );
        
        summary
    }
}

/// Request for code correction
#[derive(Debug, Clone)]
pub struct CorrectionRequest {
    /// Original code
    pub original_code: String,
    
    /// Issues to fix
    pub issues: Vec<String>,
    
    /// Language
    pub language: Language,
    
    /// Additional context
    pub context: String,
}

impl CorrectionRequest {
    /// Create new correction request
    pub fn new(original_code: impl Into<String>, language: Language) -> Self {
        Self {
            original_code: original_code.into(),
            issues: Vec::new(),
            language,
            context: String::new(),
        }
    }

    /// Add issue to fix
    pub fn with_issue(mut self, issue: impl Into<String>) -> Self {
        self.issues.push(issue.into());
        self
    }

    /// Add context
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = context.into();
        self
    }
}

/// Response from code generation
#[derive(Debug, Clone)]
pub struct CodeResponse {
    /// Generated code
    pub code: String,
    
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    
    /// Explanation of generation
    pub explanation: String,
    
    /// Warnings or notes
    pub warnings: Vec<String>,
}

impl CodeResponse {
    /// Create new response
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            confidence: 1.0,
            explanation: String::new(),
            warnings: Vec::new(),
        }
    }

    /// With confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// With explanation
    pub fn with_explanation(mut self, explanation: impl Into<String>) -> Self {
        self.explanation = explanation.into();
        self
    }

    /// Add warning
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }
}

/// Stub LLM provider for testing and development
/// Uses template-based generation without actual LLM calls
pub struct StubLlmProvider {
    /// Provider name
    name: String,
}

impl StubLlmProvider {
    /// Create new stub provider
    pub fn new() -> Self {
        Self {
            name: "stub".to_string(),
        }
    }
}

impl Default for StubLlmProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmProvider for StubLlmProvider {
    fn generate_code(&self, request: &CodeRequest) -> Result<CodeResponse, ContextError> {
        let code = generate_stub_code(request);
        Ok(CodeResponse::new(code)
            .with_confidence(0.5)
            .with_explanation("Generated using template-based stub provider")
            .with_warning("Replace with actual LLM provider for production"))
    }

    fn correct_code(&self, request: &CorrectionRequest) -> Result<CodeResponse, ContextError> {
        // Stub correction just returns original with TODO comments
        let code = format!(
            "// TODO: Fix the following issues:\n{}\n\n{}",
            request.issues.iter().map(|i| format!("// - {}", i)).collect::<Vec<_>>().join("\n"),
            request.original_code
        );
        Ok(CodeResponse::new(code)
            .with_confidence(0.3)
            .with_explanation("Stub correction - issues marked as TODO"))
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Generate stub code based on request
fn generate_stub_code(request: &CodeRequest) -> String {
    match request.language {
        Language::Rust => generate_rust_stub(request),
        Language::TypeScript => generate_typescript_stub(request),
        Language::JavaScript => generate_javascript_stub(request),
        Language::Python => generate_python_stub(request),
        Language::Go => generate_go_stub(request),
        Language::PHP => generate_php_stub(request),
        _ => generate_generic_stub(request),
    }
}

fn generate_rust_stub(request: &CodeRequest) -> String {
    let name = extract_name(&request.description);
    let fn_name = to_snake_case(&name);
    
    match request.gen_type {
        GenerationIntent::Test => format!(
            r#"//! Tests for {}
//!
//! {}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_{}_basic() {{
        // TODO: Implement test
        // {}
        assert!(true);
    }}

    #[test]
    fn test_{}_edge_cases() {{
        // TODO: Implement edge case tests
        assert!(true);
    }}
}}
"#,
            name, request.description, fn_name, request.description, fn_name
        ),
        _ => format!(
            r#"//! {}
//!
//! {}

use std::error::Error;

/// {}
///
/// # Examples
///
/// ```
/// // TODO: Add examples
/// ```
pub fn {}() -> Result<(), Box<dyn Error>> {{
    // TODO: Implement {}
    // Patterns: {}
    todo!("Implementation pending")
}}
"#,
            name,
            request.description,
            request.description,
            fn_name,
            request.description,
            request.patterns.join(", ")
        ),
    }
}

fn generate_typescript_stub(request: &CodeRequest) -> String {
    let name = extract_name(&request.description);
    
    match request.gen_type {
        GenerationIntent::Test => format!(
            r#"/**
 * Tests for {}
 * {}
 */

import {{ describe, it, expect }} from 'vitest';
import {{ {} }} from './{}';

describe('{}', () => {{
  it('should work correctly', () => {{
    // TODO: Implement test
    expect(true).toBe(true);
  }});
}});
"#,
            name, request.description, to_camel_case(&name), to_kebab_case(&name), name
        ),
        _ => format!(
            r#"/**
 * {}
 * {}
 */

export interface {}Props {{
  // TODO: Define props
  id?: string;
}}

/**
 * {}
 */
export function {}(props: {}Props): void {{
  // TODO: Implement
  // Patterns: {}
  throw new Error('Not implemented');
}}
"#,
            name, request.description, name, request.description, 
            to_camel_case(&name), name, request.patterns.join(", ")
        ),
    }
}

fn generate_javascript_stub(request: &CodeRequest) -> String {
    let name = extract_name(&request.description);
    
    format!(
        r#"/**
 * {}
 * {}
 */

/**
 * {}
 * @returns {{void}}
 */
export function {}() {{
  // TODO: Implement
  throw new Error('Not implemented');
}}
"#,
        name, request.description, request.description, to_camel_case(&name)
    )
}

fn generate_python_stub(request: &CodeRequest) -> String {
    let name = extract_name(&request.description);
    let fn_name = to_snake_case(&name);
    
    format!(
        r#"""
{}
{}
"""

def {}():
    """
    {}
    
    Returns:
        TODO: Document return type
    
    Raises:
        NotImplementedError: Implementation pending
    """
    # TODO: Implement
    # Patterns: {}
    raise NotImplementedError()
"#,
        name, request.description, fn_name, request.description, request.patterns.join(", ")
    )
}

fn generate_go_stub(request: &CodeRequest) -> String {
    let name = extract_name(&request.description);
    let fn_name = to_pascal_case(&name);
    
    format!(
        r#"// {}
// {}
package main

import "errors"

// {} implements the required functionality.
// TODO: Implement
func {}() error {{
    // Patterns: {}
    return errors.New("not implemented")
}}
"#,
        name, request.description, fn_name, fn_name, request.patterns.join(", ")
    )
}

fn generate_php_stub(request: &CodeRequest) -> String {
    let name = extract_name(&request.description);
    
    format!(
        r#"<?php
/**
 * {}
 * {}
 */

declare(strict_types=1);

/**
 * {}
 */
function {}(): void
{{
    // TODO: Implement
    // Patterns: {}
    throw new \RuntimeException('Not implemented');
}}
"#,
        name, request.description, request.description, to_snake_case(&name), request.patterns.join(", ")
    )
}

fn generate_generic_stub(request: &CodeRequest) -> String {
    format!(
        "// {}\n// {}\n// TODO: Implement\n// Patterns: {}\n",
        extract_name(&request.description),
        request.description,
        request.patterns.join(", ")
    )
}

/// Extract a name from description
fn extract_name(description: &str) -> String {
    description.split_whitespace()
        .filter(|w| w.chars().next().map(|c| c.is_uppercase()).unwrap_or(false))
        .next()
        .unwrap_or_else(|| description.split_whitespace().next().unwrap_or("Generated"))
        .to_string()
}

/// Convert to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(c.to_lowercase().next().unwrap_or(c));
    }
    result
}

/// Convert to camelCase
fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    
    for (i, c) in s.chars().enumerate() {
        if c == '_' || c == '-' {
            capitalize_next = true;
            continue;
        }
        
        if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else if i == 0 {
            result.extend(c.to_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// Convert to PascalCase
fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    
    for c in s.chars() {
        if c == '_' || c == '-' {
            capitalize_next = true;
            continue;
        }
        
        if capitalize_next {
            result.extend(c.to_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

/// Convert to kebab-case
fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('-');
        }
        result.push(c.to_lowercase().next().unwrap_or(c));
    }
    result
}

/// Computer Use LLM Provider Configuration
#[derive(Debug, Clone)]
pub struct ComputerUseConfig {
    /// API endpoint URL
    pub api_url: String,
    /// API authentication key
    pub api_key: String,
    /// Model ID to use
    pub model_id: String,
    /// Maximum tokens to generate
    pub max_tokens: u32,
    /// Temperature for generation (0.0 - 1.0)
    pub temperature: f32,
}

impl Default for ComputerUseConfig {
    fn default() -> Self {
        Self {
            api_url: "http://127.0.0.1:8315".to_string(),
            api_key: "sk-LSF18xOTg4QqB0QA7Md7xFQ8DZdvXbvV".to_string(),
            model_id: "gemini-2.5-computer-use-preview-10-2025".to_string(),
            max_tokens: 4096,
            temperature: 0.2,
        }
    }
}

/// Computer Use LLM Provider - connects to local LLM API for code generation
pub struct ComputerUseLlmProvider {
    config: ComputerUseConfig,
}

impl ComputerUseLlmProvider {
    /// Create new provider with default config
    pub fn new() -> Self {
        Self {
            config: ComputerUseConfig::default(),
        }
    }

    /// Create new provider with custom config
    pub fn with_config(config: ComputerUseConfig) -> Self {
        Self { config }
    }

    /// Create new provider from environment variables
    pub fn from_env() -> Result<Self, ContextError> {
        let api_url = std::env::var("DOOZ_LLM_API_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8315".to_string());
        let api_key = std::env::var("DOOZ_LLM_API_KEY")
            .map_err(|_| ContextError::ConfigurationError {
                message: "DOOZ_LLM_API_KEY environment variable not set".to_string(),
            })?;
        let model_id = std::env::var("DOOZ_LLM_MODEL")
            .unwrap_or_else(|_| "gemini-2.5-computer-use-preview-10-2025".to_string());

        Ok(Self::with_config(ComputerUseConfig {
            api_url,
            api_key,
            model_id,
            max_tokens: 4096,
            temperature: 0.2,
        }))
    }

    /// Make API call to LLM
    fn call_llm(&self, prompt: &str) -> Result<String, ContextError> {
        let request_body = serde_json::json!({
            "model": self.config.model_id,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "max_tokens": self.config.max_tokens,
            "temperature": self.config.temperature,
        });

        let response = ureq::post(&format!("{}/v1/chat/completions", self.config.api_url))
            .set("Authorization", &format!("Bearer {}", self.config.api_key))
            .set("Content-Type", "application/json")
            .send_string(&request_body.to_string());

        match response {
            Ok(res) => {
                let text = res.into_string().map_err(|e| ContextError::NetworkError {
                    message: format!("Failed to read response: {}", e),
                })?;
                let parsed: serde_json::Value = serde_json::from_str(&text)
                    .map_err(|e| ContextError::NetworkError {
                        message: format!("Failed to parse response: {}", e),
                    })?;

                let content = parsed["choices"]
                    .get(0)
                    .and_then(|c| c["message"]["content"].as_str())
                    .ok_or_else(|| ContextError::NetworkError {
                        message: "Invalid response format from LLM API".to_string(),
                    })?;

                Ok(content.to_string())
            }
            Err(e) => Err(ContextError::NetworkError {
                message: format!("LLM API request failed: {}", e),
            }),
        }
    }

    /// Extract code from LLM response (handles markdown code blocks)
    fn extract_code(&self, response: &str) -> String {
        let mut code = response.to_string();

        // Try to extract from markdown code blocks
        if let Some(start) = response.find("```") {
            let after_start = &response[start + 3..];
            // Skip language identifier if present
            let content_start = if after_start.starts_with('\n') {
                1
            } else {
                let newline = after_start.find('\n').unwrap_or(0);
                newline + 1
            };

            if let Some(end) = after_start[content_start..].find("```") {
                code = after_start[content_start..content_start + end].to_string();
            }
        }

        code.trim().to_string()
    }
}

impl Default for ComputerUseLlmProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmProvider for ComputerUseLlmProvider {
    fn generate_code(&self, request: &CodeRequest) -> Result<CodeResponse, ContextError> {
        let prompt = request.to_prompt();
        let raw_response = self.call_llm(&prompt)?;
        let code = self.extract_code(&raw_response);

        Ok(CodeResponse::new(code)
            .with_confidence(0.9)
            .with_explanation("Generated using Computer Use LLM Provider"))
    }

    fn correct_code(&self, request: &CorrectionRequest) -> Result<CodeResponse, ContextError> {
        let prompt = format!(
            r#"Correct the following code based on these issues:

Original Code:
```
{}
```

Issues to fix:
{}

Context:
{}

Please provide the corrected code only, wrapped in a markdown code block with the appropriate language.
"#,
            request.original_code,
            request.issues.join("\n"),
            request.context
        );

        let raw_response = self.call_llm(&prompt)?;
        let code = self.extract_code(&raw_response);

        Ok(CodeResponse::new(code)
            .with_confidence(0.85)
            .with_explanation("Corrected using Computer Use LLM Provider"))
    }

    fn name(&self) -> &str {
        "computer-use"
    }
}

/// Factory for creating LLM providers
pub enum LlmProviderFactory {
    /// Stub provider for testing
    Stub,
    /// Computer Use provider for production
    ComputerUse,
}

impl LlmProviderFactory {
    /// Create a provider based on enum variant
    pub fn create(&self) -> Box<dyn LlmProvider> {
        match self {
            LlmProviderFactory::Stub => Box::new(StubLlmProvider::new()),
            LlmProviderFactory::ComputerUse => Box::new(ComputerUseLlmProvider::new()),
        }
    }

    /// Try to create ComputerUse provider from environment
    pub fn try_create_computer_use() -> Result<Box<dyn LlmProvider>, ContextError> {
        Ok(Box::new(ComputerUseLlmProvider::from_env()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_provider_generates_code() {
        let provider = StubLlmProvider::new();
        let request = CodeRequest::new("User authentication", "src/auth.rs", Language::Rust);
        
        let response = provider.generate_code(&request).unwrap();
        assert!(!response.code.is_empty());
        assert!(response.confidence > 0.0);
    }

    #[test]
    fn code_request_builds_prompt() {
        let request = CodeRequest::new("Create login function", "src/auth.rs", Language::Rust)
            .with_constraint("No external dependencies")
            .with_pattern("Repository pattern");
        
        let prompt = request.to_prompt();
        assert!(prompt.contains("login"));
        assert!(prompt.contains("No external dependencies"));
        assert!(prompt.contains("Repository pattern"));
    }

    #[test]
    fn stub_generates_typescript_test() {
        let provider = StubLlmProvider::new();
        let request = CodeRequest::new("UserService", "src/user.test.ts", Language::TypeScript)
            .with_intent(GenerationIntent::Test);
        
        let response = provider.generate_code(&request).unwrap();
        assert!(response.code.contains("describe"));
        assert!(response.code.contains("expect"));
    }

    #[test]
    fn case_conversions() {
        assert_eq!(to_snake_case("MyFunction"), "my_function");
        assert_eq!(to_camel_case("my_function"), "myFunction");
        assert_eq!(to_pascal_case("my_function"), "MyFunction");
        assert_eq!(to_kebab_case("MyFunction"), "my-function");
    }

    #[test]
    fn computer_use_config_default() {
        let config = ComputerUseConfig::default();
        assert_eq!(config.api_url, "http://127.0.0.1:8315");
        assert_eq!(config.model_id, "gemini-2.5-computer-use-preview-10-2025");
        assert_eq!(config.max_tokens, 4096);
        assert_eq!(config.temperature, 0.2);
    }

    #[test]
    fn computer_use_provider_creation() {
        let provider = ComputerUseLlmProvider::new();
        assert_eq!(provider.name(), "computer-use");
    }

    #[test]
    fn computer_use_provider_with_custom_config() {
        let config = ComputerUseConfig {
            api_url: "http://localhost:8080".to_string(),
            api_key: "test-key".to_string(),
            model_id: "test-model".to_string(),
            max_tokens: 2048,
            temperature: 0.5,
        };
        let provider = ComputerUseLlmProvider::with_config(config);
        assert_eq!(provider.name(), "computer-use");
    }

    #[test]
    fn code_extraction_from_markdown() {
        let provider = ComputerUseLlmProvider::new();
        let response = r#"
Here is the code you requested:

```rust
fn hello() {
    println!("Hello, World!");
}
```

Let me know if you need anything else!
"#;

        let code = provider.extract_code(response);
        assert!(code.contains("fn hello()"));
        assert!(code.contains("println!"));
        assert!(!code.contains("```"));
    }

    #[test]
    fn code_extraction_plain_text() {
        let provider = ComputerUseLlmProvider::new();
        let response = "fn hello() { return true; }";

        let code = provider.extract_code(response);
        assert_eq!(code, "fn hello() { return true; }");
    }

    #[test]
    fn llm_provider_factory_stub() {
        let provider = LlmProviderFactory::Stub.create();
        assert_eq!(provider.name(), "stub");
    }

    #[test]
    fn llm_provider_factory_computer_use() {
        let provider = LlmProviderFactory::ComputerUse.create();
        assert_eq!(provider.name(), "computer-use");
    }

    #[test]
    fn code_response_builder() {
        let response = CodeResponse::new("test code")
            .with_confidence(0.95)
            .with_explanation("Generated successfully")
            .with_warning("Check for edge cases");

        assert_eq!(response.code, "test code");
        assert_eq!(response.confidence, 0.95);
        assert_eq!(response.explanation, "Generated successfully");
        assert_eq!(response.warnings.len(), 1);
    }

    #[test]
    fn correction_request_builder() {
        let request = CorrectionRequest::new("old code", Language::Rust)
            .with_issue("Missing error handling")
            .with_issue("No validation")
            .with_context("This is a user authentication module");

        assert!(request.issues.contains(&"Missing error handling".to_string()));
        assert_eq!(request.issues.len(), 2);
    }
}
