use crate::{Component, World, EntityId};
use std::marker::PhantomData;
use std::cell::{Ref, RefMut};
use xsparseset::SparseSet;
use crate::query::{add_ptr, add_mut_ptr, distance_ptr, distance_mut_ptr};

pub struct Query2<'a,A : Component,B : Component >{
    pub(in crate::query) world : &'a mut World,
    pub(in crate::query) _marker : PhantomData<(A,B)>
}

pub struct QueryEntity2<'a,A : Component,B : Component>{
    pub(in crate::query) world : &'a mut World,
    pub(in crate::query) _marker : PhantomData<(A,B)>
}

enum GroupInfo {
    A,
    B,
    Grouped(usize) // length of group
}

pub struct Iter<'a,A,B> {
    data_a_ptr : (*const A,*const A),
    data_b_ptr : (*const B,*const B),
    group_info : GroupInfo,
    set_a : Ref<'a,SparseSet<EntityId,A>>,
    set_b : Ref<'a,SparseSet<EntityId,B>>,
}

pub struct IterMut<'a,A,B> {
    data_a_ptr : (*mut A,*mut A),
    data_b_ptr : (*mut B,*mut B),
    group_info : GroupInfo,
    set_a : RefMut<'a,SparseSet<EntityId,A>>,
    set_b : RefMut<'a,SparseSet<EntityId,B>>,
}
pub struct EntityIter<'a,A,B> {
    data_a_ptr : (*const A,*const A),
    data_b_ptr : (*const B,*const B),
    group_info : GroupInfo,
    set_a : Ref<'a,SparseSet<EntityId,A>>,
    set_b : Ref<'a,SparseSet<EntityId,B>>,
}

pub struct EntityIterMut<'a,A,B> {
    data_a_ptr : (*mut A,*mut A),
    data_b_ptr : (*mut B,*mut B),
    group_info : GroupInfo,
    set_a : RefMut<'a,SparseSet<EntityId,A>>,
    set_b : RefMut<'a,SparseSet<EntityId,B>>,
}
impl<'a,A,B> Query2<'a,A,B>
    where A : Component,
          B : Component{

    pub fn query(self) -> Iter<'a,A,B> {
        let set_a = self.world.components::<A>().unwrap();
        let set_b = self.world.components::<B>().unwrap();
        if let Some(group) = self.world.group::<A,B>() {
            Iter{
                data_a_ptr: {
                    let ptr = unsafe { add_ptr(set_a.data().as_ptr(),group.range.start) };
                    (ptr,ptr)
                },
                data_b_ptr: {
                    let ptr = unsafe { add_ptr(set_b.data().as_ptr(),group.range.start) };
                    (ptr,ptr)
                },
                group_info : GroupInfo::Grouped(group.range.len()),
                set_a,
                set_b
            }
        }else{
            Iter{
                data_a_ptr: (set_a.data().as_ptr(), set_a.data().as_ptr()),
                data_b_ptr: (set_b.data().as_ptr(), set_b.data().as_ptr()),
                group_info : if set_a.len() < set_b.len() {
                    GroupInfo::A
                }else{
                    GroupInfo::B
                },
                set_a,
                set_b
            }
        }
    }

    pub fn query_mut(self) -> IterMut<'a,A,B> {
        let mut set_a = self.world.components_mut::<A>().unwrap();
        let mut set_b = self.world.components_mut::<B>().unwrap();
        if let Some(group) = self.world.group::<A,B>() {
            IterMut{
                data_a_ptr: {
                    let ptr = unsafe { add_mut_ptr(set_a.data_mut().as_mut_ptr(),group.range.start) };
                    (ptr,ptr)
                },
                data_b_ptr: {
                    let ptr = unsafe { add_mut_ptr(set_b.data_mut().as_mut_ptr(),group.range.start) };
                    (ptr,ptr)
                },
                group_info : GroupInfo::Grouped(group.range.len()),
                set_a,
                set_b
            }
        }else{
            IterMut{
                data_a_ptr: (set_a.data_mut().as_mut_ptr(), set_a.data_mut().as_mut_ptr()),
                data_b_ptr: (set_b.data_mut().as_mut_ptr(), set_b.data_mut().as_mut_ptr()),
                group_info : if set_a.len() < set_b.len() {
                    GroupInfo::A
                }else{
                    GroupInfo::B
                },
                set_a,
                set_b
            }
        }
    }

    pub fn entities(self) -> QueryEntity2<'a,A,B> {
        QueryEntity2{
            world: self.world,
            _marker: Default::default()
        }
    }
}

