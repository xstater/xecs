use std::{collections::HashMap, any::TypeId};

use crate::{ComponentTypeId, EntityId, dyn_type_vec::DynTypeVec, Component};

struct Storage {
    component_type_id: ComponentTypeId,
    data: Box<dyn DynTypeVec>
}

pub struct Archetype {
    sparse: HashMap<EntityId,usize>,
    entities: Vec<EntityId>,
    storages: Vec<Storage>,
}

// Safe functions
impl Archetype {
    /// Create an empty archetype
    pub(crate) fn new() -> Archetype {
        Archetype {
            sparse: HashMap::new(),
            entities: Vec::new(),
            storages: Vec::new(),
        }
    }

    /// Create a new storage in Archetype
    /// # Details
    /// * The storages in archetype will be sorted by `component_type_id`
    /// * `storage` must be empty
    pub(crate) fn create_storage(&mut self, component_type_id: ComponentTypeId, storage: Box<dyn DynTypeVec>) {
        let storage = Storage {
            component_type_id,
            data: storage,
        };
        // get the index that storage will be inserted
        let index = match self.storages.binary_search_by_key(&component_type_id,|storage|storage.component_type_id) {
            Ok(index) => index,
            Err(index) => index,
        };
        self.storages.insert(index, storage)
    }

    pub(crate) fn create_rust_storage<T: Component>(&mut self) {
        let component_type_id = ComponentTypeId::from_rust_type::<T>();
        let storage = Box::new(Vec::<T>::new());
        self.create_storage(component_type_id, storage)
    }

    /// Check archetype is empty
    pub fn is_empty(&self) -> bool {
        self.sparse.is_empty()
    }

    /// Get the count of entities in Archetype
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Get the index of entity in Archetype by its id
    pub fn get_index(&self, entity_id: EntityId) -> Option<usize> {
        self.sparse.get(&entity_id).copied()
    }

    /// Check an entity is in archetype
    pub fn contains(&self, entity_id: EntityId) -> bool {
        self.sparse.contains_key(&entity_id)
    }

    /// Get all ids in Archetype
    pub fn ids(&self) -> &'_ [EntityId] {
        &self.entities
    }

    /// Get all component_type_id of storages in archetype
    pub fn component_type_ids(&self) -> impl Iterator<Item = ComponentTypeId> + '_ {
        self.storages.iter().map(|storage|storage.component_type_id)
    }

    /// Get the real type of data stored in storage in Archetype
    pub fn types(&self) -> impl Iterator<Item = TypeId> + '_ {
        self.storages.iter().map(|storage|storage.data.type_id())
    }

    /// Get the storage of archetype by component_type_id
    pub fn storage_ref(&self, component_type_id: ComponentTypeId) -> Option<&'_ dyn DynTypeVec> {
        match self.storages.binary_search_by_key(&component_type_id, |storage| storage.component_type_id) {
            Ok(index) => unsafe {
                Some(&*self.storages.get_unchecked(index).data)
            },
            Err(_) => None,
        }
    }

    /// Get the storage of archetype by component_type_id
    pub fn storage_mut(&mut self, component_type_id: ComponentTypeId) -> Option<&'_ mut dyn DynTypeVec> {
        match self.storages.binary_search_by_key(&component_type_id, |storage| storage.component_type_id) {
            Ok(index) => unsafe {
                Some(&mut *self.storages.get_unchecked_mut(index).data)
            },
            Err(_) => None,
        }
    }

    pub fn insert<T: crate::tuple::Tuple>(&mut self, entity_id: EntityId, data: T) -> Option<T>{
        let mut ptrs = vec![std::ptr::null(); data.len()];
        data.get_ptrs(&mut ptrs);

        if self.contains(entity_id) {
            let mut out_ptrs = vec![std::ptr::null(); self.storages.len()];
            unsafe {
                self.get_mut_ptr_unchecked(entity_id, &mut out_ptrs);

                let out = T::from_ptrs(out_ptrs);
                self.insert_any_and_forget_unchecked(entity_id, ptrs);
                std::mem::forget(data);
                Some(out)
            }
        } else {
            let mut ptrs = vec![std::ptr::null(); self.storages.len()];
            unsafe {
                self.get_mut_ptr_unchecked(entity_id, &mut ptrs);

                let out = T::from_ptrs(ptrs);
                self.insert_any_and_drop_unchecked(entity_id, ptrs);
                std::mem::forget(data);
                Some(out)
            }
        }
    }
}

