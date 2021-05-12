use crate::{World, Component, EntityId};
use std::marker::PhantomData;
use std::cell::{Ref, RefMut};
use xsparseset::SparseSet;
use crate::query::query2::Query2;

pub struct Query<'a,T : Component>{
    pub(in crate) world : &'a mut World,
    pub(in crate) _marker : PhantomData<T>,
}

pub struct QueryEntity<'a,T : Component> {
    pub(in crate::query) world : &'a mut World,
    pub(in crate::query) _marker : PhantomData<T>,
}

pub struct Iter<'a,T> {
    data_ptr: *const T,
    start_ptr : *const T,
    set : Ref<'a,SparseSet<EntityId,T>>
}
pub struct IterMut<'a,T> {
    data_ptr: *mut T,
    start_ptr : *mut T,
    set : RefMut<'a,SparseSet<EntityId,T>>
}

pub struct EntityIter<'a,T> {
    data_ptr: (*const T,*const T),
    entity_ptr: *const EntityId,
    set : Ref<'a,SparseSet<EntityId,T>>
}
pub struct EntityIterMut<'a,T> {
    data_ptr: (*mut T,*mut T),
    entity_ptr: *const EntityId,
    set : RefMut<'a,SparseSet<EntityId,T>>
}

impl<'a,A : Component> Query<'a,A> {
    pub fn query(self) -> Iter<'a,A>{
        let set = self.world.components::<A>().unwrap();
        Iter{
            data_ptr: set.data().as_ptr(),
            start_ptr:set.data().as_ptr(),
            set
        }
    }
    pub fn query_mut(self) -> IterMut<'a,A> {
        let mut set = self.world.components_mut::<A>().unwrap();
        IterMut{
            data_ptr: set.data_mut().as_mut_ptr(),
            start_ptr: set.data_mut().as_mut_ptr(),
            set
        }
    }

    pub fn entities(self) -> QueryEntity<'a,A> {
        QueryEntity{
            world: self.world,
            _marker: Default::default()
        }
    }

    pub fn with<B:Component>(self) -> Query2<'a,A,B> {
        Query2 {
            world: self.world,
            _marker: Default::default()
        }
    }
}

impl<'a,A : Component> QueryEntity<'a,A> {
    pub fn query(self) -> EntityIter<'a, A> {
        let set = self.world.components::<A>().unwrap();
        EntityIter{
            data_ptr: (set.data().as_ptr() ,set.data().as_ptr()),
            entity_ptr: set.entities().as_ptr(),
            set
        }
    }
    pub fn query_mut(self) -> EntityIterMut<'a, A> {
        let mut set = self.world.components_mut::<A>().unwrap();
        EntityIterMut{
            data_ptr: (set.data_mut().as_mut_ptr(),set.data_mut().as_mut_ptr()),
            entity_ptr : set.entities().as_ptr(),
            set
        }
    }
}

impl<'a,A : Component> Iterator for Iter<'a,A> {
    type Item = &'a A;

    fn next(&mut self) -> Option<Self::Item> {
        let index = unsafe {self.data_ptr.offset_from(self.start_ptr)};
        let index = index.abs() as usize;
        if index < self.set.len() {
            let ptr = self.data_ptr;
            self.data_ptr = unsafe { self.data_ptr.offset(1)};
            Some(unsafe{&*ptr})
        }else{
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0,Some(self.set.len()))
    }
}

impl<'a, A : Component> ExactSizeIterator for Iter<'a, A>{}

impl<'a,A : Component> Iterator for IterMut<'a,A> {
    type Item = &'a mut A;

    fn next(&mut self) -> Option<Self::Item> {
        let index = unsafe {self.data_ptr.offset_from(self.start_ptr)};
        let index = index.abs() as usize;
        if index < self.set.len() {
            let ptr = self.data_ptr;
            self.data_ptr = unsafe { self.data_ptr.offset(1)};
            Some(unsafe{&mut *ptr})
        }else{
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0,Some(self.set.len()))
    }
}

impl<'a,A : Component> ExactSizeIterator for IterMut<'a,A> {}

impl<'a,A : Component> Iterator for EntityIter<'a,A>{
    type Item = (EntityId,&'a A);

    fn next(&mut self) -> Option<Self::Item> {
        let index = unsafe{self.data_ptr.1.offset_from(self.data_ptr.0)};
        let index = index.abs() as usize;
        if index < self.set.len() {
            let eid = unsafe {*self.entity_ptr};
            let ptr = self.data_ptr.1;
            self.entity_ptr = unsafe {self.entity_ptr.offset(1)};
            self.data_ptr.1 = unsafe {self.data_ptr.1.offset(1)};
            Some((eid,unsafe{&*ptr}))
        }else{
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0,Some(self.set.len()))
    }
}

impl<'a,A : Component> Iterator for EntityIterMut<'a,A>{
    type Item = (EntityId,&'a mut A);

    fn next(&mut self) -> Option<Self::Item> {
        let index = unsafe{self.data_ptr.1.offset_from(self.data_ptr.0)};
        let index = index.abs() as usize;
        if index < self.set.len() {
            let eid = unsafe {*self.entity_ptr};
            let ptr = self.data_ptr.1;
            self.entity_ptr = unsafe {self.entity_ptr.offset(1)};
            self.data_ptr.1 = unsafe {self.data_ptr.1.offset(1)};
            Some((eid,unsafe{&mut *ptr}))
        }else{
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0,Some(self.set.len()))
    }
}
