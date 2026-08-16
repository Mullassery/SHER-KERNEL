// SHER LKI: Device Model Translation
// Maps Linux device/bus/driver registration to SHER primitives

use serde::{Deserialize, Serialize};
use sher_common::{Error, ObjectId, Result};
use std::collections::HashMap;

// ============================================================================
// PCI DEVICE MODEL
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PciDeviceId {
    pub vendor: u16,
    pub device: u16,
    pub subvendor: u16,
    pub subdevice: u16,
}

impl PciDeviceId {
    pub fn new(vendor: u16, device: u16) -> Self {
        PciDeviceId {
            vendor,
            device,
            subvendor: 0xFFFF,
            subdevice: 0xFFFF,
        }
    }

    pub fn matches(&self, other: &PciDeviceId) -> bool {
        (self.vendor == 0xFFFF || self.vendor == other.vendor)
            && (self.device == 0xFFFF || self.device == other.device)
            && (self.subvendor == 0xFFFF || self.subvendor == other.subvendor)
            && (self.subdevice == 0xFFFF || self.subdevice == other.subdevice)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PciDevice {
    pub device_id: ObjectId,
    pub pci_id: PciDeviceId,
    pub bus: u8,
    pub slot: u8,
    pub function: u8,
    pub class_code: u32,
    pub revision: u8,
    pub bar_regions: Vec<(u32, u32)>, // (address, size)
    pub irq: u32,
    pub enabled: bool,
}

impl PciDevice {
    pub fn new(pci_id: PciDeviceId, bus: u8, slot: u8, function: u8) -> Self {
        PciDevice {
            device_id: ObjectId::new(),
            pci_id,
            bus,
            slot,
            function,
            class_code: 0,
            revision: 0,
            bar_regions: Vec::new(),
            irq: 0,
            enabled: false,
        }
    }

    pub fn bdf(&self) -> u32 {
        ((self.bus as u32) << 16) | ((self.slot as u32) << 11) | (self.function as u32)
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn add_bar(&mut self, address: u32, size: u32) {
        self.bar_regions.push((address, size));
    }
}

// ============================================================================
// PCI DRIVER
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PciDriver {
    pub driver_id: ObjectId,
    pub driver_name: String,
    pub supported_devices: Vec<PciDeviceId>,
    pub registered_at: u64,
    pub probed_devices: u64,
    pub successful_probes: u64,
    pub failed_probes: u64,
}

impl PciDriver {
    pub fn new(name: &str) -> Self {
        PciDriver {
            driver_id: ObjectId::new(),
            driver_name: name.to_string(),
            supported_devices: Vec::new(),
            registered_at: 0,
            probed_devices: 0,
            successful_probes: 0,
            failed_probes: 0,
        }
    }

    pub fn add_device(&mut self, device_id: PciDeviceId) {
        self.supported_devices.push(device_id);
    }

    pub fn supports_device(&self, device_id: &PciDeviceId) -> bool {
        self.supported_devices.iter().any(|d| d.matches(device_id))
    }

    pub fn record_probe(&mut self, success: bool) {
        self.probed_devices += 1;
        if success {
            self.successful_probes += 1;
        } else {
            self.failed_probes += 1;
        }
    }

    pub fn probe_success_rate(&self) -> f64 {
        if self.probed_devices == 0 {
            0.0
        } else {
            (self.successful_probes as f64 / self.probed_devices as f64) * 100.0
        }
    }
}

// ============================================================================
// DEVICE BUS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BusType {
    Pci,
    Usb,
    I2c,
    Spi,
    Platform,
    Acpi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceBus {
    pub bus_id: ObjectId,
    pub bus_type: BusType,
    pub bus_name: String,
    pub devices: Vec<ObjectId>,
    pub drivers: Vec<ObjectId>,
}

impl DeviceBus {
    pub fn new(bus_type: BusType, name: &str) -> Self {
        DeviceBus {
            bus_id: ObjectId::new(),
            bus_type,
            bus_name: name.to_string(),
            devices: Vec::new(),
            drivers: Vec::new(),
        }
    }

    pub fn add_device(&mut self, device_id: ObjectId) {
        if !self.devices.contains(&device_id) {
            self.devices.push(device_id);
        }
    }

    pub fn remove_device(&mut self, device_id: ObjectId) {
        self.devices.retain(|d| d != &device_id);
    }

    pub fn add_driver(&mut self, driver_id: ObjectId) {
        if !self.drivers.contains(&driver_id) {
            self.drivers.push(driver_id);
        }
    }

    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    pub fn driver_count(&self) -> usize {
        self.drivers.len()
    }
}

// ============================================================================
// DEVICE MANAGER
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct DeviceManager {
    pub pci_devices: HashMap<u32, PciDevice>, // BDF -> device
    pub pci_drivers: HashMap<ObjectId, PciDriver>, // driver_id -> driver
    pub buses: HashMap<ObjectId, DeviceBus>,  // bus_id -> bus
    pub device_to_driver: HashMap<ObjectId, ObjectId>, // device_id -> driver_id
    pub total_devices: u64,
    pub total_drivers: u64,
}

impl DeviceManager {
    pub fn new() -> Self {
        DeviceManager::default()
    }

    /// Translate pci_driver_register(driver, id_table)
    pub fn register_pci_driver(
        &mut self,
        name: &str,
        device_ids: Vec<PciDeviceId>,
    ) -> Result<ObjectId> {
        let mut driver = PciDriver::new(name);
        for id in device_ids {
            driver.add_device(id);
        }

        let driver_id = driver.driver_id;
        self.pci_drivers.insert(driver_id, driver);
        self.total_drivers += 1;

        Ok(driver_id)
    }

    /// Translate pci_driver_unregister(driver)
    pub fn unregister_pci_driver(&mut self, driver_id: ObjectId) -> Result<()> {
        if self.pci_drivers.remove(&driver_id).is_some() {
            Ok(())
        } else {
            Err(Error::Driver("Driver not found".to_string()))
        }
    }

    /// Register a PCI device
    pub fn register_pci_device(
        &mut self,
        pci_id: PciDeviceId,
        bus: u8,
        slot: u8,
        func: u8,
    ) -> Result<ObjectId> {
        let device = PciDevice::new(pci_id, bus, slot, func);
        let bdf = device.bdf();
        let device_id = device.device_id;

        self.pci_devices.insert(bdf, device);
        self.total_devices += 1;

        // Try to probe with available drivers
        self.probe_device_drivers(device_id, pci_id)?;

        Ok(device_id)
    }

    /// Probe device against all drivers
    pub fn probe_device_drivers(&mut self, device_id: ObjectId, pci_id: PciDeviceId) -> Result<()> {
        let mut matched_driver = None;

        for (driver_id, driver) in self.pci_drivers.iter_mut() {
            if driver.supports_device(&pci_id) {
                driver.record_probe(true);
                matched_driver = Some(*driver_id);
                break;
            } else {
                driver.record_probe(false);
            }
        }

        if let Some(driver_id) = matched_driver {
            self.device_to_driver.insert(device_id, driver_id);
        }

        Ok(())
    }

    /// Get driver for device
    pub fn get_driver_for_device(&self, device_id: ObjectId) -> Option<&PciDriver> {
        self.device_to_driver
            .get(&device_id)
            .and_then(|driver_id| self.pci_drivers.get(driver_id))
    }

    /// Translate bus_register(bus)
    pub fn register_bus(&mut self, bus_type: BusType, name: &str) -> Result<ObjectId> {
        let bus = DeviceBus::new(bus_type, name);
        let bus_id = bus.bus_id;
        self.buses.insert(bus_id, bus);
        Ok(bus_id)
    }

    /// Add device to bus
    pub fn bus_add_device(&mut self, bus_id: ObjectId, device_id: ObjectId) -> Result<()> {
        if let Some(bus) = self.buses.get_mut(&bus_id) {
            bus.add_device(device_id);
            Ok(())
        } else {
            Err(Error::Driver("Bus not found".to_string()))
        }
    }

    /// Add driver to bus
    pub fn bus_add_driver(&mut self, bus_id: ObjectId, driver_id: ObjectId) -> Result<()> {
        if let Some(bus) = self.buses.get_mut(&bus_id) {
            bus.add_driver(driver_id);
            Ok(())
        } else {
            Err(Error::Driver("Bus not found".to_string()))
        }
    }

    /// Get device by BDF
    pub fn get_device_by_bdf(&self, bdf: u32) -> Option<&PciDevice> {
        self.pci_devices.get(&bdf)
    }

    /// Find devices by vendor
    pub fn find_devices_by_vendor(&self, vendor: u16) -> Vec<&PciDevice> {
        self.pci_devices
            .values()
            .filter(|d| d.pci_id.vendor == vendor)
            .collect()
    }

    /// Find devices by class
    pub fn find_devices_by_class(&self, class_code: u32) -> Vec<&PciDevice> {
        self.pci_devices
            .values()
            .filter(|d| d.class_code == class_code)
            .collect()
    }

    /// Enable device
    pub fn enable_device(&mut self, device_id: u32) -> Result<()> {
        if let Some(device) = self.pci_devices.get_mut(&device_id) {
            device.enable();
            Ok(())
        } else {
            Err(Error::Driver("Device not found".to_string()))
        }
    }

    /// Disable device
    pub fn disable_device(&mut self, device_id: u32) -> Result<()> {
        if let Some(device) = self.pci_devices.get_mut(&device_id) {
            device.disable();
            Ok(())
        } else {
            Err(Error::Driver("Device not found".to_string()))
        }
    }

    /// Get device statistics
    pub fn get_stats(&self) -> DeviceStats {
        let registered_devices = self.pci_devices.len() as u64;
        let enabled_devices = self.pci_devices.values().filter(|d| d.enabled).count() as u64;

        DeviceStats {
            total_devices: self.total_devices,
            registered_devices,
            enabled_devices,
            total_drivers: self.total_drivers,
            matched_pairs: self.device_to_driver.len() as u64,
            total_buses: self.buses.len() as u64,
        }
    }
}

// ============================================================================
// DEVICE STATISTICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStats {
    pub total_devices: u64,
    pub registered_devices: u64,
    pub enabled_devices: u64,
    pub total_drivers: u64,
    pub matched_pairs: u64,
    pub total_buses: u64,
}

// ============================================================================
// BLOCK DEVICE MODEL
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockDeviceType {
    HardDrive,
    SolidState,
    Nvme,
    Ramdisk,
    LoopbackDevice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDevice {
    pub device_id: ObjectId,
    pub device_type: BlockDeviceType,
    pub major_number: u32,
    pub minor_number: u32,
    pub capacity_bytes: u64,
    pub block_size: u32,
    pub read_only: bool,
    pub registered: bool,
}

impl BlockDevice {
    pub fn new(device_type: BlockDeviceType, major: u32, minor: u32, capacity: u64) -> Self {
        BlockDevice {
            device_id: ObjectId::new(),
            device_type,
            major_number: major,
            minor_number: minor,
            capacity_bytes: capacity,
            block_size: 4096,
            read_only: false,
            registered: false,
        }
    }

    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }

    pub fn register(&mut self) {
        self.registered = true;
    }

    pub fn unregister(&mut self) {
        self.registered = false;
    }
}

#[derive(Debug, Clone, Default)]
pub struct BlockDeviceManager {
    pub devices: HashMap<(u32, u32), BlockDevice>, // (major, minor) -> device
    pub total_registered: u64,
}

impl BlockDeviceManager {
    pub fn new() -> Self {
        BlockDeviceManager::default()
    }

    pub fn register_block_device(&mut self, device: BlockDevice) -> Result<ObjectId> {
        let key = (device.major_number, device.minor_number);
        if self.devices.contains_key(&key) {
            return Err(Error::Driver("Device already registered".to_string()));
        }

        let device_id = device.device_id;
        let mut d = device;
        d.register();
        self.devices.insert(key, d);
        self.total_registered += 1;

        Ok(device_id)
    }

    pub fn unregister_block_device(&mut self, major: u32, minor: u32) -> Result<()> {
        let key = (major, minor);
        if self.devices.remove(&key).is_some() {
            Ok(())
        } else {
            Err(Error::Driver("Device not found".to_string()))
        }
    }

    pub fn get_device(&self, major: u32, minor: u32) -> Option<&BlockDevice> {
        self.devices.get(&(major, minor))
    }
}

// ============================================================================
// NETWORK DEVICE MODEL
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetDeviceType {
    Ethernet,
    Wireless,
    Loopback,
    Bridge,
    Vlan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDevice {
    pub device_id: ObjectId,
    pub device_type: NetDeviceType,
    pub name: String,
    pub mac_address: [u8; 6],
    pub mtu: u32,
    pub registered: bool,
}

impl NetworkDevice {
    pub fn new(device_type: NetDeviceType, name: &str) -> Self {
        NetworkDevice {
            device_id: ObjectId::new(),
            device_type,
            name: name.to_string(),
            mac_address: [0; 6],
            mtu: 1500,
            registered: false,
        }
    }

    pub fn set_mac(&mut self, mac: [u8; 6]) {
        self.mac_address = mac;
    }

    pub fn register(&mut self) {
        self.registered = true;
    }

    pub fn unregister(&mut self) {
        self.registered = false;
    }
}

#[derive(Debug, Clone, Default)]
pub struct NetworkDeviceManager {
    pub devices: HashMap<String, NetworkDevice>, // name -> device
    pub total_registered: u64,
}

impl NetworkDeviceManager {
    pub fn new() -> Self {
        NetworkDeviceManager::default()
    }

    pub fn register_net_device(&mut self, device: NetworkDevice) -> Result<ObjectId> {
        if self.devices.contains_key(&device.name) {
            return Err(Error::Driver("Device already registered".to_string()));
        }

        let device_id = device.device_id;
        let mut d = device;
        d.register();
        self.devices.insert(d.name.clone(), d);
        self.total_registered += 1;

        Ok(device_id)
    }

    pub fn unregister_net_device(&mut self, name: &str) -> Result<()> {
        if self.devices.remove(name).is_some() {
            Ok(())
        } else {
            Err(Error::Driver("Device not found".to_string()))
        }
    }

    pub fn get_device(&self, name: &str) -> Option<&NetworkDevice> {
        self.devices.get(name)
    }

    pub fn list_devices(&self) -> Vec<&NetworkDevice> {
        self.devices.values().collect()
    }
}
