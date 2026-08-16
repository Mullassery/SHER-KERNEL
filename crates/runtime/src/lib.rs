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
            Err(sher_common::Error::Unknown(format!(
                "Service not found: {}",
                name
            )))
        }
    }

    pub fn is_loaded(&self, name: &str) -> bool {
        self.loaded.iter().any(|s| s == name)
    }

    pub fn loaded_services(&self) -> &[String] {
        &self.loaded
    }

    pub fn is_registered(&self, name: &str) -> bool {
        self.services.contains_key(name)
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn descriptor(name: &str, lazy: bool) -> ServiceDescriptor {
        ServiceDescriptor {
            name: name.to_string(),
            required_for: Vec::new(),
            lazy,
        }
    }

    #[test]
    fn loading_unregistered_service_errors() {
        let mut registry = ServiceRegistry::new();
        assert!(registry.load_service("storage").is_err());
    }

    #[test]
    fn registered_service_loads_and_is_tracked() {
        let mut registry = ServiceRegistry::new();
        registry.register(descriptor("networking", true));
        assert!(registry.is_registered("networking"));
        assert!(!registry.is_loaded("networking"));

        registry.load_service("networking").unwrap();
        assert!(registry.is_loaded("networking"));
        assert_eq!(registry.loaded_services(), &["networking".to_string()]);
    }

    #[test]
    fn loading_twice_is_idempotent() {
        let mut registry = ServiceRegistry::new();
        registry.register(descriptor("gpu", true));
        registry.load_service("gpu").unwrap();
        registry.load_service("gpu").unwrap();
        assert_eq!(registry.loaded_services().len(), 1);
    }
}
