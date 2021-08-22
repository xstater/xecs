//! # 3 types of group , can be used for improving the speed of iteration
//! # Motivation
//! XECS stores entities with sparse set which can fast iterate and quickly get data from a sparse ID.
//! Sparse set has a dense array, if we can re-arrange the order of entities in dense array,we will get
//! much improvement of iterating speed.
//! # Owned component or Non-Owned component
//! if a components can be re-sorted ,we call it was owned by a group.Each component can only be owned
//! by one Group.
//! # Group type
//! ### Full-Owning Group
//! This group owns 2 components.It can get the fastest speed of iteration.
//! ### Partial-Owning Group
//! This group only owns 1 component.The iteration speed is slow than Full-Owning group.
//! ### Non-Owning group
//! This group doesn't own any component.It need another sparse set for recording some iteration information.
//! The speed is the slowest in 3 type groups,but it's faster than raw query iter in average situation.
//! # Example
//! Usually create a group by ```World::make_query()```
//! ```no_run
//! world.make_group::<(Position,Particle)>(true,true);//full-owning group
//! ```
use std::any::{TypeId};
use crate::sparse_set::SparseSet;
use crate::{EntityId, World, Component};
use crate::components::ComponentStorage;

#[derive(Debug,Clone,Copy)]
pub(in crate) enum OwningType{
    Owning(TypeId),
    NonOwning(TypeId)
}

#[derive(Debug,Clone,Copy)]
pub(in crate) struct OwningGroup {
    pub(in crate) types : (OwningType, OwningType),
    pub(in crate) length : usize
}

impl OwningGroup {
    pub(in crate) fn full(&self) -> bool {
        self.types.0.is_owning() && self.types.1.is_owning()
    }

    pub(in crate) fn is_owned(&self,type_id : TypeId) -> bool {
        (self.types.0.type_id() == type_id && self.types.0.is_owning())
     || (self.types.1.type_id() == type_id && self.types.1.is_owning())
    }
}

#[derive(Debug,Clone)]
pub(in crate) struct NonOwningGroup {
    pub(in crate) types : (TypeId, TypeId),
    pub(in crate) sparse_set : SparseSet<EntityId,(usize, usize)>
}

#[derive(Debug,Clone)]
pub(in crate) enum Group {
    Owning(OwningGroup),
    NonOwning(NonOwningGroup)
}

impl OwningType {
    fn type_id(&self) -> TypeId {
        *match self {
            OwningType::Owning(t) => t,
            OwningType::NonOwning(t) => t
        }
    }

    fn is_owning(&self) -> bool {
        match self {
            OwningType::Owning(_) => true,
            OwningType::NonOwning(_) => false
        }
    }
}

impl Group {
    pub(in crate) fn contains(&self,tid : TypeId) -> bool {
        match self {
            Group::Owning(group) => {
                group.types.0.type_id() == tid || group.types.1.type_id() == tid
            }
            Group::NonOwning(group) => {
                group.types.0 == tid || group.types.1 == tid
            }
        }
    }

    pub(in crate) fn is_owned(&self,tid : TypeId) -> bool {
        match self {
            Group::Owning(group) => {
                if group.types.0.is_owning() && group.types.0.type_id() == tid {
                    return true;
                }
                if group.types.1.is_owning() && group.types.1.type_id() == tid {
                    return true;
                }
            }
            Group::NonOwning(_) => return false
        }
        false
    }

    pub(in crate) fn in_two_components(&self,world : &World,entity_id : EntityId) -> bool {
        let (type_a,type_b) = match self {
            Group::Owning(group) => {
                (group.types.0.type_id(),group.types.1.type_id())
            }
            Group::NonOwning(group) => {
                group.types
            }
        };
        let comp_a = world.components_storage_dyn_ref(type_a);
        let comp_b = world.components_storage_dyn_ref(type_b);
        comp_a.has(entity_id) && comp_b.has(entity_id)
    }

    pub(in crate) fn in_group(&self,world : &World,entity_id : EntityId) -> bool {
        debug_assert!(self.in_two_components(world,entity_id),
            "Cannot call in_group without in_two_components = true");
        match self {
            Group::Owning(group) => {
                if let OwningType::Owning(type_id) = &group.types.0 {
                    let comp = world.components_storage_dyn_ref(*type_id);
                    let index = comp.index(entity_id).unwrap();
                    if index >= group.length {
                        return false;
                    }
                }
                if let OwningType::Owning(type_id) = &group.types.1 {
                    let comp = world.components_storage_dyn_ref(*type_id);
                    let index = comp.index(entity_id).unwrap();
                    if index >= group.length {
                        return false;
                    }
                }
            }
            Group::NonOwning(group) => {
                if !group.sparse_set.has(entity_id) {
                    return false;
                }
            }
        }
        true
    }

