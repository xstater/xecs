use std::marker::PhantomData;
use crate::{entity::EntityId, world::World};
use super::{QueryIterator, Queryable};

pub struct Without<T>{
    _marker : PhantomData<T>
}

impl<'a,A : 'a + Queryable<'a>,B : 'a + Queryable<'a>> Queryable<'a> for (Without<A>,B) {
    type Item = <B as Queryable<'a>>::Item;

    fn query(world : &'a World) -> Box<(dyn QueryIterator<Item = Self::Item> + 'a)> {
        let iter_a = world.query::<A>();
        let iter_b = world.query::<B>();
        Box::new(WithoutIterLeft{
            iter_a,
            iter_b
        })
    }
}

pub struct WithoutIterLeft<A,B>{
    iter_a : A,
    iter_b : B
}

impl<'a,A : QueryIterator,B : QueryIterator> Iterator for WithoutIterLeft<A,B> {
    type Item = B::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((id,b)) = self.iter_b.next_with_id() {
            if let None = self.iter_a.from_id(id) {
                return Some(b);
            }
        }
        None
    }
}

impl<'a,A : QueryIterator,B : QueryIterator> QueryIterator for WithoutIterLeft<A,B> {
    fn from_id(&mut self,id : EntityId) -> Option<Self::Item> {
        if let None = self.iter_a.from_id(id) {
            if let Some(b) = self.iter_b.from_id(id) {
                return Some(b)
            }
        }
        None
    }

    fn next_with_id(&mut self) -> Option<(EntityId,Self::Item)> {
        while let Some((id,b)) = self.iter_b.next_with_id() {
            if let None = self.iter_a.from_id(id) {
                return Some((id,b));
            }
        }
        None
    }
}





impl<'a,A : 'a + Queryable<'a>,B : 'a + Queryable<'a>> Queryable<'a> for (A,Without<B>) {
    type Item = <A as Queryable<'a>>::Item;

    fn query(world : &'a World) -> Box<(dyn QueryIterator<Item = Self::Item> + 'a)> {
        let iter_a = world.query::<A>();
        let iter_b = world.query::<B>();
        Box::new(WithoutIterRight{
            iter_a,
            iter_b
        })
    }
}

pub struct WithoutIterRight<A,B>{
    iter_a : A,
    iter_b : B
}

impl<'a,A : QueryIterator,B : QueryIterator> Iterator for WithoutIterRight<A,B> {
    type Item = A::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((id,a)) = self.iter_a.next_with_id() {
            if let None = self.iter_b.from_id(id) {
                return Some(a);
            }
        }
        None
    }
}

impl<'a,A : QueryIterator,B : QueryIterator> QueryIterator for WithoutIterRight<A,B> {
    fn from_id(&mut self,id : EntityId) -> Option<Self::Item> {
        if let None = self.iter_b.from_id(id) {
            if let Some(a) = self.iter_a.from_id(id) {
                return Some(a)
            }
        }
        None
    }

    fn next_with_id(&mut self) -> Option<(EntityId,Self::Item)> {
        while let Some((id,a)) = self.iter_a.next_with_id() {
            if let None = self.iter_b.from_id(id) {
                return Some((id,a));
            }
        }
        None
    }
}
