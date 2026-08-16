use serde::{Deserialize, Serialize};
use sher_common::ObjectId;
use std::collections::HashMap;

// ============================================================================
// DEVICE STATE MACHINE
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceState {
    Discovered,  // Enumerated but not initialized
    Initialized, // Initialized, waiting for driver
    Ready,       // Driver loaded, ready for use
    Running,     // Active and operational
    Paused,      // Temporarily suspended
    Error,       // In error state
    Removing,    // Hot-removal in progress
    Removed,     // Removed from system
}

impl DeviceState {
    pub fn is_operational(&self) -> bool {
        matches!(self, DeviceState::Ready | DeviceState::Running)
    }

    pub fn can_transition_to(&self, target: DeviceState) -> bool {
        matches!(
            (self, target),
            (DeviceState::Discovered, DeviceState::Initialized)
                | (DeviceState::Initialized, DeviceState::Ready)
                | (DeviceState::Ready, DeviceState::Running)
                | (DeviceState::Running, DeviceState::Paused)
                | (DeviceState::Paused, DeviceState::Running)
                | (_, DeviceState::Error)
                | (_, DeviceState::Removing)
                | (DeviceState::Removing, DeviceState::Removed)
        )
    }
}

// ============================================================================
// DEVICE TELEMETRY
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceTelemetry {
    pub total_errors: u64,
    pub total_resets: u64,
    pub total_interrupts: u64,
    pub last_error: Option<String>,
    pub uptime_seconds: u64,
    pub discovery_time_ms: u64,
}

// ============================================================================
// DEVICE REGISTRY
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredDevice {
    pub id: ObjectId,
    pub name: String,
    pub parent_id: Option<ObjectId>,
    pub device_type: String,
    pub state: DeviceState,
    pub properties: HashMap<String, String>,
    pub driver_id: Option<ObjectId>,
    pub driver_loaded: bool,
    pub telemetry: DeviceTelemetry,
    pub created_at: u64,
    pub last_seen: u64,
}

impl RegisteredDevice {
    pub fn new(id: ObjectId, name: String, device_type: String, created_at: u64) -> Self {
        RegisteredDevice {
            id,
            name,
            parent_id: None,
            device_type,
            state: DeviceState::Discovered,
            properties: HashMap::new(),
            driver_id: None,
            driver_loaded: false,
            telemetry: DeviceTelemetry::default(),
            created_at,
            last_seen: created_at,
        }
    }

    pub fn is_operational(&self) -> bool {
        self.state.is_operational()
    }

    pub fn transition_to(&mut self, target: DeviceState) -> Result<(), String> {
        if !self.state.can_transition_to(target) {
            return Err(format!(
                "Cannot transition from {:?} to {:?}",
                self.state, target
            ));
        }
        self.state = target;
        Ok(())
    }

    pub fn record_error(&mut self, error: String) {
        self.telemetry.total_errors += 1;
        self.telemetry.last_error = Some(error);
        self.state = DeviceState::Error;
    }
}

// ============================================================================
// DEVICE HIERARCHY
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct DeviceHierarchy {
    pub parent_map: HashMap<ObjectId, ObjectId>, // child -> parent
    pub children_map: HashMap<ObjectId, Vec<ObjectId>>, // parent -> children
}

impl DeviceHierarchy {
    pub fn add_device(&mut self, device_id: ObjectId, parent_id: Option<ObjectId>) {
        if let Some(parent) = parent_id {
            self.parent_map.insert(device_id, parent);
            self.children_map.entry(parent).or_default().push(device_id);
        }
    }

    pub fn get_parent(&self, device_id: ObjectId) -> Option<ObjectId> {
        self.parent_map.get(&device_id).copied()
    }

    pub fn get_children(&self, device_id: ObjectId) -> Option<&Vec<ObjectId>> {
        self.children_map.get(&device_id)
    }

