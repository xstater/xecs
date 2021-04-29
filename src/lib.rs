use std::num::NonZeroU32;

mod world;
mod component;
mod entity;

pub use entity::EntityId;
pub use world::World;
pub use component::Component;

#[cfg(test)]
mod tests {
    #[test]
    fn test(){

    }
}
