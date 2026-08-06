// SHER Device Manager: Comprehensive Unit Tests
// Coverage: Discovery, Registry, Policy, Matching

#[cfg(test)]
mod tests {
    use crate::*;
    use sher_common::ObjectId;

    // ========================================================================
    // PCI DISCOVERY TESTS
    // ========================================================================

    #[test]
    fn test_pci_enumerator_new() {
        let enumerator = PciEnumerator::new();
        assert_eq!(enumerator.get_device_count(), 0);
        assert!(!enumerator.enumeration_complete);
    }

    #[test]
    fn test_pci_enumerate_returns_devices() {
        let mut enumerator = PciEnumerator::new();
        let count = enumerator.enumerate().expect("Enumeration failed");
        assert!(count > 0, "Should discover at least one device");
        assert!(enumerator.enumeration_complete);
    }

    #[test]
    fn test_pci_enumerate_finds_host_bridge() {
        let mut enumerator = PciEnumerator::new();
        enumerator.enumerate().expect("Enumeration failed");

        let bridge = enumerator.find_by_vendor_device(0x8086, 0x0100);
        assert!(bridge.is_some(), "Should find Intel host bridge");
        assert_eq!(bridge.unwrap().class_code, 0x06);  // Bridge
    }

    #[test]
    fn test_pci_enumerate_finds_ethernet() {
        let mut enumerator = PciEnumerator::new();
        enumerator.enumerate().expect("Enumeration failed");

        let ethernet = enumerator.find_by_vendor_device(0x8086, 0x1234);
        assert!(ethernet.is_some(), "Should find ethernet controller");
        assert_eq!(ethernet.unwrap().class_code, 0x02);  // Network
    }

    #[test]
    fn test_pci_find_by_class() {
        let mut enumerator = PciEnumerator::new();
        enumerator.enumerate().expect("Enumeration failed");

        let bridges = enumerator.find_by_class(0x06, 0x00);  // Class 6, Subclass 0
        assert!(!bridges.is_empty(), "Should find at least one bridge");
    }

    #[test]
    fn test_pci_device_bar_regions() {
        let mut enumerator = PciEnumerator::new();
        enumerator.enumerate().expect("Enumeration failed");

        let device = enumerator.find_by_vendor_device(0x8086, 0x1234);
        assert!(device.is_some());
        let dev = device.unwrap();
        assert_ne!(dev.bar_regions[0], 0);  // Should have BAR region
    }

    #[test]
    fn test_pci_multiple_enumerate_clears_previous() {
        let mut enumerator = PciEnumerator::new();
        let count1 = enumerator.enumerate().expect("First enumeration failed");
        let count2 = enumerator.enumerate().expect("Second enumeration failed");
        assert_eq!(count1, count2, "Enumeration should be stable");
    }

    // ========================================================================
    // USB DISCOVERY TESTS
    // ========================================================================

    #[test]
    fn test_usb_enumerator_new() {
        let enumerator = UsbEnumerator::new();
        assert_eq!(enumerator.get_device_count(), 0);
        assert!(!enumerator.enumeration_complete);
    }

    #[test]
    fn test_usb_enumerate_returns_devices() {
        let mut enumerator = UsbEnumerator::new();
        let count = enumerator.enumerate().expect("Enumeration failed");
        assert!(count > 0, "Should discover at least one USB device");
        assert!(enumerator.enumeration_complete);
    }

    #[test]
    fn test_usb_enumerate_finds_mouse() {
        let mut enumerator = UsbEnumerator::new();
        enumerator.enumerate().expect("Enumeration failed");

        let mouse = enumerator.find_by_vendor_product(0x046d, 0xc534);
        assert!(mouse.is_some(), "Should find Logitech mouse");
    }

    #[test]
    fn test_usb_enumerate_finds_hub() {
        let mut enumerator = UsbEnumerator::new();
        enumerator.enumerate().expect("Enumeration failed");

        let hubs = enumerator.find_hubs();
        assert!(!hubs.is_empty(), "Should find at least one hub");
    }

    #[test]
    fn test_usb_find_by_class() {
        let mut enumerator = UsbEnumerator::new();
        enumerator.enumerate().expect("Enumeration failed");

        let hub_devices = enumerator.find_by_class(0x09);  // Hub class
        assert!(!hub_devices.is_empty(), "Should find hub devices");
    }

