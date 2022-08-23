//! # Why not RangeVec<T>?
//! Because T must be `Step`, but the trait is unstable now
use std::{
    cmp::{max, min},
    ops::{Range, RangeBounds},
};

#[derive(Debug, Clone)]
pub struct RangeVec {
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

// # Safety
// Safe only when `overlap(r1,r2)||connected(r1,r2)` is true
#[inline]
unsafe fn merge(r1: Range<usize>, r2: Range<usize>) -> Range<usize> {
    min(r1.start, r2.start)..max(r1.end, r2.end)
}

impl RangeVec {
    pub fn new() -> Self {
        RangeVec { ranges: Vec::new() }
    }

    /// insert range into RangeVec
    pub fn insert_range(&mut self, range: Range<usize>) {
        let mut need_merge_indices = Vec::new();
        let mut first_index = None;
        for (i, current) in self.ranges.iter().enumerate() {
            // find all ranges we need to merge
            if overlap(current, &range) || connected(current, &range) {
                need_merge_indices.push(i);
            }
            // find the last one `range.start > current.end`
            // this can be used to determine the insertion position
            // when need_merge_indice is empty
            if range.start > current.end {
                first_index = Some(i);
            }
        }
        dbg!(&need_merge_indices);
        dbg!(&first_index);
        dbg!(&self.ranges);
        if need_merge_indices.is_empty() {
            // just insert it on index
            let index = first_index.map(|x| x as isize).unwrap_or(-1);
            self.ranges.insert((index + 1).try_into().unwrap(), range);
        } else {
            let mut range = range;
            // # Panic safety
            // When entered this branch, the need_merge_indices cannot be empty.
            let first_index = need_merge_indices.first().copied().unwrap();
            // iterate from back so that we can remove it one by one
            for index in need_merge_indices.into_iter().rev() {
                // # Panic safety
                // index is from big to small.
                // Remove make all indices behind invalid
                let current = self.ranges.remove(index);
                dbg!(index);
                dbg!(&current);
                // # Safety
                // we have checked the overlap and connected before
                range = unsafe { merge(current, range) };
            }
            // insert the result range to t
            self.ranges.insert(first_index, range);
        }
    }

    pub fn insert(&mut self, data: usize) {
        self.insert_range(data..(data + 1));
    }

    pub fn remove(&mut self, data: usize) {}
}

#[cfg(test)]
mod tests {
    use super::RangeVec;

    #[test]
    fn basic_test() {
        unsafe {
            let mut v = RangeVec::new();
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
    }
}
