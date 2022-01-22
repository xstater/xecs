use std::{any::TypeId, sync::{RwLockReadGuard, RwLockWriteGuard}};
use crate::{component::{Component, ComponentStorage}, entity::EntityId, sparse_set::SparseSet, world::World};

mod with;
mod without;

pub use with::{
    WithIter,
    WithIter3,
    WithIter4,
    WithIter5
};

pub use without::{
    Without,
    WithoutIterLeft,
    WithoutIterRight
};

pub trait Queryable<'a> {
    type Item;

    fn query(world : &'a World) -> Box<(dyn QueryIterator<Item = Self::Item> + 'a)>;
}

pub trait QueryIterator : Iterator {
    fn from_id(&mut self,id : EntityId) -> Option<Self::Item>;
    fn next_with_id(&mut self) -> Option<(EntityId,Self::Item)>;
}

impl<T : QueryIterator + ?Sized> QueryIterator for Box<T> {
    fn from_id(&mut self,id : EntityId) -> Option<Self::Item> {
        (**self)
            .from_id(id)
    }

    fn next_with_id(&mut self) -> Option<(EntityId,Self::Item)> {
        (**self)
            .next_with_id()
    }
}




pub struct IterRef<'a,T> {
    index : usize,
    sparse_set : *const SparseSet<EntityId,T>,
    borrow : RwLockReadGuard<'a,Box<dyn ComponentStorage>>
}

impl<'a,T : Component> Queryable<'a> for &'a T {
    type Item = Self;

    fn query(world : &'a World) -> Box<(dyn QueryIterator<Item = Self::Item> + 'a)> {
        assert!(world.has_registered::<T>(),
                "Queryable for &'a T: Component was not registered in world");
        let type_id = TypeId::of::<T>();
        // Unwrap here
        // assert before ensures this
        let storage = world.storage_ref(type_id).unwrap();
        // Safety:
        // storage is SparseSet<EntityId,T>
        let sparse_set = unsafe {
            storage.downcast_ref::<SparseSet<EntityId,T>>()
        };
        let ptr = &*sparse_set;
        Box::new(IterRef{
            index : 0,
            sparse_set : ptr,
            borrow : storage
        })
    }
}


impl<'a,T : Component> Iterator for IterRef<'a,T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.borrow.count() {
            // Safety:
            // Safe here, because self.sparse_set is 
            // a pointer from borrow,
            // This pointer is valid now.
            let sparse_set = unsafe { &*self.sparse_set };
            // Safety:
            // Safe here, because we checked before.
            let data = unsafe {
                sparse_set.data().get_unchecked(self.index)
            };
            self.index += 1;
            Some(data)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.borrow.count() - self.index;
        (rem,Some(rem))
    }
}
impl<'a,T : Component> ExactSizeIterator for IterRef<'a,T>{}

impl<'a,T : Component> QueryIterator for IterRef<'a,T> {

    fn from_id(&mut self,id : EntityId) -> Option<Self::Item> {
        // Safety:
        // Safe here, because self.sparse_set is 
        // a pointer from borrow:Ref<'a,SparseSet<...>>,
        // This pointer is valid now.
        let sparse_set = unsafe { &*self.sparse_set };
        sparse_set.get(id)
    }

    fn next_with_id(&mut self) -> Option<(EntityId,Self::Item)> {
        if self.index < self.borrow.count() {
            // Safety:
            // Safe here, because self.sparse_set is 
            // a pointer from borrow,
            // This pointer is valid now.
            let sparse_set = unsafe { &*self.sparse_set };
            // Safety:
            // Safe here, because we have already checked.
            let id = *unsafe {
                sparse_set.entities().get_unchecked(self.index)
            };
            // Safety:
            // Safe here, because we have already checked.
            let data = unsafe {
                sparse_set.data().get_unchecked(self.index)
            };
            self.index += 1;
            Some((id,data))
        } else {
            None
        }
    }
}




pub struct IterMut<'a,T> {
    index : usize,
    sparse_set : *mut SparseSet<EntityId,T>,
    borrow : RwLockWriteGuard<'a,Box<dyn ComponentStorage>>
}

impl<'a,T : Component> Queryable<'a> for &'a mut T {
    type Item = Self;

    fn query(world : &'a World) -> Box<(dyn QueryIterator<Item = Self::Item> + 'a)> {
        assert!(world.has_registered::<T>(),
                "Queryable for &'a mut T: Component was not registered in world");
        let type_id = TypeId::of::<T>();
        // Unwrap here
        // assert before ensures this
        let mut storage = world.storage_mut(type_id).unwrap();
        // Safety:
        // storage is SparseSet<EntityId,T>
        let sparse_set = unsafe {
            storage.downcast_mut::<SparseSet<EntityId,T>>()
        };
        let ptr = &mut *sparse_set;
        Box::new(IterMut{
            index : 0,
            sparse_set : ptr,
            borrow : storage
        })
    }
}


