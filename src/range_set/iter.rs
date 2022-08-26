use std::ops::Range;

use super::{Node, RangeSet};

#[derive(Debug)]
pub struct IntoIter {
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

#[derive(Debug)]
pub struct Iter<'a> {
    stack: Vec<&'a Box<Node>>,
    range: Option<Range<usize>>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.range.is_none() {
            if let Some(top) = self.stack.pop() {
                let is_leaf = top.is_leaf();
                if let Some(right) = top.right.as_ref() {
                    self.stack.push(right);
                }
                if let Some(left) = top.left.as_ref() {
                    self.stack.push(left);
                }
                if is_leaf {
                    self.range.replace(top.range.clone());
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

impl<'a> IntoIterator for &'a RangeSet {
    type Item = usize;

    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            stack: self.root.as_ref().map(|root| vec![root]).unwrap_or(vec![]),
            range: None,
        }
    }
}