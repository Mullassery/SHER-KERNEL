use sher_common::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: ObjectId,
    pub actor: ObjectId,
    pub action: String,
    pub timestamp: u64,
    pub details: String,
    pub result: bool,
}

#[derive(Debug, Clone, Default)]
pub struct AuditLog {
    pub events: Vec<AuditEvent>,
}

impl AuditLog {
    pub fn log(&mut self, actor: ObjectId, action: impl Into<String>, result: bool) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.events.push(AuditEvent {
            id: ObjectId::new(),
            actor,
            action: action.into(),
            timestamp,
            details: String::new(),
            result,
        });
    }
}
