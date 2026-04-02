//! Brain MCP Context Integration for dooz-code
//!
//! Queries Dooz Brain's organizational memory before code execution
//! to enrich the LLM prompt with relevant context:
//! - Coding standards and conventions
//! - Architecture decisions
//! - Past implementation patterns
//! - Known issues and workarounds

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainMcpConfig {
    pub enabled: bool,
    pub base_url: String,
    pub scope_id: Option<String>,
    pub timeout_secs: u64,
}

impl Default for BrainMcpConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_url: "http://localhost:1420".to_string(),
            scope_id: None,
            timeout_secs: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainContext {
    pub coding_standards: Vec<String>,
    pub architecture_decisions: Vec<String>,
    pub past_patterns: Vec<String>,
    pub known_issues: Vec<String>,
    pub raw_memories: Vec<BrainMemory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrainMemory {
    pub id: String,
    pub title: String,
    pub content: String,
    pub confidence: f64,
    pub decay_state: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BrainResponse {
    success: bool,
    results: Option<Vec<BrainResult>>,
    error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BrainResult {
    memory: BrainMemoryRaw,
    score: f64,
    ranking_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BrainMemoryRaw {
    id: String,
    title: String,
    content: String,
    confidence: f64,
    decay_state: String,
    updated_at: String,
}

pub struct BrainMcpClient {
    config: BrainMcpConfig,
    client: reqwest::blocking::Client,
}

impl BrainMcpClient {
    pub fn new(config: BrainMcpConfig) -> Self {
        let timeout = std::time::Duration::from_secs(config.timeout_secs);
        Self {
            config,
            client: reqwest::blocking::Client::builder()
                .timeout(timeout)
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn from_env() -> Self {
        let enabled = std::env::var("DOOZ_BRAIN_ENABLED")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let base_url = std::env::var("DOOZ_BRAIN_URL")
            .unwrap_or_else(|_| "http://localhost:1420".to_string());
        let scope_id = std::env::var("DOOZ_BRAIN_SCOPE_ID").ok();

        Self::new(BrainMcpConfig {
            enabled,
            base_url,
            scope_id,
            ..Default::default()
        })
    }

    pub fn query_context(&self, query: &str) -> Result<BrainContext, String> {
        if !self.config.enabled {
            return Ok(BrainContext::default());
        }

        let scope_id = self.config.scope_id.clone()
            .or_else(|| std::env::var("DOOZ_BRAIN_SCOPE_ID").ok())
            .ok_or("Brain scope_id not configured")?;

        let mut context = BrainContext::default();

        for topic in &["coding standards", "architecture decisions", "implementation patterns", "known issues"] {
            let full_query = format!("{} {}", query, topic);
            match self.query_memories(&scope_id, &full_query, 5) {
                Ok(memories) => {
                    for m in &memories {
                        match topic.as_str() {
                            "coding standards" => context.coding_standards.push(format!("{}: {}", m.title, m.content)),
                            "architecture decisions" => context.architecture_decisions.push(format!("{}: {}", m.title, m.content)),
                            "implementation patterns" => context.past_patterns.push(format!("{}: {}", m.title, m.content)),
                            "known issues" => context.known_issues.push(format!("{}: {}", m.title, m.content)),
                            _ => {}
                        }
                        context.raw_memories.push(m.clone());
                    }
                }
                Err(e) => {
                    tracing::warn!("Brain MCP query failed for '{}': {}", topic, e);
                }
            }
        }

        Ok(context)
    }

    fn query_memories(&self, scope_id: &str, query: &str, limit: usize) -> Result<Vec<BrainMemory>, String> {
        let url = format!(
            "{}/mcp/query?scope_id={}&query={}&limit={}",
            self.config.base_url,
            urlencoding::encode(scope_id),
            urlencoding::encode(query),
            limit
        );

        let resp = self.client.get(&url).send()
            .map_err(|e| format!("HTTP error: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Brain returned {}", resp.status()));
        }

        let body: BrainResponse = resp.json()
            .map_err(|e| format!("Parse error: {}", e))?;

        if !body.success {
            return Err(body.error.unwrap_or_else(|| "Unknown error".to_string()));
        }

        let memories = body.results.unwrap_or_default()
            .into_iter()
            .map(|r| BrainMemory {
                id: r.memory.id,
                title: r.memory.title,
                content: r.memory.content,
                confidence: r.memory.confidence,
                decay_state: r.memory.decay_state,
                updated_at: r.memory.updated_at,
            })
            .collect();

        Ok(memories)
    }

    pub fn format_prompt_context(&self, context: &BrainContext) -> String {
        let mut sections = Vec::new();

        if !context.coding_standards.is_empty() {
            sections.push(format!(
                "## Coding Standards\n{}\n",
                context.coding_standards.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")
            ));
        }

        if !context.architecture_decisions.is_empty() {
            sections.push(format!(
                "## Architecture Decisions\n{}\n",
                context.architecture_decisions.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")
            ));
        }

        if !context.past_patterns.is_empty() {
            sections.push(format!(
                "## Past Implementation Patterns\n{}\n",
                context.past_patterns.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")
            ));
        }

        if !context.known_issues.is_empty() {
            sections.push(format!(
                "## Known Issues\n{}\n",
                context.known_issues.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n")
            ));
        }

        if sections.is_empty() {
            "No organizational context found in Brain.\n".to_string()
        } else {
            format!(
                "# Organizational Context (from Dooz Brain)\n\n{}\n",
                sections.join("\n")
            )
        }
    }
}

impl Default for BrainContext {
    fn default() -> Self {
        Self {
            coding_standards: Vec::new(),
            architecture_decisions: Vec::new(),
            past_patterns: Vec::new(),
            known_issues: Vec::new(),
            raw_memories: Vec::new(),
        }
    }
}
