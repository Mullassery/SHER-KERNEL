use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMonitor {
    pub detections: Vec<String>,
}

impl Default for AiMonitor {
    fn default() -> Self {
        Self {
            detections: vec![
                "memory_leak".to_string(),
                "interrupt_storm".to_string(),
                "dma_abuse".to_string(),
                "excessive_latency".to_string(),
            ],
        }
    }
}

impl AiMonitor {
    pub fn detect_anomaly(&self, metric: &str) -> bool {
        self.detections.iter().any(|d| d == metric)
    }
}
