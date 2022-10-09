use xsparseset::{SparseSet, SparseStorage};

use crate::EntityId;

use super::{Component, Storage};

impl<T, S> Storage for SparseSet<EntityId, T, S>
where
    T: Component,
    S: SparseStorage<EntityId = EntityId>,
{
    type Item = T;

    fn len(&self) -> usize {
        todo!()
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

    fn remove(&mut self, entity_id: EntityId) -> Option<Self::Item> {
        SparseSet::remove(self, entity_id)
    }

    unsafe fn swap_by_index_unchecked(&mut self, index_a: usize, index_b: usize) {
        SparseSet::swap_by_index_unchecked(self, index_a, index_b)
    }

    fn swap_by_id(&mut self, id_a: EntityId, id_b: EntityId) {
        SparseSet::swap_by_entity_id(self, id_a, id_b)
    }

    fn insert(&mut self,id: EntityId, data: Self::Item) -> Option<Self::Item> {
        SparseSet::insert(self, id, data)
    }

    fn insert_batch(&mut self, ids: &[EntityId], data: Vec<Self::Item>) {
        SparseSet::insert_batch(self, ids, data)
    }

    fn get(&self, id: EntityId) -> Option<&Self::Item> {
        todo!()
    }

    fn get_mut(&mut self, id: EntityId) -> Option<&mut Self::Item> {
        todo!()
    }

    fn ids(&self) -> &[EntityId] {
        todo!()
    }

    fn data(&self) -> &[Self::Item] {
        todo!()
    }
}
