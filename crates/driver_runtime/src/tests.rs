// SHER Driver Runtime: Comprehensive Tests

#[cfg(test)]
mod tests {
    use crate::*;
    use sher_common::ObjectId;

    // ========================================================================
    // CONTAINER TESTS
    // ========================================================================

    #[test]
    fn test_driver_container_new() {
        let container = DriverContainer::new("test_driver");
        assert_eq!(container.driver_name, "test_driver");
        assert_eq!(container.state, ContainerState::Created);
        assert!(!container.is_operational());
    }

    #[test]
    fn test_container_start() {
        let mut container = DriverContainer::new("test");
        assert!(container.start().is_ok());
        assert_eq!(container.state, ContainerState::Running);
        assert!(container.is_operational());
        assert_eq!(container.telemetry.start_count, 1);
    }

    #[test]
    fn test_container_stop() {
        let mut container = DriverContainer::new("test");
        container.start().ok();
        assert!(container.stop().is_ok());
        assert_eq!(container.state, ContainerState::Stopped);
        assert!(!container.is_operational());
    }

    #[test]
    fn test_container_pause_resume() {
        let mut container = DriverContainer::new("test");
        container.start().ok();

        assert!(container.pause().is_ok());
        assert_eq!(container.state, ContainerState::Paused);

        assert!(container.resume().is_ok());
        assert_eq!(container.state, ContainerState::Running);
    }

    #[test]
    fn test_container_state_transitions() {
        let mut container = DriverContainer::new("test");
        assert_eq!(container.state, ContainerState::Created);

        assert!(container.start().is_ok());
        assert_eq!(container.state, ContainerState::Running);

        assert!(container.pause().is_ok());
        assert_eq!(container.state, ContainerState::Paused);

        assert!(container.resume().is_ok());
        assert_eq!(container.state, ContainerState::Running);

        assert!(container.stop().is_ok());
        assert_eq!(container.state, ContainerState::Stopped);
    }

    #[test]
    fn test_container_capabilities() {
        let mut container = DriverContainer::new("test");

        assert!(!container.has_capability(DriverCapability::Admin));

        container.grant_capability(DriverCapability::Admin);
        assert!(container.has_capability(DriverCapability::Admin));

        container.revoke_capability(DriverCapability::Admin);
        assert!(!container.has_capability(DriverCapability::Admin));
    }

    #[test]
    fn test_container_resource_limits() {
        let limits = ResourceLimits {
            memory_limit_bytes: 512 * 1024 * 1024,
            cpu_quota_percent: 75,
            max_file_descriptors: 1024,
            max_threads: 32,
            network_bandwidth_kbps: 25000,
        };

        let container = DriverContainer::new("test").with_limits(limits);
        assert_eq!(
            container.resource_limits.memory_limit_bytes,
            512 * 1024 * 1024
        );
        assert_eq!(container.resource_limits.cpu_quota_percent, 75);
    }

    #[test]
    fn test_container_environment() {
        let mut container = DriverContainer::new("test");
        container.set_env("KEY".to_string(), "value".to_string());

        assert_eq!(container.get_env("KEY"), Some(&"value".to_string()));
        assert_eq!(container.get_env("MISSING"), None);
    }

    #[test]
    fn test_container_error_recording() {
        let mut container = DriverContainer::new("test");
        container.start().ok();

        container.record_error("Test error".to_string());
        assert_eq!(container.state, ContainerState::Error);
        assert!(container.telemetry.last_error.is_some());
    }

    #[test]
    fn test_container_crash_recording() {
        let mut container = DriverContainer::new("test");
        container.start().ok();

        container.record_crash("Segmentation fault".to_string());
        assert_eq!(container.state, ContainerState::Crashed);
        assert_eq!(container.telemetry.crash_count, 1);
    }

    #[test]
    fn test_container_pool_register() {
        let mut pool = ContainerPool::new();
        let container = DriverContainer::new("driver1");
        let id = container.id;

        pool.register(container);
        assert!(pool.get(id).is_some());
        assert_eq!(pool.count(), 1);
    }

    #[test]
    fn test_container_pool_find_by_name() {
        let mut pool = ContainerPool::new();
        let container = DriverContainer::new("test_driver");
        pool.register(container);

        let found = pool.find_by_name("test_driver");
        assert!(found.is_some());
    }

