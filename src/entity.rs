use crate::World;
use crate::Component;

pub type EntityId = usize;

pub struct Entity<'a>{
    world : &'a mut World,
    entity_id : EntityId
}

impl<'a> Entity<'a> {
    pub(in crate) fn new(world: &'a mut World, entity_id: EntityId) -> Entity<'a>{
        Entity{
            world,
            entity_id
        }
    }

    pub fn with<T : Component>(self,component: T) -> Entity<'a> {
        self.world.add_component_for_entity::<T>(self.entity_id,component);
        self
    }

    pub fn id(&self) -> EntityId {
        self.entity_id
    }
}
