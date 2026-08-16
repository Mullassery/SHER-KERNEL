// SHER Driver Runtime: Network Isolation
// Network access control and bandwidth management for drivers

use serde::{Deserialize, Serialize};
use sher_common::{Error, ObjectId, Result};
use std::collections::HashMap;

// ============================================================================
// NETWORK POLICY
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpProtocol {
    Tcp,
    Udp,
    Icmp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRule {
    pub protocol: IpProtocol,
    pub src_ip: Option<String>,
    pub dst_ip: Option<String>,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
    pub allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    pub driver_id: ObjectId,
    pub allow_network: bool,
    pub rules: Vec<NetworkRule>,
    pub bandwidth_limit_kbps: u32,
    pub max_connections: u32,
}

impl NetworkPolicy {
    pub fn new(driver_id: ObjectId) -> Self {
        NetworkPolicy {
            driver_id,
            allow_network: true,
            rules: Vec::new(),
            bandwidth_limit_kbps: 10000, // 10Mbps default
            max_connections: 32,
        }
    }

    pub fn add_rule(&mut self, rule: NetworkRule) {
        self.rules.push(rule);
    }

    pub fn check_connection(&self, protocol: IpProtocol, _dst: &str) -> Result<bool> {
        if !self.allow_network {
            return Err(Error::AllocationFailed("Network access denied".to_string()));
        }

        // Check rules
        for rule in &self.rules {
            if rule.protocol == protocol && rule.allowed {
                return Ok(true);
            }
        }

        // Default allow
        Ok(true)
    }
}

// ============================================================================
// BANDWIDTH THROTTLER
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub last_measurement_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct BandwidthThrottler {
    pub metrics: HashMap<ObjectId, BandwidthMetrics>,
    pub limits: HashMap<ObjectId, u32>, // kbps
}

impl BandwidthThrottler {
    pub fn new() -> Self {
        BandwidthThrottler::default()
    }

    pub fn set_limit(&mut self, driver_id: ObjectId, kbps: u32) {
        self.limits.insert(driver_id, kbps);
    }

    pub fn get_limit(&self, driver_id: ObjectId) -> Option<u32> {
        self.limits.get(&driver_id).copied()
    }

    pub fn record_traffic(&mut self, driver_id: ObjectId, bytes: u64, is_send: bool) -> Result<()> {
        // Check limit first
        let limit_exceeded = if let Some(limit_kbps) = self.limits.get(&driver_id) {
            if let Some(metrics) = self.metrics.get(&driver_id) {
                let total_bytes = metrics.bytes_sent + metrics.bytes_received + bytes;
                let total_kbytes = total_bytes / 1024;
                total_kbytes as u32 > *limit_kbps
            } else {
                bytes / 1024 > *limit_kbps as u64
            }
        } else {
            false
        };

        if limit_exceeded {
            return Err(Error::AllocationFailed(
                "Bandwidth limit exceeded".to_string(),
            ));
        }

        // Record traffic
        let metrics = self.metrics.entry(driver_id).or_insert(BandwidthMetrics {
            bytes_sent: 0,
            bytes_received: 0,
            packets_sent: 0,
            packets_received: 0,
            last_measurement_ms: 0,
        });

        if is_send {
            metrics.bytes_sent += bytes;
            metrics.packets_sent += 1;
        } else {
            metrics.bytes_received += bytes;
            metrics.packets_received += 1;
        }

        Ok(())
    }

    pub fn get_metrics(&self, driver_id: ObjectId) -> Option<&BandwidthMetrics> {
        self.metrics.get(&driver_id)
    }

    pub fn reset_metrics(&mut self, driver_id: ObjectId) {
        self.metrics.remove(&driver_id);
    }
}

// ============================================================================
// NETWORK ISOLATION MANAGER
// ============================================================================

#[derive(Debug, Clone, Default)]
pub struct NetworkIsolationManager {
    pub policies: HashMap<ObjectId, NetworkPolicy>,
    pub throttler: BandwidthThrottler,
    pub active_connections: HashMap<ObjectId, Vec<String>>,
}

impl NetworkIsolationManager {
    pub fn new() -> Self {
        NetworkIsolationManager {
            policies: HashMap::new(),
            throttler: BandwidthThrottler::new(),
            active_connections: HashMap::new(),
        }
    }

    pub fn register_policy(&mut self, policy: NetworkPolicy) {
        let driver_id = policy.driver_id;
        self.policies.insert(driver_id, policy);
        self.active_connections.insert(driver_id, Vec::new());
    }

    pub fn check_connection(
        &self,
        driver_id: ObjectId,
        protocol: IpProtocol,
        dst: &str,
    ) -> Result<bool> {
        if let Some(policy) = self.policies.get(&driver_id) {
            policy.check_connection(protocol, dst)
        } else {
            Err(Error::AllocationFailed(format!(
                "Driver {} not found",
                driver_id
            )))
        }
    }

    pub fn add_connection(&mut self, driver_id: ObjectId, connection_id: String) -> Result<()> {
        if let Some(policy) = self.policies.get(&driver_id) {
            if let Some(conns) = self.active_connections.get_mut(&driver_id) {
                if conns.len() >= policy.max_connections as usize {
                    return Err(Error::AllocationFailed(
                        "Max connections exceeded".to_string(),
                    ));
                }
                conns.push(connection_id);
                Ok(())
            } else {
                Err(Error::AllocationFailed(
                    "No connection list for driver".to_string(),
                ))
            }
        } else {
            Err(Error::AllocationFailed(format!(
                "Driver {} not found",
                driver_id
            )))
        }
    }

    pub fn remove_connection(&mut self, driver_id: ObjectId, connection_id: &str) {
        if let Some(conns) = self.active_connections.get_mut(&driver_id) {
            conns.retain(|c| c != connection_id);
        }
    }

    pub fn get_connection_count(&self, driver_id: ObjectId) -> usize {
        self.active_connections
            .get(&driver_id)
            .map(|c| c.len())
            .unwrap_or(0)
    }

    pub fn record_traffic(&mut self, driver_id: ObjectId, bytes: u64, is_send: bool) -> Result<()> {
        self.throttler.record_traffic(driver_id, bytes, is_send)
    }

    pub fn get_metrics(&self, driver_id: ObjectId) -> Option<&BandwidthMetrics> {
        self.throttler.get_metrics(driver_id)
    }
}

// ============================================================================
// DEVICE ISOLATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceIsolation {
    pub driver_id: ObjectId,
    pub device_ids: Vec<ObjectId>,
    pub io_ports: Vec<(u16, u16)>, // (start, end) ranges
}

