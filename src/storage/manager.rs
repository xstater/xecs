use std::{hint::unreachable_unchecked, collections::HashSet};

use parking_lot::RwLock;
use xdag::Dag;
use xsparseset::SparseSetHashMap;

use crate::{ComponentStorage, EntityId, ComponentTypeId, StorageId};

enum Group {
    Full(usize),
    Partial(usize),
    Non(SparseSetHashMap<EntityId, (usize, usize)>),
}

enum Node {
    /// Left, Right, Group
    Group(StorageId,StorageId, RwLock<Group>),
    /// Storage
    Storage(RwLock<Box<dyn ComponentStorage>>),
}

pub struct StorageManager {
    next_group_id: u32,
    dag_storages: Dag<StorageId, Node, bool>,
}

impl StorageManager {
    pub fn new() -> Self {
        StorageManager {
            next_group_id: 0,
            dag_storages: Dag::new(),
        }
    }

    fn next_group_id(&mut self) -> StorageId {
        let id = self.next_group_id;
        self.next_group_id += 1;
        StorageId::Group(id)
    }

    fn is_owned(&self, storage_id: StorageId) -> bool {
        todo!()
    }

    pub fn contains(&self, storage_id: StorageId) -> bool {
        self.dag_storages.contains_node(storage_id)
    }

    /// Insert a component storage
    /// # Remarks
    /// * Replace the old one if storage_id is already in Manager
    pub fn insert_component_storage(
        &mut self,
        storage_id: StorageId,
        storage: RwLock<Box<dyn ComponentStorage>>,
    ) {
        self.dag_storages
            .insert_node(storage_id, Node::Storage(storage));
    }

    /// make a full owning group
    /// # Safety
    /// * `storage_id_1` and `storage_id_2` cannot be owned
    pub unsafe fn make_full_owning(&mut self,storage_id_1: StorageId, storage_id_2: StorageId) -> StorageId {
        let id = self.next_group_id();
        self.dag_storages.insert_node(id,Node::Group(storage_id_1, storage_id_2,RwLock::new(Group::Full(0))));
        self.dag_storages.insert_edge(id, storage_id_1, true).unwrap_or_else(|_|unreachable!());
        self.dag_storages.insert_edge(id, storage_id_2, true).unwrap_or_else(|_|unreachable!());
        id
    }

    /// This function call `ComponentStorage::insert_any_unchecked`
    pub unsafe fn insert_component_unchecked(&self, storage_id: StorageId, entity_id: EntityId, data:*mut u8) {
        todo!()
    }
}
