use crate::{Component, World, EntityId};
use std::marker::PhantomData;
use crate::query::{QueryWith, QueryEntitiesWith};
use xsparseset::SparseSet;

pub struct Query<'a,T : Component>{
    pub(in crate::query) world : &'a mut World,
    pub(in crate::query) _marker : PhantomData<T>
}

pub struct QueryEntities<'a,T : Component>{
    pub(in crate::query) world : &'a mut World,
    pub(in crate::query) _marker : PhantomData<T>
}

impl<'a,T : Component> Query<'a,T> {
    pub(in crate) fn from_world(world: &'a mut World) -> Self {
        Query {
            world,
            _marker : Default::default()
        }
    }

    pub fn query(self) -> impl Iterator<Item=&'a T>{
        self.world
            .components::<T>()
            .expect(format!("Type {} has not been registered",std::any::type_name::<T>()).as_str())
            .data()
            .iter()
    }

    pub fn query_mut(self) -> impl Iterator<Item=&'a mut T> {
        self.world
            .components_mut::<T>()
            .expect(format!("Type {} has not been registered",std::any::type_name::<T>()).as_str())
            .data_mut()
            .iter_mut()
    }

    pub fn entities(self) -> QueryEntities<'a,T>{
        QueryEntities{
            world: self.world,
            _marker: Default::default()
        }
    }

    pub fn with<U : Component>(self) -> QueryWith<'a,(T,U)>{
        QueryWith{
            world: self.world,
            _marker : Default::default(),
        }
    }
}

impl<'a,T : Component> QueryEntities<'a,T> {
    pub fn query(self) -> impl Iterator<Item=(EntityId,&'a T)>{
        self.world
            .components::<T>()
            .expect(format!("Type {} has not been registered",std::any::type_name::<T>()).as_str())
            .entity_iter()
    }

    pub fn query_mut(self) -> impl Iterator<Item=(EntityId,&'a mut T)> {
        self.world
            .components_mut::<T>()
            .expect(format!("Type {} has not been registered",std::any::type_name::<T>()).as_str())
            .entity_iter_mut()
    }

    pub fn with<U : Component>(self) -> QueryEntitiesWith<'a,(T,U)>{
        QueryEntitiesWith{
            world: self.world,
            _marker : Default::default(),
        }
    }
}

