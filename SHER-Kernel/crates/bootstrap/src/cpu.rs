use sher_common::Result;

pub fn initialize() -> Result<()> {
    // CPU bring-up:
    // - ACPI/CPUID detection
    // - BSP vs AP startup
    // - CPU flags (VMX, SVM, etc.)
    // - Cache hierarchy
    Ok(())
}

pub struct CpuInfo {
    pub num_cpus: u32,
    pub features: Vec<String>,
}

pub fn get_info() -> Result<CpuInfo> {
    Ok(CpuInfo {
        num_cpus: 4,
        features: vec!["VMX".to_string(), "RDRAND".to_string()],
    })
}
