//! Status Signals
//!
//! Emits status signals and maintains audit trail.
//! All emissions are append-only for traceability.

mod status;
mod audit;

pub use status::*;
pub use audit::*;

/// Status emitter component
pub struct StatusEmitter {
    /// Audit log
    audit: AuditLog,
    
    /// Subscribers
    subscribers: Vec<Box<dyn StatusSubscriber>>,
}

impl StatusEmitter {
    /// Create new emitter
    pub fn new() -> Self {
        Self {
            audit: AuditLog::new(),
            subscribers: Vec::new(),
        }
    }

    /// Emit status update
    pub fn emit(&mut self, status: Status) {
        self.audit.append(status.clone());
        
        for subscriber in &self.subscribers {
            subscriber.on_status(&status);
        }
    }

    /// Add subscriber
    pub fn subscribe(&mut self, subscriber: Box<dyn StatusSubscriber>) {
        self.subscribers.push(subscriber);
    }

    /// Get audit log
    pub fn audit(&self) -> &AuditLog {
        &self.audit
    }
}

impl Default for StatusEmitter {
    fn default() -> Self {
        Self::new()
    }
}

/// Status subscriber trait
pub trait StatusSubscriber: Send + Sync {
    /// Called when status is emitted
    fn on_status(&self, status: &Status);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emitter_creation() {
        let emitter = StatusEmitter::new();
        assert!(emitter.audit.is_empty());
    }

    #[test]
    fn emit_status() {
        let mut emitter = StatusEmitter::new();
        emitter.emit(Status::started("pkg-001"));
        
        assert!(!emitter.audit.is_empty());
        assert_eq!(emitter.audit.len(), 1);
    }
}
