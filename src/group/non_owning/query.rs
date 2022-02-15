use std::{any::TypeId, sync::{RwLockReadGuard, RwLockWriteGuard}};
use crate::{component::{Component, ComponentStorage}, entity::EntityId, group::{Group, non_owning}, query::{QueryIterator, Queryable}, sparse_set::SparseSet, world::World};
use super::NonOwning;

pub struct IterRefRef<'a,A,B> {
    index: usize,
    sparse_set_group: *const SparseSet<EntityId,(usize,usize)>,
    sparse_set_a: *const SparseSet<EntityId,A>,
    sparse_set_b: *const SparseSet<EntityId,B>,
    #[allow(unused)]
    borrow_group: RwLockReadGuard<'a,Box<dyn Group>>,
    #[allow(unused)]
    borrow_a: RwLockReadGuard<'a,Box<dyn ComponentStorage>>,
    #[allow(unused)]
    borrow_b: RwLockReadGuard<'a,Box<dyn ComponentStorage>>
}

impl<'a,A : Component,B : Component> Queryable<'a> for NonOwning<&'a A,&'a B> {
    type Item = (&'a A,&'a B);

    fn query(world : &'a World) -> Box<(dyn QueryIterator<Item = Self::Item> + 'a)> {
        assert!(world.has_registered::<A>() && world.has_registered::<B>(),
                "Queryable for NonOwning: Component was not registered in world");
        let type_id_a = TypeId::of::<A>();
        let type_id_b = TypeId::of::<B>();
        // Unwrap here
        // assert before ensures this
        let storage_a = world.raw_storage_read(type_id_a).unwrap();
        let storage_b = world.raw_storage_read(type_id_b).unwrap();
        // Safety:
        // storage is SparseSet<EntityId,...>
        let sparse_set_a = unsafe {
            storage_a.downcast_ref::<SparseSet<EntityId,A>>()
        };
        let sparse_set_b = unsafe {
            storage_b.downcast_ref::<SparseSet<EntityId,B>>()
        };
        let ptr_a = &*sparse_set_a;
        let ptr_b = &*sparse_set_b;
        let group = non_owning::<A,B>();
        assert!(world.has_group(&group),"Queryable for NonOwning: Group is not in world");
        let group = world.group(&group);
        // Safety:
        // group type is NonOwning<A,B>
        let group_data = unsafe {
            group.downcast_ref::<NonOwning<A,B>>()
        };
        let group_data = &group_data.sparse_set;
        let ptr_group = &*group_data;
        Box::new(IterRefRef{
            index: 0,
            sparse_set_group: ptr_group,
            sparse_set_a: ptr_a,
            sparse_set_b: ptr_b,
            borrow_group: group,
            borrow_a: storage_a,
            borrow_b: storage_b
        })
    }
}

impl<'a,A: Component,B : Component> Iterator for IterRefRef<'a,A,B> {
    type Item = (&'a A,&'a B);

