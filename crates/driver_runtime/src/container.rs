// SHER Driver Runtime: Container Isolation
// Each driver executes in isolated container with restricted capabilities

use sher_common::{ObjectId, Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// DRIVER CONTAINER STATE
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerState {
    Created,
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
    Error,
    Crashed,
}

impl ContainerState {
    pub fn is_operational(&self) -> bool {
        matches!(self, ContainerState::Running | ContainerState::Paused)
    }

    pub fn can_transition_to(&self, target: ContainerState) -> bool {
        match (self, target) {
            (ContainerState::Created, ContainerState::Starting) => true,
            (ContainerState::Starting, ContainerState::Running) => true,
            (ContainerState::Running, ContainerState::Paused) => true,
            (ContainerState::Paused, ContainerState::Running) => true,
            (ContainerState::Running, ContainerState::Stopping) => true,
            (ContainerState::Stopping, ContainerState::Stopped) => true,
            (ContainerState::Stopped, ContainerState::Starting) => true,  // Allow restart
            (ContainerState::Error, ContainerState::Starting) => true,
            (ContainerState::Error, ContainerState::Stopped) => true,
            (ContainerState::Crashed, ContainerState::Starting) => true,
            (ContainerState::Crashed, ContainerState::Stopped) => true,
            (_, ContainerState::Error) => true,
            (_, ContainerState::Crashed) => true,
            _ => false,
        }
    }
}

// ============================================================================
// RESOURCE LIMITS
// ============================================================================

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub memory_limit_bytes: u64,
    pub cpu_quota_percent: u32,
    pub max_file_descriptors: u32,
    pub max_threads: u32,
    pub network_bandwidth_kbps: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        ResourceLimits {
            memory_limit_bytes: 256 * 1024 * 1024,  // 256MB default
            cpu_quota_percent: 50,                   // 50% CPU
            max_file_descriptors: 256,
            max_threads: 8,
            network_bandwidth_kbps: 10000,           // 10Mbps
        }
    }
}

// ============================================================================
// CAPABILITY RESTRICTIONS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DriverCapability {
    ReadMemory,
    WriteMemory,
    ReadDma,
    WriteDma,
    InterruptHandling,
    TimerAccess,
    NetworkAccess,
    StorageAccess,
    GpuAccess,
    Admin,
}

// ============================================================================
// CONTAINER TELEMETRY
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContainerTelemetry {
    pub start_count: u64,
    pub crash_count: u64,
    pub total_runtime_ms: u64,
    pub memory_peak_bytes: u64,
    pub cpu_time_ms: u64,
    pub last_error: Option<String>,
}

// ============================================================================
// DRIVER CONTAINER
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverContainer {
    pub id: ObjectId,
    pub driver_name: String,
    pub device_id: Option<ObjectId>,
    pub state: ContainerState,
    pub resource_limits: ResourceLimits,
    pub capabilities: Vec<DriverCapability>,
    pub telemetry: ContainerTelemetry,
    pub created_at: u64,
    pub started_at: Option<u64>,
    pub environment: HashMap<String, String>,
}

impl DriverContainer {
    /// Create new driver container
    pub fn new(driver_name: impl Into<String>) -> Self {
        DriverContainer {
            id: ObjectId::new(),
            driver_name: driver_name.into(),
            device_id: None,
            state: ContainerState::Created,
            resource_limits: ResourceLimits::default(),
            capabilities: Vec::new(),
            telemetry: ContainerTelemetry::default(),
            created_at: 0,
            started_at: None,
            environment: HashMap::new(),
        }
    }

