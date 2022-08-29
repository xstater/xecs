mod manager;


pub struct Entity<'a> {
    _nop : std::marker::PhantomData<&'a u32>
}