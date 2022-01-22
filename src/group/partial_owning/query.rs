use std::{any::TypeId, sync::{RwLockReadGuard, RwLockWriteGuard}};
use crate::{component::{Component, ComponentStorage}, entity::EntityId, group::partial_owning, query::{QueryIterator, Queryable}, sparse_set::SparseSet, world::World};
use super::PartialOwning;

pub struct IterRefRef<'a,A,B> {
    index: usize,
    length: usize,
    sparse_set_a: *const SparseSet<EntityId,A>,
    sparse_set_b: *const SparseSet<EntityId,B>,
    #[allow(unused)]
    borrow_a: RwLockReadGuard<'a,Box<dyn ComponentStorage>>,
    #[allow(unused)]
    borrow_b: RwLockReadGuard<'a,Box<dyn ComponentStorage>>
}

impl<'a,A : Component,B : Component> Queryable<'a> for PartialOwning<&'a A,&'a B> {
    type Item = (&'a A,&'a B);

    fn query(world : &'a World) -> Box<(dyn QueryIterator<Item = Self::Item> + 'a)> {
        assert!(world.has_registered::<A>() && world.has_registered::<B>(),
                "Queryable for PartialOwning: Component was not registered in world");
        let type_id_a = TypeId::of::<A>();
        let type_id_b = TypeId::of::<B>();
        // Unwrap here
        // assert before ensures this
        let storage_a = world.storage_ref(type_id_a).unwrap();
        let storage_b = world.storage_ref(type_id_b).unwrap();
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
        let group = partial_owning::<A,B>();
        assert!(world.has_group(&group),"Queryable for PartialOwning: Group is not in world");
        let length = world.group(&group).len();
        Box::new(IterRefRef {
            index: 0,
            length,
            sparse_set_a: ptr_a,
            sparse_set_b: ptr_b,
            borrow_a: storage_a,
            borrow_b: storage_b
        })
    }
}

impl<'a,A : Component,B : Component> Iterator for IterRefRef<'a,A,B> {
    type Item = (&'a A,&'a B);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.length {
            // Safety:
            // Safe here,because self.sparse_set is 
            // a pointer from borrow,
            // This pointer is valid now.
            let sparse_set_a = unsafe { &*self.sparse_set_a };
            let sparse_set_b = unsafe { &*self.sparse_set_b };
            // Safety:
            // Safe here, because we checked before.
            let id = *unsafe {
                sparse_set_a.entities()
                    .get_unchecked(self.index)
            };
            let data_a = unsafe {
                sparse_set_a.data().get_unchecked(self.index)
            };
            // Unwrap here
            // If this panic,the group are destroyed.
            // But group is matained by internal.
            // So it never fails
            let data_b = sparse_set_b.get(id).unwrap();
            self.index += 1;
            Some((data_a,data_b))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.length - self.index;
        (rem,Some(rem))
    }
}

impl<'a,A: Component,B: Component> ExactSizeIterator for IterRefRef<'a,A,B> {}

impl<'a,A : Component,B : Component> QueryIterator for IterRefRef<'a,A,B> {
    fn from_id(&mut self,id : EntityId) -> Option<Self::Item> {
        // Safety:
        // Safe here,because self.sparse_set is 
        // a pointer from borrow,
        // This pointer is valid now.
        let sparse_set_a = unsafe { &*self.sparse_set_a };
        let sparse_set_b = unsafe { &*self.sparse_set_b };
        if let Some(a) = sparse_set_a.get(id) {
            if let Some(b) = sparse_set_b.get(id) {
                return Some((a,b))
            }
        }
        None
    }

    fn next_with_id(&mut self) -> Option<(EntityId,Self::Item)> {
        if self.index < self.length {
            // Safety:
            // Safe here,because self.sparse_set is 
            // a pointer from borrow,
            // This pointer is valid now.
            let sparse_set_a = unsafe { &*self.sparse_set_a };
            let sparse_set_b = unsafe { &*self.sparse_set_b };
            // Safety:
            // Safe here, because we checked before.
            let id = *unsafe {
                sparse_set_a.entities()
                    .get_unchecked(self.index)
            };
            let data_a = unsafe {
                sparse_set_a.data().get_unchecked(self.index)
            };
            // Unwrap here
            // If this panic,the group are destroyed.
            // But group is matained by internal.
            // So it never fails
            let data_b = sparse_set_b.get(id).unwrap();
            self.index += 1;
            Some((id,(data_a,data_b)))
        } else {
            None
        }
    }
}






