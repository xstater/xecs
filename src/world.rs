use std::{any::TypeId, collections::HashMap, num::NonZeroUsize};

use parking_lot::RwLock;
use xsparseset::SparseSet;

use crate::{
    entity::EntityManager,
    storage::{ComponentStorage, StorageRead, StorageWrite},
    Component, Entity, EntityId, StorageId,
};

/// The core of XECS
pub struct World {
    next_other_storage_id: u32,
    storages: HashMap<StorageId, RwLock<Box<dyn ComponentStorage>>>,
    entities: RwLock<EntityManager>,
}

impl World {
    /// Create a new empty World
    pub fn new() -> Self {
        World {
            next_other_storage_id: 0,
            storages: HashMap::new(),
            entities: RwLock::new(EntityManager::new()),
        }
    }

    /// Allocate a `StorageId` for storing the foreign data
    pub fn allocate_other_storage_id(&mut self) -> StorageId {
        let id = self.next_other_storage_id;
        self.next_other_storage_id += 1;
        StorageId::Other(id)
    }

    /// Register A custom component storage in world
    /// # Panics
    /// * Panic if the `storage_id` is already registered
    pub fn register_with_storage<S: ComponentStorage + 'static>(
        &mut self,
        storage_id: StorageId,
        storage: S,
    ) {
        if self.storages.contains_key(&storage_id) {
            panic!("Cannot register a component storage twice")
        }
        self.storages
            .insert(storage_id, RwLock::new(Box::new(storage)));
    }

    /// A fast function to register a rust type component storage
    /// # Details
    /// * The default storage is `xsparseset::SparseSet<EntityId,T,HashMap<EntityId,NonZeroUsize>>`
    /// # Panics
    /// * Panic if the `storage_id` is already registered
    pub fn register<T: Component>(&mut self) -> StorageId {
        let storage_id = StorageId::from_rust_type::<T>();
        let storage: SparseSet<EntityId, T, HashMap<EntityId, NonZeroUsize>> = SparseSet::default();
        self.register_with_storage(storage_id, storage);
        storage_id
    }

    /// Unregister a storage and return the storage if unregistering is successful
    pub fn unregister(&mut self, storage_id: StorageId) -> Option<Box<dyn ComponentStorage>> {
        let rwlock = self.storages.remove(&storage_id)?;
        Some(rwlock.into_inner())
    }

    /// Check the `storage_id` was registered
    pub fn has_registered(&self, storage_id: StorageId) -> bool {
        self.storages.contains_key(&storage_id)
    }

    /// Get a storage in a read guard
    pub fn storage_read(&self, storage_id: StorageId) -> Option<StorageRead<'_>> {
        let lock = self.storages.get(&storage_id)?.read();
        Some(StorageRead::from_gurad(lock))
    }

    /// Get a storage in a write gurad
    pub fn storage_write(&self, storage_id: StorageId) -> Option<StorageWrite<'_>> {
        let lock = self.storages.get(&storage_id)?.write();
        Some(StorageWrite::from_gurad(lock))
    }

    /// Create an empty entity and return a `Entity` which can
    /// manuiplate the entity conveniently
    pub fn create_entity(&self) -> Entity<'_> {
        let mut manager = self.entities.write();
        let id = manager.allocate();
        std::mem::drop(manager);
        let manager = self.entities.read();
        Entity {
            world: self,
            id,
            _manager: manager,
        }
    }
}

#[cfg(test)]
mod tests {
    use xsparseset::SparseSet;

    use crate::{EntityId, StorageId, World};

    #[test]
    fn register_test() {
        let mut world = World::new();

        let id_i32 = world.register::<i32>();
        assert!(world.has_registered(id_i32));
        let id_char = StorageId::from_rust_type::<char>();
        assert!(!world.has_registered(id_char));
        let storage_char: SparseSet<EntityId, char, xsparseset::VecStorage<EntityId>> =
            SparseSet::default();
        world.register_with_storage(id_char, storage_char);
        assert!(world.has_registered(id_char));

        let ids = (0..10)
            .map(|_| {
                let id = world.allocate_other_storage_id();
                let storage: SparseSet<EntityId, char, xsparseset::VecStorage<EntityId>> =
                    SparseSet::default();
                world.register_with_storage(id, storage);
                id
            })
            .collect::<Vec<_>>();

        for id in ids.iter().copied() {
            assert!(world.has_registered(id));
        }
    }

    #[test]
    fn storage_get_test() {
        let mut world = World::new();

        let char_id = world.register::<char>();

        {
            let char_storage = world.storage_read(char_id);
            assert!(char_storage.is_some());
            let char_storage = char_storage.unwrap();
            char_storage.as_ref().is_empty();
        }

        {
            let char_storage = world.storage_write(char_id);
            assert!(char_storage.is_some());
            let char_storage = char_storage.unwrap();
            char_storage.as_ref().is_empty();
        }
    }
}
