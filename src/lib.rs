mod entity;
mod range_set;
mod storage;
mod world;
mod dag;

use std::{
    any::Any,
    num::NonZeroUsize,
};

pub use entity::Entity;
pub use storage::ComponentStorage;
pub use world::World;
pub use storage::StorageId;

/// An id represent an entity, it's just a `NonZeroUsize`
pub type EntityId = NonZeroUsize;

/// Component in XECS is just anything that implements `Send + Sync`
pub trait Component: Send + Sync + 'static {}
impl<T> Component for T where T: Send + Sync + 'static {}

/// A combined trait with `Component` and `Any`
pub trait ComponentAny: Component + Any {}
impl<T> ComponentAny for T where T: Component + Any {}