#[derive(Debug, Clone, Default)]
pub struct DeviceIsolationManager {
    pub isolations: HashMap<ObjectId, DeviceIsolation>,
}

impl DeviceIsolationManager {
    pub fn new() -> Self {
        DeviceIsolationManager::default()
    }

    pub fn register_isolation(&mut self, isolation: DeviceIsolation) {
        self.isolations.insert(isolation.driver_id, isolation);
    }

    pub fn can_access_device(&self, driver_id: ObjectId, device_id: ObjectId) -> bool {
        if let Some(isolation) = self.isolations.get(&driver_id) {
            isolation.device_ids.contains(&device_id)
        } else {
            false
        }
    }

    pub fn can_access_io_port(&self, driver_id: ObjectId, port: u16) -> bool {
        if let Some(isolation) = self.isolations.get(&driver_id) {
            isolation
                .io_ports
                .iter()
                .any(|(start, end)| port >= *start && port <= *end)
        } else {
            false
        }
    }

    pub fn check_device_access(&self, driver_id: ObjectId, device_id: ObjectId) -> Result<()> {
        if self.can_access_device(driver_id, device_id) {
            Ok(())
        } else {
            Err(Error::AllocationFailed(format!(
                "Access to device {} denied",
                device_id
            )))
        }
    }

    pub fn check_io_access(&self, driver_id: ObjectId, port: u16) -> Result<()> {
        if self.can_access_io_port(driver_id, port) {
            Ok(())
        } else {
            Err(Error::AllocationFailed(format!(
                "Access to I/O port {} denied",
                port
            )))
        }
    }

    pub fn get_assigned_devices(&self, driver_id: ObjectId) -> Option<&Vec<ObjectId>> {
        self.isolations.get(&driver_id).map(|i| &i.device_ids)
    }
}
