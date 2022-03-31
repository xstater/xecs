//! # System in XECS
//! System is a [Stream](futures::Stream) with [World](crate::world::World) 
//! as Context. Because [Stream](futures::Stream) is not stable 
//! in [std](std), XECS use [futures](futures)::[Stream](futures::Stream) instead.
//! # To Run System
//! Because system is just an async trait, you need a wrapper of runtime from 
//! [tokio](https://tokio.rs) or [async-std](https://async.rs)
use std::sync::Arc;
use parking_lot::RwLock;
use tokio_stream::Stream;
use crate::world::World;

/// System core trait
pub trait System : Stream {
    /// Get the [world](crate::world::World) of System
    fn world(&self) -> Arc<RwLock<World>>;
}
