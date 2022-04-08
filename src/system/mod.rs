use std::sync::Arc;
use parking_lot::RwLock;
use futures::stream::Stream;
use crate::world::World;

/// System core trait
pub trait System : Stream {
    /// Get the [world](crate::world::World) of System
    fn world(&self) -> Arc<RwLock<World>>;
}

