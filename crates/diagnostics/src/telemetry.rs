//! System-wide diagnostics telemetry collection.
//!
//! Distinct from `sher_objectmodel::Telemetry` (which tracks per-object
//! health), this collector aggregates named counters and gauges across the
//! whole running kernel process for reporting/debugging. It is an in-memory
//! simulation: there is no persistent store or real crash-analytics pipeline
//! behind it, matching the "Later: Persistent logs, telemetry, crash
//! analytics, profiling" note in the crate root docs.

use std::collections::HashMap;

/// A single point-in-time telemetry sample recorded into the ring buffer
/// alongside counters.
#[derive(Debug, Clone, PartialEq)]
pub struct TelemetryEvent {
    pub name: String,
    pub value: f64,
}

#[derive(Debug, Default)]
pub struct Telemetry {
    counters: HashMap<String, u64>,
    gauges: HashMap<String, f64>,
    events: Vec<TelemetryEvent>,
}

impl Telemetry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Monotonically increase a named counter by `delta`.
    pub fn increment(&mut self, name: &str, delta: u64) {
        *self.counters.entry(name.to_string()).or_insert(0) += delta;
    }

    pub fn counter(&self, name: &str) -> u64 {
        self.counters.get(name).copied().unwrap_or(0)
    }

    /// Set a named gauge (point-in-time value, can go up or down).
    pub fn set_gauge(&mut self, name: &str, value: f64) {
        self.gauges.insert(name.to_string(), value);
    }

    pub fn gauge(&self, name: &str) -> Option<f64> {
        self.gauges.get(name).copied()
    }

    /// Record a named, valued event (e.g. a latency sample).
    pub fn record_event(&mut self, name: impl Into<String>, value: f64) {
        self.events.push(TelemetryEvent {
            name: name.into(),
            value,
        });
    }

    pub fn events(&self) -> &[TelemetryEvent] {
        &self.events
    }

    /// Average of all recorded event values with the given name.
    pub fn average(&self, name: &str) -> Option<f64> {
        let matching: Vec<f64> = self
            .events
            .iter()
            .filter(|e| e.name == name)
            .map(|e| e.value)
            .collect();
        if matching.is_empty() {
            return None;
        }
        Some(matching.iter().sum::<f64>() / matching.len() as f64)
    }

    pub fn counters_snapshot(&self) -> HashMap<String, u64> {
        self.counters.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_accumulate() {
        let mut t = Telemetry::new();
        t.increment("driver_restarts", 1);
        t.increment("driver_restarts", 2);
        assert_eq!(t.counter("driver_restarts"), 3);
        assert_eq!(t.counter("unknown"), 0);
    }

    #[test]
    fn gauges_overwrite() {
        let mut t = Telemetry::new();
        t.set_gauge("memory_pressure", 0.2);
        t.set_gauge("memory_pressure", 0.7);
        assert_eq!(t.gauge("memory_pressure"), Some(0.7));
        assert_eq!(t.gauge("missing"), None);
    }

    #[test]
    fn events_recorded_and_averaged() {
        let mut t = Telemetry::new();
        t.record_event("latency_ms", 10.0);
        t.record_event("latency_ms", 20.0);
        t.record_event("other", 5.0);
        assert_eq!(t.events().len(), 3);
        assert_eq!(t.average("latency_ms"), Some(15.0));
        assert_eq!(t.average("nonexistent"), None);
    }

    #[test]
    fn counters_snapshot_is_independent_copy() {
        let mut t = Telemetry::new();
        t.increment("a", 5);
        let snap = t.counters_snapshot();
        t.increment("a", 5);
        assert_eq!(snap.get("a"), Some(&5));
        assert_eq!(t.counter("a"), 10);
    }
}
