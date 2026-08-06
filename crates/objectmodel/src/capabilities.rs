use sher_common::{Capability, PermissionTier};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityGrant {
    pub capability: Capability,
    pub tier: PermissionTier,
    pub granted_at: u64,
    pub expires_at: Option<u64>,
}

impl CapabilityGrant {
    pub fn new(capability: Capability, tier: PermissionTier) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let expires_at = match tier {
            PermissionTier::Low => Some(now + 3600),
            PermissionTier::Medium => Some(now + 86400),
            PermissionTier::High => Some(now + 7200),
            PermissionTier::Critical => Some(now + 1800),
        };

        Self {
            capability,
            tier,
            granted_at: now,
            expires_at,
        }
    }

    pub fn is_valid(&self) -> bool {
        match self.expires_at {
            Some(expiry) => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                now < expiry
            }
            None => true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilitySet {
    pub grants: Vec<CapabilityGrant>,
}

impl CapabilitySet {
    pub fn grant(&mut self, capability: Capability, tier: PermissionTier) {
        self.grants.push(CapabilityGrant::new(capability, tier));
    }

    pub fn has_capability(&self, capability: Capability) -> bool {
        self.grants
            .iter()
            .any(|g| g.capability == capability && g.is_valid())
    }

    pub fn revoke(&mut self, capability: Capability) {
        self.grants.retain(|g| g.capability != capability);
    }

    pub fn cleanup_expired(&mut self) {
        self.grants.retain(|g| g.is_valid());
    }
}
