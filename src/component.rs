use std::any::{TypeId, Any};


/// Component in XECS is just anything that implements `Send + Sync`
pub trait Component: Send + Sync + 'static {}
impl<T> Component for T where T: Send + Sync + 'static {}

/// A combined trait with `Component` and `Any`
pub trait ComponentAny: Component + Any {}
impl<T> ComponentAny for T where T: Component + Any {}

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