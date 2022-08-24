//! # RangeSet
//! RangeSet is a `Vec` based data-structure that can efficiently
//! store `usize`
//! # Why not RangeVec<T>?
//! Because T must be `Step`, but the trait is unstable now
//! # Profermance
//! * This is super fast when inserted  values are strictly increased
use std::{
    cmp::{max, min},
    ops::Range,
};

#[derive(Debug, Clone)]
pub struct RangeSet {
    ranges: Vec<Range<usize>>,
}

#[inline]
fn overlap(r1: &Range<usize>, r2: &Range<usize>) -> bool {
    r1.contains(&r2.start) || r2.contains(&r1.start)
}

#[inline]
fn connected(r1: &Range<usize>, r2: &Range<usize>) -> bool {
    r1.end == r2.start || r2.end == r1.start
}

#[inline]
fn need_merged(r1: &Range<usize>, r2: &Range<usize>) -> bool {
    overlap(r1, r2) || connected(r1, r2)
}

// # Safety
// Safe only when `overlap(r1,r2)||connected(r1,r2)` is true
#[inline]
unsafe fn merge(r1: Range<usize>, r2: Range<usize>) -> Range<usize> {
    min(r1.start, r2.start)..max(r1.end, r2.end)
}

impl RangeSet {
    pub fn new() -> Self {
        RangeSet { ranges: Vec::new() }
    }

    /// insert range into RangeVec
    pub fn insert_range(&mut self, range: Range<usize>) {
        // binary search the end points 
        let left_result = self.ranges
            .binary_search_by_key(&range.start, |range|range.start);
        let right_result = self.ranges
            .binary_search_by_key(&range.end, |range|range.end);
        let mut left_index = match left_result {
            Ok(index) => index,
            Err(index) => index.checked_sub(1).unwrap_or(0),
        };
        let mut right_index = match right_result {
            Ok(index) => index + 1,
            Err(index) => index,
        };
        // ckeck ranges in the end points can be merged
        if let Some(left) = (&self.ranges[left_index..right_index]).first() {
            if !need_merged(left, &range) {
                left_index += 1;
            }
        }
        if let Some(right) = (&self.ranges[left_index..right_index]).last() {
            if !need_merged(right, &range) {
                right_index = right_index.checked_sub(1).unwrap_or(0);
            }
        }
        // merge these ranges
        let mut range = range;
        for index in (left_index..right_index).rev() {
            // # Unwrap
            // 
            let removed = self.ranges.remove(index);
            // # Safety
            // 
            range = unsafe { merge(range,removed) };
        }
        self.ranges.insert(left_index,range);
    }

    pub fn insert(&mut self, data: usize) {
        self.insert_range(data..(data + 1));
    }

    pub fn remove_range(&mut self, range: Range<usize>) {
        // binary search the index that the range will be insert
        
    }

    pub fn remove(&mut self, data: usize) {
        self.remove_range(data..(data + 1))
    }
}

pub struct IntoIter {
    remain_ranges: std::vec::IntoIter<Range<usize>>,
    current_range: Option<Range<usize>>,
}

impl IntoIterator for RangeSet {
    type Item = usize;

    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            remain_ranges: self.ranges.into_iter(),
            current_range: None,
        }
    }
}

impl Iterator for IntoIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_range.is_none() {
            // try to fetch one range
            if let Some(range) = self.remain_ranges.next() {
                self.current_range.replace(range);
            } else {
                return None;
            }
        }

        if let Some(range) = self.current_range.as_mut() {
            if let Some(result) = range.next() {
                Some(result)
            } else {
                // Drop the mutable borrow
                std::mem::drop(range);
                self.current_range.take();
                self.next()
            }
        } else {
            unreachable!("The `current_range` should always be Some() in this time")
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::RangeSet;

    #[test]
    fn basic_test() {
        let mut v = RangeSet::new();
        // insert 1 value
        v.insert(5);
        assert_eq!(&v.ranges, &[5..6]);
        v.insert(6);
        assert_eq!(&v.ranges, &[5..7]);
        v.insert(7);
        assert_eq!(&v.ranges, &[5..8]);

        v.insert(10);
        assert_eq!(&v.ranges, &[5..8, 10..11]);
        v.insert(11);
        assert_eq!(&v.ranges, &[5..8, 10..12]);

        v.insert(2);
        assert_eq!(&v.ranges, &[2..3, 5..8, 10..12]);
        v.insert(3);
        assert_eq!(&v.ranges, &[2..4, 5..8, 10..12]);

        // insert ranges
        v.insert_range(15..20);
        assert_eq!(&v.ranges, &[2..4, 5..8, 10..12, 15..20]);
        v.insert_range(13..14);
        assert_eq!(&v.ranges, &[2..4, 5..8, 10..12, 13..14, 15..20]);
        v.insert_range(4..9);
        assert_eq!(&v.ranges, &[2..9, 10..12, 13..14, 15..20]);

        // remove something
    }

    #[test]
    fn rand_test() {
        let mut rng = rand::thread_rng();
        let mut values = Vec::new();
        let mut ranges = RangeSet::new();

        let count = 100_000;
        for _ in 0..count {
            let value = rng.gen_range(0..1000000);
            values.push(value);
            ranges.insert(value);
        }

        values.sort_unstable();
        values.dedup();

        for (a, b) in values.into_iter().zip(ranges.into_iter()) {
            assert_eq!(a, b)
        }
    }

    #[test]
    fn increased_test() {
        let mut rng = rand::thread_rng();
        let mut values = Vec::new();
        let mut ranges = RangeSet::new();

        let count = 10_000_000;
        let mut value = 1;
        for _ in 0..count {
            values.push(value);
            ranges.insert(value);

            let dice = rng.gen_range(0..100);
            if dice < 50 {
                // we have 50% chance to just increse 1
                value += 1;
            } else {
                let offset = rng.gen_range(1..10);
                value += offset;
            }
        }

        for (a, b) in values.into_iter().zip(ranges.into_iter()) {
            assert_eq!(a, b)
        }
    }
}
