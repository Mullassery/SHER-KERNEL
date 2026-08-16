use serde::{Deserialize, Serialize};
use sher_common::{Capability, PermissionTier};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grant_is_valid_and_has_future_expiry() {
        let grant = CapabilityGrant::new(Capability::Read, PermissionTier::Low);
        assert!(grant.is_valid());
        assert!(grant.expires_at.unwrap() > grant.granted_at);
    }

    #[test]
    fn expired_grant_is_invalid() {
        let mut grant = CapabilityGrant::new(Capability::Write, PermissionTier::Critical);
        grant.expires_at = Some(0); // far in the past
        assert!(!grant.is_valid());
    }

    #[test]
    fn tier_durations_are_time_bounded_not_permanent() {
        for tier in [
            PermissionTier::Low,
            PermissionTier::Medium,
            PermissionTier::High,
            PermissionTier::Critical,
        ] {
            let grant = CapabilityGrant::new(Capability::Admin, tier);
            assert!(
                grant.expires_at.is_some(),
                "capability grants must always expire, tier {:?} did not",
                tier
            );
        }
    }

    #[test]
    fn capability_set_grant_and_revoke() {
        let mut set = CapabilitySet::default();
        assert!(!set.has_capability(Capability::NetworkAccess));

        set.grant(Capability::NetworkAccess, PermissionTier::Medium);
        assert!(set.has_capability(Capability::NetworkAccess));
        assert!(!set.has_capability(Capability::Admin));

        set.revoke(Capability::NetworkAccess);
        assert!(!set.has_capability(Capability::NetworkAccess));
    }

    #[test]
    fn cleanup_expired_removes_only_expired_grants() {
        let mut set = CapabilitySet::default();
        set.grant(Capability::Read, PermissionTier::Low);
        set.grants.push(CapabilityGrant {
            capability: Capability::Write,
            tier: PermissionTier::Low,
            granted_at: 0,
            expires_at: Some(1),
        });

        assert_eq!(set.grants.len(), 2);
        set.cleanup_expired();
        assert_eq!(set.grants.len(), 1);
        assert!(set.has_capability(Capability::Read));
        assert!(!set.has_capability(Capability::Write));
    }
}
