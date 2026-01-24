//! Technical Specification Types

use serde::{Deserialize, Serialize};

/// Complete technical specification derived from a PRD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalSpec {
    pub id: String,
    pub title: String,
    pub description: String,
    pub entities: Vec<Entity>,
    pub apis: Vec<ApiEndpoint>,
    pub ui_flows: Vec<UiFlow>,
    pub edge_cases: Vec<String>,
    pub assumptions: Vec<String>,
    pub acceptance_criteria: Vec<String>,
}

impl TechnicalSpec {
    pub fn new(title: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            description: String::new(),
            entities: Vec::new(),
            apis: Vec::new(),
            ui_flows: Vec::new(),
            edge_cases: Vec::new(),
            assumptions: Vec::new(),
            acceptance_criteria: Vec::new(),
        }
    }
}

/// Data entity in the spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub entity_type: EntityType,
    pub attributes: Vec<Attribute>,
    pub relationships: Vec<Relationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Model,
    Service,
    Component,
    Module,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub name: String,
    pub data_type: String,
    pub required: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub target: String,
    pub relation_type: String,
    pub description: String,
}

/// API endpoint in the spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpoint {
    pub method: HttpMethod,
    pub path: String,
    pub description: String,
    pub request_body: Option<serde_json::Value>,
    pub response: Option<serde_json::Value>,
    pub auth_required: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

/// UI flow in the spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiFlow {
    pub name: String,
    pub screens: Vec<Screen>,
    pub navigation: Vec<Navigation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Screen {
    pub name: String,
    pub components: Vec<String>,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Navigation {
    pub from: String,
    pub to: String,
    pub trigger: String,
}