    #[test]
    fn test_usb_device_speeds() {
        let mut enumerator = UsbEnumerator::new();
        enumerator.enumerate().expect("Enumeration failed");

        let high_speed = enumerator.devices.iter().find(|d| d.speed == UsbSpeed::High);
        assert!(high_speed.is_some(), "Should have high-speed devices");
    }

    #[test]
    fn test_usb_hub_hierarchy() {
        let mut enumerator = UsbEnumerator::new();
        enumerator.enumerate().expect("Enumeration failed");

        let devices_with_parent = enumerator.devices.iter().filter(|d| d.parent_hub.is_some());
        assert!(devices_with_parent.count() > 0, "Should have devices connected to hubs");
    }

    // ========================================================================
    // FIRMWARE DISCOVERY TESTS
    // ========================================================================

    #[test]
    fn test_firmware_discovery_new() {
        let discovery = FirmwareDiscovery::new();
        assert_eq!(discovery.get_device_count(), 0);
        assert!(discovery.firmware_type.is_none());
    }

    #[test]
    fn test_firmware_scan() {
        let mut discovery = FirmwareDiscovery::new();
        let count = discovery.scan().expect("Scan failed");
        assert!(count > 0, "Should discover firmware devices");
        assert_eq!(discovery.firmware_type, Some(FirmwareType::Acpi));
    }

    #[test]
    fn test_firmware_discovery_cpu() {
        let mut discovery = FirmwareDiscovery::new();
        discovery.scan().expect("Scan failed");

        let cpu = discovery.devices.iter().find(|d| d.name.starts_with("CPU"));
        assert!(cpu.is_some(), "Should find CPU device");
    }

    // ========================================================================
    // UNIFIED DISCOVERY TESTS
    // ========================================================================

    #[test]
    fn test_device_discovery_new() {
        let discovery = DeviceDiscovery::new();
        assert_eq!(discovery.get_total_device_count(), 0);
    }

    #[test]
    fn test_device_discovery_scan_all() {
        let mut discovery = DeviceDiscovery::new();
        let count = discovery.scan_all().expect("Scan failed");
        assert!(count > 0, "Should discover devices");
        assert_eq!(discovery.get_total_device_count(), count);
    }

    #[test]
    fn test_device_discovery_pci_count() {
        let mut discovery = DeviceDiscovery::new();
        discovery.scan_all().expect("Scan failed");
        assert!(discovery.get_pci_device_count() > 0);
    }

    #[test]
    fn test_device_discovery_usb_count() {
        let mut discovery = DeviceDiscovery::new();
        discovery.scan_all().expect("Scan failed");
        assert!(discovery.get_usb_device_count() > 0);
    }

    // ========================================================================
    // DEVICE STATE TESTS
    // ========================================================================

    #[test]
    fn test_device_state_transitions_valid() {
        let mut state = DeviceState::Discovered;
        assert!(state.can_transition_to(DeviceState::Initialized));
        state = DeviceState::Initialized;
        assert!(state.can_transition_to(DeviceState::Ready));
    }

    #[test]
    fn test_device_state_transitions_invalid() {
        let state = DeviceState::Discovered;
        assert!(!state.can_transition_to(DeviceState::Running));  // Can't skip Ready
    }

    #[test]
    fn test_device_state_operational() {
        assert!(!DeviceState::Discovered.is_operational());
        assert!(DeviceState::Ready.is_operational());
        assert!(DeviceState::Running.is_operational());
    }

    // ========================================================================
    // DEVICE REGISTRY TESTS
    // ========================================================================

    #[test]
    fn test_registry_new() {
        let registry = DeviceRegistry::new();
        assert_eq!(registry.get_device_count(), 0);
    }

    #[test]
    fn test_registry_register_device() {
        let mut registry = DeviceRegistry::new();
        let device = RegisteredDevice::new(
            ObjectId::new(),
            "eth0".to_string(),
            "ethernet".to_string(),
            0,
        );
        let device_id = device.id;

        registry.register(device);
        assert_eq!(registry.get_device_count(), 1);
        assert!(registry.get_device(device_id).is_some());
    }

    #[test]
    fn test_registry_find_by_name() {
        let mut registry = DeviceRegistry::new();
        let device = RegisteredDevice::new(
            ObjectId::new(),
            "pci:0000:00:1f.0".to_string(),
            "pci".to_string(),
            0,
        );
        registry.register(device);

        let found = registry.find_by_name("pci:0000:00:1f.0");
        assert!(found.is_some());
    }

