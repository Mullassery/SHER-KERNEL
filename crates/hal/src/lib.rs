//! Hardware Abstraction Layer (HAL) - Phase 11 Layer 1
//!
//! Foundation for all hardware access:
//! - Device discovery and enumeration
//! - Memory-mapped I/O management
//! - Interrupt handling interface
//! - Device capabilities querying
//! - Resource management and tracking

use std::collections::HashMap;
use sher_common::{ObjectId, Result};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DeviceType {
    Gpu,
    Audio,
    Input,
    Network,
    Storage,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub id: ObjectId,
    pub device_type: DeviceType,
    pub name: String,
    pub vendor_id: u32,
    pub device_id: u32,
    pub capabilities: Vec<String>,
    pub mmio_base: Option<usize>,
    pub mmio_size: Option<usize>,
    pub irq: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct MemoryMapping {
    pub physical_addr: usize,
    pub virtual_addr: usize,
    pub size: usize,
    pub write_protected: bool,
}

pub trait HardwareDriver: Send + Sync {
    fn probe(&self) -> Result<Vec<DeviceInfo>>;
    fn initialize(&mut self, device: &DeviceInfo) -> Result<()>;
    fn shutdown(&mut self, device_id: &ObjectId) -> Result<()>;
    fn get_capabilities(&self, device_id: &ObjectId) -> Result<Vec<String>>;
    fn read_register(&self, device_id: &ObjectId, offset: usize) -> Result<u32>;
    fn write_register(&self, device_id: &ObjectId, offset: usize, value: u32) -> Result<()>;
}

pub struct HardwareAbstractionLayer {
    devices: HashMap<ObjectId, DeviceInfo>,
    mappings: HashMap<ObjectId, MemoryMapping>,
    drivers: HashMap<DeviceType, Box<dyn HardwareDriver>>,
}

impl HardwareAbstractionLayer {
    pub fn new() -> Self {
        HardwareAbstractionLayer {
            devices: HashMap::new(),
            mappings: HashMap::new(),
            drivers: HashMap::new(),
        }
    }

    pub fn register_driver(
        &mut self,
        device_type: DeviceType,
        driver: Box<dyn HardwareDriver>,
    ) -> Result<()> {
        self.drivers.insert(device_type, driver);
        Ok(())
    }

    pub fn probe_devices(&mut self) -> Result<Vec<DeviceInfo>> {
        let mut all_devices = Vec::new();

        for (_, driver) in &self.drivers {
            let devices = driver.probe()?;
            for device in devices {
                self.devices.insert(device.id.clone(), device.clone());
                all_devices.push(device);
            }
        }

        Ok(all_devices)
    }

    pub fn get_device(&self, device_id: &ObjectId) -> Option<DeviceInfo> {
        self.devices.get(device_id).cloned()
    }

    pub fn get_devices_by_type(&self, device_type: DeviceType) -> Vec<DeviceInfo> {
        self.devices
            .values()
            .filter(|d| d.device_type == device_type)
            .cloned()
            .collect()
    }

    pub fn map_memory(
        &mut self,
        device_id: &ObjectId,
        physical_addr: usize,
        size: usize,
    ) -> Result<usize> {
        let mapping = MemoryMapping {
            physical_addr,
            virtual_addr: physical_addr,
            size,
            write_protected: false,
        };

        self.mappings.insert(device_id.clone(), mapping);
        Ok(physical_addr)
    }

    pub fn get_mapping(&self, device_id: &ObjectId) -> Option<MemoryMapping> {
        self.mappings.get(device_id).cloned()
    }

    pub fn read_register(&self, device_id: &ObjectId, offset: usize) -> Result<u32> {
        let device = self.get_device(device_id)
            .ok_or_else(|| sher_common::Error::Device("Device not found".to_string()))?;

        if let Some(driver) = self.drivers.get(&device.device_type) {
            driver.read_register(device_id, offset)
        } else {
            Err(sher_common::Error::Device("No driver for device type".to_string()))
        }
    }

    pub fn write_register(&self, device_id: &ObjectId, offset: usize, value: u32) -> Result<()> {
        let device = self.get_device(device_id)
            .ok_or_else(|| sher_common::Error::Device("Device not found".to_string()))?;

        if let Some(driver) = self.drivers.get(&device.device_type) {
            driver.write_register(device_id, offset, value)
        } else {
            Err(sher_common::Error::Device("No driver for device type".to_string()))
        }
    }

    pub fn device_count(&self) -> usize {
        self.devices.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockDriver;

    impl HardwareDriver for MockDriver {
        fn probe(&self) -> Result<Vec<DeviceInfo>> {
            Ok(vec![DeviceInfo {
                id: ObjectId::new(),
                device_type: DeviceType::Gpu,
                name: "Mock GPU".to_string(),
                vendor_id: 0x1234,
                device_id: 0x5678,
                capabilities: vec!["3D".to_string(), "2D".to_string()],
                mmio_base: Some(0x1000),
                mmio_size: Some(0x1000),
                irq: Some(10),
            }])
        }

        fn initialize(&mut self, _device: &DeviceInfo) -> Result<()> {
            Ok(())
        }

        fn shutdown(&mut self, _device_id: &ObjectId) -> Result<()> {
            Ok(())
        }

        fn get_capabilities(&self, _device_id: &ObjectId) -> Result<Vec<String>> {
            Ok(vec!["3D".to_string()])
        }

        fn read_register(&self, _device_id: &ObjectId, _offset: usize) -> Result<u32> {
            Ok(0x12345678)
        }

        fn write_register(&self, _device_id: &ObjectId, _offset: usize, _value: u32) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_hal_creation() {
        let hal = HardwareAbstractionLayer::new();
        assert_eq!(hal.device_count(), 0);
    }

    #[test]
    fn test_driver_registration() {
        let mut hal = HardwareAbstractionLayer::new();
        let driver = Box::new(MockDriver);
        let result = hal.register_driver(DeviceType::Gpu, driver);
        assert!(result.is_ok());
    }

    #[test]
    fn test_device_probing() {
        let mut hal = HardwareAbstractionLayer::new();
        let driver = Box::new(MockDriver);
        let _ = hal.register_driver(DeviceType::Gpu, driver);

        let devices = hal.probe_devices().unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].device_type, DeviceType::Gpu);
    }

    #[test]
    fn test_device_retrieval() {
        let mut hal = HardwareAbstractionLayer::new();
        let driver = Box::new(MockDriver);
        let _ = hal.register_driver(DeviceType::Gpu, driver);

        let devices = hal.probe_devices().unwrap();
        let device_id = devices[0].id.clone();

        let retrieved = hal.get_device(&device_id);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_get_devices_by_type() {
        let mut hal = HardwareAbstractionLayer::new();
        let driver = Box::new(MockDriver);
        let _ = hal.register_driver(DeviceType::Gpu, driver);

        let _ = hal.probe_devices();
        let gpu_devices = hal.get_devices_by_type(DeviceType::Gpu);

        assert_eq!(gpu_devices.len(), 1);
    }

    #[test]
    fn test_memory_mapping() {
        let mut hal = HardwareAbstractionLayer::new();
        let device_id = ObjectId::new();

        let result = hal.map_memory(&device_id, 0x1000, 0x1000);
        assert!(result.is_ok());

        let mapping = hal.get_mapping(&device_id);
        assert!(mapping.is_some());
        assert_eq!(mapping.unwrap().size, 0x1000);
    }

    #[test]
    fn test_register_operations() {
        let mut hal = HardwareAbstractionLayer::new();
        let driver = Box::new(MockDriver);
        let _ = hal.register_driver(DeviceType::Gpu, driver);

        let devices = hal.probe_devices().unwrap();
        let device_id = devices[0].id.clone();

        let value = hal.read_register(&device_id, 0);
        assert!(value.is_ok());
        assert_eq!(value.unwrap(), 0x12345678);

        let write_result = hal.write_register(&device_id, 0, 0x87654321);
        assert!(write_result.is_ok());
    }
}
