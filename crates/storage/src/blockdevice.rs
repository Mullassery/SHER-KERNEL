//! `BlockDevice` models a sector-addressable block device with real,
//! bounds-checked read/write semantics — backed by an in-memory buffer
//! (`Vec<u8>`) rather than a real SATA/NVMe/USB/eMMC/SD controller. This is
//! a deliberate simulation: talking to actual storage hardware needs
//! kernel-level (or at minimum root/raw-device) access this userspace crate
//! does not have. What's real here is the sector arithmetic, bounds
//! checking, and read/write/flush behavior a higher layer would depend on.

use serde::{Deserialize, Serialize};
use sher_common::{Error, ObjectId, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockDevice {
    pub id: ObjectId,
    pub sector_size: u64,
    pub sector_count: u64,
    /// In-memory backing store simulating persistent media.
    #[serde(skip)]
    data: Vec<u8>,
    writes: u64,
    reads: u64,
}

impl BlockDevice {
    pub fn new(sector_size: u64, sector_count: u64) -> Self {
        let len = (sector_size * sector_count) as usize;
        Self {
            id: ObjectId::new(),
            sector_size,
            sector_count,
            data: vec![0u8; len],
            writes: 0,
            reads: 0,
        }
    }

    pub fn total_size(&self) -> u64 {
        self.sector_size * self.sector_count
    }

    fn sector_range(&self, sector: u64) -> Result<std::ops::Range<usize>> {
        if sector >= self.sector_count {
            return Err(Error::Storage(format!(
                "sector {sector} out of range (device has {} sectors)",
                self.sector_count
            )));
        }
        let start = (sector * self.sector_size) as usize;
        let end = start + self.sector_size as usize;
        Ok(start..end)
    }

    /// Write `data` into `sector`. `data` must be exactly `sector_size`
    /// bytes.
    pub fn write_sector(&mut self, sector: u64, data: &[u8]) -> Result<()> {
        if data.len() as u64 != self.sector_size {
            return Err(Error::Storage(format!(
                "write of {} bytes does not match sector size {}",
                data.len(),
                self.sector_size
            )));
        }
        let range = self.sector_range(sector)?;
        self.data[range].copy_from_slice(data);
        self.writes += 1;
        Ok(())
    }

    /// Read the contents of `sector`.
    pub fn read_sector(&mut self, sector: u64) -> Result<Vec<u8>> {
        let range = self.sector_range(sector)?;
        self.reads += 1;
        Ok(self.data[range].to_vec())
    }

    pub fn write_count(&self) -> u64 {
        self.writes
    }

    pub fn read_count(&self) -> u64 {
        self.reads
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_device_reads_as_zeroed() {
        let mut dev = BlockDevice::new(512, 4);
        assert_eq!(dev.total_size(), 2048);
        let sector = dev.read_sector(0).unwrap();
        assert_eq!(sector, vec![0u8; 512]);
        assert_eq!(dev.read_count(), 1);
    }

    #[test]
    fn write_then_read_round_trips() {
        let mut dev = BlockDevice::new(4, 2);
        dev.write_sector(1, &[1, 2, 3, 4]).unwrap();
        assert_eq!(dev.read_sector(1).unwrap(), vec![1, 2, 3, 4]);
        // Sector 0 remains untouched.
        assert_eq!(dev.read_sector(0).unwrap(), vec![0, 0, 0, 0]);
        assert_eq!(dev.write_count(), 1);
    }

    #[test]
    fn out_of_range_sector_is_rejected() {
        let mut dev = BlockDevice::new(512, 2);
        assert!(dev.read_sector(2).is_err());
        assert!(dev.write_sector(5, &[0u8; 512]).is_err());
    }

    #[test]
    fn wrong_size_write_is_rejected() {
        let mut dev = BlockDevice::new(512, 2);
        assert!(dev.write_sector(0, &[1, 2, 3]).is_err());
    }
}
