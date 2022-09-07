use parking_lot::RwLock;
use rand::Rng;
use std::{
    collections::HashMap,
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
        archetype.get_ptr_unchecked(id, buffer.as_mut_slice());

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
                archetype.get_ptr_unchecked(id, buffer.as_mut_slice());
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

    assert_eq!(*drop_count.read(), replaced_count + count);
}

#[test]
fn rand_insert_and_remove_and_drop() {
    let mut rng = rand::thread_rng();
    let mut archetype = Archetype::new();

    let drop_count = Arc::new(RwLock::new(0));

    let i32_cid = ComponentTypeId::from_rust_type::<i32>();
    let char_cid = ComponentTypeId::from_rust_type::<char>();
    let test_cid = ComponentTypeId::from_rust_type::<Test>();

    archetype.create_storage::<i32>(i32_cid);
    archetype.create_storage::<char>(char_cid);
    archetype.create_storage::<Test>(test_cid);

    let mut checker = HashMap::new();

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

            checker.insert(id, (data1, data2));

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

    let mut need_removed = Vec::new();
    for id in &archetype.entities {
        if rng.gen_bool(0.233) {
            need_removed.push(*id);
        }
    }

    let count = replaced_count + need_removed.len();
    let mut buffer = [null(), null(), null()];
    for id in need_removed {
        unsafe {
            archetype.get_ptr_unchecked(id, &mut buffer);

            let data1 = &*(buffer[0] as *const i32);
            let data2 = &*(buffer[1] as *const char);

            let checker_data = checker.get(&id).unwrap();

            assert_eq!(&checker_data.0, data1);
            assert_eq!(&checker_data.1, data2);

            archetype.remove_and_drop_unchecked(id)
        }
    }

    assert_eq!(count, *drop_count.read());
}

#[test]
fn rand_insert_and_remove_bacth_and_drop() {
    let mut rng = rand::thread_rng();
    let mut archetype = Archetype::new();

    let drop_count = Arc::new(RwLock::new(0));

    archetype.create_rust_storage::<i32>();
    archetype.create_rust_storage::<char>();
    archetype.create_rust_storage::<Test>();

    let mut checker = HashMap::new();

    let count = 10_000;
    let mut ids = Vec::new();
    let mut ints = Vec::new();
    let mut chars = Vec::new();
    let mut tests = Vec::new();
    for _ in 0..count {
        let mut id;
        loop {
            id = rng.gen_range(1..100000000);
            let eid = EntityId::new(id).unwrap();
            if !checker.contains_key(&eid) {
                break;
            }
        }
        let id = EntityId::new(id).unwrap();

        let int = rng.gen_range(0..100000000);
        let chr = rng.gen_range('a'..='z');

        checker.insert(id, (int, chr));

        ids.push(id);
        ints.push(int);
        chars.push(chr);
        tests.push(Test {
            count: drop_count.clone(),
        });
    }

    let ptrs = [
        &mut ints as *mut _,
        &mut chars as *mut _,
        &mut tests as *mut _,
    ];

    unsafe {
        archetype.insert_any_batch_unchecked(&mut ids, ptrs.as_slice());
    }

    assert_eq!(archetype.len(), checker.len());
    assert_eq!(*drop_count.read(),0);

    {
        let mut buffer = [null(), null(), null()];
        for (id, (int, chr)) in checker.iter() {
            let id = *id;
            let int = *int;
            let chr = *chr;

            unsafe {
                archetype.get_ptr_unchecked(id, &mut buffer);

                let data1 = &*(buffer[0] as *const i32);
                let data2 = &*(buffer[1] as *const char);

                assert_eq!(int, *data1);
                assert_eq!(chr, *data2);
            }
        }
    }

    let start = rng.gen_range(100..300);
    let end = start + rng.gen_range(100..400);

    let (removed_ids, mut removed_data) = unsafe { archetype.remove_batch_unchecked(start..end) };

    let removed_tests = removed_data.pop().unwrap();
    let removed_chars = removed_data.pop().unwrap();
    let removed_ints = removed_data.pop().unwrap();

    std::mem::drop(removed_tests);

    let removed_ints = unsafe {
         removed_ints.downcast::<Vec<i32>>()
    };
    let removed_chars = unsafe {
         removed_chars.downcast::<Vec<char>>()
    };
    
    let removed_count = removed_ids.len();

    let zip = removed_ints.into_iter().zip(removed_chars.into_iter());
    let zip = removed_ids.into_iter().zip(zip);
    for (id,(int,chr)) in zip {
        assert!(checker.contains_key(&id));

        let (checker_int,checker_chr) = checker.get(&id).unwrap();

        assert_eq!(*checker_int,int);
        assert_eq!(*checker_chr,chr);
        assert!(!archetype.contains(id));
    }

    assert_eq!(*drop_count.read(), removed_count);

    std::mem::drop(archetype);
    
    assert_eq!(*drop_count.read(), checker.len());
}
