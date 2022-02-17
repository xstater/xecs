//! ## Concurrency Safety
//! Because [Component](crate::component::Component) is just ```T : Send + Sync```.
//! [World](crate::world::World) can use [RwLock](std::sync::RwLock) to 
//! ensure the borrow check relations of all components.And [World](crate::world::World) can also
//! be ```Send + Sync```.Therefore,the all other states of world can be guarded
//! by [RwLock](std::sync::RwLock).So we can use world in concurrency environment by ```RwLock<World>```.
use crate::component::{Component, StorageRead, ComponentStorage, StorageWrite};
use crate::entity::{Entity, EntityId, EntityManager};
use crate::group::{Group, GroupType};
use crate::query::{QueryIterator, Queryable};
use crate::resource::{Resource, ResourceRead, ResourceWrite};
use crate::sparse_set::SparseSet;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// World is the core of XECS.It manages all components and entities
pub struct World {
    entity_manager: RwLock<EntityManager>,
    // Box<SparseSet<EntityId,Component>>
    components: HashMap<TypeId,RwLock<Box<dyn ComponentStorage>>>,
    groups: Vec<RwLock<Box<dyn Group>>>,
    resources : HashMap<TypeId,RwLock<Box<dyn Resource>>>
}

impl World {
    /// Create a empty world.
    pub fn new() -> World {
        World {
            entity_manager: RwLock::new(EntityManager::new()),
            components: Default::default(),
            groups: Default::default(),
            resources : Default::default()
        }
    }

    /// Register resource in world 
    pub fn register_resource<R : Resource>(&mut self,resource : R) {
        let type_id = TypeId::of::<R>();
        self.resources.insert(type_id,RwLock::new(Box::new(resource)));
    }