impl<'a,A,B> QueryEntity2<'a,A,B>
    where A : Component,
          B : Component{

    pub fn query(self) -> EntityIter<'a,A,B> {
        let set_a = self.world.components::<A>().unwrap();
        let set_b = self.world.components::<B>().unwrap();
        if let Some(group) = self.world.group::<A,B>() {
            EntityIter{
                data_a_ptr: {
                    let ptr = unsafe { add_ptr(set_a.data().as_ptr(),group.range.start) };
                    (ptr,ptr)
                },
                data_b_ptr: {
                    let ptr = unsafe { add_ptr(set_b.data().as_ptr(),group.range.start) };
                    (ptr,ptr)
                },
                group_info : GroupInfo::Grouped(group.range.len()),
                set_a,
                set_b
            }
        }else{
            EntityIter{
                data_a_ptr: (set_a.data().as_ptr(), set_a.data().as_ptr()),
                data_b_ptr: (set_b.data().as_ptr(), set_b.data().as_ptr()),
                group_info : if set_a.len() < set_b.len() {
                    GroupInfo::A
                }else{
                    GroupInfo::B
                },
                set_a,
                set_b
            }
        }
    }

    pub fn query_mut(self) -> EntityIterMut<'a,A,B> {
        let mut set_a = self.world.components_mut::<A>().unwrap();
        let mut set_b = self.world.components_mut::<B>().unwrap();
        if let Some(group) = self.world.group::<A,B>() {
            EntityIterMut{
                data_a_ptr: {
                    let ptr = unsafe { add_mut_ptr(set_a.data_mut().as_mut_ptr(),group.range.start) };
                    (ptr,ptr)
                },
                data_b_ptr: {
                    let ptr = unsafe { add_mut_ptr(set_b.data_mut().as_mut_ptr(),group.range.start) };
                    (ptr,ptr)
                },
                group_info : GroupInfo::Grouped(group.range.len()),
                set_a,
                set_b
            }
        }else{
            EntityIterMut{
                data_a_ptr: (set_a.data_mut().as_mut_ptr(), set_a.data_mut().as_mut_ptr()),
                data_b_ptr: (set_b.data_mut().as_mut_ptr(), set_b.data_mut().as_mut_ptr()),
                group_info : if set_a.len() < set_b.len() {
                    GroupInfo::A
                }else{
                    GroupInfo::B
                },
                set_a,
                set_b
            }
        }
    }
}

impl<'a,A,B> Iterator for Iter<'a,A,B> {
    type Item = (&'a A,&'a B);

