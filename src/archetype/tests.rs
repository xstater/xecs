use parking_lot::RwLock;
use rand::Rng;
use std::{
    collections::{HashMap, HashSet},
    ptr::{null, null_mut},
    sync::Arc,
};

use crate::{Archetype, ComponentTypeId, EntityId};

struct Test {
    count: Arc<RwLock<usize>>,
}

impl Drop for Test {
    fn drop(&mut self) {
        let mut count = self.count.write();
        *count += 1;
    }
}

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

#[test]
fn rand_insert() {
    let mut rng = rand::thread_rng();
    let mut archetype = Archetype::new();

    let i32_cid = ComponentTypeId::from_rust_type::<i32>();
    let char_cid = ComponentTypeId::from_rust_type::<char>();

    archetype.create_storage::<i32>(i32_cid);
    archetype.create_storage::<char>(char_cid);

    let mut ids = HashMap::new();

    let count = 100_000;
    {
        let mut buffer = [null_mut(), null_mut()];

        for _ in 0..count {
            let id = rng.gen_range(1..1_000_000);
            let id = EntityId::new(id).unwrap();

            let mut data1 = rng.gen_range(0..10_000_000);
            let mut data2 = rng.gen_range('a'..='z');

            ids.insert(id, (data1, data2));

            unsafe {
                buffer[0] = &mut data1 as *mut i32 as *mut u8;
                buffer[1] = &mut data2 as *mut char as *mut u8;

                archetype.insert_any_and_drop_unchecked(id, &buffer);

                std::mem::forget(data1);
                std::mem::forget(data2);
            }
        }
    }

    assert_eq!(archetype.len(), ids.len());

    {
        let mut buffer = [null(), null()];

        for (id, (data1, data2)) in ids {
            unsafe {
                archetype.get_unchecked(id, buffer.as_mut_slice());
                let int = &*(buffer[0] as *const i32);
                let chr = &*(buffer[1] as *const char);

                assert_eq!(&data1, int);
                assert_eq!(&data2, chr);
            }
        }
    }
}

#[test]
fn rand_insert_drop() {
    let mut rng = rand::thread_rng();
    let mut archetype = Archetype::new();

    let drop_count = Arc::new(RwLock::new(0));

    let i32_cid = ComponentTypeId::from_rust_type::<i32>();
    let char_cid = ComponentTypeId::from_rust_type::<char>();
    let test_cid = ComponentTypeId::from_rust_type::<Test>();

    archetype.create_storage::<i32>(i32_cid);
    archetype.create_storage::<char>(char_cid);
    archetype.create_storage::<Test>(test_cid);

    let count = 10_000;
    let mut replaced_count = 0;
    {
        let mut buffer = [null_mut(), null_mut(), null_mut()];

        for _ in 0..count {
            let id = rng.gen_range(1..1_000_000);
            let id = EntityId::new(id).unwrap();

            let mut data1 = rng.gen_range(0..10_000_000);
            let mut data2 = rng.gen_range('a'..='z');
            let mut data3 = Test {
                count: drop_count.clone(),
            };

            if archetype.contains(id) {
                replaced_count += 1;
            }

            unsafe {
                buffer[0] = &mut data1 as *mut i32 as *mut u8;
                buffer[1] = &mut data2 as *mut char as *mut u8;
                buffer[2] = &mut data3 as *mut Test as *mut u8;

                archetype.insert_any_and_drop_unchecked(id, &buffer);

                std::mem::forget(data1);
                std::mem::forget(data2);
                std::mem::forget(data3);
            }
        }
    }

    {
        assert_eq!(*drop_count.read(), replaced_count);
    }

    let count = archetype.len();

    std::mem::drop(archetype);
    
    assert_eq!(*drop_count.read(),replaced_count + count);

}

#[test]
fn rand_insert_and_remove() {

}