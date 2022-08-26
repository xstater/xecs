#![cfg(test)]
use std::collections::BTreeSet;

use crate::range_set::remove;

use super::{insert, RangeSet};
use rand::Rng;

#[test]
fn basic_insert_test() {
    // basic insert test
    let mut root = None;
    insert(&mut root, 0..5, 0..10);
    assert!(root.is_some());
    {
        let root = root.as_ref().unwrap();
        assert_eq!(root.range, 0..10);
        assert!(root.left.is_some());
        assert!(root.right.is_none());
        {
            let left = root.left.as_ref().unwrap();
            assert_eq!(left.range, 0..5);
            assert!(left.is_leaf());
        }
    }
    // insert a short range
    // this has no effect
    insert(&mut root, 2..3, 0..10);
    assert!(root.is_some());
    {
        let root = root.as_ref().unwrap();
        assert_eq!(root.range, 0..10);
        assert!(root.left.is_some());
        assert!(root.right.is_none());
        {
            let left = root.left.as_ref().unwrap();
            assert_eq!(left.range, 0..5);
            assert!(left.is_leaf());
        }
    }
    // combine test
    insert(&mut root, 5..10, 0..10);
    assert!(root.is_some());
    {
        let root = root.as_ref().unwrap();
        assert_eq!(root.range, 0..10);
        assert!(root.is_leaf());
    }
}

#[test]
fn basic_insert_and_remove_test() {
    // test for remove a whole range
    let mut root = None;
    insert(&mut root, 0..5, 0..10);
    assert!(root.is_some());
    {
        let root = root.as_ref().unwrap();
        assert_eq!(root.range, 0..10);
        assert!(root.left.is_some());
        assert!(root.right.is_none());
        {
            let left = root.left.as_ref().unwrap();
            assert_eq!(left.range, 0..5);
            assert!(left.is_leaf());
        }
    }
    remove(&mut root, 0..5);
    assert!(root.is_none());
    // test for remove partial range
    insert(&mut root, 0..5, 0..10);
    assert!(root.is_some());
    {
        let root = root.as_ref().unwrap();
        assert_eq!(root.range, 0..10);
        assert!(root.left.is_some());
        assert!(root.right.is_none());
        {
            let left = root.left.as_ref().unwrap();
            assert_eq!(left.range, 0..5);
            assert!(left.is_leaf());
        }
    }
    remove(&mut root, 0..2);
    assert!(root.is_some());
    {
        let root = root.as_ref().unwrap();
        assert_eq!(root.range, 0..10);
        assert!(root.left.is_some());
        assert!(root.right.is_none());
        {
            let left = root.left.as_ref().unwrap();
            assert_eq!(left.range, 0..5);
            assert!(left.left.is_none());
            assert!(left.right.is_some());
            {
                let right = left.right.as_ref().unwrap();
                assert_eq!(right.range, 2..5);
                assert!(right.is_leaf())
            }
        }
    }
    // remove a range which does not in seg_tree
    remove(&mut root, 6..7);
    assert!(root.is_some());
    {
        let root = root.as_ref().unwrap();
        assert_eq!(root.range, 0..10);
        assert!(root.left.is_some());
        assert!(root.right.is_none());
        {
            let left = root.left.as_ref().unwrap();
            assert_eq!(left.range, 0..5);
            assert!(left.left.is_none());
            assert!(left.right.is_some());
            {
                let right = left.right.as_ref().unwrap();
                assert_eq!(right.range, 2..5);
                assert!(right.is_leaf())
            }
        }
    }
    // remove cross middle
    remove(&mut root, 3..7);
    assert!(root.is_some());
    {
        let root = root.as_ref().unwrap();
        assert_eq!(root.range, 0..10);
        assert!(root.left.is_some());
        assert!(root.right.is_none());
        {
            let left = root.left.as_ref().unwrap();
            assert_eq!(left.range, 0..5);
            assert!(left.left.is_none());
            assert!(left.right.is_some());
            {
                let right = left.right.as_ref().unwrap();
                assert_eq!(right.range, 2..5);
                assert!(right.left.is_some());
                assert!(right.right.is_none());
                {
                    let left = right.left.as_ref().unwrap();
                    assert_eq!(left.range, 2..3);
                    assert!(left.is_leaf())
                }
            }
        }
    }
}

#[test]
fn rand_insert_test() {
    let mut rng = rand::thread_rng();
    let mut values = Vec::new();
    let mut segs_tree = RangeSet::new();

    let count = 100_000;
    for _ in 0..count {
        let value = rng.gen_range(0..1000000);
        values.push(value);
        segs_tree.insert(value);
    }

    values.sort_unstable();
    values.dedup();

    let s = segs_tree.into_iter().collect::<Vec<_>>();

    for (a, b) in values.into_iter().zip(s.into_iter()) {
        assert_eq!(a, b)
    }
}

#[test]
fn rand_insert_and_remove_test() {
    let mut rng = rand::thread_rng();
    let mut values = BTreeSet::new();
    let mut segs_tree = RangeSet::new();

    let count = 100_000;
    for _ in 0..count {
        let value = rng.gen_range(0..1000000);
        values.insert(value);
        segs_tree.insert(value);
    }

    // randomly choose some values to be removed
    let to_be_removed = values
        .iter()
        // 50% chance to remove a value
        .filter(|_| rng.gen_bool(0.5))
        .copied()
        .collect::<Vec<_>>();

    for value in to_be_removed {
        values.remove(&value);
        segs_tree.remove(value);
    }

    let s = segs_tree.into_iter().collect::<Vec<_>>();

    for (a, b) in values.into_iter().zip(s.into_iter()) {
        assert_eq!(a, b)
    }
}

#[test]
fn increased_insert_test() {
    let mut rng = rand::thread_rng();
    let mut values = Vec::new();
    let mut segs_tree = RangeSet::new();

    let count = 1_000_000;
    let mut value = 1;
    for _ in 0..count {
        values.push(value);
        segs_tree.insert(value);

        if rng.gen_bool(0.5) {
            // we have 50% chance to just increse 1
            value += 1;
        } else {
            let offset = rng.gen_range(1..10);
            value += offset;
        }
    }

    for (a, b) in values.into_iter().zip(segs_tree.into_iter()) {
        assert_eq!(a, b)
    }
}

#[test]
fn rand_range_insert_and_remove_test() {
    let mut rng = rand::thread_rng();
    let mut values = BTreeSet::new();
    let mut segs_tree = RangeSet::new();

    let count = 1_000;
    for _ in 0..count {
        let value = rng.gen_range(0..1_000_000);
        let len = rng.gen_range(0..1_000);
        if rng.gen_bool(0.6) {
            // 60% chance to excute a insertion
            segs_tree.insert_range(value..(value + len));
            for i in value..(value + len) {
                values.insert(i);
            }
        } else {
            // 40% chance to excute a remove
            segs_tree.remove_range(value..(value + len));
            for i in value..(value + len) {
                values.remove(&i);
            }
        }
    }

    let s = segs_tree.into_iter().collect::<Vec<_>>();

    for (a, b) in values.into_iter().zip(s.into_iter()) {
        assert_eq!(a, b)
    }
}

#[test]
fn contains_test() {
    let mut rng = rand::thread_rng();
    let mut values = Vec::new();
    let mut segs_tree = RangeSet::new();

    let count = 100_000;
    for _ in 0..count {
        let value = rng.gen_range(0..1000000);
        values.push(value);
        segs_tree.insert(value);
    }

    values.sort_unstable();
    values.dedup();

    values.into_iter().for_each(|value| {
        assert!(segs_tree.contains(value));
    });
}
