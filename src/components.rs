use std::any::TypeId;

use crate::EntityId;

pub trait Component : Send + Sync + 'static{}

impl<T> Component for T where T: Send + Sync + 'static {}

pub trait Storage {
    /// Get the type of data stored in Storage
    fn type_id(&self) -> TypeId;
    /// Check if storage has ```entity_id```
    fn contains(&self, entity_id: EntityId) -> bool;
    /// Get the raw index from ```entity_id``` in storage
    fn get_index(&self, entity_id: EntityId) -> Option<usize>;
    /// Get the Id from ```index``` in storage
    fn get_id(&self, index: usize) -> Option<EntityId>;
    /// Remove entity by ```entity_id``` and drop the removed data
    fn remove_and_drop(&mut self, entity_id: EntityId);
    /// Remove entity without dropping it
    fn remove_and_forget(&mut self, entity_id: EntityId);
    /// Swap two items by their indices without any check
    /// # Safety
    /// * `index_a` and `index_b` must be in range
    unsafe fn swap_by_index_unchecked(&mut self, index_a: usize, index_b: usize);
    /// Swap two items by their ids
    fn swap_by_id(&mut self, id_a: EntityId, id_b: EntityId);
    /// Insert data which implements `Any` (rust type) in component storage
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
    /// * `T` must have the same type of the storage
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