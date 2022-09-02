mod guards;
mod manager;
mod id;
#[cfg(test)]
mod tests;

pub use guards::{StorageRead, StorageWrite};
pub use id::StorageId;

use crate::{Component, ComponentAny, EntityId};
use std::{
    any::{type_name, TypeId},
    ops::Range,
};
use xsparseset::{SparseSet, SparseStorage};

/// A trait to make sparse set dynamic
pub trait ComponentStorage: Send + Sync {
    /// Get the `TypeId` of components in storage
    fn component_type_id(&self) -> TypeId;
    /// Check if storage has ```entity_id```
    fn contains(&self, entity_id: EntityId) -> bool;
    /// Get the raw index from ```entity_id``` in storage
    fn get_index(&self, entity_id: EntityId) -> Option<usize>;
    /// Get the Id from ```index``` in storage
    fn get_id(&self, index: usize) -> Option<EntityId>;
    /// Remove entity by ```entity_id``` and ignored the removed data
    fn remove_ignored(&mut self, entity_id: EntityId);
    /// Remove entity without dropping it
    fn remove_ignored_and_forget(&mut self, entity_id: EntityId);
    /// Swap two items by their indices
    fn swap_by_index(&mut self, index_a: usize, index_b: usize);
    /// Swap two items by their indices
    fn swap_by_id(&mut self, id_a: EntityId, id_b: EntityId);
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
    /// * `data` must have the same type of the storage
    /// * Don't use `data` after this call, Because the ownership of `data` was moved
    unsafe fn insert_any_unchecked(&mut self, entity_id: EntityId, data: *mut u8);
    /// Insert data without any check and don't call drop if replaced
    /// # Safety
    /// * `data` must have the same type of the storage
    /// * Don't use `data` after this call, Because the ownership of `data` was moved
    unsafe fn insert_any_unchecked_and_forget(&mut self, entity_id: EntityId, data: *mut u8);
    /// Insert data batch without any check
    /// # Details
    /// * `data` is a pointer to `Vec<T>`
    /// # Safety
    /// * `data` must have real type `Vec<T>`
    /// * `T` must have the same type of the stroage
    /// * Don't use `data` after this call, Because the ownership of `data` was moved
    /// * `Vec<T>::len() == entity_ids.count()`
    unsafe fn insert_any_batch_unchecked(&mut self, entity_ids: Range<EntityId>, data: *mut u8);
    /// Get the pointer of data by given `entity_id`
    /// # Returns
    /// * Return `Some(v)` if storage contains the `entity_id`, return `None` if not
    /// * `v` is a pointer to data
    fn get_ptr(&self, entity_id: EntityId) -> Option<*const u8>;
    /// Get the mutable pointer of data by given `entity_id`
    /// # Returns
    /// * Return `Some(v)` if storage contains the `entity_id`, return `None` if not
    /// * `v` is a pointer to data
    fn get_mut_ptr(&mut self, entity_id: EntityId) -> Option<*mut u8>;
    /// Get all data
    /// # Returns
    /// * return a pointer to data
    fn data_ptr(&self) -> *const u8;
    /// Get all mutable data
    /// # Returns
    /// * return a pointer to data
    fn data_mut_ptr(&mut self) -> *mut u8;
    /// Get a slice of `EntityId`
    fn ids(&self) -> &[EntityId];
}

