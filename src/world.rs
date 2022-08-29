use std::{collections::HashMap, any::TypeId, num::NonZeroUsize};

use parking_lot::RwLock;
use xsparseset::SparseSet;

use crate::{StorageId, Component, storage::ComponentStorage, EntityId};

pub struct World {
    next_other_storage_id: u32,
    storages: HashMap<StorageId,RwLock<Box<dyn ComponentStorage>>>
}

impl World {
    // Create a new empty World
    pub fn new() -> Self {
        World { 
            next_other_storage_id: 0,
            storages: HashMap::new(),
        }
    }

    /// Get a `StoageId` from rust type
    pub fn get_rust_storage_id<T: Component>() -> StorageId {
        StorageId::Rust(TypeId::of::<T>())
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
    pub fn register_with_storage<S: ComponentStorage + 'static>(&mut self,storage_id: StorageId,storage: S){
        if self.storages.contains_key(&storage_id) {
            panic!("Cannot register a component storage twice")
        }
        self.storages.insert(storage_id, RwLock::new(Box::new(storage)));
    }
    
    /// A fast function to register a rust type component storage
    /// # Details
    /// * The default storage is `xsparseset::SparseSet<EntityId,T,HashMap<EntityId,NonZeroUsize>>`
    /// # Panics
    /// * Panic if the `storage_id` is already registered
    pub fn register<T: Component>(&mut self) -> StorageId {
        let storage_id = Self::get_rust_storage_id::<T>();
        let storage: SparseSet<EntityId,T,HashMap<EntityId,NonZeroUsize>> = SparseSet::default();
        self.register_with_storage(storage_id, storage);
        storage_id
    }

    /// Unregister a storage and return the storage if unregistering is successful
    pub fn unregister(&mut self,storage_id: StorageId) -> Option<Box<dyn ComponentStorage>>{
        let rwlock = self.storages.remove(&storage_id)?;
        Some(rwlock.into_inner())
    }

    /// Check the `storage_id` was registered
    pub fn has_registered(&self,storage_id: StorageId) -> bool {
        self.storages.contains_key(&storage_id)
    }

}