mod world;
mod entity;
mod query;

pub trait Component : Send + Sync + 'static {}
impl<T : Send + Sync + 'static> Component for T{}

pub use entity::EntityId;
pub use world::World;

#[cfg(test)]
mod tests {
    #[test]
    fn test(){

    }
}
