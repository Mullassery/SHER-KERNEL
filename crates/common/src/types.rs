use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(Uuid);

impl ObjectId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn nil() -> Self {
        Self(Uuid::nil())
    }
}

impl Default for ObjectId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum ObjectType {
    Process = 1,
    Thread = 2,
    Driver = 3,
    Device = 4,
    Socket = 5,
    StorageVolume = 6,
    Gpu = 7,
    Npu = 8,
    Sensor = 9,
    Robot = 10,
    Container = 11,
    VirtualMachine = 12,
    AiModel = 13,
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectType::Process => write!(f, "Process"),
            ObjectType::Thread => write!(f, "Thread"),
            ObjectType::Driver => write!(f, "Driver"),
            ObjectType::Device => write!(f, "Device"),
            ObjectType::Socket => write!(f, "Socket"),
            ObjectType::StorageVolume => write!(f, "StorageVolume"),
            ObjectType::Gpu => write!(f, "Gpu"),
            ObjectType::Npu => write!(f, "Npu"),
            ObjectType::Sensor => write!(f, "Sensor"),
            ObjectType::Robot => write!(f, "Robot"),
            ObjectType::Container => write!(f, "Container"),
            ObjectType::VirtualMachine => write!(f, "VirtualMachine"),
            ObjectType::AiModel => write!(f, "AiModel"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u32)]
pub enum Capability {
    Read = 1,
    Write = 2,
    Execute = 4,
    Allocate = 8,
    Interrupt = 16,
    Schedule = 32,
    NetworkAccess = 64,
    DmaAccess = 128,
    Admin = 256,
    GpuMemoryAlloc = 512,
    GpuCommandSubmit = 1024,
    GpuAdmin = 2048,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionTier {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn object_id_new_generates_unique_nonzero_ids() {
        let a = ObjectId::new();
        let b = ObjectId::new();
        assert_ne!(a, b);
        assert_ne!(a, ObjectId::nil());
    }

    #[test]
    fn object_id_nil_is_stable() {
        assert_eq!(ObjectId::nil(), ObjectId::nil());
    }

    #[test]
    fn object_id_display_matches_uuid_format() {
        let id = ObjectId::new();
        // UUID string form is 36 chars: 8-4-4-4-12 hex groups.
        assert_eq!(id.to_string().len(), 36);
    }

    #[test]
    fn object_id_default_is_unique_each_call() {
        assert_ne!(ObjectId::default(), ObjectId::default());
    }

    #[test]
    fn object_type_display_is_human_readable() {
        assert_eq!(ObjectType::Gpu.to_string(), "Gpu");
        assert_eq!(ObjectType::VirtualMachine.to_string(), "VirtualMachine");
    }

    #[test]
    fn capability_values_are_distinct_bitflags() {
        let caps = [
            Capability::Read,
            Capability::Write,
            Capability::Execute,
            Capability::Allocate,
            Capability::Interrupt,
            Capability::Schedule,
            Capability::NetworkAccess,
            Capability::DmaAccess,
            Capability::Admin,
            Capability::GpuMemoryAlloc,
            Capability::GpuCommandSubmit,
            Capability::GpuAdmin,
        ];
        let values: HashSet<u32> = caps.iter().map(|c| *c as u32).collect();
        assert_eq!(
            values.len(),
            caps.len(),
            "capability bit values must be distinct"
        );
        for v in values {
            assert!(v.is_power_of_two(), "capability {v} is not a single bit");
        }
    }

    #[test]
    fn permission_tiers_are_hashable_and_distinct() {
        let set: HashSet<PermissionTier> = [
            PermissionTier::Low,
            PermissionTier::Medium,
            PermissionTier::High,
            PermissionTier::Critical,
        ]
        .into_iter()
        .collect();
        assert_eq!(set.len(), 4);
    }
}
