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

    /// Check it is rust type
    pub fn is_rust_type(&self) -> bool {
        if let StorageId::Rust(_) = self {
            true
        } else {
            false
        }
    }

    /// Check it is group storage
    pub fn is_group(&self) -> bool {
        if let StorageId::Group(_) = self {
            true
        } else {
            false
        }
    }

    /// Check it is other type
    pub fn is_other_type(&self) -> bool {
        if let StorageId::Other(_) = self {
            true
        } else {
            false
        }
    }
}
