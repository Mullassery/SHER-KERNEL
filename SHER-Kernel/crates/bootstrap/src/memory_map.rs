use sher_common::Result;

pub struct MemoryRegion {
    pub base: u64,
    pub size: u64,
    pub region_type: String,
}

pub fn discover() -> Result<Vec<MemoryRegion>> {
    Ok(vec![
        MemoryRegion {
            base: 0x0,
            size: 0x100000,
            region_type: "BIOS".to_string(),
        },
        MemoryRegion {
            base: 0x100000,
            size: 8 * 1024 * 1024 * 1024, // 8GB
            region_type: "RAM".to_string(),
        },
    ])
}
