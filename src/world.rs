mod iter;
#[cfg(test)]
mod tests;

use std::ptr::null_mut;

use crate::{
    dyn_type_vec::DynTypeVec, entity::EntityManager, Archetype, Component, ComponentTypeId, Entity,
    EntityId,
};
use parking_lot::RwLock;

/// XECS的核心
pub struct World {
    next_other_storage_id: u64,
    entities: RwLock<EntityManager>,
    archetypes: RwLock<Vec<RwLock<Archetype>>>
}

impl World {
    /// 创建一个空的World
    pub fn new() -> Self {
        World {
            next_other_storage_id: 0,
            entities: RwLock::new(EntityManager::new()),
            archetypes: RwLock::new(Vec::new()),
        }
    }

    /// 为外部类型分配一个id
    pub fn allocate_other_component_id(&mut self) -> ComponentTypeId {
        let id = self.next_other_storage_id;
        self.next_other_storage_id += 1;
        ComponentTypeId::Other(id)
    }

    /// 判断是否有一个由`component_ids`确定的archetype
    pub fn has_archetype(&self, component_ids: &[ComponentTypeId]) -> bool {
        self.archetypes
            .read()
            .iter()
            .find(|archetype| archetype.read().types() == component_ids)
            .is_some()
    }

    /// 获得一个由`component_ids`确定的archetype
    pub fn archetype(&self, component_ids: &[ComponentTypeId]) -> Option<&RwLock<Archetype>> {
        self.archetypes
            .iter()
            .find(|archetype| archetype.read().types() == component_ids)
    }

    /// 找到所有能储存`component_ids`中类型的Archetype
    /// # Remarks
    /// * 不考虑顺序
    pub fn find_archetypes(&self, component_ids: &[ComponentTypeId]) -> Vec<&RwLock<Archetype>> {
        self.archetypes
            .iter()
            .filter(|archetype| archetype.read().contains_storages(component_ids))
            .collect::<Vec<_>>()
    }

    /// 查找entity位于哪个archetype中
    pub fn get_archetype_by_id(&self, entity_id: EntityId) -> Option<&RwLock<Archetype>> {
        self.archetypes
            .iter()
            .find(|archetype| archetype.read().contains(entity_id))
    }

    /// 创建一个entity并返回该entity的hanle以方便操作
    pub fn create_entity(&self) -> Entity<'_> {
        todo!()
    }

    /// 插入一个Rust类型component到entity上
    /// # Details
    /// 如果之前已经存在该类型的数据，则会被替换并返回
    /// # Panics
    /// * `entity_id`不存在
    pub fn attach_component<T: Component>(&self, entity_id: EntityId, component: T) -> Option<T> {
        self.attach_component_other(entity_id, ComponentTypeId::from_rust_type::<T>(), component)
    }

    /// 插入一个component到entity上
    /// # Details
    /// 如果之前已经存在该类型的数据，则会被替换并返回
    /// # Panics
    /// * `entity_id`不存在
    pub fn attach_component_other<T: Component>(
        &self,
        entity_id: EntityId,
        component_id: ComponentTypeId,
        component: T,
    ) -> Option<T> {
        let manager = self.entities.read();
        if !manager.has(entity_id) {
            panic!(
                "Cannot attach component to a non-existence entity with ID = {}",
                entity_id
            );
        }



        todo!()
    }
}