impl<T, S> ComponentStorage for SparseSet<EntityId, T, S>
where
    T: Component,
    S: SparseStorage<EntityId = EntityId> + Send + Sync,
{
    fn component_type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn contains(&self, entity_id: EntityId) -> bool {
        SparseSet::contains(self, entity_id)
    }

    fn get_index(&self, entity_id: EntityId) -> Option<usize> {
        SparseSet::get_index(self, entity_id)
    }

    fn get_id(&self, index: usize) -> Option<EntityId> {
        SparseSet::get_id(self, index)
    }

    fn remove_ignored(&mut self, entity_id: EntityId) {
        SparseSet::remove(self, entity_id);
    }

    fn remove_ignored_and_forget(&mut self, entity_id: EntityId) {
        if let Some(data) = SparseSet::remove(self, entity_id) {
            std::mem::forget(data)
        }
    }

    fn swap_by_index(&mut self, index_a: usize, index_b: usize) {
        SparseSet::swap_by_index(self, index_a, index_b)
    }

    fn swap_by_id(&mut self, id_a: EntityId, id_b: EntityId) {
        SparseSet::swap_by_entity_id(self, id_a, id_b)
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

    unsafe fn insert_any_unchecked_and_forget(&mut self, entity_id: EntityId, data: *mut u8) {
        let data = data as *mut T;
        let data = std::ptr::read(data);
        if let Some(replaced) = self.insert(entity_id, data) {
            std::mem::forget(replaced);
        }
    }

    unsafe fn insert_any_batch_unchecked(&mut self, entity_ids: Range<EntityId>, data: *mut u8) {
        let data = data as *mut Vec<T>;
        let mut data = std::ptr::read(data);
        let mut ids = (entity_ids.start.get()..entity_ids.end.get())
            .map(|id| EntityId::new_unchecked(id))
            .collect::<Vec<_>>();
        self.insert_batch(&mut ids, &mut data);
    }

    fn get_ptr(&self, entity_id: EntityId) -> Option<*const u8> {
        self.get(entity_id).map(|data| data as *const T as *const _)
    }

    fn get_mut_ptr(&mut self, entity_id: EntityId) -> Option<*mut u8> {
        self.get_mut(entity_id).map(|data| data as *mut T as *mut _)
    }

    fn data_ptr(&self) -> *const u8 {
        self.data().as_ptr() as *mut _
    }

    fn data_mut_ptr(&mut self) -> *mut u8 {
        self.data_mut().as_mut_ptr() as *mut _
    }

    fn ids(&self) -> &[EntityId] {
        SparseSet::ids(self)
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

impl dyn ComponentStorage {
    /// Insert a type `T` data into sparse_set
    /// # Returns
    /// * Return `Some(data)` which was replaced if success, `None` if not
    /// # Panics
    /// * Panic if the type of `data` is not same as the type of component type in Storage
    pub fn insert<T: Component>(&mut self, entity_id: EntityId, data: T) -> Option<T> {
        let type_id = TypeId::of::<T>();
        if type_id != self.component_type_id() {
            panic!("Insert data to storage failed. The data type '{}' is not same as type of components in storage",type_name::<T>())
        }
        let result = if let Some(ptr) = self.get_mut_ptr(entity_id) {
            let ptr = ptr as *mut T;
            // # Safety
            // * ptr has the real type `T`, we checked before
            Some(unsafe { std::ptr::read(ptr) })
        } else {
            None
        };
        let mut data = data;
        let ptr = &mut data as *mut T as *mut _;
        // # Safety
        // * ptr has the real type `T`, we checked before
        // * call `forget(data)` after this calling
        unsafe {
            self.insert_any_unchecked_and_forget(entity_id, ptr);
        }
        std::mem::forget(data);
        result
    }

    /// Insert data batch
    /// # Panics
    /// * Panic if the type of `data` is not same as the type of component type in Storage
    /// * Panic if `entity_ids.count() != data.len()`
    pub fn insert_batch<T: Component>(&mut self, entity_ids: Range<EntityId>, data: Vec<T>) {
        let type_id = TypeId::of::<T>();
        if type_id != self.component_type_id() {
            panic!("Insert batch data to storage failed. The data type '{}' is not same as type of components in storage",type_name::<T>());
        }
        let length = entity_ids.end.get() - entity_ids.start.get();
        if data.len() != length {
            panic!("Insert batch data to storage failed. The count of id is '{}' but the `data.len()` is {}. They are not equal",length,data.len());
        }
        let mut data = data;
        let ptr = &mut data as *mut Vec<T> as *mut u8;
        // # Safety
        // * `data` is `Vec<T>`
        // * `data` has type `T`, we checked before
        // * `data.len() == entity_ids.count()`, we checked before
        // * call forget after this call
        unsafe { self.insert_any_batch_unchecked(entity_ids,ptr) }
        std::mem::forget(data)
    }

    /// Remove the data in storage by given `entity_id`
    /// # Returns
    /// * return `Some(data)` when success, return `None` if not
    /// # Panics
    /// * Panic if the type of `data` is not same as the type of component type in Storage
    pub fn remove<T: Component>(&mut self, entity_id: EntityId) -> Option<T> {
        let type_id = TypeId::of::<T>();
        if type_id != self.component_type_id() {
            panic!("Remove data from storage failed. The data type '{}' is not same as type of components in storage",type_name::<T>())
        }
        let result = if let Some(ptr) = self.get_mut_ptr(entity_id) {
            let ptr = ptr as *mut T;
            // # Safety
            // * ptr has the real type `T`, we checked before
            Some(unsafe { std::ptr::read(ptr) })
        } else {
            None
        };
        // we take the ownership before
        // just forget it
        self.remove_ignored_and_forget(entity_id);
        result
    }

    /// Get the all data in storage
    /// # Panics
    /// * Panic if the type of `data` is not same as the type of component type in Storage
    pub fn data<T: Component>(&self) -> &[T] {
        let type_id = TypeId::of::<T>();
        if type_id != self.component_type_id() {
            panic!("Get data from storage failed. The data type '{}' is not same as type of components in storage",type_name::<T>())
        }
        let data = self.data_ptr() as *const T;
        let len = self.len();
        // # Safety
        // * the data has type `T`
        // * the data has length `len`
        unsafe {
            std::slice::from_raw_parts(data,len)
        }
    }

    /// Get the all mutable data in storage
    /// # Panics
    /// * Panic if the type of `data` is not same as the type of component type in Storage
    pub fn data_mut<T: Component>(&mut self) -> &mut [T] {
        let type_id = TypeId::of::<T>();
        if type_id != self.component_type_id() {
            panic!("Get mutable data from storage failed. The data type '{}' is not same as type of components in storage",type_name::<T>())
        }
        let data = self.data_mut_ptr() as *mut T;
        let len = self.len();
        // # Safety
        // * the data has type `T`
        // * the data has length `len`
        unsafe {
            std::slice::from_raw_parts_mut(data,len)
        }
    }

    /// Get the data by given `entity_id`
    /// # Panics
    /// * Panic if the type of `data` is not same as the type of component type in Storage
    pub fn get<T: Component>(&self,entity_id: EntityId) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        if type_id != self.component_type_id() {
            panic!("Get data from storage failed. The data type '{}' is not same as type of components in storage",type_name::<T>())
        }
        self.get_ptr(entity_id)
            .map(|ptr|ptr as *const T)
            // # Safety
            // * `ptr` has type `T`, we checked before
            .map(|ptr| unsafe { &*ptr })
    }

    /// Get the mutable data by given `entity_id`
    /// # Panics
    /// * Panic if the type of `data` is not same as the type of component type in Storage
    pub fn get_mut<T: Component>(&mut self,entity_id: EntityId) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        if type_id != self.component_type_id() {
            panic!("Get data from storage failed. The data type '{}' is not same as type of components in storage",type_name::<T>())
        }
        self.get_mut_ptr(entity_id)
            .map(|ptr|ptr as *mut T)
            // # Safety
            // * `ptr` has type `T`, we checked before
            .map(|ptr| unsafe { &mut *ptr })
    }
}
