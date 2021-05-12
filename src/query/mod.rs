pub mod query;
pub mod query2;

pub use query::{Query,QueryEntity};

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

        // for (eid,u) in world.make_query::<u32>().entities().query_mut() {
        //     *u = eid as u32;
        // }

        for u in world.make_query::<u32>().query() {
            print!("{} ",u)
        }
        println!();

        for u in world.make_query::<char>().query() {
            print!("{:?} ",u);
        }
        println!();

        for (eid,ch) in world.make_query::<char>().entities().query() {
            print!("({},{:?}), ",eid,ch);
        }
        println!();
        
        for (ch,u) in world.make_query::<char>().with::<u32>().query() {
            print!("({:?},{})",ch,u);
        }
        println!();
    }
}

