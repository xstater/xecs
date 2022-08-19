use std::{any::TypeId, num::NonZeroUsize, ops::Range};
use parking_lot::RwLockReadGuard;
use crate::{component::{Component, ComponentRead, ComponentWrite}, group::Group, sparse_set::SparseSet, world::World};

pub mod recycle_manager;
pub mod unique_manager;

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
    borrow_entity_manager : RwLockReadGuard<'a,Box<dyn EntityManager>>
}

impl<'a> Entity<'a>{ 
    pub(in crate) fn new(world : &'a World,
                         borrow_entity_manager : RwLockReadGuard<'a,Box<dyn EntityManager>>,
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
            sparse_set.insert(self.id,component);
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
        drop(self.borrow_entity_manager);
        self.world.remove_entity(self.id);
    }
}

/// A useful struct for building a lot of entities
pub struct Entities<'a>{
    world: &'a World,
    ids : Range<EntityId>,
    // To avoid remove this ID from world
    // The ID must be valid during Entity is alive
    #[allow(unused)]
    borrow_entity_manager : RwLockReadGuard<'a,Box<dyn EntityManager>>
}

impl<'a> Entities<'a> {
    pub(in crate) fn new(world : &'a World,
                         ids : Range<EntityId>,
                         borrow_entity_manager : RwLockReadGuard<'a,Box<dyn EntityManager>>) -> Self{
        Entities{
            world,
            ids,
            borrow_entity_manager,
        }
    }

    /// Get ID range
    /// # Details
    /// Because create_entites() ensure the id is continuous,
    /// so we can just return the range of EntityId for optimization
    pub fn into_ids(self) -> Range<EntityId> {
        self.ids
    }
}

pub trait EntityManager {
    fn allocate(&mut self) -> EntityId;
    fn allocate_n(&mut self,count: usize) -> Range<EntityId>;
    fn remove(&mut self,id: EntityId);
    fn has(&self,id: EntityId) -> bool;
    fn len(&self) -> usize;
    fn entities(&self) -> Box<dyn Iterator<Item=EntityId> + '_>;
}


