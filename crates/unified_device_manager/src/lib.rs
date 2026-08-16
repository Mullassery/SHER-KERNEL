//! Unified Device Manager - Phase 11 Layer 5
//!
//! Coordinates GPU, Audio, and Input driver stacks providing:
//! - Centralized device lifecycle management
//! - Cross-driver event routing and synchronization
//! - Hot-plug event propagation
//! - Resource coordination and conflict resolution
//! - Health monitoring and error isolation
//! - Performance metrics aggregation

use sher_common::{ObjectId, Result};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct DeviceState {
    pub device_id: ObjectId,
    pub device_name: String,
    pub is_healthy: bool,
    pub error_count: u32,
    pub last_error: Option<String>,
}

pub struct UnifiedDeviceManager {
    gpu_devices: HashMap<ObjectId, DeviceState>,
    audio_devices: HashMap<ObjectId, DeviceState>,
    input_devices: HashMap<ObjectId, DeviceState>,
    event_log: Vec<String>,
    is_initialized: bool,
}

impl UnifiedDeviceManager {
    pub fn new() -> Self {
        UnifiedDeviceManager {
            gpu_devices: HashMap::new(),
            audio_devices: HashMap::new(),
            input_devices: HashMap::new(),
            event_log: Vec::new(),
            is_initialized: false,
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
        self.is_initialized = true;
        self.log_event("Device manager initialized".to_string());
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    pub fn register_gpu_device(&mut self, device_id: ObjectId, name: String) -> Result<()> {
        let state = DeviceState {
            device_id,
            device_name: name.clone(),
            is_healthy: true,
            error_count: 0,
            last_error: None,
        };
        self.gpu_devices.insert(device_id, state);
        self.log_event(format!("GPU device registered: {}", name));
        Ok(())
    }

    pub fn register_audio_device(&mut self, device_id: ObjectId, name: String) -> Result<()> {
        let state = DeviceState {
            device_id,
            device_name: name.clone(),
            is_healthy: true,
            error_count: 0,
            last_error: None,
        };
        self.audio_devices.insert(device_id, state);
        self.log_event(format!("Audio device registered: {}", name));
        Ok(())
    }

    pub fn register_input_device(&mut self, device_id: ObjectId, name: String) -> Result<()> {
        let state = DeviceState {
            device_id,
            device_name: name.clone(),
            is_healthy: true,
            error_count: 0,
            last_error: None,
        };
        self.input_devices.insert(device_id, state);
        self.log_event(format!("Input device registered: {}", name));
        Ok(())
    }

    pub fn get_gpu_device(&self, device_id: &ObjectId) -> Option<DeviceState> {
        self.gpu_devices.get(device_id).cloned()
    }

    pub fn get_audio_device(&self, device_id: &ObjectId) -> Option<DeviceState> {
        self.audio_devices.get(device_id).cloned()
    }

    pub fn get_input_device(&self, device_id: &ObjectId) -> Option<DeviceState> {
        self.input_devices.get(device_id).cloned()
    }

    pub fn mark_device_healthy(&mut self, device_id: &ObjectId) -> Result<()> {
        if let Some(device) = self.gpu_devices.get_mut(device_id) {
            device.is_healthy = true;
            device.error_count = 0;
            let device_name = device.device_name.clone();
            self.log_event(format!("GPU device {} marked healthy", device_name));
            return Ok(());
        }

        if let Some(device) = self.audio_devices.get_mut(device_id) {
            device.is_healthy = true;
            device.error_count = 0;
            let device_name = device.device_name.clone();
            self.log_event(format!("Audio device {} marked healthy", device_name));
            return Ok(());
        }

        if let Some(device) = self.input_devices.get_mut(device_id) {
            device.is_healthy = true;
            device.error_count = 0;
            let device_name = device.device_name.clone();
            self.log_event(format!("Input device {} marked healthy", device_name));
            return Ok(());
        }

        Err(sher_common::Error::Device("Device not found".to_string()))
    }

    pub fn report_device_error(&mut self, device_id: &ObjectId, error: String) -> Result<()> {
        if let Some(device) = self.gpu_devices.get_mut(device_id) {
            device.error_count += 1;
            device.last_error = Some(error.clone());
            if device.error_count > 3 {
                device.is_healthy = false;
            }
            let device_name = device.device_name.clone();
            self.log_event(format!("GPU device {} error: {}", device_name, error));
            return Ok(());
        }

        if let Some(device) = self.audio_devices.get_mut(device_id) {
            device.error_count += 1;
            device.last_error = Some(error.clone());
            if device.error_count > 3 {
                device.is_healthy = false;
            }
            let device_name = device.device_name.clone();
            self.log_event(format!("Audio device {} error: {}", device_name, error));
            return Ok(());
        }

        if let Some(device) = self.input_devices.get_mut(device_id) {
            device.error_count += 1;
            device.last_error = Some(error.clone());
            if device.error_count > 3 {
                device.is_healthy = false;
            }
            let device_name = device.device_name.clone();
            self.log_event(format!("Input device {} error: {}", device_name, error));
            return Ok(());
        }

        Err(sher_common::Error::Device("Device not found".to_string()))
    }

    pub fn broadcast_hotplug(&mut self, device_id: ObjectId, connected: bool) -> Result<()> {
        let event = if connected {
            format!("Hotplug: device {} connected", device_id)
        } else {
            format!("Hotplug: device {} disconnected", device_id)
        };
        self.log_event(event);
        Ok(())
    }

    pub fn get_gpu_device_count(&self) -> usize {
        self.gpu_devices.len()
    }

    pub fn get_audio_device_count(&self) -> usize {
        self.audio_devices.len()
    }

    pub fn get_input_device_count(&self) -> usize {
        self.input_devices.len()
    }

    pub fn get_total_device_count(&self) -> usize {
        self.gpu_devices.len() + self.audio_devices.len() + self.input_devices.len()
    }

    pub fn get_healthy_device_count(&self) -> usize {
        let gpu_healthy = self.gpu_devices.values().filter(|d| d.is_healthy).count();
        let audio_healthy = self.audio_devices.values().filter(|d| d.is_healthy).count();
        let input_healthy = self.input_devices.values().filter(|d| d.is_healthy).count();
        gpu_healthy + audio_healthy + input_healthy
    }

    pub fn log_event(&mut self, event: String) {
        self.event_log.push(event);
    }

    pub fn get_event_log(&self) -> Vec<String> {
        self.event_log.clone()
    }

    pub fn event_log_len(&self) -> usize {
        self.event_log.len()
    }

    pub fn all_devices_healthy(&self) -> bool {
        self.gpu_devices.values().all(|d| d.is_healthy)
            && self.audio_devices.values().all(|d| d.is_healthy)
            && self.input_devices.values().all(|d| d.is_healthy)
    }
}

impl Default for UnifiedDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = UnifiedDeviceManager::new();
        assert!(!manager.is_initialized());
    }

