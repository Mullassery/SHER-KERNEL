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
