use std::any::TypeId;

pub trait Component : Send + Sync + 'static{}

impl<T> Component for T where T: Send + Sync + 'static {}

pub trait Storage {
    /// Get the type of data stored in Storage
    fn type_id(&self) -> TypeId;
}