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
use std::any::TypeId;
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

use self::{
    full_owning::FullOwningData,
    non_owning::NonOwningData,
    partial_owning::PartialOwningData
};

#[derive(PartialEq)]
pub enum Group {
    FullOwning(FullOwningData),
    PartialOwning(PartialOwningData),
    NonOwning(NonOwningData)
}

impl Group {
    pub fn len(&self) -> usize {
        match self {
            Group::FullOwning(data) => data.len(),
            Group::PartialOwning(data) => data.len(),
            Group::NonOwning(data) => data.len(),
        }
    }

    pub fn types(&self) -> (TypeId,TypeId) {
        match &self {
            Group::FullOwning(data) => data.types(),
            Group::PartialOwning(data) => data.types(),
            Group::NonOwning(data) => data.types(),
        }
    }

    pub fn owned(&self,type_id : TypeId) -> bool {
        match self {
            Group::FullOwning(data) => data.owned(type_id),
            Group::PartialOwning(data) => data.owned(type_id),
            Group::NonOwning(data) => data.owned(type_id),
        }
    }

    pub fn owning(&self) -> Vec<TypeId> {
        match self {
            Group::FullOwning(data) => data.owning(),
            Group::PartialOwning(data) => data.owning(),
            Group::NonOwning(data) => data.owning(),
        }
    }

    pub fn in_components(&self,
                id : EntityId,
                comp_a : &Box<dyn ComponentStorage>,
                comp_b : &Box<dyn ComponentStorage>) -> bool {
        match self {
            Group::FullOwning(data) => data.in_components(id,comp_a,comp_b),
            Group::PartialOwning(data) => data.in_components(id, comp_a, comp_b),
            Group::NonOwning(data) => data.in_components(id, comp_a, comp_b),
        }
    }

    pub fn in_group(&self,
                id : EntityId,
                comp_a : &Box<dyn ComponentStorage>,
                comp_b : &Box<dyn ComponentStorage>) -> bool {
        match self {
            Group::FullOwning(data) => data.in_group(id, comp_a, comp_b),
            Group::PartialOwning(data) => data.in_group(id, comp_a, comp_b),
            Group::NonOwning(data) => data.in_group(id, comp_a, comp_b),
        }
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

