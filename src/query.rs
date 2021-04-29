use crate::{Component, World};

pub struct Query<'a,T : Component>{
    world : &'a mut World,
}

pub struct QueryWith<'a,T,U> {
    world : &'a mut world
}

impl<'a,T : Component> Query<'a,T> {
    pub fn from_world(world: &'a mut World) -> Self {
        Query {
            world
        }
    }

    pub fn query(self) -> &[T] {
        self.world.components()
    }

    pub fn query_mut(mut self) -> &mut [T] {
        self.world.components_mut()
    }

    pub fn with<U : Component>(self) -> QueryWith<'a,T,U>{
        QueryWith{
            world: self.world
        }
    }
}
