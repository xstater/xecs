use std::sync::{Arc, RwLock};
use futures::Stream;
use crate::world::World;

pub trait System : Stream {
    fn world(&self) -> Arc<RwLock<World>>;
}
