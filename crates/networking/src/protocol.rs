use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NetworkProtocol {
    Ethernet,
    WiFi,
    Bluetooth,
    Cellular,
    Rdma,
    IndustrialEthernet,
    CanBus,
    TimeSensitiveNetworking,
}

impl std::fmt::Display for NetworkProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkProtocol::Ethernet => write!(f, "Ethernet"),
            NetworkProtocol::WiFi => write!(f, "Wi-Fi"),
            NetworkProtocol::Bluetooth => write!(f, "Bluetooth"),
            NetworkProtocol::Cellular => write!(f, "Cellular"),
            NetworkProtocol::Rdma => write!(f, "RDMA"),
            NetworkProtocol::IndustrialEthernet => write!(f, "Industrial Ethernet"),
            NetworkProtocol::CanBus => write!(f, "CAN Bus"),
            NetworkProtocol::TimeSensitiveNetworking => write!(f, "TSN"),
        }
    }
}
