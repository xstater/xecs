//! # XECS
//! xecs is a rust Entity-Component-System library
//! # Details
//! XECS is a Grouped ECS library.
//! # Examples
//! ### Create an empty world
//! ```no_run
//! # use xecs::World;
//! let mut world = World::new();
//! ```
//! ### Register some components
//! Component is T : Send + Sync + 'static
//! ```no_run
//! struct Position(f64,f64,f64);
//! struct Particle;
//! world.register::<Position>();
//! world.register::<Particle>();
//! ```
//! ### Create 100 entity with Position and Particle components
//! ```no_run
//! for _ in 0..100 {
//!     world
//!         .create_entity()
//!         .attach(Position(1.0,2.0,1.2))
//!         .attach(Particle);
//! }
//!
//! ```
//! ### Make a full-owning group to improve the performance of query iteration
//! ```no_run
//! world.make_group::<(Particle,Position)>(true,true);
//! ```
//! ### Create a system and update all entities with position and particle components
//! ```no_run
//! # use xecs::{System, World};
//! # use std::cell::RefMut;
//! struct UpdatePosition;
//! impl<'a> System<'a> for UpdatePosition {
//!     type Resource = (&'a mut World);
//!     type Dependencies = ();
//!
//!     fn update(&'a mut self, world : RefMut<'a,World>) {
//!         for (pos,_tag) in world.query::<(&mut Position,&Particle)>() {
//!             pos.0 += 1.1;
//!             pos.1 += 1.2;
//!             pos.3 += 1.4;
//!         }
//!     }
//! }
//! ```
//! ### Add system to stage and run this stage
//! ```no_run
//! # use xecs::Stage;
//! let mut stage = Stage::from_world(world);
//! stage.add_system(UpdatePosition);
//! stage.run();
//! ```

pub mod world;
pub mod entity;
pub mod components;
pub mod group;
pub mod query;
pub mod sparse_set;
pub mod system;
pub mod stage;
pub mod resource;

pub use entity::{ EntityId,Entities };
pub use world::World;
pub use components::Component;
pub use system::System;
pub use stage::Stage;

