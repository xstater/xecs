use crate::EntityId;
/// Component in XECS is just anything that implements `Send + Sync`
pub trait Component: Send + Sync + 'static {}
impl<T> Component for T where T: Send + Sync + 'static {}

pub trait Storage {
    type Item: Component;

    /// Get the count of items in storages
    fn len(&self) -> usize;
    /// Check if storage has ```entity_id```
    fn contains(&self, entity_id: EntityId) -> bool;
    /// Get the raw index from ```entity_id``` in storage
    fn get_index(&self, entity_id: EntityId) -> Option<usize>;
    /// Get the Id from ```index``` in storage
    fn get_id(&self, index: usize) -> Option<EntityId>;
    /// Remove entity by ```entity_id```
    fn remove(&mut self, entity_id: EntityId) -> Option<Self::Item>;
    /// Swap two items by their indices without any check
    /// # Safety
    /// * `index_a` and `index_b` must be in range
    unsafe fn swap_by_index_unchecked(&mut self, index_a: usize, index_b: usize);
    /// Swap two items by their ids
    fn swap_by_id(&mut self, id_a: EntityId, id_b: EntityId);
    /// insert a data to storage
    fn insert(&mut self,id: EntityId, data: Self::Item) -> Option<Self::Item>;
    /// insert a lot of data to storage
    fn insert_batch(&mut self, ids: &[EntityId], data: Vec<Self::Item>);
    /// Get a borrow of data stored in storage by given id
    fn get(&self, id: EntityId) -> Option<&Self::Item>;
    /// Get a borrow of data stored in storage by given id
    fn get_mut(&mut self, id: EntityId) -> Option<&mut Self::Item>;
    /// Get a slice of `EntityId`
    fn ids(&self) -> &[EntityId];
    /// Get a slice of data stored in storage
    fn data(&self) -> &[Self::Item];
}
