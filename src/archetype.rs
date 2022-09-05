use std::{any::TypeId, collections::HashMap};

use crate::{dyn_type_vec::DynTypeVec, Component, ComponentTypeId, EntityId};

pub struct Archetype {
    types: Vec<ComponentTypeId>,
    raw_types: Vec<TypeId>,
    // 只需要get_index就行
    sparse: HashMap<EntityId, usize>,
    entities: Vec<EntityId>,
    storages: Vec<Box<dyn DynTypeVec>>,
}

impl Archetype {
    pub(crate) fn new() -> Self {
        Archetype {
            types: Vec::new(),
            raw_types: Vec::new(),
            sparse: HashMap::new(),
            entities: Vec::new(),
            storages: Vec::new(),
        }
    }

    pub(crate) fn create_storage<T: Component>(&mut self, component_id: ComponentTypeId) {
        self.types.push(component_id);
        self.raw_types.push(TypeId::of::<T>());
        self.storages.push(Box::new(Vec::<T>::new()));
    }

    /// 获得Archetype中元素个数
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// 获得Archetype中储存的元素`ComponentTypeId`
    pub fn types(&self) -> &[ComponentTypeId] {
        &self.types
    }

    /// 获得Archetype中每个容器的实际储存数据的类型
    pub fn raw_types(&self) -> &[TypeId] {
        &self.raw_types
    }

    /// 获得Archetype中所有entity的id
    pub fn ids(&self) -> &[EntityId] {
        &self.entities
    }

    /// 检查Archetype有没有指定的entity
    pub fn contains(&self, entity_id: EntityId) -> bool {
        self.entities.contains(&entity_id)
    }

    /// 插入buffer中的数据到Archetype中
    /// # Remarks
    /// 如果已存在id，则之前的数据将会被`drop`
    /// # Safety
    /// * `data_ptrs`中每一个指针必须有效
    /// * `data_ptrs`中的每一个指针指向的数据类型必须与`self.raw_types()`中元素类型对应相同（顺序也相同）
    /// * 在调用该方法之后,`data_ptrs`中的每一个指针指向的内容都不应该被使用（`drop`也是一种使用，请调用`forget`以防止`drop`）
    pub unsafe fn insert_any_and_drop_unchecked(
        &mut self,
        entity_id: EntityId,
        data_ptrs: &[*mut u8],
    ) {
        if let Some(index) = self.sparse.get(&entity_id).copied() {
            for i in 0..data_ptrs.len() {
                let ptr = *data_ptrs.get_unchecked(i);
                let storage = self.storages.get_unchecked_mut(i);
                storage.replace_any_and_drop_unchecked(index, ptr);
            }
        } else {
            self.sparse.insert(entity_id, self.len());
            self.entities.push(entity_id);
            for i in 0..data_ptrs.len() {
                let ptr = *data_ptrs.get_unchecked(i);
                let storage = self.storages.get_unchecked_mut(i);
                storage.push_any_unchecked(ptr);
            }
        };
    }

    /// 插入buffer中的数据到Archetype中
    /// # Remarks
    /// 如果已存在id，则之前的数据将会被`forget`
    /// # Safety
    /// * `data_ptrs`中每一个指针必须有效
    /// * `data_ptrs`中的每一个指针指向的数据类型必须与`self.raw_types()`中元素类型对应相同（顺序也相同）
    /// * 在调用该方法之后,`data_ptrs`中的每一个指针指向的内容都不应该被使用（`drop`也是一种使用，请调用`forget`以防止`drop`）
    pub unsafe fn insert_any_and_forget_unchecked(
        &mut self,
        entity_id: EntityId,
        data_ptrs: &[*mut u8],
    ) {
        if let Some(index) = self.sparse.get(&entity_id).copied() {
            for i in 0..data_ptrs.len() {
                let ptr = *data_ptrs.get_unchecked(i);
                let storage = self.storages.get_unchecked_mut(i);
                storage.replace_any_and_forget_unchecked(index, ptr);
            }
        } else {
            self.sparse.insert(entity_id, self.len());
            self.entities.push(entity_id);
            for i in 0..data_ptrs.len() {
                let ptr = *data_ptrs.get_unchecked(i);
                let storage = self.storages.get_unchecked_mut(i);
                storage.push_any_unchecked(ptr);
            }
        };
    }
    