    fn next(&mut self) -> Option<Self::Item> {
        // Safety:
        // Safe here, because self.group is 
        // a pointer from borrow,
        // This pointer is valid now.
        let group = unsafe { &*self.sparse_set_group };
        if self.index < group.len() {
            // Safety:
            // we checked in if condition
            let (index_a,index_b) = unsafe {
                group.data().get_unchecked(self.index)
            };
            // Safety:
            // Safe here, because self.sparse_set is 
            // a pointer from borrow,
            // This pointer is valid now.
            let sparse_set_a = unsafe { &*self.sparse_set_a };
            let sparse_set_b = unsafe { &*self.sparse_set_b };
            // Safety:
            // Safe here, because the index stored in group is valid.
            let data_a = unsafe {
                sparse_set_a.data().get_unchecked(*index_a)
            };
            let data_b = unsafe {
                sparse_set_b.data().get_unchecked(*index_b)
            };
            self.index += 1;
            Some((data_a,data_b))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.borrow_group.len() - self.index;
        (rem,Some(rem))
    }
}

impl<'a,A : Component,B : Component> ExactSizeIterator for IterRefRef<'a,A,B>{ }

impl<'a,A : Component,B : Component> QueryIterator for IterRefRef<'a,A,B> {
    fn from_id(&mut self, id : EntityId) -> Option<Self::Item> {
        // Safety:
        // Safe here, because these are
        // pointers from borrow,
        // This pointer is valid now.
        let group = unsafe { &*self.sparse_set_group };
        let sparse_set_a = unsafe { &*self.sparse_set_a };
        let sparse_set_b = unsafe { &*self.sparse_set_b };
        if let Some((index_a,index_b)) = group.get(id) {
            // Safety:
            // Safe here, because the index stored in group is valid.
            let data_a = unsafe {
                sparse_set_a.data().get_unchecked(*index_a)
            };
            let data_b = unsafe {
                sparse_set_b.data().get_unchecked(*index_b)
            };
            Some((data_a,data_b))
        } else {
            None
        }
    }

    fn next_with_id(&mut self) -> Option<(EntityId,Self::Item)> {
        // Safety:
        // Safe here, because self.group is 
        // a pointer from borrow,
        // This pointer is valid now.
        let group = unsafe { &*self.sparse_set_group };
        if self.index < group.len() {
            // Safety:
            // Safe here, because if condition ensures this.
            let id = *unsafe {
                group.entities().get_unchecked(self.index)
            };
            let (index_a,index_b) = unsafe {
                group.data().get_unchecked(self.index)
            };
            // Safety:
            // Safe here, because these are
            // pointers from borrow,
            // This pointer is valid now.
            let sparse_set_a = unsafe { &*self.sparse_set_a };
            let sparse_set_b = unsafe { &*self.sparse_set_b };
            // Safety:
            // Safe here, because the index stored in group is valid.
            let data_a = unsafe {
                sparse_set_a.data().get_unchecked(*index_a)
            };
            let data_b = unsafe {
                sparse_set_b.data().get_unchecked(*index_b)
            };
            self.index += 1;
            Some((id,(data_a,data_b)))
        } else {
            None
        }
    }
}






pub struct IterRefMut<'a,A,B> {
    index: usize,
    sparse_set_group: *const SparseSet<EntityId,(usize,usize)>,
    sparse_set_a: *const SparseSet<EntityId,A>,
    sparse_set_b: *mut SparseSet<EntityId,B>,
    #[allow(unused)]
    borrow_group: RwLockReadGuard<'a,Box<dyn Group>>,
    #[allow(unused)]
    borrow_a: RwLockReadGuard<'a,Box<dyn ComponentStorage>>,
    #[allow(unused)]
    borrow_b: RwLockWriteGuard<'a,Box<dyn ComponentStorage>>
}

impl<'a,A : Component,B : Component> Queryable<'a> for NonOwning<&'a A,&'a mut B> {
    type Item = (&'a A,&'a mut B);

    fn query(world : &'a World) -> Box<(dyn QueryIterator<Item = Self::Item> + 'a)> {
        assert!(world.has_registered::<A>() && world.has_registered::<B>(),
                "Queryable for NonOwning: Component was not registered in world");
        let type_id_a = TypeId::of::<A>();
        let type_id_b = TypeId::of::<B>();
        // Unwrap here
        // assert before ensures this
        let storage_a = world.raw_storage_read(type_id_a).unwrap();
        let mut storage_b = world.raw_storage_write(type_id_b).unwrap();
        // Safety:
        // storage is SparseSet<EntityId,...>
        let sparse_set_a = unsafe {
            storage_a.downcast_ref::<SparseSet<EntityId,A>>()
        };
        let sparse_set_b = unsafe {
            storage_b.downcast_mut::<SparseSet<EntityId,B>>()
        };
        let ptr_a = &*sparse_set_a;
        let ptr_b = &mut *sparse_set_b;
        let group = non_owning::<A,B>();
        assert!(world.has_group(&group),"Queryable for NonOwning: Group is not in world");
        let group = world.group(&group);
        // Safety:
        // group type is NonOwning<A,B>
        let group_data = unsafe {
            group.downcast_ref::<NonOwning<A,B>>()
        };
        let group_data = &group_data.sparse_set;
        let ptr_group = &*group_data;
        Box::new(IterRefMut{
            index: 0,
            sparse_set_group: ptr_group,
            sparse_set_a: ptr_a,
            sparse_set_b: ptr_b,
            borrow_group: group,
            borrow_a: storage_a,
            borrow_b: storage_b
        })
    }
}

