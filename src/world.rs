mod iter;
#[cfg(test)]
mod tests;

use crate::{
    archetype::{ArchetypeRead, ArchetypeWrite}, entity::EntityManager, Archetype, Component, ComponentTypeId, Entity,
    EntityId,
};
use parking_lot::RwLock;

/// XECS的核心
pub struct World {
    next_other_storage_id: u64,
    entities: RwLock<EntityManager>,
    archetypes_lock: RwLock<()>,
    // 同时储存ComponentTypeId信息，可以在不加锁的情况下获得Archetype的类型信息
    archetypes: Vec<(Vec<ComponentTypeId>, RwLock<Archetype>)>,
}

impl World {
    /// 创建一个空的World
    pub fn new() -> Self {
        World {
            next_other_storage_id: 0,
            entities: RwLock::new(EntityManager::new()),
            archetypes_lock: RwLock::new(()),
            archetypes: Vec::new(),
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
            .iter()
            .find(|(archetype_components,_)| archetype_components == component_ids)
            .is_some()
    }

    /// 获得`component_ids`指定的Archetype
    /// # Details
    /// * `component_ids`顺序不必一致
    pub fn archetype(&self, component_ids: &[ComponentTypeId]) -> Option<ArchetypeRead<'_>> {
        let lock = self.archetypes_lock.read();
        let archetype = self.archetypes
            .iter()
            .find(|(archetype_components,_)| archetype_components == component_ids)
            .map(|(_,archetype)|archetype.read())?;
        Some(ArchetypeRead { _lock: lock, archetype })
    }

    /// 获得`component_ids`指定的Archetype
    /// # Details
    /// * `component_ids`顺序不必一致
    pub fn archetype_mut(&self, component_ids: &[ComponentTypeId]) -> Option<ArchetypeWrite<'_>> {
        let lock = self.archetypes_lock.read();
        let archetype = self.archetypes
            .iter()
            .find(|(archetype_components,_)| archetype_components == component_ids)
            .map(|(_,archetype)|archetype.write())?;
        Some(ArchetypeWrite{ _lock: lock, archetype })
    }


    fn push_archetype(&self, archetype: Archetype) {
        let component_ids = archetype.types().to_owned();
        let _lock = self.archetypes_lock.write();
        let ptr = &self.archetypes as *const _ as *mut Vec<(Vec<ComponentTypeId>,RwLock<Archetype>)>;
        // # Safety
        // _lock确保了此时拥有所有权，获得&mut借用是安全的
        let archetypes = unsafe { &mut *ptr };
        archetypes.push((component_ids,RwLock::new(archetype)));
    }

    /// 创建一个entity并返回该entity的handle以方便操作
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
