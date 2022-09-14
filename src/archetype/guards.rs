use std::ops::{Deref, DerefMut};

use parking_lot::{RwLockReadGuard,RwLockWriteGuard};
use crate::Archetype;

pub struct ArchetypeRead<'a> {
    pub(crate) _lock: RwLockReadGuard<'a,()>,
    pub(crate) archetype:RwLockReadGuard<'a,Archetype>,
}

impl<'a> Deref for ArchetypeRead<'a> {
    type Target = Archetype;

    fn deref(&self) -> &Self::Target {
        &self.archetype
    }
}


pub struct ArchetypeWrite<'a> {
    pub(crate) _lock: RwLockReadGuard<'a,()>,
    pub(crate) archetype: RwLockWriteGuard<'a,Archetype>,
}

impl<'a> Deref for ArchetypeWrite<'a> {
    type Target = Archetype;

    fn deref(&self) -> &Self::Target {
        &self.archetype
    }
}

impl<'a> DerefMut for ArchetypeWrite<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.archetype
    }
}


