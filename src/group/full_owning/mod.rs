use std::{any::TypeId, marker::PhantomData};
use crate::{component::{Component, ComponentStorage}, entity::EntityId};

mod query;

pub use query::{
    IterRefRef,
    IterRefMut,
    IterMutRef,
    IterMutMut
};

use super::Group;

pub struct FullOwningData{
    length : usize,
    type_a : TypeId,
    type_b : TypeId
}

impl PartialEq for FullOwningData {
    fn eq(&self, other: &Self) -> bool {
        self.type_a == other.type_a && self.type_b == other.type_b
    }
}

impl FullOwningData {
    pub(in crate) fn len(&self) -> usize {
        self.length
    }
    
    pub(in crate) fn types(&self) -> (TypeId,TypeId) {
        (self.type_a,self.type_b)
    }

    pub(in crate) fn owned(&self,type_id : TypeId) -> bool {
        type_id == self.type_a || type_id == self.type_b
    }

    pub(in crate) fn owning(&self) -> Vec<TypeId> {
        vec![self.type_a,self.type_b]
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

        // get indexes in both component storages
        // This unwrap never fails because the in_components() ensures that it's already in components
        let index_a = comp_a.index(id).unwrap();
        let index_b = comp_b.index(id).unwrap();
        if index_a < self.length && index_b < self.length {
            true
        } else {
            false
        }
    }

    pub(in crate) fn add(&mut self,
           id : EntityId,
           comp_a : &mut Box<dyn ComponentStorage>,
           comp_b : &mut Box<dyn ComponentStorage>) {
        if !self.in_components(id,&comp_a,&comp_b) {
            return;
        }
        if self.in_group(id,&comp_a,&comp_b) {
            return;
        }
        
        // get indexes in both component storages
        // This unwrap never fails because the in_components() ensures that it's already in components
        let index_a = comp_a.index(id).unwrap();
        let index_b = comp_b.index(id).unwrap();

        comp_a.swap_by_index(index_a,self.length);
        comp_b.swap_by_index(index_b,self.length);

        self.length += 1;
    }

    pub(in crate) fn remove(&mut self,
              id : EntityId,
              comp_a : &mut Box<dyn ComponentStorage>,
              comp_b : &mut Box<dyn ComponentStorage>) {
        if !self.in_group(id,&comp_a,&comp_b) {
            return;
        }

        // get indexes in both component storages
        // This unwrap never fails because the in_group() ensure that it's already in components
        let index_a = comp_a.index(id).unwrap();
        let index_b = comp_b.index(id).unwrap();

        self.length -= 1;

        comp_a.swap_by_index(index_a,self.length);
        comp_b.swap_by_index(index_b,self.length);
    }

    pub(in crate) fn make(&mut self,
            comp_a : &mut Box<dyn ComponentStorage>,
            comp_b : &mut Box<dyn ComponentStorage>) {
        self.length = 0;

        let len_a = comp_a.count();
        let len_b = comp_b.count();

        if len_a < len_b {
            for index_a in 0..len_a {
                    // Unwrap here never fails
                    // the for loop ensure this
                    let id = comp_a.id(index_a).unwrap();
                    if let Some(index_b) = comp_b.index(id) {
                        comp_a.swap_by_index(index_a,self.length);
                        comp_b.swap_by_index(index_b,self.length);
                        self.length += 1;
                    }
                }
            } else {
                for index_b in 0..len_b {
                    // Unwrap here never fails
                    // the for loop ensure this
                    let id = comp_b.id(index_b).unwrap();
                    if let Some(index_a) = comp_a.index(id) {
                        comp_a.swap_by_index(index_a,self.length);
                        comp_b.swap_by_index(index_b,self.length);
                        self.length += 1;
                    }
                }
        }
    }
}

#[derive(Clone,Copy)]
pub struct FullOwning<A,B>{
    _marker_a : PhantomData<A>,
    _marker_b : PhantomData<B>
}

impl<A : Component,B : Component> FullOwning<A,B> {
    pub(in crate) fn new() -> Self {
        FullOwning {
            _marker_a: PhantomData::default(),
            _marker_b: PhantomData::default(),
        }
    }
}

impl<A : Component,B : Component> Into<Group> for FullOwning<A,B> {
    fn into(self) -> Group {
        Group::FullOwning(FullOwningData {
            length: 0,
            type_a: TypeId::of::<A>(),
            type_b: TypeId::of::<B>()
        })
    }
}
