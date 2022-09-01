mod error;
mod iter;
#[cfg(test)]
mod tests;

pub use error::DagError;
pub use iter::{ChildrenIter, ParentsIter};

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

#[derive(Debug, Clone)]
pub struct Dag<NodeId, NodeData, EdgeData> {
    nodes: HashMap<NodeId, NodeData>,
    edges: HashMap<NodeId, HashMap<NodeId, EdgeData>>,
    back_edges: HashMap<NodeId, HashSet<NodeId>>,
}

impl<NodeId, NodeData, EdgeData> Dag<NodeId, NodeData, EdgeData>
where
    NodeId: Copy + Hash + Eq,
{
    pub fn new() -> Self {
        Dag {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            back_edges: HashMap::new(),
        }
    }

    /// Check a node is in a cycle, this will destory DAG
    fn in_cycle(&self,node_id: NodeId) -> bool {
        // DFS
        let mut visited = HashSet::new();
        let mut stack = vec![node_id];

        while let Some(top) = stack.pop() {
            if visited.contains(&top) {
                return true;
            }
            visited.insert(top);
            for child_id in self.children(top).map(|(id,_)|id) {
                stack.push(child_id)
            }
        }

        false
    }

    /// Check `node_id` is contained in `Dag`
    pub fn contains_node(&self, node_id: NodeId) -> bool {
        self.nodes.contains_key(&node_id)
    }

    /// Insert a node with data
    /// # Returns
    /// * Return `Some(data)` when `node_id` is already in `Dag`
    pub fn insert_node(&mut self, node_id: NodeId, node_data: NodeData) -> Option<NodeData> {
        self.nodes.insert(node_id, node_data)
    }

    /// Check the edge is in `Dag`
    pub fn contains_edge(&self, from: NodeId, to: NodeId) -> bool {
        if let Some(children) = self.edges.get(&from) {
            return children.contains_key(&to);
        }
        false
    }

    /// Insert an edge with data in `Dag`
    /// # Return
    /// * Return `Ok(Some(data))` when there is an same edge in `Dag`
    /// # Errors
    /// * `Err(NodeNotFound(id))` when `from` or `to` CANNOT be found in `Dag`
    /// * `Err(HasCycle)` when detected a cycle
    pub fn insert_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        edge_data: EdgeData,
    ) -> Result<Option<EdgeData>, DagError<NodeId,EdgeData>> {
        if !self.nodes.contains_key(&from) {
            return Err(DagError::NodeNotFound(from));
        }
        if !self.nodes.contains_key(&to) {
            return Err(DagError::NodeNotFound(to));
        }
        let result = if let Some(children) = self.edges.get_mut(&from) {
            children.insert(to, edge_data)
        } else {
            self.edges.insert(from, {
                let mut children = HashMap::new();
                children.insert(to, edge_data);
                children
            });
            None
        };
        if self.in_cycle(from) {
            // remove that edge
            let children = self.edges
                .get_mut(&from)
                .unwrap_or_else(||unreachable!());
            let data = children.remove(&to).unwrap_or_else(||unreachable!());
            return Err(DagError::HasCycle(from,to,data));
        }
        // added back edge
        if let Some(parents) = self.back_edges.get_mut(&to) {
            parents.insert(from);
        } else {
            self.back_edges.insert(to, {
                let mut parents = HashSet::new();
                parents.insert(from);
                parents
            });
        }
        Ok(result)
    }

    /// Get a iterator of all children node by give `node_id`
    pub fn children(&self, node_id: NodeId) -> ChildrenIter<'_, NodeId, EdgeData> {
        ChildrenIter {
            iter: self.edges.get(&node_id).map(|map| map.iter()),
        }
    }

    /// Get a iterator of all children node by give `node_id`
    pub fn parents(&self, node_id: NodeId) -> ParentsIter<'_, NodeId> {
        ParentsIter {
            iter: self.back_edges.get(&node_id).map(|set| set.iter()),
        }
    }

    /// Get the count of nodes
    pub fn nodes_len(&self) -> usize {
        self.nodes.len()
    }

    /// Get all nodes in `Dag`
    pub fn nodes(&self) -> impl Iterator<Item = (NodeId,&'_ NodeData)> {
        self.nodes.iter().map(|(id,data)|(*id,data))
    }

    /// Get all leaf nodes in `Dag`
    pub fn leaves(&self) -> impl Iterator<Item = (NodeId,&'_ NodeData)> {
        self.nodes().filter(|(id,_)|self.children(*id).len() == 0)
    }

    /// Get all root nodes in `Dag`
    pub fn roots(&self) -> impl Iterator<Item = (NodeId,&'_ NodeData)> {
        self.nodes().filter(|(id,_)|self.parents(*id).len() == 0)
    }
}
