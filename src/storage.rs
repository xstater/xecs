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
        read_locks: &mut HashMap<StorageId, RwLockReadGuard<'_, Box<dyn Storage>>>,
        write_locks: &HashMap<StorageId, RwLockWriteGuard<'_, Box<dyn Storage>>>,
    ) -> bool {
        let storage = if let Some(read) = read_locks.get(&storage_id) {
            read.as_ref()
        } else if let Some(write) = write_locks.get(&storage_id) {
            write.as_ref()
        } else {
            let read = self.storages.get_node(storage_id).unwrap_unchecked().read();
            read_locks.insert(storage_id, read);
            read_locks.get(&storage_id).unwrap_unchecked().as_ref()
        };

        if storage_id.is_component_storage() {
            let storage = storage.as_component_storage_ref().unwrap_unchecked();
            storage.contains(entity_id)
        } else {
            let mut children_iter = self.storages.children(storage_id);
            let (child_id1, _) = children_iter.next().unwrap_unchecked();
            let (child_id2, _) = children_iter.next().unwrap_unchecked();
            let result_1 = self.contains_entity(child_id1, entity_id, read_locks, write_locks);
            let result_2 = self.contains_entity(child_id2, entity_id, read_locks, write_locks);

            result_1 && result_2
        }
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
        read_locks: &mut HashMap<StorageId, RwLockReadGuard<'_, Box<dyn Storage>>>,
        write_locks: &HashMap<StorageId, RwLockWriteGuard<'_, Box<dyn Storage>>>,
    ) -> Option<usize> {
        let storage = if let Some(read) = read_locks.get(&storage_id) {
            read.as_ref()
        } else if let Some(write) = write_locks.get(&storage_id) {
            write.as_ref()
        } else {
            let read = self.storages.get_node(storage_id).unwrap_unchecked().read();
            read_locks.insert(storage_id, read);
            read_locks.get(&storage_id).unwrap_unchecked().as_ref()
        };

        if storage_id.is_component_storage() {
            let storage = storage.as_component_storage_ref().unwrap_unchecked();
            storage.get_index(entity_id)
        } else {
            let mut children_iter = self.storages.children(storage_id);
            let (child_id1, _) = children_iter.next().unwrap_unchecked();
            let (child_id2, _) = children_iter.next().unwrap_unchecked();
            let index_1 = self.get_index(child_id1, entity_id, read_locks, write_locks)?;
            let index_2 = self.get_index(child_id2, entity_id, read_locks, write_locks)?;
            if index_1 == index_2 {
                Some(index_1)
            } else {
                None
            }
        }
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
        read_locks: &mut HashMap<StorageId, RwLockReadGuard<'_, Box<dyn Storage>>>,
        write_locks: &mut HashMap<StorageId, RwLockWriteGuard<'_, Box<dyn Storage>>>,
    ) {
        let storage = if let Some(write) = write_locks.get_mut(&storage_id) {
            write.as_mut()
        } else {
            if read_locks.contains_key(&storage_id) {
                read_locks.remove(&storage_id);
            }
            let write = self.storages.get_node(storage_id).unwrap_unchecked().write();
            write_locks.insert(storage_id, write);
            write_locks.get_mut(&storage_id).unwrap_unchecked().as_mut()
        };

        if storage_id.is_component_storage() {
            let storage = storage.as_component_storage_ref().unwrap_unchecked();
            storage.swap_by_index_unchecked(index_a, index_b);
        } else {
            let mut children_iter = self.storages.children(storage_id);
            let (child_id1, is_owned1) = children_iter.next().unwrap_unchecked();
            let (child_id2, is_owned2) = children_iter.next().unwrap_unchecked();
            // can only swap owning storage
            if *is_owned1 {
                self.swap_entity_by_index_unchecked(child_id1, index_a, index_b,read_locks, write_locks);
            }
            if *is_owned2 {
                self.swap_entity_by_index_unchecked(child_id2, index_a, index_b, read_locks, write_locks);
            }
        }
    }

    pub(crate) unsafe fn add_entity_to_group_unchecked(
        &self,
        storage_id: StorageId,
        entity_id: EntityId,
        read_locks: &mut HashMap<StorageId, RwLockReadGuard<'_, Box<dyn Storage>>>,
        write_locks: &mut HashMap<StorageId, RwLockWriteGuard<'_, Box<dyn Storage>>>,
    ) {
        let storage = if let Some(write) = write_locks.get_mut(&storage_id) {
            write.as_mut()
        } else {
            let write = self.storages.get_node(storage_id).unwrap_unchecked().write();
            write_locks.insert(storage_id, write);
            write_locks.get_mut(&storage_id).unwrap_unchecked().as_mut()
        };

        if storage_id.is_component_storage() {
            return;
        }

        let group = storage.as_group_storage_mut().unwrap_unchecked();
        let mut children_iter = self.storages.children(storage_id);
        let (child_id1, is_owned1) = children_iter.next().unwrap_unchecked();
        let (child_id2, is_owned2) = children_iter.next().unwrap_unchecked();
        
        self.add_entity_to_group_unchecked(child_id1, entity_id, read_locks, write_locks);
        self.add_entity_to_group_unchecked(child_id2, entity_id, read_locks, write_locks);

        if let Some(index_a) = self.get_index(child_id1, entity_id, &mut HashMap::new(), write_locks) 
        && let Some(index_b) = self.get_index(child_id2, entity_id, &mut HashMap::new(), write_locks){
            // need add to group
            // this cannot be overflow 
            // because when `len() == 0`,`get_index` will return None
            let last_index = group.len() - 1;
            // we can only swap the owning storage
            if *is_owned1 {

            }
        }
    }
}
