use std::num::NonZeroUsize;
use crate::{World, Component};

pub type EntityId = NonZeroUsize;

#[derive(Debug)]
pub struct EntityRef<'a>{
    world : &'a mut World,
    id : EntityId
}

impl<'a> EntityRef<'a>{
    pub(in crate) fn new(world : &'a mut World,entity_id : EntityId) -> EntityRef<'a>{
        EntityRef{
            world,
            id: entity_id
        }
    }

    pub fn into_id(self) -> EntityId{
        self.id
    }

    pub fn attach<T : Component>(self,component : T) -> EntityRef<'a>{
        self.world.attach_component(self.id,component);
        self
    }

    pub fn detach<T : Component>(self) -> EntityRef<'a>{
        self.world.detach_component::<T>(self.id);//ignore the error
        self
    }
}

pub trait IntoEntityRef {
    fn into_entity_ref(self,world : &mut World) -> EntityRef<'_>;
}

impl IntoEntityRef for EntityId{
    fn into_entity_ref(self, world: &mut World) -> EntityRef<'_> {
        EntityRef{
            world,
            id: self
        }
    }
}