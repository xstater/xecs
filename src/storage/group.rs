use crate::EntityId;

use super::Storage;


pub struct FullOwningGroup {
    length: usize
}

impl Storage for FullOwningGroup {
    fn len(&self) -> usize {
        self.length
    }

    fn is_empty(&self) -> bool {
        self.length == 0
    }

    fn as_component_storage_ref(&self) -> Option<&dyn super::ComponentStorage> {
        None
    }

    fn as_component_storage_mut(&mut self) -> Option<&mut dyn super::ComponentStorage> {
        None
    }

    fn as_group_storage_ref(&self) -> Option<&dyn GroupStorage> {
        Some(self)
    }

    fn as_group_storage_mut(&mut self) -> Option<&mut dyn GroupStorage> {
        Some(self)
    }
}

pub trait GroupStorage: Storage {
    /// Add an entity to Group
    /// # Remarks
    /// * This just add a record to group and don't do any other things
    fn add_entity(&mut self, entity_id: EntityId, index_a: usize, index_b: usize);
    /// Remove an entity from Group
    /// # Remarks
    /// * This just remove a record from group and don't do any other things
    fn remove_entity(&mut self,entity_id: EntityId);
}

impl GroupStorage for FullOwningGroup {
    fn add_entity(&mut self, _entity_id: EntityId, _index_a: usize, _index_b: usize) {
        self.length += 1;
    }

    fn remove_entity(&mut self,_entity_id: EntityId) {
        self.length -= 1;
    }
}