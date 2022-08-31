mod manager;
#[cfg(test)]
mod tests;

use crate::{Component, EntityId, StorageId, World};
pub use manager::EntityManager;
use parking_lot::RwLockReadGuard;
use std::any::{type_name, TypeId};

/// A `World` handle with an id, so that it can be used to manipulate entity conveniently
/// # Remarks
/// * It contains a read lock guard of `EnityManager` to ensure the id in it is always valid.
///   So you cannot create or remove an entity when an `Entity` is living.
pub struct Entity<'a> {
    pub(crate) world: &'a World,
    pub(crate) id: EntityId,
    // keep this lock guard to avoid id being removed
    pub(crate) _manager: RwLockReadGuard<'a, EntityManager>,
}

impl<'a> Entity<'a> {
    /// Attach a component to entity
    /// # Panics
    /// * Panic when `T` is not registered in `World`
    pub fn attach<T: Component>(self, component: T) -> Self {
        let type_id = TypeId::of::<T>();
        if let Some(mut storage) = self.world.storage_write(StorageId::Rust(type_id)) {
            let _ = storage.insert::<T>(self.id, component);
        } else {
            panic!("Attach component to entity 'id={}' failed. The type of Component '{}' is not registered in world",self.id,type_name::<T>());
        }
        self
    }

    /// Detach a component from entity
    /// # Panics
    /// * Panic when `T` is not registered in `World`
    pub fn detach<T: Component>(self) -> Self {
        let type_id = TypeId::of::<T>();
        if let Some(mut storage) = self.world.storage_write(StorageId::Rust(type_id)) {
            let _ = storage.remove_ignored(self.id);
        } else {
            panic!("Detach component from entity 'id={}' failed. The type of Component '{}' is not registered in world",self.id,type_name::<T>());
        }
        self
    }

    /// Consume the `Entity` and get the id in it
    pub fn into_id(self) -> EntityId {
        self.id
    }
}
