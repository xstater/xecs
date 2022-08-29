mod manager;

pub use manager::EntityManager;

use crate::{World, EntityId};

pub struct Entity<'a> {
    world: &'a World,
    id: EntityId
}

impl<'a> Entity<'a> {
    pub(in crate) fn new(world: &'a World,id: EntityId) -> Self {
        Entity { world, id }
    }
}