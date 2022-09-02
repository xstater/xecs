use parking_lot::RwLock;
use crate::entity::EntityManager;


/// The core of XECS
pub struct World {
    next_other_storage_id: u64,
    entities: RwLock<EntityManager>,
}

impl World {
    /// Create a new empty World
    pub fn new() -> Self {
        World {
            next_other_storage_id: 0,
            entities: RwLock::new(EntityManager::new()),
        }
    }
}

#[cfg(test)]
mod tests {
}
