use std::ptr::{null_mut, null};

use crate::{Archetype, ComponentTypeId, EntityId};

#[test]
fn basic_test() {
    let mut archetype = Archetype::new();

    let i32_cid = ComponentTypeId::from_rust_type::<i32>();
    let char_cid = ComponentTypeId::from_rust_type::<char>();

    archetype.create_storage::<i32>(i32_cid);
    archetype.create_storage::<char>(char_cid);

    assert_eq!(archetype.types(), &[i32_cid, char_cid]);
    assert_eq!(archetype.len(), 0);

    let id = EntityId::new(10).unwrap();
    unsafe {
        // first data
        let mut data1 = 114514;
        let mut data2 = 'c';

        let mut buffer = [null_mut(), null_mut()];

        buffer[0] = &mut data1 as *mut i32 as *mut u8;
        buffer[1] = &mut data2 as *mut char as *mut u8;

        archetype.insert_any_and_drop_unchecked(id, &buffer);
    }

    unsafe {
        // check
        assert_eq!(archetype.len(), 1);
        assert!(archetype.contains(id));
        // get data
        let mut buffer = [null(), null()];
        archetype.get_unchecked(id, buffer.as_mut_slice());

        let a = buffer[0] as *const i32;
        let b = buffer[1] as *mut char;

        assert_eq!(*a, 114514);
        assert_eq!(*b, 'c');
    }
}
