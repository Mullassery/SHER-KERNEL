//! High-level restore/rollback helper built on top of [`crate::store::SnapshotStore`].

use crate::store::SnapshotStore;

/// Roll `component` back to `target_version`. Fails if that version was
/// never captured, or if the captured data has been corrupted (checksum
/// mismatch) since it was recorded.
pub fn restore(
    store: &mut SnapshotStore,
    component: &str,
    target_version: u32,
) -> Result<(), String> {
    {
        let snapshot = store
            .get(component, target_version)
            .ok_or_else(|| format!("no snapshot for '{component}' at version {target_version}"))?;
        if !snapshot.is_intact() {
            return Err(format!(
                "snapshot for '{component}' at version {target_version} failed integrity check"
            ));
        }
    }
    store.activate(component, target_version)
}

/// Roll back to the previous version relative to the currently active one,
/// if one exists.
pub fn restore_previous(store: &mut SnapshotStore, component: &str) -> Result<u32, String> {
    let versions = store.versions(component);
    let current = store
        .active_version(component)
        .ok_or_else(|| format!("component '{component}' has no active version"))?;
    let idx = versions.iter().position(|&v| v == current).ok_or_else(|| {
        "active version missing from version list (invariant violated)".to_string()
    })?;
    if idx == 0 {
        return Err(format!("no version older than {current} for '{component}'"));
    }
    let previous = versions[idx - 1];
    restore(store, component, previous)?;
    Ok(previous)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restore_switches_active_version() {
        let mut store = SnapshotStore::new();
        store.create("browser", 12, "v12", vec![1]);
        store.create("browser", 13, "v13", vec![2]);

        restore(&mut store, "browser", 12).unwrap();
        assert_eq!(store.active_version("browser"), Some(12));
    }

    #[test]
    fn restore_unknown_version_errors() {
        let mut store = SnapshotStore::new();
        store.create("browser", 12, "v12", vec![1]);
        assert!(restore(&mut store, "browser", 999).is_err());
    }

    #[test]
    fn restore_previous_steps_back_one_version() {
        let mut store = SnapshotStore::new();
        store.create("os", 1, "a", vec![1]);
        store.create("os", 2, "b", vec![2]);
        store.create("os", 3, "c", vec![3]);

        let restored = restore_previous(&mut store, "os").unwrap();
        assert_eq!(restored, 2);
        assert_eq!(store.active_version("os"), Some(2));
    }

    #[test]
    fn restore_previous_at_oldest_version_errors() {
        let mut store = SnapshotStore::new();
        store.create("os", 1, "only", vec![1]);
        assert!(restore_previous(&mut store, "os").is_err());
    }
}
