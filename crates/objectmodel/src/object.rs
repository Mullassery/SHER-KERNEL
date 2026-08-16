use serde::{Deserialize, Serialize};
use sher_common::{ObjectId, ObjectType};
use std::collections::HashMap;

use crate::capabilities::CapabilitySet;
use crate::lifecycle::Lifecycle;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_object_starts_healthy_with_no_dependencies() {
        let obj = KernelObject::new(ObjectType::Driver, "test-driver");
        assert_eq!(obj.name, "test-driver");
        assert!(obj.dependencies.is_empty());
        assert!(obj.is_healthy());
    }

    #[test]
    fn add_dependency_is_idempotent() {
        let mut obj = KernelObject::new(ObjectType::Process, "proc");
        let dep = ObjectId::new();
        obj.add_dependency(dep);
        obj.add_dependency(dep);
        assert_eq!(obj.dependencies.len(), 1);
    }

    #[test]
    fn set_metadata_overwrites_existing_key() {
        let mut obj = KernelObject::new(ObjectType::Device, "dev");
        obj.set_metadata("vendor", "acme");
        obj.set_metadata("vendor", "other-corp");
        assert_eq!(
            obj.metadata.get("vendor").map(String::as_str),
            Some("other-corp")
        );
    }

    #[test]
    fn unhealthy_when_error_rate_high() {
        let mut obj = KernelObject::new(ObjectType::Driver, "flaky");
        for _ in 0..10 {
            obj.telemetry.record_event();
        }
        for _ in 0..5 {
            obj.telemetry.record_error();
        }
        assert!(!obj.is_healthy());
    }
}
