use sher_common::Result;

#[derive(Debug, Clone, Default)]
pub struct DeviceDiscovery {
    pub discovered_count: u32,
}

impl DeviceDiscovery {
    pub fn scan_bus(&mut self, bus_name: &str) -> Result<u32> {
        self.discovered_count += 1;
        Ok(self.discovered_count)
    }
}
