//! # About entity
//! Entity in XECS is just an number ID.In XECS, it's just a 
//! [NonZeroUsize](std::num::NonZeroUsize).
//! The ID is allocated from 1 by world automatically. The ```id=0``` 
//! represents a recycled ID without any other flags through ```Option<EntityId>```.
//! # ID recycling
//! When you call ```world.create_entity()```, an ID will be allocated automatically. 
//! If you call ```world.remove_entity(id)```, this ID will be a pit. If the 
//! next ```world.create_entity()``` is called, it will allocate this ID to fill 
//! the pit.Thanks to [sparse set](crate::sparse_set), it's still fast to 
//! iterate all components no matter how random of ID
use std::any::TypeId;
use std::num::NonZeroUsize;
use crate::component::Component;
use crate::sparse_set::SparseSet;
use crate::world::World;

/// The type of ID of entity which starts from 1 and can be recycled automatically
pub type EntityId = NonZeroUsize;

/// A useful struct for manipulating a entity
#[derive(Debug)]
pub struct EntityRef<'a>{
    world : &'a mut World,
    id : EntityId,
}

impl<'a> EntityRef<'a>{
    pub(in crate) fn new(world : &'a mut World,entity_id : EntityId) -> EntityRef<'a>{
        EntityRef{
            world,
            id: entity_id,
        }
    }

    /// Consume EntityRef and return a valid EntityId
    pub fn into_id(self) -> EntityId{
        self.id
    }

    /// Attach a component to entity
    pub fn attach<T : Component>(self,component : T) -> EntityRef<'a>{
        self.world.attach_component(self.id,component);
        self
    }

    /// Detach a component from entity
    pub fn detach<T : Component>(self) -> EntityRef<'a>{
        self.world.detach_component::<T>(self.id);//ignore the error
        self
    }

    /// Manipulate component of current entity
    /// # Panics
    /// Panic if component is not registered
    pub fn with_component<T,F>(self,mut f : F) -> EntityRef<'a>
    where T : Component,
          F : FnMut(&T) {
        assert!(self.world.has_registered::<T>(),
                "EntityRef: Component was not registered in world");
        {
            // unwrap here:
            // assert before ensure this
            let type_id = TypeId::of::<T>();
            let storage = self.world.storage_ref(type_id).unwrap();
            // SAFTY:
            // Safe here because the raw type of Box<dyn ...> is SparseSet<EntityId,T>
            let sparse_set = unsafe {
                storage.downcast_ref::<SparseSet<EntityId,T>>()
            };
            // SAFTY:
            // Safe here because id was valid when EntityRef is alive
            let component = unsafe {
                sparse_set.get_unchecked(self.id)
            };
            f(component);
        }
        self
    }

    /// Manipulate component of current entity
    /// # Panics
    /// Panic if component is not registered
    pub fn with_component_mut<T,F>(self,mut f : F) -> EntityRef<'a>
    where T : Component,
          F : FnMut(&mut T) {
        assert!(self.world.has_registered::<T>(),
                "EntityRef: Component was not registered in world");
        {
            // unwrap here:
            // assert before ensure this
            let type_id = TypeId::of::<T>();
            let mut storage = self.world.storage_mut(type_id).unwrap();
            // SAFTY:
            // Safe here because the raw type of Box<dyn ...> is SparseSet<EntityId,T>
            let sparse_set = unsafe {
                storage.downcast_mut::<SparseSet<EntityId,T>>()
            };
            // SAFTY:
            // Safe here because id was valid when EntityRef is alive
            let component = unsafe {
                sparse_set.get_unchecked_mut(self.id)
            };
            f(component);
        }
        self
    }
}

#[derive(Debug,Copy,Clone)]
enum EntityFlag{
    /// store the next available EntityID
    Available(EntityId),
    /// store the index of EntityID in entities array
    Unavailable(usize)
}

#[derive(Debug,Clone)]
pub(in crate) struct EntityManager {
    // entity_flags[0] : Because the ID 0 is not a valid ID,
    //     so the first one can be used to store the last removed ID
    //     Unavailable(_)      -> there is no entityID for reuse
    //     Available(EntityID) -> the EntityID
    entity_flags : Vec<EntityFlag>,
    entities : Vec<EntityId>,
}

impl EntityManager {
    pub(in crate) fn new() -> EntityManager {
        EntityManager {
            entity_flags: vec![EntityFlag::Unavailable(0)],
            entities: vec![]
        }
    }

    pub(in crate) fn create(&mut self) -> EntityId {
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
        }else{
            //full
            let id = self.entity_flags.len();
            // safe here because id cannot be zero
            let id = unsafe { EntityId::new_unchecked(id) };
            self.entities.push(id);
            self.entity_flags.push(EntityFlag::Unavailable(self.entities.len() - 1));
            //safe here because this id can't be 0
            id
        }
    }

    // remove entity id
    // Do nothing if entity_id not exist
    pub(in crate) fn remove(&mut self,entity_id : EntityId) {
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

    pub(in crate) fn has(&self,entity_id : EntityId) -> bool {
        if let EntityFlag::Unavailable(_) = self.entity_flags[entity_id.get()] {
            true
        } else {
            false
        }
    }

    pub(in crate) fn entities(&self) -> &[EntityId] {
        self.entities.as_slice()
    }

    #[allow(dead_code)]
    pub(in crate) fn len(&self) -> usize {
        self.entities.len()
    }
}

#[cfg(test)]
mod tests{
    use crate::entity::{EntityId, EntityManager};

    #[test]
    fn manager_test() {
        let mut manager = EntityManager::new();

        manager.create(); // 1
        manager.create(); // 2
        manager.create(); // 3
        manager.create(); // 4
        manager.create(); // 5
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
        assert_eq!(manager.create(),EntityId::new(1).unwrap());
        println!("#create a new entity, id = 1");
        println!("flags :{:?}",manager.entity_flags.as_slice());
        println!("entities :{:?}",manager.entities.as_slice());
        println!();
        assert_eq!(manager.create(),EntityId::new(5).unwrap());
        println!("#create a new entity, id = 5");
        println!("flags :{:?}",manager.entity_flags.as_slice());
        println!("entities :{:?}",manager.entities.as_slice());
        println!();
        assert_eq!(manager.create(),EntityId::new(3).unwrap());
        println!("#create a new entity, id = 3");
        println!("flags :{:?}",manager.entity_flags.as_slice());
        println!("entities :{:?}",manager.entities.as_slice());
        println!();
        assert_eq!(manager.create(),EntityId::new(6).unwrap());
        println!("#create a new entity, id = 6");
        println!("flags :{:?}",manager.entity_flags.as_slice());
        println!("entities :{:?}",manager.entities.as_slice());
        println!();
    }
}