// Unsafe functions
impl Archetype {

    /// Insert data in `data_ptrs` to Archetype
    /// # Remarks
    /// If `entity_id` exists, the data will be `forget`
    /// # Safety
    /// * All pointer in `data_ptrs` must be valid
    /// * `data_ptrs` must has same length and order as `types()`
    /// * All data in `data_ptrs` cannot be used after this call (include `drop`, please `forget` them)
    pub unsafe fn insert_any_and_drop_unchecked(
        &mut self,
        entity_id: EntityId,
        data_ptrs: &[*mut u8],
    ) {
        if let Some(index) = self.sparse.get(&entity_id).copied() {
            for i in 0..data_ptrs.len() {
                let ptr = *data_ptrs.get_unchecked(i);
                let storage = self.storages.get_unchecked_mut(i);
                storage.data.replace_any_and_drop_unchecked(index, ptr);
            }
        } else {
            self.sparse.insert(entity_id, self.len());
            self.entities.push(entity_id);
            for i in 0..data_ptrs.len() {
                let ptr = *data_ptrs.get_unchecked(i);
                let storage = self.storages.get_unchecked_mut(i);
                storage.data.push_any_unchecked(ptr);
            }
        };
    }

    /// Insert data in `data_ptrs` to Archetype
    /// # Remarks
    /// If `entity_id` exists, the data will be `forget`
    /// # Safety
    /// * All pointer in `data_ptrs` must be valid
    /// * `data_ptrs` must has same length and order as `types()`
    /// * All data in `data_ptrs` cannot be used after this call (include `drop`, please `forget` them)
    pub unsafe fn insert_any_and_forget_unchecked(
        &mut self,
        entity_id: EntityId,
        data_ptrs: &[*mut u8],
    ) {
        if let Some(index) = self.sparse.get(&entity_id).copied() {
            for i in 0..data_ptrs.len() {
                let ptr = *data_ptrs.get_unchecked(i);
                let storage = self.storages.get_unchecked_mut(i);
                storage.data.replace_any_and_forget_unchecked(index, ptr);
            }
        } else {
            self.sparse.insert(entity_id, self.len());
            self.entities.push(entity_id);
            for i in 0..data_ptrs.len() {
                let ptr = *data_ptrs.get_unchecked(i);
                let storage = self.storages.get_unchecked_mut(i);
                storage.data.push_any_unchecked(ptr);
            }
        };
    }

    /// Insert a lot of data to Archetype
    /// # Details
    /// `ids` and pointers in `data` will be moved to archetype
    /// # Safety
    /// * All id in `ids` must be insert to archetype firstly
    /// * `ids.len()` must equal to all `DynTypeVec` in data
    /// * `data` must has same types order and length as `Archetype`
    /// * all pointers in `data` must be valid
    pub unsafe fn insert_any_batch_unchecked(
        &mut self,
        ids: &mut Vec<EntityId>,
        data: &[*mut dyn DynTypeVec],
    ) {
        let mut index = self.len();
        for id in ids.iter().copied() {
            self.sparse.insert(id, index);
            index += 1;
        }

        self.entities.append(ids);

        for i in 0..data.len() {
            let data = *data.get_unchecked(i);
            let storage = self.storages.get_unchecked_mut(i);

            storage.data.append_any_unchecked(&mut *data);
        }
    }

