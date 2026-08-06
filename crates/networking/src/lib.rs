//! SHER Kernel Networking Subsystem
//!
//! Support for Ethernet, Wi-Fi, Bluetooth, Cellular, RDMA, Industrial Ethernet, CAN Bus, TSN

pub mod device;
pub mod protocol;

pub use device::NetworkDevice;
pub use protocol::NetworkProtocol;
