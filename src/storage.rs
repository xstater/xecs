mod component;
mod group;
mod guards;
mod id;
#[cfg(test)]
mod tests;

pub use component::ComponentStorage;
pub use group::{FullOwningGroup, GroupStorage};
pub use guards::{StorageRead, StorageWrite};
pub use id::{ComponentTypeId, StorageId};

pub trait Storage: Send + Sync {
    /// Get how many item in storage
    fn len(&self) -> usize;
    /// Check if storage is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// downcast to component storage
    fn as_component_storage_ref(&self) -> Option<&dyn ComponentStorage>;
    /// downcast to componentstorage
    fn as_component_storage_mut(&mut self) -> Option<&mut dyn ComponentStorage>;
    /// downcast to group storage
    fn as_group_storage_ref(&self) -> Option<&dyn GroupStorage>;
    /// downcast to component
    fn as_group_storage_mut(&mut self) -> Option<&mut dyn GroupStorage>;
}
