//! # world struct
use crate::component::{Component, ComponentStorage};
use crate::entity::{EntityId, EntityManager, EntityRef};
use crate::group::Group;
use crate::query::{QueryIterator, Queryable};
use crate::sparse_set::SparseSet;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// World is core struct of xecs.It manages all entities and components.Using RefCell to ensure the
/// borrow relations.
pub struct World {
    entity_manager: EntityManager,
    // Box<SparseSet<EntityId,Component>>
    components: HashMap<TypeId,RwLock<Box<dyn ComponentStorage>>>,
    groups: Vec<RwLock<Box<dyn Group>>>,
}

impl World {
    /// Create a empty world.
    pub fn new() -> World {
        World {
            entity_manager: EntityManager::new(),
            components: Default::default(),
            groups: Default::default(),
        }
    }

    /// Register a component.
    /// # Panics
    /// Panic if component has been registered.
    pub fn register<T: Component>(&mut self) -> &mut Self {
        assert!(!self.has_registered::<T>(),
                "World:Cannot register a component twice");
        let type_id = TypeId::of::<T>();
        self.components.insert(
            type_id,
            RwLock::new(Box::new(SparseSet::<EntityId, T>::new())),
        );
        self
    }

    /// Check if a component has been registered
    pub fn has_registered<T: Component>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.components.contains_key(&type_id)
    }

    /// Create an empty entity in world, return an EntityRef.
    pub fn create_entity(&mut self) -> EntityRef<'_> {
        let id = self.entity_manager.create();
        EntityRef::new(self, id)
    }

    /// Remove an entity and its components.
    pub fn remove_entity(&mut self, entity_id: EntityId) {
        assert!(self.exist(entity_id),
                "World:Cannot remove a non-exists entity");
        self.entity_manager.remove(entity_id);
        // remove entity in group
        for group in &self.groups {
            let mut group = group.write().unwrap();
            group.remove(self, entity_id);
        }
        // remove all components of this entity
        for (_, storage) in &mut self.components {
            let mut storage = storage.write().unwrap();
            if storage.has(entity_id) {
                storage.remove(entity_id);
            }
        }
    }

    /// Get lock guard of raw component storage
    /// return None if component was not registered
    pub(in crate) fn storage_ref(&self,id : TypeId) 
        -> Option<RwLockReadGuard<'_,Box<dyn ComponentStorage>>> {
        self.components
            .get(&id)
            .map(|rwlock|rwlock.read().unwrap())
    }

    /// Get lock guard of raw component storage
    /// return None if component was not registered
    pub(in crate) fn storage_mut(&self,id : TypeId) 
        -> Option<RwLockWriteGuard<'_,Box<dyn ComponentStorage>>> {
        self.components
            .get(&id)
            .map(|rwlock|rwlock.write().unwrap())
    }

    /// Attach a component to an entity.  
    /// # Panics
    /// * Panic if T was not registered
    /// * Panic if entity_id was not existence
    pub fn attach_component<T: Component>(&mut self, entity_id: EntityId,component: T) {
        assert!(self.has_registered::<T>(),
                "World:Cannot attach component because components has not been registered.");
        assert!(self.exist(entity_id),
                "World:Cannot attach component to a non-exist entity");
        let type_id = TypeId::of::<T>();
        {
            // Unwrap never fails because assert ensures this
            let mut storage = self.storage_mut(type_id).unwrap();
            // SAFTY:
            // storage is SparseSet<EntityId,T>
            let sparse_set = unsafe {
                storage.downcast_mut::<SparseSet<EntityId,T>>()
            };
            sparse_set.add(entity_id,component);
        }
        for group in &self.groups {
            let mut group = group.write().unwrap();
            if group.type_id_a() == type_id || group.type_id_b() == type_id {
                group.add(self, entity_id);
            }
        }
    }

    /// Detach a component from an entity.
    /// # Details
    /// Return None if entity didn't have this component,  
    /// otherwise return Some(component)
    /// # Panics
    /// * Panic if T was not registered
    /// * Panic if entity_id was not existence
    pub fn detach_component<T: Component>(&mut self, entity_id: EntityId) -> Option<T> {
        assert!(self.has_registered::<T>(),
                "World:Cannot detach component because components has not been registered.");
        assert!(self.exist(entity_id),
                "World:Cannot detach component from a non-exist entity");
        let type_id = TypeId::of::<T>();
        for group in &self.groups {
            let mut group = group.write().unwrap();
            if group.type_id_a() == type_id || group.type_id_b() == type_id {
                group.remove(self, entity_id)
            }
        }
        // Unwrap never fails because assert ensures this
        let mut storage = self.storage_mut(type_id).unwrap();
        // SAFTY:
        // storage is SparseSet<EntityId,T>
        let sparse_set = unsafe {
            storage.downcast_mut::<SparseSet<EntityId,T>>()
        };
        sparse_set.remove(entity_id)
    }

    /// Check if an id exists in world.
    pub fn exist(&mut self, entity_id: EntityId) -> bool {
        self.entity_manager.has(entity_id)
    }

    /// Get an EntityRef from an EntityId, return None if id doesn't exist in world.
    pub fn entity(&mut self, entity_id: EntityId) -> Option<EntityRef<'_>> {
        if self.exist(entity_id) {
            Some(EntityRef::new(self, entity_id))
        } else {
            None
        }
    }

    /// Get all entities ids
    pub fn entities(&self) -> &[EntityId] {
        self.entity_manager.entities()
    }

    /// Make a group to accelerate the iteration.
    /// ## Details
    /// See [group](crate::group)
    /// ## Panics
    /// * Panic if group same as any group in world
    /// * Panic if component had already been owned by another group
    pub fn make_group<G: Group + 'static>(&mut self, group: G) {
        assert!(!self.has_group(&group),
                "World: Cannot make group because world has a same group");
        assert!(
            {
                let mut ok = true;
                'outer: for world_group in &self.groups {
                    let world_group = world_group.read().unwrap();
                    for world_group_owning in &world_group.owning_types() {
                        for owning in &group.owning_types() {
                            if *owning == *world_group_owning {
                                ok = false;
                                break 'outer;
                            }
                        }
                    }
                }
                ok
            },
            "World: Cannot make group because component was owned by another group"
        );

        let mut group = group;
        group.make_group_in_world(&self);
        self.groups.push(RwLock::new(Box::new(group)));
    }

    /// Check world has same group  
    /// Return true if group was same as any group in world
    pub fn has_group<G: Group + 'static>(&self, group: &G) -> bool {
        for world_group in &self.groups {
            let world_group = world_group.read().unwrap();
            if group.type_id_a() == world_group.type_id_a()
                && group.type_id_b() == world_group.type_id_b()
                && group.owning_types() == world_group.owning_types()
            {
                return true;
            }
        }
        false
    }

    pub(in crate) fn group<G: Group + 'static>(&self, group: &G) ->RwLockReadGuard<Box<dyn Group>> {
        self.groups
            .iter()
            .find(|world_group| {
                let world_group = world_group.read().unwrap();
                group.type_id_a() == world_group.type_id_a()
                    && group.type_id_b() == world_group.type_id_b()
                    && group.owning_types() == world_group.owning_types()
            })
            // unwrap here
            // the existence will be ensured by outside function
            .unwrap()
            .read()
            .unwrap()
    }

    /// Query entities
    /// ## Details
    /// See [query](crate::query)
    pub fn query<'a, T: Queryable<'a>>(
        &'a self,
    ) -> Box<dyn QueryIterator<Item = <T as Queryable>::Item> + 'a> {
        <T as Queryable<'a>>::query(self)
    }

    #[allow(dead_code)]
    fn with_components_ref<T,F>(&self,mut f : F)
    where F : FnMut(&[T]),
          T : Component{
        let id = TypeId::of::<T>();
        let storage = self.storage_ref(id).unwrap();
        let components = unsafe {
            storage.downcast_ref::<SparseSet<EntityId,T>>()
        }.data();
        f(components)
    }

}

