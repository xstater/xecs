mod query;
mod query_with;

pub use query::{Query,QueryEntities};
pub use query_with::{QueryWith,QueryEntitiesWith};

#[cfg(test)]
mod tests{
    use crate::World;

    #[test]
    fn basic_test() {
        let mut world = World::new();

        world
            .register::<u32>()
            .register::<char>();

        world.create_entity(1u32);
        world.create_entity(2u32);
        world.create_entity(3u32)
            .with('a');
        world.create_entity(4u32)
            .with('b');
        world.create_entity(5u32)
            .with('c');
        world.create_entity(6u32);
        world.create_entity('d');

        for u in world.make_query::<u32>().query() {
            print!("{:?} ",u);
        }
        println!();

        for (eid,ch) in world.make_query::<char>().entities().query() {
            print!("({},{:?}), ",eid,ch);
        }
        println!();
    }
}
