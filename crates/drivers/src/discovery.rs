//! Hardware discovery: enumerates devices on a simulated bus.
//!
//! There is no real PCI/USB bus enumeration here (that needs kernel/root
//! access this userspace crate does not have). `HardwareDiscovery` models
//! the discovery *process* over an in-memory device list a caller
//! populates (e.g. from a config file or test fixture), which is enough to
//! exercise driver-matching logic end to end.

use sher_common::ObjectId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredDevice {
    pub id: ObjectId,
    pub vendor: String,
    pub device_class: String,
}

impl DiscoveredDevice {
    pub fn new(vendor: impl Into<String>, device_class: impl Into<String>) -> Self {
        Self {
            id: ObjectId::new(),
            vendor: vendor.into(),
            device_class: device_class.into(),
        }
    }
}

#[derive(Debug, Default)]
pub struct HardwareDiscovery {
    bus: Vec<DiscoveredDevice>,
}

impl HardwareDiscovery {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a device to the simulated bus (stands in for a real enumeration
    /// step finding it on PCI/USB/etc).
    pub fn add_device(&mut self, device: DiscoveredDevice) {
        self.bus.push(device);
    }

    /// Enumerate every device currently on the simulated bus. Hardware
    /// discovery finds devices but doesn't load drivers — matching drivers
    /// to devices is `DriverRegistry`'s job.
    pub fn scan(&self) -> Vec<DiscoveredDevice> {
        self.bus.clone()
    }

    pub fn scan_by_class(&self, device_class: &str) -> Vec<DiscoveredDevice> {
        self.bus
            .iter()
            .filter(|d| d.device_class == device_class)
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_returns_all_added_devices() {
        let mut discovery = HardwareDiscovery::new();
        discovery.add_device(DiscoveredDevice::new("intel", "network"));
        discovery.add_device(DiscoveredDevice::new("nvidia", "gpu"));
        assert_eq!(discovery.scan().len(), 2);
    }

    #[test]
    fn scan_by_class_filters() {
        let mut discovery = HardwareDiscovery::new();
        discovery.add_device(DiscoveredDevice::new("intel", "network"));
        discovery.add_device(DiscoveredDevice::new("realtek", "network"));
        discovery.add_device(DiscoveredDevice::new("nvidia", "gpu"));

        let network_devices = discovery.scan_by_class("network");
        assert_eq!(network_devices.len(), 2);
    }

    #[test]
    fn empty_bus_scans_to_empty() {
        let discovery = HardwareDiscovery::new();
        assert!(discovery.scan().is_empty());
    }
}
