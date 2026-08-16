//! In-process network stack simulation: device registry plus send/receive
//! packet counters. There is no real NIC/socket I/O here — see the crate
//! root docs; that requires privileged access to actual network hardware or
//! the OS socket layer this crate deliberately does not attempt to own
//! (SHER-Display/other subsystems own real I/O where it exists).

use crate::device::NetworkDevice;
use crate::protocol::NetworkProtocol;
use sher_common::{Error, ObjectId, Result};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
struct DeviceStats {
    packets_sent: u64,
    packets_received: u64,
    bytes_sent: u64,
    bytes_received: u64,
}

#[derive(Default)]
pub struct NetworkStack {
    devices: HashMap<ObjectId, (NetworkDevice, DeviceStats)>,
}

impl NetworkStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, device: NetworkDevice) -> ObjectId {
        let id = device.id;
        self.devices.insert(id, (device, DeviceStats::default()));
        id
    }

    pub fn unregister(&mut self, id: ObjectId) -> bool {
        self.devices.remove(&id).is_some()
    }

    pub fn device(&self, id: ObjectId) -> Option<&NetworkDevice> {
        self.devices.get(&id).map(|(d, _)| d)
    }

    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Simulate sending a packet: validates the payload fits the device
    /// MTU, then updates counters. No bytes leave the process.
    pub fn send(&mut self, id: ObjectId, protocol: NetworkProtocol, payload: &[u8]) -> Result<()> {
        let (device, stats) = self
            .devices
            .get_mut(&id)
            .ok_or_else(|| Error::Networking(format!("unknown device: {id}")))?;

        if payload.len() > device.mtu as usize {
            return Err(Error::Networking(format!(
                "{protocol} payload of {} bytes exceeds MTU {} on device '{}'",
                payload.len(),
                device.mtu,
                device.name
            )));
        }

        stats.packets_sent += 1;
        stats.bytes_sent += payload.len() as u64;
        Ok(())
    }

    /// Simulate receiving a packet on `id` (e.g. driven by a test harness
    /// or a loopback pairing), updating receive counters.
    pub fn receive(&mut self, id: ObjectId, payload: &[u8]) -> Result<()> {
        let (_, stats) = self
            .devices
            .get_mut(&id)
            .ok_or_else(|| Error::Networking(format!("unknown device: {id}")))?;
        stats.packets_received += 1;
        stats.bytes_received += payload.len() as u64;
        Ok(())
    }

    pub fn packets_sent(&self, id: ObjectId) -> u64 {
        self.devices
            .get(&id)
            .map(|(_, s)| s.packets_sent)
            .unwrap_or(0)
    }

    pub fn packets_received(&self, id: ObjectId) -> u64 {
        self.devices
            .get(&id)
            .map(|(_, s)| s.packets_received)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_lookup() {
        let mut stack = NetworkStack::new();
        let id = stack.register(NetworkDevice::new("eth0", "00:11:22:33:44:55"));
        assert!(stack.device(id).is_some());
        assert_eq!(stack.device_count(), 1);
    }

    #[test]
    fn send_within_mtu_updates_counters() {
        let mut stack = NetworkStack::new();
        let id = stack.register(NetworkDevice::new("eth0", "00:11:22:33:44:55"));
        stack
            .send(id, NetworkProtocol::Ethernet, &[0u8; 100])
            .unwrap();
        assert_eq!(stack.packets_sent(id), 1);
    }

    #[test]
    fn send_over_mtu_is_rejected() {
        let mut stack = NetworkStack::new();
        let id = stack.register(NetworkDevice::new("eth0", "00:11:22:33:44:55"));
        let oversized = vec![0u8; 2000]; // default MTU is 1500
        assert!(stack
            .send(id, NetworkProtocol::Ethernet, &oversized)
            .is_err());
        assert_eq!(stack.packets_sent(id), 0);
    }

    #[test]
    fn send_to_unknown_device_errors() {
        let mut stack = NetworkStack::new();
        assert!(stack
            .send(ObjectId::new(), NetworkProtocol::WiFi, &[1])
            .is_err());
    }

    #[test]
    fn unregister_removes_device() {
        let mut stack = NetworkStack::new();
        let id = stack.register(NetworkDevice::new("wlan0", "aa:bb:cc:dd:ee:ff"));
        assert!(stack.unregister(id));
        assert!(stack.device(id).is_none());
    }

    #[test]
    fn receive_updates_counters() {
        let mut stack = NetworkStack::new();
        let id = stack.register(NetworkDevice::new("eth0", "00:11:22:33:44:55"));
        stack.receive(id, &[1, 2, 3]).unwrap();
        assert_eq!(stack.packets_received(id), 1);
    }
}
