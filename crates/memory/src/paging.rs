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
