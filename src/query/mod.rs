pub mod query;
pub mod query2;
pub mod query3;

pub use query::{Query,QueryEntity};
pub use query2::{Query2,QueryEntity2};

pub(in crate::query) unsafe fn distance_ptr<T>(ptr_left : *const T,ptr_right : *const T) -> isize {
    if std::mem::size_of::<T>() > 0 {
        ptr_right.offset_from(ptr_left)
    }else{
        (ptr_right as isize) - (ptr_left as isize)
    }
}

pub(in crate::query) unsafe fn distance_mut_ptr<T>(ptr_left : *mut T,ptr_right : *mut T) -> isize {
    if std::mem::size_of::<T>() > 0 {
        ptr_right.offset_from(ptr_left)
    }else{
        (ptr_right as isize) - (ptr_left as isize)
    }
}

pub(in crate::query) unsafe fn add_ptr<T>(ptr : *const T,offset : usize) -> *const T {
    if std::mem::size_of::<T>() > 0 {
        ptr.offset(offset as isize)
    }else{
        ((ptr as usize) + offset) as *const _
    }
}

pub(in crate::query) unsafe fn add_mut_ptr<T>(ptr : *mut T,offset : usize) -> *mut T {
    if std::mem::size_of::<T>() > 0 {
        ptr.offset(offset as isize)
    }else{
        ((ptr as usize) + offset) as *mut _
    }
}
#[cfg(test)]
mod tests{
    use crate::World;

    #[test]
    fn basic_test() {
        let mut world = World::new();

        struct Tag;

        world
            .register::<u32>()
            .register::<char>()
            .register::<Tag>();

        world.create_entity(1u32);
        world.create_entity(2u32);
        world.create_entity(3u32)
            .with('a');
        world.create_entity(4u32)
            .with('b')
            .with(Tag);
        world.create_entity(5u32)
            .with('c');
        world.create_entity(6u32)
            .with(Tag);
        world.create_entity('d');
        world.create_entity(7u32)
            .with('e');
        world.create_entity(8u32);
        world.create_entity('f')
            .with(Tag);

        let v = world.make_query::<u32>()
            .query()
            .collect::<Vec<_>>();
        assert_eq!(v,[&1,&2,&3,&4,&5,&6,&7,&8]);

        let v = world.make_query::<char>()
            .query()
            .collect::<Vec<_>>();
        assert_eq!(v,[&'a',&'b',&'c',&'d',&'e',&'f']);

        let v = world.make_query::<char>()
            .entities()
            .query()
            .collect::<Vec<_>>();
        assert_eq!(v,[(2,&'a'),(3,&'b'),(4,&'c'),(6,&'d'),(7,&'e'),(9,&'f')]);

        let v = world.make_query::<Tag>()
            .entities()
            .query()
            .map(|(eid,_)|eid)
            .collect::<Vec<_>>();
        assert_eq!(v,[3,5,9]);

        let v = world.make_query::<u32>()
            .with::<char>()
            .query()
            .collect::<Vec<_>>();
        assert_eq!(v,[(&3,&'a'),(&4,&'b'),(&5,&'c'),(&7,&'e')]);

        world.make_group::<char,u32>();
        let v = world.make_query::<u32>()
            .with::<char>()
            .query()
            .collect::<Vec<_>>();
        assert_eq!(v,[(&3,&'a'),(&4,&'b'),(&5,&'c'),(&7,&'e')]);

        let v = world.make_query::<Tag>()
            .with::<char>()
            .query()
            .map(|(_,ch)|ch)
            .collect::<Vec<_>>();
        assert_eq!(v,[&'b',&'f']);

        let v = world.make_query::<Tag>()
            .entities()
            .with::<char>()
            // .entities()
            .query()
            .map(|(eid,_,ch)|(eid,ch))
            .collect::<Vec<_>>();
        assert_eq!(v,[(3,&'b'),(9,&'f')]);

        // let v = world.make_query::<>
    }
}

