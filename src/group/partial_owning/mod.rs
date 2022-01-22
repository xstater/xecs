use std::{any::TypeId, marker::PhantomData};
use crate::{component::{Component, ComponentStorage}, entity::EntityId, world::World};
use super::Group;

mod query;

// Owning A & Non-Owning B
pub struct PartialOwning<A,B> {
    length : usize,
    _marker_a : PhantomData<A>,
    _marker_b : PhantomData<B>
}

impl<A : Component,B : Component> PartialOwning<A,B> {
    pub(in crate) fn new() -> Self {
        PartialOwning {
            length: 0,
            _marker_a: PhantomData::default(),
            _marker_b: PhantomData::default(),
        }
    }
}

impl<A : Component,B : Component> Group for PartialOwning<A,B> {
    fn len(&self) -> usize {
        self.length
    }

    fn type_id_a(&self) -> std::any::TypeId {
        TypeId::of::<A>()
    }

    fn type_id_b(&self) -> TypeId {
        TypeId::of::<B>()
    }

    fn owning_types(&self) -> Vec<TypeId> {
        vec![self.type_id_a()]
    }

    fn in_group(&self,
                id : EntityId,
                comp_a : &Box<dyn ComponentStorage>,
                comp_b : &Box<dyn ComponentStorage>) -> bool {
        if !self.in_components(id,comp_a,comp_b) {
            return false;
        }

        // get index in components storage
        // This unwrap never failed because the in_components() ensure it's already in components
        let index_a = comp_a.index(id).unwrap();

        if index_a < self.length {
            true
        } else {
            false
        }
    }

    fn add(&mut self, world : &World, id : EntityId) {
        let mut comp_a = self.storage_a_mut(world);
        let comp_b = self.storage_b(world);

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

    fn remove(&mut self, world : &World, id : EntityId) {
        let mut comp_a = self.storage_a_mut(world);
        let comp_b = self.storage_b(world);

        if !self.in_group(id,&comp_a,&comp_b) {
            return;
        }

        // Unwrap will never fail
        // because in_group() ensures that id is in comp_a
        let index_a = comp_a.index(id).unwrap();

        self.length -= 1;

        comp_a.swap_by_index(index_a,self.length);
    }

    fn make_group_in_world(&mut self, world : &World) {
        self.length = 0;

        let mut comp_a = self.storage_a_mut(world);
        let comp_b = self.storage_b(world);

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

