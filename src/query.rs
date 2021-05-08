use crate::{Component, World};
use std::marker::PhantomData;

pub struct Query<'a,T : Component>{
    world : &'a mut World,
    _marker : PhantomData<T>
}

pub struct QueryWith<'a,T,U> {
    world : &'a mut World,
    _marker1 : PhantomData<T>,
    _marker2 : PhantomData<U>
}

impl<'a,T : Component> Query<'a,T> {
    pub fn from_world(world: &'a mut World) -> Self {
        Query {
            world,
            _marker : Default::default()
        }
    }

    pub fn query(self) -> &'a [T] {
        unimplemented!()
    }

    pub fn query_mut(self) -> &'a mut [T] {
        unimplemented!()
    }

    pub fn with<U : Component>(self) -> QueryWith<'a,T,U>{
        QueryWith{
            world: self.world,
            _marker1 : Default::default(),
            _marker2 : Default::default(),
        }
    }
}