impl<'a,A: Component,B : Component> Iterator for IterRefMut<'a,A,B> {
    type Item = (&'a A,&'a mut B);

    fn next(&mut self) -> Option<Self::Item> {
        // Safety:
        // Safe here, because self.group is 
        // a pointer from borrow,
        // This pointer is valid now.
        let group = unsafe { &*self.sparse_set_group };
        if self.index < group.len() {
            // Safety:
            // we checked in if condition
            let (index_a,index_b) = unsafe {
                group.data().get_unchecked(self.index)
            };
            // Safety:
            // Safe here, because self.sparse_set is 
            // a pointer from borrow,
            // This pointer is valid now.
            let sparse_set_a = unsafe { &*self.sparse_set_a };
            let sparse_set_b = unsafe { &mut *self.sparse_set_b };
            // Safety:
            // Safe here, because the index stored in group is valid.
            let data_a = unsafe {
                sparse_set_a.data().get_unchecked(*index_a)
            };
            let data_b = unsafe {
                sparse_set_b.data_mut().get_unchecked_mut(*index_b)
            };
            self.index += 1;
            Some((data_a,data_b))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.borrow_group.len() - self.index;
        (rem,Some(rem))
    }
}

impl<'a,A : Component,B : Component> ExactSizeIterator for IterRefMut<'a,A,B>{ }

impl<'a,A : Component,B : Component> QueryIterator for IterRefMut<'a,A,B> {
    fn from_id(&mut self, id : EntityId) -> Option<Self::Item> {
        // Safety:
        // Safe here, because these are
        // pointers from borrow,
        // This pointer is valid now.
        let group = unsafe { &*self.sparse_set_group };
        let sparse_set_a = unsafe { &*self.sparse_set_a };
        let sparse_set_b = unsafe { &mut *self.sparse_set_b };
        if let Some((index_a,index_b)) = group.get(id) {
            // Safety:
            // Safe here, because index stored in group is valid.
            let data_a = unsafe {
                sparse_set_a.data().get_unchecked(*index_a)
            };
            let data_b = unsafe {
                sparse_set_b.data_mut().get_unchecked_mut(*index_b)
            };
            Some((data_a,data_b))
        } else {
            None
        }
    }

    fn next_with_id(&mut self) -> Option<(EntityId,Self::Item)> {
        // Safety:
        // Safe here, because self.group is 
        // a pointer from borrow,
        // This pointer is valid now.
        let group = unsafe { &*self.sparse_set_group };
        if self.index < group.len() {
            // Safety:
            // Safe here, because if condition ensures this.
            let id = *unsafe {
                group.entities().get_unchecked(self.index)
            };
            let (index_a,index_b) = unsafe {
                group.data().get_unchecked(self.index)
            };
            // Safety:
            // Safe here, because these are
            // pointers from borrow,
            // This pointer is valid now.
            let sparse_set_a = unsafe { &*self.sparse_set_a };
            let sparse_set_b = unsafe { &mut *self.sparse_set_b };
            // Safety:
            // Safe here, because the index stored in group is valid.
            let data_a = unsafe {
                sparse_set_a.data().get_unchecked(*index_a)
            };
            let data_b = unsafe {
                sparse_set_b.data_mut().get_unchecked_mut(*index_b)
            };
            self.index += 1;
            Some((id,(data_a,data_b)))
        } else {
            None
        }
    }
}






pub struct IterMutRef<'a,A,B> {
    index: usize,
    sparse_set_group: *const SparseSet<EntityId,(usize,usize)>,
    sparse_set_a: *mut SparseSet<EntityId,A>,
    sparse_set_b: *const SparseSet<EntityId,B>,
    #[allow(unused)]
    borrow_group: RwLockReadGuard<'a,Box<dyn Group>>,
    #[allow(unused)]
    borrow_a: RwLockWriteGuard<'a,Box<dyn ComponentStorage>>,
    #[allow(unused)]
    borrow_b: RwLockReadGuard<'a,Box<dyn ComponentStorage>>
}

