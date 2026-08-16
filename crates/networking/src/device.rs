use serde::{Deserialize, Serialize};
use sher_common::ObjectId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDevice {
    pub id: ObjectId,
    pub name: String,
    pub mac_address: String,
    pub mtu: u16,
}

impl NetworkDevice {
    pub fn new(name: impl Into<String>, mac_address: impl Into<String>) -> Self {
        Self {
            id: ObjectId::new(),
            name: name.into(),
            mac_address: mac_address.into(),
            mtu: 1500,
        }
    }

    pub fn with_mtu(mut self, mtu: u16) -> Self {
        self.mtu = mtu;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_device_defaults_to_1500_mtu() {
        let dev = NetworkDevice::new("eth0", "00:11:22:33:44:55");
        assert_eq!(dev.mtu, 1500);
        assert_eq!(dev.name, "eth0");
    }

    #[test]
    fn with_mtu_overrides_default() {
        let dev = NetworkDevice::new("eth0", "00:11:22:33:44:55").with_mtu(9000);
        assert_eq!(dev.mtu, 9000);
    }
}
