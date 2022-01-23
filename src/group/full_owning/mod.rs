use std::{any::TypeId, marker::PhantomData};
use crate::{component::{Component, ComponentStorage}, entity::EntityId, world::World};
use super::Group;

mod query;

pub use query::{
    IterRefRef,
    IterRefMut,
    IterMutRef,
    IterMutMut
};

pub struct FullOwning<A,B>{
    length: usize,
    _marker_a : PhantomData<A>,
    _marker_b : PhantomData<B>
}

impl<A : Component,B : Component> FullOwning<A,B> {
    pub(in crate) fn new() -> Self {
        FullOwning {
            length: 0,
            _marker_a: PhantomData::default(),
            _marker_b: PhantomData::default(),
        }
    }
}

impl<A : Component,B : Component> Group for FullOwning<A,B> {
    fn len(&self) -> usize {
        self.length
    }

    fn type_id_a(&self) -> TypeId {
        TypeId::of::<A>()
    }

    fn type_id_b(&self) -> TypeId {
        TypeId::of::<B>()
    }

    fn owning_types(&self) -> Vec<TypeId> {
        vec![self.type_id_a(),self.type_id_b()]
    }

    fn in_group(&self,
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

    fn add(&mut self, world : &World, id : EntityId) {
        let mut comp_a = self.storage_a_mut(world);
        let mut comp_b = self.storage_b_mut(world);

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

    fn remove(&mut self, world : &World, id : EntityId) {
        let mut comp_a = self.storage_a_mut(world);
        let mut comp_b = self.storage_b_mut(world);

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

    fn make_group_in_world(&mut self, world : &World) {
        self.length = 0;

        let mut comp_a = self.storage_a_mut(world);
        let mut comp_b = self.storage_b_mut(world);

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

