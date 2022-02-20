//! ## Concurrency Safety
//! Because [Component](crate::component::Component) is just ```T : Send + Sync```.
//! [World](crate::world::World) can use [RwLock](std::sync::RwLock) to 
//! ensure the borrow check relations of all components.And [World](crate::world::World) can also
//! be ```Send + Sync```.Therefore,the all other states of world can be guarded
//! by [RwLock](std::sync::RwLock).So we can use world in concurrency environment by ```RwLock<World>```.
use crate::component::{Component, ComponentRead, ComponentStorage, ComponentWrite, StorageRead, StorageWrite};
use crate::entity::{Entity, EntityId, EntityManager};
use crate::group::Group;
use crate::query::{QueryIterator, Queryable};
use crate::resource::{Resource, ResourceRead, ResourceWrite};
use crate::sparse_set::SparseSet;
use std::any::TypeId;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// World is the core of XECS.It manages all components and entities
pub struct World {
    entity_manager: RwLock<EntityManager>,
    // Box<SparseSet<EntityId,Component>>
    components: HashMap<TypeId,RwLock<Box<dyn ComponentStorage>>>,
    groups: Vec<RwLock<Group>>,
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
            .read();
        Some(ResourceRead::new(lock))
    }

    /// Get a write guard of resource
    pub fn resource_write<R : Resource>(&self) -> Option<ResourceWrite<'_,R>> {
        let type_id = TypeId::of::<R>();
        let lock = self.resources.get(&type_id)?
            .write();
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
    ///  return an [Entity](crate::entity::Entity).
    pub fn create_entity(&self) -> Entity<'_> {
        let id = {
            let mut entity_manager = self.entity_manager.write();
            entity_manager.create()
        };
        self.entity(id).unwrap()
    }

    /// Remove entity and its components.
    pub fn remove_entity(&self, entity_id: EntityId) {
        assert!(self.exist(entity_id),
                "World:Cannot remove a non-exists entity");
        // find all groups need remove 
        let mut groups = vec![];
        for group in &self.groups {
            let need_remove = {
                let group = group.read();
                let (type_a,type_b) = group.types();
                let comp_a = self.raw_storage_read(type_a).unwrap();
                let comp_b = self.raw_storage_read(type_b).unwrap();
                group.in_group(entity_id, &comp_a, &comp_b)
            };
            if need_remove {
                groups.push(group.write());
            };
        }
        // remove entity in group and its storages
        for mut group in groups {
            match &mut *group{
                Group::FullOwning(data) => {
                    let (type_a,type_b) = data.types();
                    let mut comp_a = self.raw_storage_write(type_a).unwrap();
                    let mut comp_b = self.raw_storage_write(type_b).unwrap();
                    data.remove(entity_id,&mut comp_a,&mut comp_b);
                    comp_a.remove(entity_id);
                    comp_b.remove(entity_id);
                },
                Group::PartialOwning(data) => {
                    let (type_a,type_b) = data.types();
                    let mut comp_a = self.raw_storage_write(type_a).unwrap();
                    let comp_b = self.raw_storage_read(type_b).unwrap();
                    data.remove(entity_id,&mut comp_a,&comp_b);
                    comp_a.remove(entity_id);
                }
                Group::NonOwning(data) => {
                    let (type_a,type_b) = data.types();
                    let comp_a = self.raw_storage_read(type_a).unwrap();
                    let comp_b = self.raw_storage_read(type_b).unwrap();
                    data.remove(entity_id,&comp_a,&comp_b);
                },
            }
        }
        // remove entity in other storages
        let mut storages = vec![];
        for storage in self.components.values() {
            let need_remove = {
                let storage = storage.read();
                storage.has(entity_id)
            };
            if need_remove {
                storages.push(storage.write());
            }
        }
        for mut storage in storages {
            storage.remove(entity_id);
        }
        // remove entity from manager
        {
            let mut entity_manager = self.entity_manager.write();
            entity_manager.remove(entity_id);
        }
    }

    /// Get lock guard of raw component storage,
    /// return None if component is not registered.
    pub(in crate) fn raw_storage_read(&self,id : TypeId) 
        -> Option<RwLockReadGuard<'_,Box<dyn ComponentStorage>>> {
        self.components
            .get(&id)
            .map(|rwlock|rwlock.read())
    }

    /// Get lock guard of raw component storage,
    /// return None if component is not registered.
    pub(in crate) fn raw_storage_write(&self,id : TypeId) 
        -> Option<RwLockWriteGuard<'_,Box<dyn ComponentStorage>>> {
        self.components
            .get(&id)
            .map(|rwlock|rwlock.write())
    }

    /// Attach a component to an entity.  
    /// # Panics
    /// * Panic if ```T``` is not registered.
    /// * Panic if ```entity_id``` not exist.
    pub fn attach_component<T: Component>(&self, entity_id: EntityId,component: T) {
        self.entity(entity_id)
            .expect("World: Cannot attach component to a non-existence entity")
            .attach(component);
    }

    /// Detach a component from an entity.
    /// # Details
    /// Return ```None``` if entity doesn't have this component,  
    /// otherwise return ```Some(component)```
    /// # Panics
    /// * Panic if ```T``` is not registered.
    /// * Panic if ```entity_id``` not exist.
    pub fn detach_component<T: Component>(&self, entity_id: EntityId) -> Option<T> {
        self.entity(entity_id)
            .expect("World: Cannot detach component to a non-existence entity")
            .detach::<T>()
    }

    /// Check if ```entity_id``` exists in World.
    pub fn exist(&self, entity_id: EntityId) -> bool {
        let entity_manager = self.entity_manager.read();
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

    /// Get the read guard of component of an entity
    pub fn entity_component_read<T : Component>(&self,id : EntityId) -> Option<ComponentRead<'_,T>> {
        let lock = self.components_read::<T>()?;
        if lock.exist(id) {
            Some(unsafe {
                ComponentRead::new(id,lock)
            })
        } else {
            None
        }
    }

    /// Get the write guard of component of an entity
    pub fn entity_component_write<T : Component>(&self,id : EntityId) -> Option<ComponentWrite<'_,T>> {
        let lock = self.components_write::<T>()?;
        if lock.exist(id) {
            Some(unsafe {
                ComponentWrite::new(id,lock)
            })
        } else {
            None
        }
    }

    /// Get an [Entity](crate::entity::Entity) from an entity id
    pub fn entity(&self,id : EntityId) -> Option<Entity<'_>> {
        let lock = self.entity_manager.read();
        if lock.has(id) {
            Some(Entity::new(&self, lock, id))
        } else {
            None
        }
    }

    /// Make a [group](crate::group) to accelerate the iteration.
    /// ## Panics
    /// * Panic if ```group``` is the same as another group in [World](crate::world::World).
    /// * Panic if component is owned by another group.
    pub fn make_group<G : Into<Group> + 'static + Copy>(&mut self, group: G) {
        assert!(!self.has_group(group),
                "World: Cannot make group because world has a same group");
        let group = group.into();
        assert!(
            {
                let mut ok = true;
                'outer: for world_group in &self.groups {
                    let world_group = world_group.read();
                    for owning_type in world_group.owning() {
                        if group.owned(owning_type) {
                            ok = false;
                            break 'outer;
                        }
                    }
                }
                ok
            },
            "World: Cannot make group because component was owned by another group"
        );

        self.groups.push(RwLock::new(group));
        let group = self.groups.last().unwrap();
        let mut group = group.write();
        match &mut *group{
            Group::FullOwning(data) => {
                let (type_a,type_b) = data.types();
                let mut comp_a = self.raw_storage_write(type_a).unwrap();
                let mut comp_b = self.raw_storage_write(type_b).unwrap();
                data.make(&mut comp_a,&mut comp_b);
            },
            Group::PartialOwning(data) => {
                let (type_a,type_b) = data.types();
                let mut comp_a = self.raw_storage_write(type_a).unwrap();
                let comp_b = self.raw_storage_read(type_b).unwrap();
                data.make(&mut comp_a,&comp_b);
            },
            Group::NonOwning(data) => {
                let (type_a,type_b) = data.types();
                let comp_a = self.raw_storage_read(type_a).unwrap();
                let comp_b = self.raw_storage_read(type_b).unwrap();
                data.make(&comp_a,&comp_b);
            },
        }
    }

    /// Check if (group)[crate::group] exists in [World](crate::world::World).
    /// Return true if group is same as another group in World.
    pub(in crate) fn has_group<G : Into<Group> + 'static>(&self, group: G) -> bool {
        let group = group.into();
        for world_group in &self.groups {
            let world_group = world_group.read();
            if world_group.eq(&group) {
                return true;
            }
        }
        false
    }

    pub(in crate) fn group<G : Into<Group> + 'static>(&self, group: G) -> RwLockReadGuard<Group> {
        let group = group.into();
        self.groups
            .iter()
            .find(|world_group| {
                let world_group = world_group.read();
                world_group.eq(&group)
            })
            // unwrap here
            // existence will be ensured by an outside function
            .unwrap()
            .read()
    }

    pub(in crate) fn groups(&self,type_id : TypeId) -> Vec<RwLockWriteGuard<'_,Group>> {
        let mut groups = vec![];
        for group in &self.groups {
            let need_add = {
                let group = group.read();
                let (type_id_a,type_id_b) = group.types();
                type_id_a == type_id || type_id_b == type_id
            };
            if need_add {
                groups.push(group.write())
            }
        }
        groups
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
    use crate::resource::Resource;
    use std::fmt::Debug;
    use crate::group::{full_owning, non_owning, partial_owning};
    use crate::query::WithId;
    use crate::world::World;

    #[derive(Debug,Clone,Copy,PartialEq)]
    struct Char(char);
    #[derive(Debug,Clone,Copy,PartialEq)]
    struct U32(u32);
    #[derive(Debug,Clone,Copy,PartialEq)]
    struct Unit;

    impl Component for Char{}
    impl Component for U32{}
    impl Component for Unit{}

    #[test]
    fn component_test() {
        let mut world = World::new();
        world.register::<Char>();
        let id1 = world.create_entity().into_id();
        let id2 = world.create_entity().into_id();
        let _id3 = world.create_entity().into_id();

        world.attach_component(id1,Char('c'));
        world.attach_component(id2,Char('a'));

        {
            let components = world.components_read::<Char>().unwrap();
            let components = components.data();
            assert_eq!(components,&[Char('c'),Char('a')])
        }
        world.remove_entity(id1);

        {
            let components = world.components_read::<Char>().unwrap();
            let components = components.data();
            assert_eq!(components,&[Char('a')])
        }
    }

    #[test]
    fn group_test() {

        let mut world = World::new();
        world.register::<U32>();
        world.register::<Char>();
        world.register::<Unit>();

        fn print<T>(world : &World,msg : &str)
        where T: Component + Clone + Debug {
            let v = world.query::<&T>()
                .with_id()
                .map(|(id,data)|(id,data.clone()))
                .collect::<Vec<_>>();
            println!("{}:{:?}",msg,&v);
        }

        world.create_entity().attach(U32(1)).attach(Unit);
        let id2 = world.create_entity().attach(U32(2)).into_id();
        let id3 = world
            .create_entity()
            .attach(U32(3))
            .attach(Char('a'))
            .attach(Unit)
            .into_id();
        world.create_entity().attach(U32(4)).attach(Char('b'));
        world.create_entity().attach(U32(5)).attach(Char('c'));
        world.create_entity().attach(U32(6));
        let id7 = world.create_entity().attach(Char('d')).attach(Unit).into_id();
        println!("#initial");
        print::<U32>(&world, "u32 :");
        print::<Char>(&world, "char:");
        print::<Unit>(&world, "()  :");
        println!();

        dbg!("Here");
        world.make_group(full_owning::<U32, Char>());
        dbg!("Here");
        world.make_group(non_owning::<U32, Char>());
        dbg!("Here");
        world.make_group(partial_owning::<Unit, U32>());
        dbg!("Here");
        println!("#Made group full/non<u32,char> partial_owning<(),u32>");
        print::<U32>(&world, "u32 :");
        print::<Char>(&world, "char:");
        print::<Unit>(&world, "()  :");
        println!();

        world.attach_component(id2,Char('b'));
        println!("#attach component char b for id=2");
        print::<U32>(&world, "u32 :");
        print::<Char>(&world, "char:");
        print::<Unit>(&world, "()  :");
        println!();

        world.attach_component(id7,U32(2));
        println!("#attach component u32=2 for id=7");
        print::<U32>(&world, "u32 :");
        print::<Char>(&world, "char:");
        print::<Unit>(&world, "()  :");
        println!();

        world.detach_component::<U32>(id3);
        println!("#detach component u32 for id=3");
        print::<U32>(&world, "u32 :");
        print::<Char>(&world, "char:");
        print::<Unit>(&world, "()  :");
        println!();
    }

    #[test]
    fn debug_trait_test() {
        let mut world = World::new();

        world.register::<Char>();
        world.register::<U32>();

        world.create_entity().attach(Char('c')).attach(U32(12));
        world.create_entity().attach(Char('a'));

        world.make_group(full_owning::<Char, U32>());

        world.create_entity().attach(Char('c')).attach(U32(12));
        world.create_entity().attach(Char('a'));

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
        impl Resource for Test {}
        
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

    #[test]
    fn entity_component_test() {
        let mut world = World::new();

        world.register::<U32>();

        world.create_entity().attach(U32(5));
        let id = world.create_entity().attach(U32(7)).into_id();
        world.create_entity().attach(U32(2));

        {
            let v = world.entity_component_read::<U32>(id).unwrap();
            assert_eq!(*v,U32(7));
        }

        {
            let mut v = world.entity_component_write::<U32>(id).unwrap();
            *v = U32(3);
        }

        {
            let v = world.entity_component_read::<U32>(id).unwrap();
            assert_eq!(*v,U32(3));
        }
    }

    #[test]
    fn entity_test() {
        let mut world = World::new();

        world.register::<U32>();

        world.create_entity().attach(U32(5));
        let id = world.create_entity().attach(U32(7)).into_id();
        world.create_entity().attach(U32(2));

        let entity = world.entity(id).unwrap();

        {
            let v = entity.component_read::<U32>().unwrap();
            assert_eq!(*v,U32(7));
        }

        {
            let mut v = entity.component_write::<U32>().unwrap();
            *v = U32(3);
        }

        {
            let v = entity.component_read::<U32>().unwrap();
            assert_eq!(*v,U32(3));
        }
    }
}