    #[test]
    fn test_container_pool_unregister() {
        let mut pool = ContainerPool::new();
        let container = DriverContainer::new("driver1");
        let id = container.id;

        pool.register(container);
        pool.unregister(id);

        assert!(pool.get(id).is_none());
        assert_eq!(pool.count(), 0);
    }

    #[test]
    fn test_container_pool_list_operational() {
        let mut pool = ContainerPool::new();

        let mut c1 = DriverContainer::new("driver1");
        c1.start().ok();
        pool.register(c1);

        let c2 = DriverContainer::new("driver2");
        pool.register(c2);

        let operational = pool.list_operational();
        assert_eq!(operational.len(), 1);
    }

    // ========================================================================
    // DRIVER LOADER TESTS
    // ========================================================================

    #[test]
    fn test_driver_loader_new() {
        let loader = DriverLoader::new();
        assert_eq!(loader.count_loaded(), 0);
    }

    #[test]
    fn test_loader_load_driver() {
        let mut loader = DriverLoader::new();
        let result = loader.load_driver("driver.ko", "test_driver");

        assert!(result.is_ok());
        let container = result.unwrap();
        assert_eq!(container.driver_name, "test_driver");
        assert_eq!(loader.count_loaded(), 1);
    }

    #[test]
    fn test_loader_load_duplicate() {
        let mut loader = DriverLoader::new();
        loader.load_driver("driver1.ko", "driver1").ok();

        let result = loader.load_driver("driver2.ko", "driver1");
        assert!(result.is_err());
    }

    #[test]
    fn test_loader_unload_driver() {
        let mut loader = DriverLoader::new();
        loader.load_driver("driver.ko", "test_driver").ok();

        assert!(loader.unload_driver("test_driver").is_ok());
        assert_eq!(loader.count_loaded(), 0);
    }

    #[test]
    fn test_loader_get_driver() {
        let mut loader = DriverLoader::new();
        loader.load_driver("driver.ko", "test_driver").ok();

        let driver = loader.get_driver("test_driver");
        assert!(driver.is_some());
        assert_eq!(driver.unwrap().driver_name, "test_driver");
    }

    #[test]
    fn test_loader_reload_driver() {
        let mut loader = DriverLoader::new();
        loader.load_driver("driver.ko", "test_driver").ok();

        let result = loader.reload_driver("test_driver", "driver_new.ko");
        assert!(result.is_ok());
        assert_eq!(loader.count_loaded(), 1);
    }

    #[test]
    fn test_loader_linux_driver_info() {
        let mut loader = DriverLoader::new();
        loader.load_driver("linux_driver.ko", "linux_driver").ok();

        let info = loader.get_linux_driver("linux_driver");
        assert!(info.is_some());
        assert_eq!(info.unwrap().name, "linux_driver");
    }

