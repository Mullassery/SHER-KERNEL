use sher_common::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sandbox {
    pub id: ObjectId,
    pub name: String,
    pub isolated_resources: bool,
    pub network_access: bool,
    pub filesystem_access: bool,
}

impl Sandbox {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: ObjectId::new(),
            name: name.into(),
            isolated_resources: true,
            network_access: false,
            filesystem_access: false,
        }
    }
}