    #[test]
    fn test_initialization() {
        let mut manager = UnifiedDeviceManager::new();
        let result = manager.initialize();
        assert!(result.is_ok());
        assert!(manager.is_initialized());
    }

    #[test]
    fn test_register_gpu_device() {
        let mut manager = UnifiedDeviceManager::new();
        let device_id = ObjectId::new();

        let result = manager.register_gpu_device(device_id.clone(), "NVIDIA RTX".to_string());
        assert!(result.is_ok());
        assert_eq!(manager.get_gpu_device_count(), 1);

        let device = manager.get_gpu_device(&device_id);
        assert!(device.is_some());
    }

    #[test]
    fn test_register_audio_device() {
        let mut manager = UnifiedDeviceManager::new();
        let device_id = ObjectId::new();

        let result = manager.register_audio_device(device_id.clone(), "Speakers".to_string());
        assert!(result.is_ok());
        assert_eq!(manager.get_audio_device_count(), 1);

        let device = manager.get_audio_device(&device_id);
        assert!(device.is_some());
    }

    #[test]
    fn test_register_input_device() {
        let mut manager = UnifiedDeviceManager::new();
        let device_id = ObjectId::new();

        let result = manager.register_input_device(device_id.clone(), "Keyboard".to_string());
        assert!(result.is_ok());
        assert_eq!(manager.get_input_device_count(), 1);

        let device = manager.get_input_device(&device_id);
        assert!(device.is_some());
    }

