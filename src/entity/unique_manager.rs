use std::ops::Range;

use crate::EntityId;

#[derive(Debug,Clone)]
pub struct EntityManager {
    next_id: usize,
    entities: Vec<Range<usize>>
}

impl EntityManager {
    pub fn new() -> EntityManager {
        EntityManager{
            next_id: 1,
            entities: Vec::new(),
        }
    }
}

impl super::EntityManager for EntityManager {
    fn allocate(&mut self) -> EntityId {
        // # Safety
        // * next_id is start from 1
        // * next_id is always increased
        // * overflow a usize will panic, it cannot be here with next_id = 0
        let id = unsafe {
            super::EntityId::new_unchecked(self.next_id)
        };
        self.next_id += 1;
        id
    }

    fn allocate_n(&mut self,count: usize) -> std::ops::Range<EntityId> {
        todo!()
    }

    fn remove(&mut self,id: EntityId) {
        todo!()
    }

    fn has(&self,id: EntityId) -> bool {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn entities(&self) -> Box<dyn Iterator<Item=EntityId> + '_> {
        todo!()
    }
}