    #[test]
    fn test_loader_register_manifest() {
        let mut loader = DriverLoader::new();
        let manifest = DriverManifest {
            name: "eth_driver".to_string(),
            version: "1.0".to_string(),
            compatible_devices: vec![(0x8086, 0x1234)],
            required_capabilities: vec![DriverCapability::ReadDma, DriverCapability::WriteDma],
            memory_required_bytes: 256 * 1024 * 1024,
            entry_point: "eth_init".to_string(),
        };

        loader.register_manifest(manifest);
        let retrieved = loader.get_manifest("eth_driver");
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_loader_find_drivers_for_device() {
        let mut loader = DriverLoader::new();
        loader.load_driver("eth.ko", "eth_driver").ok();

        let manifest = DriverManifest {
            name: "eth_driver".to_string(),
            version: "1.0".to_string(),
            compatible_devices: vec![(0x8086, 0x1234), (0x8086, 0x5678)],
            required_capabilities: vec![],
            memory_required_bytes: 256 * 1024 * 1024,
            entry_point: "init".to_string(),
        };

        loader.register_manifest(manifest);
        let drivers = loader.find_drivers_for_device(0x8086, 0x1234);
        assert_eq!(drivers.len(), 1);
    }

    // ========================================================================
    // TRANSLATION ENGINE TESTS
    // ========================================================================

    #[test]
    fn test_translation_engine_new() {
        let engine = TranslationEngine::new();
        assert!(engine.translate("kmalloc").is_some());
        assert!(engine.translate("kfree").is_some());
    }

    #[test]
    fn test_translation_kmalloc() {
        let engine = TranslationEngine::new();
        let translation = engine.translate("kmalloc");
        assert!(translation.is_some());
        assert_eq!(translation.unwrap(), "master_allocate");
    }

    #[test]
    fn test_translation_kfree() {
        let engine = TranslationEngine::new();
        let translation = engine.translate("kfree");
        assert!(translation.is_some());
        assert_eq!(translation.unwrap(), "master_deallocate");
    }

    #[test]
    fn test_translation_unknown() {
        let engine = TranslationEngine::new();
        let translation = engine.translate("unknown_api");
        assert!(translation.is_none());
    }

    #[test]
    fn test_translation_validation_level() {
        let engine = TranslationEngine::new();
        let level = engine.get_validation_level("kmalloc");
        assert_eq!(level, Some(ValidationLevel::Strict));
    }

    #[test]
    fn test_translation_record_call() {
        let mut engine = TranslationEngine::new();
        engine.record_call("kmalloc");
        engine.record_call("kmalloc");

        assert_eq!(engine.get_call_count("kmalloc"), 2);
    }

    #[test]
    fn test_translation_record_error() {
        let mut engine = TranslationEngine::new();
        engine.record_call("kmalloc");
        engine.record_error("kmalloc");

        assert_eq!(engine.get_call_count("kmalloc"), 1);
        assert_eq!(engine.get_error_count("kmalloc"), 1);
        assert_eq!(engine.get_success_rate(), 0.0);
    }

    #[test]
    fn test_translation_success_rate() {
        let mut engine = TranslationEngine::new();
        engine.record_call("kmalloc");
        engine.record_call("kmalloc");

        assert_eq!(engine.get_success_rate(), 100.0);

        engine.record_error("kmalloc");
        assert_eq!(engine.get_success_rate(), 50.0);
    }

    #[test]
    fn test_translation_list_mappings() {
        let engine = TranslationEngine::new();
        let mappings = engine.list_mappings();
        assert!(mappings.len() > 0);
    }

    #[test]
    fn test_linux_api_call_strings() {
        assert_eq!(LinuxApiCall::KMalloc.as_str(), "kmalloc");
        assert_eq!(LinuxApiCall::Kfree.as_str(), "kfree");
        assert_eq!(LinuxApiCall::RequestIrq.as_str(), "request_irq");
        assert_eq!(LinuxApiCall::FreeIrq.as_str(), "free_irq");
    }

    #[test]
    fn test_sher_primitive_strings() {
        assert_eq!(SherPrimitive::MasterAllocate.as_str(), "master_allocate");
        assert_eq!(
            SherPrimitive::MasterDeallocate.as_str(),
            "master_deallocate"
        );
        assert_eq!(
            SherPrimitive::InterruptRegister.as_str(),
            "interrupt_register"
        );
        assert_eq!(
            SherPrimitive::InterruptUnregister.as_str(),
            "interrupt_unregister"
        );
    }

    // ========================================================================
    // INTEGRATION TESTS
    // ========================================================================

    #[test]
    fn test_full_driver_lifecycle() {
        let mut loader = DriverLoader::new();

        // Load driver
        let container = loader
            .load_driver("test.ko", "test_driver")
            .expect("Failed to load");
        assert_eq!(container.state, ContainerState::Running);

        // Get driver reference
        let driver = loader.get_driver("test_driver");
        assert!(driver.is_some());
        assert!(driver.unwrap().is_operational());

        // Unload driver
        assert!(loader.unload_driver("test_driver").is_ok());
        assert!(loader.get_driver("test_driver").is_none());
    }

    #[test]
    fn test_driver_with_capabilities() {
        let mut container = DriverContainer::new("eth_driver");
        container.grant_capability(DriverCapability::ReadDma);
        container.grant_capability(DriverCapability::WriteDma);
        container.grant_capability(DriverCapability::InterruptHandling);

        assert!(container.has_capability(DriverCapability::ReadDma));
        assert!(container.has_capability(DriverCapability::WriteDma));
        assert!(container.has_capability(DriverCapability::InterruptHandling));
        assert!(!container.has_capability(DriverCapability::Admin));
    }

    #[test]
    fn test_linux_driver_translation() {
        let mut engine = TranslationEngine::new();
        let mut loader = DriverLoader::new();

        // Load Linux driver
        let result = loader.load_driver("e1000.ko", "e1000");
        assert!(result.is_ok());

        // Verify translation mappings exist
        assert!(engine.translate("kmalloc").is_some());
        assert!(engine.translate("kfree").is_some());
        assert!(engine.translate("request_irq").is_some());

        // Record API calls
        engine.record_call("kmalloc");
        engine.record_call("kmalloc");
        engine.record_call("request_irq");

        assert_eq!(engine.get_total_calls(), 3);
        assert_eq!(engine.get_call_count("kmalloc"), 2);
    }

    #[test]
    fn test_container_pool_with_devices() {
        let mut pool = ContainerPool::new();
        let device_id = ObjectId::new();

        let mut container = DriverContainer::new("pci_driver");
        container.device_id = Some(device_id);
        pool.register(container);

        let drivers = pool.get_by_device(device_id);
        assert_eq!(drivers.len(), 1);
        assert_eq!(drivers[0].driver_name, "pci_driver");
    }

    #[test]
    fn test_multiple_drivers_same_device() {
        let mut pool = ContainerPool::new();
        let device_id = ObjectId::new();

        // Register primary driver
        let mut c1 = DriverContainer::new("primary");
        c1.device_id = Some(device_id);
        pool.register(c1);

        // Register fallback driver
        let mut c2 = DriverContainer::new("fallback");
        c2.device_id = Some(device_id);
        pool.register(c2);

        let drivers = pool.get_by_device(device_id);
        assert_eq!(drivers.len(), 2);
    }

    #[test]
    fn test_error_recovery_and_restart() {
        let mut container = DriverContainer::new("test");
        container.start().ok();
        assert_eq!(container.telemetry.start_count, 1);

        // Simulate error
        container.record_error("Connection timeout".to_string());
        assert_eq!(container.state, ContainerState::Error);

        // Restart
        let _ = container.stop();
        let _ = container.start();
        assert_eq!(container.telemetry.start_count, 2);
        assert_eq!(container.state, ContainerState::Running);
    }

    // ========================================================================
    // SANDBOX TESTS
    // ========================================================================

    #[test]
    fn test_sandbox_policy_new() {
        let driver_id = ObjectId::new();
        let policy = SandboxPolicy::new(driver_id, SecurityLevel::Restricted);

        assert_eq!(policy.driver_id, driver_id);
        assert_eq!(policy.security_level, SecurityLevel::Restricted);
        assert!(policy.enabled);
    }

    #[test]
    fn test_sandbox_allowed_syscall() {
        let driver_id = ObjectId::new();
        let policy = SandboxPolicy::new(driver_id, SecurityLevel::Restricted);

        assert!(policy.check_syscall("read").is_ok());
        assert!(policy.check_syscall("write").is_ok());
    }

    #[test]
    fn test_sandbox_blocked_syscall() {
        let driver_id = ObjectId::new();
        let policy = SandboxPolicy::new(driver_id, SecurityLevel::Restricted);

        assert!(policy.check_syscall("ptrace").is_err());
        assert!(policy.check_syscall("kexec_load").is_err());
    }

    #[test]
    fn test_sandbox_file_access_allowed() {
        let driver_id = ObjectId::new();
        let policy = SandboxPolicy::new(driver_id, SecurityLevel::Restricted);

        assert!(policy
            .check_file_access("/sys/bus/pci/devices", false)
            .is_ok());
        assert!(policy.check_file_access("/dev/mem", false).is_ok());
    }

    #[test]
    fn test_sandbox_file_access_blocked() {
        let driver_id = ObjectId::new();
        let policy = SandboxPolicy::new(driver_id, SecurityLevel::Restricted);

        assert!(policy.check_file_access("/etc/shadow", false).is_err());
        assert!(policy.check_file_access("/root/secrets", false).is_err());
    }

    #[test]
    fn test_sandbox_read_only_paths() {
        let driver_id = ObjectId::new();
        let policy = SandboxPolicy::new(driver_id, SecurityLevel::Restricted);

        // Read should be OK (matches allowed path)
        assert!(policy
            .check_file_access("/sys/bus/pci/devices", false)
            .is_ok());
        // Write should be blocked (matches read-only path)
        assert!(policy
            .check_file_access("/sys/bus/pci/devices", true)
            .is_err());
    }

    #[test]
    fn test_sandbox_manager_register() {
        let mut manager = SandboxManager::new();
        let driver_id = ObjectId::new();
        let policy = SandboxPolicy::new(driver_id, SecurityLevel::Strict);

        manager.register_policy(policy);
        assert!(manager.get_policy(driver_id).is_some());
    }

    #[test]
    fn test_sandbox_manager_syscall_logging() {
        let mut manager = SandboxManager::new();
        let driver_id = ObjectId::new();
        let policy = SandboxPolicy::new(driver_id, SecurityLevel::Restricted);

        manager.register_policy(policy);
        let _ = manager.check_syscall(driver_id, "read");

        let log = manager.get_syscall_log(driver_id);
        assert!(log.is_some());
        assert!(log.unwrap().len() > 0);
    }

    #[test]
    fn test_capability_set_grant_deny() {
        let mut caps = CapabilitySet::new();

        caps.grant("CAP_SYS_ADMIN");
        assert!(caps.has("CAP_SYS_ADMIN"));

        caps.deny("CAP_SYS_ADMIN");
        assert!(!caps.has("CAP_SYS_ADMIN"));
    }

    #[test]
    fn test_capability_check() {
        let mut caps = CapabilitySet::new();
        caps.grant("CAP_NET_BIND_SERVICE");

        assert!(caps.check("CAP_NET_BIND_SERVICE").is_ok());
        assert!(caps.check("CAP_ADMIN").is_err());
    }

    // ========================================================================
    // NETWORK ISOLATION TESTS
    // ========================================================================

    #[test]
    fn test_network_policy_new() {
        let driver_id = ObjectId::new();
        let policy = NetworkPolicy::new(driver_id);

        assert_eq!(policy.driver_id, driver_id);
        assert!(policy.allow_network);
        assert_eq!(policy.bandwidth_limit_kbps, 10000);
    }

    #[test]
    fn test_network_policy_allow() {
        let driver_id = ObjectId::new();
        let policy = NetworkPolicy::new(driver_id);

        assert!(policy
            .check_connection(IpProtocol::Tcp, "8.8.8.8:53")
            .is_ok());
    }

    #[test]
    fn test_network_policy_deny() {
        let driver_id = ObjectId::new();
        let mut policy = NetworkPolicy::new(driver_id);
        policy.allow_network = false;

        assert!(policy
            .check_connection(IpProtocol::Tcp, "8.8.8.8:53")
            .is_err());
    }

    #[test]
    fn test_bandwidth_throttler_limit() {
        let mut throttler = BandwidthThrottler::new();
        let driver_id = ObjectId::new();

        throttler.set_limit(driver_id, 100); // 100 kbps
        assert!(throttler.record_traffic(driver_id, 50 * 1024, true).is_ok()); // 50 KB
        assert!(throttler
            .record_traffic(driver_id, 60 * 1024, true)
            .is_err()); // 60 KB (exceeds limit)
    }

    #[test]
    fn test_bandwidth_metrics() {
        let mut throttler = BandwidthThrottler::new();
        let driver_id = ObjectId::new();

        throttler.set_limit(driver_id, 10000); // 10 Mbps
        throttler.record_traffic(driver_id, 1024, true).ok(); // 1 KB send
        throttler.record_traffic(driver_id, 2048, false).ok(); // 2 KB receive

        let metrics = throttler.get_metrics(driver_id);
        assert!(metrics.is_some());
        assert_eq!(metrics.unwrap().bytes_sent, 1024);
        assert_eq!(metrics.unwrap().bytes_received, 2048);
    }

    #[test]
    fn test_network_isolation_manager_connections() {
        let mut manager = NetworkIsolationManager::new();
        let driver_id = ObjectId::new();
        let policy = NetworkPolicy::new(driver_id);

        manager.register_policy(policy);
        assert!(manager
            .add_connection(driver_id, "conn_1".to_string())
            .is_ok());
        assert_eq!(manager.get_connection_count(driver_id), 1);

        manager.remove_connection(driver_id, "conn_1");
        assert_eq!(manager.get_connection_count(driver_id), 0);
    }

    #[test]
    fn test_network_isolation_max_connections() {
        let mut manager = NetworkIsolationManager::new();
        let driver_id = ObjectId::new();
        let mut policy = NetworkPolicy::new(driver_id);
        policy.max_connections = 2;

        manager.register_policy(policy);
        assert!(manager
            .add_connection(driver_id, "conn_1".to_string())
            .is_ok());
        assert!(manager
            .add_connection(driver_id, "conn_2".to_string())
            .is_ok());
        assert!(manager
            .add_connection(driver_id, "conn_3".to_string())
            .is_err());
    }

    #[test]
    fn test_device_isolation_manager() {
        let mut manager = DeviceIsolationManager::new();
        let driver_id = ObjectId::new();
        let device_id = ObjectId::new();

        let isolation = DeviceIsolation {
            driver_id,
            device_ids: vec![device_id],
            io_ports: vec![(0x60, 0x64)],
        };

        manager.register_isolation(isolation);
        assert!(manager.can_access_device(driver_id, device_id));
        assert!(manager.can_access_io_port(driver_id, 0x61));
        assert!(!manager.can_access_io_port(driver_id, 0x80));
    }

    #[test]
    fn test_device_isolation_check() {
        let mut manager = DeviceIsolationManager::new();
        let driver_id = ObjectId::new();
        let device_id = ObjectId::new();

        let isolation = DeviceIsolation {
            driver_id,
            device_ids: vec![device_id],
            io_ports: vec![],
        };

        manager.register_isolation(isolation);
        assert!(manager.check_device_access(driver_id, device_id).is_ok());

        let other_device = ObjectId::new();
        assert!(manager
            .check_device_access(driver_id, other_device)
            .is_err());
    }

    #[test]
    fn test_device_isolation_multiple_ports() {
        let mut manager = DeviceIsolationManager::new();
        let driver_id = ObjectId::new();
        let device_id = ObjectId::new();

        let isolation = DeviceIsolation {
            driver_id,
            device_ids: vec![device_id],
            io_ports: vec![(0x60, 0x64), (0x70, 0x77)],
        };

        manager.register_isolation(isolation);
        assert!(manager.can_access_io_port(driver_id, 0x61));
        assert!(manager.can_access_io_port(driver_id, 0x75));
        assert!(!manager.can_access_io_port(driver_id, 0x80));
    }

    // ========================================================================
    // HOT-PLUG INTEGRATION TESTS
    // ========================================================================

    #[test]
    fn test_hotplug_integration_new() {
        let integration = HotPlugIntegration::new();
        assert_eq!(integration.device_driver_map.len(), 0);
        assert_eq!(integration.driver_device_map.len(), 0);
        assert_eq!(integration.event_queue.len(), 0);
        assert_eq!(integration.event_count, 0);
        assert_eq!(integration.error_count, 0);
    }

    #[test]
    fn test_hotplug_register_device_driver() {
        let mut integration = HotPlugIntegration::new();
        let device_id = ObjectId::new();
        let driver_id = ObjectId::new();

        integration.register_device_driver(device_id, driver_id);

        assert_eq!(
            integration.get_driver_for_device(device_id),
            Some(driver_id)
        );
        assert_eq!(
            integration.get_devices_for_driver(driver_id),
            Some(&vec![device_id])
        );
    }

    #[test]
    fn test_hotplug_unregister_device_driver() {
        let mut integration = HotPlugIntegration::new();
        let device_id = ObjectId::new();
        let driver_id = ObjectId::new();

        integration.register_device_driver(device_id, driver_id);
        assert_eq!(
            integration.get_driver_for_device(device_id),
            Some(driver_id)
        );

        integration.unregister_device_driver(device_id);
        assert_eq!(integration.get_driver_for_device(device_id), None);
    }

    #[test]
    fn test_hotplug_queue_event() {
        let mut integration = HotPlugIntegration::new();
        let device_id = ObjectId::new();

        let event = HotPlugEvent {
            event_type: HotPlugEventType::DeviceInserted,
            device_id,
            driver_name: Some("test_driver".to_string()),
            timestamp: 0,
            details: None,
        };

        integration.queue_event(event);

        assert_eq!(integration.pending_events(), 1);
        assert_eq!(integration.event_count, 1);
    }

    #[test]
    fn test_hotplug_next_event() {
        let mut integration = HotPlugIntegration::new();
        let device_id = ObjectId::new();

        let event = HotPlugEvent {
            event_type: HotPlugEventType::DeviceInserted,
            device_id,
            driver_name: Some("test_driver".to_string()),
            timestamp: 0,
            details: None,
        };

        integration.queue_event(event.clone());
        let retrieved = integration.next_event();

        assert!(retrieved.is_some());
        assert_eq!(
            retrieved.unwrap().event_type,
            HotPlugEventType::DeviceInserted
        );
        assert_eq!(integration.pending_events(), 0);
    }

    #[test]
    fn test_hotplug_multiple_devices() {
        let mut integration = HotPlugIntegration::new();
        let driver_id = ObjectId::new();
        let device_id1 = ObjectId::new();
        let device_id2 = ObjectId::new();
        let device_id3 = ObjectId::new();

        integration.register_device_driver(device_id1, driver_id);
        integration.register_device_driver(device_id2, driver_id);
        integration.register_device_driver(device_id3, driver_id);

        let devices = integration.get_devices_for_driver(driver_id);
        assert!(devices.is_some());
        assert_eq!(devices.unwrap().len(), 3);
    }

    #[test]
    fn test_hotplug_record_error() {
        let mut integration = HotPlugIntegration::new();
        let device_id = ObjectId::new();

        integration.record_error(
            device_id,
            "test_driver".to_string(),
            "Driver crashed".to_string(),
        );

        assert_eq!(integration.error_count, 1);
        assert_eq!(integration.pending_events(), 1);

        let event = integration.next_event();
        assert!(event.is_some());
        assert_eq!(event.unwrap().event_type, HotPlugEventType::DriverError);
    }

    #[test]
    fn test_hotplug_event_queue_fifo() {
        let mut integration = HotPlugIntegration::new();
        let device_id1 = ObjectId::new();
        let device_id2 = ObjectId::new();

        let event1 = HotPlugEvent {
            event_type: HotPlugEventType::DeviceInserted,
            device_id: device_id1,
            driver_name: Some("driver1".to_string()),
            timestamp: 0,
            details: None,
        };

        let event2 = HotPlugEvent {
            event_type: HotPlugEventType::DeviceRemoved,
            device_id: device_id2,
            driver_name: Some("driver2".to_string()),
            timestamp: 0,
            details: None,
        };

        integration.queue_event(event1);
        integration.queue_event(event2);

        let first = integration.next_event().unwrap();
        assert_eq!(first.event_type, HotPlugEventType::DeviceInserted);

        let second = integration.next_event().unwrap();
        assert_eq!(second.event_type, HotPlugEventType::DeviceRemoved);
    }

    #[test]
    fn test_driver_lifecycle_manager_new() {
        let manager = DriverLifecycleManager::new();
        assert_eq!(manager.driver_count(), 0);
        assert_eq!(manager.operational_driver_count(), 0);
        assert_eq!(manager.error_count(), 0);
        assert_eq!(manager.event_count(), 0);
    }

    #[test]
    fn test_driver_lifecycle_manager_default() {
        let manager = DriverLifecycleManager::default();
        assert_eq!(manager.driver_count(), 0);
    }

    #[test]
    fn test_driver_lifecycle_get_driver() {
        let manager = DriverLifecycleManager::new();
        let driver_id = ObjectId::new();

        let driver = manager.get_driver(driver_id);
        assert!(driver.is_none());
    }

    #[test]
    fn test_driver_lifecycle_check_driver_health() {
        let manager = DriverLifecycleManager::new();
        let driver_id = ObjectId::new();

        let result = manager.check_driver_health(driver_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_driver_lifecycle_get_active_drivers() {
        let manager = DriverLifecycleManager::new();
        let active = manager.get_active_drivers();
        assert_eq!(active.len(), 0);
    }

    #[test]
    fn test_hotplug_event_types() {
        assert_ne!(
            HotPlugEventType::DeviceInserted,
            HotPlugEventType::DeviceRemoved
        );
        assert_ne!(
            HotPlugEventType::DriverLoaded,
            HotPlugEventType::DriverUnloaded
        );
        assert_ne!(
            HotPlugEventType::DriverError,
            HotPlugEventType::DriverRecovered
        );
    }

    #[test]
    fn test_hotplug_event_clone() {
        let device_id = ObjectId::new();
        let event1 = HotPlugEvent {
            event_type: HotPlugEventType::DeviceInserted,
            device_id,
            driver_name: Some("test_driver".to_string()),
            timestamp: 0,
            details: Some("Test details".to_string()),
        };

        let event2 = event1.clone();
        assert_eq!(event1.event_type, event2.event_type);
        assert_eq!(event1.device_id, event2.device_id);
    }

    #[test]
    fn test_hotplug_integration_bidirectional_mapping() {
        let mut integration = HotPlugIntegration::new();
        let driver_id = ObjectId::new();
        let device_id1 = ObjectId::new();
        let device_id2 = ObjectId::new();

        integration.register_device_driver(device_id1, driver_id);
        integration.register_device_driver(device_id2, driver_id);

        // Device to driver mapping
        assert_eq!(
            integration.get_driver_for_device(device_id1),
            Some(driver_id)
        );
        assert_eq!(
            integration.get_driver_for_device(device_id2),
            Some(driver_id)
        );

        // Driver to devices mapping
        let devices = integration.get_devices_for_driver(driver_id);
        assert!(devices.is_some());
        assert_eq!(devices.unwrap().len(), 2);
    }

    #[test]
    fn test_hotplug_event_processing_order() {
        let mut integration = HotPlugIntegration::new();
        let mut events = Vec::new();

        for i in 0..5 {
            let event = HotPlugEvent {
                event_type: HotPlugEventType::DeviceInserted,
                device_id: ObjectId::new(),
                driver_name: Some(format!("driver_{}", i)),
                timestamp: 0,
                details: None,
            };
            events.push(event.clone());
            integration.queue_event(event);
        }

        for expected_event in events.iter() {
            let retrieved = integration.next_event().unwrap();
            assert_eq!(retrieved.driver_name, expected_event.driver_name);
        }
    }

    #[test]
    fn test_driver_lifecycle_manager_metrics() {
        let mut manager = DriverLifecycleManager::new();
        assert_eq!(manager.error_count(), 0);
        assert_eq!(manager.event_count(), 0);

        let device_id = ObjectId::new();
        manager.hotplug_integration.record_error(
            device_id,
            "test".to_string(),
            "error".to_string(),
        );

        assert_eq!(manager.error_count(), 1);
        assert_eq!(manager.event_count(), 1);
    }

    #[test]
    fn test_hotplug_multiple_drivers() {
        let mut integration = HotPlugIntegration::new();
        let driver_id1 = ObjectId::new();
        let driver_id2 = ObjectId::new();
        let device_id = ObjectId::new();

        integration.register_device_driver(device_id, driver_id1);
        assert_eq!(
            integration.get_driver_for_device(device_id),
            Some(driver_id1)
        );

        // Unregister and re-register with different driver
        integration.unregister_device_driver(device_id);
        integration.register_device_driver(device_id, driver_id2);
        assert_eq!(
            integration.get_driver_for_device(device_id),
            Some(driver_id2)
        );
    }

    #[test]
    fn test_hotplug_empty_event_queue() {
        let mut integration = HotPlugIntegration::new();
        assert_eq!(integration.next_event(), None);
        assert_eq!(integration.pending_events(), 0);
    }

    #[test]
    fn test_hotplug_event_with_details() {
        let mut integration = HotPlugIntegration::new();
        let device_id = ObjectId::new();
        let details = "Device connected via USB 3.0 at 5Gbps";

        let event = HotPlugEvent {
            event_type: HotPlugEventType::DeviceInserted,
            device_id,
            driver_name: Some("usb3_driver".to_string()),
            timestamp: 123456,
            details: Some(details.to_string()),
        };

        integration.queue_event(event);
        let retrieved = integration.next_event().unwrap();

        assert_eq!(retrieved.details, Some(details.to_string()));
        assert_eq!(retrieved.timestamp, 123456);
    }
}