impl Debug for World {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("entities", &self.entity_manager)
            .field(
                "components",
                &self.components.keys().cloned().collect::<Vec<TypeId>>(),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::component::Component;
    use std::fmt::Debug;
    use crate::group::{full_owning, non_owning, partial_owning};
    use crate::query::WithId;
    use crate::world::World;

    #[test]
    fn component_test() {
        let mut world = World::new();
        world.register::<char>();
        let id1 = world.create_entity().into_id();
        let id2 = world.create_entity().into_id();
        let _id3 = world.create_entity().into_id();

        world.attach_component(id1, 'c');
        world.attach_component(id2, 'a');

        world.with_components_ref(|components : &[char]|{
            assert_eq!(components,&['c','a'])
        });

        world.remove_entity(id1);

        world.with_components_ref(|components : &[char]|{
            assert_eq!(components,&['a'])
        });
    }

    #[test]
    fn group_test() {

        let mut world = World::new();

        world.register::<u32>();
        world.register::<char>();
        world.register::<()>();

        fn print<T>(world : &World,msg : &str)
        where T: Component + Clone + Debug {
            let v = world.query::<&T>()
                .with_id()
                .map(|(id,data)|(id,data.clone()))
                .collect::<Vec<_>>();
            println!("{}:{:?}",msg,&v);
        }

        world.create_entity().attach(1u32).attach(());
        let id2 = world.create_entity().attach(2u32).into_id();
        let id3 = world
            .create_entity()
            .attach(3u32)
            .attach('a')
            .attach(())
            .into_id();
        world.create_entity().attach(4u32).attach('b');
        world.create_entity().attach(5u32).attach('c');
        world.create_entity().attach(6u32);
        let id7 = world.create_entity().attach('d').attach(()).into_id();
        println!("#initial");
        print::<u32>(&world, "u32 :");
        print::<char>(&world, "char:");
        print::<()>(&world, "()  :");
        println!();

        world.make_group(full_owning::<u32, char>());
        world.make_group(non_owning::<u32, char>());
        world.make_group(partial_owning::<(), u32>());
        println!("#Made group full/non<u32,char> partial_owning<(),u32>");
        print::<u32>(&world, "u32 :");
        print::<char>(&world, "char:");
        print::<()>(&world, "()  :");
        println!();

        world
            .entity(id2)
            .and_then(|entity| Some(entity.attach('b')));
        println!("#attach component char b for id=2");
        print::<u32>(&world, "u32 :");
        print::<char>(&world, "char:");
        print::<()>(&world, "()  :");
        println!();

        world.entity(id7).unwrap().attach(2u32);
        println!("#attach component u32=2 for id=7");
        print::<u32>(&world, "u32 :");
        print::<char>(&world, "char:");
        print::<()>(&world, "()  :");
        println!();

        world.entity(id3).unwrap().detach::<u32>();
        println!("#detach component u32 for id=3");
        print::<u32>(&world, "u32 :");
        print::<char>(&world, "char:");
        print::<()>(&world, "()  :");
        println!();
    }

    #[test]
    fn debug_trait_test() {
        let mut world = World::new();

        world.register::<char>();
        world.register::<u32>();

        world.create_entity().attach('c').attach(12_u32);
        world.create_entity().attach('a');

        world.make_group(full_owning::<char, u32>());

        world.create_entity().attach('c').attach(12_u32);
        world.create_entity().attach('a');

        println!("{:?}", world);
    }

}
