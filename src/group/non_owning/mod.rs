use std::{any::TypeId, marker::PhantomData};
use crate::{component::{Component, ComponentStorage}, entity::EntityId, sparse_set::SparseSet};
use super::Group;

mod query;

pub use query::{
    IterRefRef,
    IterRefMut,
    IterMutRef,
    IterMutMut
};


pub struct NonOwningData {
    sparse_set : SparseSet<EntityId,(usize,usize)>,
    type_a : TypeId,
    type_b : TypeId
}

impl PartialEq for NonOwningData {
    fn eq(&self, other: &Self) -> bool {
        self.type_a == other.type_a && self.type_b == other.type_b
    }
}

impl NonOwningData {
    pub(in crate) fn len(&self) -> usize {
        self.sparse_set.len()
    }
    pub(in crate) fn types(&self) -> (TypeId,TypeId) {
        (self.type_a,self.type_b)
    }

    pub(in crate) fn owned(&self,_type_id : TypeId) -> bool {
        false
    }

    pub(in crate) fn owning(&self) -> Vec<TypeId> {
        vec![]
    }

    pub(in crate) fn in_components( &self,
                id : EntityId,
                comp_a : &Box<dyn ComponentStorage>,
                comp_b : &Box<dyn ComponentStorage>) -> bool {
        comp_a.has(id) && comp_b.has(id)
    }
    pub(in crate) fn in_group(&self,
                id : EntityId,
                comp_a : &Box<dyn ComponentStorage>,
                comp_b : &Box<dyn ComponentStorage>) -> bool {
        if !self.in_components(id,comp_a,comp_b) {
            return false;
        }

        self.sparse_set.exist(id)
    }
    pub(in crate) fn add(&mut self,
           id : EntityId,
           comp_a : &Box<dyn ComponentStorage>,
           comp_b : &Box<dyn ComponentStorage>) {
        if !self.in_components(id,&comp_a,&comp_b) {
            return;
        }
        if self.in_group(id,&comp_a,&comp_b) {
            return;
        }

        // get index in component storage
        // This unwrap never fails because the in_components() ensures that it's already in components.
        let index_a = comp_a.index(id).unwrap();
        let index_b = comp_b.index(id).unwrap();

        self.sparse_set.add(id,(index_a,index_b));
    }

    pub(in crate) fn remove(&mut self,
              id : EntityId,
              comp_a : &Box<dyn ComponentStorage>,
              comp_b : &Box<dyn ComponentStorage>) {
        if !self.in_group(id,&comp_a,&comp_b) {
            return;
        }

        // Unwrap here
        // This never fails because in_group ensures that it's already in group.
        self.sparse_set.remove(id).unwrap();
    }

    pub(in crate) fn make(&mut self,
            comp_a : &Box<dyn ComponentStorage>,
            comp_b : &Box<dyn ComponentStorage>) {
        self.sparse_set.clear();

        let len_a = comp_a.count();
        let len_b = comp_b.count();

        if len_a < len_b {
            for index_a in 0..len_a {
                // Unwrap here never fails
                // the for loop ensures this
                let entity_id = comp_a.id(index_a).unwrap();
                if let Some(index_b) = comp_b.index(entity_id) {
                    self.sparse_set.add(entity_id,(index_a,index_b));
                }
            }
        } else {
            for index_b in 0..len_b {
                // Unwrap here never fails
                // the for loop ensures this
                let entity_id = comp_b.id(index_b).unwrap();
                if let Some(index_a) = comp_a.index(entity_id) {
                    self.sparse_set.add(entity_id,(index_a,index_b));
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct NonOwning<A,B>{
    _marker_a : PhantomData<A>,
    _marker_b : PhantomData<B>
}

impl<A : Component,B : Component> NonOwning<A,B> {
    pub(in crate) fn new() -> Self {
        NonOwning {
            _marker_a : PhantomData::default(),
            _marker_b : PhantomData::default()
        }
    }
}

impl<A : Component,B : Component> Into<Group> for NonOwning<A,B> {
    fn into(self) -> Group {
        Group::NonOwning(NonOwningData {
            sparse_set : SparseSet::new(),
            type_a: TypeId::of::<A>(),
            type_b: TypeId::of::<B>()
        })
    }
}
