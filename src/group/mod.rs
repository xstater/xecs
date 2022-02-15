//! # Group
//! Group is a useful method to accelerate the query iteration. 
//! ## Acceleration Principle
//! To make iteration more fast and more cache friendly, we can rearrange the ord 
//! of items. Group rearranges all group owning components which are both exist in their 
//! sparse set to the front of sparse set.  
//! We classify the groups as 3 types by the owner of components storage.  
//! **Component storage can only be owned by one group** 
//! ### Full-Owning Group
//! Full-owning group owns 2 component storages as its name.It's the fastest group type 
//! because its can rearrange these 2 component storages to make them aligned.
//! ### Partial-Owning Group
//! Partial-Owning only owns the first storage.It's not faster than Full-Owning group but 
//! it can stil make iteration fast
//! ### Non-Owning Group
//! This group does not own any storage.It use an extra sparse set to 
//! record the entities owned by all storage.Although it's the slowest group and it need more 
//! memory to accelerate the iteration,it sill fast than raw query iteration.
use std::{any::TypeId, sync::{RwLockReadGuard, RwLockWriteGuard}};
use crate::{component::{Component, ComponentStorage}, entity::EntityId, world::World};

/// Full-owning group and its [Queryable](crate::query::Queryable) impls
pub mod full_owning;
/// Partial-owning group and its [Queryable](crate::query::Queryable) impls
pub mod partial_owning;
/// Non-owning group and its [Queryable](crate::query::Queryable) impls
pub mod non_owning;

pub use full_owning::FullOwning;
pub use partial_owning::PartialOwning;
pub use non_owning::NonOwning;

/// A trait to make group dynamic, just like [ComponentStorage](crate::component::ComponentStorage)
pub trait Group : Send + Sync{
    fn len(&self) -> usize;
    fn type_id_a(&self) -> TypeId;
    fn type_id_b(&self) -> TypeId;
    fn owning_types(&self) -> Vec<TypeId>;

    fn storage_a<'a>(&self,world : &'a World) -> RwLockReadGuard<'a,Box<dyn ComponentStorage>>{
        world.raw_storage_read(self.type_id_a())
            .expect("Group: Component was not registered in world")
    }
    fn storage_a_mut<'a>(&self,world : &'a World) ->RwLockWriteGuard<'a,Box<dyn ComponentStorage>>{
        world.raw_storage_write(self.type_id_a())
            .expect("Group: Component was not registered in world")
    }
    fn storage_b<'a>(&self,world : &'a World) -> RwLockReadGuard<'a,Box<dyn ComponentStorage>>{
        world.raw_storage_read(self.type_id_b())
            .expect("Group: Component was not registered in world")
    }
    fn storage_b_mut<'a>(&self,world : &'a World) -> RwLockWriteGuard<'a,Box<dyn ComponentStorage>>{
        world.raw_storage_write(self.type_id_b())
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

/// A useful function to create FullOwning group
pub fn full_owning<A : Component,B : Component>() -> FullOwning<A,B> {
    FullOwning::<A,B>::new()
}

/// A useful function to create PartialOwning group
pub fn partial_owning<A : Component,B : Component>() -> PartialOwning<A,B> {
    PartialOwning::<A,B>::new()
}

/// A useful function to create NonOwning group
pub fn non_owning<A : Component,B : Component>() -> NonOwning<A,B> {
    NonOwning::<A,B>::new()
}

