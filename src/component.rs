use std::sync::{RwLockReadGuard, RwLockWriteGuard};

use crate::{entity::EntityId, sparse_set::SparseSet};

/// The Component trait  
pub trait Component : Send + Sync + 'static {}
impl<T : Send + Sync + 'static> Component for T{}

/// A trait to make sparse set dynamic  
pub trait ComponentStorage : Send + Sync{
    /// Check if storage has ```entity_id```
    fn has(&self,entity_id : EntityId) -> bool;
    /// Get the raw index from ```entity_id``` in storage
    fn index(&self,entity_id : EntityId) -> Option<usize>;
    /// Get the Id from ```index``` in storage
    fn id(&self,index : usize) -> Option<EntityId>;
    /// Remove entity by ```entity_id```
    fn remove(&mut self,entity_id : EntityId);
    /// Swap two items by their indices
    fn swap_by_index(&mut self,index_a : usize,index_b : usize);
    /// Get how many item in storage
    fn count(&self) -> usize;
    /// Check if storage is empty
    fn is_empty(&self) -> bool{
        self.count() == 0
    }
}

impl<T : Component> ComponentStorage for SparseSet<EntityId,T>{
    fn has(&self, entity_id: EntityId) -> bool {
        self.exist(entity_id)
    }

    fn index(&self, entity_id: EntityId) -> Option<usize> {
        self.get_index(entity_id)
    }

    fn id(&self, index : usize) -> Option<EntityId> {
        self.entities().get(index).cloned()
    }

    fn remove(&mut self, entity_id: EntityId) {
        self.remove(entity_id).unwrap();
    }

    fn swap_by_index(&mut self, index_a: usize, index_b: usize) {
        self.swap_by_index(index_a,index_b);
    }

    fn count(&self) -> usize {
        self.len()
    }

}

impl dyn 'static + ComponentStorage {
    pub(in crate) unsafe fn downcast_ref<T : ComponentStorage>(&self) -> &T{
        &*(self as *const dyn ComponentStorage as *const T)
    }
    pub(in crate) unsafe fn downcast_mut<T : ComponentStorage>(&mut self) -> &mut T{
        &mut *(self as *mut dyn ComponentStorage as *mut T)
    }
}


pub struct ComponentRead<'a,T>{
    _lock : RwLockReadGuard<'a,Box<dyn ComponentStorage>>,
    ptr : *const SparseSet<EntityId,T>
}

impl<'a,T : Component> ComponentRead<'a,T> {
    pub(in crate) fn from_lock(lock : RwLockReadGuard<'a,Box<dyn ComponentStorage>>) -> Self {
        // Safety:
        // 1.box has type SparseSet<EntityId,T>
        let ptr = unsafe {
            lock.downcast_ref::<SparseSet<EntityId,T>>()
        } as *const _;
        ComponentRead {
            _lock : lock,
            ptr,
        }
    }

    pub fn count(&self) -> usize {
        let sparse_set = unsafe { &*self.ptr };
        sparse_set.len()
    }

    pub fn exist(&self,id : EntityId) -> bool {
        let sparse_set = unsafe { &*self.ptr };
        sparse_set.exist(id)
    }

    pub fn get(&self,id : EntityId) -> Option<&T> {
        let sparse_set = unsafe { &*self.ptr };
        sparse_set.get(id)
    }

    pub unsafe fn get_unchecked(&self,id : EntityId) -> &T {
        let sparse_set = &*self.ptr;
        sparse_set.get_unchecked(id)
    }

    pub fn is_empty(&self) -> bool {
        let sparse_set = unsafe { &*self.ptr };
        sparse_set.is_empty()
    }

    pub fn data(&self) -> &[T] {
        let sparse_set = unsafe { &*self.ptr };
        sparse_set.data()
    }

}




pub struct ComponentWrite<'a,T>{
    _lock : RwLockWriteGuard<'a,Box<dyn ComponentStorage>>,
    ptr : *mut SparseSet<EntityId,T>
}

impl<'a,T : Component> ComponentWrite<'a,T> {
    pub(in crate) fn from_lock(mut lock : RwLockWriteGuard<'a,Box<dyn ComponentStorage>>) -> Self {
        // Safety:
        // 1.box has type SparseSet<EntityId,T>
        let ptr = unsafe {
            lock.downcast_mut::<SparseSet<EntityId,T>>()
        } as *mut _;
        ComponentWrite{
            _lock : lock,
            ptr,
        }
    }

    pub fn count(&self) -> usize {
        let sparse_set = unsafe { &*self.ptr };
        sparse_set.len()
    }

    pub fn exist(&self,id : EntityId) -> bool {
        let sparse_set = unsafe { &*self.ptr };
        sparse_set.exist(id)
    }

    pub fn get(&self,id : EntityId) -> Option<&T> {
        let sparse_set = unsafe { &*self.ptr };
        sparse_set.get(id)
    }

    pub unsafe fn get_unchecked(&self,id : EntityId) -> &T {
        let sparse_set = &*self.ptr;
        sparse_set.get_unchecked(id)
    }

    pub fn is_empty(&self) -> bool {
        let sparse_set = unsafe { &*self.ptr };
        sparse_set.is_empty()
    }

    pub fn get_mut(&mut self,id : EntityId) -> Option<&mut T> {
        let sparse_set = unsafe { &mut *self.ptr };
        sparse_set.get_mut(id)
    }

    pub unsafe fn get_unchecked_mut(&mut self,id : EntityId) -> &mut T {
        let sparse_set = &mut *self.ptr;
        sparse_set.get_unchecked_mut(id)
    }

    pub fn data(&self) -> &[T] {
        let sparse_set = unsafe { &*self.ptr };
        sparse_set.data()
    }

    pub fn data_mut(&mut self) -> &mut [T] {
        let sparse_set = unsafe { &mut *self.ptr };
        sparse_set.data_mut()
    }
}

