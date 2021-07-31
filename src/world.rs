use crate::{EntityId, Component};
use std::collections::HashMap;
use std::any::TypeId;
use std::cell::{RefCell, Ref, RefMut};
use crate::components::ComponentStorage;
use xsparseset::SparseSet;
use crate::entity::EntityRef;
use crate::group::{Group, NonOwningGroup, OwningType, OwningGroup};
use std::fmt::{Debug, Formatter};

pub struct World {
    // entity_flags[0] : Because the ID 0 is not a valid ID,
    //     so the first one can be used to store the last removed ID
    //     Unavailable(_)      -> there is no entityID for reuse
    //     Available(EntityID) -> the EntityID
    entity_flags : Vec<EntityFlag>,
    entities : Vec<EntityId>,
    // Box<SparseSet<EntityId,Component>>
    components: HashMap<TypeId,RefCell<Box<dyn ComponentStorage>>>,
    groups : Vec<RefCell<Group>>
}

impl World {
    pub fn new() -> World {
        World {
            entity_flags : vec![EntityFlag::Unavailable(0)],
            entities : vec![],
            components: Default::default(),
            groups : vec![]
        }
    }

    pub fn register<T : Component>(&mut self) {
        let type_id = TypeId::of::<T>();
        debug_assert!(
            !self.components.contains_key(&type_id),
            "Cannot register a component as twice");
        self.components.insert(
           type_id,
            RefCell::new(
                Box::new(
                    SparseSet::<EntityId,T>::new()
                ))
        );
    }

    pub fn create_entity(&mut self) -> EntityRef<'_>{
        //safe here:
        // the entity_flags[0] cannot be removed
        if let EntityFlag::Available(last_id) = self.entity_flags.first().unwrap() {
            let last_id = *last_id;
            //we got an id can be reused
            let new_id = self.entity_flags[last_id.get()];
            self.entities.push(last_id);
            self.entity_flags[last_id.get()] = EntityFlag::Unavailable(self.entities.len() - 1);
            self.entity_flags[0] = new_id;
            EntityRef::new(self,last_id)
        }else{
            //full
            let id = self.entity_flags.len();
            // safe here: because id cannot be zero
            let id = unsafe { EntityId::new_unchecked(id) };
            self.entities.push(id);
            self.entity_flags.push(EntityFlag::Unavailable(self.entities.len() - 1));
            //safe here because this id can't be 0
            EntityRef::new(self, id)
        }
    }

    pub fn remove_entity(&mut self,entity_id : EntityId){
        let entity_id_ = entity_id.get();
        if let EntityFlag::Unavailable(index) = self.entity_flags[entity_id_] {
            // unwrap safe : in this branch,we must has one entity at least
            let the_last_one_id = self.entities.last().unwrap();
            // move this entity to the end of entities
            self.entity_flags[the_last_one_id.get()] = EntityFlag::Unavailable(index);
            self.entities.swap_remove(index);
            // keep this destroyed id be a chain
            self.entity_flags[entity_id_] = self.entity_flags[0];
            self.entity_flags[0] = EntityFlag::Available(entity_id);
        } else {
            panic!("Entity is not existence");
        }
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

    pub fn components_ref<T : Component>(&self) -> Ref<'_,[T]>{
        let slice_ref = self.components_storage_ref();
        Ref::map(slice_ref,|raw|{
            raw.data()
        })
    }

    pub fn components_mut<T : Component>(&self) -> RefMut<'_,[T]>{
        let slice_mut = self.components_storage_mut();
        RefMut::map(slice_mut,|raw|{
            raw.data_mut()
        })
    }

    pub fn entities(&self) -> &[EntityId] {
        self.entities.as_slice()
    }

    pub fn entities_in<T : Component>(&self) -> Ref<'_,[EntityId]> {
        let storage = self.components_storage_ref::<T>();
        Ref::map(storage,|raw|{
            raw.entities()
        })
    }

    pub fn exist(&mut self,entity_id : EntityId) -> bool {
        let entity_id = entity_id.get();
        if entity_id < self.entity_flags.len() {
            if let EntityFlag::Unavailable(_) = self.entity_flags[entity_id]{
                return true
            }
        }
        return false;
    }

    pub fn entity(&mut self,entity_id : EntityId) -> Option<EntityRef<'_>> {
        if self.exist(entity_id) {
            Some(EntityRef::new(self, entity_id))
        }else{
            None
        }
    }

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
}

impl Debug for World {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("entities",&self.entities)
            .field("groups",&self.groups)
            .field("components",
                   &self.components
                       .keys()
                       .cloned()
                       .collect::<Vec<TypeId>>())
            .finish()
    }
}

