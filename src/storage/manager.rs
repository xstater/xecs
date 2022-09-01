use std::collections::HashMap;

use parking_lot::RwLock;
use xdag::Dag;

use crate::{ComponentStorage, StorageId};

enum Node {
    Group,
    Storage(RwLock<Box<dyn ComponentStorage>>)
}

pub struct Manager {
    next_group_id: u64,
    dag_storages: Dag<StorageId,Node,bool>
}

impl Manager {
    pub fn new() -> Self {
        Manager {
            next_group_id: 1,
            dag_storages: Dag::new()
        }
    }

    fn next_group_id(&mut self) -> StorageId {
        let id = self.next_group_id;
        self.next_group_id += 1;
        StorageId::Group(id)
    }

    fn is_owned(&self,storage_id: StorageId) -> bool{
        todo!()
    }

    pub fn contains(&self, storage_id: StorageId) -> bool {
        self.dag_storages.contains_node(storage_id)
    }

    pub fn insert_storage(&mut self, storage_id: StorageId, storage: RwLock<Box<dyn ComponentStorage>>) {
        self.dag_storages.insert_node(storage_id, Node::Storage(storage));
    }

    pub fn remove_storage(&mut self,storage_id: StorageId) -> Option<RwLock<Box<dyn ComponentStorage>>> {
        
        todo!()
    }
}
