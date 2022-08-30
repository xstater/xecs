mod world;
mod storage;
mod entity;
mod range_set;

use std::{num::NonZeroUsize, any::{TypeId, Any}};

pub use world::World;
pub use storage::ComponentStorage;
pub use entity::Entity;

/// An id represent an entity, it's just a `NonZeroUsize`
pub type EntityId = NonZeroUsize;

/// An ID allocated by World.  
/// It can be used to indicate the storage of compoonents in `World`
#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub enum StorageId {
    /// Rust type
    Rust(TypeId),
    /// Other Type   
    /// Used for FFI type
    Other(u32)
}

/// Component in XECS is just anything that implements `Send + Sync`
pub trait Component: Send + Sync + 'static {}
impl<T> Component for T
where T: Send + Sync + 'static {}


/// A combined trait with `Component` and `Any`
pub trait ComponentAny: Component + Any{}
impl<T> ComponentAny for T
where T: ComponentAny + Any {}