    /// Remove data from archetype
    /// # Remarks
    /// * The removed data will be `drop`
    /// # Safety
    /// * `entity_id` must exist in archetype
    pub unsafe fn remove_and_drop_unchecked(&mut self, entity_id: EntityId) {
        let index = self.sparse.get(&entity_id).copied().unwrap_unchecked();
        if index != self.len() {
            // swap to last
            let last_id = self.entities.last().unwrap_unchecked();
            *self.sparse.get_mut(last_id).unwrap_unchecked() = index;
            let last_index = self.len() - 1;
            self.entities.swap(index, last_index);
            for storage in &mut self.storages {
                storage.data.swap(index, last_index)
            }
        }
        self.sparse.remove(&entity_id);
        self.entities.pop();
        for storage in &mut self.storages {
            storage.data.pop_and_drop()
        }
    }

    /// Remove data from archetype
    /// # Remarks
    /// * The removed data will be `forget`
    /// # Safety
    /// * `entity_id` must exist in archetype
    pub unsafe fn remove_and_forget_unchecked(&mut self, entity_id: EntityId) {
        let index = self.sparse.get(&entity_id).copied().unwrap_unchecked();
        if index != self.len() {
            // swap to last
            let last_id = self.entities.last().unwrap_unchecked();
            *self.sparse.get_mut(last_id).unwrap_unchecked() = index;
            let last_index = self.len() - 1;
            self.entities.swap(index, last_index);
            for storage in &mut self.storages {
                storage.data.swap(index, last_index)
            }
        }
        self.sparse.remove(&entity_id);
        self.entities.pop();
        for storage in &mut self.storages {
            storage.data.pop_and_forget()
        }
    }

    /// Remove a range of data in Archetype
    /// # Details
    /// * All data will be packed in Vec and returned
    /// # Safety
    /// * `range` must be valid
    pub unsafe fn remove_batch_unchecked(&mut self, range: Range<usize>) -> (Vec<EntityId>,Vec<Box<dyn DynTypeVec>>){
        let removed_ids = {
            let mut removed_ids = self.entities.split_off(range.start);
            let mut remain_ids = removed_ids.split_off(range.end - range.start);
            self.entities.append(&mut remain_ids);
            removed_ids
        };

        for id in removed_ids.iter() {
            self.sparse.remove(id);
        }

        let removed_data = self.storages.iter_mut()
            .map(|storage| storage.data.remove_range(range.clone()))
            .collect::<Vec<_>>();
        (removed_ids,removed_data)
    }

    /// Get data of entity in archetype
    /// # Details
    /// * all pointer of data will be write to `data_ptrs`
    /// # Safety
    /// * `entity_id` must exists in archetype
    /// * `data_ptrs.len() >= self.types().len()`
    pub unsafe fn get_ptr_unchecked(&self, entity_id: EntityId, data_ptrs: &mut [*const u8]) {
        let index = self.sparse.get(&entity_id).copied().unwrap_unchecked();
        for i in 0..self.storages.len() {
            let storage = self.storages.get_unchecked(i);
            let ptr = storage.data.get_ptr_unchecked(index);
            *data_ptrs.get_unchecked_mut(i) = ptr;
        }
    }

    /// Get the data of entity in archetype
    /// # Details
    /// * all pointer of data will be write to `data_ptrs`
    /// # Safety
    /// * `entity_id` must exists in archetype
    /// * `data_ptrs.len() >= self.types().len()`
    pub unsafe fn get_mut_ptr_unchecked(&mut self, entity_id: EntityId, data_ptrs: &mut [*mut u8]) {
        let index = self.sparse.get(&entity_id).copied().unwrap_unchecked();
        for i in 0..self.storages.len() {
            let storage = self.storages.get_unchecked_mut(i);
            let ptr = storage.data.get_mut_ptr_unchecked(index);
            *data_ptrs.get_unchecked_mut(i) = ptr;
        }
    }
}