    #[test]
    fn test_registry_find_by_type() {
        let mut registry = DeviceRegistry::new();
        let dev1 = RegisteredDevice::new(ObjectId::new(), "eth0".to_string(), "ethernet".to_string(), 0);
        let dev2 = RegisteredDevice::new(ObjectId::new(), "eth1".to_string(), "ethernet".to_string(), 0);
        registry.register(dev1);
        registry.register(dev2);

        let devices = registry.find_by_type("ethernet");
        assert_eq!(devices.len(), 2);
    }

    #[test]
    fn test_registry_find_by_state() {
        let mut registry = DeviceRegistry::new();
        let device = RegisteredDevice::new(ObjectId::new(), "usb1".to_string(), "usb".to_string(), 0);
        registry.register(device);

        let discovered = registry.find_by_state(DeviceState::Discovered);
        assert_eq!(discovered.len(), 1);
    }

    #[test]
    fn test_registry_device_state_transition() {
        let mut registry = DeviceRegistry::new();
        let device = RegisteredDevice::new(ObjectId::new(), "nvme0".to_string(), "nvme".to_string(), 0);
        let device_id = device.id;
        registry.register(device);

        let result = registry.update_device_state(device_id, DeviceState::Initialized);
        assert!(result.is_ok());

        let device = registry.get_device(device_id).unwrap();
        assert_eq!(device.state, DeviceState::Initialized);
    }

    #[test]
    fn test_registry_device_hierarchy() {
        let mut registry = DeviceRegistry::new();
        let parent = RegisteredDevice::new(ObjectId::new(), "pci0".to_string(), "pci_bus".to_string(), 0);
        let parent_id = parent.id;
        registry.register(parent);

        let mut child = RegisteredDevice::new(ObjectId::new(), "eth0".to_string(), "ethernet".to_string(), 0);
        child.parent_id = Some(parent_id);
        registry.register(child);

        let children = registry.find_children(parent_id);
        assert_eq!(children.len(), 1);
    }

    #[test]
    fn test_registry_device_properties() {
        let mut registry = DeviceRegistry::new();
        let mut device = RegisteredDevice::new(ObjectId::new(), "dev0".to_string(), "generic".to_string(), 0);
        device.properties.insert("vendor_id".to_string(), "0x8086".to_string());
        device.properties.insert("device_id".to_string(), "0x1234".to_string());

        let device_id = device.id;
        registry.register(device);

        let dev = registry.get_device(device_id).unwrap();
        assert_eq!(dev.properties.get("vendor_id").unwrap(), "0x8086");
    }

    #[test]
    fn test_registry_operational_count() {
        let mut registry = DeviceRegistry::new();
        let mut dev1 = RegisteredDevice::new(ObjectId::new(), "dev1".to_string(), "type1".to_string(), 0);
        dev1.state = DeviceState::Running;

        let dev2 = RegisteredDevice::new(ObjectId::new(), "dev2".to_string(), "type2".to_string(), 0);

        registry.register(dev1);
        registry.register(dev2);

        assert_eq!(registry.get_operational_count(), 1);
    }

    // ========================================================================
    // DRIVER POLICY TESTS
    // ========================================================================

    #[test]
    fn test_driver_policy_default() {
        let policy = DriverPolicy::default();
        assert!(!policy.allow_unsigned_drivers);
        assert!(policy.require_capability_match);
        assert!(policy.prefer_native_drivers);
    }

    #[test]
    fn test_device_policy_default() {
        let policy = DevicePolicy::default();
        assert!(policy.power_management_enabled);
        assert!(policy.auto_restart_on_error);
        assert!(policy.enable_hotplug);
    }

    // ========================================================================
    // DRIVER MATCHING TESTS
    // ========================================================================

    #[test]
    fn test_driver_database_new() {
        let db = DriverDatabase::new();
        assert_eq!(db.drivers.len(), 0);
    }

