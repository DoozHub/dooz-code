//! Inter-Agent Communication
//!
//! Message passing system for coordination between dooz-* agents.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Message envelope for agent communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: String,
    pub message_type: MessageType,
    pub from: String,
    pub to: String,
    pub payload: MessagePayload,
    pub timestamp: SystemTime,
    pub correlation_id: Option<String>,
}

/// Types of messages between agents
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageType {
    TaskRequest,
    TaskResponse,
    StatusUpdate,
    Heartbeat,
    HelpRequest,
    CapabilityAnnounce,
    Error,
    Shutdown,
    Ack,
}

/// Message payload - the actual content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    Task {
        task_id: String,
        task_type: String,
        payload: serde_json::Value,
    },
    Result {
        task_id: String,
        status: String,
        output: String,
        metadata: serde_json::Value,
    },
    Status {
        agent: String,
        state: String,
        load: f32,
    },
    Heartbeat {
        agent: String,
        uptime_seconds: u64,
    },
    Help {
        request_id: String,
        description: String,
        context: serde_json::Value,
    },
    Capabilities {
        agent: String,
        capabilities: Vec<String>,
    },
    Error {
        code: String,
        message: String,
        recoverable: bool,
    },
    Empty,
}

impl AgentMessage {
    /// Create a new task request message
    pub fn task_request(from: String, to: String, task_id: String, task_type: String, payload: serde_json::Value) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message_type: MessageType::TaskRequest,
            from,
            to,
            payload: MessagePayload::Task { task_id, task_type, payload },
            timestamp: SystemTime::now(),
            correlation_id: None,
        }
    }

    /// Create a task response message
    pub fn task_response(from: String, to: String, task_id: String, status: String, output: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message_type: MessageType::TaskResponse,
            from,
            to,
            payload: MessagePayload::Result {
                task_id,
                status,
                output,
                metadata: serde_json::json!({}),
            },
            timestamp: SystemTime::now(),
            correlation_id: None,
        }
    }

    /// Create a heartbeat message
    pub fn heartbeat(agent: String, uptime_seconds: u64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message_type: MessageType::Heartbeat,
            from: agent.clone(),
            to: "agency".to_string(),
            payload: MessagePayload::Heartbeat { agent, uptime_seconds },
            timestamp: SystemTime::now(),
            correlation_id: None,
        }
    }

    /// Create a shutdown signal
    pub fn shutdown(from: String, reason: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            message_type: MessageType::Shutdown,
            from,
            to: "*".to_string(),
            payload: MessagePayload::Error {
                code: "SHUTDOWN".to_string(),
                message: reason,
                recoverable: false,
            },
            timestamp: SystemTime::now(),
            correlation_id: None,
        }
    }
}

/// Event bus for agency-wide notifications
#[derive(Clone, Debug)]
pub struct EventBus {
    tx: tokio::sync::broadcast::Sender<AgencyEvent>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            tx: tokio::sync::broadcast::channel(100).0,
        }
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<AgencyEvent> {
        self.tx.subscribe()
    }

    /// Publish an event
    pub fn publish(&self, event: AgencyEvent) {
        let _ = self.tx.send(event);
    }
}

/// Agency-wide events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgencyEvent {
    AgentRegistered { agent: String },
    AgentUnregistered { agent: String },
    AgentStatusChanged { agent: String, status: String },
    TaskStarted { task_id: String, agent: String },
    TaskCompleted { task_id: String, agent: String },
    TaskFailed { task_id: String, agent: String, reason: String },
    HealthCheck { healthy: bool },
}
