//! # XECS
//! A grouped Entity-Component-System library

/// The core of XECS, world struct
pub mod world;
/// Some useful structs about entities
pub mod entity;
/// Component core trait
pub mod component;
/// Some things to accelerate the iteration
pub mod group;
/// The query functions
pub mod query;
/// An implemention of sparse set
pub mod sparse_set;
/// The system trait
pub mod system;

