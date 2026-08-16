//! CPU bring-up (Stage 0).
//!
//! **Simulation notice**: real CPU bring-up (ACPI/CPUID probing, BSP vs AP
//! startup, VMX/SVM flag detection, cache hierarchy discovery) requires
//! ring-0 access this userspace crate does not have — `initialize()` is a
//! no-op placeholder that documents the steps a real bootstrap stage would
//! perform, it does not perform them.
//!
//! `get_info()` is the one piece of this module that reports something
//! real: the number of logical CPUs actually available to this process,
//! via `std::thread::available_parallelism` (a real stdlib query, not a
//! hardcoded constant). The `features` list remains illustrative — reading
//! real CPUID feature flags is out of scope for a userspace crate.

use sher_common::Result;

pub fn initialize() -> Result<()> {
    // CPU bring-up (simulated — see module docs):
    // - ACPI/CPUID detection
    // - BSP vs AP startup
    // - CPU flags (VMX, SVM, etc.)
    // - Cache hierarchy
    Ok(())
}

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub num_cpus: u32,
    /// Illustrative only — not read from real CPUID.
    pub features: Vec<String>,
}

pub fn get_info() -> Result<CpuInfo> {
    let num_cpus = std::thread::available_parallelism()
        .map(|n| n.get() as u32)
        .unwrap_or(1);

    Ok(CpuInfo {
        num_cpus,
        features: vec!["VMX".to_string(), "RDRAND".to_string()],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_never_fails() {
        assert!(initialize().is_ok());
    }

    #[test]
    fn get_info_reports_real_cpu_count() {
        let info = get_info().unwrap();
        assert!(info.num_cpus >= 1);
        assert_eq!(
            info.num_cpus,
            std::thread::available_parallelism()
                .map(|n| n.get() as u32)
                .unwrap_or(1)
        );
    }
}