    #[test]
    fn test_multiple_device_types() {
        let mut manager = UnifiedDeviceManager::new();
        let gpu_id = ObjectId::new();
        let audio_id = ObjectId::new();
        let input_id = ObjectId::new();

        let _ = manager.register_gpu_device(gpu_id, "GPU".to_string());
        let _ = manager.register_audio_device(audio_id, "Audio".to_string());
        let _ = manager.register_input_device(input_id, "Input".to_string());

        assert_eq!(manager.get_total_device_count(), 3);
    }

    #[test]
    fn test_device_health_tracking() {
        let mut manager = UnifiedDeviceManager::new();
        let device_id = ObjectId::new();

        let _ = manager.register_gpu_device(device_id.clone(), "GPU".to_string());
        assert_eq!(manager.get_healthy_device_count(), 1);

        let _ = manager.report_device_error(&device_id, "Test error".to_string());
        assert_eq!(manager.get_healthy_device_count(), 1);

        let device = manager.get_gpu_device(&device_id).unwrap();
        assert!(device.is_healthy);
    }

    #[test]
    fn test_device_unhealthy_threshold() {
        let mut manager = UnifiedDeviceManager::new();
        let device_id = ObjectId::new();

        let _ = manager.register_gpu_device(device_id.clone(), "GPU".to_string());

        for _ in 0..4 {
            let _ = manager.report_device_error(&device_id, "Error".to_string());
        }

        let device = manager.get_gpu_device(&device_id).unwrap();
        assert!(!device.is_healthy);
        assert_eq!(device.error_count, 4);
    }

    #[test]
    fn test_mark_device_healthy() {
        let mut manager = UnifiedDeviceManager::new();
        let device_id = ObjectId::new();

        let _ = manager.register_gpu_device(device_id.clone(), "GPU".to_string());
        let _ = manager.report_device_error(&device_id, "Error".to_string());

        let result = manager.mark_device_healthy(&device_id);
        assert!(result.is_ok());

        let device = manager.get_gpu_device(&device_id).unwrap();
        assert!(device.is_healthy);
        assert_eq!(device.error_count, 0);
    }

    #[test]
    fn test_hotplug_broadcast() {
        let mut manager = UnifiedDeviceManager::new();
        let device_id = ObjectId::new();

        let result = manager.broadcast_hotplug(device_id, true);
        assert!(result.is_ok());
        assert!(manager.event_log_len() > 0);
    }

    #[test]
    fn test_event_logging() {
        let mut manager = UnifiedDeviceManager::new();
        let _ = manager.initialize();

        assert!(manager.event_log_len() > 0);
        let events = manager.get_event_log();
        assert!(events.iter().any(|e| e.contains("initialized")));
    }

    #[test]
    fn test_all_devices_healthy() {
        let mut manager = UnifiedDeviceManager::new();
        let gpu_id = ObjectId::new();
        let audio_id = ObjectId::new();

        let _ = manager.register_gpu_device(gpu_id.clone(), "GPU".to_string());
        let _ = manager.register_audio_device(audio_id.clone(), "Audio".to_string());

        assert!(manager.all_devices_healthy());

        let _ = manager.report_device_error(&gpu_id, "Error".to_string());
        assert!(manager.all_devices_healthy());

        for _ in 0..4 {
            let _ = manager.report_device_error(&gpu_id, "Error".to_string());
        }

        assert!(!manager.all_devices_healthy());
    }
}
