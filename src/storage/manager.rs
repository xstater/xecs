use std::collections::HashMap;

use parking_lot::RwLock;

use crate::{ComponentStorage, StorageId};

enum Node {
    Group,
    Storage(RwLock<Box<dyn ComponentStorage>>),
}

pub struct Edges<T> {
    edges: HashMap<StorageId, HashMap<StorageId, T>>,
}

impl<T> Edges<T> {
    fn new() -> Self {
        Edges {
            edges: HashMap::new(),
        }
    }

    fn reached(&self, node: StorageId) -> Vec<StorageId> {
        self.edges.get(&node)
            .map(|node| node.keys().copied().collect())
            .unwrap_or_default()
    }

    fn reached_with_data(&self, node: StorageId) -> Vec<(StorageId,&T)> {
        self.edges.get(&node)
            .map(|node|node.iter().map(|(k,v)|(k.clone(),v)).collect())
            .unwrap_or_default()
    }

    fn insert(&mut self, start: StorageId, to: StorageId, data: T) {
        if let Some(node) = self.edges.get_mut(&start) {
            node.insert(to, data);
        } else {
            self.edges.insert(start, {
                let mut map = HashMap::new();
                map.insert(to, data);
                map
            });
        }
    }

    fn remove(&mut self, start: StorageId, to: StorageId) -> Option<T> {
        let node = self.edges.get_mut(&start)?;
        node.remove(&to)
    }

    fn get(&self, start: StorageId, to: StorageId) -> Option<&T> {
        let node = self.edges.get(&start)?;
        node.get(&to)
    }

    fn get_mut(&mut self, start: StorageId, to: StorageId) -> Option<&mut T> {
        let node = self.edges.get_mut(&start)?;
        node.get_mut(&to)
    }
}

pub struct Manager {
    next_group_id: u64,
    forward_edges: Edges<bool>,
    back_edges: Edges<()>,
    nodes: HashMap<StorageId, Node>,
}

impl Manager {
    pub fn new() -> Self {
        Manager {
            next_group_id: 1,
            forward_edges: Edges::new(),
            back_edges: Edges::new(),
            nodes: HashMap::new(),
        }
    }

    fn next_group_id(&mut self) -> StorageId {
        let id = self.next_group_id;
        self.next_group_id += 1;
        StorageId::Group(id)
    }

    fn is_owned(&self,storage_id: StorageId) -> bool{
        let parents = self.back_edges.reached(storage_id);
        for parent in parents {
            let is_owned = self.forward_edges
                .get(parent, storage_id)
                .copied()
                .unwrap_or_else(||unreachable!());
            if is_owned {
                return true;
            }
        }
        false
    }

    pub fn contains(&self, storage_id: StorageId) -> bool {
        self.nodes.contains_key(&storage_id)
    }

    /// # Safety
    /// * Safe when `!self.contains(storage_id)`
    pub unsafe fn insert_storage(&mut self,storage_id: StorageId,storage: RwLock<Box<dyn ComponentStorage>>){
        self.nodes.insert(storage_id,Node::Storage(storage));
    }

    /// # Safety
    /// * Safe when `!self.contains(storage_id)`
    unsafe fn insert_group(&mut self,storage_id: StorageId) {
        self.nodes.insert(storage_id, Node::Group);
    }

    /// # Safety
    /// * `self.contains(storage_id1) && self.contains(storage_id2)`
    /// * `!self.is_owned(storage_id1) && !self.is_owned(storage_id2)`
    pub unsafe fn make_full_owning_group(&mut self,storage_id1: StorageId, storage_id2: StorageId) -> StorageId {
        let group_id = self.next_group_id();
        self.insert_group(group_id);
        self.forward_edges.insert(group_id, storage_id1, true);
        self.back_edges.insert(storage_id1,group_id,());
        self.forward_edges.insert(group_id, storage_id2, true);
        self.back_edges.insert(storage_id2,group_id,());
        self.rearrange_group(group_id);
        group_id
    }

    pub unsafe fn rearrange_group(&self,group_id: StorageId) {
        if let Node::Storage(_) = self.nodes.get(&group_id).unwrap_or_else(|| unreachable!()) {
            return;
        }
        let children = self.forward_edges.reached_with_data(group_id);
        for (storage_id,is_owned) in children.into_iter().map(|(id,f)|(id,*f)) {
            if !is_owned {
                continue;
            }
            self.rearrange_group(storage_id)
        }
        todo!()
    }
}
