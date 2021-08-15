use crate::{Component, World, EntityId};
use std::cell::{Ref, RefMut};
use crate::group::Group;
use crate::sparse_set::SparseSet;
use std::any::TypeId;

pub trait Queryable<'a>{
    type Item;

    fn query(world : &'a World) -> Box<dyn Iterator<Item=Self::Item> + 'a>;
}

impl<'a> Queryable<'a> for EntityId {
    type Item = EntityId;

    fn query(world: &'a World) -> Box<dyn Iterator<Item=Self::Item> + 'a> {
        Box::new(world.entities().iter().cloned())
    }
}

macro_rules! build_iter1 {
    (@pointer_type Ref) => { *const SparseSet<EntityId,T> };
    (@pointer_type Mut) => { *mut   SparseSet<EntityId,T> };
    (@to_refcell Ref) => { Ref<'a,SparseSet<EntityId,T>> };
    (@to_refcell Mut) => { RefMut<'a,SparseSet<EntityId,T>> };
    // SAFETY:
    // 1 self.ptr is a pointer to Ref<'a>,and its valid in this time,so deref here is safe
    // 2 index was checked before,so get_unchecked is safe
    (@get_data Ref $self:ident $index:expr) => {
        unsafe {
            (&*$self.ptr).data().get_unchecked($index)
        }
    };
    (@get_data Mut $self:ident $index:expr) => {
        unsafe {
            (&mut *$self.ptr).data_mut().get_unchecked_mut($index)
        }
    };
    (@output_type Ref NoId) => { &'a T };
    (@output_type Mut NoId) => { &'a mut T };
    (@output_type Ref Id) => { (EntityId,&'a T) };
    (@output_type Mut Id) => { (EntityId,&'a mut T) };
    (@output_data NoId $id:ident $data:ident) => { $data };
    (@output_data   Id $id:ident $data:ident) => { ($id,$data) };
    (@get_components Ref $world:ident) => { $world.components_storage_ref::<T>() };
    (@get_components Mut $world:ident) => { $world.components_storage_mut::<T>() };
    (@get_pointer Ref $comp:ident) => { &*$comp     as *const SparseSet<EntityId,T> };
    (@get_pointer Mut $comp:ident) => { &mut *$comp as *mut   SparseSet<EntityId,T> };
    ($iter_name:ident $with_id:ident $ref_type:ident) => {
        impl<'a,T : Component> Queryable<'a> for build_iter1!(@output_type $ref_type $with_id) {
           type Item = build_iter1!(@output_type $ref_type $with_id);

           fn query(world: &'a World) -> Box<dyn Iterator<Item=Self::Item> + 'a> {
               #[allow(unused_mut)]
               let mut comp = build_iter1!(@get_components $ref_type world);
               let ptr = build_iter1!(@get_pointer $ref_type comp);
               Box::new($iter_name {
                   now_index: 0,
                   ptr,
                   borrow: comp
               })
           }
        }
        pub struct $iter_name<'a,T>{
            now_index : usize,
            ptr : build_iter1!(@pointer_type $ref_type),
            borrow : build_iter1!(@to_refcell $ref_type)
        }
        impl<'a,T> Iterator for $iter_name<'a,T> {
            type Item = build_iter1!(@output_type $ref_type $with_id);

            fn next(&mut self) -> Option<Self::Item> {
                if self.now_index < self.borrow.len() {
                    let _id =  unsafe{ &*self.ptr }.entities()[self.now_index];
                    let data = build_iter1!(@get_data $ref_type self self.now_index);
                    self.now_index += 1;
                    Some(build_iter1!(@output_data $with_id _id data))
                } else {
                    None
                }
            }

            fn size_hint(&self) -> (usize,Option<usize>) {
                let rem = self.borrow.len() - self.now_index;
                (rem,Some(rem))
            }
        }
        impl<'a,T> ExactSizeIterator for $iter_name<'a,T> {}
    };
}

build_iter1!(IterRef   NoId Ref);
build_iter1!(IterMut   NoId Mut);
build_iter1!(IterIdRef   Id Ref);
build_iter1!(IterIdMut   Id Mut);

macro_rules! build_iter2 {
    (@pointer_type Ref $name:tt) => { *const SparseSet<EntityId,$name> };
    (@pointer_type Mut $name:tt) => { *mut   SparseSet<EntityId,$name> };
    (@to_refcell Ref $name:tt) => { Ref<'a,SparseSet<EntityId,$name>> };
    (@to_refcell Mut $name:tt) => { RefMut<'a,SparseSet<EntityId,$name>> };
    (@unref Ref $name:tt) => { &'a $name };
    (@unref Mut $name:tt) => { &'a mut $name };
    (@output_type Id $ref_type_a:ident $ref_type_b:ident) => {
        (EntityId,build_iter2!(@unref $ref_type_a A),build_iter2!(@unref $ref_type_b B))
    };
    (@output_type NoId $ref_type_a:ident $ref_type_b:ident) => {
        (build_iter2!(@unref $ref_type_a A),build_iter2!(@unref $ref_type_b B))
    };
    (@output_data Id   $id:expr,$data_a:expr,$data_b:expr) => { ($id,$data_a,$data_b) };
    (@output_data NoId $id:expr,$data_a:expr,$data_b:expr) => { ($data_a,$data_b) };
    // SAFETY:
    // 1 self.ptr is a pointer to Ref<'a>,and its valid in this time,so deref here is safe
    // 2 index was checked before,so get_unchecked is safe
    (@get_id $ptr:expr,$index:expr) => {
        *unsafe {
            (&*$ptr).entities().get_unchecked($index)
        }
    };
    (@get_data Ref $ptr:expr,$index:expr) => {
        unsafe {
            (&*$ptr).data.get_unchecked($index)
        }
    };
    (@get_data Mut $ptr:expr,$index:expr) => {
        unsafe {
            (&mut *$ptr).data.get_unchecked_mut($index)
        }
    };
    (@get_data_from_id Ref $ptr:expr,$id:expr) => {
        unsafe {
            (&*$ptr).get_unchecked($id)
        }
    };
    (@get_data_from_id Mut $ptr:expr,$id:expr) => {
        unsafe {
            (&mut *$ptr).get_unchecked_mut($id)
        }
    };
    (@get_components Ref $world:ident $type:tt) => { $world.components_storage_ref::<$type>() };
    (@get_components Mut $world:ident $type:tt) => { $world.components_storage_mut::<$type>() };
    (@get_pointer Ref $comp:ident $type:tt) => { &    *$comp as *const SparseSet<EntityId,$type> };
    (@get_pointer Mut $comp:ident $type:tt) => { &mut *$comp as *mut   SparseSet<EntityId,$type> };
    (@build_non_owning_iter $iter_name:ident $with_id:ident $ref_type_a:ident $ref_type_b:ident) => {
        struct $iter_name<'a,A,B> {
            #[allow(dead_code)]
            borrow_a : build_iter2!(@to_refcell $ref_type_a A),
            #[allow(dead_code)]
            borrow_b : build_iter2!(@to_refcell $ref_type_b B),
            entities : Option<Ref<'a,[EntityId]>>,
            indices : Option<Ref<'a,[(usize,usize)]>>,
            ptr_a : build_iter2!(@pointer_type $ref_type_a A),
            ptr_b : build_iter2!(@pointer_type $ref_type_b B)
        }
        impl<'a,A,B> Iterator for $iter_name<'a,A,B> {
            type Item = build_iter2!(@output_type $with_id $ref_type_a $ref_type_b);

            fn next(&mut self) -> Option<Self::Item> {
                let entities = self.entities.take()?;
                let indices = self.indices.take()?;
                if !entities.is_empty() {
                    let (id,rem) = Ref::map_split(
                        entities,
                        |slice|slice.split_first().unwrap());
                    let _id = *id;
                    self.entities = Some(rem);
                    let (index,rem) = Ref::map_split(
                        indices,
                        |slice|slice.split_first().unwrap());
                    let (index_a,index_b) = *index;
                    self.indices = Some(rem);
                    let data_a = build_iter2!(@get_data $ref_type_a self.ptr_a,index_a);
                    let data_b = build_iter2!(@get_data $ref_type_b self.ptr_b,index_b);
                    Some(build_iter2!(@output_data $with_id _id,data_a,data_b))
                } else {
                    None
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                if let Some(entities) = &self.entities {
                    let len = entities.len();
                    (len,Some(len))
                } else {
                    (0,Some(0))
                }
            }
        }
        impl<'a,A,B> ExactSizeIterator for $iter_name<'a,A,B> {}
    };
    (@build_partial_iter A $iter_name:ident $with_id:ident $ref_type_a:ident $ref_type_b:ident) => {
        struct $iter_name<'a,A,B>{
            #[allow(dead_code)]
            borrow_a : build_iter2!(@to_refcell $ref_type_a A),
            #[allow(dead_code)]
            borrow_b : build_iter2!(@to_refcell $ref_type_b B),
            now_index : usize,
            len : usize,
            ptr_a : build_iter2!(@pointer_type $ref_type_a A),
            ptr_b : build_iter2!(@pointer_type $ref_type_b B)
        }
        impl<'a,A,B> Iterator for $iter_name<'a,A,B> {
            type Item = build_iter2!(@output_type $with_id $ref_type_a $ref_type_b);

            fn next(&mut self) -> Option<Self::Item> {
                if self.now_index < self.len {
                    let _id = *unsafe {
                        (&*self.ptr_a).entities().get_unchecked(self.now_index)
                    };
                    let data_a = build_iter2!(@get_data $ref_type_a self.ptr_a,self.now_index);
                    let data_b = build_iter2!(@get_data_from_id $ref_type_b self.ptr_b,_id);
                    self.now_index += 1;
                    Some(build_iter2!(@output_data $with_id _id,data_a,data_b))
                } else {
                    None
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let rem = self.len - self.now_index;
                (rem,Some(rem))
            }
        }
        impl<'a,A,B> ExactSizeIterator for $iter_name<'a,A,B> {}
    };
    (@build_partial_iter B $iter_name:ident $with_id:ident $ref_type_a:ident $ref_type_b:ident) => {
        struct $iter_name<'a,A,B>{
            #[allow(dead_code)]
            borrow_a : build_iter2!(@to_refcell $ref_type_a A),
            #[allow(dead_code)]
            borrow_b : build_iter2!(@to_refcell $ref_type_b B),
            now_index : usize,
            len : usize,
            ptr_a : build_iter2!(@pointer_type $ref_type_a A),
            ptr_b : build_iter2!(@pointer_type $ref_type_b B)
        }
        impl<'a,A,B> Iterator for $iter_name<'a,A,B> {
            type Item = build_iter2!(@output_type $with_id $ref_type_a $ref_type_b);

            fn next(&mut self) -> Option<Self::Item> {
                if self.now_index < self.len {
                    let _id = *unsafe {
                        (&*self.ptr_b).entities().get_unchecked(self.now_index)
                    };
                    let data_a = build_iter2!(@get_data_from_id $ref_type_a self.ptr_a,_id);
                    let data_b = build_iter2!(@get_data $ref_type_b self.ptr_b,self.now_index);
                    self.now_index += 1;
                    Some(build_iter2!(@output_data $with_id _id,data_a,data_b))
                } else {
                    None
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let rem = self.len - self.now_index;
                (rem,Some(rem))
            }
        }
        impl<'a,A,B> ExactSizeIterator for $iter_name<'a,A,B> {}
    };
    (@build_full_group_iter $iter_name:ident $with_id:ident $ref_type_a:ident $ref_type_b:ident) => {
        pub struct $iter_name<'a,A,B> {
            #[allow(dead_code)]
            borrow_a : build_iter2!(@to_refcell $ref_type_a A),
            #[allow(dead_code)]
            borrow_b : build_iter2!(@to_refcell $ref_type_b B),
            ptr_a : build_iter2!(@pointer_type $ref_type_a A),
            ptr_b : build_iter2!(@pointer_type $ref_type_b B),
            now_index : usize,
            len : usize
        }
        impl<'a,A,B> Iterator for $iter_name<'a,A,B> {
            type Item = build_iter2!(@output_type $with_id $ref_type_a $ref_type_b);

            fn next(&mut self) -> Option<Self::Item> {
                if self.now_index < self.len {
                    let _id = *unsafe {
                        (&*self.ptr_a).entities().get_unchecked(self.now_index)
                    };
                    let data_a = build_iter2!(@get_data $ref_type_a self.ptr_a,self.now_index);
                    let data_b = build_iter2!(@get_data $ref_type_b self.ptr_b,self.now_index);
                    self.now_index += 1;
                    Some(build_iter2!(@output_data $with_id _id,data_a,data_b))
                } else {
                    None
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let rem = self.len - self.now_index;
                (rem,Some(rem))
            }
        }
        impl<'a,A,B> ExactSizeIterator for $iter_name<'a,A,B>{}
    };
    (@build_iter $iter_name:ident $with_id:ident $ref_type_a:ident $ref_type_b:ident) => {
        pub struct $iter_name<'a,A,B> {
            borrow_a : build_iter2!(@to_refcell $ref_type_a A),
            borrow_b : build_iter2!(@to_refcell $ref_type_b B),
            now_index : usize,
            ptr_a : build_iter2!(@pointer_type $ref_type_a A),
            ptr_b : build_iter2!(@pointer_type $ref_type_b B)
        }
        impl<'a,A,B> Iterator for $iter_name<'a,A,B> {
            type Item = build_iter2!(@output_type $with_id $ref_type_a $ref_type_b);

            fn next(&mut self) -> Option<Self::Item> {
                while self.now_index < self.borrow_a.len() {
                    let _id = *unsafe {
                        (&*self.ptr_a).entities.get_unchecked(self.now_index)
                    };
                    if let Some(index_b) = self.borrow_b.get_index(_id) {
                        let data_a = build_iter2!(@get_data $ref_type_a self.ptr_a,self.now_index);
                        let data_b = build_iter2!(@get_data $ref_type_b self.ptr_b,index_b);
                        self.now_index += 1;
                        return Some(build_iter2!(@output_data $with_id _id,data_a,data_b))
                    }
                    self.now_index += 1;
                }
                None
            }
        }
    };
    (   $iter_name:ident
        $iter_name_full_group:ident
        $iter_name_partial_a_group:ident
        $iter_name_partial_b_group:ident
        $iter_name_non_owning_group:ident
        $with_id:ident
        $ref_type_a:ident
        $ref_type_b:ident) => {
        impl<'a,A : Component,B : Component> Queryable<'a> for
            build_iter2!(@output_type $with_id $ref_type_a $ref_type_b){
            type Item = build_iter2!(@output_type $with_id $ref_type_a $ref_type_b);

            fn query(world: &'a World) -> Box<dyn Iterator<Item=Self::Item> + 'a> {
                #[allow(unused_mut)]
                let mut comp_a = build_iter2!(@get_components $ref_type_a world A);
                #[allow(unused_mut)]
                let mut comp_b = build_iter2!(@get_components $ref_type_b world B);
                let ptr_a = build_iter2!(@get_pointer $ref_type_a comp_a A);
                let ptr_b = build_iter2!(@get_pointer $ref_type_b comp_b B);
                if let Some(group) = world.group::<A,B>(){
                    let group_ref = Ref::clone(&group);
                    match &*group {
                        Group::Owning(group) => {
                            if group.full() {
                                Box::new($iter_name_full_group {
                                    borrow_a : comp_a,
                                    borrow_b : comp_b,
                                    ptr_a,
                                    ptr_b,
                                    now_index : 0,
                                    len : group.length
                                })
                            } else {
                                if group.is_owned(TypeId::of::<A>()) {
                                    Box::new($iter_name_partial_a_group {
                                        borrow_a : comp_a,
                                        borrow_b : comp_b,
                                        now_index : 0,
                                        len : group.length,
                                        ptr_a,
                                        ptr_b,
                                    })
                                } else {
                                    Box::new($iter_name_partial_b_group {
                                        borrow_a : comp_a,
                                        borrow_b : comp_b,
                                        now_index : 0,
                                        len : group.length,
                                        ptr_a,
                                        ptr_b,
                                    })
                                }
                            }
                        }
                        Group::NonOwning(_) => {
                            let (entities,indices) = Ref::map_split(
                                group_ref,
                                |group|{
                                    match &group {
                                        Group::Owning(_) => { unreachable!() }
                                        Group::NonOwning(group) => {
                                            unsafe { group.sparse_set.data_with_id() }
                                        }
                                    }
                                }
                            );
                            Box::new($iter_name_non_owning_group {
                                borrow_a : comp_a,
                                borrow_b : comp_b,
                                entities : Some(entities),
                                indices : Some(indices),
                                ptr_a,
                                ptr_b
                            })
                        }
                    }
                } else {
                    Box::new($iter_name{
                        borrow_a: comp_a,
                        borrow_b: comp_b,
                        now_index: 0,
                        ptr_a,
                        ptr_b
                    })
                }
            }
        }
        build_iter2!(@build_iter $iter_name $with_id $ref_type_a $ref_type_b);
        build_iter2!(@build_full_group_iter $iter_name_full_group $with_id $ref_type_a $ref_type_b);
        build_iter2!(@build_partial_iter A $iter_name_partial_a_group $with_id $ref_type_a $ref_type_b);
        build_iter2!(@build_partial_iter B $iter_name_partial_b_group $with_id $ref_type_a $ref_type_b);
        build_iter2!(@build_non_owning_iter $iter_name_non_owning_group $with_id $ref_type_a $ref_type_b);
    };
}

build_iter2!(IterRefRef IterFullRefRef IterPartialARefRef IterPartialBRefRef IterNonOwningRefRef NoId Ref Ref);
build_iter2!(IterRefMut IterFullRefMut IterPartialARefMut IterPartialBRefMut IterNonOwningRefMut NoId Ref Mut);
build_iter2!(IterMutRef IterFullMutRef IterPartialAMutRef IterPartialBMutRef IterNonOwningMutRef NoId Mut Ref);
build_iter2!(IterMutMut IterFullMutMut IterPartialAMutMut IterPartialBMutMut IterNonOwningMutMut NoId Mut Mut);
build_iter2!(IterIdRefRef IterFullIdRefRef IterPartialAIdRefRef IterPartialBIdRefRef IterNonOwningIdRefRef Id Ref Ref);
build_iter2!(IterIdRefMut IterFullIdRefMut IterPartialAIdRefMut IterPartialBIdRefMut IterNonOwningIdRefMut Id Ref Mut);
build_iter2!(IterIdMutRef IterFullIdMutRef IterPartialAIdMutRef IterPartialBIdMutRef IterNonOwningIdMutRef Id Mut Ref);
build_iter2!(IterIdMutMut IterFullIdMutMut IterPartialAIdMutMut IterPartialBIdMutMut IterNonOwningIdMutMut Id Mut Mut);

#[cfg(test)]
mod tests{
    use crate::{World, EntityId};
    use std::any::TypeId;

    #[test]
    fn query1_test() {
        let mut world = World::new();

        world.register::<char>();

        world.create_entity().attach('a');
        world.create_entity().attach('b').into_id();
        world.create_entity().attach('c');

        let ids = world.query::<EntityId>().collect::<Vec<_>>();
        dbg!(ids);

        let asciis = world.query::<&mut char>()
            .map(|ch|*ch as u32)
            .collect::<Vec<_>>();
        dbg!(asciis);

        let chars = world.query::<&mut char>()
            .collect::<Vec<_>>();
        dbg!(chars);

        let ent = world.query::<(EntityId,&mut char)>()
            .collect::<Vec<_>>();
        dbg!(ent);
    }

    #[test]
    fn query2_test() {
        let mut world = World::new();

        world.register::<u32>();
        world.register::<char>();

        dbg!(TypeId::of::<u32>());
        dbg!(TypeId::of::<char>());
        // dbg!(&world);

        // world.make_group::<char,u32>(true,true);
        world.make_group::<char,u32>(false,false);


        world.create_entity().attach('c');
        world.create_entity().attach('a').attach(1u32);
        world.create_entity().attach('b').attach(2u32);
        world.create_entity().attach('f');
        world.create_entity().attach('d').attach(3u32);
        world.create_entity().attach('g');
        world.create_entity().attach(4u32);

        for (id,a,b) in world.query::<(EntityId,&char,&u32)>() {
            println!("{}:{:?},{}",id,a,b)
        }
    }
}