impl<'a,A : Component,B : Component> Queryable<'a> for NonOwning<&'a mut A,&'a B> {
    type Item = (&'a mut A,&'a B);

    fn query(world : &'a World) -> Box<(dyn QueryIterator<Item = Self::Item> + 'a)> {
        assert!(world.has_registered::<A>() && world.has_registered::<B>(),
                "Queryable for NonOwning: Component was not registered in world");
        let type_id_a = TypeId::of::<A>();
        let type_id_b = TypeId::of::<B>();
        // Unwrap here
        // assert before ensures this
        let mut storage_a = world.raw_storage_write(type_id_a).unwrap();
        let storage_b = world.raw_storage_read(type_id_b).unwrap();
        // Safety:
        // storage is SparseSet<EntityId,...>
        let sparse_set_a = unsafe {
            storage_a.downcast_mut::<SparseSet<EntityId,A>>()
        };
        let sparse_set_b = unsafe {
            storage_b.downcast_ref::<SparseSet<EntityId,B>>()
        };
        let ptr_a = &mut *sparse_set_a;
        let ptr_b = &*sparse_set_b;
        let group = non_owning::<A,B>();
        assert!(world.has_group(&group),"Queryable for NonOwning: Group is not in world");
        let group = world.group(&group);
        // Safety:
        // group type is NonOwning<A,B>
        let group_data = unsafe {
            group.downcast_ref::<NonOwning<A,B>>()
        };
        let group_data = &group_data.sparse_set;
        let ptr_group = &*group_data;
        Box::new(IterMutRef{
            index: 0,
            sparse_set_group: ptr_group,
            sparse_set_a: ptr_a,
            sparse_set_b: ptr_b,
            borrow_group: group,
            borrow_a: storage_a,
            borrow_b: storage_b
        })
    }
}

impl<'a,A: Component,B : Component> Iterator for IterMutRef<'a,A,B> {
    type Item = (&'a mut A,&'a B);

    fn next(&mut self) -> Option<Self::Item> {
        // Safety:
        // Safe here, because self.group is 
        // a pointer from borrow,
        // This pointer is valid now.
        let group = unsafe { &*self.sparse_set_group };
        if self.index < group.len() {
            // Safety:
            // we checked in if condition
            let (index_a,index_b) = unsafe {
                group.data().get_unchecked(self.index)
            };
            // Safety:
            // Safe here, because self.sparse_set is 
            // a pointer from borrow,
            // This pointer is valid now.
            let sparse_set_a = unsafe { &mut *self.sparse_set_a };
            let sparse_set_b = unsafe { &*self.sparse_set_b };
            // Safety:
            // Safe here, because the index stored in group is valid.
            let data_a = unsafe {
                sparse_set_a.data_mut().get_unchecked_mut(*index_a)
            };
            let data_b = unsafe {
                sparse_set_b.data().get_unchecked(*index_b)
            };
            self.index += 1;
            Some((data_a,data_b))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.borrow_group.len() - self.index;
        (rem,Some(rem))
    }
}

impl<'a,A : Component,B : Component> ExactSizeIterator for IterMutRef<'a,A,B>{ }

impl<'a,A : Component,B : Component> QueryIterator for IterMutRef<'a,A,B> {
    fn from_id(&mut self, id : EntityId) -> Option<Self::Item> {
        // Safety:
        // Safe here, because these are
        // pointers from borrow,
        // This pointer is valid now.
        let group = unsafe { &*self.sparse_set_group };
        let sparse_set_a = unsafe { &mut *self.sparse_set_a };
        let sparse_set_b = unsafe { &*self.sparse_set_b };
        if let Some((index_a,index_b)) = group.get(id) {
            // Safety:
            // Safe here, because the index stored in group is valid.
            let data_a = unsafe {
                sparse_set_a.data_mut().get_unchecked_mut(*index_a)
            };
            let data_b = unsafe {
                sparse_set_b.data().get_unchecked(*index_b)
            };
            Some((data_a,data_b))
        } else {
            None
        }
    }

