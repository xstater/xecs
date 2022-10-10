use crate::EntityId;

use super::Storage;

pub struct FullOwning<A, B> {
    len: usize,
    storage_a: A,
    storage_b: B,
}

impl<A, B> FullOwning<A, B>
where
    A: Storage,
    B: Storage,
{
    /// Move 2 items to group and return their index in group
    /// # Returns
    /// Return Some(index) if move successfully or they are already in group
    fn move_to_group(&mut self, entity_id: EntityId) -> Option<usize> {
        if let Some(index_a) = self.storage_a.get_index(entity_id) &&
           let Some(index_b) = self.storage_b.get_index(entity_id) {
            if index_a == index_b && index_a < self.len {
                // already in group
                return Some(index_a);
            }

            let index = self.len;
            self.len += 1;

            // # Safety
            // index_a and index_b are from `get_index`, so it always in range
            // len is always less than the `min(storage_a.len, storage_b.len)`
            unsafe {
                self.storage_a.swap_by_index_unchecked(index_a, index);
                self.storage_b.swap_by_index_unchecked(index_b, index);
            }

            Some(index)
        } else {
            None
        }
    }

    /// Move items out from group and return its index
    /// # Returns
    /// Return None if they are not in group
    fn move_out_from_group(&mut self, index: usize) -> Option<usize> {
        if index < self.len {
            // # No Panics
            // never overflow here, because when `self.len == 0` will never enter this branch
            self.len -= 1;

            // # Safety
            // index_a and index_b are from `get_index`, so it always in range
            // len is always less than the `min(storage_a.len, storage_b.len)`
            unsafe {
                self.storage_a.swap_by_index_unchecked(index, self.len);
                self.storage_b.swap_by_index_unchecked(index, self.len);
            }

            Some(self.len)
        } else {
            None
        }
    }
}

impl<A, B> Storage for FullOwning<A, B>
where
    A: Storage,
    B: Storage,
{
    type Item = (<A as Storage>::Item, <B as Storage>::Item);

    fn len(&self) -> usize {
        self.len
    }

    fn contains(&self, entity_id: EntityId) -> bool {
        self.get_index(entity_id).is_some()
    }

    fn get_index(&self, entity_id: EntityId) -> Option<usize> {
        self.storage_a
            .get_index(entity_id)
            .filter(|index| *index < self.len)
    }

    fn get_id(&self, index: usize) -> Option<EntityId> {
        if index < self.len {
            self.storage_a.get_id(index)
        } else {
            None
        }
    }

    fn remove_by_id(&mut self, entity_id: EntityId) -> Option<Self::Item> {
        let index = self.get_index(entity_id)?;
        self.remove_by_index(index)
    }

    fn remove_by_index(&mut self, index: usize) -> Option<Self::Item> {
        let index = self.move_out_from_group(index)?;
        let a = self.storage_a.remove_by_index(index)?;
        let b = self.storage_b.remove_by_index(index)?;
        Some((a, b))
    }

    fn swap_by_index(&mut self, index_a: usize, index_b: usize) {
        if index_a >= self.len || index_b >= self.len {
            panic!("Cannot swap because index out of range");
        }

        self.storage_a.swap_by_index(index_a, index_b);
        self.storage_b.swap_by_index(index_a, index_b);
    }

    fn swap_by_id(&mut self, id_a: EntityId, id_b: EntityId) {
        if let Some(index_a) = self.get_index(id_a) &&
           let Some(index_b) = self.get_index(id_b) {

            self.swap_by_index(index_a, index_b)
        } else {
            panic!("Cannot swap because id is not in group")
        }
    }

    fn insert(&mut self, id: EntityId, data: Self::Item){
        let (a,b) = data;
        self.storage_a.insert(id,a);
        self.storage_b.insert(id,b);
        self.move_to_group(id);
    }

    fn insert_batch(&mut self, ids: Vec<EntityId>, data: Vec<Self::Item>) {
        let (mut a,mut b) :(Vec<_>,Vec<_>) = data.into_iter().unzip();
        let mut ids_a = ids.clone();
        let mut ids_b = ids;

        
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
