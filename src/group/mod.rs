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
use std::{any::TypeId, intrinsics::transmute};
use crate::{component::{Component, ComponentStorage}, entity::EntityId};

/// Full-owning group and its [Queryable](crate::query::Queryable) impls
pub mod full_owning;
/// Partial-owning group and its [Queryable](crate::query::Queryable) impls
pub mod partial_owning;
/// Non-owning group and its [Queryable](crate::query::Queryable) impls
pub mod non_owning;

pub use full_owning::FullOwning;
pub use partial_owning::PartialOwning;
pub use non_owning::NonOwning;

#[derive(Debug,Clone,Copy,PartialEq)]
pub enum GroupType {
    FullOwning(TypeId,TypeId),
    PartialOwning(TypeId,TypeId),
    NonOwning(TypeId,TypeId)
}

impl GroupType {
    pub fn types(&self) -> (TypeId,TypeId) {
        match self {
            GroupType::FullOwning(a,b) => (*a,*b),
            GroupType::PartialOwning(a,b) => (*a,*b),
            GroupType::NonOwning(a,b) => (*a,*b),
        }
    }

    pub fn owned(&self,type_id : TypeId) -> bool {
        match self {
            GroupType::FullOwning(a,b) => 
                type_id == *a || type_id == *b,
            GroupType::PartialOwning(a,_) =>
                type_id == *a,
            GroupType::NonOwning(_, _) => false,
        }
    }

    pub fn owning(&self) -> Vec<TypeId> {
        match self {
            GroupType::FullOwning(a,b) => vec![*a,*b],
            GroupType::PartialOwning(a, _) => vec![*a],
            GroupType::NonOwning(_, _) => vec![],
        }
    }
}

/// A trait to make group dynamic, just like [ComponentStorage](crate::component::ComponentStorage)
pub trait Group : Send + Sync{
    fn len(&self) -> usize;
    fn group_type(&self) -> GroupType;

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
}

pub(in crate) trait FullOwningGroup : Group{
    fn add(&mut self,
           id : EntityId,
           comp_a : &mut Box<dyn ComponentStorage>,
           comp_b : &mut Box<dyn ComponentStorage>);

    fn remove(&mut self,
              id : EntityId,
              comp_a : &mut Box<dyn ComponentStorage>,
              comp_b : &mut Box<dyn ComponentStorage>);

    fn make(&mut self,
            comp_a : &mut Box<dyn ComponentStorage>,
            comp_b : &mut Box<dyn ComponentStorage>);
}
   
pub(in crate) trait PartialOwningGroup : Group{
    fn add(&mut self,
           id : EntityId,
           comp_a : &mut Box<dyn ComponentStorage>,
           comp_b : &Box<dyn ComponentStorage>);

    fn remove(&mut self,
              id : EntityId,
              comp_a : &mut Box<dyn ComponentStorage>,
              comp_b : &Box<dyn ComponentStorage>);

    fn make(&mut self,
            comp_a : &mut Box<dyn ComponentStorage>,
            comp_b : &Box<dyn ComponentStorage>);
}

pub(in crate) trait NonOwningGroup : Group{
    fn add(&mut self,
           id : EntityId,
           comp_a : &Box<dyn ComponentStorage>,
           comp_b : &Box<dyn ComponentStorage>);

    fn remove(&mut self,
              id : EntityId,
              comp_a : &Box<dyn ComponentStorage>,
              comp_b : &Box<dyn ComponentStorage>);

    fn make(&mut self,
            comp_a : &Box<dyn ComponentStorage>,
            comp_b : &Box<dyn ComponentStorage>);
}

impl dyn 'static + Group{
    pub(in crate) unsafe fn downcast_ref<T : Group>(&self) -> &T{
        &*(self as *const dyn Group as *const T)
    }
}

impl dyn Group {
    // Safety:
    // Safe only self impl FullOwningGroup trait
    pub(in crate) unsafe fn downcast_full_owning(&mut self) -> &mut dyn FullOwningGroup {
        transmute(self)
    }
    // Safety:
    // Safe only self impl FullOwningGroup trait
    pub(in crate) unsafe fn downcast_partial_owning(&mut self) -> &mut dyn PartialOwningGroup {
        transmute(self)
    }
    // Safety:
    // Safe only self impl FullOwningGroup trait
    pub(in crate) unsafe fn downcast_non_owning(&mut self) -> &mut dyn NonOwningGroup{
        transmute(self)
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

