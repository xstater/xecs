mod error;
mod iter;
#[cfg(test)]
mod tests;

pub use error::DagError;
pub use iter::{ChildrenIter, ParentsIter,EdgesIter};

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
    fn in_cycle(&self, node_id: NodeId) -> bool {
        // DFS
        let mut visited = HashSet::new();
        let mut stack = vec![node_id];

        while let Some(top) = stack.pop() {
            if visited.contains(&top) {
                return true;
            }
            visited.insert(top);
            for child_id in self.children(top).map(|(id, _)| id) {
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
        if !self.edges.contains_key(&node_id) {
            self.edges.insert(node_id, HashMap::new());
        }
        if !self.back_edges.contains_key(&node_id) {
            self.back_edges.insert(node_id, HashSet::new());
        }
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
    /// * `Err(HasCycle(from,to,data))` when detected a cycle
    pub fn insert_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
        edge_data: EdgeData,
    ) -> Result<Option<EdgeData>, DagError<NodeId, EdgeData>> {
        if !self.nodes.contains_key(&from) {
            return Err(DagError::NodeNotFound(from));
        }
        if !self.nodes.contains_key(&to) {
            return Err(DagError::NodeNotFound(to));
        }
        let children = self.edges.get_mut(&from).unwrap_or_else(|| unreachable!());
        let result = children.insert(to, edge_data);
        if self.in_cycle(from) {
            // roll back
            // remove that edge
            let children = self.edges.get_mut(&from).unwrap_or_else(|| unreachable!());
            let data = children.remove(&to).unwrap_or_else(|| unreachable!());
            return Err(DagError::HasCycle(from, to, data));
        }
        // added back edge
        let parents = self
            .back_edges
            .get_mut(&to)
            .unwrap_or_else(|| unreachable!());
        parents.insert(from);
        Ok(result)
    }

    /// Remove an edge from `Dag`
    /// # Returns
    /// * Return `Ok(Some(data))` when success
    /// * Return `Ok(None)` when no such edge
    /// # Errors
    /// * `Err(NodeNotFound(id))` when `from` or `to` are NOT in `Dag`
    pub fn remove_edge(
        &mut self,
        from: NodeId,
        to: NodeId,
    ) -> Result<Option<EdgeData>, DagError<NodeId, EdgeData>> {
        if !self.nodes.contains_key(&from) {
            return Err(DagError::NodeNotFound(from));
        }
        if !self.nodes.contains_key(&to) {
            return Err(DagError::NodeNotFound(to));
        }
        let children = self.edges.get_mut(&from).unwrap_or_else(|| unreachable!());
        let result = children.remove(&to);
        let parents = self
            .back_edges
            .get_mut(&to)
            .unwrap_or_else(|| unreachable!());
        parents.remove(&from);
        Ok(result)
    }

    /// remove a node and all edges
    /// # Returns
    /// * Return `(Some(data),edges_data)` if successed
    pub fn remove_node(&mut self, node_id: NodeId) -> (Option<NodeData>, Vec<EdgeData>) {
        if !self.contains_node(node_id) {
            return (None, Vec::new());
        }
        let mut edge_datas = Vec::new();
        // remove children edges
        let ids = self.children(node_id).map(|(id, _)| id).collect::<Vec<_>>();
        for child_id in ids {
            let data = self
                .remove_edge(node_id, child_id)
                .unwrap_or_else(|_| unreachable!())
                .unwrap_or_else(|| unreachable!());
            edge_datas.push(data);
        }
        // remove parents edges
        let ids = self.parents(node_id).collect::<Vec<_>>();
        for parent_id in ids {
            let data = self
                .remove_edge(parent_id, node_id)
                .unwrap_or_else(|_| unreachable!())
                .unwrap_or_else(|| unreachable!());
            edge_datas.push(data)
        }
        // remove node
        let node_data = self.nodes.remove(&node_id);
        (node_data, edge_datas)
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
    pub fn nodes(&self) -> impl Iterator<Item = (NodeId, &'_ NodeData)> {
        self.nodes.iter().map(|(id, data)| (*id, data))
    }
    
    pub fn edges(&self) -> EdgesIter<'_,NodeId,EdgeData> {
        EdgesIter {
            from_iter: self.edges.iter(),
            to_iter: None,
        }
    }

    /// Get all leaf nodes in `Dag`
    pub fn leaves(&self) -> impl Iterator<Item = (NodeId, &'_ NodeData)> {
        self.nodes().filter(|(id, _)| self.children(*id).len() == 0)
    }

    /// Get all root nodes in `Dag`
    pub fn roots(&self) -> impl Iterator<Item = (NodeId, &'_ NodeData)> {
        self.nodes().filter(|(id, _)| self.parents(*id).len() == 0)
    }

    /// Get data stored with node
    pub fn get_node(&self, node_id: NodeId) -> Option<&NodeData> {
        self.nodes.get(&node_id)
    }

    /// Get mutable data stored with node
    pub fn get_node_mut(&mut self, node_id: NodeId) -> Option<&mut NodeData> {
        self.nodes.get_mut(&node_id)
    }
    
    /// Get data stored with edge
    pub fn get_edge(&self, from: NodeId, to: NodeId) -> Result<Option<&EdgeData>,DagError<NodeId,EdgeData>> {
        if !self.nodes.contains_key(&from) {
            return Err(DagError::NodeNotFound(from));
        }
        if !self.nodes.contains_key(&to) {
            return Err(DagError::NodeNotFound(to));
        }
        let children = self.edges.get(&from)
            .unwrap_or_else(|| unreachable!());
        Ok(children.get(&to))
    }

    /// Get mutable data stored with edge
    pub fn get_edge_mut(&mut self, from: NodeId, to: NodeId) -> Result<Option<&mut EdgeData>,DagError<NodeId,EdgeData>> {
        if !self.nodes.contains_key(&from) {
            return Err(DagError::NodeNotFound(from));
        }
        if !self.nodes.contains_key(&to) {
            return Err(DagError::NodeNotFound(to));
        }
        let children = self.edges.get_mut(&from)
            .unwrap_or_else(|| unreachable!());
        Ok(children.get_mut(&to))
    }
}
