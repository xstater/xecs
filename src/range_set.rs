mod tests;
mod iter;

pub use iter::IntoIter;

use std::ops::Range;

use self::iter::Iter;

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

    pub fn iter(&self) -> Iter<'_> {
        (&self).into_iter()
    }
}
