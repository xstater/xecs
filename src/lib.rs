mod world;
mod entity;
mod components;
mod group;
mod query;
mod sparse_set;

pub use entity::{ EntityId,Entities };
pub use world::World;
pub use components::Component;

#[cfg(test)]
mod tests {
    #[test]
    fn test(){

    }
}
