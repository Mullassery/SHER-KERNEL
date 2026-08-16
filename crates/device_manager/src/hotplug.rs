// SHER Device Manager: Hot-Plug Support
// Handles dynamic device insertion, removal, and recovery

use crate::{DeviceRegistry, DeviceState, RegisteredDevice};
use sher_common::{Error, ObjectId, Result};
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;

// ============================================================================
// DEVICE EVENTS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceEventType {
    Inserted,
    Removed,
    Connected,
    Disconnected,
    Error,
    Recovered,
    StateChanged,
}

#[derive(Debug, Clone)]
pub struct DeviceEvent {
    pub event_type: DeviceEventType,
    pub device_id: ObjectId,
    pub device_name: String,
    pub timestamp: u64,
    pub details: Option<String>,
}

impl DeviceEvent {
    pub fn new(event_type: DeviceEventType, device_id: ObjectId, device_name: String) -> Self {
        DeviceEvent {
            event_type,
            device_id,
            device_name,
            timestamp: 0, // Would be set to current time in production
            details: None,
        }
    }

    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }
}

// ============================================================================
// EVENT CALLBACKS
// ============================================================================

pub type EventCallback = Arc<dyn Fn(&DeviceEvent) + Send + Sync>;

#[derive(Clone)]
pub struct EventSubscription {
    pub event_type: DeviceEventType,
    pub callback: EventCallback,
}

// ============================================================================
// HOT-PLUG MANAGER
// ============================================================================

pub struct HotPlugManager {
    pub enabled: bool,
    pub max_reconnect_attempts: u32,
    pub reconnect_backoff_ms: u32,
    pub event_queue: VecDeque<DeviceEvent>,
    pub subscriptions: Vec<EventSubscription>,
    pub removed_devices: Vec<ObjectId>,
    pub pending_devices: Vec<ObjectId>,
}

impl HotPlugManager {
    pub fn new() -> Self {
        HotPlugManager {
            enabled: true,
            max_reconnect_attempts: 3,
            reconnect_backoff_ms: 100,
            event_queue: VecDeque::new(),
            subscriptions: Vec::new(),
            removed_devices: Vec::new(),
            pending_devices: Vec::new(),
        }
    }

    /// Subscribe to device events
    pub fn subscribe(&mut self, event_type: DeviceEventType, callback: EventCallback) {
        self.subscriptions.push(EventSubscription {
            event_type,
            callback,
        });
    }

    /// Handle device insertion
    pub fn handle_device_insertion(&mut self, device: RegisteredDevice) -> Result<()> {
        if !self.enabled {
            return Err(Error::AllocationFailed("Hot-plug disabled".to_string()));
        }

        let event = DeviceEvent::new(DeviceEventType::Inserted, device.id, device.name.clone());

        self.event_queue.push_back(event.clone());
        self.emit_event(&event);

        // Mark as pending until driver loads
        self.pending_devices.push(device.id);

        Ok(())
    }

    /// Handle device removal
    pub fn handle_device_removal(
        &mut self,
        device_id: ObjectId,
        device_name: String,
    ) -> Result<()> {
        if !self.enabled {
            return Err(Error::AllocationFailed("Hot-plug disabled".to_string()));
        }

        let event = DeviceEvent::new(DeviceEventType::Removed, device_id, device_name);

        self.event_queue.push_back(event.clone());
        self.emit_event(&event);
        self.removed_devices.push(device_id);

        Ok(())
    }

    /// Gracefully shutdown device
    pub fn graceful_shutdown(
        &mut self,
        registry: &mut DeviceRegistry,
        device_id: ObjectId,
    ) -> Result<()> {
        // Get device name before mutation
        let device_name = if let Some(device) = registry.get_device(device_id) {
            device.name.clone()
        } else {
            "unknown".to_string()
        };

        // Transition to removing state
        registry
            .update_device_state(device_id, DeviceState::Removing)
            .map_err(Error::AllocationFailed)?;

        // Drain I/O operations (simulated - in production would wait for actual I/O)
        // ... wait for I/O ...

        // Transition to removed state
        registry
            .update_device_state(device_id, DeviceState::Removed)
            .map_err(Error::AllocationFailed)?;

        let event = DeviceEvent::new(DeviceEventType::Disconnected, device_id, device_name);

        self.event_queue.push_back(event.clone());
        self.emit_event(&event);

        Ok(())
    }

    /// Re-enumerate devices (after insertion/removal)
    pub fn reenumerate(&mut self, registry: &mut DeviceRegistry) -> Result<usize> {
        // Clear pending devices that successfully loaded
        self.pending_devices.clear();

        // Clear removed devices (optional cleanup)
        // In production, might keep for a grace period
        self.removed_devices.clear();

        // Count operational devices
        let reenumerated_count = registry.get_operational_count();

        Ok(reenumerated_count)
    }

    /// Check and handle recovery of failed devices
    pub fn check_recovery(
        &mut self,
        registry: &mut DeviceRegistry,
        device_id: ObjectId,
    ) -> Result<bool> {
        // Check current state and error count
        let (is_error, error_count, device_name) =
            if let Some(device) = registry.get_device(device_id) {
                (
                    device.state == DeviceState::Error,
                    device.telemetry.total_errors,
                    device.name.clone(),
                )
            } else {
                return Err(Error::AllocationFailed(format!(
                    "Device {} not found",
                    device_id
                )));
            };

        if is_error {
            // Attempt recovery
            let mut recovered = false;

            if error_count < self.max_reconnect_attempts as u64 {
                // Try to recover
                if registry
                    .update_device_state(device_id, DeviceState::Initialized)
                    .is_ok()
                {
                    let event =
                        DeviceEvent::new(DeviceEventType::Recovered, device_id, device_name);
                    self.event_queue.push_back(event.clone());
                    self.emit_event(&event);
                    recovered = true;
                }
            }

            Ok(recovered)
        } else {
            Ok(false)
        }
    }

