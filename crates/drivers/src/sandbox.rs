//! Driver sandbox bring-up: creates the isolated [`sher_security::Sandbox`]
//! a driver loads into on first access, defaulting to zero-trust (no
//! network, no filesystem) until the caller explicitly widens it.

use sher_security::Sandbox;

/// Create and return a fresh, fully locked-down sandbox for `driver_name`.
pub fn init(driver_name: &str) -> Sandbox {
    Sandbox::new(driver_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_creates_locked_down_sandbox() {
        let sandbox = init("e1000e");
        assert_eq!(sandbox.name, "e1000e");
        assert!(sandbox.isolated_resources);
        assert!(!sandbox.network_access);
        assert!(!sandbox.filesystem_access);
    }

    #[test]
    fn each_init_call_creates_a_distinct_sandbox() {
        let a = init("driver-a");
        let b = init("driver-a");
        assert_ne!(a.id, b.id);
    }
}
