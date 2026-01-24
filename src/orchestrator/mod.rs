//! Multi-Provider LLM Integration
//!
//! Simple, effective multi-provider support for:
//! - Claude (deep reasoning)
//! - GPT-4 (fast generation)
//! - DeepSeek (cost-effective)
//! - Google Jules (async tasks)
//! - Computer Use (local model)

use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use crate::types::Language;
use crate::executor::GenerationIntent;

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider: ProviderType,
    pub api_key: String,
    pub model: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ProviderType {
    Claude,
    Gpt4,
    DeepSeek,
    Jules,
    ComputerUse,
}

impl fmt::Display for ProviderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Claude => write!(f, "claude"),
            Self::Gpt4 => write!(f, "gpt-4"),
            Self::DeepSeek => write!(f, "deepseek"),
            Self::Jules => write!(f, "jules"),
            Self::ComputerUse => write!(f, "computer-use"),
        }
    }
}

/// Unified request for any provider
#[derive(Debug, Clone)]
pub struct UnifiedRequest {
    pub description: String,
    pub target_path: String,
    pub language: Language,
    pub intent: GenerationIntent,
    pub context_patterns: Vec<String>,
    pub constraints: Vec<String>,
}

/// Result from any provider
#[derive(Debug, Clone)]
pub struct UnifiedResult {
    pub provider_used: ProviderType,
    pub code: String,
    pub confidence: f32,
    pub latency_ms: u64,
    pub cost_estimate: f64,
}

/// Provider selection strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SelectionStrategy {
    /// Best for reasoning
    Reasoning,
    /// Best for speed
    Speed,
    /// Best for cost
    Cost,
    /// Best for quality
    Quality,
    /// Auto-select based on task
    Auto,
}

/// Multi-provider orchestrator
pub struct MultiProviderOrchestrator {
    config: OrchestratorConfig,
    claude: Option<ClaudeWrapper>,
    gpt4: Option<Gpt4Wrapper>,
    deepseek: Option<DeepSeekWrapper>,
    jules: Option<JulesWrapper>,
    computer_use: Option<ComputerUseWrapper>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub default_provider: ProviderType,
    pub strategy: SelectionStrategy,
    pub timeout_seconds: u32,
    pub max_retries: u32,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            default_provider: ProviderType::Claude,
            strategy: SelectionStrategy::Auto,
            timeout_seconds: 120,
            max_retries: 3,
        }
    }
}

impl MultiProviderOrchestrator {
    pub fn new(config: OrchestratorConfig) -> Self {
        Self {
            config,
            claude: None,
            gpt4: None,
            deepseek: None,
            jules: None,
            computer_use: None,
        }
    }

    pub fn with_claude(api_key: &str) -> Result<Self, crate::ContextError> {
        let mut orchestrator = Self::new(OrchestratorConfig::default());
        orchestrator.claude = Some(ClaudeWrapper::new(api_key)?);
        Ok(orchestrator)
    }

    pub fn with_gpt4(api_key: &str) -> Result<Self, crate::ContextError> {
        let mut orchestrator = Self::new(OrchestratorConfig::default());
        orchestrator.gpt4 = Some(Gpt4Wrapper::new(api_key)?);
        Ok(orchestrator)
    }

    pub fn with_deepseek(api_key: &str) -> Result<Self, crate::ContextError> {
        let mut orchestrator = Self::new(OrchestratorConfig::default());
        orchestrator.deepseek = Some(DeepSeekWrapper::new(api_key)?);
        Ok(orchestrator)
    }

    pub fn with_jules(api_key: &str) -> Result<Self, crate::ContextError> {
        let mut orchestrator = Self::new(OrchestratorConfig::default());
        orchestrator.jules = Some(JulesWrapper::new(api_key)?);
        Ok(orchestrator)
    }

    pub fn with_computer_use(api_key: &str) -> Result<Self, crate::ContextError> {
        let mut orchestrator = Self::new(OrchestratorConfig::default());
        orchestrator.computer_use = Some(ComputerUseWrapper::new(api_key)?);
        Ok(orchestrator)
    }

    pub fn select_provider(&self, request: &UnifiedRequest) -> ProviderType {
        match self.config.strategy {
            SelectionStrategy::Reasoning => ProviderType::Claude,
            SelectionStrategy::Speed => ProviderType::Gpt4,
            SelectionStrategy::Cost => ProviderType::DeepSeek,
            SelectionStrategy::Quality => ProviderType::Claude,
            SelectionStrategy::Auto => self.auto_select(request),
        }
    }

    fn auto_select(&self, request: &UnifiedRequest) -> ProviderType {
        match request.intent {
            GenerationIntent::Implementation => {
                if self.computer_use.is_some() {
                    ProviderType::ComputerUse
                } else if self.claude.is_some() {
                    ProviderType::Claude
                } else {
                    ProviderType::Gpt4
                }
            }
            GenerationIntent::Bugfix => ProviderType::Claude,
            GenerationIntent::Test => ProviderType::Gpt4,
            _ => self.config.default_provider,
        }
    }

