// SHER LKI: Comprehensive Tests

#[cfg(test)]
mod tests {
    use crate::memory_translation::LinuxMemoryAllocator;
    use crate::interrupt_translation::{InterruptManager, InterruptHandler, IrqTrigger};
    use crate::device_translation::{
        DeviceManager, PciDriver, PciDevice, PciDeviceId, DeviceBus, BusType,
        BlockDevice, BlockDeviceManager, BlockDeviceType, NetworkDevice, NetworkDeviceManager, NetDeviceType,
    };
    use crate::audit::{AuditLog, AuditEntry, AuditLevel, AuditFilter};
    use crate::security::{Capability, PermissionTier, CapabilityGrant, CapabilityManager, SecurityPolicy, SecurityLevel, ReauthMethod};
    use crate::enforcement::{SecurityContext, SecurityEnforcer, PermissionChecker};
    use crate::validation::Validator;
    use sher_common::ObjectId;

    // ========================================================================
    // VALIDATION TESTS
    // ========================================================================

    #[test]
    fn test_validator_new() {
        let validator = Validator::new();
        assert_eq!(validator.total_validations, 0);
        assert_eq!(validator.failed_validations, 0);
        assert_eq!(validator.success_rate(), 100.0);
    }

    #[test]
    fn test_validate_allocation_valid() {
        let mut validator = Validator::new();
        let result = validator.validate_allocation(1024, 0);
        assert!(result.is_ok());
        assert_eq!(validator.total_validations, 1);
        assert_eq!(validator.failed_validations, 0);
    }

    #[test]
    fn test_validate_allocation_zero_size() {
        let mut validator = Validator::new();
        let result = validator.validate_allocation(0, 0);
        assert!(result.is_err());
        assert_eq!(validator.failed_validations, 1);
    }

