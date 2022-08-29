use std::ops::{Deref, DerefMut};
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use xsparseset::{SparseSet, SparseStorage};
use crate::{Component, EntityId};

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
/// A Read lock gurad for Component Storage
pub struct StorageRead<'a> {
    lock: RwLockReadGuard<'a,Box<dyn ComponentStorage>>
}

impl<'a> StorageRead<'a> {
    pub(in crate) fn from_gurad(lock: RwLockReadGuard<'a,Box<dyn ComponentStorage>>) -> Self{
        StorageRead { lock }
    }
}

impl Deref for StorageRead<'_> {
    type Target = Box<dyn ComponentStorage>;

    fn deref(&self) -> &Self::Target {
        RwLockReadGuard::deref(&self.lock)
    }
}


/// A Write lock gurad for Component Storage
pub struct StorageWrite<'a> {
    lock: RwLockWriteGuard<'a,Box<dyn ComponentStorage>>
}

impl<'a> StorageWrite<'a> {
    pub(in crate) fn from_gurad(lock: RwLockWriteGuard<'a,Box<dyn ComponentStorage>>) -> Self{
        StorageWrite { lock }
    }
}

impl Deref for StorageWrite<'_> {
    type Target = Box<dyn ComponentStorage>;

    fn deref(&self) -> &Self::Target {
        RwLockWriteGuard::deref(&self.lock)
    }
}

impl DerefMut for StorageWrite<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        RwLockWriteGuard::deref_mut(&mut self.lock)
    }
}