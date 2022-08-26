use std::ops::Range;

#[derive(Debug)]
struct Node {
    range: Range<usize>,
    middle: usize,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

/// r1 :    |----------|
/// r2 : |---------------| -> true
#[inline]
fn include(r1: &Range<usize>, r2: &Range<usize>) -> bool {
    r2.start <= r1.start && r1.end <= r2.end
}

/// calculate the remain of range
/// r1 must be included in r2
/// r1 ：    |---|
/// r2 : |----------|
/// l  : |--|
/// r  :         |--|
#[inline]
fn remain(r1: &Range<usize>, r2: &Range<usize>) -> (Range<usize>, Range<usize>) {
    (r2.start..r1.start, r1.end..r2.end)
}

impl Node {
    fn new(range: Range<usize>) -> Node {
        // Use u128 to avoid overflow
        let middle = (range.start as u128 + range.end as u128) / 2;
        Node {
            range,
            middle: middle.try_into().unwrap_or_else(|_| unreachable!()),
            left: None,
            right: None,
        }
    }

    fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    #[inline]
    fn create_left(&mut self) -> &mut Box<Node> {
        let left = Node::new(self.range.start..self.middle);
        self.left.replace(Box::new(left));
        self.left.as_mut().unwrap_or_else(|| unreachable!())
    }

    #[inline]
    fn create_right(&mut self) -> &mut Box<Node> {
        let right = Node::new(self.middle..self.range.end);
        self.right.replace(Box::new(right));
        self.right.as_mut().unwrap_or_else(|| unreachable!())
    }
}

fn insert(node: &mut Option<Box<Node>>, range: Range<usize>, node_range: Range<usize>) {
    if range.start >= range.end {
        return;
    }
    let node = if let Some(node) = node {
        // already has a node
        // and its a leaf
        // and include this range
        // we don't need insert it again
        if node.is_leaf() && include(&range, &node_range) {
            return;
        }
        node
    } else {
        if node_range.start >= node_range.end {
            return;
        }
        let new_node = Node::new(node_range.clone());
        node.replace(Box::new(new_node));
        if range == node_range {
            return;
        }
        node.as_mut().unwrap_or_else(|| unreachable!())
    };

    let middle = node.middle;

    if range.start < middle && middle < range.end {
        insert(
            &mut node.left,
            range.start..middle,
            node_range.start..middle,
        );
        insert(&mut node.right, middle..range.end, middle..node_range.end);
    } else if range.end <= middle {
        insert(&mut node.left, range, node_range.start..middle);
    } else if middle <= range.start {
        insert(&mut node.right, range, middle..node_range.end);
    } else {
        unreachable!();
    }

    // combine
    let mut need_combine = false;
    if let Some(left) = &node.left {
        if let Some(right) = &node.right {
            if left.is_leaf() && right.is_leaf() {
                need_combine = true;
            }
        }
    }
    if need_combine {
        node.left.take();
        node.right.take();
    }
}

fn remove(raw_node: &mut Option<Box<Node>>, range: Range<usize>) {
    if range.start >= range.end {
        return;
    }
    if let Some(node) = raw_node {
        if node.is_leaf() {
            if node.range == range {
                // Just remove itself
                raw_node.take();
                return;
            }
            if include(&range, &node.range) {
                let (left, right) = remain(&range, &node.range);
                let middle = node.middle;
                if left.start < left.end {
                    // left is cross the middle
                    if left.start < middle && middle < left.end {
                        insert(&mut node.left, left.start..middle, node.range.start..middle);
                        insert(&mut node.right, middle..left.end, middle..node.range.end);
                    } else if left.end <= middle {
                        insert(&mut node.left, left, node.range.start..middle);
                    } else {
                        unreachable!(
                            "The left range from result of remain() cannot be in right of node, left:{:?},node:{:?}",
                            &left,&node.range
                        );
                    }
                }
                if right.start < right.end {
                    if right.start < middle && middle < right.end {
                        insert(
                            &mut node.left,
                            right.start..middle,
                            node.range.start..middle,
                        );
                        insert(&mut node.right, middle..right.end, middle..node.range.end);
                    } else if middle <= right.start {
                        insert(&mut node.right, right, middle..node.range.end);
                    } else {
                        unreachable!(
                            "The right range from result of remain() cannot be in left of node, right:{:?},node:{:?}",
                            &right,&node.range
                        );
                    }
                }
                return;
            }
        } else {
            // not the leaf
            let middle = node.middle;
            if range.start < middle && middle < range.end {
                remove(&mut node.left, range.start..middle);
                remove(&mut node.right, middle..range.end);
            } else if range.end <= middle {
                remove(&mut node.left, range);
            } else if middle <= range.start {
                remove(&mut node.right, range);
            } else {
                unreachable!();
            }
            // if remove action make this node be a leaf
            // remove itself
            if node.is_leaf() {
                raw_node.take();
            }
        }
    }
}

fn has(node: &Option<Box<Node>>, range: Range<usize>) -> bool {
    if range.start >= range.end {
        return false;
    }
    if let Some(node) = node {
        if node.is_leaf() {
            return true;
        } else {
            let middle = node.middle;
            if range.start < middle && middle < range.end {
                return has(&node.left, range.start..middle) && has(&node.right, middle..range.end);
            } else if range.end <= middle {
                return has(&node.left, range);
            } else if middle <= range.start {
                return has(&node.right, range);
            }
            unreachable!()
        }
    } 
    false
}

#[derive(Debug)]
pub struct RangeSet {
    root: Option<Box<Node>>,
}

impl RangeSet {
    pub fn new() -> RangeSet {
        RangeSet { root: None }
    }

    pub fn insert_range(&mut self, range: Range<usize>) {
        insert(&mut self.root, range, 0..std::usize::MAX);
    }

    pub fn insert(&mut self, data: usize) {
        self.insert_range(data..(data + 1));
    }

    pub fn remove_range(&mut self, range: Range<usize>) {
        remove(&mut self.root, range)
    }

    pub fn remove(&mut self, data: usize) {
        self.remove_range(data..(data + 1))
    }

    pub fn contains_range(&self, range: Range<usize>) -> bool {
        has(&self.root, range)
    }

    pub fn contains(&self, data: usize) -> bool {
        self.contains_range(data..(data + 1))
    }
}

pub struct IntoIter {
    // Self reference
    stack: Vec<Box<Node>>,
    range: Option<Range<usize>>,
}

impl Iterator for IntoIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.range.is_none() {
            if let Some(mut top) = self.stack.pop() {
                let is_leaf = top.is_leaf();
                if let Some(right) = top.right.take() {
                    self.stack.push(right);
                }
                if let Some(left) = top.left.take() {
                    self.stack.push(left);
                }
                if is_leaf {
                    self.range.replace(top.range);
                } else {
                    continue;
                }
            } else {
                return None;
            }
        }

        let range = self.range.as_mut().unwrap_or_else(|| unreachable!());
        if let Some(result) = range.next() {
            Some(result)
        } else {
            // Drop the mutable borrow
            std::mem::drop(range);
            self.range.take();
            self.next()
        }
    }
}

impl IntoIterator for RangeSet {
    type Item = usize;

    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            stack: self.root.map(|root| vec![root]).unwrap_or(vec![]),
            range: None,
        }
    }
}

#[cfg(test)]
mod tests {
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
}
