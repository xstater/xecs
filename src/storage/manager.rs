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
    dag_groups: Dag<StorageId,Node,bool>
}

impl Manager {
    pub fn new() -> Self {
        Manager {
            next_group_id: 1,
            dag_groups: Dag::new()
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
        todo!()
    }
}
