use std::collections::HashMap;

use xsparseset::SparseSetHashMap;

use crate::{ComponentTypeId, EntityId, dyn_type_vec::DynTypeVec, Component};

pub struct Archetype {
    types: Vec<ComponentTypeId>,
    // 只需要get_index就行
    sparse: HashMap<EntityId,usize>,
    entities: Vec<EntityId>,
    storages: Vec<Box<dyn DynTypeVec>>
}

impl Archetype {
    pub(crate) fn new() -> Self {
        Archetype {
            types: Vec::new(),
            sparse: HashMap::new(),
            entities: Vec::new(),
            storages: Vec::new(),
        }
    }

    pub(crate) fn create_storage<T: Component>(&mut self, component_id: ComponentTypeId) {
        self.types.push(component_id);
        self.storages.push(Box::new(Vec::<T>::new()))
    }

    pub fn types(&self) -> &[ComponentTypeId] {
        &self.types
    }

    pub fn storages(&self) -> &[Box<dyn DynTypeVec>] {
        &self.storages
    }

    pub fn insert_any_unchecked(&mut self,entity_id: EntityId,data_ptrs: &[*mut u8]) {
        let index = if let Some(index) = self.entities.get_index(entity_id) {
            index
        } else {
            self.entities.insert(, dat)
        };
    }
}