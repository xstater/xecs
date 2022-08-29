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

// pub struct StorageRead<'a> {
    
// }