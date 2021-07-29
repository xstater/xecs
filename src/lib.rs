mod world;
mod entity;
mod components;
mod group;

pub use entity::EntityId;
pub use world::World;
pub use components::Component;

#[cfg(test)]
mod tests {
    #[test]
    fn test(){

    }
}
