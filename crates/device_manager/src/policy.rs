use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// DRIVER POLICY
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverPolicy {
    pub allow_unsigned_drivers: bool,
    pub require_capability_match: bool,
    pub prefer_native_drivers: bool,
    pub allow_emulated_drivers: bool,
    pub allow_generic_drivers: bool,
}

impl Default for DriverPolicy {
    fn default() -> Self {
        Self {
            allow_unsigned_drivers: false,
            require_capability_match: true,
            prefer_native_drivers: true,
            allow_emulated_drivers: true,
            allow_generic_drivers: false,
        }
    }
}

// ============================================================================
// DEVICE POLICY
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePolicy {
    pub driver_policy: DriverPolicy,
    pub power_management_enabled: bool,
    pub auto_restart_on_error: bool,
    pub max_restart_attempts: u32,
    pub restart_backoff_ms: u32,
    pub enable_hotplug: bool,
    pub enable_suspend_resume: bool,
    pub error_threshold: u64,
    pub error_action: ErrorAction,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ErrorAction {
    Ignore,
    Log,
    Restart,
    Isolate,
    Shutdown,
}

impl Default for DevicePolicy {
    fn default() -> Self {
        Self {
            driver_policy: DriverPolicy::default(),
            power_management_enabled: true,
            auto_restart_on_error: true,
            max_restart_attempts: 3,
            restart_backoff_ms: 100,
            enable_hotplug: true,
            enable_suspend_resume: true,
            error_threshold: 10,
            error_action: ErrorAction::Restart,
        }
    }
}

// ============================================================================
// DRIVER MATCHER
// ============================================================================

#[derive(Debug, Clone)]
pub struct DriverMatch {
    pub driver_id: String,
    pub match_type: MatchType,
    pub confidence: f64, // 0.0-1.0
    pub native: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchType {
    ExactVendorDevice, // VendorID + DeviceID exact match
    ClassCode,         // Class/subclass match
    Generic,           // Generic class driver
}

impl MatchType {
    pub fn priority(&self) -> u8 {
        match self {
            MatchType::ExactVendorDevice => 100,
            MatchType::ClassCode => 50,
            MatchType::Generic => 10,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DriverDatabase {
    pub drivers: HashMap<String, DriverEntry>,
    pub vendor_device_map: HashMap<(u16, u16), Vec<String>>, // (vendor, device) -> [driver_ids]
    pub class_map: HashMap<(u8, u8), Vec<String>>,           // (class, subclass) -> [driver_ids]
}

#[derive(Debug, Clone)]
pub struct DriverEntry {
    pub id: String,
    pub name: String,
    pub vendor_id: Option<u16>,
    pub device_id: Option<u16>,
    pub device_class: Option<u8>,
    pub device_subclass: Option<u8>,
    pub native: bool,
    pub version: String,
    pub required_capabilities: Vec<String>,
}

impl DriverDatabase {
    pub fn new() -> Self {
        DriverDatabase {
            drivers: HashMap::new(),
            vendor_device_map: HashMap::new(),
            class_map: HashMap::new(),
        }
    }

    pub fn register_driver(&mut self, driver: DriverEntry) {
        let driver_id = driver.id.clone();

        if let (Some(vendor), Some(device)) = (driver.vendor_id, driver.device_id) {
            self.vendor_device_map
                .entry((vendor, device))
                .or_default()
                .push(driver_id.clone());
        }

        if let (Some(class), Some(subclass)) = (driver.device_class, driver.device_subclass) {
            self.class_map
                .entry((class, subclass))
                .or_default()
                .push(driver_id.clone());
        }

        self.drivers.insert(driver_id, driver);
    }

    pub fn find_exact_match(&self, vendor: u16, device: u16) -> Vec<&DriverEntry> {
        self.vendor_device_map
            .get(&(vendor, device))
            .map(|ids| ids.iter().filter_map(|id| self.drivers.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn find_class_match(&self, class: u8, subclass: u8) -> Vec<&DriverEntry> {
        self.class_map
            .get(&(class, subclass))
            .map(|ids| ids.iter().filter_map(|id| self.drivers.get(id)).collect())
            .unwrap_or_default()
    }
}

// ============================================================================
// MATCHING ENGINE
// ============================================================================

pub struct DriverMatcher {
    pub database: DriverDatabase,
    pub policy: DriverPolicy,
}

impl DriverMatcher {
    pub fn new(policy: DriverPolicy) -> Self {
        DriverMatcher {
            database: DriverDatabase::new(),
            policy,
        }
    }

    pub fn find_best_match(
        &self,
        vendor: u16,
        device: u16,
        class: u8,
        subclass: u8,
    ) -> Option<DriverMatch> {
        let mut candidates: Vec<(DriverEntry, MatchType, u8)> = Vec::new();

        // Try exact vendor/device match first
        for driver in self.database.find_exact_match(vendor, device) {
            if self.policy.allow_unsigned_drivers || driver.native {
                candidates.push((
                    driver.clone(),
                    MatchType::ExactVendorDevice,
                    MatchType::ExactVendorDevice.priority(),
                ));
            }
        }

        // Try class match if no exact match
        if candidates.is_empty() && self.policy.allow_emulated_drivers {
            for driver in self.database.find_class_match(class, subclass) {
                candidates.push((
                    driver.clone(),
                    MatchType::ClassCode,
                    MatchType::ClassCode.priority(),
                ));
            }
        }

        // Sort by priority (descending)
        candidates.sort_by_key(|c| std::cmp::Reverse(c.2));

        // Return best match
        candidates.first().map(|(driver, match_type, priority)| {
            let confidence = (*priority as f64) / 100.0;
            DriverMatch {
                driver_id: driver.id.clone(),
                match_type: *match_type,
                confidence,
                native: driver.native,
            }
        })
    }
}
