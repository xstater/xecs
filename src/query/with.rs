use crate::{entity::EntityId, world::World};

use super::{QueryIterator, Queryable};

impl<'a,A : 'a + Queryable<'a>,B :'a + Queryable<'a>> Queryable<'a> for (A,B) {
    type Item = (<A as Queryable<'a>>::Item,<B as Queryable<'a>>::Item);

    fn query(world : &'a World) -> Box<(dyn QueryIterator<Item = Self::Item> + 'a)> {
        let iter_a = world.query::<A>();
        let iter_b = world.query::<B>();
        Box::new(WithIter{
            iter_a,
            iter_b
        })
    }
}

pub struct WithIter<A,B> {
    iter_a : A,
    iter_b : B
}

impl<'a,A : QueryIterator,B : QueryIterator> Iterator for WithIter<A,B>{
    type Item = (A::Item,B::Item);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((id,a)) = self.iter_a.next_with_id() {
            if let Some(b) = self.iter_b.from_id(id) {
                return Some((a,b))
            }
        }
        None
    }
}

impl<A : QueryIterator,B : QueryIterator> QueryIterator for WithIter<A,B> {
    fn from_id(&mut self,id : EntityId) -> Option<Self::Item>{
        if let Some(a) = self.iter_a.from_id(id) {
            if let Some(b) = self.iter_b.from_id(id) {
                return Some((a,b))
            }
        }
        None
    }

    fn next_with_id(&mut self) -> Option<(EntityId,Self::Item)> {
        while let Some((id,a)) = self.iter_a.next_with_id() {
            if let Some(b) = self.iter_b.from_id(id) {
                return Some((id,(a,b)))
            }
        }
        None
    }
}



impl<'a,A,B,C> Queryable<'a> for (A,B,C)
    where A : 'a + Queryable<'a>,
          B : 'a + Queryable<'a>,
          C : 'a + Queryable<'a>{
    type Item = (<A as Queryable<'a>>::Item,
                 <B as Queryable<'a>>::Item,
                 <C as Queryable<'a>>::Item);

    fn query(world : &'a World) -> Box<(dyn QueryIterator<Item = Self::Item> + 'a)> {
        let iter_a = world.query::<A>();
        let iter_b = world.query::<B>();
        let iter_c = world.query::<C>();
        Box::new(WithIter3{
            iter_a,
            iter_b,
            iter_c
        })
    }
}

pub struct WithIter3<A,B,C> {
    iter_a : A,
    iter_b : B,
    iter_c : C
}

impl<'a,A,B,C> Iterator for WithIter3<A,B,C>
    where A : QueryIterator,
          B : QueryIterator,
          C : QueryIterator{
    type Item = (A::Item,B::Item,C::Item);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((id,a)) = self.iter_a.next_with_id() {
            if let Some(b) = self.iter_b.from_id(id) {
                if let Some(c) = self.iter_c.from_id(id) {
                    return Some((a,b,c))
                }
            }
        }
        None
    }
}

impl<A,B,C> QueryIterator for WithIter3<A,B,C>
    where A : QueryIterator,
          B : QueryIterator,
          C : QueryIterator{
    fn from_id(&mut self,id : EntityId) -> Option<Self::Item>{
        if let Some(a) = self.iter_a.from_id(id) {
            if let Some(b) = self.iter_b.from_id(id) {
                if let Some(c) = self.iter_c.from_id(id) {
                    return Some((a,b,c))
                }
            }
        }
        None
    }

    fn next_with_id(&mut self) -> Option<(EntityId,Self::Item)> {
        while let Some((id,a)) = self.iter_a.next_with_id() {
            if let Some(b) = self.iter_b.from_id(id) {
                if let Some(c) = self.iter_c.from_id(id) {
                    return Some((id,(a,b,c)))
                }
            }
        }
        None
    }
}




impl<'a,A,B,C,D> Queryable<'a> for (A,B,C,D)
    where A : 'a + Queryable<'a>,
          B : 'a + Queryable<'a>,
          C : 'a + Queryable<'a>,
          D : 'a + Queryable<'a>{
    type Item = (<A as Queryable<'a>>::Item,
                 <B as Queryable<'a>>::Item,
                 <C as Queryable<'a>>::Item,
                 <D as Queryable<'a>>::Item);

    fn query(world : &'a World) -> Box<(dyn QueryIterator<Item = Self::Item> + 'a)> {
        let iter_a = world.query::<A>();
        let iter_b = world.query::<B>();
        let iter_c = world.query::<C>();
        let iter_d = world.query::<D>();
        Box::new(WithIter4{
            iter_a,
            iter_b,
            iter_c,
            iter_d
        })
    }
}

pub struct WithIter4<A,B,C,D> {
    iter_a : A,
    iter_b : B,
    iter_c : C,
    iter_d : D
}

impl<'a,A,B,C,D> Iterator for WithIter4<A,B,C,D>
    where A : QueryIterator,
          B : QueryIterator,
          C : QueryIterator,
          D : QueryIterator{
    type Item = (A::Item,B::Item,C::Item,D::Item);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((id,a)) = self.iter_a.next_with_id() {
            if let Some(b) = self.iter_b.from_id(id) {
                if let Some(c) = self.iter_c.from_id(id) {
                    if let Some(d) = self.iter_d.from_id(id) {
                       return Some((a,b,c,d))
                    }
                }
            }
        }
        None
    }
}

