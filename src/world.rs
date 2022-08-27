use std::{collections::HashMap, any::TypeId};

use parking_lot::RwLock;
use xsparseset::SparseSet;

use crate::{StorageId, Component, storage::ComponentStorage};

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
    
    pub fn register<T: Component>(&mut self) -> StorageId {
        todo!()
    }

    pub fn register_other(&mut self) -> StorageId {
        todo!()
    }

    pub fn unregister(&mut self,storage_id: StorageId) {
        todo!()
    }
}