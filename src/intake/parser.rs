//! PRD Parser - heuristic extraction

use super::spec::*;
use super::IntakeError;
use regex::Regex;

/// Heuristic PRD parser
pub struct PrdParser {
    entity_patterns: Vec<Regex>,
    api_patterns: Vec<Regex>,
}

impl PrdParser {
    pub fn new() -> Self {
        Self {
            entity_patterns: vec![
                Regex::new(r"(?i)\b(user|customer|account|product|order|item|cart|payment)\b").unwrap(),
                Regex::new(r"(?i)\b(\w+)\s+model\b").unwrap(),
                Regex::new(r"(?i)\b(\w+)\s+entity\b").unwrap(),
            ],
            api_patterns: vec![
                Regex::new(r"(?i)(create|add|new)\s+(\w+)").unwrap(),
                Regex::new(r"(?i)(get|fetch|list|retrieve)\s+(\w+)").unwrap(),
                Regex::new(r"(?i)(update|modify|edit)\s+(\w+)").unwrap(),
                Regex::new(r"(?i)(delete|remove)\s+(\w+)").unwrap(),
            ],
        }
    }

    /// Parse PRD text into a technical spec
    pub fn parse(&self, prd: &str) -> Result<TechnicalSpec, IntakeError> {
        if prd.trim().is_empty() {
            return Err(IntakeError::InvalidPrd("Empty PRD".to_string()));
        }

        let title = self.extract_title(prd);
        let mut spec = TechnicalSpec::new(&title);
        
        spec.description = self.extract_description(prd);
        spec.entities = self.extract_entities(prd);
        spec.apis = self.extract_apis(prd);
        spec.edge_cases = self.extract_edge_cases(prd);
        spec.assumptions = self.extract_assumptions(prd);
        spec.acceptance_criteria = self.extract_criteria(prd);

        Ok(spec)
    }

    fn extract_title(&self, prd: &str) -> String {
        // Take first line or first 50 chars
        prd.lines()
            .next()
            .map(|l| l.trim().trim_start_matches('#').trim())
            .filter(|l| !l.is_empty())
            .unwrap_or("Untitled Feature")
            .chars()
            .take(100)
            .collect()
    }

    fn extract_description(&self, prd: &str) -> String {
        prd.lines()
            .skip(1)
            .take(5)
            .collect::<Vec<_>>()
            .join(" ")
            .chars()
            .take(500)
            .collect()
    }

    fn extract_entities(&self, prd: &str) -> Vec<Entity> {
        let mut entities = Vec::new();
        let mut found: std::collections::HashSet<String> = std::collections::HashSet::new();

        for pattern in &self.entity_patterns {
            for cap in pattern.captures_iter(prd) {
                if let Some(name) = cap.get(1) {
                    let name_str = name.as_str().to_lowercase();
                    if !found.contains(&name_str) {
                        found.insert(name_str.clone());
                        entities.push(Entity {
                            name: capitalize(&name_str),
                            entity_type: EntityType::Model,
                            attributes: vec![
                                Attribute {
                                    name: "id".to_string(),
                                    data_type: "UUID".to_string(),
                                    required: true,
                                    description: "Primary identifier".to_string(),
                                },
                            ],
                            relationships: Vec::new(),
                        });
                    }
                }
            }
        }

        entities
    }

    fn extract_apis(&self, prd: &str) -> Vec<ApiEndpoint> {
        let mut apis = Vec::new();

        for pattern in &self.api_patterns {
            for cap in pattern.captures_iter(prd) {
                if let (Some(action), Some(resource)) = (cap.get(1), cap.get(2)) {
                    let action_str = action.as_str().to_lowercase();
                    let resource_str = resource.as_str().to_lowercase();
                    
                    let (method, path) = match action_str.as_str() {
                        "create" | "add" | "new" => (HttpMethod::Post, format!("/api/{}", pluralize(&resource_str))),
                        "get" | "fetch" | "retrieve" => (HttpMethod::Get, format!("/api/{}/:id", pluralize(&resource_str))),
                        "list" => (HttpMethod::Get, format!("/api/{}", pluralize(&resource_str))),
                        "update" | "modify" | "edit" => (HttpMethod::Put, format!("/api/{}/:id", pluralize(&resource_str))),
                        "delete" | "remove" => (HttpMethod::Delete, format!("/api/{}/:id", pluralize(&resource_str))),
                        _ => continue,
                    };

                    apis.push(ApiEndpoint {
                        method,
                        path,
                        description: format!("{} {}", action_str, resource_str),
                        request_body: None,
                        response: None,
                        auth_required: true,
                    });
                }
            }
        }

        apis
    }

    fn extract_edge_cases(&self, prd: &str) -> Vec<String> {
        let mut cases = Vec::new();
        let prd_lower = prd.to_lowercase();
        
        if prd_lower.contains("error") || prd_lower.contains("fail") {
            cases.push("Handle error states gracefully".to_string());
        }
        if prd_lower.contains("empty") || prd_lower.contains("null") {
            cases.push("Handle empty/null inputs".to_string());
        }
        if prd_lower.contains("concurrent") || prd_lower.contains("parallel") {
            cases.push("Handle concurrent access".to_string());
        }
        if prd_lower.contains("timeout") {
            cases.push("Handle timeout scenarios".to_string());
        }
        
        if cases.is_empty() {
            cases.push("Handle invalid input".to_string());
            cases.push("Handle network failures".to_string());
        }
        
        cases
    }

    fn extract_assumptions(&self, prd: &str) -> Vec<String> {
        let mut assumptions = Vec::new();
        
        if !prd.to_lowercase().contains("auth") {
            assumptions.push("Authentication is handled separately".to_string());
        }
        if !prd.to_lowercase().contains("database") {
            assumptions.push("Database schema will be auto-migrated".to_string());
        }
        
        assumptions.push("API follows REST conventions".to_string());
        assumptions
    }

    fn extract_criteria(&self, prd: &str) -> Vec<String> {
        let mut criteria = Vec::new();
        
        // Look for explicit criteria
        let lines: Vec<&str> = prd.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.to_lowercase().contains("accept") || line.to_lowercase().contains("criteria") {
                // Take following lines that start with - or *
                for next_line in lines.iter().skip(i + 1) {
                    let trimmed = next_line.trim();
                    if trimmed.starts_with('-') || trimmed.starts_with('*') {
                        criteria.push(trimmed.trim_start_matches(['-', '*', ' ']).to_string());
                    } else if !trimmed.is_empty() && !trimmed.starts_with('#') {
                        break;
                    }
                }
            }
        }
        
        // Default criteria if none found
        if criteria.is_empty() {
            criteria.push("Feature works as described in PRD".to_string());
            criteria.push("All tests pass".to_string());
            criteria.push("No regressions in existing functionality".to_string());
        }
        
        criteria
    }
}

impl Default for PrdParser {
    fn default() -> Self {
        Self::new()
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().chain(chars).collect(),
    }
}

fn pluralize(s: &str) -> String {
    if s.ends_with('s') {
        s.to_string()
    } else if s.ends_with('y') {
        format!("{}ies", &s[..s.len()-1])
    } else {
        format!("{}s", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_prd() {
        let parser = PrdParser::new();
        let prd = "# User Authentication\n\nCreate user login and registration.";
        
        let spec = parser.parse(prd).unwrap();
        assert_eq!(spec.title, "User Authentication");
        assert!(!spec.entities.is_empty());
    }
}