    /// Set resource limits
    pub fn with_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }

    /// Grant capability
    pub fn grant_capability(&mut self, capability: DriverCapability) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
    }

    /// Revoke capability
    pub fn revoke_capability(&mut self, capability: DriverCapability) {
        self.capabilities.retain(|&c| c != capability);
    }

    /// Check if capability is granted
    pub fn has_capability(&self, capability: DriverCapability) -> bool {
        self.capabilities.contains(&capability)
    }

    /// Start container
    pub fn start(&mut self) -> Result<()> {
        if !self.state.can_transition_to(ContainerState::Starting) &&
           !self.state.can_transition_to(ContainerState::Running) {
            return Err(Error::AllocationFailed(
                format!("Cannot start container in state {:?}", self.state)
            ));
        }

        self.state = ContainerState::Starting;
        self.started_at = Some(0);  // Would be set to current time
        self.telemetry.start_count += 1;
        self.state = ContainerState::Running;

        Ok(())
    }

    /// Stop container gracefully
    pub fn stop(&mut self) -> Result<()> {
        // Allow stopping from Running or Error/Crashed states
        match self.state {
            ContainerState::Running | ContainerState::Error | ContainerState::Crashed => {},
            _ => return Err(Error::AllocationFailed(
                format!("Cannot stop container in state {:?}", self.state)
            )),
        }

        self.state = ContainerState::Stopping;
        self.state = ContainerState::Stopped;

        Ok(())
    }

    /// Pause container
    pub fn pause(&mut self) -> Result<()> {
        if self.state != ContainerState::Running {
            return Err(Error::AllocationFailed("Container must be running to pause".to_string()));
        }

        self.state = ContainerState::Paused;
        Ok(())
    }

    /// Resume container
    pub fn resume(&mut self) -> Result<()> {
        if self.state != ContainerState::Paused {
            return Err(Error::AllocationFailed("Container must be paused to resume".to_string()));
        }

        self.state = ContainerState::Running;
        Ok(())
    }

    /// Record error
    pub fn record_error(&mut self, error: String) {
        self.telemetry.last_error = Some(error);
        self.state = ContainerState::Error;
    }

    /// Record crash
    pub fn record_crash(&mut self, error: String) {
        self.telemetry.crash_count += 1;
        self.telemetry.last_error = Some(error);
        self.state = ContainerState::Crashed;
    }

    /// Set environment variable
    pub fn set_env(&mut self, key: String, value: String) {
        self.environment.insert(key, value);
    }

    /// Get environment variable
    pub fn get_env(&self, key: &str) -> Option<&String> {
        self.environment.get(key)
    }

    pub fn is_operational(&self) -> bool {
        self.state.is_operational()
    }

    pub fn get_memory_usage_percent(&self) -> u32 {
        // Simplified: in production would query actual usage
        if self.telemetry.memory_peak_bytes == 0 {
            return 0;
        }
        ((self.telemetry.memory_peak_bytes * 100) / self.resource_limits.memory_limit_bytes) as u32
    }
}

// ============================================================================
// CONTAINER POOL
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct ContainerPool {
    pub containers: HashMap<ObjectId, DriverContainer>,
    pub name_index: HashMap<String, ObjectId>,
}

impl ContainerPool {
    pub fn new() -> Self {
        ContainerPool::default()
    }

    pub fn register(&mut self, container: DriverContainer) {
        let id = container.id;
        let name = container.driver_name.clone();
        self.containers.insert(id, container);
        self.name_index.insert(name, id);
    }

    pub fn unregister(&mut self, id: ObjectId) {
        if let Some(container) = self.containers.remove(&id) {
            self.name_index.remove(&container.driver_name);
        }
    }

    pub fn get(&self, id: ObjectId) -> Option<&DriverContainer> {
        self.containers.get(&id)
    }

    pub fn get_mut(&mut self, id: ObjectId) -> Option<&mut DriverContainer> {
        self.containers.get_mut(&id)
    }

    pub fn find_by_name(&self, name: &str) -> Option<ObjectId> {
        self.name_index.get(name).copied()
    }

    pub fn get_by_name(&self, name: &str) -> Option<&DriverContainer> {
        self.find_by_name(name)
            .and_then(|id| self.get(id))
    }

    pub fn get_by_device(&self, device_id: ObjectId) -> Vec<&DriverContainer> {
        self.containers
            .values()
            .filter(|c| c.device_id == Some(device_id))
            .collect()
    }

    pub fn list_operational(&self) -> Vec<&DriverContainer> {
        self.containers
            .values()
            .filter(|c| c.is_operational())
            .collect()
    }

    pub fn count(&self) -> usize {
        self.containers.len()
    }
}
