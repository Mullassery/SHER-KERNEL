use sher_common::{ObjectId, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxDeviceApi {
    pub registered_drivers: Vec<ObjectId>,
}

impl Default for LinuxDeviceApi {
    fn default() -> Self {
        Self {
            registered_drivers: Vec::new(),
        }
    }
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
