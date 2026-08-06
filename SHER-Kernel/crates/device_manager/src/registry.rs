use sher_common::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredDevice {
    pub id: ObjectId,
    pub name: String,
    pub driver_loaded: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DeviceRegistry {
    pub devices: HashMap<ObjectId, RegisteredDevice>,
}

impl DeviceRegistry {
    pub fn register(&mut self, device: RegisteredDevice) {
        self.devices.insert(device.id, device);
    }

    pub fn unregister(&mut self, id: ObjectId) {
        self.devices.remove(&id);
    }

    pub fn get_device(&self, id: ObjectId) -> Option<&RegisteredDevice> {
        self.devices.get(&id)
    }

    pub fn list_devices(&self) -> Vec<&RegisteredDevice> {
        self.devices.values().collect()
    }
}
