use std::ops::{Deref, DerefMut};
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};
use crate::ComponentStorage;


/// A Read lock gurad for Component Storage
pub struct StorageRead<'a> {
    lock: RwLockReadGuard<'a, Box<dyn ComponentStorage>>,
}

impl<'a> StorageRead<'a> {
    pub(crate) fn from_gurad(lock: RwLockReadGuard<'a, Box<dyn ComponentStorage>>) -> Self {
        StorageRead { lock }
    }
}

impl Deref for StorageRead<'_> {
    type Target = Box<dyn ComponentStorage>;

    fn deref(&self) -> &Self::Target {
        RwLockReadGuard::deref(&self.lock)
    }
}

/// A Write lock gurad for Component Storage
pub struct StorageWrite<'a> {
    lock: RwLockWriteGuard<'a, Box<dyn ComponentStorage>>,
}

impl<'a> StorageWrite<'a> {
    pub(crate) fn from_gurad(lock: RwLockWriteGuard<'a, Box<dyn ComponentStorage>>) -> Self {
        StorageWrite { lock }
    }
}

impl Deref for StorageWrite<'_> {
    type Target = Box<dyn ComponentStorage>;

    fn deref(&self) -> &Self::Target {
        RwLockWriteGuard::deref(&self.lock)
    }
}

impl DerefMut for StorageWrite<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        RwLockWriteGuard::deref_mut(&mut self.lock)
    }
}