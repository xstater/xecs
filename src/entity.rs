use std::{any::TypeId, num::NonZeroUsize, ops::Range};
use parking_lot::RwLockReadGuard;
use crate::{component::{Component, ComponentRead, ComponentWrite}, group::Group, sparse_set::SparseSet, world::World};

/// The type of ID of entity which starts from 1 and can be recycled automatically
pub type EntityId = NonZeroUsize;

/// A useful struct for building a entity
// #[derive(Debug)]
pub struct Entity<'a>{
    world : &'a World,
    id : EntityId,
    // To avoid remove this ID from world
    // The ID must be valid during Entity is alive
    #[allow(unused)]
    borrow_entity_manager : RwLockReadGuard<'a,EntityManager>
}

impl<'a> Entity<'a>{
    pub(in crate) fn new(world : &'a World,
                         borrow_entity_manager : RwLockReadGuard<'a,EntityManager>,
                         entity_id : EntityId) -> Self{
        Entity{
            world,
            id: entity_id,
            borrow_entity_manager,
        }
    }

    /// Consume Entity and get an ID
    pub fn into_id(self) -> EntityId{
        self.id
    }

    /// Attach a component to entity
    /// # Panics
    /// * Panics if ```T``` has not been registered
    pub fn attach<T : Component>(self,component : T) -> Self{
        let world = self.world;
        assert!(world.has_registered::<T>(),
                "Entity:Cannot attach component because components has not been registered.");
        let type_id = TypeId::of::<T>();
        {
            // Unwrap never fails because assert ensures this
            let mut storage = world.raw_storage_write(type_id).unwrap();
            // SAFTY:
            // storage is SparseSet<EntityId,T>
            let sparse_set = unsafe {
                storage.downcast_mut::<SparseSet<EntityId,T>>()
            };
            sparse_set.add(self.id,component);
        }
        for mut group in world.groups(type_id) {
            match &mut *group {
                Group::FullOwning(data) => {
                    let (type_a,type_b) = data.types();
                    let mut comp_a = world.raw_storage_write(type_a).unwrap();
                    let mut comp_b = world.raw_storage_write(type_b).unwrap();
                    data.add(self.id,&mut comp_a,&mut comp_b);
                },
                Group::PartialOwning(data) => {
                    let (type_a,type_b) = data.types();
                    let mut comp_a = world.raw_storage_write(type_a).unwrap();
                    let comp_b = world.raw_storage_read(type_b).unwrap();
                    data.add(self.id,&mut comp_a,&comp_b);
                },
                Group::NonOwning(data) => {
                    let (type_a,type_b) = data.types();
                    let comp_a = world.raw_storage_read(type_a).unwrap();
                    let comp_b = world.raw_storage_read(type_b).unwrap();
                    data.add(self.id,&comp_a,&comp_b);
                }
            }
        }
        self
    }

    /// Detach a component from entity
    /// # Panics
    /// * Panics if ```T``` has not been registered
    pub fn detach<T : Component>(&self) -> Option<T> {
        let world = self.world;
        assert!(world.has_registered::<T>(),
                "World:Cannot detach component because components has not been registered.");
        let type_id = TypeId::of::<T>();
        for mut group in world.groups(type_id) {
            match &mut *group{
                Group::FullOwning(data) => {
                    let (type_a,type_b) = data.types();
                    let mut comp_a = world.raw_storage_write(type_a).unwrap();
                    let mut comp_b = world.raw_storage_write(type_b).unwrap();
                    data.remove(self.id,&mut comp_a,&mut comp_b);
                },
                Group::PartialOwning(data) => {
                    let (type_a,type_b) = data.types();
                    let mut comp_a = world.raw_storage_write(type_a).unwrap();
                    let comp_b = world.raw_storage_read(type_b).unwrap();
                    data.remove(self.id,&mut comp_a,&comp_b);
                },
                Group::NonOwning(data) => {
                    let (type_a,type_b) = data.types();
                    let comp_a = world.raw_storage_read(type_a).unwrap();
                    let comp_b = world.raw_storage_read(type_b).unwrap();
                    data.remove(self.id,&comp_a,&comp_b);
                }
            }
        }

        // Unwrap never fails because assert ensures this
        let mut storage = world.raw_storage_write(type_id).unwrap();
        // SAFTY:
        // storage is SparseSet<EntityId,T>
        let sparse_set = unsafe {
            storage.downcast_mut::<SparseSet<EntityId,T>>()
        };
        sparse_set.remove(self.id)
    }