    fn next_with_id(&mut self) -> Option<(EntityId,Self::Item)> {
        // Safety:
        // Safe here, because self.group is 
        // a pointer from borrow,
        // This pointer is valid now.
        let group = unsafe { &*self.sparse_set_group };
        if self.index < group.len() {
            // Safety:
            // Safe here, because if condition ensure this
            let id = *unsafe {
                group.entities().get_unchecked(self.index)
            };
            let (index_a,index_b) = unsafe {
                group.data().get_unchecked(self.index)
            };
            // Safety:
            // Safe here, because these are
            // pointers from borrow,
            // This pointer is valid now.
            let sparse_set_a = unsafe { &mut *self.sparse_set_a };
            let sparse_set_b = unsafe { &*self.sparse_set_b };
            // Safety:
            // Safe here, because the index stored in group is valid.
            let data_a = unsafe {
                sparse_set_a.data_mut().get_unchecked_mut(*index_a)
            };
            let data_b = unsafe {
                sparse_set_b.data().get_unchecked(*index_b)
            };
            self.index += 1;
            Some((id,(data_a,data_b)))
        } else {
            None
        }
    }
}







pub struct IterMutMut<'a,A,B> {
    index: usize,
    sparse_set_group: *const SparseSet<EntityId,(usize,usize)>,
    sparse_set_a: *mut SparseSet<EntityId,A>,
    sparse_set_b: *mut SparseSet<EntityId,B>,
    #[allow(unused)]
    borrow_group: RwLockReadGuard<'a,Box<dyn Group>>,
    #[allow(unused)]
    borrow_a: RwLockWriteGuard<'a,Box<dyn ComponentStorage>>,
    #[allow(unused)]
    borrow_b: RwLockWriteGuard<'a,Box<dyn ComponentStorage>>
}

impl<'a,A : Component,B : Component> Queryable<'a> for NonOwning<&'a mut A,&'a mut B> {
    type Item = (&'a mut A,&'a mut B);

    fn query(world : &'a World) -> Box<(dyn QueryIterator<Item = Self::Item> + 'a)> {
        assert!(world.has_registered::<A>() && world.has_registered::<B>(),
                "Queryable for NonOwning: Component was not registered in world");
        let type_id_a = TypeId::of::<A>();
        let type_id_b = TypeId::of::<B>();
        // Unwrap here
        // assert before ensures this
        let mut storage_a = world.raw_storage_write(type_id_a).unwrap();
        let mut storage_b = world.raw_storage_write(type_id_b).unwrap();
        // Safety:
        // storage is SparseSet<EntityId,...>
        let sparse_set_a = unsafe {
            storage_a.downcast_mut::<SparseSet<EntityId,A>>()
        };
        let sparse_set_b = unsafe {
            storage_b.downcast_mut::<SparseSet<EntityId,B>>()
        };
        let ptr_a = &mut *sparse_set_a;
        let ptr_b = &mut *sparse_set_b;
        let group = non_owning::<A,B>();
        assert!(world.has_group(&group),"Queryable for NonOwning: Group is not in world");
        let group = world.group(&group);
        // Safety:
        // group type is NonOwning<A,B>
        let group_data = unsafe {
            group.downcast_ref::<NonOwning<A,B>>()
        };
        let group_data = &group_data.sparse_set;
        let ptr_group = &*group_data;
        Box::new(IterMutMut{
            index: 0,
            sparse_set_group: ptr_group,
            sparse_set_a: ptr_a,
            sparse_set_b: ptr_b,
            borrow_group: group,
            borrow_a: storage_a,
            borrow_b: storage_b
        })
    }
}

