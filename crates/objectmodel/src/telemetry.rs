use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Telemetry {
    pub events_processed: u64,
    pub errors_encountered: u64,
    pub last_activity: Option<u64>,
    pub memory_used: u64,
    pub cpu_time_ms: u64,
    pub custom_metrics: std::collections::HashMap<String, f64>,
}

impl Telemetry {
    pub fn record_event(&mut self) {
        self.events_processed += 1;
        self.last_activity = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
    }

    pub fn record_error(&mut self) {
        self.errors_encountered += 1;
    }

    pub fn set_metric(&mut self, key: impl Into<String>, value: f64) {
        self.custom_metrics.insert(key.into(), value);
    }

    pub fn is_healthy(&self) -> bool {
        if self.events_processed == 0 {
            return true;
        }
        let error_rate = self.errors_encountered as f64 / self.events_processed as f64;
        error_rate < 0.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_events_is_healthy_by_default() {
        let t = Telemetry::default();
        assert!(t.is_healthy());
    }

    #[test]
    fn record_event_increments_counter_and_timestamp() {
        let mut t = Telemetry::default();
        t.record_event();
        t.record_event();
        assert_eq!(t.events_processed, 2);
        assert!(t.last_activity.is_some());
    }

    #[test]
    fn high_error_rate_is_unhealthy() {
        let mut t = Telemetry::default();
        for _ in 0..10 {
            t.record_event();
        }
        for _ in 0..2 {
            t.record_error();
        }
        // 2/10 = 20% error rate, over the 10% threshold.
        assert!(!t.is_healthy());
    }

    #[test]
    fn low_error_rate_stays_healthy() {
        let mut t = Telemetry::default();
        for _ in 0..100 {
            t.record_event();
        }
        t.record_error();
        // 1/100 = 1% error rate, under the 10% threshold.
        assert!(t.is_healthy());
    }

    #[test]
    fn custom_metrics_are_settable() {
        let mut t = Telemetry::default();
        t.set_metric("queue_depth", 42.0);
        assert_eq!(t.custom_metrics.get("queue_depth"), Some(&42.0));
    }
}
