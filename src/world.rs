//! # world struct
use crate::{EntityId, Component, Entities};
use std::collections::HashMap;
use std::any::TypeId;
use std::cell::{RefCell, Ref, RefMut};
use crate::components::ComponentStorage;
use crate::sparse_set::SparseSet;
use crate::entity::{EntityRef, EntityManager};
use crate::group::{Group, NonOwningGroup, OwningType, OwningGroup};
use std::fmt::{Debug, Formatter};
use crate::query::{Queryable};

/// World is core struct of xecs.It manages all entities and components.Using RefCell to ensure the
/// borrow relations.
pub struct World {
    entity_manager : EntityManager,
    // Box<SparseSet<EntityId,Component>>
    components: HashMap<TypeId,RefCell<Box<dyn ComponentStorage>>>,
    groups : Vec<RefCell<Group>>
}

impl World {
    /// Create a empty world.
    pub fn new() -> World {
        World {
            entity_manager : EntityManager::new(),
            components: Default::default(),
            groups : vec![]
        }
    }

    /// Register a component.
    /// # Detail
    /// Do nothing if component has been registered.
    pub fn register<T : Component>(&mut self) -> &mut Self {
        let type_id = TypeId::of::<T>();
        if self.components.contains_key(&type_id) {
            return self;
        }
        self.components.insert(
           type_id,
            RefCell::new(
                Box::new(
                    SparseSet::<EntityId,T>::new()
                ))
        );
        self
    }

