# XECS
xecs is a rust Entity-Component-System library
# Details
XECS is a Grouped ECS library.
# Examples
### Create an empty world
```rust
let mut world = World::new();
```
### Register some components
Component is T : Send + Sync + 'static
```rust
struct Position(f64,f64,f64);
struct Particle;
world.register::<Position>();
world.register::<Particle>();
```
### Create 100 entity with Position and Particle components
```rust
for _ in 0..100 {
    world
        .create_entity()
        .attach(Position(1.0,2.0,1.2))
        .attach(Particle);
}

```
### Make a full-owning group to improve the performance of query iteration
```rust
world.make_group::<(Particle,Position)>(true,true);
```
### Create a system and update all entities with position and particle components
```rust
struct UpdatePosition;
impl<'a> System<'a> for UpdatePosition {
    type InitResource = ();
    type Resource = (&'a mut World);
    type Dependencies = ();
    type Error = Infallible;


    fn update(&'a mut self, world : RefMut<'a,World>) -> Result<(),Self::Error> {
        for (pos,_tag) in world.query::<(&mut Position,&Particle)>() {
            pos.0 += 1.1;
            pos.1 += 1.2;
            pos.3 += 1.4;
        }
        Ok(())
    }
}
```
### Add system to stage and run this stage
```rust
let mut stage = Stage::from_world(world);
stage.add_system(UpdatePosition);
stage.run();
```
