use parking_lot::RwLock;
use crate::{entity::EntityManager, Archetype};


/// XECS的核心
pub struct World {
    next_other_storage_id: u64,
    entities: RwLock<EntityManager>,
    archetypes: Vec<RwLock<Archetype>>
}

impl World {
    /// 创建一个空的World
    pub fn new() -> Self {
        World {
            next_other_storage_id: 0,
            entities: RwLock::new(EntityManager::new()),
            archetypes: Vec::new()
        }
    }

}

#[cfg(test)]
mod tests {
}
