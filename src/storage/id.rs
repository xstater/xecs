use std::any::TypeId;

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