    #[test]
    fn test_validate_allocation_too_large() {
        let mut validator = Validator::new();
        let result = validator.validate_allocation(2 * 1024 * 1024 * 1024, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_alignment_power_of_two() {
        let mut validator = Validator::new();
        assert!(validator.validate_allocation(1024, 16).is_ok());
        assert!(validator.validate_allocation(1024, 32).is_ok());
        assert!(validator.validate_allocation(1024, 64).is_ok());
    }

    #[test]
    fn test_validate_alignment_non_power_of_two() {
        let mut validator = Validator::new();
        let result = validator.validate_allocation(1024, 7);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_deallocation_valid() {
        let mut validator = Validator::new();
        let result = validator.validate_deallocation(0x1000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_deallocation_null() {
        let mut validator = Validator::new();
        let result = validator.validate_deallocation(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_irq_valid() {
        let mut validator = Validator::new();
        let result = validator.validate_irq(32);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_irq_out_of_range() {
        let mut validator = Validator::new();
        let result = validator.validate_irq(256);
        assert!(result.is_err());
    }

    #[test]
    fn test_validator_success_rate() {
        let mut validator = Validator::new();
        validator.validate_allocation(1024, 0).ok();
        validator.validate_allocation(0, 0).ok();
        let rate = validator.success_rate();
        assert!(rate > 40.0 && rate < 60.0);
    }

    // ========================================================================
    // MEMORY ALLOCATION TESTS
    // ========================================================================

    #[test]
    fn test_memory_allocator_new() {
        let allocator = LinuxMemoryAllocator::new();
        assert_eq!(allocator.allocation_count, 0);
        assert_eq!(allocator.total_allocated, 0);
        assert_eq!(allocator.active_allocations(), 0);
    }

    #[test]
    fn test_kmalloc_basic() {
        let mut allocator = LinuxMemoryAllocator::new();
        let driver_id = ObjectId::new();

        let result = allocator.kmalloc(driver_id, 1024, 0);
        assert!(result.is_ok());
        assert_eq!(allocator.allocation_count, 1);
        assert_eq!(allocator.total_allocated, 1024);
        assert_eq!(allocator.active_allocations(), 1);
    }

    #[test]
    fn test_kmalloc_multiple() {
        let mut allocator = LinuxMemoryAllocator::new();
        let driver_id = ObjectId::new();

        for i in 0..5 {
            let result = allocator.kmalloc(driver_id, (i + 1) * 1024, 0);
            assert!(result.is_ok());
        }

        assert_eq!(allocator.allocation_count, 5);
        assert_eq!(allocator.active_allocations(), 5);
    }

    #[test]
    fn test_kmalloc_too_large() {
        let mut allocator = LinuxMemoryAllocator::new();
        let driver_id = ObjectId::new();

        let result = allocator.kmalloc(driver_id, 200 * 1024, 0);
        assert!(result.is_err());
        assert_eq!(allocator.failed_allocations, 1);
    }

    #[test]
    fn test_kzalloc() {
        let mut allocator = LinuxMemoryAllocator::new();
        let driver_id = ObjectId::new();

        let addr = allocator.kzalloc(driver_id, 512, 0).unwrap();
        let alloc = allocator.get_allocation(addr).unwrap();
        assert!(alloc.is_zeroed);
    }

    #[test]
    fn test_vmalloc() {
        let mut allocator = LinuxMemoryAllocator::new();
        let driver_id = ObjectId::new();

        let result = allocator.vmalloc(driver_id, 10 * 1024 * 1024);
        assert!(result.is_ok());
        assert_eq!(allocator.allocation_count, 1);
    }

    #[test]
    fn test_dma_alloc() {
        let mut allocator = LinuxMemoryAllocator::new();
        let driver_id = ObjectId::new();

        let result = allocator.dma_alloc(driver_id, 4096, 16);
        assert!(result.is_ok());
    }

    #[test]
    fn test_kfree_valid() {
        let mut allocator = LinuxMemoryAllocator::new();
        let driver_id = ObjectId::new();

        let addr = allocator.kmalloc(driver_id, 1024, 0).unwrap();
        assert_eq!(allocator.active_allocations(), 1);

        let result = allocator.kfree(addr);
        assert!(result.is_ok());
        assert_eq!(allocator.active_allocations(), 0);
    }

    #[test]
    fn test_kfree_double_free() {
        let mut allocator = LinuxMemoryAllocator::new();
        let driver_id = ObjectId::new();

        let addr = allocator.kmalloc(driver_id, 1024, 0).unwrap();
        allocator.kfree(addr).unwrap();

        let result = allocator.kfree(addr);
        assert!(result.is_err());
    }

    #[test]
    fn test_kfree_invalid_pointer() {
        let mut allocator = LinuxMemoryAllocator::new();

        let result = allocator.kfree(0xDEADBEEF);
        assert!(result.is_err());
        assert_eq!(allocator.failed_allocations, 1);
    }

    #[test]
    fn test_memory_stats() {
        let mut allocator = LinuxMemoryAllocator::new();
        let driver_id = ObjectId::new();

        allocator.kmalloc(driver_id, 1024, 0).ok();
        allocator.kmalloc(driver_id, 2048, 0).ok();

        let stats = allocator.get_stats();
        assert_eq!(stats.total_allocated, 3072);
        assert_eq!(stats.active_allocations, 2);
        assert_eq!(stats.total_allocations, 2);
    }

    #[test]
    fn test_memory_peak_tracking() {
        let mut allocator = LinuxMemoryAllocator::new();
        let driver_id = ObjectId::new();

        allocator.kmalloc(driver_id, 1000, 0).ok();
        allocator.kmalloc(driver_id, 1000, 0).ok();
        assert_eq!(allocator.peak_usage(), 2000);

        let addr1 = allocator.allocations.keys().next().unwrap().clone();
        allocator.kfree(addr1).ok();
        assert_eq!(allocator.current_usage(), 1000);
        assert_eq!(allocator.peak_usage(), 2000);
    }

    #[test]
    fn test_find_memory_leaks() {
        let mut allocator = LinuxMemoryAllocator::new();
        let driver_id = ObjectId::new();

        allocator.kmalloc(driver_id, 1024, 0).ok();
        allocator.kmalloc(driver_id, 1024, 0).ok();

        let leaks = allocator.find_leaks();
        assert_eq!(leaks.len(), 2);
    }

    // ========================================================================
    // INTERRUPT TESTS
    // ========================================================================

    #[test]
    fn test_interrupt_manager_new() {
        let manager = InterruptManager::new();
        assert_eq!(manager.registered_interrupts(), 0);
        assert_eq!(manager.active_interrupts(), 0);
    }

    #[test]
    fn test_request_irq_basic() {
        let mut manager = InterruptManager::new();
        let driver_id = ObjectId::new();

        let result = manager.request_irq(driver_id, 32, IrqTrigger::Rising, 0);
        assert!(result.is_ok());
        assert_eq!(manager.registered_interrupts(), 1);
        assert_eq!(manager.active_interrupts(), 1);
    }

    #[test]
    fn test_request_irq_duplicate_non_shared() {
        let mut manager = InterruptManager::new();
        let driver1 = ObjectId::new();
        let driver2 = ObjectId::new();

        manager.request_irq(driver1, 32, IrqTrigger::Rising, 0).ok();
        let result = manager.request_irq(driver2, 32, IrqTrigger::Rising, 0);

        assert!(result.is_err());
    }

    #[test]
    fn test_request_irq_shared() {
        let mut manager = InterruptManager::new();
        let driver1 = ObjectId::new();
        let driver2 = ObjectId::new();

        const IRQF_SHARED: u32 = 0x00000080;

        manager.request_irq(driver1, 32, IrqTrigger::Rising, IRQF_SHARED).ok();
        let result = manager.request_irq(driver2, 32, IrqTrigger::Rising, IRQF_SHARED);

        assert!(result.is_ok());
        assert_eq!(manager.registered_interrupts(), 1);
    }

    #[test]
    fn test_free_irq() {
        let mut manager = InterruptManager::new();
        let driver_id = ObjectId::new();

        manager.request_irq(driver_id, 32, IrqTrigger::Rising, 0).ok();
        let result = manager.free_irq(32, driver_id);

        assert!(result.is_ok());
        assert_eq!(manager.registered_interrupts(), 0);
    }

    #[test]
    fn test_free_irq_wrong_driver() {
        let mut manager = InterruptManager::new();
        let driver1 = ObjectId::new();
        let driver2 = ObjectId::new();

        manager.request_irq(driver1, 32, IrqTrigger::Rising, 0).ok();
        let result = manager.free_irq(32, driver2);

        assert!(result.is_err());
    }

    #[test]
    fn test_enable_disable_irq() {
        let mut manager = InterruptManager::new();
        let driver_id = ObjectId::new();

        manager.request_irq(driver_id, 32, IrqTrigger::Rising, 0).ok();
        assert_eq!(manager.active_interrupts(), 1);

        manager.disable_irq(32).ok();
        assert_eq!(manager.active_interrupts(), 0);

        manager.enable_irq(32).ok();
        assert_eq!(manager.active_interrupts(), 1);
    }

    #[test]
    fn test_interrupt_handler_latency() {
        let mut handler = InterruptHandler::new(ObjectId::new(), 32, IrqTrigger::Rising);

        handler.record_call(100);
        handler.record_call(150);
        handler.record_call(200);

        assert_eq!(handler.call_count, 3);
        assert_eq!(handler.peak_latency_us, 200);
        assert_eq!(handler.avg_latency_us, 150);
    }

    #[test]
    fn test_interrupt_stats() {
        let mut manager = InterruptManager::new();
        let driver1 = ObjectId::new();
        let driver2 = ObjectId::new();

        manager.request_irq(driver1, 32, IrqTrigger::Rising, 0).ok();
        manager.request_irq(driver2, 33, IrqTrigger::Falling, 0).ok();

        let stats = manager.get_stats();
        assert_eq!(stats.total_registered, 2);
        assert_eq!(stats.active_interrupts, 2);
    }

    // ========================================================================
    // AUDIT LOG TESTS
    // ========================================================================

    #[test]
    fn test_audit_log_new() {
        let log = AuditLog::new(100);
        assert_eq!(log.total_entries(), 0);
        assert_eq!(log.entries().len(), 0);
    }

    #[test]
    fn test_audit_log_entry() {
        let mut log = AuditLog::new(100);
        let driver_id = ObjectId::new();

        let entry = AuditEntry::new(driver_id, "kmalloc", "alloc 1024 bytes")
            .with_result("success");

        log.log(entry);
        assert_eq!(log.total_entries(), 1);
        assert_eq!(log.info_count(), 1);
    }

    #[test]
    fn test_audit_log_levels() {
        let mut log = AuditLog::new(100);
        let driver_id = ObjectId::new();

        log.log(AuditEntry::new(driver_id, "api1", "op1").with_level(AuditLevel::Info));
        log.log(AuditEntry::new(driver_id, "api2", "op2").with_level(AuditLevel::Warning));
        log.log(AuditEntry::new(driver_id, "api3", "op3").with_level(AuditLevel::Error));
        log.log(AuditEntry::new(driver_id, "api4", "op4").with_level(AuditLevel::Critical));

        let stats = log.stats();
        assert_eq!(stats.info_count, 1);
        assert_eq!(stats.warning_count, 1);
        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.critical_count, 1);
    }

    #[test]
    fn test_audit_log_max_entries() {
        let mut log = AuditLog::new(5);
        let driver_id = ObjectId::new();

        for i in 0..10 {
            let entry = AuditEntry::new(driver_id, "api", &format!("op{}", i));
            log.log(entry);
        }

        assert_eq!(log.entries().len(), 5);
        assert_eq!(log.total_entries(), 10);
    }

    #[test]
    fn test_audit_entries_by_driver() {
        let mut log = AuditLog::new(100);
        let driver1 = ObjectId::new();
        let driver2 = ObjectId::new();

        log.log(AuditEntry::new(driver1, "api1", "op1"));
        log.log(AuditEntry::new(driver1, "api2", "op2"));
        log.log(AuditEntry::new(driver2, "api3", "op3"));

        let entries = log.entries_by_driver(driver1);
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_audit_entries_by_level() {
        let mut log = AuditLog::new(100);
        let driver_id = ObjectId::new();

        log.log(AuditEntry::new(driver_id, "api1", "op1").with_level(AuditLevel::Error));
        log.log(AuditEntry::new(driver_id, "api2", "op2").with_level(AuditLevel::Error));
        log.log(AuditEntry::new(driver_id, "api3", "op3").with_level(AuditLevel::Info));

        let entries = log.entries_by_level(AuditLevel::Error);
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_audit_filter() {
        let mut log = AuditLog::new(100);
        let driver_id = ObjectId::new();

        log.log(AuditEntry::new(driver_id, "kmalloc", "op1").with_level(AuditLevel::Info).with_duration(100));
        log.log(AuditEntry::new(driver_id, "kmalloc", "op2").with_level(AuditLevel::Error).with_duration(50));
        log.log(AuditEntry::new(driver_id, "kfree", "op3").with_level(AuditLevel::Info).with_duration(200));

        let filter = AuditFilter::new()
            .with_api("kmalloc")
            .with_min_latency(100);

        let entries = log.entries();
        let matching: Vec<_> = entries.iter().filter(|e| filter.matches(e)).collect();
        assert_eq!(matching.len(), 1);
    }

    #[test]
    fn test_audit_error_rate() {
        let mut log = AuditLog::new(100);
        let driver_id = ObjectId::new();

        log.log(AuditEntry::new(driver_id, "api1", "op1").with_level(AuditLevel::Info));
        log.log(AuditEntry::new(driver_id, "api2", "op2").with_level(AuditLevel::Error));
        log.log(AuditEntry::new(driver_id, "api3", "op3").with_level(AuditLevel::Critical));

        let stats = log.stats();
        assert!(stats.error_rate > 60.0 && stats.error_rate < 70.0);
    }

    // ========================================================================
    // DEVICE TRANSLATION TESTS
    // ========================================================================

    #[test]
    fn test_pci_device_id_new() {
        let pci_id = PciDeviceId::new(0x8086, 0x1234);
        assert_eq!(pci_id.vendor, 0x8086);
        assert_eq!(pci_id.device, 0x1234);
        assert_eq!(pci_id.subvendor, 0xFFFF);
        assert_eq!(pci_id.subdevice, 0xFFFF);
    }

    #[test]
    fn test_pci_device_id_matches_exact() {
        let id1 = PciDeviceId::new(0x8086, 0x1234);
        let id2 = PciDeviceId::new(0x8086, 0x1234);

        assert!(id1.matches(&id2));
        assert!(id2.matches(&id1));
    }

    #[test]
    fn test_pci_device_id_matches_wildcard() {
        let wildcard = PciDeviceId {
            vendor: 0xFFFF,
            device: 0x1234,
            subvendor: 0xFFFF,
            subdevice: 0xFFFF,
        };
        let specific = PciDeviceId::new(0x8086, 0x1234);

        assert!(wildcard.matches(&specific));
    }

    #[test]
    fn test_pci_device_bdf() {
        let pci_id = PciDeviceId::new(0x8086, 0x1234);
        let device = PciDevice::new(pci_id, 5, 10, 2);

        let bdf = device.bdf();
        let expected = ((5 as u32) << 16) | ((10 as u32) << 11) | 2;
        assert_eq!(bdf, expected);
    }

    #[test]
    fn test_pci_device_enable_disable() {
        let pci_id = PciDeviceId::new(0x8086, 0x1234);
        let mut device = PciDevice::new(pci_id, 5, 10, 2);

        assert!(!device.enabled);
        device.enable();
        assert!(device.enabled);
        device.disable();
        assert!(!device.enabled);
    }

    #[test]
    fn test_pci_device_add_bar() {
        let pci_id = PciDeviceId::new(0x8086, 0x1234);
        let mut device = PciDevice::new(pci_id, 5, 10, 2);

        device.add_bar(0xF0000000, 0x10000);
        device.add_bar(0xF0010000, 0x1000);

        assert_eq!(device.bar_regions.len(), 2);
        assert_eq!(device.bar_regions[0], (0xF0000000, 0x10000));
    }

    #[test]
    fn test_pci_driver_new() {
        let driver = PciDriver::new("intel_ether");
        assert_eq!(driver.driver_name, "intel_ether");
        assert_eq!(driver.supported_devices.len(), 0);
        assert_eq!(driver.probed_devices, 0);
    }

    #[test]
    fn test_pci_driver_add_device() {
        let mut driver = PciDriver::new("intel_ether");
        let id1 = PciDeviceId::new(0x8086, 0x1234);
        let id2 = PciDeviceId::new(0x8086, 0x5678);

        driver.add_device(id1);
        driver.add_device(id2);

        assert_eq!(driver.supported_devices.len(), 2);
    }

    #[test]
    fn test_pci_driver_supports_device() {
        let mut driver = PciDriver::new("intel_ether");
        let supported = PciDeviceId::new(0x8086, 0x1234);
        let unsupported = PciDeviceId::new(0x1000, 0x9999);

        driver.add_device(supported);

        assert!(driver.supports_device(&supported));
        assert!(!driver.supports_device(&unsupported));
    }

    #[test]
    fn test_pci_driver_probe_tracking() {
        let mut driver = PciDriver::new("intel_ether");

        driver.record_probe(true);
        driver.record_probe(true);
        driver.record_probe(false);

        assert_eq!(driver.probed_devices, 3);
        assert_eq!(driver.successful_probes, 2);
        assert_eq!(driver.failed_probes, 1);
    }

    #[test]
    fn test_device_bus_new() {
        let bus = DeviceBus::new(BusType::Pci, "pci0");
        assert_eq!(bus.bus_type, BusType::Pci);
        assert_eq!(bus.bus_name, "pci0");
        assert_eq!(bus.device_count(), 0);
        assert_eq!(bus.driver_count(), 0);
    }

    #[test]
    fn test_device_bus_add_device() {
        let mut bus = DeviceBus::new(BusType::Pci, "pci0");
        let dev_id = ObjectId::new();

        bus.add_device(dev_id);
        assert_eq!(bus.device_count(), 1);

        bus.add_device(dev_id);
        assert_eq!(bus.device_count(), 1);  // No duplicates
    }

    #[test]
    fn test_device_bus_remove_device() {
        let mut bus = DeviceBus::new(BusType::Pci, "pci0");
        let dev_id = ObjectId::new();

        bus.add_device(dev_id);
        bus.remove_device(dev_id);
        assert_eq!(bus.device_count(), 0);
    }

    #[test]
    fn test_device_manager_new() {
        let manager = DeviceManager::new();
        assert_eq!(manager.total_devices, 0);
        assert_eq!(manager.total_drivers, 0);
    }

    #[test]
    fn test_device_manager_register_pci_driver() {
        let mut manager = DeviceManager::new();
        let device_ids = vec![PciDeviceId::new(0x8086, 0x1234)];

        let result = manager.register_pci_driver("intel_ether", device_ids);
        assert!(result.is_ok());
        assert_eq!(manager.total_drivers, 1);
    }

    #[test]
    fn test_device_manager_register_pci_device() {
        let mut manager = DeviceManager::new();
        let pci_id = PciDeviceId::new(0x8086, 0x1234);

        let result = manager.register_pci_device(pci_id, 0, 15, 0);
        assert!(result.is_ok());
        assert_eq!(manager.total_devices, 1);
    }

    #[test]
    fn test_device_manager_driver_device_matching() {
        let mut manager = DeviceManager::new();
        let device_ids = vec![PciDeviceId::new(0x8086, 0x1234)];

        manager.register_pci_driver("intel_ether", device_ids).ok();
        let device_id = manager.register_pci_device(PciDeviceId::new(0x8086, 0x1234), 0, 15, 0).ok();

        if let Some(dev_id) = device_id {
            let driver = manager.get_driver_for_device(dev_id);
            assert!(driver.is_some());
        }
    }

    #[test]
    fn test_device_manager_register_bus() {
        let mut manager = DeviceManager::new();

        let result = manager.register_bus(BusType::Pci, "pci0");
        assert!(result.is_ok());
    }

    #[test]
    fn test_device_manager_bus_add_device() {
        let mut manager = DeviceManager::new();
        let bus_id = manager.register_bus(BusType::Pci, "pci0").unwrap();
        let dev_id = ObjectId::new();

        let result = manager.bus_add_device(bus_id, dev_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_device_manager_find_by_vendor() {
        let mut manager = DeviceManager::new();

        manager.register_pci_device(PciDeviceId::new(0x8086, 0x1234), 0, 15, 0).ok();
        manager.register_pci_device(PciDeviceId::new(0x8086, 0x5678), 0, 16, 0).ok();
        manager.register_pci_device(PciDeviceId::new(0x1000, 0x9999), 0, 17, 0).ok();

        let devices = manager.find_devices_by_vendor(0x8086);
        assert_eq!(devices.len(), 2);
    }

    #[test]
    fn test_device_manager_enable_disable() {
        let mut manager = DeviceManager::new();
        let pci_id = PciDeviceId::new(0x8086, 0x1234);
        manager.register_pci_device(pci_id, 0, 15, 0).ok();

        let bdf = ((0 as u32) << 16) | ((15 as u32) << 11) | 0;
        manager.enable_device(bdf).ok();

        if let Some(device) = manager.get_device_by_bdf(bdf) {
            assert!(device.enabled);
        }
    }

    #[test]
    fn test_device_manager_stats() {
        let mut manager = DeviceManager::new();

        manager.register_pci_driver("driver1", vec![PciDeviceId::new(0x8086, 0x1234)]).ok();
        manager.register_pci_driver("driver2", vec![PciDeviceId::new(0x1000, 0x5678)]).ok();
        manager.register_pci_device(PciDeviceId::new(0x8086, 0x1234), 0, 15, 0).ok();

        let stats = manager.get_stats();
        assert_eq!(stats.total_drivers, 2);
        assert_eq!(stats.registered_devices, 1);
    }

    #[test]
    fn test_block_device_new() {
        let device = BlockDevice::new(BlockDeviceType::HardDrive, 8, 0, 1024 * 1024 * 1024);
        assert_eq!(device.major_number, 8);
        assert_eq!(device.minor_number, 0);
        assert_eq!(device.capacity_bytes, 1024 * 1024 * 1024);
        assert!(!device.registered);
    }

    #[test]
    fn test_block_device_register() {
        let mut device = BlockDevice::new(BlockDeviceType::HardDrive, 8, 0, 1024 * 1024 * 1024);
        assert!(!device.registered);

        device.register();
        assert!(device.registered);

        device.unregister();
        assert!(!device.registered);
    }

    #[test]
    fn test_block_device_manager_register() {
        let mut manager = BlockDeviceManager::new();
        let device = BlockDevice::new(BlockDeviceType::HardDrive, 8, 0, 1024 * 1024 * 1024);

        let result = manager.register_block_device(device);
        assert!(result.is_ok());
        assert_eq!(manager.total_registered, 1);
    }

    #[test]
    fn test_block_device_manager_duplicate() {
        let mut manager = BlockDeviceManager::new();
        let device1 = BlockDevice::new(BlockDeviceType::HardDrive, 8, 0, 1024 * 1024 * 1024);
        let device2 = BlockDevice::new(BlockDeviceType::HardDrive, 8, 0, 1024 * 1024 * 1024);

        manager.register_block_device(device1).ok();
        let result = manager.register_block_device(device2);

        assert!(result.is_err());
    }

    #[test]
    fn test_network_device_new() {
        let device = NetworkDevice::new(NetDeviceType::Ethernet, "eth0");
        assert_eq!(device.name, "eth0");
        assert_eq!(device.device_type, NetDeviceType::Ethernet);
        assert_eq!(device.mtu, 1500);
        assert!(!device.registered);
    }

    #[test]
    fn test_network_device_set_mac() {
        let mut device = NetworkDevice::new(NetDeviceType::Ethernet, "eth0");
        let mac = [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE];

        device.set_mac(mac);
        assert_eq!(device.mac_address, mac);
    }

    #[test]
    fn test_network_device_manager_register() {
        let mut manager = NetworkDeviceManager::new();
        let device = NetworkDevice::new(NetDeviceType::Ethernet, "eth0");

        let result = manager.register_net_device(device);
        assert!(result.is_ok());
        assert_eq!(manager.total_registered, 1);
    }

    #[test]
    fn test_network_device_manager_duplicate() {
        let mut manager = NetworkDeviceManager::new();
        let device1 = NetworkDevice::new(NetDeviceType::Ethernet, "eth0");
        let device2 = NetworkDevice::new(NetDeviceType::Ethernet, "eth0");

        manager.register_net_device(device1).ok();
        let result = manager.register_net_device(device2);

        assert!(result.is_err());
    }

    #[test]
    fn test_network_device_manager_list() {
        let mut manager = NetworkDeviceManager::new();

        manager.register_net_device(NetworkDevice::new(NetDeviceType::Ethernet, "eth0")).ok();
        manager.register_net_device(NetworkDevice::new(NetDeviceType::Ethernet, "eth1")).ok();
        manager.register_net_device(NetworkDevice::new(NetDeviceType::Loopback, "lo")).ok();

        let devices = manager.list_devices();
        assert_eq!(devices.len(), 3);
    }

    // ========================================================================
    // SECURITY & CAPABILITY TESTS
    // ========================================================================

    #[test]
    fn test_capability_grant_new() {
        let driver_id = ObjectId::new();
        let grant = CapabilityGrant::new(driver_id, Capability::AllocateMemory, PermissionTier::Low);

        assert_eq!(grant.driver_id, driver_id);
        assert_eq!(grant.capability, Capability::AllocateMemory);
        assert_eq!(grant.tier, PermissionTier::Low);
    }

    #[test]
    fn test_capability_grant_with_duration() {
        let driver_id = ObjectId::new();
        let grant = CapabilityGrant::new(driver_id, Capability::AllocateMemory, PermissionTier::Low)
            .with_duration(1_800_000).unwrap();

        assert_eq!(grant.lifetime_ms(), 1_800_000);
    }

    #[test]
    fn test_capability_grant_duration_limit() {
        let driver_id = ObjectId::new();
        let grant = CapabilityGrant::new(driver_id, Capability::AllocateMemory, PermissionTier::High);

        // High tier max is 2 hours (7,200,000 ms)
        let result = grant.with_duration(14_400_000);  // 4 hours
        assert!(result.is_err());
    }

    #[test]
    fn test_capability_grant_validity() {
        let driver_id = ObjectId::new();
        let mut grant = CapabilityGrant::new(driver_id, Capability::AllocateMemory, PermissionTier::Low);
        grant.granted_at_ms = 1000;
        grant.expires_at_ms = 5000;

        assert!(grant.is_valid(2000));
        assert!(grant.is_valid(4999));
        assert!(!grant.is_valid(5000));
        assert!(!grant.is_valid(6000));
    }

    #[test]
    fn test_capability_grant_time_remaining() {
        let driver_id = ObjectId::new();
        let mut grant = CapabilityGrant::new(driver_id, Capability::AllocateMemory, PermissionTier::Low);
        grant.granted_at_ms = 1000;
        grant.expires_at_ms = 5000;

        assert_eq!(grant.time_remaining_ms(1000), 4000);
        assert_eq!(grant.time_remaining_ms(3000), 2000);
        assert_eq!(grant.time_remaining_ms(5000), 0);
    }

    #[test]
    fn test_permission_tier_durations() {
        assert_eq!(PermissionTier::Low.max_duration_ms(), 3_600_000);
        assert_eq!(PermissionTier::Medium.max_duration_ms(), 86_400_000);
        assert_eq!(PermissionTier::High.max_duration_ms(), 7_200_000);
        assert_eq!(PermissionTier::Critical.max_duration_ms(), 1_800_000);
    }

    #[test]
    fn test_capability_manager_grant() {
        let mut manager = CapabilityManager::new();
        let driver_id = ObjectId::new();
        let grant = CapabilityGrant::new(driver_id, Capability::AllocateMemory, PermissionTier::Low);

        let result = manager.grant(grant);
        assert!(result.is_ok());
        assert_eq!(manager.total_grants, 1);
    }

    #[test]
    fn test_capability_manager_has_capability() {
        let mut manager = CapabilityManager::new();
        let driver_id = ObjectId::new();
        let mut grant = CapabilityGrant::new(driver_id, Capability::AllocateMemory, PermissionTier::Low);
        grant.granted_at_ms = 1000;
        grant.expires_at_ms = 5000;

        manager.grant(grant).ok();

        assert!(manager.has_capability(driver_id, Capability::AllocateMemory, 2000));
        assert!(!manager.has_capability(driver_id, Capability::AllocateMemory, 6000));
    }

    #[test]
    fn test_capability_manager_revoke() {
        let mut manager = CapabilityManager::new();
        let driver_id = ObjectId::new();
        let grant = CapabilityGrant::new(driver_id, Capability::AllocateMemory, PermissionTier::Low);

        manager.grant(grant).ok();
        assert_eq!(manager.total_grants, 1);

        let result = manager.revoke(driver_id, Capability::AllocateMemory);
        assert!(result.is_ok());
        assert_eq!(manager.revoked_grants, 1);
    }

    #[test]
    fn test_security_policy_new() {
        let driver_id = ObjectId::new();
        let policy = SecurityPolicy::new(driver_id, SecurityLevel::Balanced);

        assert_eq!(policy.driver_id, driver_id);
        assert_eq!(policy.security_level, SecurityLevel::Balanced);
        assert_eq!(policy.max_capability_tier, PermissionTier::High);
    }

    #[test]
    fn test_security_policy_tier_limits() {
        let driver_id = ObjectId::new();

        let permissive = SecurityPolicy::new(driver_id, SecurityLevel::Permissive);
        assert_eq!(permissive.max_capability_tier, PermissionTier::Critical);

        let strict = SecurityPolicy::new(driver_id, SecurityLevel::Strict);
        assert_eq!(strict.max_capability_tier, PermissionTier::Medium);

        let critical = SecurityPolicy::new(driver_id, SecurityLevel::Critical);
        assert_eq!(critical.max_capability_tier, PermissionTier::Low);
    }

    #[test]
    fn test_security_policy_allows_tier() {
        let driver_id = ObjectId::new();
        let policy = SecurityPolicy::new(driver_id, SecurityLevel::Balanced);

        assert!(policy.allows_tier(PermissionTier::Low));
        assert!(policy.allows_tier(PermissionTier::Medium));
        assert!(policy.allows_tier(PermissionTier::High));
        assert!(!policy.allows_tier(PermissionTier::Critical));
    }

    #[test]
    fn test_security_context_check_unrestricted() {
        let driver_id = ObjectId::new();
        let policy = SecurityPolicy::new(driver_id, SecurityLevel::Unrestricted);
        let mut context = SecurityContext::new(driver_id, policy);

        let result = context.check_operation(Capability::AllocateMemory, 1000);
        assert!(result.is_ok());
        assert_eq!(context.approved_operations, 1);
    }

    #[test]
    fn test_security_context_check_with_grant() {
        let driver_id = ObjectId::new();
        let policy = SecurityPolicy::new(driver_id, SecurityLevel::Strict);
        let mut context = SecurityContext::new(driver_id, policy);

        let mut grant = CapabilityGrant::new(driver_id, Capability::AllocateMemory, PermissionTier::Low);
        grant.granted_at_ms = 1000;
        grant.expires_at_ms = 5000;

        context.capability_manager.grant(grant).ok();

        let result = context.check_operation(Capability::AllocateMemory, 2000);
        assert!(result.is_ok());
        assert_eq!(context.approved_operations, 1);
    }

    #[test]
    fn test_security_context_denial_rate() {
        let driver_id = ObjectId::new();
        let policy = SecurityPolicy::new(driver_id, SecurityLevel::Strict);
        let mut context = SecurityContext::new(driver_id, policy);

        let _ = context.check_operation(Capability::AllocateMemory, 1000);
        let _ = context.check_operation(Capability::RegisterDevice, 1000);
        let _ = context.check_operation(Capability::NetworkAccess, 1000);

        assert_eq!(context.denied_operations, 3);
        assert_eq!(context.denial_rate(), 100.0);
    }

    #[test]
    fn test_security_enforcer_register() {
        let mut enforcer = SecurityEnforcer::new();
        let driver_id = ObjectId::new();
        let policy = SecurityPolicy::new(driver_id, SecurityLevel::Balanced);

        let result = enforcer.register_driver(driver_id, policy);
        assert!(result.is_ok());
        assert_eq!(enforcer.contexts.len(), 1);
    }

    #[test]
    fn test_security_enforcer_enforce() {
        let mut enforcer = SecurityEnforcer::new();
        let driver_id = ObjectId::new();
        let policy = SecurityPolicy::new(driver_id, SecurityLevel::Unrestricted);

        let context_id = enforcer.register_driver(driver_id, policy).unwrap();

        let result = enforcer.enforce(context_id, Capability::AllocateMemory, 1000);
        assert!(result.is_ok());
        assert_eq!(enforcer.total_checks, 1);
    }

    #[test]
    fn test_security_enforcer_denial() {
        let mut enforcer = SecurityEnforcer::new();
        let driver_id = ObjectId::new();
        let policy = SecurityPolicy::new(driver_id, SecurityLevel::Strict);

        let context_id = enforcer.register_driver(driver_id, policy).unwrap();

        let result = enforcer.enforce(context_id, Capability::AllocateMemory, 1000);
        assert!(result.is_err());
        assert_eq!(enforcer.total_denials, 1);
    }

    #[test]
    fn test_permission_checker_caching() {
        let mut enforcer = SecurityEnforcer::new();
        let mut checker = PermissionChecker::new();
        let driver_id = ObjectId::new();
        let policy = SecurityPolicy::new(driver_id, SecurityLevel::Unrestricted);

        let context_id = enforcer.register_driver(driver_id, policy).unwrap();

        let _ = checker.check(&mut enforcer, context_id, Capability::AllocateMemory, 1000);
        let _ = checker.check(&mut enforcer, context_id, Capability::AllocateMemory, 1000);

        assert_eq!(checker.checks_performed, 2);
        assert_eq!(checker.capability_cache.len(), 1);  // Cached result
    }

    #[test]
    fn test_permission_checker_cache_clear() {
        let mut enforcer = SecurityEnforcer::new();
        let mut checker = PermissionChecker::new();
        let driver_id = ObjectId::new();
        let policy = SecurityPolicy::new(driver_id, SecurityLevel::Unrestricted);

        let context_id = enforcer.register_driver(driver_id, policy).unwrap();

        let _ = checker.check(&mut enforcer, context_id, Capability::AllocateMemory, 1000);
        assert_eq!(checker.capability_cache.len(), 1);

        checker.clear_cache(context_id);
        assert_eq!(checker.capability_cache.len(), 0);
    }

    #[test]
    fn test_capability_manager_cleanup_expired() {
        let mut manager = CapabilityManager::new();
        let driver_id = ObjectId::new();

        let mut grant1 = CapabilityGrant::new(driver_id, Capability::AllocateMemory, PermissionTier::Low);
        grant1.granted_at_ms = 1000;
        grant1.expires_at_ms = 2000;

        let mut grant2 = CapabilityGrant::new(driver_id, Capability::RegisterDevice, PermissionTier::Low);
        grant2.granted_at_ms = 1000;
        grant2.expires_at_ms = 5000;

        manager.grant(grant1).ok();
        manager.grant(grant2).ok();

        manager.cleanup_expired(3000);
        assert_eq!(manager.expired_grants, 1);
    }

    #[test]
    fn test_reauth_requirement() {
        let driver_id = ObjectId::new();
        let grant = CapabilityGrant::new(driver_id, Capability::RegisterDevice, PermissionTier::High)
            .with_reauth(ReauthMethod::Biometric);

        assert!(grant.reauth_required);
        assert_eq!(grant.reauth_method, ReauthMethod::Biometric);
    }

    #[test]
    fn test_enforcer_stats() {
        let mut enforcer = SecurityEnforcer::new();
        let driver_id = ObjectId::new();
        let policy = SecurityPolicy::new(driver_id, SecurityLevel::Unrestricted);

        let context_id = enforcer.register_driver(driver_id, policy).unwrap();

        enforcer.enforce(context_id, Capability::AllocateMemory, 1000).ok();
        enforcer.enforce(context_id, Capability::RegisterDevice, 1000).ok();

        let stats = enforcer.get_stats();
        assert_eq!(stats.total_checks, 2);
        assert_eq!(stats.contexts, 1);
    }

    #[test]
    fn test_permission_checker_stats() {
        let mut enforcer = SecurityEnforcer::new();
        let mut checker = PermissionChecker::new();
        let driver_id = ObjectId::new();
        let policy = SecurityPolicy::new(driver_id, SecurityLevel::Unrestricted);

        let context_id = enforcer.register_driver(driver_id, policy).unwrap();

        let _ = checker.check(&mut enforcer, context_id, Capability::AllocateMemory, 1000);
        let _ = checker.check(&mut enforcer, context_id, Capability::RegisterDevice, 1000);

        let stats = checker.get_stats();
        assert_eq!(stats.checks_performed, 2);
        assert_eq!(stats.checks_passed, 2);
        assert!(stats.success_rate > 99.0);
    }
}
