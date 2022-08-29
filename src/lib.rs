mod world;
mod storage;

use std::{num::NonZeroUsize, any::TypeId};

pub use world::World;
pub use storage::ComponentStorage;

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