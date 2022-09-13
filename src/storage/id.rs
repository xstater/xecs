use std::any::TypeId;

use crate::Component;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InnerStorageId{
    Group(u32),
    Storage(ComponentTypeId)
}

/// An ID allocated by World.  
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ComponentTypeId {
    /// Rust type
    Rust(TypeId),
    /// Other Type   
    /// Used for FFI type
    Other(u64),
}

impl ComponentTypeId {
    /// Get `StorageId` from a rust type
    pub fn from_rust_type<T: Component>() -> ComponentTypeId {
        ComponentTypeId::Rust(TypeId::of::<T>())
    }

    /// Check it is rust type
    pub fn is_rust_type(&self) -> bool {
        if let ComponentTypeId::Rust(_) = self {
            true
        } else {
            false
        }
    }

    /// Check it is other type
    pub fn is_other_type(&self) -> bool {
        if let ComponentTypeId::Other(_) = self {
            true
        } else {
            false
        }
    }

    /// Try to convert it to Rust Type Id
    pub fn try_into_rust_type(self) -> Option<TypeId> {
        self.try_into().ok()
    }
}

impl TryInto<TypeId> for ComponentTypeId {
    type Error = ();

    fn try_into(self) -> Result<TypeId, Self::Error> {
        match self {
            ComponentTypeId::Rust(rust_type) => Ok(rust_type),
            ComponentTypeId::Other(_) => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StorageId(InnerStorageId);

impl StorageId {
    /// Check a stroage is component stroage
    pub fn is_component_storage(&self) -> bool{
        match &self.0 {
            InnerStorageId::Group(_) => false,
            InnerStorageId::Storage(_) => true,
        }
    }

    /// Check a stroage is group stroage
    pub fn is_group_storage(&self) -> bool{
        match &self.0 {
            InnerStorageId::Group(_) => true,
            InnerStorageId::Storage(_) => false,
        }
    }
}
