use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelConfig {
    pub total_memory: u64,
    pub num_cpus: u32,
    pub enable_ai_services: bool,
    pub enable_linux_compatibility: bool,
    pub max_drivers: u32,
    pub security_level: SecurityLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    Standard,
    Strict,
    Maximum,
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self {
            total_memory: 8 * 1024 * 1024 * 1024,
            num_cpus: 4,
            enable_ai_services: true,
            enable_linux_compatibility: true,
            max_drivers: 256,
            security_level: SecurityLevel::Strict,
        }
    }
}
