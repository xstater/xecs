pub mod world;
pub mod entity;
pub mod components;
pub mod group;
pub mod query;
pub mod sparse_set;
pub mod systems;
pub mod stage;
pub mod resource;

pub use entity::{ EntityId,Entities };
pub use world::World;
pub use components::Component;
pub use systems::System;
pub use stage::Stage;

#[cfg(test)]
mod tests {
    #[test]
    fn test(){

    }
}
