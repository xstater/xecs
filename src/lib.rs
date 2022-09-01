mod entity;
mod range_set;
mod storage;
mod world;
mod dag;

use std::{
    any::{Any, TypeId},
    num::NonZeroUsize,
};

pub use entity::Entity;
pub use storage::ComponentStorage;
pub use world::World;

/// An id represent an entity, it's just a `NonZeroUsize`
pub type EntityId = NonZeroUsize;

/// An ID allocated by World.  
/// It can be used to indicate the storage of compoonents in `World`
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StorageId {
    /// Rust type
    Rust(TypeId),
    /// Group id
    Group(u64),
    /// Other Type   
    /// Used for FFI type
    Other(u64),
}

impl StorageId {
    /// Get `StorageId` from a rust type
    pub fn from_rust_type<T: 'static>() -> StorageId {
        StorageId::Rust(TypeId::of::<T>())
    }
}

/// Component in XECS is just anything that implements `Send + Sync`
pub trait Component: Send + Sync + 'static {}
impl<T> Component for T where T: Send + Sync + 'static {}

/// A combined trait with `Component` and `Any`
pub trait ComponentAny: Component + Any {}
impl<T> ComponentAny for T where T: Component + Any {}
