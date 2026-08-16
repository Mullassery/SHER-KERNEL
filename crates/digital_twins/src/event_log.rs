//! Event Logging for Digital Twin Recording
//!
//! Records all kernel operations (allocations, driver loads, interrupts, etc.)
//! for later replay and analysis.

use sher_common::{ObjectId, Result};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, PartialEq)]
pub enum EventType {
    MemoryAllocate {
        size: usize,
    },
    MemoryDeallocate {
        size: usize,
    },
    DriverLoad {
        driver_id: String,
    },
    DriverUnload {
        driver_id: String,
    },
    InterruptRaised {
        irq: usize,
    },
    InterruptHandled {
        irq: usize,
    },
    DeviceDiscovered {
        device_id: String,
    },
    DeviceRemoved {
        device_id: String,
    },
    CapabilityGranted {
        driver_id: String,
        capability: String,
    },
    CapabilityRevoked {
        driver_id: String,
        capability: String,
    },
    SchedulingDecision {
        workload_type: String,
    },
    AnomalyDetected {
        anomaly_type: String,
    },
    CrashRecovery {
        driver_id: String,
    },
    PerformanceMetric {
        metric_name: String,
        value: f64,
    },
}

#[derive(Clone, Debug)]
pub struct KernelEvent {
    pub timestamp: u64,
    pub sequence: u64,
    pub event_type: EventType,
    pub cpu_id: usize,
    pub context_id: Option<ObjectId>,
}

pub struct EventLog {
    events: VecDeque<KernelEvent>,
    max_events: usize,
    sequence_counter: u64,
    start_time: u64,
    enabled: bool,
}

impl EventLog {
    pub fn new(max_events: usize) -> Self {
        EventLog {
            events: VecDeque::with_capacity(max_events),
            max_events,
            sequence_counter: 0,
            start_time: Self::current_time(),
            enabled: true,
        }
    }

    pub fn record_event(
        &mut self,
        event_type: EventType,
        cpu_id: usize,
        context_id: Option<ObjectId>,
    ) -> Result<()> {
        if !self.enabled {
            return Ok(());
        }

        let event = KernelEvent {
            timestamp: Self::current_time(),
            sequence: self.sequence_counter,
            event_type,
            cpu_id,
            context_id,
        };

        self.sequence_counter += 1;

        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }

