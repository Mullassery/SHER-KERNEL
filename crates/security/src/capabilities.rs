use sher_common::ObjectId;
use serde::{Deserialize, Serialize};

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
}