    fn next(&mut self) -> Option<Self::Item> {
        match &self.group_info {
            GroupInfo::A => {
                let index_a = unsafe { distance_ptr(self.data_a_ptr.0,self.data_a_ptr.1) } as usize;
                if index_a < self.set_a.len() {
                    let ptr_a = self.data_a_ptr.1;
                    let entity_id = self.set_a.entities()[index_a];
                    self.data_a_ptr.1 = unsafe { add_ptr(self.data_a_ptr.1,1) };
                    return if let Some(index_b) = self.set_b.get_index(entity_id) {
                        self.data_b_ptr.1 = unsafe { add_ptr(self.data_b_ptr.0,index_b) };
                        Some((unsafe { &*ptr_a }, unsafe { &*self.data_b_ptr.1 }))
                    } else {
                        self.next()
                    }
                }
                None
            }
            GroupInfo::B => {
                let index_b = unsafe { distance_ptr(self.data_b_ptr.0,self.data_b_ptr.1) } as usize;
                if index_b < self.set_b.len() {
                    let ptr_b = self.data_b_ptr.1;
                    let entity_id = self.set_b.entities()[index_b];
                    self.data_b_ptr.1 = unsafe { add_ptr(self.data_b_ptr.1,1) };
                    return if let Some(index_a) = self.set_a.get_index(entity_id) {
                        self.data_a_ptr.1 = unsafe { add_ptr(self.data_a_ptr.0,index_a) };
                        Some((unsafe{&*self.data_a_ptr.1},unsafe{&*ptr_b}))
                    } else {
                        self.next()
                    }
                }
                None
            }
            GroupInfo::Grouped(group_len) => {
                let index = unsafe { distance_ptr(self.data_a_ptr.0,self.data_a_ptr.1) } as usize;
                if index < *group_len {
                    let ptr_a = self.data_a_ptr.1;
                    let ptr_b = self.data_b_ptr.1;
                    self.data_a_ptr.1 = unsafe { add_ptr(self.data_a_ptr.1,1) };
                    self.data_b_ptr.1 = unsafe { add_ptr(self.data_b_ptr.1,1) };
                    Some((unsafe{&*ptr_a},unsafe{&*ptr_b}))
                }else{
                    None
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.group_info {
            GroupInfo::A => (0,Some(self.set_a.len())),
            GroupInfo::B => (0,Some(self.set_b.len())),
            GroupInfo::Grouped(group_len) => (0,Some(*group_len))
        }
    }
}

impl<'a,A,B> ExactSizeIterator for Iter<'a,A,B> {}

impl<'a,A,B> Iterator for IterMut<'a,A,B> {
    type Item = (&'a mut A,&'a mut B);

    fn next(&mut self) -> Option<Self::Item> {
        match &self.group_info {
            GroupInfo::A => {
                let index_a = unsafe{ self.data_a_ptr.1.offset_from(self.data_a_ptr.0) } as usize;
                if index_a < self.set_a.len() {
                    let ptr_a = self.data_a_ptr.1;
                    let entity_id = self.set_a.entities()[index_a];
                    self.data_a_ptr.1 = unsafe { self.data_a_ptr.1.offset(1) };
                    return if let Some(index_b) = self.set_b.get_index(entity_id) {
                        self.data_b_ptr.1 = unsafe { self.data_b_ptr.0.offset(index_b as isize) };
                        Some((unsafe { &mut *ptr_a }, unsafe { &mut *self.data_b_ptr.1 }))
                    } else {
                        self.next()
                    }
                }
                None
            }
            GroupInfo::B => {
                let index_b = unsafe{ self.data_b_ptr.1.offset_from(self.data_b_ptr.0) } as usize;
                if index_b < self.set_b.len() {
                    let ptr_b = self.data_b_ptr.1;
                    let entity_id = self.set_b.entities()[index_b];
                    self.data_b_ptr.1 = unsafe { self.data_b_ptr.1.offset(1) };
                    return if let Some(index_a) = self.set_a.get_index(entity_id) {
                        self.data_a_ptr.1 = unsafe { self.data_a_ptr.0.offset(index_a as isize) };
                        Some((unsafe{&mut *self.data_a_ptr.1},unsafe{&mut *ptr_b}))
                    } else {
                        self.next()
                    }
                }
                None
            }
            GroupInfo::Grouped(group_len) => {
                let index = unsafe { self.data_a_ptr.1.offset_from(self.data_a_ptr.0) } as usize;
                if index < *group_len {
                    let ptr_a = self.data_a_ptr.1;
                    let ptr_b = self.data_b_ptr.1;
                    self.data_a_ptr.1 = unsafe { self.data_a_ptr.1.offset(1) };
                    self.data_b_ptr.1 = unsafe { self.data_b_ptr.1.offset(1) };
                    Some((unsafe{&mut *ptr_a},unsafe{&mut *ptr_b}))
                }else{
                    None
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.group_info {
            GroupInfo::A => (0,Some(self.set_a.len())),
            GroupInfo::B => (0,Some(self.set_b.len())),
            GroupInfo::Grouped(group_len) => (0,Some(*group_len))
        }
    }
}

impl<'a,A,B> ExactSizeIterator for IterMut<'a,A,B>{}

impl<'a,A,B> Iterator for EntityIter<'a,A,B> {
    type Item = (EntityId,&'a A,&'a B);

    fn next(&mut self) -> Option<Self::Item> {
        match &self.group_info {
            GroupInfo::A => {
                let index_a = unsafe { distance_ptr(self.data_a_ptr.0,self.data_a_ptr.1) } as usize;
                if index_a < self.set_a.len() {
                    let ptr_a = self.data_a_ptr.1;
                    let entity_id = self.set_a.entities()[index_a];
                    self.data_a_ptr.1 = unsafe { add_ptr(self.data_a_ptr.1,1) };
                    return if let Some(index_b) = self.set_b.get_index(entity_id) {
                        self.data_b_ptr.1 = unsafe { add_ptr(self.data_b_ptr.0,index_b) };
                        Some((entity_id,unsafe { &*ptr_a }, unsafe { &*self.data_b_ptr.1 }))
                    } else {
                        self.next()
                    }
                }
                None
            }
            GroupInfo::B => {
                let index_b = unsafe { distance_ptr(self.data_b_ptr.0,self.data_b_ptr.1) } as usize;
                if index_b < self.set_b.len() {
                    let ptr_b = self.data_b_ptr.1;
                    let entity_id = self.set_b.entities()[index_b];
                    self.data_b_ptr.1 = unsafe { add_ptr(self.data_b_ptr.1,1) };
                    return if let Some(index_a) = self.set_a.get_index(entity_id) {
                        self.data_a_ptr.1 = unsafe { add_ptr(self.data_a_ptr.0,index_a) };
                        Some((entity_id,unsafe{&*self.data_a_ptr.1},unsafe{&*ptr_b}))
                    } else {
                        self.next()
                    }
                }
                None
            }
            GroupInfo::Grouped(group_len) => {
                let index = unsafe { distance_ptr(self.data_a_ptr.0,self.data_a_ptr.1) } as usize;
                if index < *group_len {
                    let entity_id = self.set_a.entities()[index];
                    let ptr_a = self.data_a_ptr.1;
                    let ptr_b = self.data_b_ptr.1;
                    self.data_a_ptr.1 = unsafe { add_ptr(self.data_a_ptr.1,1) };
                    self.data_b_ptr.1 = unsafe { add_ptr(self.data_b_ptr.1,1) };
                    Some((entity_id,unsafe{&*ptr_a},unsafe{&*ptr_b}))
                }else{
                    None
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.group_info {
            GroupInfo::A => (0, Some(self.set_a.len())),
            GroupInfo::B => (0, Some(self.set_b.len())),
            GroupInfo::Grouped(group_len) => (0, Some(*group_len))
        }
    }
}

impl<'a,A,B> ExactSizeIterator for EntityIter<'a,A,B>{}

impl<'a,A,B> Iterator for EntityIterMut<'a,A,B> {
    type Item = (EntityId,&'a A,&'a B);

    fn next(&mut self) -> Option<Self::Item> {
        match &self.group_info {
            GroupInfo::A => {
                let index_a = unsafe { distance_mut_ptr(self.data_a_ptr.0,self.data_a_ptr.1) } as usize;
                if index_a < self.set_a.len() {
                    let ptr_a = self.data_a_ptr.1;
                    let entity_id = self.set_a.entities()[index_a];
                    self.data_a_ptr.1 = unsafe { add_mut_ptr(self.data_a_ptr.1,1) };
                    return if let Some(index_b) = self.set_b.get_index(entity_id) {
                        self.data_b_ptr.1 = unsafe { add_mut_ptr(self.data_b_ptr.0,index_b) };
                        Some((entity_id,unsafe { &mut *ptr_a }, unsafe { &mut *self.data_b_ptr.1 }))
                    } else {
                        self.next()
                    }
                }
                None
            }
            GroupInfo::B => {
                let index_b = unsafe { distance_mut_ptr(self.data_b_ptr.0,self.data_b_ptr.1) } as usize;
                if index_b < self.set_b.len() {
                    let ptr_b = self.data_b_ptr.1;
                    let entity_id = self.set_b.entities()[index_b];
                    self.data_b_ptr.1 = unsafe { add_mut_ptr(self.data_b_ptr.1,1) };
                    return if let Some(index_a) = self.set_a.get_index(entity_id) {
                        self.data_a_ptr.1 = unsafe { add_mut_ptr(self.data_a_ptr.0,index_a) };
                        Some((entity_id,unsafe{&mut *self.data_a_ptr.1},unsafe{&mut *ptr_b}))
                    } else {
                        self.next()
                    }
                }
                None
            }
            GroupInfo::Grouped(group_len) => {
                let index = unsafe { distance_mut_ptr(self.data_a_ptr.0,self.data_a_ptr.1) } as usize;
                if index < *group_len {
                    let entity_id = self.set_a.entities()[index];
                    let ptr_a = self.data_a_ptr.1;
                    let ptr_b = self.data_b_ptr.1;
                    self.data_a_ptr.1 = unsafe { add_mut_ptr(self.data_a_ptr.1,1) };
                    self.data_b_ptr.1 = unsafe { add_mut_ptr(self.data_b_ptr.1,1) };
                    Some((entity_id,unsafe{&*ptr_a},unsafe{&*ptr_b}))
                }else{
                    None
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match &self.group_info {
            GroupInfo::A => (0, Some(self.set_a.len())),
            GroupInfo::B => (0, Some(self.set_b.len())),
            GroupInfo::Grouped(group_len) => (0, Some(*group_len))
        }
    }
}

impl<'a,A,B> ExactSizeIterator for EntityIterMut<'a,A,B>{}