    /// Create an empty entity in world, return an EntityRef.
    pub fn create_entity(&mut self) -> EntityRef<'_>{
        let id = self.entity_manager.create();
        EntityRef::new(self,id)
    }

    /// Remove an entity and its components.
    pub fn remove_entity(&mut self,entity_id : EntityId){
        self.entity_manager.remove(entity_id);
        //remove all components of this entity
        for (_,storage) in &mut self.components{
            let mut storage = storage.borrow_mut();
            storage.remove(entity_id);
        }
    }

    pub(in crate) fn components_storage_dyn_ref(&self,type_id : TypeId) -> Ref<'_,Box<dyn ComponentStorage>> {
        debug_assert!(self.components.contains_key(&type_id),
                      "Component has not been registered.");
        self.components
            .get(&type_id)
            .unwrap()
            .borrow()
    }
    pub(in crate) fn components_storage_dyn_mut(&self,type_id : TypeId) -> RefMut<'_,Box<dyn ComponentStorage>> {
        debug_assert!(self.components.contains_key(&type_id),
                      "Component has not been registered.");
        self.components
            .get(&type_id)
            .unwrap()
            .borrow_mut()
    }

    pub(in crate) fn components_storage_ref<T : Component>(&self) -> Ref<'_,SparseSet<EntityId,T>>{
        debug_assert!(self.components.contains_key(&TypeId::of::<T>()),
                      "Component has not been registered.");
        let components = self.components
            .get(&TypeId::of::<T>())
            .unwrap()
            .borrow();
        Ref::map(components,|raw|{
            // safe: because 'raw' is Box<SparseSet<EntityId,T>>
            unsafe {
                raw.downcast_ref::<SparseSet<EntityId,T>>()
            }
        })
    }

    pub(in crate) fn components_storage_mut<T : Component>(&self) -> RefMut<'_,SparseSet<EntityId,T>>{
        debug_assert!(self.components.contains_key(&TypeId::of::<T>()),
                      "Component has not been registered.");
        let components = self.components
            .get(&TypeId::of::<T>())
            .unwrap()
            .borrow_mut();
        RefMut::map(components,|raw|{
            // safe: because 'raw' is Box<SparseSet<EntityId,T>>
            unsafe {
                raw.downcast_mut::<SparseSet<EntityId,T>>()
            }
        })
    }

    /// Attach a component to an entity.
    pub fn attach_component<T : Component>(&mut self,entity_id : EntityId,component : T){
        self.components_storage_mut::<T>()
            .add(entity_id,component);
        for group in &self.groups{
            let mut group = group.borrow_mut();
            if group.contains(TypeId::of::<T>()) {
                group.add(self,entity_id);
            }
        }
    }

    /// Detach a component from an entity.
    pub fn detach_component<T : Component>(&mut self,entity_id : EntityId) -> Option<T>{
        for group in &self.groups {
            let mut group = group.borrow_mut();
            if group.contains(TypeId::of::<T>()) {
                group.remove(self,entity_id);
            }
        }
        self.components_storage_mut::<T>()
            .remove(entity_id)
    }

    /// Get a reference of all components.
    pub fn components_ref<T : Component>(&self) -> Ref<'_,[T]>{
        let slice_ref = self.components_storage_ref::<T>();
        Ref::map(slice_ref,|raw|{
            raw.data()
        })
    }

    /// Get a mutable reference of all components.
    pub fn components_mut<T : Component>(&self) -> RefMut<'_,[T]>{
        let slice_mut = self.components_storage_mut::<T>();
        RefMut::map(slice_mut,|raw|{
            raw.data_mut()
        })
    }

    /// Get all entities' id.
    pub fn entities(&self) -> &Entities {
        self.entity_manager.entities()
    }

    /// Get all entities' id of a component
    pub fn entities_in<T : Component>(&self) -> Ref<'_,Entities> {
        let storage = self.components_storage_ref::<T>();
        Ref::map(storage,|raw|{
            raw.entities()
        })
    }

    /// Check if an id exists in world.
    pub fn exist(&mut self,entity_id : EntityId) -> bool {
        self.entity_manager.has(entity_id)
    }

    /// Get an EntityRef from an EntityId, return None if id doesn't exist in world.
    pub fn entity(&mut self,entity_id : EntityId) -> Option<EntityRef<'_>> {
        if self.exist(entity_id) {
            Some(EntityRef::new(self, entity_id))
        }else{
            None
        }
    }

    /// Get a reference of component of entity, return None if id doesn't exist in world.
    pub fn entity_component_ref<T : Component>(&self,entity_id : EntityId) -> Option<Ref<'_,T>> {
        let storage = self.components_storage_ref();
        if storage.exist(entity_id) {
            Some(Ref::map(storage, |sparse_set| {
                sparse_set.get(entity_id).unwrap()
            }))
        } else {
            None
        }
    }

    /// Get a mutable reference of component of entity, return None if id doesn't exist in world.
    pub fn entity_component_mut<T : Component>(&self,entity_id : EntityId) -> Option<RefMut<'_,T>> {
        let storage = self.components_storage_mut();
        if storage.exist(entity_id) {
            Some(RefMut::map(storage, |sparse_set| {
                sparse_set.get_mut(entity_id).unwrap()
            }))
        } else {
            None
        }
    }

    /// Make a group to accelerate the iteration.
    /// ## Details
    /// See [group](crate::group)
    /// ### Examples
    /// Create a full-owning group
    /// ```no_run
    /// world.make_group::<Position,Particle>(true,true);
    /// ```
    /// Create a partial-owning group
    /// ```no_run
    /// world.make_group::<Position,Particle>(true,false);
    /// ```
    /// Create a non-owning group
    /// ```no_run
    /// world.make_group::<Position,Particle>(false,false);
    /// ```
    pub fn make_group<A : Component,B : Component>(&mut self,owning_a : bool,owning_b : bool){
        debug_assert!(
            {
                let mut ok = true;
                for group in &self.groups {
                    let group = group.borrow();
                    if (owning_a && group.is_owned(TypeId::of::<A>()))
                    || (owning_b && group.is_owned(TypeId::of::<B>())) {
                        ok = false;
                        break;
                    }
                }
                ok
            }
        ,"Component in group cannot be owned twice");
        let mut group = if owning_a || owning_b {
            let a = if owning_a {
                OwningType::Owning(TypeId::of::<A>())
            } else {
                OwningType::NonOwning(TypeId::of::<A>())
            };
            let b = if owning_b {
                OwningType::Owning(TypeId::of::<B>())
            } else {
                OwningType::NonOwning(TypeId::of::<B>())
            };
            Group::Owning(OwningGroup{
                types: (a,b),
                length: 0
            })
        } else {
            Group::NonOwning(NonOwningGroup{
                types: (TypeId::of::<A>(), TypeId::of::<B>()),
                sparse_set: SparseSet::new()
            })
        };
        group.make_group_in_world::<A,B>(&self);
        self.groups.push(RefCell::new(group));
    }

    pub(in crate) fn group<A : Component,B : Component>(&self) -> Option<Ref<'_,Group>>{
        self.groups.iter()
            .map(|group|group.borrow())
            .find(|group|{
                group.contains(TypeId::of::<A>()) && group.contains(TypeId::of::<B>())
            })
    }

    /// Query entities
    /// ## Details
    /// See [query](crate::query)
    pub fn query<'a,T : Queryable<'a>>(&'a self) -> Box<dyn Iterator<Item=<T as Queryable>::Item> + 'a> {
        <T as Queryable<'a>>::query(self)
    }

}

