use sher_common::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageDevice {
    pub id: ObjectId,
    pub name: String,
    pub capacity: u64,
    pub device_type: String,
}

impl StorageDevice {
    pub fn new(name: impl Into<String>, capacity: u64, device_type: impl Into<String>) -> Self {
        Self {
            id: ObjectId::new(),
            name: name.into(),
            capacity,
            device_type: device_type.into(),
        }
    }
}
