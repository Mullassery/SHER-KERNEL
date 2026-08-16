//! Service lifecycle manager.
//!
//! Implements the lazy-loading policy described in the crate root docs:
//! a boot *profile* determines which optional services are loaded by
//! default, and any service can additionally be loaded on demand (e.g. the
//! first time a socket/filesystem/display call is made). This is real
//! in-process state-machine logic; the "services" themselves
//! (`filesystem`, `networking`, `storage`, `display`, `audio`) are thin
//! marker types — this crate does not implement a real filesystem or
//! display server, it only implements the lifecycle policy that would
//! decide when such a service gets activated in a full kernel.

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceKind {
    Filesystem,
    Networking,
    Storage,
    Display,
    Audio,
}

impl ServiceKind {
    pub const ALL: [ServiceKind; 5] = [
        ServiceKind::Filesystem,
        ServiceKind::Networking,
        ServiceKind::Storage,
        ServiceKind::Display,
        ServiceKind::Audio,
    ];
}

/// Boot profile controlling which services are loaded by default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// Server boot: never loads display, audio.
    Server,
    /// Workstation: loads filesystem, networking, storage, display, audio.
    Workstation,
    /// Headless: minimal services only (filesystem).
    Headless,
}

impl Profile {
    /// Services this profile loads automatically at boot.
    pub fn default_services(&self) -> Vec<ServiceKind> {
        match self {
            Profile::Server => vec![
                ServiceKind::Filesystem,
                ServiceKind::Networking,
                ServiceKind::Storage,
            ],
            Profile::Workstation => vec![
                ServiceKind::Filesystem,
                ServiceKind::Networking,
                ServiceKind::Storage,
                ServiceKind::Display,
                ServiceKind::Audio,
            ],
            Profile::Headless => vec![ServiceKind::Filesystem],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceState {
    Unloaded,
    Loading,
    Loaded,
    Failed(String),
}

#[derive(Debug, Default)]
pub struct ServiceManager {
    state: HashMap<ServiceKind, ServiceState>,
}

impl ServiceManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self, kind: ServiceKind) -> ServiceState {
        self.state
            .get(&kind)
            .cloned()
            .unwrap_or(ServiceState::Unloaded)
    }

    pub fn is_loaded(&self, kind: ServiceKind) -> bool {
        matches!(self.state(kind), ServiceState::Loaded)
    }

    /// Load a single service on demand (idempotent: loading an already
    /// loaded service is a no-op success).
    pub fn load(&mut self, kind: ServiceKind) -> Result<(), String> {
        if self.is_loaded(kind) {
            return Ok(());
        }
        self.state.insert(kind, ServiceState::Loading);
        // Real service startup would happen here; this simulation always
        // succeeds since there is no actual subsystem to fail against.
        self.state.insert(kind, ServiceState::Loaded);
        Ok(())
    }

    pub fn unload(&mut self, kind: ServiceKind) {
        self.state.insert(kind, ServiceState::Unloaded);
    }

    /// Load every service the given profile enables by default.
    pub fn apply_profile(&mut self, profile: Profile) -> Result<(), String> {
        for kind in profile.default_services() {
            self.load(kind)?;
        }
        Ok(())
    }

    pub fn loaded_services(&self) -> Vec<ServiceKind> {
        ServiceKind::ALL
            .iter()
            .copied()
            .filter(|k| self.is_loaded(*k))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_profile_never_loads_display_or_audio() {
        let mut mgr = ServiceManager::new();
        mgr.apply_profile(Profile::Server).unwrap();
        assert!(mgr.is_loaded(ServiceKind::Filesystem));
        assert!(mgr.is_loaded(ServiceKind::Networking));
        assert!(!mgr.is_loaded(ServiceKind::Display));
        assert!(!mgr.is_loaded(ServiceKind::Audio));
    }

    #[test]
    fn workstation_profile_loads_everything() {
        let mut mgr = ServiceManager::new();
        mgr.apply_profile(Profile::Workstation).unwrap();
        for kind in ServiceKind::ALL {
            assert!(mgr.is_loaded(kind), "{:?} should be loaded", kind);
        }
    }

    #[test]
    fn headless_profile_is_minimal() {
        let mut mgr = ServiceManager::new();
        mgr.apply_profile(Profile::Headless).unwrap();
        assert_eq!(mgr.loaded_services(), vec![ServiceKind::Filesystem]);
    }

    #[test]
    fn load_is_idempotent() {
        let mut mgr = ServiceManager::new();
        mgr.load(ServiceKind::Networking).unwrap();
        mgr.load(ServiceKind::Networking).unwrap();
        assert!(mgr.is_loaded(ServiceKind::Networking));
    }

    #[test]
    fn unload_resets_state() {
        let mut mgr = ServiceManager::new();
        mgr.load(ServiceKind::Storage).unwrap();
        assert!(mgr.is_loaded(ServiceKind::Storage));
        mgr.unload(ServiceKind::Storage);
        assert!(!mgr.is_loaded(ServiceKind::Storage));
        assert_eq!(mgr.state(ServiceKind::Storage), ServiceState::Unloaded);
    }

    #[test]
    fn on_demand_load_can_happen_outside_profile() {
        let mut mgr = ServiceManager::new();
        mgr.apply_profile(Profile::Server).unwrap();
        assert!(!mgr.is_loaded(ServiceKind::Display));
        mgr.load(ServiceKind::Display).unwrap();
        assert!(mgr.is_loaded(ServiceKind::Display));
    }
}
