mod component;
mod group;
mod guards;
mod id;
#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet, VecDeque};

pub use component::ComponentStorage;
pub use group::{FullOwningGroup, GroupStorage};
pub use guards::{StorageRead, StorageWrite};
pub use id::{ComponentTypeId, StorageId};
use parking_lot::{RwLock, RwLockReadGuard, RwLockUpgradableReadGuard, RwLockWriteGuard};
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
    /// downcast to component storage
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

    /// Get all locks
    /// # Details
    /// * It sort the locks by StorageId to avoid dead-lock
    /// * all functions can only get locks from this function, it ensured the order of locks
    /// # Safety
    /// * All id in `storage_ids` must exists in `Storages`
    /// * `storage_ids` cannot has repeat id
    unsafe fn locks(
        &self,
        storage_ids: impl Iterator<Item = (StorageId, bool)>,
    ) -> (HashMap<StorageId, RwLockReadGuard<'_, Box<dyn Storage>>>, HashMap<StorageId, RwLockWriteGuard<'_,Box<dyn Storage>>>){
        let mut ids = storage_ids.collect::<Vec<_>>();

        ids.sort_unstable();

        let mut read_locks = HashMap::new();
        let mut write_locks = HashMap::new();

        for (id,is_read) in ids {
            if is_read {
                let read = self.storages.get_node(id)
                    .unwrap_unchecked()
                    .read();
                read_locks.insert(id, read);
            } else {
                let write = self.storages.get_node(id)
                    .unwrap_unchecked()
                    .write();
                write_locks.insert(id,write);
            }
        }

        (read_locks,write_locks)
    }

    /// Get all roots of the storage by given `storage_id`
    fn roots_of(&self, storage_id: StorageId) -> Vec<StorageId> {
        let mut roots = Vec::new();

        let mut queue = VecDeque::new();
        queue.push_back(storage_id);

        while let Some(current) = queue.pop_front() {
            let mut count = 0;
            for parent in self.storages.parents(current) {
                queue.push_back(parent);
                count += 1;
            }
            if count == 0 {
                roots.push(current);
            }
        }

        roots
    }

    /// Get all storages of sub graph which `storage_id` is in
    fn sub_graph_of(&self, storage_id: StorageId) -> Vec<StorageId> {
        let roots = self.roots_of(storage_id);
        let mut queue = VecDeque::new();

        for root in roots {
            queue.push_back(root);
        }

        let mut ids = Vec::new();
        while let Some(current) = queue.pop_front() {
            if !ids.contains(&current) {
                ids.push(current);

                for (child, _) in self.storages.children(current) {
                    queue.push_back(child)
                }
            }
        }

        ids
    }

    /// Check an entity exists in storage
    /// # Details
    /// * storage can be a group
    /// * when storage is a group ,return `Some(index)` only if entity exists in all its children
    /// * if a storage is not locked (lock guard cannot be found in `read_locks` or write_locks`),
    ///   this function will read lock it and add guard to `read_locks`
    /// # Safety
    /// * `self.storages.contains_node(storage_id) == true`
    unsafe fn contains_entity<'func, 'this>(
        &'this self,
        storage_id: StorageId,
        entity_id: EntityId,
        locks: &'func HashMap<StorageId, RwLockReadGuard<'this, Box<dyn Storage>>>,
    ) -> bool {
        if storage_id.is_component_storage() {
            let storage = locks.get(&storage_id).unwrap_unchecked();
            storage
                .as_component_storage_ref()
                .unwrap_unchecked()
                .contains(entity_id)
        } else {
            for (child, _) in self.storages.children(storage_id) {
                if !self.contains_entity(child, entity_id, locks) {
                    return false;
                }
            }
            true
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
    ) {
        todo!()
    }

    pub(crate) unsafe fn add_entity_to_group_unchecked(
        &self,
        storage_id: StorageId,
        entity_id: EntityId,
    ) {
        let sub_graph_storages = self.sub_graph_of(storage_id);
        let read_locks = self.locks(sub_graph_storages.into_iter().map(|id|(id,false)));

        let mut need_upgrade = Vec::new();

        let roots = self.roots_of(storage_id);

        


    }
}
