// SHER LKI: Security & Capability System
// Time-bounded capability grants with zero-trust enforcement

use serde::{Deserialize, Serialize};
use sher_common::{Error, ObjectId, Result};
use std::collections::HashMap;

// ============================================================================
// CAPABILITY TYPES & PERMISSIONS
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    // Memory capabilities
    AllocateMemory,
    DeallocateMemory,
    ReadMemory,
    WriteMemory,
    DmaAccess,

    // Interrupt capabilities
    RegisterInterrupt,
    UnregisterInterrupt,
    EnableInterrupt,
    DisableInterrupt,

    // Device capabilities
    RegisterDevice,
    UnregisterDevice,
    EnableDevice,
    DisableDevice,
    ProbeDevices,

    // Network capabilities
    NetworkAccess,
    BindSocket,
    ListenSocket,
    SendPacket,
    ReceivePacket,

    // Storage capabilities
    BlockRead,
    BlockWrite,
    DirectDma,

    // Admin capabilities
    ModifyPolicy,
    AccessAuditLog,
    TerminateDriver,
    EmergencyShutdown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionTier {
    Low,      // Configurable (default 1h)
    Medium,   // 24 hours max
    High,     // 2 hours max
    Critical, // 30 minutes max
}

impl PermissionTier {
    pub fn max_duration_ms(&self) -> u64 {
        match self {
            PermissionTier::Low => 3_600_000,      // 1 hour
            PermissionTier::Medium => 86_400_000,  // 24 hours
            PermissionTier::High => 7_200_000,     // 2 hours
            PermissionTier::Critical => 1_800_000, // 30 minutes
        }
    }

    pub fn recommended_duration_ms(&self) -> u64 {
        match self {
            PermissionTier::Low => 3_600_000,
            PermissionTier::Medium => 3_600_000,
            PermissionTier::High => 1_800_000,
            PermissionTier::Critical => 900_000,
        }
    }
}

// ============================================================================
// CAPABILITY GRANT
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityGrant {
    pub grant_id: ObjectId,
    pub driver_id: ObjectId,
    pub capability: Capability,
    pub tier: PermissionTier,
    pub granted_at_ms: u64,
    pub expires_at_ms: u64,
    pub granted_by: String,
    pub reason: String,
    pub reauth_required: bool,
    pub reauth_method: ReauthMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReauthMethod {
    None,
    Click,
    Pin(u32),
    Password,
    Biometric,
    SecurityKey,
}

impl CapabilityGrant {
    pub fn new(driver_id: ObjectId, capability: Capability, tier: PermissionTier) -> Self {
        let duration = tier.recommended_duration_ms();
        CapabilityGrant {
            grant_id: ObjectId::new(),
            driver_id,
            capability,
            tier,
            granted_at_ms: 0,
            expires_at_ms: duration,
            granted_by: "system".to_string(),
            reason: "Initial grant".to_string(),
            reauth_required: matches!(tier, PermissionTier::High | PermissionTier::Critical),
            reauth_method: ReauthMethod::None,
        }
    }

    pub fn with_duration(mut self, duration_ms: u64) -> Result<Self> {
        let max = self.tier.max_duration_ms();
        if duration_ms > max {
            return Err(Error::Driver(format!(
                "Duration exceeds tier limit of {}ms",
                max
            )));
        }
        self.expires_at_ms = self.granted_at_ms + duration_ms;
        Ok(self)
    }

    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = reason.to_string();
        self
    }

    pub fn with_reauth(mut self, method: ReauthMethod) -> Self {
        self.reauth_required = true;
        self.reauth_method = method;
        self
    }

    pub fn is_valid(&self, current_time_ms: u64) -> bool {
        current_time_ms < self.expires_at_ms
    }

    pub fn is_expired(&self, current_time_ms: u64) -> bool {
        current_time_ms >= self.expires_at_ms
    }

    pub fn time_remaining_ms(&self, current_time_ms: u64) -> u64 {
        self.expires_at_ms.saturating_sub(current_time_ms)
    }

    pub fn lifetime_ms(&self) -> u64 {
        self.expires_at_ms - self.granted_at_ms
    }
}

// ============================================================================
// CAPABILITY MANAGER
// ============================================================================

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityManager {
    pub grants: HashMap<ObjectId, Vec<CapabilityGrant>>, // driver_id -> grants
    pub total_grants: u64,
    pub expired_grants: u64,
    pub revoked_grants: u64,
    pub reauth_requests: u64,
}

impl CapabilityManager {
    pub fn new() -> Self {
        CapabilityManager::default()
    }

    /// Grant capability to driver
    pub fn grant(&mut self, grant: CapabilityGrant) -> Result<ObjectId> {
        // Validate tier duration
        let max = grant.tier.max_duration_ms();
        if grant.lifetime_ms() > max {
            return Err(Error::Driver(
                "Capability lifetime exceeds tier limit".to_string(),
            ));
        }

        let grant_id = grant.grant_id;
        self.grants.entry(grant.driver_id).or_default().push(grant);
        self.total_grants += 1;

        Ok(grant_id)
    }

    /// Revoke capability
    pub fn revoke(&mut self, driver_id: ObjectId, capability: Capability) -> Result<()> {
        if let Some(grants) = self.grants.get_mut(&driver_id) {
            grants.retain(|g| g.capability != capability);
            self.revoked_grants += 1;
            Ok(())
        } else {
            Err(Error::Driver("Driver not found".to_string()))
        }
    }

