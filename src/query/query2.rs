use crate::{Component, World, EntityId};
use std::marker::PhantomData;
use std::cell::Ref;
use xsparseset::SparseSet;

pub struct Query2<'a,A : Component,B : Component >{
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

impl<'a,A,B> Query2<'a,A,B>
    where A : Component,
          B : Component{

    pub fn query(self) -> Iter<'a,A,B> {
        let set_a = self.world.components::<A>().unwrap();
        let set_b = self.world.components::<B>().unwrap();
        if let Some(group) = self.world.group::<A,B>() {
            Iter{
                data_a_ptr: {
                    let ptr = unsafe { set_a.data().as_ptr().offset(group.range.start as isize) };
                    (ptr,ptr)
                },
                data_b_ptr: {
                    let ptr = unsafe { set_b.data().as_ptr().offset(group.range.start as isize) };
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
}

impl<'a,A,B> Iterator for Iter<'a,A,B> {
    type Item = (&'a A,&'a B);

    fn next(&mut self) -> Option<Self::Item> {
        match &self.group_info {
            GroupInfo::A => {
                let index_a = unsafe{ self.data_a_ptr.1.offset_from(self.data_a_ptr.0) } as usize;
                if index_a < self.set_a.len() {
                    let ptr_a = self.data_a_ptr.1;
                    let entity_id = self.set_a.entities()[index_a];
                    self.data_a_ptr.1 = unsafe { self.data_a_ptr.1.offset(1) };
                    if let Some(index_b) = self.set_b.get_index(entity_id) {
                        self.data_b_ptr.1 = unsafe{ self.data_b_ptr.0.offset(index_b as isize) };
                        return Some((unsafe{&*ptr_a},unsafe{&*self.data_b_ptr.1}));
                    }
                }
                None
            }
            GroupInfo::B => {None}
            GroupInfo::Grouped(group_len) => {
                let index = unsafe { self.data_a_ptr.1.offset_from(self.data_a_ptr.0) } as usize;
                if index < *group_len {
                    let ptr_a = self.data_a_ptr.1;
                    let ptr_b = self.data_b_ptr.1;
                    self.data_a_ptr.1 = unsafe { self.data_a_ptr.1.offset(1) };
                    self.data_b_ptr.1 = unsafe { self.data_b_ptr.1.offset(1) };
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