impl Debug for World {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("entities",&self.entity_manager)
            .field("groups",&self.groups)
            .field("components",
                   &self.components
                       .keys()
                       .cloned()
                       .collect::<Vec<TypeId>>())
            .finish()
    }
}

#[cfg(test)]
mod tests{
    use crate::{World};
    use crate::group::Group;

    #[test]
    fn component_test(){
        let mut world = World::new();
        world.register::<char>();
        let id1 = world.create_entity().into_id();
        let id2 = world.create_entity().into_id();
        let _id3 = world.create_entity().into_id();

        world.attach_component(id1,'c');
        world.attach_component(id2,'a');

        assert_eq!(world.components_ref::<char>().as_ref(),&['c','a']);

        world.remove_entity(id1);

        assert_eq!(world.components_ref::<char>().as_ref(),&['a']);
    }


    #[test]
    fn group_test(){
        let mut world = World::new();

        world.register::<u32>();
        world.register::<char>();
        world.register::<()>();

        world.create_entity().attach(1u32).attach(());
        let id2 = world.create_entity().attach(2u32).into_id();
        let id3 = world.create_entity()
            .attach(3u32)
            .attach('a')
            .attach(())
            .into_id();
        world.create_entity().attach(4u32).attach('b');
        world.create_entity().attach(5u32).attach('c');
        world.create_entity().attach(6u32);
        let id7 = world.create_entity().attach('d').attach(()).into_id();
        println!("#initial");
        println!("u32 :{:?}",world.entities_in::<u32>());
        println!("char:{:?}",world.entities_in::<char>());
        println!("()  :{:?}",world.entities_in::<()>());
        println!();

        world.make_group::<u32,char>(true,true);
        world.make_group::<u32,char>(false,false);
        world.make_group::<u32,()>(false,true);
        println!("#Made group <u32,char> <u32,()>");
        println!("u32 :{:?}",world.entities_in::<u32>());
        println!("char:{:?}",world.entities_in::<char>());
        println!("()  :{:?}",world.entities_in::<()>());
        println!();

        world.entity(id2)
            .and_then(|entity|Some(entity.attach('b')));
        println!("#attach component b for id=2");
        println!("u32 :{:?}",world.entities_in::<u32>());
        println!("char:{:?}",world.entities_in::<char>());
        println!("()  :{:?}",world.entities_in::<()>());
        println!();

        world.entity(id7)
            .unwrap()
            .attach(2u32);
        println!("#attach component 2 for id=7");
        println!("u32 :{:?}",world.entities_in::<u32>());
        println!("char:{:?}",world.entities_in::<char>());
        println!("()  :{:?}",world.entities_in::<()>());
        println!();

        world.entity(id3)
            .unwrap()
            .detach::<u32>();
        println!("#detach component u32 for id=3");
        println!("u32 :{:?}",world.entities_in::<u32>());
        println!("char:{:?}",world.entities_in::<char>());
        println!("()  :{:?}",world.entities_in::<()>());
        println!();

        for group in &world.groups {
            println!("{:?}",group);
        }

    }

    #[test]
    fn size_test(){
        println!("Size of bool:{}Bytes",std::mem::size_of::<bool>());
        println!("Size of u64:{}Bytes",std::mem::size_of::<u64>());
        println!("Size of Group:{}Bytes",std::mem::size_of::<Group>());
    }

    #[test]
    fn debug_trait_test(){
        let mut world = World::new();

        world.register::<char>();
        world.register::<u32>();

        world.make_group::<char,u32>(true,true);

        world.create_entity()
            .attach('c')
            .attach(12_u32);
        world.create_entity()
            .attach('a');

        println!("{:?}",world);
    }
}