    /// Read component of this entity
    pub fn component_read<T : Component>(&self) -> Option<ComponentRead<'_,T>> {
        self.world.entity_component_read(self.id)
    }

    /// Write component of this entity
    pub fn component_write<T : Component>(&self) -> Option<ComponentWrite<'_,T>> {
        self.world.entity_component_write(self.id)
    }

    /// remove this entity from the world
    pub fn manaully_drop(self) {
        self.world.remove_entity(self.id)
    }
}

/// A useful struct for building a lot of entities
pub struct Entities<'a>{
    world: &'a World,
    ids : Range<EntityId>,
    // To avoid remove this ID from world
    // The ID must be valid during Entity is alive
    #[allow(unused)]
    borrow_entity_manager : RwLockReadGuard<'a,EntityManager>
}

impl<'a> Entities<'a> {
    pub(in crate) fn new(world : &'a World,
                         ids : Range<EntityId>,
                         borrow_entity_manager : RwLockReadGuard<'a,EntityManager>,) -> Self{
        Entities{
            world,
            ids,
            borrow_entity_manager,
        }
    }

    /// Attach components to all entities
    /// # Panics
    /// * Panics if ```T``` has not been registered
    /// * Panics if ```components.len()``` is not equal to the count of entities
    pub fn attach<T,C>(self,components: C) -> Self
    where T : Component,
          C : Into<Vec<T>>{
        // ensure the slice length is equal to the entity count
        let count = self.ids.end.get() - self.ids.start.get();
        let components : Vec<T> = components.into();
        assert_eq!(components.len(),count);
        let type_id = TypeId::of::<T>();
        let mut sparse_set = self.world.raw_storage_write(type_id)
            .expect("Entities:Cannot attach component because components has not been registered.");
        // Safety:
        // sparse_set is SparseSet<EntityId,T>
        let sparse_set = unsafe {
            sparse_set.downcast_mut::<SparseSet<EntityId,T>>()
        };
        // create Id slice
        let ids = (self.ids.start.get()..self.ids.end.get())
            // Safety:
            // Safe here id cannot be zero 
            .map(|id|unsafe{EntityId::new_unchecked(id)})
            .collect::<Vec<_>>();
        sparse_set.add_batch(&ids,components);
        self
    }

    /// Get ID range
    /// # Details
    /// Because create_entites() ensure the id is continuous,
    /// so we can just return the range of EntityId for optimization
    pub fn into_ids(self) -> Range<EntityId> {
        self.ids
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

    pub(in crate) fn allocate(&mut self) -> EntityId {
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

    /// Allocate ```n``` entities
    /// This ensure the entity id is continuous
    pub(in crate) fn allocate_n(&mut self, n : usize) -> Range<EntityId> {
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

    #[allow(dead_code)]
    pub(in crate) fn entities(&self) -> &[EntityId] {
        &self.entities
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
        let mut manager = EntityManager::new();

        let range = manager.allocate_n(5);
        let range = range.start.get()..range.end.get();
        let entities = range.map(|id|EntityId::new(id).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(manager.entities(),&entities);
        println!("flags:{:?}",manager.entity_flags.as_slice());
        println!("entities:{:?}",manager.entities.as_slice());

        let range = manager.allocate_n(3);
        let range = range.start.get()..range.end.get();
        let entities = range.map(|id|EntityId::new(id).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(&manager.entities()[5..8],&entities);
        println!("flags:{:?}",manager.entity_flags.as_slice());
        println!("entities:{:?}",manager.entities.as_slice());
    }
    
}
