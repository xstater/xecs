mod world;
mod component;
mod entity;
mod query;

pub use entity::EntityId;
pub use world::World;
pub use component::Component;

#[cfg(test)]
mod tests {
    #[test]
    fn test(){

    }
}
