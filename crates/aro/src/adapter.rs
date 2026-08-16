//! Runtime adaptation policy: given battery/thermal/workload signals,
//! decide which optional features should be scaled back. Real decision
//! logic, but the *inputs* (battery state, temperature) must be supplied by
//! a caller with actual sensor access — this crate has none, so it never
//! invents readings.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThermalState {
    Normal,
    Warm,
    Throttled,
}

impl ThermalState {
    fn from_celsius(celsius: f64) -> Self {
        if celsius >= 90.0 {
            ThermalState::Throttled
        } else if celsius >= 75.0 {
            ThermalState::Warm
        } else {
            ThermalState::Normal
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdaptationDecision {
    pub reduce_background_work: bool,
    pub disable_predictive_loading: bool,
    pub reason: String,
}

#[derive(Debug, Default)]
pub struct RuntimeAdapter {
    on_battery: bool,
    thermal: Option<ThermalState>,
    workload: Option<String>,
}

impl RuntimeAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    /// `charging = false` means running on battery power.
    pub fn adapt_to_battery(&mut self, charging: bool) -> AdaptationDecision {
        self.on_battery = !charging;
        if self.on_battery {
            AdaptationDecision {
                reduce_background_work: true,
                disable_predictive_loading: true,
                reason: "on battery power".to_string(),
            }
        } else {
            AdaptationDecision {
                reduce_background_work: false,
                disable_predictive_loading: false,
                reason: "on external power".to_string(),
            }
        }
    }

    pub fn adapt_to_temperature(&mut self, celsius: f64) -> AdaptationDecision {
        let state = ThermalState::from_celsius(celsius);
        self.thermal = Some(state);
        match state {
            ThermalState::Throttled => AdaptationDecision {
                reduce_background_work: true,
                disable_predictive_loading: true,
                reason: format!("thermal throttled at {celsius:.1}C"),
            },
            ThermalState::Warm => AdaptationDecision {
                reduce_background_work: true,
                disable_predictive_loading: false,
                reason: format!("running warm at {celsius:.1}C"),
            },
            ThermalState::Normal => AdaptationDecision {
                reduce_background_work: false,
                disable_predictive_loading: false,
                reason: format!("thermal normal at {celsius:.1}C"),
            },
        }
    }

    pub fn adapt_to_workload(&mut self, workload: &str) {
        self.workload = Some(workload.to_string());
    }

    /// Combine the most recent battery + thermal signals into one
    /// decision. Battery and thermal pressure both independently justify
    /// scaling back; either being true is sufficient.
    pub fn current_decision(&self) -> AdaptationDecision {
        let battery_pressure = self.on_battery;
        let thermal_pressure = matches!(
            self.thermal,
            Some(ThermalState::Warm) | Some(ThermalState::Throttled)
        );

        AdaptationDecision {
            reduce_background_work: battery_pressure || thermal_pressure,
            disable_predictive_loading: battery_pressure
                || matches!(self.thermal, Some(ThermalState::Throttled)),
            reason: format!(
                "battery_pressure={battery_pressure}, thermal={:?}",
                self.thermal
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn charging_relaxes_constraints() {
        let mut adapter = RuntimeAdapter::new();
        let decision = adapter.adapt_to_battery(true);
        assert!(!decision.reduce_background_work);
    }

    #[test]
    fn on_battery_reduces_background_work() {
        let mut adapter = RuntimeAdapter::new();
        let decision = adapter.adapt_to_battery(false);
        assert!(decision.reduce_background_work);
        assert!(decision.disable_predictive_loading);
    }

    #[test]
    fn high_temperature_throttles() {
        let mut adapter = RuntimeAdapter::new();
        let decision = adapter.adapt_to_temperature(95.0);
        assert!(decision.reduce_background_work);
        assert!(decision.disable_predictive_loading);
    }

    #[test]
    fn normal_temperature_no_reduction() {
        let mut adapter = RuntimeAdapter::new();
        let decision = adapter.adapt_to_temperature(45.0);
        assert!(!decision.reduce_background_work);
    }

    #[test]
    fn current_decision_combines_signals() {
        let mut adapter = RuntimeAdapter::new();
        adapter.adapt_to_battery(true); // charging, no pressure
        adapter.adapt_to_temperature(80.0); // warm
        let decision = adapter.current_decision();
        assert!(decision.reduce_background_work); // thermal pressure alone triggers it
        assert!(!decision.disable_predictive_loading); // warm (not throttled) + not on battery
    }
}
