// SHER Driver Runtime: Driver Loader
// Loads and manages driver binaries

use crate::container::{DriverCapability, DriverContainer, ResourceLimits};
use sher_common::{Error, Result};
use std::collections::HashMap;

// ============================================================================
// DRIVER MANIFEST
// ============================================================================

#[derive(Debug, Clone)]
pub struct DriverManifest {
    pub name: String,
    pub version: String,
    pub compatible_devices: Vec<(u16, u16)>, // (vendor_id, device_id)
    pub required_capabilities: Vec<DriverCapability>,
    pub memory_required_bytes: u64,
    pub entry_point: String,
}

// ============================================================================
// LINUX DRIVER WRAPPER
// ============================================================================

#[derive(Debug, Clone)]
pub struct LinuxDriver {
    pub path: String,
    pub name: String,
    pub version: String,
}

impl LinuxDriver {
    pub fn new(path: String, name: String, version: String) -> Self {
        LinuxDriver {
            path,
            name,
            version,
        }
    }
}

// ============================================================================
// DRIVER LOADER
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct DriverLoader {
    pub loaded_drivers: HashMap<String, DriverContainer>,
    pub linux_drivers: HashMap<String, LinuxDriver>,
    pub driver_cache: HashMap<String, DriverManifest>,
    pub total_loaded: u64,
}

impl DriverLoader {
    pub fn new() -> Self {
        DriverLoader::default()
    }

    /// Load a driver (Linux or native)
    pub fn load_driver(&mut self, path: &str, name: &str) -> Result<DriverContainer> {
        // Check if already loaded
        if self.loaded_drivers.contains_key(name) {
            return Err(Error::AllocationFailed(format!(
                "Driver {} already loaded",
                name
            )));
        }

        // Create container for driver
        let mut container = DriverContainer::new(name);

        // Grant default capabilities
        container.grant_capability(DriverCapability::ReadMemory);
        container.grant_capability(DriverCapability::WriteMemory);
        container.grant_capability(DriverCapability::InterruptHandling);

        // Set resource limits based on driver type
        if path.contains("linux") {
            // Linux drivers get more memory
            container.resource_limits = ResourceLimits {
                memory_limit_bytes: 512 * 1024 * 1024, // 512MB for Linux drivers
                cpu_quota_percent: 100,
                max_file_descriptors: 512,
                max_threads: 16,
                network_bandwidth_kbps: 50000,
            };
        }

        // Store Linux driver metadata if applicable
        if path.contains("linux") {
            self.linux_drivers.insert(
                name.to_string(),
                LinuxDriver::new(path.to_string(), name.to_string(), "1.0".to_string()),
            );
        }

        // Start the container
        container.start()?;

        // Register in pool
        self.loaded_drivers
            .insert(name.to_string(), container.clone());
        self.total_loaded += 1;

        Ok(container)
    }

    /// Unload a driver
    pub fn unload_driver(&mut self, name: &str) -> Result<()> {
        if let Some(mut container) = self.loaded_drivers.remove(name) {
            container.stop()?;
            self.linux_drivers.remove(name);
            Ok(())
        } else {
            Err(Error::AllocationFailed(format!(
                "Driver {} not found",
                name
            )))
        }
    }

    /// Reload a driver (stop and restart)
    pub fn reload_driver(&mut self, name: &str, path: &str) -> Result<DriverContainer> {
        self.unload_driver(name)?;
        self.load_driver(path, name)
    }

    /// Get loaded driver
    pub fn get_driver(&self, name: &str) -> Option<&DriverContainer> {
        self.loaded_drivers.get(name)
    }

    /// Get mutable reference to driver
    pub fn get_driver_mut(&mut self, name: &str) -> Option<&mut DriverContainer> {
        self.loaded_drivers.get_mut(name)
    }

    /// List all loaded drivers
    pub fn list_drivers(&self) -> Vec<&DriverContainer> {
        self.loaded_drivers.values().collect()
    }

    /// Count loaded drivers
    pub fn count_loaded(&self) -> usize {
        self.loaded_drivers.len()
    }

    /// Get Linux driver info
    pub fn get_linux_driver(&self, name: &str) -> Option<&LinuxDriver> {
        self.linux_drivers.get(name)
    }

    /// Register driver manifest (for capability checking)
    pub fn register_manifest(&mut self, manifest: DriverManifest) {
        self.driver_cache.insert(manifest.name.clone(), manifest);
    }

    /// Get driver manifest
    pub fn get_manifest(&self, name: &str) -> Option<&DriverManifest> {
        self.driver_cache.get(name)
    }

    /// Find drivers compatible with device
    pub fn find_drivers_for_device(&self, vendor_id: u16, device_id: u16) -> Vec<&DriverContainer> {
        self.loaded_drivers
            .values()
            .filter(|container| {
                if let Some(manifest) = self.driver_cache.get(&container.driver_name) {
                    manifest
                        .compatible_devices
                        .contains(&(vendor_id, device_id))
                } else {
                    false
                }
            })
            .collect()
    }

    /// Verify driver meets capability requirements
    pub fn verify_capabilities(&self, name: &str) -> Result<bool> {
        if let Some(manifest) = self.driver_cache.get(name) {
            if let Some(container) = self.get_driver(name) {
                for required_cap in &manifest.required_capabilities {
                    if !container.has_capability(*required_cap) {
                        return Err(Error::AllocationFailed(format!(
                            "Driver {} missing capability",
                            name
                        )));
                    }
                }
                Ok(true)
            } else {
                Err(Error::AllocationFailed(format!(
                    "Driver {} not loaded",
                    name
                )))
            }
        } else {
            Ok(false) // No manifest, assume compatible
        }
    }
}
