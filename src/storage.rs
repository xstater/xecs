mod guards;

pub use guards::{
    StorageRead,
    StorageWrite
};
use std::{
    any::{type_name, Any, TypeId},
    ops:: Range,
};
use xsparseset::{SparseSet, SparseStorage};
use crate::{Component, ComponentAny, EntityId};

/// A trait to make sparse set dynamic
pub trait ComponentStorage: Send + Sync {
    /// Check if storage has ```entity_id```
    fn has(&self, entity_id: EntityId) -> bool;
    /// Get the raw index from ```entity_id``` in storage
    fn index(&self, entity_id: EntityId) -> Option<usize>;
    /// Get the Id from ```index``` in storage
    fn id(&self, index: usize) -> Option<EntityId>;
    /// Remove entity by ```entity_id```
    fn delete(&mut self, entity_id: EntityId);
    /// Swap two items by their indices
    fn swap(&mut self, index_a: usize, index_b: usize);
    /// Get how many item in storage
    fn count(&self) -> usize;
    /// Check if storage is empty
    fn is_empty(&self) -> bool {
        self.count() == 0
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
    fn has(&self, entity_id: EntityId) -> bool {
        self.contains(entity_id)
    }

    fn index(&self, entity_id: EntityId) -> Option<usize> {
        self.get_index(entity_id)
    }

    fn id(&self, index: usize) -> Option<EntityId> {
        self.get_id(index)
    }

    fn delete(&mut self, entity_id: EntityId) {
        self.remove(entity_id);
    }

    fn swap(&mut self, index_a: usize, index_b: usize) {
        self.swap_by_index(index_a, index_b)
    }

    fn count(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        Self::is_empty(&self)
    }

    fn insert_any(&mut self, entity_id: EntityId, data: Box<dyn ComponentAny>) {
        let type_id = TypeId::of::<T>();
        if data.type_id() != type_id {
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

pub trait ComponentStorageStatic: ComponentStorage {
    type Component: Component;

    fn get(&self, entity_id: EntityId) -> &Self::Component;
    fn get_mut(&mut self, entity_id: EntityId) -> &mut Self::Component;
    fn data(&self) -> &[Self::Component];
    fn data_mut(&mut self) -> &mut [Self::Component];
    fn ids(&self) -> &[EntityId];
    fn insert(&mut self, entity_id: EntityId, data: Self::Component);
    fn insert_batch(&mut self,entity_ids: Range<EntityId>, data: Vec<Self::Component>);
    fn remove(&mut self, entity_id: EntityId) -> Option<Self::Component>;
}
