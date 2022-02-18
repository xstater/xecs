use std::{any::TypeId, marker::PhantomData};
use crate::{component::{Component, ComponentStorage}, entity::EntityId};

use super::Group;

mod query;

pub struct PartialOwningData {
    length : usize,
    type_a : TypeId,
    type_b : TypeId
}

impl PartialEq for PartialOwningData {
    fn eq(&self, other: &Self) -> bool {
        self.type_a == other.type_a && self.type_b == other.type_b
    }
}

impl PartialOwningData {
    pub(in crate) fn len(&self) -> usize {
        self.length
    }

    pub(in crate) fn types(&self) -> (TypeId,TypeId) {
        (self.type_a,self.type_b)
    }

    pub(in crate) fn owned(&self,type_id : TypeId) -> bool {
        type_id == self.type_a
    }

    pub(in crate) fn owning(&self) -> Vec<TypeId> {
        vec![self.type_a]
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

        // get index in component storage
        // This unwrap never failed because the in_components() ensures that it's already in components
        let index_a = comp_a.index(id).unwrap();

        if index_a < self.length {
            true
        } else {
            false
        }
    }

    pub(in crate) fn add(&mut self,
           id : EntityId,
           comp_a : &mut Box<dyn ComponentStorage>,
           comp_b : &Box<dyn ComponentStorage>) {
        if !self.in_components(id,&comp_a,&comp_b) {
            return;
        }
        if self.in_group(id,&comp_a,&comp_b) {
            return;
        }

        // Unwrap will never fail
        // because in_components() ensures that id is in comp_a
        let index_a = comp_a.index(id).unwrap();

        comp_a.swap_by_index(index_a,self.length);

        self.length += 1;
    }

    pub(in crate) fn remove(&mut self,
              id : EntityId,
              comp_a : &mut Box<dyn ComponentStorage>,
              comp_b : &Box<dyn ComponentStorage>) {
        if !self.in_group(id,&comp_a,&comp_b) {
            return;
        }

        // Unwrap will never fail
        // because in_group() ensures that id is in comp_a
        let index_a = comp_a.index(id).unwrap();

        self.length -= 1;

        comp_a.swap_by_index(index_a,self.length);
    }

    pub(in crate) fn make(&mut self,
            comp_a : &mut Box<dyn ComponentStorage>,
            comp_b : &Box<dyn ComponentStorage>) {
        self.length = 0;

        for index in 0..comp_a.count() {
            // Unwrap will never fail
            // for loop ensures the range is valid
            let entity_id = comp_a.id(index).unwrap();
            if comp_b.has(entity_id) {
                comp_a.swap_by_index(index,self.length);
                self.length += 1;
            }
        }
    }
}

// Owning A & Non-Owning B
#[derive(Clone, Copy)]
pub struct PartialOwning<A,B> {
    _marker_a : PhantomData<A>,
    _marker_b : PhantomData<B>
}

impl<A : Component,B : Component> PartialOwning<A,B> {
    pub(in crate) fn new() -> Self {
        PartialOwning {
            _marker_a: PhantomData::default(),
            _marker_b: PhantomData::default(),
        }
    }

}

impl<A : Component,B : Component> Into<Group> for PartialOwning<A,B> {
    fn into(self) -> Group {
        Group::PartialOwning(PartialOwningData {
            length: 0,
            type_a: TypeId::of::<A>(),
            type_b: TypeId::of::<B>()
        })
    }
}

