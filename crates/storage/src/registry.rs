//! In-memory registry mapping discovered [`StorageDevice`] descriptors to
//! their (simulated) [`BlockDevice`] backing store.

use crate::blockdevice::BlockDevice;
use crate::device::StorageDevice;
use sher_common::{Error, ObjectId, Result};
use std::collections::HashMap;

#[derive(Default)]
pub struct StorageRegistry {
    devices: HashMap<ObjectId, (StorageDevice, BlockDevice)>,
}

impl StorageRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, device: StorageDevice, block_device: BlockDevice) -> ObjectId {
        let id = device.id;
        self.devices.insert(id, (device, block_device));
        id
    }

    pub fn unregister(&mut self, id: ObjectId) -> bool {
        self.devices.remove(&id).is_some()
    }

    pub fn get(&self, id: ObjectId) -> Option<&StorageDevice> {
        self.devices.get(&id).map(|(dev, _)| dev)
    }

    pub fn block_device_mut(&mut self, id: ObjectId) -> Option<&mut BlockDevice> {
        self.devices.get_mut(&id).map(|(_, bd)| bd)
    }

    pub fn len(&self) -> usize {
        self.devices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }

    pub fn total_capacity(&self) -> u64 {
        self.devices.values().map(|(dev, _)| dev.capacity).sum()
    }

    /// Convenience: write a sector on a registered device by id.
    pub fn write_sector(&mut self, id: ObjectId, sector: u64, data: &[u8]) -> Result<()> {
        self.block_device_mut(id)
            .ok_or_else(|| Error::Storage(format!("no such device: {id}")))?
            .write_sector(sector, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_lookup() {
        let mut reg = StorageRegistry::new();
        let dev = StorageDevice::new("nvme0", 1_000_000, "NVMe");
        let bd = BlockDevice::new(512, 1953);
        let id = reg.register(dev, bd);
        assert!(reg.get(id).is_some());
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn unregister_removes_device() {
        let mut reg = StorageRegistry::new();
        let id = reg.register(
            StorageDevice::new("sd0", 1000, "SD"),
            BlockDevice::new(512, 1),
        );
        assert!(reg.unregister(id));
        assert!(reg.is_empty());
        assert!(!reg.unregister(id));
    }

    #[test]
    fn total_capacity_sums_devices() {
        let mut reg = StorageRegistry::new();
        reg.register(
            StorageDevice::new("a", 100, "SATA"),
            BlockDevice::new(512, 1),
        );
        reg.register(
            StorageDevice::new("b", 200, "USB"),
            BlockDevice::new(512, 1),
        );
        assert_eq!(reg.total_capacity(), 300);
    }

    #[test]
    fn write_sector_through_registry() {
        let mut reg = StorageRegistry::new();
        let id = reg.register(
            StorageDevice::new("a", 1024, "SATA"),
            BlockDevice::new(512, 2),
        );
        assert!(reg.write_sector(id, 0, &[1u8; 512]).is_ok());
        assert!(reg.write_sector(ObjectId::new(), 0, &[1u8; 512]).is_err());
    }
}
