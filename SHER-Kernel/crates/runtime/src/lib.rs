//! SHER Runtime - Stage 2 (Dynamic loading)
//!
//! Service loader that activates kernel services on demand:
//! - Storage? No. Loads only if /dev/disk accessed.
//! - Networking? No. Loads only if socket() called.
//! - GPU? No. Loads only if first GPU workload arrives.
//! - Bluetooth? No. Loads only if requested.
//!
//! This keeps the minimal kernel truly minimal while supporting rich functionality.

use sher_common::Result;
use std::collections::HashMap;
use tracing::info;

pub struct ServiceRegistry {
    services: HashMap<String, ServiceDescriptor>,
    loaded: Vec<String>,
}

pub struct ServiceDescriptor {
    pub name: String,
    pub required_for: Vec<String>,
    pub lazy: bool,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
            loaded: Vec::new(),
        }
    }

    pub fn register(&mut self, descriptor: ServiceDescriptor) {
        self.services.insert(descriptor.name.clone(), descriptor);
    }

    pub fn load_service(&mut self, name: &str) -> Result<()> {
        if self.loaded.contains(&name.to_string()) {
            return Ok(());
        }

        if let Some(service) = self.services.get(name) {
            info!("Lazy loading service: {}", service.name);
            self.loaded.push(name.to_string());
            Ok(())
        } else {
            Err(sher_common::Error::Unknown(format!("Service not found: {}", name)))
        }
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
