use std::ops::Range;

use crate::EntityId;

#[derive(Debug, Copy, Clone)]
enum EntityFlag {
    /// store the next available EntityID
    Available(EntityId),
    /// store the index of EntityID in entities array
    Unavailable(usize),
}

#[derive(Debug, Clone)]
pub struct EntityManager {
    // entity_flags[0] : Because the ID 0 is not a valid ID,
    //     so the first one can be used to store the last removed ID
    //     Unavailable(_)      -> there is no entityID for reuse
    //     Available(EntityID) -> the EntityID
    entity_flags: Vec<EntityFlag>,
    entities: Vec<EntityId>,
}

impl EntityManager {
    pub fn new() -> EntityManager {
        EntityManager {
            entity_flags: vec![EntityFlag::Unavailable(0)],
            entities: vec![],
        }
    }
}
impl super::EntityManager for EntityManager {
    fn allocate(&mut self) -> EntityId {
        //safe here:
        // the entity_flags[0] cannot be removed
        if let EntityFlag::Available(last_id) = self.entity_flags.first().unwrap() {
            let last_id = *last_id;
            //we got an id can be reused
            let new_id = self.entity_flags[last_id.get()];
            self.entities.push(last_id);
            self.entity_flags[last_id.get()] = EntityFlag::Unavailable(self.entities.len() - 1);
            self.entity_flags[0] = new_id;
            last_id
        } else {
            //full
            let id = self.entity_flags.len();
            // safe here because id cannot be zero
            let id = unsafe { EntityId::new_unchecked(id) };
            self.entities.push(id);
            self.entity_flags
                .push(EntityFlag::Unavailable(self.entities.len() - 1));
            //safe here because this id can't be 0
            id
        }
    }

    /// Allocate ```n``` entities
    /// This ensure the entity id is continuous
    fn allocate_n(&mut self, n: usize) -> Range<EntityId> {
        // Get the range of entity id
        let start_id = self.entity_flags.len();
        let end_id = start_id + n;
        // Get the range of entity index
        let start_index = self.entities.len();
        let end_index = start_index + n;
        // record indecies to self.entity_flags
        for i in start_index..end_index {
            self.entity_flags.push(EntityFlag::Unavailable(i));
        }
        for id in start_id..end_id {
            let id = unsafe { EntityId::new_unchecked(id) };
            self.entities.push(id);
        }
        let start_id = unsafe { EntityId::new_unchecked(start_id) };
        let end_id = unsafe { EntityId::new_unchecked(end_id) };
        start_id..end_id
    }

    // remove entity id
    // Do nothing if entity_id not exist
    fn remove(&mut self, entity_id: EntityId) {
        let entity_id_ = entity_id.get();
        if let EntityFlag::Unavailable(index) = self.entity_flags[entity_id_] {
            // unwrap safe: in this branch, we must have one entity at least
            let the_last_one_id = self.entities.last().unwrap();
            // move this entity to the end of entities
            self.entity_flags[the_last_one_id.get()] = EntityFlag::Unavailable(index);
            self.entities.swap_remove(index);
            // keep these destroyed ids being a chain
            self.entity_flags[entity_id_] = self.entity_flags[0];
            self.entity_flags[0] = EntityFlag::Available(entity_id);
        }
    }

    fn has(&self, entity_id: EntityId) -> bool {
        if let EntityFlag::Unavailable(_) = self.entity_flags[entity_id.get()] {
            true
        } else {
            false
        }
    }

    fn entities(&self) -> Box<dyn Iterator<Item=EntityId> + '_> {
        Box::new(self.entities.iter().copied())
    }

    fn len(&self) -> usize {
        self.entities.len()
    }
}

#[cfg(test)]
mod tests{
    use crate::{EntityId, entity::EntityManager};


    #[test]
    fn manager_test() {
        let mut manager = crate::entity::recycle_manager::EntityManager::new();

        manager.allocate(); // 1
        manager.allocate(); // 2
        manager.allocate(); // 3
        manager.allocate(); // 4
        manager.allocate(); // 5
        assert_eq!(dbg!(manager.len()),5);
        println!("#initial");
        println!("flags    :{:?}",manager.entity_flags.as_slice());
        println!("entities :{:?}",manager.entities.as_slice());
        println!();
        manager.remove(EntityId::new(3).unwrap());
        println!("#removed id=3");
        println!("flags :{:?}",manager.entity_flags.as_slice());
        println!("entities :{:?}",manager.entities.as_slice());
        println!();
        manager.remove(EntityId::new(5).unwrap());
        println!("#removed id=5");
        println!("flags :{:?}",manager.entity_flags.as_slice());
        println!("entities :{:?}",manager.entities.as_slice());
        println!();
        manager.remove(EntityId::new(1).unwrap());
        println!("#removed id=1");
        println!("flags :{:?}",manager.entity_flags.as_slice());
        println!("entities :{:?}",manager.entities.as_slice());
        println!();
        assert_eq!(manager.allocate(),EntityId::new(1).unwrap());
        println!("#create a new entity, id = 1");
        println!("flags :{:?}",manager.entity_flags.as_slice());
        println!("entities :{:?}",manager.entities.as_slice());
        println!();
        assert_eq!(manager.allocate(),EntityId::new(5).unwrap());
        println!("#create a new entity, id = 5");
        println!("flags :{:?}",manager.entity_flags.as_slice());
        println!("entities :{:?}",manager.entities.as_slice());
        println!();
        assert_eq!(manager.allocate(),EntityId::new(3).unwrap());
        println!("#create a new entity, id = 3");
        println!("flags :{:?}",manager.entity_flags.as_slice());
        println!("entities :{:?}",manager.entities.as_slice());
        println!();
        assert_eq!(manager.allocate(),EntityId::new(6).unwrap());
        println!("#create a new entity, id = 6");
        println!("flags :{:?}",manager.entity_flags.as_slice());
        println!("entities :{:?}",manager.entities.as_slice());
        println!();
    }

    #[test]
    fn create_entities() {
        let mut manager = crate::entity::recycle_manager::EntityManager::new();

        let range = manager.allocate_n(5);
        let range = range.start.get()..range.end.get();
        let entities = range.map(|id|EntityId::new(id).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(&manager.entities().collect::<Vec<_>>(),&entities);
        println!("flags:{:?}",manager.entity_flags.as_slice());
        println!("entities:{:?}",manager.entities.as_slice());

        let range = manager.allocate_n(3);
        let range = range.start.get()..range.end.get();
        let entities = range.map(|id|EntityId::new(id).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(&manager.entities().collect::<Vec<_>>()[5..8],&entities);
        println!("flags:{:?}",manager.entity_flags.as_slice());
        println!("entities:{:?}",manager.entities.as_slice());
    }
    
}