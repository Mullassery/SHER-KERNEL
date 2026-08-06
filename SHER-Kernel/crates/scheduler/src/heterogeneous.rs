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
