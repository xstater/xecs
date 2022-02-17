use std::{any::TypeId, marker::PhantomData};
use crate::{component::{Component, ComponentStorage}, entity::EntityId, sparse_set::SparseSet};
use super::{Group, GroupType, NonOwningGroup};

mod query;
pub use query::{
    IterRefRef,
    IterRefMut,
    IterMutRef,
    IterMutMut
};

pub struct NonOwning<A,B>{
    sparse_set : SparseSet<EntityId,(usize,usize)>,
    _marker_a : PhantomData<A>,
    _marker_b : PhantomData<B>
}

impl<A : Component,B : Component> NonOwning<A,B> {
    pub(in crate) fn new() -> Self {
        NonOwning {
            sparse_set : SparseSet::new(),
            _marker_a : PhantomData::default(),
            _marker_b : PhantomData::default()
        }
    }
}

impl<A : Component,B : Component> Group for NonOwning<A,B> {
    fn len(&self) -> usize {
        self.sparse_set.len()
    }

    fn group_type(&self) -> GroupType {
        GroupType::NonOwning(
            TypeId::of::<A>(),
            TypeId::of::<B>())
    }

    fn in_group(&self,
                id : EntityId,
                comp_a : &Box<dyn ComponentStorage>,
                comp_b : &Box<dyn ComponentStorage>) -> bool {
        if !self.in_components(id,comp_a,comp_b) {
            return false;
        }

        self.sparse_set.exist(id)
    }
}


impl<A : Component,B : Component> NonOwningGroup for NonOwning<A,B> {
    fn add(&mut self,
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

    fn remove(&mut self,
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

    fn make(&mut self,
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