impl<A,B,C,D> QueryIterator for WithIter4<A,B,C,D>
    where A : QueryIterator,
          B : QueryIterator,
          C : QueryIterator,
          D : QueryIterator{
    fn from_id(&mut self,id : EntityId) -> Option<Self::Item>{
        if let Some(a) = self.iter_a.from_id(id) {
            if let Some(b) = self.iter_b.from_id(id) {
                if let Some(c) = self.iter_c.from_id(id) {
                    if let Some(d) = self.iter_d.from_id(id) {
                        return Some((a,b,c,d))
                    }
                }
            }
        }
        None
    }

    fn next_with_id(&mut self) -> Option<(EntityId,Self::Item)> {
        while let Some((id,a)) = self.iter_a.next_with_id() {
            if let Some(b) = self.iter_b.from_id(id) {
                if let Some(c) = self.iter_c.from_id(id) {
                    if let Some(d) = self.iter_d.from_id(id) {
                        return Some((id,(a,b,c,d)))
                    }
                }
            }
        }
        None
    }
}





impl<'a,A,B,C,D,E> Queryable<'a> for (A,B,C,D,E)
    where A : 'a + Queryable<'a>,
          B : 'a + Queryable<'a>,
          C : 'a + Queryable<'a>,
          D : 'a + Queryable<'a>,
          E : 'a + Queryable<'a>{
    type Item = (<A as Queryable<'a>>::Item,
                 <B as Queryable<'a>>::Item,
                 <C as Queryable<'a>>::Item,
                 <D as Queryable<'a>>::Item,
                 <E as Queryable<'a>>::Item);

    fn query(world : &'a World) -> Box<(dyn QueryIterator<Item = Self::Item> + 'a)> {
        let iter_a = world.query::<A>();
        let iter_b = world.query::<B>();
        let iter_c = world.query::<C>();
        let iter_d = world.query::<D>();
        let iter_e = world.query::<E>();
        Box::new(WithIter5{
            iter_a,
            iter_b,
            iter_c,
            iter_d,
            iter_e
        })
    }
}

pub struct WithIter5<A,B,C,D,E> {
    iter_a : A,
    iter_b : B,
    iter_c : C,
    iter_d : D,
    iter_e : E
}

impl<'a,A,B,C,D,E> Iterator for WithIter5<A,B,C,D,E>
    where A : QueryIterator,
          B : QueryIterator,
          C : QueryIterator,
          D : QueryIterator,
          E : QueryIterator{
    type Item = (A::Item,B::Item,C::Item,D::Item,E::Item);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((id,a)) = self.iter_a.next_with_id() {
            if let Some(b) = self.iter_b.from_id(id) {
                if let Some(c) = self.iter_c.from_id(id) {
                    if let Some(d) = self.iter_d.from_id(id) {
                        if let Some(e) = self.iter_e.from_id(id) {
                            return Some((a,b,c,d,e))
                        }
                    }
                }
            }
        }
        None
    }
}

impl<A,B,C,D,E> QueryIterator for WithIter5<A,B,C,D,E>
    where A : QueryIterator,
          B : QueryIterator,
          C : QueryIterator,
          D : QueryIterator,
          E : QueryIterator{
    fn from_id(&mut self,id : EntityId) -> Option<Self::Item>{
        if let Some(a) = self.iter_a.from_id(id) {
            if let Some(b) = self.iter_b.from_id(id) {
                if let Some(c) = self.iter_c.from_id(id) {
                    if let Some(d) = self.iter_d.from_id(id) {
                        if let Some(e) = self.iter_e.from_id(id) {
                            return Some((a,b,c,d,e))
                        }
                    }
                }
            }
        }
        None
    }

    fn next_with_id(&mut self) -> Option<(EntityId,Self::Item)> {
        while let Some((id,a)) = self.iter_a.next_with_id() {
            if let Some(b) = self.iter_b.from_id(id) {
                if let Some(c) = self.iter_c.from_id(id) {
                    if let Some(d) = self.iter_d.from_id(id) {
                        if let Some(e) = self.iter_e.from_id(id) {
                            return Some((id,(a,b,c,d,e)))
                        }
                    }
                }
            }
        }
        None
    }
}