pub struct IterRefMut<'a,A,B> {
    index: usize,
    length: usize,
    sparse_set_a: *const SparseSet<EntityId,A>,
    sparse_set_b: *mut SparseSet<EntityId,B>,
    #[allow(unused)]
    borrow_a: RwLockReadGuard<'a,Box<dyn ComponentStorage>>,
    #[allow(unused)]
    borrow_b: RwLockWriteGuard<'a,Box<dyn ComponentStorage>>
}

impl<'a,A : Component,B : Component> Queryable<'a> for PartialOwning<&'a A,&'a mut B> {
    type Item = (&'a A,&'a mut B);

    fn query(world : &'a World) -> Box<(dyn QueryIterator<Item = Self::Item> + 'a)> {
        assert!(world.has_registered::<A>() && world.has_registered::<B>(),
                "Queryable for PartialOwning: Component was not registered in world");
        let type_id_a = TypeId::of::<A>();
        let type_id_b = TypeId::of::<B>();
        // Unwrap here
        // assert before ensures this
        let storage_a = world.storage_ref(type_id_a).unwrap();
        let mut storage_b = world.storage_mut(type_id_b).unwrap();
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
        let group = partial_owning::<A,B>();
        assert!(world.has_group(&group),"Queryable for PartialOwning: Group is not in world");
        let length = world.group(&group).len();
        Box::new(IterRefMut {
            index: 0,
            length,
            sparse_set_a: ptr_a,
            sparse_set_b: ptr_b,
            borrow_a: storage_a,
            borrow_b: storage_b
        })
    }
}

impl<'a,A : Component,B : Component> Iterator for IterRefMut<'a,A,B> {
    type Item = (&'a A,&'a mut B);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.length {
            // Safety:
            // Safe here,because self.sparse_set is 
            // a pointer from borrow,
            // This pointer is valid now.
            let sparse_set_a = unsafe { &*self.sparse_set_a };
            let sparse_set_b = unsafe { &mut *self.sparse_set_b };
            // Safety:
            // Safe here, because we checked before.
            let id = *unsafe {
                sparse_set_a.entities()
                    .get_unchecked(self.index)
            };
            let data_a = unsafe {
                sparse_set_a.data().get_unchecked(self.index)
            };
            // Unwrap here
            // If this panic,the group are destroyed.
            // But group is matained by internal.
            // So it never fails
            let data_b = sparse_set_b.get_mut(id).unwrap();
            self.index += 1;
            Some((data_a,data_b))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.length - self.index;
        (rem,Some(rem))
    }
}

impl<'a,A: Component,B: Component> ExactSizeIterator for IterRefMut<'a,A,B> {}

impl<'a,A : Component,B : Component> QueryIterator for IterRefMut<'a,A,B> {
    fn from_id(&mut self,id : EntityId) -> Option<Self::Item> {
        // Safety:
        // Safe here,because self.sparse_set is 
        // a pointer from borrow,
        // This pointer is valid now.
        let sparse_set_a = unsafe { &*self.sparse_set_a };
        let sparse_set_b = unsafe { &mut *self.sparse_set_b };
        if let Some(a) = sparse_set_a.get(id) {
            if let Some(b) = sparse_set_b.get_mut(id) {
                return Some((a,b))
            }
        }
        None
    }

    fn next_with_id(&mut self) -> Option<(EntityId,Self::Item)> {
        if self.index < self.length {
            // Safety:
            // Safe here,because self.sparse_set is 
            // a pointer from borrow,
            // This pointer is valid now.
            let sparse_set_a = unsafe { &*self.sparse_set_a };
            let sparse_set_b = unsafe { &mut *self.sparse_set_b };
            // Safety:
            // Safe here, because we checked before.
            let id = *unsafe {
                sparse_set_a.entities()
                    .get_unchecked(self.index)
            };
            let data_a = unsafe {
                sparse_set_a.data().get_unchecked(self.index)
            };
            // Unwrap here
            // If this panic,the group are destroyed.
            // But group is matained by internal.
            // So it never fails
            let data_b = sparse_set_b.get_mut(id).unwrap();
            self.index += 1;
            Some((id,(data_a,data_b)))
        } else {
            None
        }
    }
}








