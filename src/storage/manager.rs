use std::sync::{Arc, Weak};

use parking_lot::RwLock;

use crate::{StorageId, ComponentStorage, EntityId};

pub enum Group {
    Full(Arc<Group>,Arc<Group>),
    Partial(Arc<Group>,Weak<Group>),
    Non(Weak<Group>,Weak<Group>),
    Leaf(StorageId,RwLock<Box<dyn ComponentStorage>>)
}

impl Group {
    pub fn contains(&self, entity_id: EntityId) -> bool {
        match self {
            Group::Full(a, b) => 
                a.contains(entity_id) && b.contains(entity_id),
            Group::Partial(a, b) => {
                let b = Weak::upgrade(b).unwrap_or_else(||unreachable!());
                a.contains(entity_id) && b.contains(entity_id)
            }
            Group::Non(a, b) => {
                let a = Weak::upgrade(a).unwrap_or_else(||unreachable!());
                let b = Weak::upgrade(b).unwrap_or_else(||unreachable!());
                a.contains(entity_id) && b.contains(entity_id)
            },
            Group::Leaf(_, storage) => storage.read().contains(entity_id),
        }
    }
}

pub struct Manager {
    groups: Vec<Group>
}

impl Manager {
    pub fn new() -> Self {
        Manager { groups: Vec::new() }
    }
}



