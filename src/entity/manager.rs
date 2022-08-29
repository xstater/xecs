use crate::{EntityId, range_set::RangeSet};

#[derive(Debug)]
pub struct EntityManager {
    next_id: usize,
    entities: RangeSet
}

impl EntityManager {
    pub fn new() -> EntityManager {
        EntityManager{
            next_id: 1,
            entities: RangeSet::new(),
        }
    }
}

impl EntityManager {
    pub fn allocate(&mut self) -> EntityId {
        // # Safety
        // * next_id is start from 1
        // * next_id is always increased
        // * overflow a usize will panic, it cannot be here with next_id = 0
        let id = unsafe {
            EntityId::new_unchecked(self.next_id)
        };
        self.entities.insert(id.get());
        self.next_id += 1;
        id
    }

    pub fn allocate_n(&mut self,count: usize) -> std::ops::Range<EntityId> {
        let start = EntityId::new(self.next_id)
            .unwrap_or_else(|| unreachable!("EntityId Cannot be Zero"));
        self.next_id += count;
        let end = EntityId::new(self.next_id)
            .unwrap_or_else(|| unreachable!("EntityId Cannot be Zero"));
        self.entities.remove_range(start.get()..end.get());
        start..end
    }

    pub fn remove(&mut self,id: EntityId) {
        self.entities.remove(id.get());
    }

    pub fn has(&self,id: EntityId) -> bool {
        self.entities.contains(id.get())
    }

    pub fn len(&self) -> usize {
        // super slow
        self.entities.iter().count()
    }

    pub fn entities(&self) -> impl Iterator<Item = EntityId> + '_{
        self.entities.iter().map(|id| unsafe {
            // # Safety
            // id cannot be zero
            EntityId::new_unchecked(id)
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic_test() {
        todo!("add test for manager");
    }
}