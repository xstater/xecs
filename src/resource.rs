use std::{any::Any, fmt::{Debug, Display}, marker::PhantomData, ops::{Deref, DerefMut}, sync::{RwLockReadGuard, RwLockWriteGuard}};

pub struct ResourceRead<'a,T> {
    lock : RwLockReadGuard<'a,Option<Box<dyn Any + Send + Sync>>>,
    _marker : PhantomData<T>
}

impl<'a,T : 'static + Send + Sync> ResourceRead<'a,T> {
    pub(in crate) fn from_read(lock : RwLockReadGuard<'a,Option<Box<dyn Any + Send + Sync>>>) -> Self {
        ResourceRead {
            lock,
            _marker : Default::default()
        }
    }
}

impl<'a,T : 'static + Send + Sync> Deref for ResourceRead<'a,T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.lock.as_ref().unwrap() // This unwrap never fails. It's ensured by outside
            .downcast_ref::<T>().unwrap() // also
    }
}

impl<'a,T : 'static + Send + Sync + Debug> Debug for ResourceRead<'a,T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.lock.as_ref().unwrap() // This unwrap never fails. It's ensured by outside
            .downcast_ref::<T>().unwrap();// Also
        f.debug_struct("ResourceRead").field("lock", data).finish()
    }
}

impl<'a,T : 'static + Send + Sync + Display> Display for ResourceRead<'a,T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.lock.as_ref().unwrap() // This unwrap never fails. It's ensured by outside
            .downcast_ref::<T>().unwrap();// Also
        data.fmt(f)
    }
} 




pub struct ResourceWrite<'a,T> {
    lock : RwLockWriteGuard<'a,Option<Box<dyn Any + Send + Sync>>>,
    _marker : PhantomData<T>
}

impl<'a,T : 'static + Send + Sync> ResourceWrite<'a,T> {
    pub(in crate) fn from_write(lock : RwLockWriteGuard<'a,Option<Box<dyn Any + Send + Sync>>>) -> Self {
        ResourceWrite{
            lock,
            _marker : Default::default()
        }
    }
}

impl<'a,T : 'static + Send + Sync> Deref for ResourceWrite<'a,T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.lock.as_ref().unwrap() // This unwrap never fails. It's ensured by outside
            .downcast_ref::<T>().unwrap() // also
    }
}

impl<'a,T : 'static + Send + Sync> DerefMut for ResourceWrite<'a,T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.lock.as_mut().unwrap() // This unwrap never fails. It's ensured by outside
            .downcast_mut::<T>().unwrap() // also
    }
}

impl<'a,T : 'static + Send + Sync + Debug> Debug for ResourceWrite<'a,T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.lock.as_ref().unwrap() // This unwrap never fails. It's ensured by outside
            .downcast_ref::<T>().unwrap();// Also
        f.debug_struct("ResourceRead").field("lock", data).finish()
    }
}

impl<'a,T : 'static + Send + Sync + Display> Display for ResourceWrite<'a,T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = self.lock.as_ref().unwrap() // This unwrap never fails. It's ensured by outside
            .downcast_ref::<T>().unwrap();// Also
        data.fmt(f)
    }
} 
