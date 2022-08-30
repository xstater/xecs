mod manager;

pub use manager::EntityManager;

pub struct Entity<'a> {
    _nop : std::marker::PhantomData<&'a u32>
}