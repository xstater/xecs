use std::{collections::HashMap, sync::Arc};

use crate::{ComponentAny, ComponentStorage, EntityId};
use parking_lot::RwLock;
use rand::Rng;
use xsparseset::{SparseSet, SparseSetHashMap};

#[test]
fn basic_storage_dyn() {
    let sparse_set: SparseSetHashMap<EntityId, char> = SparseSet::default();
    let mut sparse_set: Box<dyn ComponentStorage> = Box::new(sparse_set);

    let id = EntityId::new(10).unwrap();
    let data: Box<dyn ComponentAny> = Box::new('c');
    sparse_set.insert_any(id, data);
    assert_eq!(sparse_set.len(), 1);
    assert!(sparse_set.contains(id));
    {
        let sparse_set = unsafe { sparse_set.downcast_ref::<SparseSetHashMap<EntityId,char>>() };
        let result = sparse_set.get(id).copied();
        assert!(result.is_some());
        assert_eq!(result.unwrap(),'c');
        assert_eq!(sparse_set.data(),&['c']);
    }

    let id = EntityId::new(14).unwrap();
    let mut ch = 'b';
    unsafe {
        sparse_set.insert_any_unchecked(id, &mut ch as *mut char as *mut _);
    }
    assert_eq!(sparse_set.len(), 2);
    assert!(sparse_set.contains(id));
    {
        let sparse_set = unsafe { sparse_set.downcast_ref::<SparseSetHashMap<EntityId,char>>() };
        let result = sparse_set.get(id).copied();
        assert!(result.is_some());
        assert_eq!(result.unwrap(),'b');
        assert_eq!(sparse_set.data(),&['c','b']);
    }
}

#[test]
fn rand_storage_dyn() {
    let mut rng = rand::thread_rng();
    let sparse_set: SparseSetHashMap<EntityId, String> = SparseSet::default();
    let mut sparse_set: Box<dyn ComponentStorage> = Box::new(sparse_set);

    let mut ids = HashMap::new();
    
    let count = 10_000;
    for _ in 0..count {
        loop {
            let id: usize = rng.gen_range(2..1000000);
            let id = EntityId::new(id).unwrap();
            if ids.contains_key(&id) {
                continue;
            }
            // gen random string
            let mut s = "".to_owned();
            for _ in 0..rng.gen_range(10..50) {
                s.push(rng.gen_range('a'..='z'));
            }
            ids.insert(id, s.clone());
            if rng.gen_bool(0.5) {
                // insert any
                let s = Box::new(s);
                sparse_set.insert_any(id, s);
            } else {
                // insert any unchecked
                let mut s = s;
                // give out the ownership of s to sparseset
                unsafe {
                    sparse_set.insert_any_unchecked(id, &mut s as *mut String as *mut _)
                }
                std::mem::forget(s);
            }
            break;
        }
    }

    assert_eq!(sparse_set.len(),ids.len());
    {
        let sparse_set = unsafe {
            sparse_set.downcast_ref::<SparseSetHashMap<EntityId,String>>()
        };
        for (id,s) in ids.into_iter() {
            assert!(sparse_set.contains(id));
            let result = sparse_set.get(id);
            assert!(result.is_some());
            assert_eq!(&s,result.unwrap());
        }
    }
}

#[test]
fn storage_dyn_drop_test() {
    // This test to ensure all data in storage will be released correctly
    // Define a struct to count how many times of called drop
    // use `RwLock` to ensure `Send + Sync`
    struct Test {
        count: Arc<RwLock<usize>>
    }

    impl Drop for Test {
        fn drop(&mut self) {
            let mut count = self.count.write();
            *count += 1;
        }
    }

    let drop_count = Arc::new(RwLock::new(0));

    let mut rng = rand::thread_rng();
    let sparse_set: SparseSetHashMap<EntityId, Test> = SparseSet::default();
    let mut sparse_set: Box<dyn ComponentStorage> = Box::new(sparse_set);

    let count = 10_000;
    for i in 1..count {
        let id = EntityId::new(i).unwrap();
        if rng.gen_bool(0.5) {
            let data = Box::new(Test {
                count: drop_count.clone()
            });
            sparse_set.insert_any(id, data);
        } else {
            let mut data = Test {
                count: drop_count.clone()
            };
            unsafe {
                sparse_set.insert_any_unchecked(id, &mut data as *mut Test as *mut _)
            };
            std::mem::forget(data);
        }
    }

    let count = sparse_set.len();
    // trig drop
    std::mem::drop(sparse_set);
    
    let drop_count = drop_count.read();
    assert_eq!(*drop_count,count);
}