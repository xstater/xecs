extern crate xecs;

use xecs::World;
use xecs::Component;

fn main() {
    #[derive(Debug)]
    struct Fuck(i32);
    #[derive(Debug)]
    struct Shit(char);

    println!("here");

    let mut world = World::new();
    println!("{:?}",world);
    world
        .register::<Fuck>()
        .register::<Shit>()
    ;
    println!("{:?}",world);
}