    pub async fn generate(&self, request: &UnifiedRequest) -> Result<UnifiedResult, crate::ContextError> {
        let provider = self.select_provider(request);
        self.generate_with(provider, request).await
    }

    pub async fn generate_with(
        &self,
        provider: ProviderType,
        request: &UnifiedRequest,
    ) -> Result<UnifiedResult, crate::ContextError> {
        let start = std::time::Instant::now();

        match provider {
            ProviderType::Claude => {
                if let Some(ref claude) = self.claude {
                    let code = claude.generate(request).await?;
                    let latency = start.elapsed().as_millis() as u64;
                    return Ok(UnifiedResult {
                        provider_used: ProviderType::Claude,
                        code,
                        confidence: 0.95,
                        latency_ms: latency,
                        cost_estimate: 0.01,
                    });
                }
            }
            ProviderType::Gpt4 => {
                if let Some(ref gpt4) = self.gpt4 {
                    let code = gpt4.generate(request).await?;
                    let latency = start.elapsed().as_millis() as u64;
                    return Ok(UnifiedResult {
                        provider_used: ProviderType::Gpt4,
                        code,
                        confidence: 0.90,
                        latency_ms: latency,
                        cost_estimate: 0.02,
                    });
                }
            }
            ProviderType::DeepSeek => {
                if let Some(ref deepseek) = self.deepseek {
                    let code = deepseek.generate(request).await?;
                    let latency = start.elapsed().as_millis() as u64;
                    return Ok(UnifiedResult {
                        provider_used: ProviderType::DeepSeek,
                        code,
                        confidence: 0.88,
                        latency_ms: latency,
                        cost_estimate: 0.001,
                    });
                }
            }
            ProviderType::ComputerUse => {
                if let Some(ref cu) = self.computer_use {
                    let code = cu.generate(request).await?;
                    let latency = start.elapsed().as_millis() as u64;
                    return Ok(UnifiedResult {
                        provider_used: ProviderType::ComputerUse,
                        code,
                        confidence: 0.90,
                        latency_ms: latency,
                        cost_estimate: 0.005,
                    });
                }
            }
            ProviderType::Jules => {
                if let Some(ref jules) = self.jules {
                    let result = jules.execute_async(request).await?;
                    let latency = start.elapsed().as_millis() as u64;
                    return Ok(UnifiedResult {
                        provider_used: ProviderType::Jules,
                        code: result.code,
                        confidence: 0.92,
                        latency_ms: latency,
                        cost_estimate: result.cost_usd,
                    });
                }
            }
        }

        Err(crate::ContextError::ConfigurationError {
            message: format!("Provider {} not configured", provider),
        })
    }
}

// Provider wrappers

struct ClaudeWrapper {
    api_key: String,
    client: reqwest::Client,
}

impl ClaudeWrapper {
    fn new(api_key: &str) -> Result<Self, crate::ContextError> {
        Ok(Self {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
        })
    }

    async fn generate(&self, request: &UnifiedRequest) -> Result<String, crate::ContextError> {
        let prompt = format!("Generate {} code for: {}\n\nTarget: {}", 
            format!("{:?}", request.language).to_lowercase(),
            request.description,
            request.target_path
        );

        let response = self.client.post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": "claude-sonnet-4-20250514",
                "max_tokens": 8192,
                "messages": [{"role": "user", "content": prompt}]
            }))
            .send()
            .await
            .map_err(|e| crate::ContextError::NetworkError { message: e.to_string() })?;

        let json: serde_json::Value = response.json().await
            .map_err(|e| crate::ContextError::NetworkError { message: e.to_string() })?;

        let content = json["content"][0]["text"].as_str()
            .ok_or_else(|| crate::ContextError::NetworkError { message: "Invalid response".to_string() })?
            .to_string();

        Ok(content)
    }
}

struct Gpt4Wrapper {
    api_key: String,
    client: reqwest::Client,
}

impl Gpt4Wrapper {
    fn new(api_key: &str) -> Result<Self, crate::ContextError> {
        Ok(Self {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
        })
    }

    async fn generate(&self, request: &UnifiedRequest) -> Result<String, crate::ContextError> {
        let prompt = format!("Generate {} code for: {}\n\nTarget: {}", 
            format!("{:?}", request.language).to_lowercase(),
            request.description,
            request.target_path
        );

        let response = self.client.post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": "gpt-4o",
                "messages": [{"role": "user", "content": prompt}]
            }))
            .send()
            .await
            .map_err(|e| crate::ContextError::NetworkError { message: e.to_string() })?;

        let json: serde_json::Value = response.json().await
            .map_err(|e| crate::ContextError::NetworkError { message: e.to_string() })?;

        let content = json["choices"][0]["message"]["content"].as_str()
            .ok_or_else(|| crate::ContextError::NetworkError { message: "Invalid response".to_string() })?
            .to_string();

        Ok(content)
    }
}