pub struct IterMutRef<'a,A,B> {
    index: usize,
    length: usize,
    sparse_set_a: *mut SparseSet<EntityId,A>,
    sparse_set_b: *const SparseSet<EntityId,B>,
    #[allow(unused)]
    borrow_a: RwLockWriteGuard<'a,Box<dyn ComponentStorage>>,
    #[allow(unused)]
    borrow_b: RwLockReadGuard<'a,Box<dyn ComponentStorage>>
}

impl<'a,A : Component,B : Component> Queryable<'a> for PartialOwning<&'a mut A,&'a B> {
    type Item = (&'a mut A,&'a B);

    fn query(world : &'a World) -> Box<(dyn QueryIterator<Item = Self::Item> + 'a)> {
        assert!(world.has_registered::<A>() && world.has_registered::<B>(),
                "Queryable for PartialOwning: Component was not registered in world");
        let type_id_a = TypeId::of::<A>();
        let type_id_b = TypeId::of::<B>();
        // Unwrap here
        // assert before ensures this
        let mut storage_a = world.storage_mut(type_id_a).unwrap();
        let storage_b = world.storage_ref(type_id_b).unwrap();
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
        let group = partial_owning::<A,B>();
        assert!(world.has_group(&group),"Queryable for PartialOwning: Group is not in world");
        let length = world.group(&group).len();
        Box::new(IterMutRef {
            index: 0,
            length,
            sparse_set_a: ptr_a,
            sparse_set_b: ptr_b,
            borrow_a: storage_a,
            borrow_b: storage_b
        })
    }
}

impl<'a,A : Component,B : Component> Iterator for IterMutRef<'a,A,B> {
    type Item = (&'a mut A,&'a B);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.length {
            // Safety:
            // Safe here,because self.sparse_set is 
            // a pointer from borrow,
            // This pointer is valid now.
            let sparse_set_a = unsafe { &mut *self.sparse_set_a };
            let sparse_set_b = unsafe { &*self.sparse_set_b };
            // Safety:
            // Safe here, because we checked before.
            let id = *unsafe {
                sparse_set_a.entities()
                    .get_unchecked(self.index)
            };
            let data_a = unsafe {
                sparse_set_a.data_mut().get_unchecked_mut(self.index)
            };
            // Unwrap here
            // If this panic,the group are destroyed.
            // But group is matained by internal.
            // So it never fails
            let data_b = sparse_set_b.get(id).unwrap();
            self.index += 1;
            Some((data_a,data_b))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.length - self.index;
        (rem,Some(rem))
    }
}

impl<'a,A: Component,B: Component> ExactSizeIterator for IterMutRef<'a,A,B> {}

impl<'a,A : Component,B : Component> QueryIterator for IterMutRef<'a,A,B> {
    fn from_id(&mut self,id : EntityId) -> Option<Self::Item> {
        // Safety:
        // Safe here,because self.sparse_set is 
        // a pointer from borrow,
        // This pointer is valid now.
        let sparse_set_a = unsafe { &mut *self.sparse_set_a };
        let sparse_set_b = unsafe { &*self.sparse_set_b };
        if let Some(a) = sparse_set_a.get_mut(id) {
            if let Some(b) = sparse_set_b.get(id) {
                return Some((a,b))
            }
        }
        None
    }

    fn next_with_id(&mut self) -> Option<(EntityId,Self::Item)> {
        if self.index < self.length {
            // Safety:
            // Safe here,because self.sparse_set is 
            // a pointer from borrow,
            // This pointer is valid now.
            let sparse_set_a = unsafe { &mut *self.sparse_set_a };
            let sparse_set_b = unsafe { &*self.sparse_set_b };
            // Safety:
            // Safe here, because we checked before.
            let id = *unsafe {
                sparse_set_a.entities()
                    .get_unchecked(self.index)
            };
            let data_a = unsafe {
                sparse_set_a.data_mut().get_unchecked_mut(self.index)
            };
            // Unwrap here
            // If this panic,the group are destroyed.
            // But group is matained by internal.
            // So it never fails
            let data_b = sparse_set_b.get(id).unwrap();
            self.index += 1;
            Some((id,(data_a,data_b)))
        } else {
            None
        }
    }
}






