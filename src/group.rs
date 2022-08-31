use std::collections::HashMap;

use parking_lot::RwLock;
use crate::{ComponentStorage, StorageId};

pub struct Storage{
    id: StorageId,
    storage: RwLock<Box<dyn ComponentStorage>>
}

pub enum Group {
    Full(Storage,Storage),
    Partial(Storage,StorageId),
    Non(StorageId,StorageId)
}

pub struct Groups {
    storages: Vec<Group>
}

impl Groups {
    pub fn new() -> Self {
        Groups { storages: Vec::new() }
    }

    pub fn push_full(&mut self, storage1: Storage,storage2: Storage) {
        self.storages.push(Group::Full(storage1, storage2));
    }

    pub fn push_partial(&mut self,storage: Storage,id: StorageId) {
        self.storages.push(Group::Partial(storage, id));
    }

    pub fn push_non(&mut self,id1: StorageId,id2: StorageId) {
        self.storages.push(Group::Non(id1,id2));
    }

    pub fn is_owned(&self,id: StorageId) -> bool{
        self.storages.iter()
            .find(|group| match group{
                Group::Full(s1, s2) => s1.id == id || s2.id == id,
                Group::Partial(s, _) => s.id == id,
                Group::Non(_, _) => false,
            })
            .is_some()
    }

    pub fn in_group(&self,id: StorageId) -> bool {
        self.storages.iter()
            .find(|group| match group {
                Group::Full(s1, s2) => s1.id == id || s2.id == id,
                Group::Partial(s, sid) => s.id == id || *sid == id,
                Group::Non(sid1, sid2) => *sid1 == id || *sid2 == id,
            })
            .is_some()
    }

}