mod manager;
#[cfg(test)]
mod tests;

use crate::{Component, EntityId, ComponentTypeId, World};
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
        todo!()
    }

    /// Detach a component from entity
    /// # Panics
    /// * Panic when `T` is not registered in `World`
    pub fn detach<T: Component>(self) -> Self {
        todo!()
    }

    pub fn id(&self) -> EntityId {
        self.id
    }

    /// Consume the `Entity` and get the id in it
    pub fn into_id(self) -> EntityId {
        self.id
    }

    /// Drop this entity manually
    pub fn manually_drop(self) {
        todo!()
    }
}