impl<'a,T : Component> Iterator for IterMut<'a,T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.borrow.count() {
            // Safety:
            // Safe here, because self.sparse_set is 
            // a pointer from borrow,
            // This pointer is valid now.
            let sparse_set = unsafe { &mut *self.sparse_set };
            // Safety:
            // Safe here, because we checked before.
            let data = unsafe {
                sparse_set.data_mut().get_unchecked_mut(self.index)
            };
            self.index += 1;
            Some(data)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.borrow.count() - self.index;
        (rem,Some(rem))
    }
}
impl<'a,T : Component> ExactSizeIterator for IterMut<'a,T>{}

impl<'a,T : Component> QueryIterator for IterMut<'a,T> {

    fn from_id(&mut self,id : EntityId) -> Option<Self::Item> {
        // Safety:
        // Safe here, because self.sparse_set is 
        // a pointer from borrow:Ref<'a,SparseSet<...>>,
        // This pointer is valid now.
        let sparse_set = unsafe { &mut *self.sparse_set };
        sparse_set.get_mut(id)
    }

    fn next_with_id(&mut self) -> Option<(EntityId,Self::Item)> {
        if self.index < self.borrow.count() {
            // Safety:
            // Safe here, because self.sparse_set is 
            // a pointer from borrow,
            // This pointer is valid now.
            let sparse_set = unsafe { &mut *self.sparse_set };
            // Safety:
            // Safe here, because we have already checked.
            let id = *unsafe {
                sparse_set.entities().get_unchecked(self.index)
            };
            // Safety:
            // Safe here, because we have already checked.
            let data = unsafe {
                sparse_set.data_mut().get_unchecked_mut(self.index)
            };
            self.index += 1;
            Some((id,data))
        } else {
            None
        }
    }
}




pub struct IdIter<A> {
    iter : A
}

pub trait WithId {
    type Inner;

    fn with_id(self) -> IdIter<Self::Inner>;
}

impl<A : QueryIterator> WithId for A {
    type Inner = A;

    fn with_id(self) -> IdIter<Self::Inner> {
        IdIter{
            iter : self
        }
    }
}

impl<A : QueryIterator> Iterator for IdIter<A> {
    type Item = (EntityId,<A as Iterator>::Item);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next_with_id()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

#[cfg(test)]
mod tests{
    use std::num::NonZeroUsize;

    use crate::{query::{WithId, Without}, world::World};


    #[test]
    fn basic_test() {
        #[derive(Debug,Clone,Copy,PartialEq)]
        struct Tag;

        let mut world = World::new();

        world.register::<u32>()
            .register::<char>()
            .register::<Tag>();

        world.create_entity().attach(1_u32);
        world.create_entity().attach(2_u32).attach('c');
        world.create_entity().attach(3_u32).attach(Tag);
        world.create_entity().attach(4_u32).attach('b');
        world.create_entity().attach(5_u32).attach('q').attach(Tag);
        world.create_entity().attach(6_u32).attach('w');
        world.create_entity().attach(7_u32);
        world.create_entity().attach(8_u32).attach('s').attach(Tag);

        let res = world.query::<(&u32,&char)>()
            .map(|(a,b)|(*a,*b))
            .collect::<Vec<_>>();
        assert_eq!(&res,&[(2,'c'),(4,'b'),(5,'q'),(6_u32,'w'),(8_u32,'s')]);

        let res = world.query::<(&u32,(&mut char,&Tag))>()
            .map(|(a,(b,c))|(*a,*b,*c))
            .collect::<Vec<_>>();
        assert_eq!(&res,&[(5,'q',Tag),(8,'s',Tag)]);

        let res = world.query::<(&u32,(&mut char,&Tag))>()
            .with_id()
            .map(|(id,(a,(b,c)))|(id,*a,*b,*c))
            .collect::<Vec<_>>();
        assert_eq!(&res,&[(NonZeroUsize::new(5).unwrap(),5,'q',Tag),(NonZeroUsize::new(8).unwrap(),8,'s',Tag)]);
    }

    #[test]
    fn without_test() {
        #[derive(Debug,Clone,Copy,PartialEq)]
        struct Tag;

        let mut world = World::new();

        world.register::<u32>()
            .register::<char>()
            .register::<Tag>();

        world.create_entity().attach(1_u32);
        world.create_entity().attach(2_u32).attach('c');
        world.create_entity().attach(3_u32).attach(Tag);
        world.create_entity().attach(4_u32).attach('b');
        world.create_entity().attach(5_u32).attach('q').attach(Tag);
        world.create_entity().attach(6_u32).attach('w');
        world.create_entity().attach(7_u32);
        world.create_entity().attach(8_u32).attach('s').attach(Tag);

        let res = world.query::<(&u32,Without<&char>)>()
            .map(|a|*a)
            .collect::<Vec<_>>();
        assert_eq!(&res,& [1,3,7]);

        let res = world.query::<(Without<(&char,&Tag)>,&u32)>()
            .map(|b|*b)
            .collect::<Vec<_>>();
        assert_eq!(&res,&[1,2,3,4,6,7]);

        let res = world.query::<(Without<&Tag>,(&u32,Without<&char>))>()
            .map(|b|*b)
            .collect::<Vec<_>>();
        assert_eq!(&res,&[1,7]);
    }
}