    /// Get a read guard of resource
    pub fn resource_read<R : Resource>(&self) -> Option<ResourceRead<'_,R>> {
        let type_id = TypeId::of::<R>();
        let lock = self.resources.get(&type_id)?
            .read().unwrap();
        Some(ResourceRead::new(lock))
    }

    /// Get a write guard of resource
    pub fn resource_write<R : Resource>(&self) -> Option<ResourceWrite<'_,R>> {
        let type_id = TypeId::of::<R>();
        let lock = self.resources.get(&type_id)?
            .write().unwrap();
        Some(ResourceWrite::new(lock))
    }

    /// Register a component.
    /// # Panics
    /// Panic if component is registered.
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

    /// Check if component is registered.
    pub fn has_registered<T: Component>(&self) -> bool {
        let type_id = TypeId::of::<T>();
        self.components.contains_key(&type_id)
    }

    /// Create an entity without any component in World,
    ///  return an [EntityBuilder](crate::entity::EntityBuilder).
    pub fn create_entity(&self) -> Entity<'_> {
        let id = {
            let mut entity_manager = self.entity_manager.write().unwrap();
            entity_manager.create()
        };
        Entity::new(self, id)
    }

    /// Remove entity and its components.
    pub fn remove_entity(&self, entity_id: EntityId) {
        assert!(self.exist(entity_id),
                "World:Cannot remove a non-exists entity");
        // remove entity from manager
        {
            let mut entity_manager = self.entity_manager.write().unwrap();
            entity_manager.remove(entity_id);
        }
        // find all groups need remove 
        let mut groups = vec![];
        for group in &self.groups {
            let need_remove = {
                let group = group.read().unwrap();
                let (type_a,type_b) = group.group_type().types();
                let comp_a = self.raw_storage_read(type_a).unwrap();
                let comp_b = self.raw_storage_read(type_b).unwrap();
                group.in_group(entity_id, &comp_a, &comp_b)
            };
            if need_remove {
                groups.push(group.write().unwrap());
            };
        }
        // remove entity in group and its storages
        for mut group in groups {
            match group.group_type() {
                GroupType::FullOwning(type_a,type_b) => {
                    let mut comp_a = self.raw_storage_write(type_a).unwrap();
                    let mut comp_b = self.raw_storage_write(type_b).unwrap();
                    unsafe {
                        group.downcast_full_owning()
                    }.remove(entity_id,&mut comp_a,&mut comp_b);
                    comp_a.remove(entity_id);
                    comp_b.remove(entity_id);
                },
                GroupType::PartialOwning(type_a,type_b) => {
                    let mut comp_a = self.raw_storage_write(type_a).unwrap();
                    let comp_b = self.raw_storage_read(type_b).unwrap();
                    unsafe {
                        group.downcast_partial_owning()
                    }.remove(entity_id,&mut comp_a,&comp_b);
                    comp_a.remove(entity_id);
                }
                GroupType::NonOwning(type_a,type_b) => {
                    let comp_a = self.raw_storage_read(type_a).unwrap();
                    let comp_b = self.raw_storage_read(type_b).unwrap();
                    unsafe {
                        group.downcast_non_owning()
                    }.remove(entity_id,&comp_a,&comp_b);
                },
            }
        }
        // remove entity in other storages
        let mut storages = vec![];
        for storage in self.components.values() {
            let need_remove = {
                let storage = storage.read().unwrap();
                storage.has(entity_id)
            };
            if need_remove {
                storages.push(storage.write().unwrap());
            }
        }
        for mut storage in storages {
            storage.remove(entity_id);
        }
    }

    /// Get lock guard of raw component storage,
    /// return None if component is not registered.
    pub(in crate) fn raw_storage_read(&self,id : TypeId) 
        -> Option<RwLockReadGuard<'_,Box<dyn ComponentStorage>>> {
        self.components
            .get(&id)
            .map(|rwlock|rwlock.read().unwrap())
    }

    /// Get lock guard of raw component storage,
    /// return None if component is not registered.
    pub(in crate) fn raw_storage_write(&self,id : TypeId) 
        -> Option<RwLockWriteGuard<'_,Box<dyn ComponentStorage>>> {
        self.components
            .get(&id)
            .map(|rwlock|rwlock.write().unwrap())
    }

    /// Attach a component to an entity.  
    /// # Panics
    /// * Panic if ```T``` is not registered.
    /// * Panic if ```entity_id``` not exist.
    pub fn attach_component<T: Component>(&self, entity_id: EntityId,component: T) {
        assert!(self.has_registered::<T>(),
                "World:Cannot attach component because components has not been registered.");
        assert!(self.exist(entity_id),
                "World:Cannot attach component to a non-exist entity");
        let type_id = TypeId::of::<T>();
        {
            // Unwrap never fails because assert ensures this
            let mut storage = self.raw_storage_write(type_id).unwrap();
            // SAFTY:
            // storage is SparseSet<EntityId,T>
            let sparse_set = unsafe {
                storage.downcast_mut::<SparseSet<EntityId,T>>()
            };
            sparse_set.add(entity_id,component);
        }
        let mut groups = vec![];
        for group in &self.groups {
            let need_add = {
                let group = group.read().unwrap();
                let (type_id_a,type_id_b) = group.group_type().types();
                type_id_a == type_id || type_id_b == type_id
            };
            if need_add {
                groups.push(group.write().unwrap())
            }
        }
        for mut group in groups {
            match group.group_type() {
                GroupType::FullOwning(type_a,type_b) => {
                    let mut comp_a = self.raw_storage_write(type_a).unwrap();
                    let mut comp_b = self.raw_storage_write(type_b).unwrap();
                    unsafe {
                        group.downcast_full_owning()
                    }.add(entity_id,&mut comp_a,&mut comp_b);
                },
                GroupType::PartialOwning(type_a,type_b) => {
                    let mut comp_a = self.raw_storage_write(type_a).unwrap();
                    let comp_b = self.raw_storage_read(type_b).unwrap();
                    unsafe {
                        group.downcast_partial_owning()
                    }.add(entity_id,&mut comp_a,&comp_b);
                },
                GroupType::NonOwning(type_a,type_b) => {
                    let comp_a = self.raw_storage_read(type_a).unwrap();
                    let comp_b = self.raw_storage_read(type_b).unwrap();
                    unsafe {
                        group.downcast_non_owning()
                    }.add(entity_id,&comp_a,&comp_b);
                }
            }
        }
    }

    /// Detach a component from an entity.
    /// # Details
    /// Return ```None``` if entity doesn't have this component,  
    /// otherwise return ```Some(component)```
    /// # Panics
    /// * Panic if ```T``` is not registered.
    /// * Panic if ```entity_id``` not exist.
    pub fn detach_component<T: Component>(&self, entity_id: EntityId) -> Option<T> {
        assert!(self.has_registered::<T>(),
                "World:Cannot detach component because components has not been registered.");
        assert!(self.exist(entity_id),
                "World:Cannot detach component from a non-exist entity");
        let type_id = TypeId::of::<T>();
        let mut groups = vec![];
        for group in &self.groups {
            let need_remove = {
                let group = group.read().unwrap();
                let (type_id_a,type_id_b) = group.group_type().types();
                type_id_a == type_id || type_id_b == type_id
            };
            if need_remove {
                groups.push(group.write().unwrap())
            }
        }
        for mut group in groups {
            match group.group_type() {
                GroupType::FullOwning(type_a,type_b) => {
                    let mut comp_a = self.raw_storage_write(type_a).unwrap();
                    let mut comp_b = self.raw_storage_write(type_b).unwrap();
                    unsafe {
                        group.downcast_full_owning()
                    }.remove(entity_id,&mut comp_a,&mut comp_b);
                },
                GroupType::PartialOwning(type_a,type_b) => {
                    let mut comp_a = self.raw_storage_write(type_a).unwrap();
                    let comp_b = self.raw_storage_read(type_b).unwrap();
                    unsafe {
                        group.downcast_partial_owning()
                    }.remove(entity_id,&mut comp_a,&comp_b);
                },
                GroupType::NonOwning(type_a,type_b) => {
                    let comp_a = self.raw_storage_read(type_a).unwrap();
                    let comp_b = self.raw_storage_read(type_b).unwrap();
                    unsafe {
                        group.downcast_non_owning()
                    }.remove(entity_id,&comp_a,&comp_b);
                }
            }
        }

        // Unwrap never fails because assert ensures this
        let mut storage = self.raw_storage_write(type_id).unwrap();
        // SAFTY:
        // storage is SparseSet<EntityId,T>
        let sparse_set = unsafe {
            storage.downcast_mut::<SparseSet<EntityId,T>>()
        };
        sparse_set.remove(entity_id)
    }

    /// Check if ```entity_id``` exists in World.
    pub fn exist(&self, entity_id: EntityId) -> bool {
        let entity_manager = self.entity_manager.read().unwrap();
        entity_manager.has(entity_id)
    }

    /// Get the component storage's read guard
    pub fn components_read<T : Component>(&self) -> Option<StorageRead<'_,T>> {
        let type_id = TypeId::of::<T>();
        let lock = self.raw_storage_read(type_id)?;
        Some(StorageRead::from_lock(lock))
    }

    /// Get the component storage's write guard
    pub fn components_write<T : Component>(&self) -> Option<StorageWrite<'_,T>> {
        let type_id = TypeId::of::<T>();
        let lock = self.raw_storage_write(type_id)?;
        Some(StorageWrite::from_lock(lock))
    }

    /// Make a [group](crate::group) to accelerate the iteration.
    /// ## Panics
    /// * Panic if ```group``` is the same as another group in [World](crate::world::World).
    /// * Panic if component is owned by another group.
    pub fn make_group<G: Group + 'static>(&mut self, group: G) {
        assert!(!self.has_group(&group),
                "World: Cannot make group because world has a same group");
        assert!(
            {
                let mut ok = true;
                'outer: for world_group in &self.groups {
                    let world_group = world_group.read().unwrap();
                    for owning_type in world_group.group_type().owning() {
                        if group.group_type().owned(owning_type) {
                            ok = false;
                            break 'outer;
                        }
                    }
                }
                ok
            },
            "World: Cannot make group because component was owned by another group"
        );

        self.groups.push(RwLock::new(Box::new(group)));
        let group = self.groups.last().unwrap();
        let mut group = group.write().unwrap();
        match group.group_type() {
            GroupType::FullOwning(type_a,type_b) => {
                let mut comp_a = self.raw_storage_write(type_a).unwrap();
                let mut comp_b = self.raw_storage_write(type_b).unwrap();
                unsafe {
                    group.downcast_full_owning()
                }.make(&mut comp_a,&mut comp_b);
            },
            GroupType::PartialOwning(type_a,type_b) => {
                let mut comp_a = self.raw_storage_write(type_a).unwrap();
                let comp_b = self.raw_storage_read(type_b).unwrap();
                unsafe {
                    group.downcast_partial_owning()
                }.make(&mut comp_a,&comp_b);
            },
            GroupType::NonOwning(type_a,type_b) => {
                let comp_a = self.raw_storage_read(type_a).unwrap();
                let comp_b = self.raw_storage_read(type_b).unwrap();
                unsafe {
                    group.downcast_non_owning()
                }.make(&comp_a,&comp_b);
            },
        }
    }

    /// Check if (group)[crate::group] exists in [World](crate::world::World).
    /// Return true if group is same as another group in World.
    pub(in crate) fn has_group<G: Group + 'static>(&self, group: &G) -> bool {
        for world_group in &self.groups {
            let world_group = world_group.read().unwrap();
            if world_group.group_type() == group.group_type() {
                return true;
            }
        }
        false
    }

    pub(in crate) fn group<G: Group + 'static>(&self, group: &G) -> RwLockReadGuard<Box<dyn Group>> {
        self.groups
            .iter()
            .find(|world_group| {
                let world_group = world_group.read().unwrap();
                world_group.group_type() == group.group_type()
            })
            // unwrap here
            // existence will be ensured by an outside function
            .unwrap()
            .read()
            .unwrap()
    }

    /// [Query](crate::query) entities with conditions
    pub fn query<'a, T: Queryable<'a>>(
        &'a self,
    ) -> Box<dyn QueryIterator<Item = <T as Queryable>::Item> + 'a> {
        <T as Queryable<'a>>::query(self)
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
        let id1 = world.create_entity().id();
        let id2 = world.create_entity().id();
        let _id3 = world.create_entity().id();

        world.attach_component(id1, 'c');
        world.attach_component(id2, 'a');

        {
            let components = world.components_read::<char>().unwrap();
            let components = components.data();
            assert_eq!(components,&['c','a'])
        }
        world.remove_entity(id1);

        {
            let components = world.components_read::<char>().unwrap();
            let components = components.data();
            assert_eq!(components,&['a'])
        }
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
        let id2 = world.create_entity().attach(2u32).id();
        let id3 = world
            .create_entity()
            .attach(3u32)
            .attach('a')
            .attach(())
            .id();
        world.create_entity().attach(4u32).attach('b');
        world.create_entity().attach(5u32).attach('c');
        world.create_entity().attach(6u32);
        let id7 = world.create_entity().attach('d').attach(()).id();
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

        world.attach_component(id2,'b');
        println!("#attach component char b for id=2");
        print::<u32>(&world, "u32 :");
        print::<char>(&world, "char:");
        print::<()>(&world, "()  :");
        println!();

        world.attach_component(id7,2u32);
        println!("#attach component u32=2 for id=7");
        print::<u32>(&world, "u32 :");
        print::<char>(&world, "char:");
        print::<()>(&world, "()  :");
        println!();

        world.detach_component::<u32>(id3);
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

    #[test]
    fn resource_test() {
        let mut world = World::new();
        #[derive(Debug)]
        struct Test {
            name : String,
            age : u32
        }
        
        world.register_resource(Test{
            name : "affff".to_string(),
            age : 12
        });

        assert!(world.resource_read::<Test>().is_some());
        assert_eq!(world.resource_read::<Test>().unwrap().age,12);

        world.resource_write::<Test>().unwrap().age = 13;

        assert_eq!(world.resource_read::<Test>().unwrap().age,13);
        assert_eq!(&world.resource_read::<Test>().unwrap().name,"affff");
    }

}
