use serde::{Deserialize, Serialize};
use sher_common::{ObjectId, Result};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_buffer_is_retrievable() {
        let mut mgr = DmaManager::default();
        let owner = ObjectId::new();
        let id = mgr.allocate_buffer(4096, owner).unwrap();
        let buf = mgr.get_buffer(id).unwrap();
        assert_eq!(buf.size, 4096);
        assert_eq!(buf.owner, owner);
    }

    #[test]
    fn buffers_get_distinct_physical_addresses() {
        let mut mgr = DmaManager::default();
        let owner = ObjectId::new();
        let a = mgr.allocate_buffer(100, owner).unwrap();
        let b = mgr.allocate_buffer(100, owner).unwrap();
        assert_ne!(
            mgr.get_buffer(a).unwrap().physical_addr,
            mgr.get_buffer(b).unwrap().physical_addr
        );
    }

    #[test]
    fn free_buffer_removes_it() {
        let mut mgr = DmaManager::default();
        let id = mgr.allocate_buffer(64, ObjectId::new()).unwrap();
        mgr.free_buffer(id).unwrap();
        assert!(mgr.get_buffer(id).is_none());
    }
}
