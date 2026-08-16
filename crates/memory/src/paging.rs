use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageTable {
    pub entries: HashMap<u64, u64>,
    pub page_size: u64,
}

impl PageTable {
    pub fn new(page_size: u64) -> Self {
        Self {
            entries: HashMap::new(),
            page_size,
        }
    }

    pub fn map(&mut self, virtual_addr: u64, physical_addr: u64) {
        self.entries.insert(virtual_addr, physical_addr);
    }

    pub fn unmap(&mut self, virtual_addr: u64) {
        self.entries.remove(&virtual_addr);
    }

    pub fn translate(&self, virtual_addr: u64) -> Option<u64> {
        self.entries.get(&virtual_addr).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unmapped_address_translates_to_none() {
        let table = PageTable::new(4096);
        assert_eq!(table.translate(0x1000), None);
    }

    #[test]
    fn map_then_translate_round_trips() {
        let mut table = PageTable::new(4096);
        table.map(0x1000, 0x9000);
        assert_eq!(table.translate(0x1000), Some(0x9000));
    }

    #[test]
    fn unmap_removes_mapping() {
        let mut table = PageTable::new(4096);
        table.map(0x1000, 0x9000);
        table.unmap(0x1000);
        assert_eq!(table.translate(0x1000), None);
    }

    #[test]
    fn remapping_overwrites_previous_translation() {
        let mut table = PageTable::new(4096);
        table.map(0x1000, 0x9000);
        table.map(0x1000, 0xA000);
        assert_eq!(table.translate(0x1000), Some(0xA000));
    }
}
