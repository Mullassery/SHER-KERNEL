//! Watchdog Monitoring System
//!
//! Continuous health monitoring with:
//! - Heartbeat tracking
//! - Deadlock detection
//! - Resource exhaustion alerts
//! - Automatic escalation

use sher_common::{ObjectId, Result};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    CriticalAlert,
    Unresponsive,
}

#[derive(Clone, Debug)]
pub struct HeartbeatRecord {
    pub last_heartbeat: u64,
    pub consecutive_misses: u32,
    pub healthy_streak: u32,
    pub status: HealthStatus,
}

/// Callback invoked when a driver's health status changes.
type AlertCallback = Box<dyn Fn(&ObjectId, &HealthStatus)>;

pub struct Watchdog {
    heartbeats: HashMap<ObjectId, HeartbeatRecord>,
    heartbeat_timeout_ms: u64,
    max_consecutive_misses: u32,
    alert_callbacks: Vec<AlertCallback>,
}

impl Watchdog {
    pub fn new(heartbeat_timeout_ms: u64) -> Self {
        Watchdog {
            heartbeats: HashMap::new(),
            heartbeat_timeout_ms,
            max_consecutive_misses: 3,
            alert_callbacks: Vec::new(),
        }
    }

    pub fn register_heartbeat(&mut self, driver_id: ObjectId) -> Result<()> {
        let record = HeartbeatRecord {
            last_heartbeat: self.current_time(),
            consecutive_misses: 0,
            healthy_streak: 0,
            status: HealthStatus::Healthy,
        };
        self.heartbeats.insert(driver_id, record);
        Ok(())
    }

    pub fn record_heartbeat(&mut self, driver_id: &ObjectId) -> Result<()> {
        let now = self.current_time();
        if let Some(record) = self.heartbeats.get_mut(driver_id) {
            record.last_heartbeat = now;
            record.consecutive_misses = 0;
            record.healthy_streak = record.healthy_streak.saturating_add(1);
            record.status = HealthStatus::Healthy;
        }
        Ok(())
    }

    pub fn check_health(&mut self, driver_id: &ObjectId) -> HealthStatus {
        let now = self.current_time();
        if let Some(record) = self.heartbeats.get_mut(driver_id) {
            let elapsed = now - record.last_heartbeat;
            let status_before = record.status.clone();

            if elapsed > self.heartbeat_timeout_ms {
                record.consecutive_misses += 1;

                record.status = match record.consecutive_misses {
                    1..=2 => HealthStatus::Degraded,
                    3 => HealthStatus::CriticalAlert,
                    _ => HealthStatus::Unresponsive,
                };

                if record.consecutive_misses >= self.max_consecutive_misses {
                    let new_status = record.status.clone();
                    let _ = record;
                    self.trigger_alerts(driver_id, &new_status);
                }
            }

            self.heartbeats
                .get(driver_id)
                .map(|r| r.status.clone())
                .unwrap_or(status_before)
        } else {
            HealthStatus::Unresponsive
        }
    }

    pub fn check_all_health(&mut self) -> HashMap<ObjectId, HealthStatus> {
        let driver_ids: Vec<_> = self.heartbeats.keys().cloned().collect();
        let mut statuses = HashMap::new();

        for driver_id in driver_ids {
            let status = self.check_health(&driver_id);
            statuses.insert(driver_id, status);
        }

        statuses
    }

    pub fn get_status(&self, driver_id: &ObjectId) -> Option<HealthStatus> {
        self.heartbeats.get(driver_id).map(|r| r.status.clone())
    }

    pub fn trigger_alerts(&self, driver_id: &ObjectId, status: &HealthStatus) {
        for callback in &self.alert_callbacks {
            callback(driver_id, status);
        }
    }

    pub fn register_alert_callback<F>(&mut self, callback: F)
    where
        F: Fn(&ObjectId, &HealthStatus) + 'static,
    {
        self.alert_callbacks.push(Box::new(callback));
    }

    pub fn unregister_driver(&mut self, driver_id: &ObjectId) -> Result<()> {
        self.heartbeats.remove(driver_id);
        Ok(())
    }

    fn current_time(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    pub fn get_statistics(&self) -> WatchdogStats {
        let total = self.heartbeats.len();
        let healthy = self
            .heartbeats
            .values()
            .filter(|r| r.status == HealthStatus::Healthy)
            .count();
        let degraded = self
            .heartbeats
            .values()
            .filter(|r| r.status == HealthStatus::Degraded)
            .count();
        let critical = self
            .heartbeats
            .values()
            .filter(|r| r.status == HealthStatus::CriticalAlert)
            .count();
        let unresponsive = self
            .heartbeats
            .values()
            .filter(|r| r.status == HealthStatus::Unresponsive)
            .count();

        WatchdogStats {
            total_monitored: total,
            healthy_count: healthy,
            degraded_count: degraded,
            critical_count: critical,
            unresponsive_count: unresponsive,
            health_percentage: if total > 0 {
                (healthy as f64 / total as f64) * 100.0
            } else {
                100.0
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct WatchdogStats {
    pub total_monitored: usize,
    pub healthy_count: usize,
    pub degraded_count: usize,
    pub critical_count: usize,
    pub unresponsive_count: usize,
    pub health_percentage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_heartbeat_registration() {
        let mut watchdog = Watchdog::new(1000);
        let driver = ObjectId::new();

        let _ = watchdog.register_heartbeat(driver.clone());
        assert!(watchdog.get_status(&driver).is_some());
    }

    #[test]
    fn test_heartbeat_recording() {
        let mut watchdog = Watchdog::new(1000);
        let driver = ObjectId::new();

        let _ = watchdog.register_heartbeat(driver.clone());
        let _ = watchdog.record_heartbeat(&driver);

        let status = watchdog.check_health(&driver);
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[test]
    fn test_degraded_status() {
        let mut watchdog = Watchdog::new(100);
        let driver = ObjectId::new();

        let _ = watchdog.register_heartbeat(driver.clone());
        std::thread::sleep(Duration::from_millis(150));

        let status = watchdog.check_health(&driver);
        assert_eq!(status, HealthStatus::Degraded);
    }

    #[test]
    fn test_multiple_drivers() {
        let mut watchdog = Watchdog::new(1000);
        let driver1 = ObjectId::new();
        let _driver2 = ObjectId::new();

        let _ = watchdog.register_heartbeat(driver1.clone());
        let _ = watchdog.register_heartbeat(_driver2.clone());

        let _ = watchdog.record_heartbeat(&driver1);

        let stats = watchdog.get_statistics();
        assert_eq!(stats.total_monitored, 2);
    }

    #[test]
    fn test_unregister_driver() {
        let mut watchdog = Watchdog::new(1000);
        let driver = ObjectId::new();

        let _ = watchdog.register_heartbeat(driver.clone());
        let _ = watchdog.unregister_driver(&driver);

        assert!(watchdog.get_status(&driver).is_none());
    }
}
