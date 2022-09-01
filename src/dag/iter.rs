use std::collections::HashMap;

pub struct ChildrenIter<'a, NodeId, EdgeData> {
    pub(super) iter: Option<std::collections::hash_map::Iter<'a, NodeId, EdgeData>>,
}

impl<'a, NodeId, EdgeData> Iterator for ChildrenIter<'a, NodeId, EdgeData>
where
    NodeId: Copy,
{
    type Item = (NodeId, &'a EdgeData);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(iter) = &mut self.iter {
            iter.next().map(|(id, data)| (*id, data))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if let Some(iter) = &self.iter {
            iter.size_hint()
        } else {
            (0, Some(0))
        }
    }
}

impl<'a, NodeId, EdgeData> ExactSizeIterator for ChildrenIter<'a, NodeId, EdgeData> where
    NodeId: Copy
{
}

pub struct ParentsIter<'a, NodeId> {
    pub(super) iter: Option<std::collections::hash_set::Iter<'a, NodeId>>,
}

impl<'a, NodeId> Iterator for ParentsIter<'a, NodeId>
where
    NodeId: Copy,
{
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(iter) = &mut self.iter {
            iter.next().copied()
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if let Some(iter) = &self.iter {
            iter.size_hint()
        } else {
            (0, Some(0))
        }
    }
}

impl<'a, NodeId> ExactSizeIterator for ParentsIter<'a, NodeId> where NodeId: Copy {}

pub struct EdgesIter<'a,NodeId,EdgeData> {
    pub(super) from_iter: std::collections::hash_map::Iter<'a,NodeId,HashMap<NodeId,EdgeData>>,
    pub(super) to_iter: Option<(NodeId,std::collections::hash_map::Iter<'a,NodeId,EdgeData>)>,
}

impl<'a,NodeId,EdgeData> Iterator for EdgesIter<'a,NodeId,EdgeData>
where NodeId: Copy{
    type Item = (NodeId,NodeId,&'a EdgeData);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((from_id,iter)) = self.to_iter.as_mut().as_mut() {
            let from_id = from_id.clone();
            if let Some((to_id,data)) = iter.next() {
                return Some((from_id,to_id.clone(),data))
            }
        }
        // yield None or to_iter is none
        if let Some((from_id,map)) = self.from_iter.next() {
            let to_iter = map.iter();
            self.to_iter.replace((from_id.clone(),to_iter));
            self.next()
        } else {
            None
        }
    }
}