//! Crash Recovery and Resilience System
//!
//! Handles driver crashes, system failures, and graceful recovery with:
//! - Crash detection and isolation
//! - Exponential backoff restart
//! - State persistence and recovery
//! - Watchdog monitoring
//! - Graceful degradation

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use sher_common::{ObjectId, Result};

#[derive(Clone, Debug, PartialEq)]
pub enum RecoveryState {
    Healthy,
    Degraded,
    Recovering,
    Failed,
}

#[derive(Clone, Debug)]
pub struct CrashMetrics {
    pub crash_count: usize,
    pub last_crash_time: u64,
    pub recovery_attempts: usize,
    pub successful_recoveries: usize,
    pub failed_recoveries: usize,
    pub total_downtime_ms: u64,
}

impl Default for CrashMetrics {
    fn default() -> Self {
        CrashMetrics {
            crash_count: 0,
            last_crash_time: 0,
            recovery_attempts: 0,
            successful_recoveries: 0,
            failed_recoveries: 0,
            total_downtime_ms: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RecoveryPolicy {
    pub max_restart_attempts: usize,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
    pub backoff_multiplier: f64,
    pub quarantine_threshold_crashes: usize,
    pub quarantine_duration_ms: u64,
}

impl Default for RecoveryPolicy {
    fn default() -> Self {
        RecoveryPolicy {
            max_restart_attempts: 5,
            initial_backoff_ms: 100,
            max_backoff_ms: 30000,
            backoff_multiplier: 2.0,
            quarantine_threshold_crashes: 3,
            quarantine_duration_ms: 60000,
        }
    }
}

pub struct CrashRecoveryManager {
    metrics: HashMap<ObjectId, CrashMetrics>,
    policy: RecoveryPolicy,
    quarantined: HashMap<ObjectId, u64>,
    recovery_state: RecoveryState,
}

impl CrashRecoveryManager {
    pub fn new(policy: RecoveryPolicy) -> Self {
        CrashRecoveryManager {
            metrics: HashMap::new(),
            policy,
            quarantined: HashMap::new(),
            recovery_state: RecoveryState::Healthy,
        }
    }

    pub fn record_crash(&mut self, driver_id: ObjectId) -> Result<()> {
        let now = self.current_time();
        let metrics = self.metrics.entry(driver_id.clone())
            .or_insert_with(CrashMetrics::default);

        metrics.crash_count += 1;
        metrics.last_crash_time = now;

        if metrics.crash_count >= self.policy.quarantine_threshold_crashes {
            self.quarantine_driver(driver_id.clone())?;
            self.recovery_state = RecoveryState::Degraded;
        }

        Ok(())
    }

    pub fn calculate_backoff(&self, attempt: usize) -> Duration {
        let backoff_ms = (self.policy.initial_backoff_ms as f64
            * self.policy.backoff_multiplier.powi(attempt as i32)) as u64;
        let capped = std::cmp::min(backoff_ms, self.policy.max_backoff_ms);
        Duration::from_millis(capped)
    }

    pub fn should_recover(&self, driver_id: &ObjectId) -> bool {
        if self.is_quarantined(driver_id) {
            return false;
        }

        if let Some(metrics) = self.metrics.get(driver_id) {
            metrics.recovery_attempts < self.policy.max_restart_attempts
        } else {
            true
        }
    }

    pub fn record_recovery_attempt(&mut self, driver_id: ObjectId) -> Result<()> {
        let metrics = self.metrics.entry(driver_id)
            .or_insert_with(CrashMetrics::default);
        metrics.recovery_attempts += 1;
        Ok(())
    }

    pub fn record_successful_recovery(&mut self, driver_id: ObjectId, downtime_ms: u64) -> Result<()> {
        let metrics = self.metrics.entry(driver_id)
            .or_insert_with(CrashMetrics::default);
        metrics.successful_recoveries += 1;
        metrics.total_downtime_ms += downtime_ms;
        Ok(())
    }

    pub fn record_failed_recovery(&mut self, driver_id: ObjectId) -> Result<()> {
        let metrics = self.metrics.entry(driver_id)
            .or_insert_with(CrashMetrics::default);
        metrics.failed_recoveries += 1;
        Ok(())
    }

    pub fn quarantine_driver(&mut self, driver_id: ObjectId) -> Result<()> {
        self.quarantined.insert(driver_id, self.current_time());
        Ok(())
    }

    pub fn is_quarantined(&self, driver_id: &ObjectId) -> bool {
        if let Some(quarantine_time) = self.quarantined.get(driver_id) {
            let elapsed = self.current_time() - quarantine_time;
            elapsed < self.policy.quarantine_duration_ms
        } else {
            false
        }
    }

    pub fn release_quarantine(&mut self, driver_id: &ObjectId) -> Result<()> {
        self.quarantined.remove(driver_id);
        Ok(())
    }

    pub fn get_metrics(&self, driver_id: &ObjectId) -> Option<CrashMetrics> {
        self.metrics.get(driver_id).cloned()
    }

    pub fn get_recovery_state(&self) -> RecoveryState {
        self.recovery_state.clone()
    }

    pub fn health_check(&mut self) -> RecoveryState {
        let healthy_drivers = self.metrics.values()
            .filter(|m| m.crash_count == 0)
            .count();

        let total_drivers = self.metrics.len();

        self.recovery_state = if healthy_drivers == total_drivers {
            RecoveryState::Healthy
        } else if healthy_drivers > total_drivers / 2 {
            RecoveryState::Degraded
        } else if self.metrics.values().any(|m| m.recovery_attempts < 2) {
            RecoveryState::Recovering
        } else {
            RecoveryState::Failed
        };

        self.recovery_state.clone()
    }

    fn current_time(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    pub fn reset_metrics(&mut self, driver_id: &ObjectId) -> Result<()> {
        self.metrics.remove(driver_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crash_detection() {
        let policy = RecoveryPolicy::default();
        let mut manager = CrashRecoveryManager::new(policy);
        let driver = ObjectId::new();

        let _ = manager.record_crash(driver.clone());
        assert_eq!(manager.get_metrics(&driver).unwrap().crash_count, 1);
    }

    #[test]
    fn test_exponential_backoff() {
        let policy = RecoveryPolicy::default();
        let manager = CrashRecoveryManager::new(policy);

        let backoff_0 = manager.calculate_backoff(0).as_millis();
        let backoff_1 = manager.calculate_backoff(1).as_millis();
        let backoff_2 = manager.calculate_backoff(2).as_millis();

        assert!(backoff_1 > backoff_0);
        assert!(backoff_2 > backoff_1);
    }

    #[test]
    fn test_quarantine_threshold() {
        let mut policy = RecoveryPolicy::default();
        policy.quarantine_threshold_crashes = 2;

        let mut manager = CrashRecoveryManager::new(policy);
        let driver = ObjectId::new();

        let _ = manager.record_crash(driver.clone());
        assert!(!manager.is_quarantined(&driver));

        let _ = manager.record_crash(driver.clone());
        assert_eq!(manager.recovery_state, RecoveryState::Degraded);
    }

    #[test]
    fn test_recovery_limits() {
        let policy = RecoveryPolicy::default();
        let mut manager = CrashRecoveryManager::new(policy);
        let driver = ObjectId::new();

        for _ in 0..5 {
            let _ = manager.record_recovery_attempt(driver.clone());
        }

        assert!(!manager.should_recover(&driver));
    }

    #[test]
    fn test_health_check() {
        let policy = RecoveryPolicy::default();
        let mut manager = CrashRecoveryManager::new(policy);
        let driver1 = ObjectId::new();

        let _ = manager.record_crash(driver1);
        let health = manager.health_check();

        assert_ne!(health, RecoveryState::Healthy);
    }

    #[test]
    fn test_reset_metrics() {
        let policy = RecoveryPolicy::default();
        let mut manager = CrashRecoveryManager::new(policy);
        let driver = ObjectId::new();

        let _ = manager.record_crash(driver.clone());
        assert!(manager.get_metrics(&driver).is_some());

        let _ = manager.reset_metrics(&driver);
        assert!(manager.get_metrics(&driver).is_none());
    }
}
