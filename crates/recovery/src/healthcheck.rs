//! Boot-test health checks: the "Boot test System B" step in the update
//! sequence. A set of named probes are run against a freshly-staged
//! partition before the boot pointer is switched to it; any failing probe
//! aborts the update and the previous partition remains active.

use crate::partition::ImmutablePartition;

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeResult {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Default)]
pub struct HealthCheckReport {
    pub results: Vec<ProbeResult>,
}

impl HealthCheckReport {
    pub fn all_passed(&self) -> bool {
        !self.results.is_empty() && self.results.iter().all(|r| r.passed)
    }

    pub fn failures(&self) -> Vec<&ProbeResult> {
        self.results.iter().filter(|r| !r.passed).collect()
    }
}

/// A single named health probe. Real probes would exercise the staged
/// system (spawn it, check it responds); here a probe is any function of
/// the partition to a pass/fail + detail string, so callers can register
/// meaningful checks (e.g. "does the image parse", "is the version newer")
/// without this crate needing real boot capability.
pub type Probe = (&'static str, fn(&ImmutablePartition) -> (bool, String));

pub const DEFAULT_PROBES: &[Probe] = &[
    ("integrity", probe_integrity),
    ("not_empty", probe_not_empty),
];

fn probe_integrity(partition: &ImmutablePartition) -> (bool, String) {
    if partition.verify() {
        (true, "checksum matches".to_string())
    } else {
        (false, "checksum mismatch or unwritten image".to_string())
    }
}

fn probe_not_empty(partition: &ImmutablePartition) -> (bool, String) {
    if partition.image_len() > 0 {
        (true, format!("{} bytes staged", partition.image_len()))
    } else {
        (false, "no image staged".to_string())
    }
}

/// Run every probe in `probes` against `partition` and collect the report.
pub fn check(partition: &ImmutablePartition, probes: &[Probe]) -> HealthCheckReport {
    let results = probes
        .iter()
        .map(|(name, probe_fn)| {
            let (passed, detail) = probe_fn(partition);
            ProbeResult {
                name: name.to_string(),
                passed,
                detail,
            }
        })
        .collect();
    HealthCheckReport { results }
}

/// Convenience: run the default probe set.
pub fn check_default(partition: &ImmutablePartition) -> HealthCheckReport {
    check(partition, DEFAULT_PROBES)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::partition::PartitionSlot;

    #[test]
    fn empty_partition_fails_health_check() {
        let partition = ImmutablePartition::empty(PartitionSlot::B);
        let report = check_default(&partition);
        assert!(!report.all_passed());
        assert_eq!(report.failures().len(), 2);
    }

    #[test]
    fn staged_partition_passes_health_check() {
        let mut partition = ImmutablePartition::empty(PartitionSlot::B);
        partition.write_image("2.0.0", vec![1, 2, 3]);
        let report = check_default(&partition);
        assert!(report.all_passed());
        assert!(report.failures().is_empty());
    }
}
