mod guards;
#[cfg(test)]
mod tests;

use crate::{Component, ComponentAny, EntityId};
pub use guards::{StorageRead, StorageWrite};
use std::{
    any::{type_name, TypeId},
    ops::Range,
};
use xsparseset::{SparseSet, SparseStorage};

/// A trait to make sparse set dynamic
pub trait ComponentStorage: Send + Sync {
    /// Check if storage has ```entity_id```
    fn contains(&self, entity_id: EntityId) -> bool;
    /// Get the raw index from ```entity_id``` in storage
    fn get_index(&self, entity_id: EntityId) -> Option<usize>;
    /// Get the Id from ```index``` in storage
    fn get_id(&self, index: usize) -> Option<EntityId>;
    /// Remove entity by ```entity_id```
    fn remove(&mut self, entity_id: EntityId);
    /// Remove entity without dropping it
    fn remove_and_forget(&mut self, entity_id: EntityId);
    /// Swap two items by their indices
    fn swap_by_index(&mut self, index_a: usize, index_b: usize);
    /// Get how many item in storage
    fn len(&self) -> usize;
    /// Check if storage is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Insert data which implements `Any` (rust type) in compoenent storage
    /// # Panics
    /// * This function should panic when downcast data to the type of storage failed
    fn insert_any(&mut self, entity_id: EntityId, data: Box<dyn ComponentAny>);
    /// Insert data without any check, can be used in pass a value on stack or FFI type
    /// # Safety
    /// * data must have the same type of the storage
    unsafe fn insert_any_unchecked(&mut self, entity_id: EntityId, data: *mut u8);
}

impl<T, S> ComponentStorage for SparseSet<EntityId, T, S>
where
    T: Component,
    S: SparseStorage<EntityId = EntityId> + Send + Sync,
{
    fn contains(&self, entity_id: EntityId) -> bool {
        SparseSet::contains(self, entity_id)
    }

    fn get_index(&self, entity_id: EntityId) -> Option<usize> {
        SparseSet::get_index(self, entity_id)
    }

    fn get_id(&self, index: usize) -> Option<EntityId> {
        SparseSet::get_id(self, index)
    }

    fn remove(&mut self, entity_id: EntityId) {
        SparseSet::remove(self, entity_id);
    }

    fn remove_and_forget(&mut self, entity_id: EntityId) {
        if let Some(data) = SparseSet::remove(self, entity_id) {
            std::mem::forget(data)
        }
    }

    fn swap_by_index(&mut self, index_a: usize, index_b: usize) {
        SparseSet::swap_by_index(self, index_a, index_b)
    }

    fn len(&self) -> usize {
        SparseSet::len(self)
    }

    fn is_empty(&self) -> bool {
        SparseSet::is_empty(self)
    }

    fn insert_any(&mut self, entity_id: EntityId, data: Box<dyn ComponentAny>) {
        let type_id = TypeId::of::<T>();
        if (&*data).type_id() != type_id {
            panic!(
                "insert_any() failed, because downcast to {} failed",
                type_name::<T>()
            );
        }
        // # Safety
        // * we checked the type before ,so the casting is safe
        let data = unsafe {
            let ptr = Box::into_raw(data);
            let ptr = ptr as *mut T;
            std::ptr::read(ptr)
        };
        self.insert(entity_id, data);
    }

    unsafe fn insert_any_unchecked(&mut self, entity_id: EntityId, data: *mut u8) {
        let data = data as *mut T;
        let data = std::ptr::read(data);
        self.insert(entity_id, data);
    }
}

impl dyn 'static + ComponentStorage {
    /// Downcast `&dyn ComponentStorage` to `&T`
    /// # Safety
    /// * Safe when `self` has type `T`
    pub unsafe fn downcast_ref<T: ComponentStorage>(&self) -> &T {
        &*(self as *const dyn ComponentStorage as *const T)
    }

    /// Downcast `&mut dyn ComponentStorage` to `&mut T`
    /// # Safety
    /// * Safe when `self` has type `T`
    pub unsafe fn downcast_mut<T: ComponentStorage>(&mut self) -> &mut T {
        &mut *(self as *mut dyn ComponentStorage as *mut T)
    }
}

pub trait ComponentStorageConcrete: ComponentStorage {
    type Component: Component;

    fn get(&self, entity_id: EntityId) -> &Self::Component;
    fn get_mut(&mut self, entity_id: EntityId) -> &mut Self::Component;
    fn data(&self) -> &[Self::Component];
    fn data_mut(&mut self) -> &mut [Self::Component];
    fn ids(&self) -> &[EntityId];
    fn insert(&mut self, entity_id: EntityId, data: Self::Component);
    fn insert_batch(&mut self, entity_ids: Range<EntityId>, data: Vec<Self::Component>);
    fn remove(&mut self, entity_id: EntityId) -> Option<Self::Component>;
}
