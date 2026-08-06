use sher_common::Result;
use crate::container::DriverContainer;

#[derive(Debug, Clone, Default)]
pub struct DriverLoader {
    pub loaded_drivers: u32,
}

impl DriverLoader {
    pub fn load_driver(&mut self, _path: &str) -> Result<DriverContainer> {
        self.loaded_drivers += 1;
        Ok(DriverContainer::new("generic_driver", 1024 * 1024))
    }

    pub fn unload_driver(&mut self) -> Result<()> {
        self.loaded_drivers = self.loaded_drivers.saturating_sub(1);
        Ok(())
    }
}
