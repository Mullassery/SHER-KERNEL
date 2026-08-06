use sher_common::{ObjectId, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DmaBuffer {
    pub id: ObjectId,
    pub physical_addr: u64,
    pub size: u64,
    pub owner: ObjectId,
}

#[derive(Debug, Clone, Default)]
pub struct DmaManager {
    pub buffers: Vec<DmaBuffer>,
}

impl DmaManager {
    pub fn allocate_buffer(&mut self, size: u64, owner: ObjectId) -> Result<ObjectId> {
        let buffer = DmaBuffer {
            id: ObjectId::new(),
            physical_addr: 0x1000 * self.buffers.len() as u64,
            size,
            owner,
        };
        let buffer_id = buffer.id;
        self.buffers.push(buffer);
        Ok(buffer_id)
    }

    pub fn get_buffer(&self, id: ObjectId) -> Option<&DmaBuffer> {
        self.buffers.iter().find(|b| b.id == id)
    }

    pub fn free_buffer(&mut self, id: ObjectId) -> Result<()> {
        self.buffers.retain(|b| b.id != id);
        Ok(())
    }
}