    /// Get next pending event
    pub fn next_event(&mut self) -> Option<DeviceEvent> {
        self.event_queue.pop_front()
    }

    /// Get event count
    pub fn pending_event_count(&self) -> usize {
        self.event_queue.len()
    }

    /// Process all pending events
    pub fn process_all_events(&mut self) -> usize {
        let count = self.event_queue.len();
        while let Some(event) = self.next_event() {
            self.emit_event(&event);
        }
        count
    }

    fn emit_event(&self, event: &DeviceEvent) {
        // Emit to all matching subscriptions
        for subscription in &self.subscriptions {
            if subscription.event_type == event.event_type
                || subscription.event_type == DeviceEventType::StateChanged
            {
                (subscription.callback)(event);
            }
        }
    }

    pub fn get_removed_device_count(&self) -> usize {
        self.removed_devices.len()
    }

    pub fn get_pending_device_count(&self) -> usize {
        self.pending_devices.len()
    }
}

impl Default for HotPlugManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// DEVICE RECOVERY MANAGER
// ============================================================================

pub struct RecoveryPolicy {
    pub max_attempts: u32,
    pub initial_backoff_ms: u32,
    pub max_backoff_ms: u32,
    pub backoff_multiplier: f64,
}

impl Default for RecoveryPolicy {
    fn default() -> Self {
        RecoveryPolicy {
            max_attempts: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 5000,
            backoff_multiplier: 2.0,
        }
    }
}

pub struct RecoveryManager {
    pub policy: RecoveryPolicy,
    pub recovery_attempts: std::collections::HashMap<ObjectId, u32>,
    pub last_recovery_time: std::collections::HashMap<ObjectId, u64>,
}

impl RecoveryManager {
    pub fn new(policy: RecoveryPolicy) -> Self {
        RecoveryManager {
            policy,
            recovery_attempts: std::collections::HashMap::new(),
            last_recovery_time: std::collections::HashMap::new(),
        }
    }

    pub fn can_recover(&self, device_id: ObjectId) -> bool {
        let attempts = self.recovery_attempts.get(&device_id).copied().unwrap_or(0);
        attempts < self.policy.max_attempts
    }

    pub fn record_recovery_attempt(&mut self, device_id: ObjectId) {
        let attempts = self.recovery_attempts.entry(device_id).or_insert(0);
        *attempts += 1;
        self.last_recovery_time.insert(device_id, 0); // Would be current time
    }

    pub fn reset_recovery(&mut self, device_id: ObjectId) {
        self.recovery_attempts.remove(&device_id);
        self.last_recovery_time.remove(&device_id);
    }

    pub fn get_backoff_ms(&self, device_id: ObjectId) -> u32 {
        let attempts = self.recovery_attempts.get(&device_id).copied().unwrap_or(0);
        let backoff = (self.policy.initial_backoff_ms as f64
            * self.policy.backoff_multiplier.powi(attempts as i32)) as u32;
        backoff.min(self.policy.max_backoff_ms)
    }
}

impl Default for RecoveryManager {
    fn default() -> Self {
        Self::new(RecoveryPolicy::default())
    }
}

// ============================================================================
// HOTPLUG CONTROLLER
// ============================================================================

pub struct HotPlugController {
    pub hotplug_manager: Arc<StdMutex<HotPlugManager>>,
    pub recovery_manager: Arc<StdMutex<RecoveryManager>>,
}

impl HotPlugController {
    pub fn new() -> Self {
        HotPlugController {
            hotplug_manager: Arc::new(StdMutex::new(HotPlugManager::new())),
            recovery_manager: Arc::new(StdMutex::new(RecoveryManager::new(
                RecoveryPolicy::default(),
            ))),
        }
    }

    pub fn enable_hotplug(&self) -> Result<()> {
        let mut manager = self
            .hotplug_manager
            .lock()
            .map_err(|_| Error::AllocationFailed("Cannot lock hot-plug manager".to_string()))?;
        manager.enabled = true;
        Ok(())
    }

    pub fn disable_hotplug(&self) -> Result<()> {
        let mut manager = self
            .hotplug_manager
            .lock()
            .map_err(|_| Error::AllocationFailed("Cannot lock hot-plug manager".to_string()))?;
        manager.enabled = false;
        Ok(())
    }

    pub fn insert_device(&self, device: RegisteredDevice) -> Result<()> {
        let mut manager = self
            .hotplug_manager
            .lock()
            .map_err(|_| Error::AllocationFailed("Cannot lock hot-plug manager".to_string()))?;
        manager.handle_device_insertion(device)
    }

    pub fn remove_device(&self, device_id: ObjectId, device_name: String) -> Result<()> {
        let mut manager = self
            .hotplug_manager
            .lock()
            .map_err(|_| Error::AllocationFailed("Cannot lock hot-plug manager".to_string()))?;
        manager.handle_device_removal(device_id, device_name)
    }
}

impl Default for HotPlugController {
    fn default() -> Self {
        Self::new()
    }
}