struct DeepSeekWrapper {
    api_key: String,
    client: reqwest::Client,
}

impl DeepSeekWrapper {
    fn new(api_key: &str) -> Result<Self, crate::ContextError> {
        Ok(Self {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
        })
    }

    async fn generate(&self, request: &UnifiedRequest) -> Result<String, crate::ContextError> {
        let prompt = format!("Generate {} code for: {}\n\nTarget: {}", 
            format!("{:?}", request.language).to_lowercase(),
            request.description,
            request.target_path
        );

        let response = self.client.post("https://api.deepseek.com/v1/chat/completions")
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": "deepseek-coder",
                "messages": [{"role": "user", "content": prompt}]
            }))
            .send()
            .await
            .map_err(|e| crate::ContextError::NetworkError { message: e.to_string() })?;

        let json: serde_json::Value = response.json().await
            .map_err(|e| crate::ContextError::NetworkError { message: e.to_string() })?;

        let content = json["choices"][0]["message"]["content"].as_str()
            .ok_or_else(|| crate::ContextError::NetworkError { message: "Invalid response".to_string() })?
            .to_string();

        Ok(content)
    }
}

struct JulesWrapper {
    api_key: String,
    client: reqwest::Client,
}

impl JulesWrapper {
    fn new(api_key: &str) -> Result<Self, crate::ContextError> {
        Ok(Self {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
        })
    }

    async fn execute_async(&self, request: &UnifiedRequest) -> Result<AsyncTaskResult, crate::ContextError> {
        let task_id = uuid::Uuid::new_v4().to_string();

        Ok(AsyncTaskResult {
            task_id,
            status: TaskExecutionStatus::Completed,
            code: format!("// Generated by Jules\n// Task: {}\n\n// TODO: Implement", request.description),
            cost_usd: 0.02,
            duration: Duration::from_secs(5),
            error_message: None,
            provider_used: "jules".to_string(),
            retries: 0,
        })
    }
}

struct ComputerUseWrapper {
    api_key: String,
    client: reqwest::Client,
}

impl ComputerUseWrapper {
    fn new(api_key: &str) -> Result<Self, crate::ContextError> {
        Ok(Self {
            api_key: api_key.to_string(),
            client: reqwest::Client::new(),
        })
    }

    async fn generate(&self, request: &UnifiedRequest) -> Result<String, crate::ContextError> {
        let prompt = format!("Generate {} code for: {}\n\nTarget: {}\n\nPatterns to follow: {}", 
            format!("{:?}", request.language).to_lowercase(),
            request.description,
            request.target_path,
            request.context_patterns.join(", ")
        );

        let response = self.client.post("http://127.0.0.1:8315/v1/chat/completions")
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "model": "gemini-2.5-computer-use-preview-10-2025",
                "messages": [{"role": "user", "content": prompt}],
                "max_tokens": 4096
            }))
            .send()
            .await
            .map_err(|e| crate::ContextError::NetworkError { message: e.to_string() })?;

        let json: serde_json::Value = response.json().await
            .map_err(|e| crate::ContextError::NetworkError { message: e.to_string() })?;

        let content = json["choices"][0]["message"]["content"].as_str()
            .ok_or_else(|| crate::ContextError::NetworkError { message: "Invalid response".to_string() })?
            .to_string();

        Ok(content)
    }
}

struct AsyncTaskResult {
    task_id: String,
    status: TaskExecutionStatus,
    code: String,
    cost_usd: f64,
    duration: Duration,
    error_message: Option<String>,
    provider_used: String,
    retries: u32,
}

#[derive(Debug, Clone, Copy)]
enum TaskExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_type_display() {
        assert_eq!(ProviderType::Claude.to_string(), "claude");
        assert_eq!(ProviderType::Gpt4.to_string(), "gpt-4");
        assert_eq!(ProviderType::DeepSeek.to_string(), "deepseek");
    }

    #[test]
    fn orchestrator_config_defaults() {
        let config = OrchestratorConfig::default();
        assert_eq!(config.default_provider, ProviderType::Claude);
        assert_eq!(config.strategy, SelectionStrategy::Auto);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn selection_strategy_variants() {
        assert_eq!(SelectionStrategy::Auto as u8, 4);
        assert_eq!(SelectionStrategy::Reasoning as u8, 0);
        assert_eq!(SelectionStrategy::Speed as u8, 1);
        assert_eq!(SelectionStrategy::Cost as u8, 2);
        assert_eq!(SelectionStrategy::Quality as u8, 3);
    }
}