impl<'a,A: Component,B : Component> Iterator for IterMutMut<'a,A,B> {
    type Item = (&'a mut A,&'a mut B);

    fn next(&mut self) -> Option<Self::Item> {
        // Safety:
        // Safe here, because self.group is 
        // a pointer from borrow,
        // This pointer is valid now.
        let group = unsafe { &*self.sparse_set_group };
        if self.index < group.len() {
            // Safety:
            // we checked in if condition
            let (index_a,index_b) = unsafe {
                group.data().get_unchecked(self.index)
            };
            // Safety:
            // Safe here, because self.sparse_set is 
            // a pointer from borrow,
            // This pointer is valid now.
            let sparse_set_a = unsafe { &mut *self.sparse_set_a };
            let sparse_set_b = unsafe { &mut *self.sparse_set_b };
            // Safety:
            // Safe here, because the index stored in group is valid.
            let data_a = unsafe {
                sparse_set_a.data_mut().get_unchecked_mut(*index_a)
            };
            let data_b = unsafe {
                sparse_set_b.data_mut().get_unchecked_mut(*index_b)
            };
            self.index += 1;
            Some((data_a,data_b))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.borrow_group.len() - self.index;
        (rem,Some(rem))
    }
}

impl<'a,A : Component,B : Component> ExactSizeIterator for IterMutMut<'a,A,B>{ }

impl<'a,A : Component,B : Component> QueryIterator for IterMutMut<'a,A,B> {
    fn from_id(&mut self, id : EntityId) -> Option<Self::Item> {
        // Safety:
        // Safe here, because these are
        // pointers from borrow,
        // This pointer is valid now.
        let group = unsafe { &*self.sparse_set_group };
        let sparse_set_a = unsafe { &mut *self.sparse_set_a };
        let sparse_set_b = unsafe { &mut *self.sparse_set_b };
        if let Some((index_a,index_b)) = group.get(id) {
            // Safety:
            // Safe here, because the index stored in group is valid.
            let data_a = unsafe {
                sparse_set_a.data_mut().get_unchecked_mut(*index_a)
            };
            let data_b = unsafe {
                sparse_set_b.data_mut().get_unchecked_mut(*index_b)
            };
            Some((data_a,data_b))
        } else {
            None
        }
    }

    fn next_with_id(&mut self) -> Option<(EntityId,Self::Item)> {
        // Safety:
        // Safe here, because self.group is 
        // a pointer from borrow,
        // This pointer is valid now.
        let group = unsafe { &*self.sparse_set_group };
        if self.index < group.len() {
            // Safety:
            // Safe here, because if condition ensure this
            let id = *unsafe {
                group.entities().get_unchecked(self.index)
            };
            let (index_a,index_b) = unsafe {
                group.data().get_unchecked(self.index)
            };
            // Safety:
            // Safe here, because these are
            // pointers from borrow,
            // This pointer is valid now.
            let sparse_set_a = unsafe { &mut *self.sparse_set_a };
            let sparse_set_b = unsafe { &mut *self.sparse_set_b };
            // Safety:
            // Safe here, because the index stored in group is valid.
            let data_a = unsafe {
                sparse_set_a.data_mut().get_unchecked_mut(*index_a)
            };
            let data_b = unsafe {
                sparse_set_b.data_mut().get_unchecked_mut(*index_b)
            };
            self.index += 1;
            Some((id,(data_a,data_b)))
        } else {
            None
        }
    }
}
