// SHER LKI: Audit Logging
// Complete audit trail of all Linux API translations

use sher_common::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

// ============================================================================
// AUDIT FRAMEWORK
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditLevel {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp_ms: u64,
    pub level: AuditLevel,
    pub driver_id: ObjectId,
    pub api_name: String,
    pub operation: String,
    pub result: String,
    pub duration_us: u32,
}

impl AuditEntry {
    pub fn new(driver_id: ObjectId, api_name: &str, operation: &str) -> Self {
        AuditEntry {
            timestamp_ms: 0,
            level: AuditLevel::Info,
            driver_id,
            api_name: api_name.to_string(),
            operation: operation.to_string(),
            result: String::new(),
            duration_us: 0,
        }
    }

    pub fn with_level(mut self, level: AuditLevel) -> Self {
        self.level = level;
        self
    }

    pub fn with_result(mut self, result: &str) -> Self {
        self.result = result.to_string();
        self
    }

    pub fn with_duration(mut self, duration_us: u32) -> Self {
        self.duration_us = duration_us;
        self
    }
}

// ============================================================================
// AUDIT LOG
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct AuditLog {
    entries: VecDeque<AuditEntry>,
    max_entries: usize,
    total_entries: u64,
    info_count: u64,
    warning_count: u64,
    error_count: u64,
    critical_count: u64,
}

impl AuditLog {
    pub fn new(max_entries: usize) -> Self {
        AuditLog {
            entries: VecDeque::with_capacity(max_entries),
            max_entries,
            total_entries: 0,
            info_count: 0,
            warning_count: 0,
            error_count: 0,
            critical_count: 0,
        }
    }

    /// Add entry to audit log
    pub fn log(&mut self, entry: AuditEntry) {
        // Update statistics
        self.total_entries += 1;
        match entry.level {
            AuditLevel::Info => self.info_count += 1,
            AuditLevel::Warning => self.warning_count += 1,
            AuditLevel::Error => self.error_count += 1,
            AuditLevel::Critical => self.critical_count += 1,
        }

        // Add to queue, removing oldest if at capacity
        if self.entries.len() >= self.max_entries {
            self.entries.pop_front();
        }
        self.entries.push_back(entry);
    }

    /// Get all entries
    pub fn entries(&self) -> Vec<&AuditEntry> {
        self.entries.iter().collect()
    }

    /// Get entries by level
    pub fn entries_by_level(&self, level: AuditLevel) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.level == level).collect()
    }

    /// Get entries by driver
    pub fn entries_by_driver(&self, driver_id: ObjectId) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.driver_id == driver_id).collect()
    }

    /// Get entries by API
    pub fn entries_by_api(&self, api_name: &str) -> Vec<&AuditEntry> {
        self.entries.iter().filter(|e| e.api_name == api_name).collect()
    }

    /// Get recent entries
    pub fn recent_entries(&self, count: usize) -> Vec<&AuditEntry> {
        self.entries.iter().rev().take(count).collect()
    }

    /// Get total entries count
    pub fn total_entries(&self) -> u64 {
        self.total_entries
    }

    /// Get info count
    pub fn info_count(&self) -> u64 {
        self.info_count
    }

    /// Get warning count
    pub fn warning_count(&self) -> u64 {
        self.warning_count
    }

    /// Get error count
    pub fn error_count(&self) -> u64 {
        self.error_count
    }

    /// Get critical count
    pub fn critical_count(&self) -> u64 {
        self.critical_count
    }

    /// Get audit statistics
    pub fn stats(&self) -> AuditStats {
        AuditStats {
            total_entries: self.total_entries,
            info_count: self.info_count,
            warning_count: self.warning_count,
            error_count: self.error_count,
            critical_count: self.critical_count,
            error_rate: if self.total_entries > 0 {
                ((self.error_count + self.critical_count) as f64 / self.total_entries as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Clear audit log
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Get average API latency
    pub fn avg_latency_for_api(&self, api_name: &str) -> u32 {
        let entries = self.entries_by_api(api_name);
        if entries.is_empty() {
            0
        } else {
            (entries.iter().map(|e| e.duration_us as u64).sum::<u64>() / entries.len() as u64) as u32
        }
    }

    /// Get peak latency for API
    pub fn peak_latency_for_api(&self, api_name: &str) -> u32 {
        self.entries_by_api(api_name)
            .iter()
            .map(|e| e.duration_us)
            .max()
            .unwrap_or(0)
    }
}

// ============================================================================
// AUDIT STATISTICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total_entries: u64,
    pub info_count: u64,
    pub warning_count: u64,
    pub error_count: u64,
    pub critical_count: u64,
    pub error_rate: f64,
}

// ============================================================================
// AUDIT FILTER
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct AuditFilter {
    pub min_level: Option<AuditLevel>,
    pub driver_id: Option<ObjectId>,
    pub api_name: Option<String>,
    pub min_latency_us: Option<u32>,
}

impl AuditFilter {
    pub fn new() -> Self {
        AuditFilter::default()
    }

    pub fn with_level(mut self, level: AuditLevel) -> Self {
        self.min_level = Some(level);
        self
    }

    pub fn with_driver(mut self, driver_id: ObjectId) -> Self {
        self.driver_id = Some(driver_id);
        self
    }

    pub fn with_api(mut self, api_name: &str) -> Self {
        self.api_name = Some(api_name.to_string());
        self
    }

    pub fn with_min_latency(mut self, us: u32) -> Self {
        self.min_latency_us = Some(us);
        self
    }

    pub fn matches(&self, entry: &AuditEntry) -> bool {
        if let Some(level) = self.min_level {
            if (entry.level as u32) < (level as u32) {
                return false;
            }
        }

        if let Some(driver_id) = self.driver_id {
            if entry.driver_id != driver_id {
                return false;
            }
        }

        if let Some(ref api_name) = self.api_name {
            if entry.api_name != *api_name {
                return false;
            }
        }

        if let Some(min_latency) = self.min_latency_us {
            if entry.duration_us < min_latency {
                return false;
            }
        }

        true
    }
}
