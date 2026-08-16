//! Versioned snapshot store: multiple versions of a component coexist, and
//! switching the "active" version is an O(1) pointer change, not a
//! reinstall — matching the crate's design note that rollback is instant.

use crate::version::Snapshot;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct SnapshotStore {
    snapshots: HashMap<String, Vec<Snapshot>>,
    active_version: HashMap<String, u32>,
}

impl SnapshotStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Capture a new snapshot for `component` and make it the active
    /// version. Older versions are retained (coexistence), not deleted.
    pub fn create(
        &mut self,
        component: impl Into<String>,
        version: u32,
        label: impl Into<String>,
        data: Vec<u8>,
    ) -> &Snapshot {
        let component = component.into();
        let snapshot = Snapshot::new(component.clone(), version, label, data);
        let list = self.snapshots.entry(component.clone()).or_default();
        list.push(snapshot);
        self.active_version.insert(component.clone(), version);
        list.last().unwrap()
    }

    pub fn versions(&self, component: &str) -> Vec<u32> {
        self.snapshots
            .get(component)
            .map(|list| list.iter().map(|s| s.version).collect())
            .unwrap_or_default()
    }

    pub fn get(&self, component: &str, version: u32) -> Option<&Snapshot> {
        self.snapshots
            .get(component)?
            .iter()
            .find(|s| s.version == version)
    }

    pub fn active_version(&self, component: &str) -> Option<u32> {
        self.active_version.get(component).copied()
    }

    pub fn active(&self, component: &str) -> Option<&Snapshot> {
        let v = self.active_version(component)?;
        self.get(component, v)
    }

    /// Switch the active pointer to an existing version. This is the
    /// "instant rollback" operation: no data is copied or reinstalled.
    pub fn activate(&mut self, component: &str, version: u32) -> Result<(), String> {
        if self.get(component, version).is_none() {
            return Err(format!(
                "no snapshot for component '{component}' at version {version}"
            ));
        }
        self.active_version.insert(component.to_string(), version);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_sets_active_version() {
        let mut store = SnapshotStore::new();
        store.create("browser", 12, "v12", vec![1]);
        assert_eq!(store.active_version("browser"), Some(12));
        store.create("browser", 13, "v13", vec![2]);
        assert_eq!(store.active_version("browser"), Some(13));
    }

    #[test]
    fn old_versions_coexist_after_new_create() {
        let mut store = SnapshotStore::new();
        store.create("browser", 12, "v12", vec![1]);
        store.create("browser", 13, "v13", vec![2]);
        store.create("browser", 14, "v14", vec![3]);
        assert_eq!(store.versions("browser"), vec![12, 13, 14]);
    }

    #[test]
    fn activate_rolls_back_without_reinstall() {
        let mut store = SnapshotStore::new();
        store.create("browser", 12, "v12", vec![1]);
        store.create("browser", 13, "v13", vec![2]);
        assert_eq!(store.active_version("browser"), Some(13));

        store.activate("browser", 12).unwrap();
        assert_eq!(store.active_version("browser"), Some(12));
        // v13 snapshot data still present, not deleted by rollback.
        assert!(store.get("browser", 13).is_some());
    }

    #[test]
    fn activate_unknown_version_fails() {
        let mut store = SnapshotStore::new();
        store.create("browser", 12, "v12", vec![1]);
        assert!(store.activate("browser", 99).is_err());
        assert_eq!(store.active_version("browser"), Some(12));
    }

    #[test]
    fn active_returns_matching_snapshot() {
        let mut store = SnapshotStore::new();
        store.create("os", 1, "initial", vec![9, 9]);
        let active = store.active("os").unwrap();
        assert_eq!(active.label, "initial");
        assert!(active.is_intact());
    }
}
