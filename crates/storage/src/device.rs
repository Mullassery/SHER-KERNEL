use serde::{Deserialize, Serialize};
use sher_common::ObjectId;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_device_has_unique_id_and_stored_fields() {
        let a = StorageDevice::new("nvme0", 512_000_000_000, "NVMe");
        let b = StorageDevice::new("nvme0", 512_000_000_000, "NVMe");
        assert_ne!(a.id, b.id);
        assert_eq!(a.name, "nvme0");
        assert_eq!(a.capacity, 512_000_000_000);
        assert_eq!(a.device_type, "NVMe");
    }
}