    pub(in crate) fn make_group_in_world<A : Component,B : Component>(&mut self,world : &World) {
        match self {
            Group::Owning(group) => {
                group.length = 0;
                if group.full() {
                    // full-owning group
                    let mut comp_a = world.components_storage_mut::<A>();
                    let mut comp_b = world.components_storage_mut::<B>();
                    let len_a = comp_a.len();
                    let len_b = comp_b.len();
                    if len_a < len_b {
                        for index_a in 0..len_a {
                            let entity_id = comp_a.entities()[index_a];
                            if let Some(index_b) = comp_b.get_index(entity_id) {
                                comp_a.swap_by_index(group.length, index_a);
                                comp_b.swap_by_index(group.length, index_b);
                                group.length += 1;
                            }
                        }
                    } else {
                        for index_b in 0..len_b {
                            let entity_id = comp_b.entities()[index_b];
                            if let Some(index_a) = comp_a.get_index(entity_id) {
                                comp_a.swap_by_index(group.length, index_a);
                                comp_b.swap_by_index(group.length, index_b);
                                group.length += 1;
                            }
                        }
                    }
                } else {
                    if group.types.0.is_owning() {
                        let mut comp_a = world.components_storage_mut::<A>();
                        let     comp_b = world.components_storage_ref::<B>();
                        for index in 0..comp_a.len() {
                            let entity_id = comp_a.entities()[index];
                            if comp_b.exist(entity_id) {
                                comp_a.swap_by_index(group.length,index);
                                group.length += 1;
                            }
                        }
                    } else {
                        let     comp_a = world.components_storage_ref::<A>();
                        let mut comp_b = world.components_storage_mut::<B>();
                        for index in 0..comp_b.len() {
                            let entity_id = comp_b.entities()[index];
                            if comp_a.exist(entity_id) {
                                comp_b.swap_by_index(group.length,index);
                                group.length += 1;
                            }
                        }
                    }
                }
            }
            Group::NonOwning(group) => {
                // group.sparse_set.clear()
                let comp_a = world.components_storage_ref::<A>();
                let comp_b = world.components_storage_ref::<B>();
                let len_a = comp_a.len();
                let len_b = comp_b.len();
                if len_a < len_b {
                    for index_a in 0..len_a {
                        let entity_id = comp_a.entities()[index_a];
                        if let Some(index_b) = comp_b.get_index(entity_id){
                            group.sparse_set.add(entity_id,(index_a,index_b));
                        }
                    }
                } else {
                    for index_b in 0..len_b {
                        let entity_id = comp_b.entities()[index_b];
                        if let Some(index_a) = comp_a.get_index(entity_id) {
                            group.sparse_set.add(entity_id,(index_a,index_b));
                        }
                    }
                }
            }
        }
    }

    // this function must be call AFTER adding to component storage
    pub(in crate) fn add(&mut self,world : &World,entity_id : EntityId) {
        if !self.in_two_components(world,entity_id) {
            return;
        }
        if self.in_group(world,entity_id) {
            return;
        }
        match self {
            Group::Owning(group) => {
                if let OwningType::Owning(type_id) = group.types.0 {
                    let mut comp = world.components_storage_dyn_mut(type_id);
                    let index = comp.index(entity_id).unwrap();
                    comp.swap_by_index(index,group.length)
                }
                if let OwningType::Owning(type_id) = group.types.1 {
                    let mut comp = world.components_storage_dyn_mut(type_id);
                    let index = comp.index(entity_id).unwrap();
                    comp.swap_by_index(index,group.length)
                }
                group.length += 1;
            }
            Group::NonOwning(group) => {
                let comp_a = world.components_storage_dyn_ref(group.types.0);
                let comp_b = world.components_storage_dyn_ref(group.types.1);
                let index_a = comp_a.index(entity_id).unwrap();
                let index_b = comp_b.index(entity_id).unwrap();
                group.sparse_set.add(entity_id,(index_a,index_b));
            }
        }
    }

    // this function must be called BEFORE remove entity from components storage
    pub(in crate) fn remove(&mut self,world : &World,entity_id : EntityId) {
        if !self.in_group(world,entity_id) {
            return;
        }
        match self {
            Group::Owning(group) => {
                group.length -= 1;
                if let OwningType::Owning(type_id) = group.types.0 {
                    let mut comp = world.components_storage_dyn_mut(type_id);
                    let index = comp.index(entity_id).unwrap();
                    comp.swap_by_index(index,group.length);
                }
                if let OwningType::Owning(type_id) = group.types.1 {
                    let mut comp = world.components_storage_dyn_mut(type_id);
                    let index = comp.index(entity_id).unwrap();
                    comp.swap_by_index(index,group.length);
                }
            }
            Group::NonOwning(group) => {
                group.sparse_set.remove(entity_id);
            }
        }
    }
}
