use sher_common::{ObjectId, ObjectType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::lifecycle::Lifecycle;
use crate::capabilities::CapabilitySet;
use crate::telemetry::Telemetry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelObject {
    pub id: ObjectId,
    pub obj_type: ObjectType,
    pub name: String,
    pub lifecycle: Lifecycle,
    pub capabilities: CapabilitySet,
    pub telemetry: Telemetry,
    pub dependencies: Vec<ObjectId>,
    pub metadata: HashMap<String, String>,
}

impl KernelObject {
    pub fn new(obj_type: ObjectType, name: impl Into<String>) -> Self {
        Self {
            id: ObjectId::new(),
            obj_type,
            name: name.into(),
            lifecycle: Lifecycle::default(),
            capabilities: CapabilitySet::default(),
            telemetry: Telemetry::default(),
            dependencies: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_dependency(&mut self, dep_id: ObjectId) {
        if !self.dependencies.contains(&dep_id) {
            self.dependencies.push(dep_id);
        }
    }

    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    pub fn is_healthy(&self) -> bool {
        self.telemetry.is_healthy()
    }
}
