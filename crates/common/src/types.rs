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
