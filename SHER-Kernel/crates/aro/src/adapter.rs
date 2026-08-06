pub struct RuntimeAdapter;

impl RuntimeAdapter {
    pub fn adapt_to_battery(_charging: bool) {}
    pub fn adapt_to_temperature(_celsius: f64) {}
    pub fn adapt_to_workload(_workload: &str) {}
}
