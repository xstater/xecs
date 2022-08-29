use std::{fmt::{Debug, Display}, marker::PhantomData, ops::{Deref, DerefMut}};
use parking_lot::{RwLockReadGuard, RwLockWriteGuard};

/// The resource trait 
pub trait Resource : Send + Sync + 'static {}
impl<T : Send + Sync + 'static> Resource for T {}

impl dyn Resource {
    pub(in crate) unsafe fn downcast_ref<T : Resource>(&self) -> &T{
        &*(self as *const dyn Resource as *const T)
    }
    pub(in crate) unsafe fn downcast_mut<T : Resource>(&mut self) -> &mut T{
        &mut *(self as *mut dyn Resource as *mut T)
    }
}

/// A read lock gurad for resource
pub struct ResourceRead<'a,T> {
    lock : RwLockReadGuard<'a,Box<dyn Resource>>,
    _marker : PhantomData<T>
}

impl<'a,T : Resource> ResourceRead<'a,T> {
    pub(in crate) fn new(lock : RwLockReadGuard<'a,Box<dyn Resource>>) -> Self {
        ResourceRead {
            lock,
            _marker : Default::default()
        }
    }
}

impl<'a,T : Resource> Deref for ResourceRead<'a,T> {
    type Target = T;

    fn deref(&self) -> &Self::Target{
        unsafe {
            self.lock .downcast_ref::<T>() // safety: safe because we checked type id outside
        }
    }
}

impl<'a,T : Resource + Debug> Debug for ResourceRead<'a,T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = unsafe {
            self.lock .downcast_ref::<T>() // safety : Safe because we checked type id outside
        };
        f.debug_struct("ResourceRead").field("lock", data).finish()
    }
}

impl<'a,T : Resource + Display> Display for ResourceRead<'a,T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = unsafe {
            self.lock .downcast_ref::<T>()// safety: Safe because we checked type id outside
        };
        data.fmt(f)
    }
} 




/// A write lock gurad for resource
pub struct ResourceWrite<'a,T> {
    lock : RwLockWriteGuard<'a,Box<dyn Resource>>,
    _marker : PhantomData<T>
}


impl<'a,T : Resource> ResourceWrite<'a,T> {
    pub(in crate) fn new(lock : RwLockWriteGuard<'a,Box<dyn Resource>>) -> Self {
        ResourceWrite{
            lock,
            _marker : Default::default()
        }
    }
}

impl<'a,T : Resource> Deref for ResourceWrite<'a,T> {
    type Target = T;

    fn deref(&self) -> &Self::Target{
        unsafe {
            self.lock .downcast_ref::<T>() // safety: safe because we checked type id outside
        }
    }
}

impl<'a,T : Resource> DerefMut for ResourceWrite<'a,T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            self.lock .downcast_mut::<T>() // safety : safe because we checked type id outside
        }
    }
}

impl<'a,T : Resource + Debug> Debug for ResourceWrite<'a,T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = unsafe {
            self.lock .downcast_ref::<T>() // safety : Safe because we checked type id outside
        };
        f.debug_struct("ResourceRead").field("lock", data).finish()
    }
}

impl<'a,T : Resource + Display> Display for ResourceWrite<'a,T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = unsafe {
            self.lock .downcast_ref::<T>()// safety: Safe because we checked type id outside
        };
        data.fmt(f)
    }
} 
