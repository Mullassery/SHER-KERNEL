// SHER Driver Runtime: Hot-Plug Integration
// Bridges Phase 2 device events with Phase 3 driver lifecycle

use sher_common::{ObjectId, Result, Error};
use crate::container::DriverContainer;
use crate::loader::DriverLoader;
use crate::sandbox::{SandboxManager, SandboxPolicy, SecurityLevel};
use crate::network::NetworkIsolationManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// HOT-PLUG EVENT TYPES
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HotPlugEventType {
    DeviceInserted,
    DeviceRemoved,
    DriverLoaded,
    DriverUnloaded,
    DriverError,
    DriverRecovered,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HotPlugEvent {
    pub event_type: HotPlugEventType,
    pub device_id: ObjectId,
    pub driver_name: Option<String>,
    pub timestamp: u64,
    pub details: Option<String>,
}

// ============================================================================
// HOT-PLUG INTEGRATION ENGINE
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct HotPlugIntegration {
    pub device_driver_map: HashMap<ObjectId, ObjectId>,  // device_id -> driver_id
    pub driver_device_map: HashMap<ObjectId, Vec<ObjectId>>,  // driver_id -> [device_ids]
    pub event_queue: Vec<HotPlugEvent>,
    pub event_count: u64,
    pub error_count: u64,
}

impl HotPlugIntegration {
    pub fn new() -> Self {
        HotPlugIntegration::default()
    }

    /// Register device-driver association
    pub fn register_device_driver(&mut self, device_id: ObjectId, driver_id: ObjectId) {
        self.device_driver_map.insert(device_id, driver_id);
        self.driver_device_map
            .entry(driver_id)
            .or_insert_with(Vec::new)
            .push(device_id);
    }

    /// Unregister device-driver association
    pub fn unregister_device_driver(&mut self, device_id: ObjectId) {
        if let Some(driver_id) = self.device_driver_map.remove(&device_id) {
            if let Some(devices) = self.driver_device_map.get_mut(&driver_id) {
                devices.retain(|&d| d != device_id);
            }
        }
    }

    /// Get driver for device
    pub fn get_driver_for_device(&self, device_id: ObjectId) -> Option<ObjectId> {
        self.device_driver_map.get(&device_id).copied()
    }

    /// Get devices for driver
    pub fn get_devices_for_driver(&self, driver_id: ObjectId) -> Option<&Vec<ObjectId>> {
        self.driver_device_map.get(&driver_id)
    }

    /// Queue hot-plug event
    pub fn queue_event(&mut self, event: HotPlugEvent) {
        self.event_queue.push(event);
        self.event_count += 1;
    }

    /// Get next event
    pub fn next_event(&mut self) -> Option<HotPlugEvent> {
        if self.event_queue.is_empty() {
            None
        } else {
            Some(self.event_queue.remove(0))
        }
    }

    /// Get pending event count
    pub fn pending_events(&self) -> usize {
        self.event_queue.len()
    }

    /// Record error event
    pub fn record_error(&mut self, device_id: ObjectId, driver_name: String, error: String) {
        self.error_count += 1;
        let event = HotPlugEvent {
            event_type: HotPlugEventType::DriverError,
            device_id,
            driver_name: Some(driver_name),
            timestamp: 0,
            details: Some(error),
        };
        self.queue_event(event);
    }
}

// ============================================================================
// DRIVER LIFECYCLE MANAGER
// ============================================================================

pub struct DriverLifecycleManager {
    pub loader: DriverLoader,
    pub sandbox_manager: SandboxManager,
    pub network_manager: NetworkIsolationManager,
    pub hotplug_integration: HotPlugIntegration,
    pub containers: HashMap<ObjectId, DriverContainer>,
}

impl DriverLifecycleManager {
    pub fn new() -> Self {
        DriverLifecycleManager {
            loader: DriverLoader::new(),
            sandbox_manager: SandboxManager::new(),
            network_manager: NetworkIsolationManager::new(),
            hotplug_integration: HotPlugIntegration::new(),
            containers: HashMap::new(),
        }
    }

    /// Handle device insertion with driver loading
    pub fn handle_device_insertion(&mut self, device_id: ObjectId, driver_name: &str) -> Result<()> {
        // Load driver
        let container = self.loader.load_driver("", driver_name)?;
        let driver_id = container.id;

        // Create sandbox policy
        let sandbox_policy = SandboxPolicy::new(driver_id, SecurityLevel::Restricted);
        self.sandbox_manager.register_policy(sandbox_policy);

        // Register in hot-plug system
        self.hotplug_integration.register_device_driver(device_id, driver_id);

        // Store container
        self.containers.insert(driver_id, container);

        // Queue event
        let event = HotPlugEvent {
            event_type: HotPlugEventType::DriverLoaded,
            device_id,
            driver_name: Some(driver_name.to_string()),
            timestamp: 0,
            details: None,
        };
        self.hotplug_integration.queue_event(event);

        Ok(())
    }

    /// Handle device removal with driver unloading
    pub fn handle_device_removal(&mut self, device_id: ObjectId) -> Result<()> {
        // Get driver for device
        if let Some(driver_id) = self.hotplug_integration.get_driver_for_device(device_id) {
            // Get driver name
            let driver_name = if let Some(container) = self.containers.get(&driver_id) {
                container.driver_name.clone()
            } else {
                "unknown".to_string()
            };

            // Stop container
            if let Some(container) = self.containers.get_mut(&driver_id) {
                container.stop()?;
            }

            // Unload driver
            self.loader.unload_driver(&driver_name)?;

            // Clean up sandbox
            // Remove from containers
            self.containers.remove(&driver_id);

            // Unregister device-driver
            self.hotplug_integration.unregister_device_driver(device_id);

            // Queue event
            let event = HotPlugEvent {
                event_type: HotPlugEventType::DriverUnloaded,
                device_id,
                driver_name: Some(driver_name),
                timestamp: 0,
                details: None,
            };
            self.hotplug_integration.queue_event(event);

            Ok(())
        } else {
            Err(Error::AllocationFailed(format!("No driver for device {}", device_id)))
        }
    }

    /// Process all pending hot-plug events
    pub fn process_events(&mut self) -> Result<usize> {
        let count = self.hotplug_integration.pending_events();

        for _ in 0..count {
            if let Some(event) = self.hotplug_integration.next_event() {
                match event.event_type {
                    HotPlugEventType::DeviceInserted => {
                        if let Some(driver_name) = &event.driver_name {
                            let _ = self.handle_device_insertion(event.device_id, driver_name);
                        }
                    }
                    HotPlugEventType::DeviceRemoved => {
                        let _ = self.handle_device_removal(event.device_id);
                    }
                    HotPlugEventType::DriverError => {
                        if let Some(container) = self.containers.get_mut(&event.device_id) {
                            if let Some(error) = &event.details {
                                container.record_error(error.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(count)
    }

    /// Get active drivers
    pub fn get_active_drivers(&self) -> Vec<&DriverContainer> {
        self.containers
            .values()
            .filter(|c| c.is_operational())
            .collect()
    }

    /// Get driver by ID
    pub fn get_driver(&self, driver_id: ObjectId) -> Option<&DriverContainer> {
        self.containers.get(&driver_id)
    }

    /// Check driver health
    pub fn check_driver_health(&self, driver_id: ObjectId) -> Result<bool> {
        if let Some(container) = self.containers.get(&driver_id) {
            Ok(container.is_operational())
        } else {
            Err(Error::AllocationFailed(format!("Driver {} not found", driver_id)))
        }
    }

    /// Get driver count
    pub fn driver_count(&self) -> usize {
        self.containers.len()
    }

    /// Get operational driver count
    pub fn operational_driver_count(&self) -> usize {
        self.get_active_drivers().len()
    }

    /// Get error count
    pub fn error_count(&self) -> u64 {
        self.hotplug_integration.error_count
    }

    /// Get event count
    pub fn event_count(&self) -> u64 {
        self.hotplug_integration.event_count
    }
}

impl Default for DriverLifecycleManager {
    fn default() -> Self {
        Self::new()
    }
}
