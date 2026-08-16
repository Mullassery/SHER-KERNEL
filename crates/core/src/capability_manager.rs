//! Capability manager: grant, revoke, and enforce time-bounded capabilities
//! per object, on top of [`sher_objectmodel::capabilities::CapabilitySet`].

use sher_common::{Capability, ObjectId, PermissionTier, Result};
use sher_objectmodel::capabilities::CapabilitySet;
use std::collections::HashMap;

#[derive(Default)]
pub struct CapabilityManager {
    grants: HashMap<ObjectId, CapabilitySet>,
}

impl CapabilityManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn grant(&mut self, owner: ObjectId, capability: Capability, tier: PermissionTier) {
        self.grants
            .entry(owner)
            .or_default()
            .grant(capability, tier);
    }

    pub fn revoke(&mut self, owner: ObjectId, capability: Capability) {
        if let Some(set) = self.grants.get_mut(&owner) {
            set.revoke(capability);
        }
    }

    /// Enforce: returns `Ok(())` if `owner` currently holds a valid,
    /// unexpired grant for `capability`; an error otherwise. This is the
    /// "no component has unrestricted access" check point.
    pub fn enforce(&self, owner: ObjectId, capability: Capability) -> Result<()> {
        let allowed = self
            .grants
            .get(&owner)
            .map(|set| set.has_capability(capability))
            .unwrap_or(false);
        if allowed {
            Ok(())
        } else {
            Err(sher_common::Error::Security(format!(
                "object {owner} lacks capability {capability:?}"
            )))
        }
    }

    /// Drop any grants across all owners whose expiration has passed.
    pub fn cleanup_expired(&mut self) {
        for set in self.grants.values_mut() {
            set.cleanup_expired();
        }
    }
}

pub fn initialize() -> Result<CapabilityManager> {
    Ok(CapabilityManager::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enforce_denies_by_default() {
        let manager = CapabilityManager::new();
        assert!(manager.enforce(ObjectId::new(), Capability::Read).is_err());
    }

    #[test]
    fn grant_then_enforce_succeeds() {
        let mut manager = CapabilityManager::new();
        let owner = ObjectId::new();
        manager.grant(owner, Capability::Read, PermissionTier::Medium);
        assert!(manager.enforce(owner, Capability::Read).is_ok());
        assert!(manager.enforce(owner, Capability::Write).is_err());
    }

    #[test]
    fn revoke_removes_access() {
        let mut manager = CapabilityManager::new();
        let owner = ObjectId::new();
        manager.grant(owner, Capability::Admin, PermissionTier::Low);
        manager.revoke(owner, Capability::Admin);
        assert!(manager.enforce(owner, Capability::Admin).is_err());
    }
}
