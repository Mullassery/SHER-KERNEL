//! Event Replay Engine for Digital Twin Simulation
//!
//! Replays recorded kernel events in controlled environments for:
//! - Debugging complex scenarios
//! - Reproducing bugs
//! - What-if analysis
//! - Performance testing

use crate::event_log::{EventType, KernelEvent};
use sher_common::Result;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum ReplayMode {
    RealTime,    // Replay at original timing
    FastForward, // Replay as fast as possible
    StepWise,    // Step through one event at a time
    Filtered,    // Replay only matching events
}

#[derive(Clone, Debug)]
pub struct ReplayStats {
    pub events_replayed: usize,
    pub events_failed: usize,
    pub total_duration_ms: u64,
    pub avg_event_latency_us: f64,
}

/// Callback invoked for each event of a given type during replay.
type EventHandler = Box<dyn Fn(&KernelEvent) -> Result<()>>;

pub struct ReplayEngine {
    mode: ReplayMode,
    events: Vec<KernelEvent>,
    current_position: usize,
    stats: ReplayStats,
    event_handlers: HashMap<String, EventHandler>,
}

impl ReplayEngine {
    pub fn new(mode: ReplayMode, events: Vec<KernelEvent>) -> Self {
        ReplayEngine {
            mode,
            events,
            current_position: 0,
            stats: ReplayStats {
                events_replayed: 0,
                events_failed: 0,
                total_duration_ms: 0,
                avg_event_latency_us: 0.0,
            },
            event_handlers: HashMap::new(),
        }
    }

    pub fn register_event_handler<F>(&mut self, event_type: &str, handler: F) -> Result<()>
    where
        F: Fn(&KernelEvent) -> Result<()> + 'static,
    {
        self.event_handlers
            .insert(event_type.to_string(), Box::new(handler));
        Ok(())
    }

    pub fn replay_all(&mut self) -> Result<()> {
        match self.mode {
            ReplayMode::RealTime => self.replay_real_time(),
            ReplayMode::FastForward => self.replay_fast_forward(),
            ReplayMode::StepWise => self.replay_step_wise(),
            ReplayMode::Filtered => self.replay_filtered(),
        }
    }

    pub fn replay_next(&mut self) -> Result<bool> {
        if self.current_position >= self.events.len() {
            return Ok(false);
        }

        let event = &self.events[self.current_position].clone();
        self.process_event(event)?;

        self.current_position += 1;
        Ok(self.current_position < self.events.len())
    }

    pub fn replay_range(&mut self, start_seq: u64, end_seq: u64) -> Result<()> {
        let range_events: Vec<_> = self
            .events
            .iter()
            .filter(|e| e.sequence >= start_seq && e.sequence <= end_seq)
            .cloned()
            .collect();

        for event in range_events {
            self.process_event(&event)?;
        }

        Ok(())
    }

    pub fn replay_until(&mut self, predicate: impl Fn(&KernelEvent) -> bool) -> Result<()> {
        while self.current_position < self.events.len() {
            let event = &self.events[self.current_position].clone();

            self.process_event(event)?;
            self.current_position += 1;

            if predicate(event) {
                break;
            }
        }

        Ok(())
    }

    pub fn get_stats(&self) -> ReplayStats {
        self.stats.clone()
    }

    pub fn reset(&mut self) -> Result<()> {
        self.current_position = 0;
        self.stats = ReplayStats {
            events_replayed: 0,
            events_failed: 0,
            total_duration_ms: 0,
            avg_event_latency_us: 0.0,
        };
        Ok(())
    }

    pub fn jump_to(&mut self, sequence: u64) -> Result<()> {
        self.current_position = self
            .events
            .iter()
            .position(|e| e.sequence == sequence)
            .unwrap_or(0);
        Ok(())
    }

    pub fn get_position(&self) -> usize {
        self.current_position
    }

    pub fn get_total_events(&self) -> usize {
        self.events.len()
    }

    fn replay_real_time(&mut self) -> Result<()> {
        let start_time = std::time::Instant::now();
        let base_timestamp = if !self.events.is_empty() {
            self.events[0].timestamp
        } else {
            0
        };

        for event in &self.events.clone() {
            let relative_time = event.timestamp.saturating_sub(base_timestamp);
            let elapsed = start_time.elapsed().as_millis() as u64;

            if elapsed < relative_time {
                std::thread::sleep(std::time::Duration::from_millis(relative_time - elapsed));
            }

            let _ = self.process_event(event);
        }

        Ok(())
    }

    fn replay_fast_forward(&mut self) -> Result<()> {
        for event in &self.events.clone() {
            self.process_event(event)?;
        }
        Ok(())
    }

    fn replay_step_wise(&mut self) -> Result<()> {
        if self.current_position < self.events.len() {
            let event = &self.events[self.current_position].clone();
            self.process_event(event)?;
            self.current_position += 1;
        }
        Ok(())
    }

