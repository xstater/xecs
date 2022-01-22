//! # Component core trait
use crate::{entity::EntityId, sparse_set::SparseSet};

/// Component is just a trait : Send + Sync + 'static.<br>
/// XECS automatically impl this trait for all suitable structs
pub trait Component : Send + Sync + 'static {}
impl<T : Send + Sync + 'static> Component for T{}

pub trait ComponentStorage : Send + Sync{
    fn has(&self,entity_id : EntityId) -> bool;
    fn index(&self,entity_id : EntityId) -> Option<usize>;
    fn id(&self,index : usize) -> Option<EntityId>;
    fn remove(&mut self,entity_id : EntityId);
    fn swap_by_index(&mut self,index_a : usize,index_b : usize);
    fn count(&self) -> usize;
    fn is_empty(&self) -> bool{
        self.count() == 0
    }
}

impl<T : Component> ComponentStorage for SparseSet<EntityId,T>{
    fn has(&self, entity_id: EntityId) -> bool {
        self.exist(entity_id)
    }

    fn index(&self, entity_id: EntityId) -> Option<usize> {
        self.get_index(entity_id)
    }

    fn id(&self, index : usize) -> Option<EntityId> {
        self.entities().get(index).cloned()
    }

    fn remove(&mut self, entity_id: EntityId) {
        self.remove(entity_id).unwrap();
    }

    fn swap_by_index(&mut self, index_a: usize, index_b: usize) {
        self.swap_by_index(index_a,index_b);
    }

    fn count(&self) -> usize {
        self.len()
    }

}

impl dyn 'static + ComponentStorage {
    pub(in crate) unsafe fn downcast_ref<T : ComponentStorage>(&self) -> &T{
        &*(self as *const dyn ComponentStorage as *const T)
    }
    pub(in crate) unsafe fn downcast_mut<T : ComponentStorage>(&mut self) -> &mut T{
        &mut *(self as *mut dyn ComponentStorage as *mut T)
    }
}
