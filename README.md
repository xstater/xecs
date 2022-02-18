# XECS
An Entity-Component-System library
## Example
```rust,no_run
// Define two components struct
// Component is Send + Sync + 'static
#[derive(Debug,Copy)]
struct Position{
    x : f32,
    y : f32
};
struct Hidden;

// create an empty world
let mut world = World::new();

// generate 10 entities
for _ in 0..10 {
    let x = random();
    let y = random();
    // andomly generate the positions
    world.create_entity()
        .attach(Position { x,y });
}

// print all postions
for pos in world.query::<&Position>() {
    print!("{:?}",pos)
}

// filter some entities need to be hidden
let ids = world.query::<&Position>()
    .with_id()
    .filter(|(_,_)|random())
    .map(|(id,_)|id)
    .collect::<Vec<_>>();

// attach hidden to id
for id in ids {
    world.attach_component(id,Hidden);
}

// make a full-owning group to accelerate the query
world.make_group(full_owning::<Hidden,Position>());

// only print postions with id that is not hidden
for (id,data) in world.query::<&Position,Without<&Hidden>>() {
    print!("{}:{:?}",id,data);
}
```

