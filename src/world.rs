
use parking_lot::RwLock;
use xsparseset::{SparseSetHashMap};

use crate::{
    entity::EntityManager,
    storage::{ComponentStorage, StorageRead, StorageWrite},
    Component, ComponentTypeId, Entity, EntityId,GroupType, StorageId,
};

/// The core of XECS
pub struct World {
    next_other_storage_id: u64,
    entities: RwLock<EntityManager>,
}

impl World {
    /// Create a new empty World
    pub fn new() -> Self {
        World {
            next_other_storage_id: 0,
            entities: RwLock::new(EntityManager::new()),
        }
    }

    /// Register a custom component storage with `ComponentTypeId`
    /// # Panics
    /// * Panic if `has_registered(component_id)`
    pub fn register_with_storage<S>(&mut self, component_id: ComponentTypeId, storage: S) -> StorageId
    where
        S: ComponentStorage + 'static,
    {
        todo!()
    }

    /// Register a default component storage with `ComponentTypeId`
    /// # Details
    /// * default component storage is `xsparseset::SparseSetHashMap<EntityId,C>`
    /// # Panics
    /// * Panic if `has_registered(component_id)`
    pub fn register<C: Component>(&mut self) -> StorageId {
        let component_id = ComponentTypeId::from_rust_type::<C>();
        let storage: SparseSetHashMap<EntityId,C> = SparseSetHashMap::default();
        self.register_with_storage(component_id, storage)
    }

    /// Check a `storage_id` is registered
    pub fn is_registered(&self, storage_id: StorageId) -> bool {
        todo!()
    }

    /// Unregister a component
    /// # Tips
    /// * This function can use to release memory
    /// # Returns 
    /// * Return `Some(storage)` if unregister successfull
    pub fn unregister(&mut self, storage_id: StorageId) -> Option<Box<dyn ComponentStorage>> {
        todo!()
    }

    /// Get a storage in a read guard
    pub fn storage_read(&self, storage_id: StorageId) -> Option<StorageRead<'_>> {
        todo!()
    }

    /// Get a storage in a write gurad
    pub fn storage_write(&self, storage_id: StorageId) -> Option<StorageWrite<'_>> {
        todo!()
    }

    /// Make a group to accelerate the query
    /// # Panics
    /// * Panic if `storage_id_1` or `storage_id_2` is already owned by another group
    pub fn make_group(&mut self,group_type: GroupType, storage_id_1: StorageId, storage_id_2: StorageId) -> StorageId {
        todo!()
    }

    /// Get the `Entity` by given `entity_id`
    pub fn entity(&self, entity_id: EntityId) -> Option<Entity<'_>> {
        let manager = self.entities.read();
        if manager.has(entity_id) {
            Some(Entity {
                world: self,
                id: entity_id,
                _manager: manager,
            })
        } else {
            None
        }
    }

    /// Create an empty entity and return a `Entity` which can
    /// manipulate the entity conveniently
    pub fn create_entity(&self) -> Entity<'_> {
        let mut manager = self.entities.write();
        let id = manager.allocate();
        std::mem::drop(manager);
        self.entity(id).unwrap_or_else(|| unreachable!())
    }
}

#[cfg(test)]
mod tests {
}
