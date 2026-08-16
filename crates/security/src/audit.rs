use serde::{Deserialize, Serialize};
use sher_common::ObjectId;

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

    pub fn failed_events(&self) -> Vec<&AuditEvent> {
        self.events.iter().filter(|e| !e.result).collect()
    }

    pub fn events_by_actor(&self, actor: ObjectId) -> Vec<&AuditEvent> {
        self.events.iter().filter(|e| e.actor == actor).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_appends_event_with_timestamp() {
        let mut log = AuditLog::default();
        let actor = ObjectId::new();
        log.log(actor, "grant_capability", true);
        assert_eq!(log.events.len(), 1);
        assert_eq!(log.events[0].actor, actor);
        assert_eq!(log.events[0].action, "grant_capability");
        assert!(log.events[0].result);
    }

    #[test]
    fn failed_events_filters_correctly() {
        let mut log = AuditLog::default();
        let actor = ObjectId::new();
        log.log(actor, "allowed_op", true);
        log.log(actor, "denied_op", false);
        let failed = log.failed_events();
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].action, "denied_op");
    }

    #[test]
    fn events_by_actor_filters_correctly() {
        let mut log = AuditLog::default();
        let a = ObjectId::new();
        let b = ObjectId::new();
        log.log(a, "op1", true);
        log.log(b, "op2", true);
        assert_eq!(log.events_by_actor(a).len(), 1);
        assert_eq!(log.events_by_actor(b).len(), 1);
    }
}
