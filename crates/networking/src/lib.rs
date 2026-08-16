//! SHER Kernel Networking Subsystem
//!
//! Device/protocol descriptors plus [`stack::NetworkStack`], a real, tested
//! in-process device registry with MTU-checked send/receive counters. There
//! is no real Ethernet/Wi-Fi/Bluetooth/Cellular/RDMA/CAN/TSN I/O — this
//! crate does not own a NIC or socket layer, it only models the bookkeeping
//! a kernel networking subsystem would need above that layer.

pub mod device;
pub mod protocol;
pub mod stack;

pub use device::NetworkDevice;
pub use protocol::NetworkProtocol;
pub use stack::NetworkStack;
