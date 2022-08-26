use std::{num::NonZeroUsize, ops::Range};

#[derive(Debug, Clone)]
struct Node {
    range: Range<usize>,
    middle: usize,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

/// check r1 is inlucde in r2
#[inline]
fn include(r1: &Range<usize>, r2: &Range<usize>) -> bool {
    r2.start <= r1.start && r1.end <= r2.end
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
            return ;
        }
        node.as_mut().unwrap_or_else(|| unreachable!())
    };

    let middle = node.middle;

    if range.start < middle && middle < range.end {
        insert( &mut node.left, range.start..middle, node_range.start..middle);
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

#[derive(Debug, Clone)]
pub struct SegsTree {
    root: Option<Box<Node>>,
}

impl SegsTree {
    pub fn new() -> SegsTree {
        SegsTree { root: None }
    }

    pub fn insert_range(&mut self, range: Range<usize>) {
        insert(&mut self.root, range, 0..std::usize::MAX);
    }

    pub fn insert(&mut self, data: usize) {
        self.insert_range(data..(data + 1));
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

impl IntoIterator for SegsTree {
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
    use super::{SegsTree, insert};
    use rand::Rng;

    #[test]
    fn basic_test() {
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
                assert_eq!(left.range,0..5);
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
                assert_eq!(left.range,0..5);
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
    fn rand_test() {
        let mut rng = rand::thread_rng();
        let mut values = Vec::new();
        let mut segs_tree = SegsTree::new();

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
    fn increased_test() {
        let mut rng = rand::thread_rng();
        let mut values = Vec::new();
        let mut segs_tree = SegsTree::new();

        let count = 10_000_000;
        let mut value = 1;
        for _ in 0..count {
            values.push(value);
            segs_tree.insert(value);

            let dice = rng.gen_range(0..100);
            if dice < 50 {
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
}
