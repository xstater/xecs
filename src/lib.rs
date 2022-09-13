mod entity;
mod range_set;
mod storage;
mod world;

use std::{any::Any, num::NonZeroUsize};

pub use entity::Entity;
pub use storage::{ComponentTypeId, StorageId};
pub use world::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum GroupType {
    Full,
    Partial,
    Non
}

/// An id represent an entity, it's just a `NonZeroUsize`
pub type EntityId = NonZeroUsize;

/// Component in XECS is just anything that implements `Send + Sync`
pub trait Component: Send + Sync + 'static {}
impl<T> Component for T where T: Send + Sync + 'static {}

/// A combined trait with `Component` and `Any`
pub trait ComponentAny: Component + Any {}
impl<T> ComponentAny for T where T: Component + Any {}
