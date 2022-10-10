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
        SparseSet::len(self)
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

    fn remove_by_id(&mut self, entity_id: EntityId) -> Option<Self::Item> {
        SparseSet::swap_remove_by_id(self, entity_id)
    }

    fn remove_by_index(&mut self, index: usize) -> Option<Self::Item> {
        SparseSet::swap_remove_by_index(self,index)
    }

    fn swap_by_index(&mut self, index_a: usize, index_b: usize) {
        SparseSet::swap_by_index(self, index_a, index_b)
    }

    fn swap_by_id(&mut self, id_a: EntityId, id_b: EntityId) {
        SparseSet::swap_by_entity_id(self, id_a, id_b)
    }

    fn insert(&mut self,id: EntityId, data: Self::Item){
        SparseSet::insert(self, id, data);
    }

    fn insert_batch(&mut self, ids: Vec<EntityId>, data: Vec<Self::Item>) {
        let mut ids = ids;
        let mut data = data;
        SparseSet::insert_batch(self, &mut ids, &mut data)
    }

    fn get(&self, id: EntityId) -> Option<&Self::Item> {
        SparseSet::get(self, id)
    }

    fn get_mut(&mut self, id: EntityId) -> Option<&mut Self::Item> {
        SparseSet::get_mut(self, id)
    }

    fn ids(&self) -> &[EntityId] {
        SparseSet::ids(self)
    }

    fn data(&self) -> &[Self::Item] {
        SparseSet::data(self)
    }
}
