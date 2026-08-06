// SHER LKI: Comprehensive Tests

#[cfg(test)]
mod tests {
    use crate::memory_translation::LinuxMemoryAllocator;
    use crate::interrupt_translation::{InterruptManager, InterruptHandler, IrqTrigger};
    use crate::audit::{AuditLog, AuditEntry, AuditLevel, AuditFilter};
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
}
