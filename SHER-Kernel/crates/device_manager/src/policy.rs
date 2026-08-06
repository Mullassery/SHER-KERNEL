use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePolicy {
    pub allow_unsigned_drivers: bool,
    pub power_management_enabled: bool,
    pub auto_restart_on_error: bool,
    pub max_restart_attempts: u32,
}

impl Default for DevicePolicy {
    fn default() -> Self {
        Self {
            allow_unsigned_drivers: false,
            power_management_enabled: true,
            auto_restart_on_error: true,
            max_restart_attempts: 3,
        }
    }
}
