use serde::{Deserialize, Serialize};
use sher_common::{ObjectId, Result};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LinuxDeviceApi {
    pub registered_drivers: Vec<ObjectId>,
}

impl LinuxDeviceApi {
    pub fn pci_driver_register(&mut self, driver_id: ObjectId) -> Result<()> {
        self.registered_drivers.push(driver_id);
        Ok(())
    }

    pub fn pci_driver_unregister(&mut self, driver_id: ObjectId) -> Result<()> {
        self.registered_drivers.retain(|id| id != &driver_id);
        Ok(())
    }

    pub fn get_registered_drivers(&self) -> Vec<ObjectId> {
        self.registered_drivers.clone()
    }
}