    pub fn remove_device(&mut self, device_id: ObjectId) {
        if let Some(parent) = self.parent_map.remove(&device_id) {
            if let Some(children) = self.children_map.get_mut(&parent) {
                children.retain(|&id| id != device_id);
            }
        }
        self.children_map.remove(&device_id);
    }
}

// ============================================================================
// DEVICE REGISTRY
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct DeviceRegistry {
    pub devices: HashMap<ObjectId, RegisteredDevice>,
    pub hierarchy: DeviceHierarchy,
    pub type_index: HashMap<String, Vec<ObjectId>>, // device_type -> [ids]
    pub state_index: HashMap<DeviceState, Vec<ObjectId>>, // state -> [ids]
}

impl DeviceRegistry {
    pub fn new() -> Self {
        DeviceRegistry {
            devices: HashMap::new(),
            hierarchy: DeviceHierarchy::default(),
            type_index: HashMap::new(),
            state_index: HashMap::new(),
        }
    }

    pub fn register(&mut self, device: RegisteredDevice) {
        let device_id = device.id;
        let device_type = device.device_type.clone();
        let parent_id = device.parent_id;
        let state = device.state;

        // Add to devices map
        self.devices.insert(device_id, device);

        // Update hierarchy
        self.hierarchy.add_device(device_id, parent_id);

        // Update type index
        self.type_index
            .entry(device_type)
            .or_default()
            .push(device_id);

        // Update state index
        self.state_index.entry(state).or_default().push(device_id);
    }

    pub fn unregister(&mut self, id: ObjectId) {
        if let Some(device) = self.devices.remove(&id) {
            // Update hierarchy
            self.hierarchy.remove_device(id);

            // Remove from type index
            if let Some(type_list) = self.type_index.get_mut(&device.device_type) {
                type_list.retain(|&dev_id| dev_id != id);
            }

            // Remove from state index
            if let Some(state_list) = self.state_index.get_mut(&device.state) {
                state_list.retain(|&dev_id| dev_id != id);
            }
        }
    }

    pub fn get_device(&self, id: ObjectId) -> Option<&RegisteredDevice> {
        self.devices.get(&id)
    }

    pub fn get_device_mut(&mut self, id: ObjectId) -> Option<&mut RegisteredDevice> {
        self.devices.get_mut(&id)
    }

    pub fn update_device_state(
        &mut self,
        id: ObjectId,
        new_state: DeviceState,
    ) -> Result<(), String> {
        if let Some(device) = self.devices.get_mut(&id) {
            // Remove from old state index
            if let Some(state_list) = self.state_index.get_mut(&device.state) {
                state_list.retain(|&dev_id| dev_id != id);
            }

            // Transition state
            device.transition_to(new_state)?;

            // Add to new state index
            self.state_index.entry(new_state).or_default().push(id);

            Ok(())
        } else {
            Err(format!("Device {} not found", id))
        }
    }

    pub fn list_devices(&self) -> Vec<&RegisteredDevice> {
        self.devices.values().collect()
    }

    pub fn find_by_type(&self, device_type: &str) -> Vec<&RegisteredDevice> {
        self.type_index
            .get(device_type)
            .map(|ids| ids.iter().filter_map(|id| self.devices.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn find_by_state(&self, state: DeviceState) -> Vec<&RegisteredDevice> {
        self.state_index
            .get(&state)
            .map(|ids| ids.iter().filter_map(|id| self.devices.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn find_children(&self, parent_id: ObjectId) -> Vec<&RegisteredDevice> {
        self.hierarchy
            .get_children(parent_id)
            .map(|ids| ids.iter().filter_map(|id| self.devices.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn find_by_name(&self, name: &str) -> Option<&RegisteredDevice> {
        self.devices.values().find(|d| d.name == name)
    }

    pub fn get_device_count(&self) -> usize {
        self.devices.len()
    }

    pub fn get_operational_count(&self) -> usize {
        self.devices.values().filter(|d| d.is_operational()).count()
    }
}
