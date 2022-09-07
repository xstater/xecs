mod entity;
mod range_set;
mod world;
mod archetype;
mod dyn_type_vec;
mod component;

use std::num::NonZeroUsize;

pub use entity::Entity;
pub use component::{
    Component,
    ComponentAny,
    ComponentTypeId
};
pub use world::World;
pub use archetype::Archetype;

/// An id represent an entity, it's just a `NonZeroUsize`
pub type EntityId = NonZeroUsize;
