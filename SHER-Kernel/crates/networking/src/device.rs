use sher_common::ObjectId;
use serde::{Deserialize, Serialize};

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
}