    /// 从Archetype中移除数据
    /// # Remarks
    /// 被移除的数据将会被`drop`
    /// # Safety
    /// * `entity_id`必须存在于archetype中
    pub unsafe fn remove_and_drop_unchecked(&mut self,entity_id: EntityId) {
        let index = self.sparse.get(&entity_id).copied().unwrap_unchecked();
        if index != self.len() {
            // swap to last
            let last_id = self.entities.last().unwrap_unchecked();
            *self.sparse.get_mut(last_id).unwrap_unchecked() = index;
            let last_index = self.len() - 1;
            self.entities.swap(index,last_index);
            for storage in &mut self.storages {
                storage.swap(index, last_index)
            }
        }
        self.sparse.remove(&entity_id);
        self.entities.pop();
        for storage in &mut self.storages {
            storage.pop_and_drop()
        }
    }

    /// 从Archetype中移除数据
    /// # Remarks
    /// 被移除的数据将会被`drop`
    /// # Safety
    /// * `entity_id`必须存在于archetype中
    pub unsafe fn remove_and_forget_unchecked(&mut self,entity_id: EntityId) {
        let index = self.sparse.get(&entity_id).copied().unwrap_unchecked();
        if index != self.len() {
            // swap to last
            let last_id = self.entities.last().unwrap_unchecked();
            *self.sparse.get_mut(last_id).unwrap_unchecked() = index;
            let last_index = self.len() - 1;
            self.entities.swap(index,last_index);
            for storage in &mut self.storages {
                storage.swap(index, last_index)
            }
        }
        self.sparse.remove(&entity_id);
        self.entities.pop();
        for storage in &mut self.storages {
            storage.pop_and_forget()
        }
    }


    /// 获得Archetype中entity_id对应数据
    /// # Details
    /// 每个Component的指针会被写入到`data_ptrs`中
    /// # Safety
    /// * `entity_id`必须存在于Archetype中
    /// * `data_ptrs.len() == self.types().len()`
    pub unsafe fn get_unchecked(&self,entity_id: EntityId, data_ptrs: &mut [*const u8]) {
        let index = self.sparse.get(&entity_id).copied().unwrap_unchecked();
        for i in 0..self.storages.len() {
            let storage = self.storages.get_unchecked(i);
            let ptr = storage.get_ptr_unchecked(index);
            *data_ptrs.get_unchecked_mut(i) = ptr;
        }
    }

    /// 获得Archetype中entity_id对应数据
    /// # Details
    /// 每个Component的指针会被写入到`data_ptrs`中
    /// # Safety
    /// * `entity_id`必须存在于Archetype中
    /// * `data_ptrs.len() == self.types().len()`
    pub unsafe fn get_mut_unchecked(&mut self,entity_id: EntityId, data_ptrs: &mut [*mut u8]) {
        let index = self.sparse.get(&entity_id).copied().unwrap_unchecked();
        for i in 0..self.storages.len() {
            let storage = self.storages.get_unchecked_mut(i);
            let ptr = storage.get_mut_ptr_unchecked(index);
            *data_ptrs.get_unchecked_mut(i) = ptr;
        }
    }
}

trait FromPtrSlice {
    unsafe fn from_ptr_slice(ptrs: &[*const u8]) -> Self;
}

impl FromPtrSlice for () {
    unsafe fn from_ptr_slice(_: &[*const u8]) -> Self {
        ()
    }
}

impl<A> FromPtrSlice for (A,) {
    unsafe fn from_ptr_slice(ptrs: &[*const u8]) -> Self {
        let a = std::ptr::read::<A>((*ptrs.get_unchecked(0)) as *mut _);
        (a,)
    }
}

impl<A,B> FromPtrSlice for (A,B) {
    unsafe fn from_ptr_slice(ptrs: &[*const u8]) -> Self {
        let a = std::ptr::read::<A>((*ptrs.get_unchecked(0)) as *mut _);
        let b = std::ptr::read::<B>((*ptrs.get_unchecked(1)) as *mut _);
        (a,b)
    }
}

impl Archetype {
}