#[derive(Debug,Copy,Clone)]
pub enum EntityFlag{
    /// store the next available EntityID
    Available(EntityId),
    /// store the index of EntityID in entities array
    Unavailable(usize)
}
#[cfg(test)]
mod tests{
    use crate::{EntityId, World};
    use crate::group::Group;
    use crate::world::EntityFlag;

    #[test]
    fn flags_test(){
        let mut world = World::new();
        world.create_entity(); // 1
        world.create_entity(); // 2
        world.create_entity(); // 3
        world.create_entity(); // 4
        world.create_entity(); // 5
        println!("#initial");
        println!("flags    :{:?}",world.entity_flags.as_slice());
        println!("entities :{:?}",world.entities.as_slice());
        println!();
        world.remove_entity(EntityId::new(3).unwrap());
        println!("#removed id=3");
        println!("flags :{:?}",world.entity_flags.as_slice());
        println!("entities :{:?}",world.entities.as_slice());
        println!();
        world.remove_entity(EntityId::new(5).unwrap());
        println!("#removed id=5");
        println!("flags :{:?}",world.entity_flags.as_slice());
        println!("entities :{:?}",world.entities.as_slice());
        println!();
        world.remove_entity(EntityId::new(1).unwrap());
        println!("#removed id=1");
        println!("flags :{:?}",world.entity_flags.as_slice());
        println!("entities :{:?}",world.entities.as_slice());
        println!();
        assert_eq!(world.create_entity().into_id(),EntityId::new(1).unwrap());
        println!("#create a new entity, id = 1");
        println!("flags :{:?}",world.entity_flags.as_slice());
        println!("entities :{:?}",world.entities.as_slice());
        println!();
        assert_eq!(world.create_entity().into_id(),EntityId::new(5).unwrap());
        println!("#create a new entity, id = 5");
        println!("flags :{:?}",world.entity_flags.as_slice());
        println!("entities :{:?}",world.entities.as_slice());
        println!();
        assert_eq!(world.create_entity().into_id(),EntityId::new(3).unwrap());
        println!("#create a new entity, id = 3");
        println!("flags :{:?}",world.entity_flags.as_slice());
        println!("entities :{:?}",world.entities.as_slice());
        println!();
        assert_eq!(world.create_entity().into_id(),EntityId::new(6).unwrap());
        println!("#create a new entity, id = 6");
        println!("flags :{:?}",world.entity_flags.as_slice());
        println!("entities :{:?}",world.entities.as_slice());
        println!();
    }

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
    fn test(){
        #[derive(Debug)]
        struct Fuck(i32);
        #[derive(Debug)]
        struct Shit(char);

        let mut world = World::new();
        world.register::<Fuck>();
        world.register::<Shit>();

        let id1 = world.create_entity()
            .attach(Shit('a'))
            .into_id();
        world.create_entity()
            .attach(Fuck(2))
            .attach(Shit('b'))
            .attach(Shit('c'));
        for c in 'a'..='z' {
            world.create_entity()
                .attach(Shit(c));
        }
        world.detach_component::<Shit>(id1);
        world.detach_component::<Shit>(EntityId::new(4).unwrap());
        world.detach_component::<Shit>(EntityId::new(5).unwrap());


        world.create_entity()
            .attach(Fuck(3));
        world.create_entity()
            .attach(Fuck(2));
        world.create_entity()
            .attach(Fuck(5));
        world.create_entity()
            .attach(Fuck(7));
        println!("Shit:{:?}",world.components_ref::<Shit>());
        println!("Fuck:{:?}",world.components_ref::<Fuck>());

        // world.create_entity().with(Fuck(2)).build();
        // let entity = world
        //     .create_entity()
        //     .with(Fuck(2))
        //     .with(Shit('c'))
        //     .build();
        //
        // for (entity, fuck) in world.query::<Fuck>().with_entity().get() {
        //
        // }
        //
        // for (fuck,shit) in world.query::<Fuck>().with::<Shit>().get() {
        //
        // }
        //
        // let fuck_system = |&mut world| {
        //     for (entity,fuck) in world.query::<Fuck>().with_entity() {
        //         println!("{}",fuck.0);
        //     }
        // }.into_system("fuck_system");
        //
        // let fuck_shit_system = |&mut world| {
        //     for (fuck,shit) in world.query::<(&Fuck,&mut Shit)>() {
        //         println!("{} {}",fuck.0,shit.0);
        //     }
        // }.into_system("fuck_shit_system");
        //
        // world
        //     .add_system(fuck_system,&[])
        //     .add_system(fuck_shit_system,&["fuck_shit_system"]);
        //
        // world.run();
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
        println!("Size of EntityFlag:{}Bytes",std::mem::size_of::<EntityFlag>());
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