    fn replay_filtered(&mut self) -> Result<()> {
        for event in &self.events.clone() {
            if self.has_handler(event) {
                self.process_event(event)?;
            }
        }
        Ok(())
    }

    fn process_event(&mut self, event: &KernelEvent) -> Result<()> {
        let event_type = self.get_event_type_string(&event.event_type);

        match self.event_handlers.get(&event_type) {
            Some(handler) => match handler(event) {
                Ok(_) => {
                    self.stats.events_replayed += 1;
                }
                Err(_) => {
                    self.stats.events_failed += 1;
                }
            },
            None => {
                self.stats.events_replayed += 1;
            }
        }

        Ok(())
    }

    fn has_handler(&self, event: &KernelEvent) -> bool {
        let event_type = self.get_event_type_string(&event.event_type);
        self.event_handlers.contains_key(&event_type)
    }

    fn get_event_type_string(&self, event_type: &EventType) -> String {
        match event_type {
            EventType::MemoryAllocate { .. } => "MemoryAllocate".to_string(),
            EventType::MemoryDeallocate { .. } => "MemoryDeallocate".to_string(),
            EventType::DriverLoad { .. } => "DriverLoad".to_string(),
            EventType::DriverUnload { .. } => "DriverUnload".to_string(),
            EventType::InterruptRaised { .. } => "InterruptRaised".to_string(),
            EventType::InterruptHandled { .. } => "InterruptHandled".to_string(),
            EventType::DeviceDiscovered { .. } => "DeviceDiscovered".to_string(),
            EventType::DeviceRemoved { .. } => "DeviceRemoved".to_string(),
            EventType::CapabilityGranted { .. } => "CapabilityGranted".to_string(),
            EventType::CapabilityRevoked { .. } => "CapabilityRevoked".to_string(),
            EventType::SchedulingDecision { .. } => "SchedulingDecision".to_string(),
            EventType::AnomalyDetected { .. } => "AnomalyDetected".to_string(),
            EventType::CrashRecovery { .. } => "CrashRecovery".to_string(),
            EventType::PerformanceMetric { .. } => "PerformanceMetric".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_engine_creation() {
        let engine = ReplayEngine::new(ReplayMode::FastForward, vec![]);
        assert_eq!(engine.get_total_events(), 0);
    }

    #[test]
    fn test_fast_forward_replay() {
        let events = vec![
            KernelEvent {
                timestamp: 1000,
                sequence: 0,
                event_type: EventType::MemoryAllocate { size: 256 },
                cpu_id: 0,
                context_id: None,
            },
            KernelEvent {
                timestamp: 2000,
                sequence: 1,
                event_type: EventType::MemoryDeallocate { size: 256 },
                cpu_id: 0,
                context_id: None,
            },
        ];

        let mut engine = ReplayEngine::new(ReplayMode::FastForward, events);
        let _ = engine.replay_all();

        assert_eq!(engine.get_stats().events_replayed, 2);
    }

    #[test]
    fn test_step_wise_replay() {
        let events = vec![
            KernelEvent {
                timestamp: 1000,
                sequence: 0,
                event_type: EventType::MemoryAllocate { size: 256 },
                cpu_id: 0,
                context_id: None,
            },
            KernelEvent {
                timestamp: 2000,
                sequence: 1,
                event_type: EventType::MemoryDeallocate { size: 256 },
                cpu_id: 0,
                context_id: None,
            },
        ];

        let mut engine = ReplayEngine::new(ReplayMode::StepWise, events);
        let _ = engine.replay_next();

        assert_eq!(engine.get_position(), 1);
        assert_eq!(engine.get_stats().events_replayed, 1);
    }

    #[test]
    fn test_jump_to_sequence() {
        let events = vec![
            KernelEvent {
                timestamp: 1000,
                sequence: 0,
                event_type: EventType::MemoryAllocate { size: 256 },
                cpu_id: 0,
                context_id: None,
            },
            KernelEvent {
                timestamp: 2000,
                sequence: 5,
                event_type: EventType::MemoryDeallocate { size: 256 },
                cpu_id: 0,
                context_id: None,
            },
        ];

        let mut engine = ReplayEngine::new(ReplayMode::StepWise, events);
        let _ = engine.jump_to(5);

        assert_eq!(engine.get_position(), 1);
    }

    #[test]
    fn test_reset() {
        let events = vec![KernelEvent {
            timestamp: 1000,
            sequence: 0,
            event_type: EventType::MemoryAllocate { size: 256 },
            cpu_id: 0,
            context_id: None,
        }];

        let mut engine = ReplayEngine::new(ReplayMode::StepWise, events);
        let _ = engine.replay_next();
        let _ = engine.reset();

        assert_eq!(engine.get_position(), 0);
    }
}
