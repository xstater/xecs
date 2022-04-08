 # XECS
 An Entity-Component-System library
 ## Simple Example
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
     lety = random();
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
 
 # About entity
 Entity in XECS is just an number ID.In XECS, it's just a 
 [NonZeroUsize](std::num::NonZeroUsize).
 The ID is allocated from 1 by world automatically. The ```id=0``` 
 represents a recycled ID without any other flags through ```Option<EntityId>```.
 
 # ID recycling
 When you call ```world.create_entity()```, an ID will be allocated automatically. 
 If you call ```world.remove_entity(id)```, this ID will be a pit. If the 
 next ```world.create_entity()``` is called, it will allocate this ID to fill 
 the pit.Thanks to sparse set, it's still fast to 
 iterate all components no matter how random of ID
 
 # Concurrency Safety
 Because [Component](crate::component::Component) is just ```T : Send + Sync```.
 [World](crate::world::World) can use [RwLock](std::sync::RwLock) to 
 ensure the borrow check relations of all components.And [World](crate::world::World) can also
 be ```Send + Sync```.Therefore,the all other states of world can be guarded
 by [RwLock](std::sync::RwLock).So we can use world in concurrency environment by ```RwLock<World>```.
 
 # System in XECS
 System is a [Stream](futures::Stream) with [World](crate::world::World) 
 as Context. Because [Stream](futures::Stream) is not stable 
 in [std](std), XECS use [futures](futures)::[Stream](futures::Stream) instead.
 # To Run System
 Because system is just an async trait, you need a wrapper of runtime from 
 [tokio](https://tokio.rs) or [async-std](https://async.rs)