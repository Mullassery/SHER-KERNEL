//! Maps a discovered device's class to a registered driver name — the
//! "driver matching algorithm" step between discovery and (isolated)
//! loading.

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct DriverRegistry {
    /// device_class -> driver name
    drivers: HashMap<String, String>,
}

impl DriverRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, device_class: impl Into<String>, driver_name: impl Into<String>) {
        self.drivers.insert(device_class.into(), driver_name.into());
    }

    pub fn unregister(&mut self, device_class: &str) -> bool {
        self.drivers.remove(device_class).is_some()
    }

    /// Find the driver registered for a device class, if any.
    pub fn match_driver(&self, device_class: &str) -> Option<&str> {
        self.drivers.get(device_class).map(String::as_str)
    }

    pub fn registered_classes(&self) -> Vec<&str> {
        let mut classes: Vec<&str> = self.drivers.keys().map(String::as_str).collect();
        classes.sort_unstable();
        classes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_match() {
        let mut registry = DriverRegistry::new();
        registry.register("network", "e1000e");
        assert_eq!(registry.match_driver("network"), Some("e1000e"));
    }

    #[test]
    fn unmatched_class_returns_none() {
        let registry = DriverRegistry::new();
        assert_eq!(registry.match_driver("gpu"), None);
    }

    #[test]
    fn re_registering_replaces_driver() {
        let mut registry = DriverRegistry::new();
        registry.register("network", "e1000e");
        registry.register("network", "r8169");
        assert_eq!(registry.match_driver("network"), Some("r8169"));
    }

    #[test]
    fn unregister_removes_mapping() {
        let mut registry = DriverRegistry::new();
        registry.register("audio", "snd_hda_intel");
        assert!(registry.unregister("audio"));
        assert_eq!(registry.match_driver("audio"), None);
        assert!(!registry.unregister("audio"));
    }
}