    /// Check if driver has capability (with expiry check)
    pub fn has_capability(
        &mut self,
        driver_id: ObjectId,
        capability: Capability,
        current_time_ms: u64,
    ) -> bool {
        if let Some(grants) = self.grants.get_mut(&driver_id) {
            // Remove expired grants
            grants.retain(|g| !g.is_expired(current_time_ms));

            // Check for valid grant
            grants
                .iter()
                .any(|g| g.capability == capability && g.is_valid(current_time_ms))
        } else {
            false
        }
    }

    /// Check capability and require reauthentication if needed
    pub fn check_with_reauth(
        &mut self,
        driver_id: ObjectId,
        capability: Capability,
        current_time_ms: u64,
    ) -> Result<bool> {
        if let Some(grants) = self.grants.get_mut(&driver_id) {
            // Remove expired grants
            grants.retain(|g| !g.is_expired(current_time_ms));

            // Find grant
            if let Some(grant) = grants
                .iter()
                .find(|g| g.capability == capability && g.is_valid(current_time_ms))
            {
                if grant.reauth_required {
                    self.reauth_requests += 1;
                    return Ok(true); // Reauthentication required
                }
                return Ok(false); // No reauthentication needed
            }
        }

        Err(Error::Driver("Capability not granted".to_string()))
    }

    /// Auto-expire old grants (cleanup)
    pub fn cleanup_expired(&mut self, current_time_ms: u64) {
        for grants in self.grants.values_mut() {
            let initial_count = grants.len();
            grants.retain(|g| !g.is_expired(current_time_ms));
            self.expired_grants += (initial_count - grants.len()) as u64;
        }
    }

    /// Get active grants for driver
    pub fn get_active_grants(
        &self,
        driver_id: ObjectId,
        current_time_ms: u64,
    ) -> Vec<&CapabilityGrant> {
        if let Some(grants) = self.grants.get(&driver_id) {
            grants
                .iter()
                .filter(|g| g.is_valid(current_time_ms))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get grant by ID
    pub fn get_grant(&self, grant_id: ObjectId) -> Option<&CapabilityGrant> {
        self.grants
            .values()
            .flat_map(|grants| grants.iter())
            .find(|g| g.grant_id == grant_id)
    }

    /// Get expiring grants (< threshold_ms)
    pub fn get_expiring_grants(
        &self,
        current_time_ms: u64,
        threshold_ms: u64,
    ) -> Vec<&CapabilityGrant> {
        self.grants
            .values()
            .flat_map(|grants| grants.iter())
            .filter(|g| {
                let remaining = g.time_remaining_ms(current_time_ms);
                remaining > 0 && remaining < threshold_ms
            })
            .collect()
    }

    /// Get capability statistics
    pub fn get_stats(&self, current_time_ms: u64) -> SecurityStats {
        let mut active_count = 0;

        for grants in self.grants.values() {
            for grant in grants {
                if grant.is_valid(current_time_ms) {
                    active_count += 1;
                }
            }
        }

        SecurityStats {
            total_grants: self.total_grants,
            active_grants: active_count,
            expired_grants: self.expired_grants,
            revoked_grants: self.revoked_grants,
            reauth_requests: self.reauth_requests,
            drivers_with_caps: self.grants.len() as u64,
        }
    }
}

// ============================================================================
// SECURITY POLICY
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub policy_id: ObjectId,
    pub driver_id: ObjectId,
    pub security_level: SecurityLevel,
    pub max_capability_tier: PermissionTier,
    pub require_reauth_for_critical: bool,
    pub auto_revoke_on_error: bool,
    pub audit_all_operations: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    Unrestricted, // No policy (dangerous)
    Permissive,   // Allow by default, audit denials
    Balanced,     // Allow known operations, deny unknown
    Strict,       // Deny by default, explicit allowlist
    Critical,     // Maximum restrictions
}

impl SecurityPolicy {
    pub fn new(driver_id: ObjectId, level: SecurityLevel) -> Self {
        SecurityPolicy {
            policy_id: ObjectId::new(),
            driver_id,
            security_level: level,
            max_capability_tier: match level {
                SecurityLevel::Unrestricted => PermissionTier::Critical,
                SecurityLevel::Permissive => PermissionTier::Critical,
                SecurityLevel::Balanced => PermissionTier::High,
                SecurityLevel::Strict => PermissionTier::Medium,
                SecurityLevel::Critical => PermissionTier::Low,
            },
            require_reauth_for_critical: matches!(
                level,
                SecurityLevel::Strict | SecurityLevel::Critical
            ),
            auto_revoke_on_error: matches!(level, SecurityLevel::Critical),
            audit_all_operations: matches!(level, SecurityLevel::Strict | SecurityLevel::Critical),
        }
    }

    pub fn allows_tier(&self, tier: PermissionTier) -> bool {
        (tier as u32) <= (self.max_capability_tier as u32)
    }
}

// ============================================================================
// SECURITY STATISTICS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStats {
    pub total_grants: u64,
    pub active_grants: u64,
    pub expired_grants: u64,
    pub revoked_grants: u64,
    pub reauth_requests: u64,
    pub drivers_with_caps: u64,
}
