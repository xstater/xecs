mod component;
mod group;
mod guards;
mod id;
#[cfg(test)]
mod tests;

use std::collections::HashMap;

pub use component::ComponentStorage;
pub use group::{FullOwningGroup, GroupStorage};
pub use guards::{StorageRead, StorageWrite};
pub use id::{ComponentTypeId, StorageId};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use xdag::Dag;

use crate::EntityId;

pub trait Storage: Send + Sync {
    /// Get how many item in storage
    fn len(&self) -> usize;
    /// Check if storage is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// downcast to component storage
    fn as_component_storage_ref(&self) -> Option<&dyn ComponentStorage>;
    /// downcast to componentstorage
    fn as_component_storage_mut(&mut self) -> Option<&mut dyn ComponentStorage>;
    /// downcast to group storage
    fn as_group_storage_ref(&self) -> Option<&dyn GroupStorage>;
    /// downcast to component
    fn as_group_storage_mut(&mut self) -> Option<&mut dyn GroupStorage>;
}

pub(crate) struct Storages {
    pub(crate) storages: Dag<StorageId, RwLock<Box<dyn Storage>>, bool>,
}

impl Storages {
    /// Add a storage to storages
    /// # Safety
    /// * `storage_id.is_component_storage() == true`
    /// * `storage` must implemented `ComponentStorage`
    /// * `self.storages.contains_node(storage_id) == false`
    pub(crate) unsafe fn add_component_storage_unchecked(
        &mut self,
        storage_id: StorageId,
        storage: Box<dyn Storage>,
    ) {
        self.storages.insert_node(storage_id, RwLock::new(storage));
    }
    /// Add a group to storages
    /// # Safety
    /// * `group_id.is_group_storage() == true`
    /// * `self.storages.contains_node(group_id) == false`
    /// * `self.storages.contains_node(storage_id1) == true`
    /// * `self.storages.contains_node(storage_id2) == true`
    /// * `storage` must implemented `GroupStorage` and ha
    /// * `self.is_owned(storage_id1) == false`
    /// * `self.is_owned(storage_id2) == false`
    pub(crate) unsafe fn add_full_owning_group_unchecked(
        &mut self,
        group_id: StorageId,
        group: Box<dyn Storage>,
        storage_id1: StorageId,
        storage_id2: StorageId,
    ) {
        self.storages.insert_node(group_id, RwLock::new(group));
        self.storages
            .insert_edge(group_id, storage_id1, true)
            .unwrap_unchecked();
        self.storages
            .insert_edge(group_id, storage_id2, true)
            .unwrap_unchecked();
    }

    /// Check a storage is owned by any other storage
    /// # Safety
    /// * `storage_id` must exist in `Storages`
    pub(crate) unsafe fn is_owned(&self, storage_id: StorageId) -> bool {
        for parent in self.storages.parents(storage_id) {
            let edge = self
                .storages
                .get_edge(parent, storage_id)
                // # Safety
                // * `storage_id` must exist in `Storage`
                .unwrap_unchecked()
                // # Safety
                // * Must have this edge because the `from` of `get_edge` is the parents of `storage`
                .unwrap_unchecked();
            if *edge {
                return true;
            }
        }
        false
    }

    /// Check an entity exists in storage
    /// # Details
    /// * storage can be a group
    /// * when storage is a group ,return `Some(index)` only if entity exists in all its children
    /// * if a storage is not locked (lock guard cannot be found in `read_locks` or write_locks`),
    ///   this function will read lock it and add guard to `read_locks`
    /// # Safety
    /// * `self.storages.contains_node(storage_id) == true`
    pub(crate) unsafe fn contains_entity(
        &self,
        storage_id: StorageId,
        entity_id: EntityId,
    ) -> bool {
        todo!();
    }

    /// Get the index of entity in storage
    /// # Details
    /// * storage can be a group
    /// * when storage is a group ,return `Some(index)` only if all indices of children storages are `Some(index)` and equal
    /// * if a storage is not locked (lock guard cannot be found in `read_locks` or write_locks`),
    ///   this function will read lock it and add guard to `read_locks`
    /// # Safety
    /// * `self.storages.contains_node(storage_id) == true`
    pub(crate) unsafe fn get_index(
        &self,
        storage_id: StorageId,
        entity_id: EntityId,
    ) -> Option<usize> {
        todo!()
    }

    /// Swap two entities in storage
    /// # Details
    /// * storage can be a group
    /// * when storage is a group , all children storages will execute this function
    /// # Safety
    /// * `self.storages.contains_node(storage_id) == true`
    /// * `index_a` and `index_b` must be in range
    pub(crate) unsafe fn swap_entity_by_index_unchecked(
        &self,
        storage_id: StorageId,
        index_a: usize,
        index_b: usize,
    ) {
        todo!()
    }

    pub(crate) unsafe fn add_entity_to_group_unchecked(
        &self,
        storage_id: StorageId,
        entity_id: EntityId,
    ) {
        todo!()
    }
}
