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

    pub fn allocate_range(&mut self,count: usize) -> std::ops::Range<EntityId> {
        let start = EntityId::new(self.next_id)
            .unwrap_or_else(|| unreachable!("EntityId Cannot be Zero"));
        self.next_id += count;
        let end = EntityId::new(self.next_id)
            .unwrap_or_else(|| unreachable!("EntityId Cannot be Zero"));
        self.entities.insert_range(start.get()..end.get());
        start..end
    }

    pub fn deallocate(&mut self,id: EntityId) {
        self.entities.remove(id.get());
    }

    pub fn deallocate_range(&mut self,range: std::ops::Range<EntityId>) {
        let start = range.start.get();
        let end = range.end.get();
        self.entities.remove_range(start..end)
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
    use rand::Rng;

    use crate::EntityId;

    use super::EntityManager;

    #[test]
    fn basic_test() {
        let mut manager = EntityManager::new();
        let mut ids = Vec::new();
        ids.push(manager.allocate());
        ids.push(manager.allocate());
        assert_eq!(manager.len(),2);
        let range = manager.allocate_range(100);
        let start = range.start.get();
        let end = range.end.get();
        for id in start..end {
            let id = EntityId::new(id).unwrap();
            ids.push(id);
        }
        assert_eq!(manager.len(),102);
        assert_eq!(&manager.entities().collect::<Vec<_>>(),&ids);
        
        let mut rng = rand::thread_rng();
        for _ in 0..ids.len() {
            // randomly choose one id to remove
            let index = rng.gen_range(0..ids.len());
            let id = ids[index];
            assert!(manager.has(id));
            let id = ids.remove(index);
            manager.deallocate(id);
            assert!(!manager.has(id));
            assert_eq!(ids.len(),manager.len());
        }
    }
}