        self.events.push_back(event);
        Ok(())
    }

    pub fn get_events(&self) -> Vec<KernelEvent> {
        self.events.iter().cloned().collect()
    }

    pub fn get_event_range(&self, start_seq: u64, end_seq: u64) -> Vec<KernelEvent> {
        self.events
            .iter()
            .filter(|e| e.sequence >= start_seq && e.sequence <= end_seq)
            .cloned()
            .collect()
    }

    pub fn filter_by_event_type(&self, event_type: EventType) -> Vec<KernelEvent> {
        self.events
            .iter()
            .filter(|e| {
                std::mem::discriminant(&e.event_type) == std::mem::discriminant(&event_type)
            })
            .cloned()
            .collect()
    }

    pub fn filter_by_cpu(&self, cpu_id: usize) -> Vec<KernelEvent> {
        self.events
            .iter()
            .filter(|e| e.cpu_id == cpu_id)
            .cloned()
            .collect()
    }

    pub fn filter_by_time_range(&self, start_ms: u64, end_ms: u64) -> Vec<KernelEvent> {
        self.events
            .iter()
            .filter(|e| {
                e.timestamp >= self.start_time + start_ms && e.timestamp <= self.start_time + end_ms
            })
            .cloned()
            .collect()
    }

    pub fn clear(&mut self) -> Result<()> {
        self.events.clear();
        self.sequence_counter = 0;
        Ok(())
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn export_to_csv(&self) -> String {
        let mut csv = String::from("timestamp,sequence,event_type,cpu_id,context_id\n");

        for event in &self.events {
            let event_str = match &event.event_type {
                EventType::MemoryAllocate { size } => format!("MemoryAllocate({})", size),
                EventType::MemoryDeallocate { size } => format!("MemoryDeallocate({})", size),
                EventType::DriverLoad { driver_id } => format!("DriverLoad({})", driver_id),
                EventType::DriverUnload { driver_id } => format!("DriverUnload({})", driver_id),
                EventType::InterruptRaised { irq } => format!("InterruptRaised({})", irq),
                EventType::InterruptHandled { irq } => format!("InterruptHandled({})", irq),
                EventType::DeviceDiscovered { device_id } => {
                    format!("DeviceDiscovered({})", device_id)
                }
                EventType::DeviceRemoved { device_id } => format!("DeviceRemoved({})", device_id),
                EventType::CapabilityGranted {
                    driver_id,
                    capability,
                } => {
                    format!("CapabilityGranted({},{})", driver_id, capability)
                }
                EventType::CapabilityRevoked {
                    driver_id,
                    capability,
                } => {
                    format!("CapabilityRevoked({},{})", driver_id, capability)
                }
                EventType::SchedulingDecision { workload_type } => {
                    format!("SchedulingDecision({})", workload_type)
                }
                EventType::AnomalyDetected { anomaly_type } => {
                    format!("AnomalyDetected({})", anomaly_type)
                }
                EventType::CrashRecovery { driver_id } => format!("CrashRecovery({})", driver_id),
                EventType::PerformanceMetric { metric_name, value } => {
                    format!("PerformanceMetric({},{})", metric_name, value)
                }
            };

            let context_str = event
                .context_id
                .as_ref()
                .map(|id| id.to_string())
                .unwrap_or_else(|| "None".to_string());

            csv.push_str(&format!(
                "{},{},{},{},{}\n",
                event.timestamp, event.sequence, event_str, event.cpu_id, context_str
            ));
        }

        csv
    }

    fn current_time() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_event() {
        let mut log = EventLog::new(100);
        let event_type = EventType::MemoryAllocate { size: 256 };

        let _ = log.record_event(event_type, 0, None);
        assert_eq!(log.event_count(), 1);
    }

    #[test]
    fn test_event_sequence() {
        let mut log = EventLog::new(100);

        for _ in 0..5 {
            let _ = log.record_event(EventType::MemoryAllocate { size: 256 }, 0, None);
        }

        let events = log.get_events();
        assert_eq!(events[0].sequence, 0);
        assert_eq!(events[4].sequence, 4);
    }

    #[test]
    fn test_max_events_limit() {
        let mut log = EventLog::new(5);

        for _ in 0..10 {
            let _ = log.record_event(EventType::MemoryAllocate { size: 256 }, 0, None);
        }

        assert_eq!(log.event_count(), 5);
    }

    #[test]
    fn test_filter_by_event_type() {
        let mut log = EventLog::new(100);

        let _ = log.record_event(EventType::MemoryAllocate { size: 256 }, 0, None);
        let _ = log.record_event(EventType::MemoryDeallocate { size: 256 }, 0, None);
        let _ = log.record_event(EventType::MemoryAllocate { size: 512 }, 0, None);

        let allocs = log.filter_by_event_type(EventType::MemoryAllocate { size: 0 });
        assert!(allocs.len() >= 2);
    }

    #[test]
    fn test_filter_by_cpu() {
        let mut log = EventLog::new(100);

        let _ = log.record_event(EventType::MemoryAllocate { size: 256 }, 0, None);
        let _ = log.record_event(EventType::MemoryAllocate { size: 256 }, 1, None);
        let _ = log.record_event(EventType::MemoryAllocate { size: 256 }, 0, None);

        let cpu0_events = log.filter_by_cpu(0);
        assert_eq!(cpu0_events.len(), 2);
    }

    #[test]
    fn test_enable_disable() {
        let mut log = EventLog::new(100);

        let _ = log.record_event(EventType::MemoryAllocate { size: 256 }, 0, None);
        assert_eq!(log.event_count(), 1);

        log.disable();
        let _ = log.record_event(EventType::MemoryAllocate { size: 256 }, 0, None);
        assert_eq!(log.event_count(), 1);

        log.enable();
        let _ = log.record_event(EventType::MemoryAllocate { size: 256 }, 0, None);
        assert_eq!(log.event_count(), 2);
    }

    #[test]
    fn test_export_csv() {
        let mut log = EventLog::new(100);

        let _ = log.record_event(EventType::MemoryAllocate { size: 256 }, 0, None);
        let csv = log.export_to_csv();

        assert!(csv.contains("MemoryAllocate(256)"));
        assert!(csv.contains("timestamp,sequence,event_type,cpu_id,context_id"));
    }
}