pub struct IterMutMut<'a,A,B> {
    index: usize,
    length: usize,
    sparse_set_a: *mut SparseSet<EntityId,A>,
    sparse_set_b: *mut SparseSet<EntityId,B>,
    #[allow(unused)]
    borrow_a: RwLockWriteGuard<'a,Box<dyn ComponentStorage>>,
    #[allow(unused)]
    borrow_b: RwLockWriteGuard<'a,Box<dyn ComponentStorage>>
}

impl<'a,A : Component,B : Component> Queryable<'a> for PartialOwning<&'a mut A,&'a mut B> {
    type Item = (&'a mut A,&'a mut B);

    fn query(world : &'a World) -> Box<(dyn QueryIterator<Item = Self::Item> + 'a)> {
        assert!(world.has_registered::<A>() && world.has_registered::<B>(),
                "Queryable for PartialOwning: Component was not registered in world");
        let type_id_a = TypeId::of::<A>();
        let type_id_b = TypeId::of::<B>();
        // Unwrap here
        // assert before ensures this
        let mut storage_a = world.storage_mut(type_id_a).unwrap();
        let mut storage_b = world.storage_mut(type_id_b).unwrap();
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
        let group = partial_owning::<A,B>();
        assert!(world.has_group(&group),"Queryable for PartialOwning: Group is not in world");
        let length = world.group(&group).len();
        Box::new(IterMutMut {
            index: 0,
            length,
            sparse_set_a: ptr_a,
            sparse_set_b: ptr_b,
            borrow_a: storage_a,
            borrow_b: storage_b
        })
    }
}

impl<'a,A : Component,B : Component> Iterator for IterMutMut<'a,A,B> {
    type Item = (&'a mut A,&'a mut B);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.length {
            // Safety:
            // Safe here,because self.sparse_set is 
            // a pointer from borrow,
            // This pointer is valid now.
            let sparse_set_a = unsafe { &mut *self.sparse_set_a };
            let sparse_set_b = unsafe { &mut *self.sparse_set_b };
            // Safety:
            // Safe here, because we checked before.
            let id = *unsafe {
                sparse_set_a.entities()
                    .get_unchecked(self.index)
            };
            let data_a = unsafe {
                sparse_set_a.data_mut().get_unchecked_mut(self.index)
            };
            // Unwrap here
            // If this panic,the group are destroyed.
            // But group is matained by internal.
            // So it never fails
            let data_b = sparse_set_b.get_mut(id).unwrap();
            self.index += 1;
            Some((data_a,data_b))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = self.length - self.index;
        (rem,Some(rem))
    }
}

impl<'a,A: Component,B: Component> ExactSizeIterator for IterMutMut<'a,A,B> {}

impl<'a,A : Component,B : Component> QueryIterator for IterMutMut<'a,A,B> {
    fn from_id(&mut self,id : EntityId) -> Option<Self::Item> {
        // Safety:
        // Safe here,because self.sparse_set is 
        // a pointer from borrow,
        // This pointer is valid now.
        let sparse_set_a = unsafe { &mut *self.sparse_set_a };
        let sparse_set_b = unsafe { &mut *self.sparse_set_b };
        if let Some(a) = sparse_set_a.get_mut(id) {
            if let Some(b) = sparse_set_b.get_mut(id) {
                return Some((a,b))
            }
        }
        None
    }

    fn next_with_id(&mut self) -> Option<(EntityId,Self::Item)> {
        if self.index < self.length {
            // Safety:
            // Safe here,because self.sparse_set is 
            // a pointer from borrow,
            // This pointer is valid now.
            let sparse_set_a = unsafe { &mut *self.sparse_set_a };
            let sparse_set_b = unsafe { &mut *self.sparse_set_b };
            // Safety:
            // Safe here, because we checked before.
            let id = *unsafe {
                sparse_set_a.entities()
                    .get_unchecked(self.index)
            };
            let data_a = unsafe {
                sparse_set_a.data_mut().get_unchecked_mut(self.index)
            };
            // Unwrap here
            // If this panic,the group are destroyed.
            // But group is matained by internal.
            // So it never fails
            let data_b = sparse_set_b.get_mut(id).unwrap();
            self.index += 1;
            Some((id,(data_a,data_b)))
        } else {
            None
        }
    }
}
