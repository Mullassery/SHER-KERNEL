use serde::{Deserialize, Serialize};
use sher_common::ObjectId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    pub owner: ObjectId,
    pub capabilities: Vec<String>,
    pub audit_enabled: bool,
}

impl SecurityContext {
    pub fn new(owner: ObjectId) -> Self {
        Self {
            owner,
            capabilities: Vec::new(),
            audit_enabled: true,
        }
    }

    pub fn grant(&mut self, capability: impl Into<String>) {
        let capability = capability.into();
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
    }

    pub fn revoke(&mut self, capability: &str) {
        self.capabilities.retain(|c| c != capability);
    }

    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_context_has_no_capabilities_and_audit_on() {
        let ctx = SecurityContext::new(ObjectId::new());
        assert!(ctx.capabilities.is_empty());
        assert!(ctx.audit_enabled);
    }

    #[test]
    fn grant_is_idempotent() {
        let mut ctx = SecurityContext::new(ObjectId::new());
        ctx.grant("read");
        ctx.grant("read");
        assert_eq!(ctx.capabilities.len(), 1);
        assert!(ctx.has_capability("read"));
    }

    #[test]
    fn revoke_removes_capability() {
        let mut ctx = SecurityContext::new(ObjectId::new());
        ctx.grant("write");
        ctx.revoke("write");
        assert!(!ctx.has_capability("write"));
    }
}
