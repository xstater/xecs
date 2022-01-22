use std::{any::TypeId, sync::{RwLockReadGuard, RwLockWriteGuard}};
use crate::{component::{Component, ComponentStorage}, entity::EntityId, world::World};

pub mod full_owning;
pub mod partial_owning;
pub mod non_owning;

pub use full_owning::FullOwning;
pub use partial_owning::PartialOwning;
pub use non_owning::NonOwning;

pub trait Group : Send + Sync{
    fn len(&self) -> usize;
    fn type_id_a(&self) -> TypeId;
    fn type_id_b(&self) -> TypeId;
    fn owning_types(&self) -> Vec<TypeId>;

    fn storage_a<'a>(&self,world : &'a World) -> RwLockReadGuard<'a,Box<dyn ComponentStorage>>{
        world.storage_ref(self.type_id_a())
            .expect("Group: Component was not registered in world")
    }
    fn storage_a_mut<'a>(&self,world : &'a World) ->RwLockWriteGuard<'a,Box<dyn ComponentStorage>>{
        world.storage_mut(self.type_id_a())
            .expect("Group: Component was not registered in world")
    }
    fn storage_b<'a>(&self,world : &'a World) -> RwLockReadGuard<'a,Box<dyn ComponentStorage>>{
        world.storage_ref(self.type_id_b())
            .expect("Group: Component was not registered in world")
    }
    fn storage_b_mut<'a>(&self,world : &'a World) -> RwLockWriteGuard<'a,Box<dyn ComponentStorage>>{
        world.storage_mut(self.type_id_b())
            .expect("Group: Component was not registered in world")
    }

    /// Check if entity exist in both component storages
    fn in_components(&self,
                     id : EntityId,
                     comp_a : &Box<dyn ComponentStorage>,
                     comp_b : &Box<dyn ComponentStorage>) -> bool {
        comp_a.has(id) && comp_b.has(id)
    }

    /// Check if entity exist in group
    fn in_group(&self,
                id : EntityId,
                comp_a : &Box<dyn ComponentStorage>,
                comp_b : &Box<dyn ComponentStorage>) -> bool;
    fn add(&mut self,world : &World,id : EntityId);
    fn remove(&mut self,world : &World,id : EntityId);

    fn make_group_in_world(&mut self,world : &World);
}

impl dyn 'static + Group{
    pub(in crate) unsafe fn downcast_ref<T : Group>(&self) -> &T{
        &*(self as *const dyn Group as *const T)
    }
}

pub fn full_owning<A : Component,B : Component>() -> FullOwning<A,B> {
    FullOwning::<A,B>::new()
}

pub fn partial_owning<A : Component,B : Component>() -> PartialOwning<A,B> {
    PartialOwning::<A,B>::new()
}
pub fn non_owning<A : Component,B : Component>() -> NonOwning<A,B> {
    NonOwning::<A,B>::new()
}

