use serde::{Deserialize, Serialize};
use sher_common::ObjectId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sandbox {
    pub id: ObjectId,
    pub name: String,
    pub isolated_resources: bool,
    pub network_access: bool,
    pub filesystem_access: bool,
}

impl Sandbox {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: ObjectId::new(),
            name: name.into(),
            isolated_resources: true,
            network_access: false,
            filesystem_access: false,
        }
    }

    pub fn allow_network(mut self) -> Self {
        self.network_access = true;
        self
    }

    pub fn allow_filesystem(mut self) -> Self {
        self.filesystem_access = true;
        self
    }

    /// Zero-trust default: a sandbox is "fully locked down" only if it
    /// isolates resources and grants neither network nor filesystem access.
    pub fn is_fully_locked_down(&self) -> bool {
        self.isolated_resources && !self.network_access && !self.filesystem_access
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sandbox_denies_by_default() {
        let sb = Sandbox::new("driver-a");
        assert!(sb.isolated_resources);
        assert!(!sb.network_access);
        assert!(!sb.filesystem_access);
        assert!(sb.is_fully_locked_down());
    }

    #[test]
    fn allow_methods_opt_in_to_access() {
        let sb = Sandbox::new("driver-b").allow_network();
        assert!(sb.network_access);
        assert!(!sb.is_fully_locked_down());

        let sb2 = Sandbox::new("driver-c").allow_filesystem().allow_network();
        assert!(sb2.filesystem_access);
        assert!(sb2.network_access);
    }
}