    #[test]
    fn test_driver_database_register_exact_match() {
        let mut db = DriverDatabase::new();
        let driver = DriverEntry {
            id: "intel_eth".to_string(),
            name: "Intel Ethernet".to_string(),
            vendor_id: Some(0x8086),
            device_id: Some(0x1234),
            device_class: None,
            device_subclass: None,
            native: true,
            version: "1.0".to_string(),
            required_capabilities: vec![],
        };

        db.register_driver(driver);
        let matches = db.find_exact_match(0x8086, 0x1234);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_driver_database_register_class_match() {
        let mut db = DriverDatabase::new();
        let driver = DriverEntry {
            id: "generic_eth".to_string(),
            name: "Generic Ethernet".to_string(),
            vendor_id: None,
            device_id: None,
            device_class: Some(0x02),
            device_subclass: Some(0x00),
            native: false,
            version: "1.0".to_string(),
            required_capabilities: vec![],
        };

        db.register_driver(driver);
        let matches = db.find_class_match(0x02, 0x00);
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_driver_matcher_exact_match() {
        let mut matcher = DriverMatcher::new(DriverPolicy::default());
        let driver = DriverEntry {
            id: "driver1".to_string(),
            name: "Driver 1".to_string(),
            vendor_id: Some(0x8086),
            device_id: Some(0x1234),
            device_class: None,
            device_subclass: None,
            native: true,
            version: "1.0".to_string(),
            required_capabilities: vec![],
        };

        matcher.database.register_driver(driver);

        let best = matcher.find_best_match(0x8086, 0x1234, 0x02, 0x00);
        assert!(best.is_some());
        let m = best.unwrap();
        assert_eq!(m.match_type, MatchType::ExactVendorDevice);
        assert_eq!(m.confidence, 1.0);
    }

    #[test]
    fn test_driver_matcher_class_fallback() {
        let mut matcher = DriverMatcher::new(DriverPolicy::default());
        let driver = DriverEntry {
            id: "generic".to_string(),
            name: "Generic Driver".to_string(),
            vendor_id: None,
            device_id: None,
            device_class: Some(0x02),
            device_subclass: Some(0x00),
            native: false,
            version: "1.0".to_string(),
            required_capabilities: vec![],
        };

        matcher.database.register_driver(driver);

        let best = matcher.find_best_match(0x9999, 0x9999, 0x02, 0x00);
        assert!(best.is_some());
        let m = best.unwrap();
        assert_eq!(m.match_type, MatchType::ClassCode);
    }

    #[test]
    fn test_match_type_priority() {
        assert!(MatchType::ExactVendorDevice.priority() > MatchType::ClassCode.priority());
        assert!(MatchType::ClassCode.priority() > MatchType::Generic.priority());
    }

    // ========================================================================
    // INTEGRATION TESTS
    // ========================================================================

    #[test]
    fn test_full_discovery_to_registry() {
        let mut discovery = DeviceDiscovery::new();
        discovery.scan_all().expect("Discovery failed");

        let mut registry = DeviceRegistry::new();

        // Register PCI devices
        for pci_dev in &discovery.pci_enumerator.devices {
            let mut device = RegisteredDevice::new(
                ObjectId::new(),
                format!("pci:{:02x}:{:02x}.{}", pci_dev.bus, pci_dev.slot, pci_dev.function),
                "pci".to_string(),
                0,
            );
            device.properties.insert("vendor_id".to_string(), format!("0x{:04x}", pci_dev.vendor_id));
            device.properties.insert("device_id".to_string(), format!("0x{:04x}", pci_dev.device_id));
            registry.register(device);
        }

        // Register USB devices
        for usb_dev in &discovery.usb_enumerator.devices {
            let device = RegisteredDevice::new(
                ObjectId::new(),
                format!("usb:{:02x}:{:02x}", usb_dev.bus, usb_dev.address),
                "usb".to_string(),
                0,
            );
            registry.register(device);
        }

        assert!(registry.get_device_count() > 0);
        assert!(registry.find_by_type("pci").len() > 0);
        assert!(registry.find_by_type("usb").len() > 0);
    }

    #[test]
    fn test_device_lifecycle_simulation() {
        let mut registry = DeviceRegistry::new();
        let device = RegisteredDevice::new(
            ObjectId::new(),
            "test_dev".to_string(),
            "test".to_string(),
            0,
        );
        let device_id = device.id;

        assert_eq!(device.state, DeviceState::Discovered);

        registry.register(device);
        registry.update_device_state(device_id, DeviceState::Initialized).ok();
        registry.update_device_state(device_id, DeviceState::Ready).ok();
        registry.update_device_state(device_id, DeviceState::Running).ok();

        let dev = registry.get_device(device_id).unwrap();
        assert_eq!(dev.state, DeviceState::Running);
    }

    #[test]
    fn test_device_error_handling() {
        let mut device = RegisteredDevice::new(
            ObjectId::new(),
            "err_dev".to_string(),
            "test".to_string(),
            0,
        );

        device.record_error("Driver crashed".to_string());
        assert_eq!(device.state, DeviceState::Error);
        assert!(device.telemetry.last_error.is_some());
        assert_eq!(device.telemetry.total_errors, 1);
    }
}
