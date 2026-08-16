//! Object manager: create, track, and look up [`sher_objectmodel::KernelObject`]
//! instances by id. Real in-process bookkeeping — this is exactly the kind
//! of primitive a userspace kernel simulation *can* implement for real,
//! unlike `sher_bootstrap`'s hardware bring-up.

use sher_common::{Error, ObjectId, ObjectType, Result};
use sher_objectmodel::KernelObject;
use std::collections::HashMap;

#[derive(Default)]
pub struct ObjectManager {
    objects: HashMap<ObjectId, KernelObject>,
    root: Option<ObjectId>,
}

impl ObjectManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create the immutable root object (idempotent: calling twice returns
    /// the existing root's id rather than creating a second one).
    pub fn create_root(&mut self) -> ObjectId {
        if let Some(root) = self.root {
            return root;
        }
        let root = KernelObject::new(ObjectType::Process, "root");
        let id = root.id;
        self.objects.insert(id, root);
        self.root = Some(id);
        id
    }

    pub fn root(&self) -> Option<ObjectId> {
        self.root
    }

    pub fn create(&mut self, obj_type: ObjectType, name: impl Into<String>) -> ObjectId {
        let object = KernelObject::new(obj_type, name);
        let id = object.id;
        self.objects.insert(id, object);
        id
    }

    pub fn get(&self, id: ObjectId) -> Option<&KernelObject> {
        self.objects.get(&id)
    }

    pub fn get_mut(&mut self, id: ObjectId) -> Option<&mut KernelObject> {
        self.objects.get_mut(&id)
    }

    /// Remove an object, refusing to remove the root while other objects
    /// still depend on it.
    pub fn remove(&mut self, id: ObjectId) -> Result<()> {
        if Some(id) == self.root && self.objects.len() > 1 {
            return Err(Error::Unknown(
                "cannot remove root object while dependents exist".to_string(),
            ));
        }
        self.objects.remove(&id);
        if Some(id) == self.root {
            self.root = None;
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}

/// Stage 1 entry point: create the object manager and its root object.
pub fn initialize() -> Result<ObjectManager> {
    let mut manager = ObjectManager::new();
    manager.create_root();
    Ok(manager)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_creates_root() {
        let manager = initialize().unwrap();
        assert!(manager.root().is_some());
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn create_root_is_idempotent() {
        let mut manager = ObjectManager::new();
        let first = manager.create_root();
        let second = manager.create_root();
        assert_eq!(first, second);
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn create_and_get_object() {
        let mut manager = ObjectManager::new();
        let id = manager.create(ObjectType::Driver, "e1000e");
        let obj = manager.get(id).unwrap();
        assert_eq!(obj.name, "e1000e");
    }

    #[test]
    fn removing_root_with_dependents_is_rejected() {
        let mut manager = ObjectManager::new();
        manager.create_root();
        manager.create(ObjectType::Driver, "d1");
        let root = manager.root().unwrap();
        assert!(manager.remove(root).is_err());
    }

    #[test]
    fn removing_lone_root_succeeds() {
        let mut manager = ObjectManager::new();
        let root = manager.create_root();
        assert!(manager.remove(root).is_ok());
        assert!(manager.root().is_none());
        assert!(manager.is_empty());
    }
}
