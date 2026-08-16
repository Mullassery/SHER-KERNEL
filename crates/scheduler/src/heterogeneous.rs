use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComputeTarget {
    Cpu,
    Gpu,
    Npu,
    Dsp,
    Fpga,
    Tpu,
    RemoteCluster,
}

impl std::fmt::Display for ComputeTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComputeTarget::Cpu => write!(f, "CPU"),
            ComputeTarget::Gpu => write!(f, "GPU"),
            ComputeTarget::Npu => write!(f, "NPU"),
            ComputeTarget::Dsp => write!(f, "DSP"),
            ComputeTarget::Fpga => write!(f, "FPGA"),
            ComputeTarget::Tpu => write!(f, "TPU"),
            ComputeTarget::RemoteCluster => write!(f, "RemoteCluster"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_matches_expected_labels() {
        assert_eq!(ComputeTarget::Cpu.to_string(), "CPU");
        assert_eq!(ComputeTarget::Gpu.to_string(), "GPU");
        assert_eq!(ComputeTarget::RemoteCluster.to_string(), "RemoteCluster");
    }

    #[test]
    fn targets_are_hashable_and_distinct() {
        use std::collections::HashSet;
        let set: HashSet<ComputeTarget> =
            [ComputeTarget::Cpu, ComputeTarget::Cpu, ComputeTarget::Gpu]
                .into_iter()
                .collect();
        assert_eq!(set.len(), 2);
    }
}
