//! GPU Driver - Phase 11 Layer 4a
//!
//! DRM/KMS abstraction layer providing:
//! - Display connector management
//! - Mode setting (resolution, refresh rate)
//! - Framebuffer allocation and management
//! - Page flipping and synchronization
//! - Hot-plug detection
//! - GPU memory management

use sher_common::{ObjectId, Result};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConnectorType {
    HDMI,
    DisplayPort,
    VGA,
    DVI,
    EDP,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectorStatus {
    Connected,
    Disconnected,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayMode {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
    pub clock: u32,
}

#[derive(Clone, Debug)]
pub struct Connector {
    pub id: ObjectId,
    pub connector_type: ConnectorType,
    pub status: ConnectorStatus,
    pub supported_modes: Vec<DisplayMode>,
    pub current_mode: Option<DisplayMode>,
}

#[derive(Clone, Debug)]
pub struct Framebuffer {
    pub id: ObjectId,
    pub width: u32,
    pub height: u32,
    pub pixel_format: u32,
    pub gpu_address: u64,
    pub size_bytes: usize,
}

#[derive(Clone, Debug)]
pub struct GPUMemory {
    pub total_bytes: usize,
    pub available_bytes: usize,
    pub allocated_bytes: usize,
}

pub struct GPUDriver {
    connectors: HashMap<ObjectId, Connector>,
    framebuffers: HashMap<ObjectId, Framebuffer>,
    gpu_memory: GPUMemory,
    primary_display: Option<ObjectId>,
    initialized: bool,
}

impl GPUDriver {
    pub fn new(total_vram: usize) -> Self {
        GPUDriver {
            connectors: HashMap::new(),
            framebuffers: HashMap::new(),
            gpu_memory: GPUMemory {
                total_bytes: total_vram,
                available_bytes: total_vram,
                allocated_bytes: 0,
            },
            primary_display: None,
            initialized: false,
        }
    }

    pub fn initialize(&mut self) -> Result<()> {
        self.initialized = true;
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    pub fn register_connector(&mut self, connector: Connector) -> Result<()> {
        self.connectors.insert(connector.id, connector);
        Ok(())
    }

    pub fn get_connector(&self, connector_id: &ObjectId) -> Option<Connector> {
        self.connectors.get(connector_id).cloned()
    }

    pub fn get_connectors(&self) -> Vec<Connector> {
        self.connectors.values().cloned().collect()
    }

    pub fn set_mode(&mut self, connector_id: &ObjectId, mode: DisplayMode) -> Result<()> {
        if let Some(connector) = self.connectors.get_mut(connector_id) {
            if !connector.supported_modes.contains(&mode) {
                return Err(sher_common::Error::Device("Unsupported mode".to_string()));
            }
            connector.current_mode = Some(mode);
            if self.primary_display.is_none() {
                self.primary_display = Some(*connector_id);
            }
            Ok(())
        } else {
            Err(sher_common::Error::Device(
                "Connector not found".to_string(),
            ))
        }
    }

    pub fn allocate_framebuffer(&mut self, width: u32, height: u32) -> Result<Framebuffer> {
        let size_bytes = (width as usize) * (height as usize) * 4;

        if size_bytes > self.gpu_memory.available_bytes {
            return Err(sher_common::Error::Memory("Insufficient VRAM".to_string()));
        }

        let framebuffer = Framebuffer {
            id: ObjectId::new(),
            width,
            height,
            pixel_format: 0x34325241,
            gpu_address: (self.gpu_memory.total_bytes - self.gpu_memory.available_bytes) as u64,
            size_bytes,
        };

        self.gpu_memory.available_bytes -= size_bytes;
        self.gpu_memory.allocated_bytes += size_bytes;

        self.framebuffers
            .insert(framebuffer.id, framebuffer.clone());

        Ok(framebuffer)
    }

    pub fn get_framebuffer(&self, fb_id: &ObjectId) -> Option<Framebuffer> {
        self.framebuffers.get(fb_id).cloned()
    }

    pub fn free_framebuffer(&mut self, fb_id: &ObjectId) -> Result<()> {
        if let Some(fb) = self.framebuffers.remove(fb_id) {
            self.gpu_memory.available_bytes += fb.size_bytes;
            self.gpu_memory.allocated_bytes -= fb.size_bytes;
            Ok(())
        } else {
            Err(sher_common::Error::Device(
                "Framebuffer not found".to_string(),
            ))
        }
    }

    pub fn page_flip(&self, connector_id: &ObjectId, fb_id: &ObjectId) -> Result<()> {
        if !self.connectors.contains_key(connector_id) {
            return Err(sher_common::Error::Device(
                "Connector not found".to_string(),
            ));
        }
        if !self.framebuffers.contains_key(fb_id) {
            return Err(sher_common::Error::Device(
                "Framebuffer not found".to_string(),
            ));
        }
        Ok(())
    }

    pub fn get_memory_info(&self) -> GPUMemory {
        self.gpu_memory.clone()
    }

    pub fn detect_hotplug(&mut self, connector_id: &ObjectId, connected: bool) -> Result<()> {
        if let Some(connector) = self.connectors.get_mut(connector_id) {
            connector.status = if connected {
                ConnectorStatus::Connected
            } else {
                ConnectorStatus::Disconnected
            };
            Ok(())
        } else {
            Err(sher_common::Error::Device(
                "Connector not found".to_string(),
            ))
        }
    }

    pub fn get_primary_display(&self) -> Option<ObjectId> {
        self.primary_display
    }

    pub fn connector_count(&self) -> usize {
        self.connectors.len()
    }

    pub fn framebuffer_count(&self) -> usize {
        self.framebuffers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_driver_creation() {
        let driver = GPUDriver::new(256 * 1024 * 1024);
        assert!(!driver.is_initialized());
    }

    #[test]
    fn test_gpu_driver_initialization() {
        let mut driver = GPUDriver::new(256 * 1024 * 1024);
        let result = driver.initialize();
        assert!(result.is_ok());
        assert!(driver.is_initialized());
    }

    #[test]
    fn test_register_connector() {
        let mut driver = GPUDriver::new(256 * 1024 * 1024);
        let connector = Connector {
            id: ObjectId::new(),
            connector_type: ConnectorType::HDMI,
            status: ConnectorStatus::Connected,
            supported_modes: vec![DisplayMode {
                width: 1920,
                height: 1080,
                refresh_rate: 60,
                clock: 148500,
            }],
            current_mode: None,
        };

        let result = driver.register_connector(connector.clone());
        assert!(result.is_ok());

        let retrieved = driver.get_connector(&connector.id);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_get_connectors() {
        let mut driver = GPUDriver::new(256 * 1024 * 1024);
        let connector1 = Connector {
            id: ObjectId::new(),
            connector_type: ConnectorType::HDMI,
            status: ConnectorStatus::Connected,
            supported_modes: vec![],
            current_mode: None,
        };
        let connector2 = Connector {
            id: ObjectId::new(),
            connector_type: ConnectorType::DisplayPort,
            status: ConnectorStatus::Disconnected,
            supported_modes: vec![],
            current_mode: None,
        };

        let _ = driver.register_connector(connector1);
        let _ = driver.register_connector(connector2);

        let connectors = driver.get_connectors();
        assert_eq!(connectors.len(), 2);
    }

    #[test]
    fn test_set_mode() {
        let mut driver = GPUDriver::new(256 * 1024 * 1024);
        let mode = DisplayMode {
            width: 1920,
            height: 1080,
            refresh_rate: 60,
            clock: 148500,
        };

        let connector = Connector {
            id: ObjectId::new(),
            connector_type: ConnectorType::HDMI,
            status: ConnectorStatus::Connected,
            supported_modes: vec![mode.clone()],
            current_mode: None,
        };

        let connector_id = connector.id;
        let _ = driver.register_connector(connector);

        let result = driver.set_mode(&connector_id, mode);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unsupported_mode() {
        let mut driver = GPUDriver::new(256 * 1024 * 1024);
        let connector = Connector {
            id: ObjectId::new(),
            connector_type: ConnectorType::HDMI,
            status: ConnectorStatus::Connected,
            supported_modes: vec![DisplayMode {
                width: 1920,
                height: 1080,
                refresh_rate: 60,
                clock: 148500,
            }],
            current_mode: None,
        };

        let connector_id = connector.id;
        let _ = driver.register_connector(connector);

        let unsupported_mode = DisplayMode {
            width: 7680,
            height: 4320,
            refresh_rate: 120,
            clock: 2000000,
        };

        let result = driver.set_mode(&connector_id, unsupported_mode);
        assert!(result.is_err());
    }

    #[test]
    fn test_allocate_framebuffer() {
        let mut driver = GPUDriver::new(256 * 1024 * 1024);
        let result = driver.allocate_framebuffer(1920, 1080);
        assert!(result.is_ok());

        let fb = result.unwrap();
        assert_eq!(fb.width, 1920);
        assert_eq!(fb.height, 1080);
    }

    #[test]
    fn test_allocate_multiple_framebuffers() {
        let mut driver = GPUDriver::new(256 * 1024 * 1024);
        let fb1 = driver.allocate_framebuffer(1920, 1080).unwrap();
        let fb2 = driver.allocate_framebuffer(1280, 720).unwrap();

        assert_eq!(driver.framebuffer_count(), 2);
        assert_ne!(fb1.id, fb2.id);
    }

    #[test]
    fn test_out_of_vram() {
        let mut driver = GPUDriver::new(1024);
        let result = driver.allocate_framebuffer(1920, 1080);
        assert!(result.is_err());
    }

    #[test]
    fn test_free_framebuffer() {
        let mut driver = GPUDriver::new(256 * 1024 * 1024);
        let fb = driver.allocate_framebuffer(1920, 1080).unwrap();

        let result = driver.free_framebuffer(&fb.id);
        assert!(result.is_ok());
        assert_eq!(driver.framebuffer_count(), 0);
    }

    #[test]
    fn test_vram_tracking() {
        let mut driver = GPUDriver::new(256 * 1024 * 1024);
        let initial_available = driver.get_memory_info().available_bytes;

        let _fb = driver.allocate_framebuffer(1920, 1080).unwrap();
        let after_alloc = driver.get_memory_info().available_bytes;

        assert!(after_alloc < initial_available);
    }

    #[test]
    fn test_page_flip() {
        let mut driver = GPUDriver::new(256 * 1024 * 1024);
        let connector = Connector {
            id: ObjectId::new(),
            connector_type: ConnectorType::HDMI,
            status: ConnectorStatus::Connected,
            supported_modes: vec![],
            current_mode: None,
        };

        let connector_id = connector.id;
        let _ = driver.register_connector(connector);

        let fb = driver.allocate_framebuffer(1920, 1080).unwrap();

        let result = driver.page_flip(&connector_id, &fb.id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_hotplug_detection() {
        let mut driver = GPUDriver::new(256 * 1024 * 1024);
        let connector = Connector {
            id: ObjectId::new(),
            connector_type: ConnectorType::HDMI,
            status: ConnectorStatus::Connected,
            supported_modes: vec![],
            current_mode: None,
        };

        let connector_id = connector.id;
        let _ = driver.register_connector(connector);

        let result = driver.detect_hotplug(&connector_id, false);
        assert!(result.is_ok());

        let updated = driver.get_connector(&connector_id).unwrap();
        assert_eq!(updated.status, ConnectorStatus::Disconnected);
    }

    #[test]
    fn test_primary_display_selection() {
        let mut driver = GPUDriver::new(256 * 1024 * 1024);
        let mode = DisplayMode {
            width: 1920,
            height: 1080,
            refresh_rate: 60,
            clock: 148500,
        };

        let connector = Connector {
            id: ObjectId::new(),
            connector_type: ConnectorType::HDMI,
            status: ConnectorStatus::Connected,
            supported_modes: vec![mode.clone()],
            current_mode: None,
        };

        let connector_id = connector.id;
        let _ = driver.register_connector(connector);

        assert!(driver.get_primary_display().is_none());

        let _ = driver.set_mode(&connector_id, mode);

        assert!(driver.get_primary_display().is_some());
    }

    #[test]
    fn test_framebuffer_vram_cleanup() {
        let mut driver = GPUDriver::new(256 * 1024 * 1024);
        let initial = driver.get_memory_info().available_bytes;

        let fb1 = driver.allocate_framebuffer(1920, 1080).unwrap();
        let fb2 = driver.allocate_framebuffer(1280, 720).unwrap();

        let allocated = driver.get_memory_info().available_bytes;
        assert!(allocated < initial);

        driver.free_framebuffer(&fb1.id).unwrap();
        driver.free_framebuffer(&fb2.id).unwrap();

        let final_available = driver.get_memory_info().available_bytes;
        assert_eq!(final_available, initial);
    }
}
