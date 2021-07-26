use crate::{EntityId, Component};
use std::collections::HashMap;
use std::any::TypeId;
use std::cell::{RefCell, Ref, RefMut};
use crate::components::ComponentStorage;
use xsparseset::SparseSet;
use crate::entity::EntityRef;

pub struct World {
    // entity_flags[0] : Because the ID 0 is not a valid ID,
    //     so the first one can be used to store the last removed ID
    // entity_flags[id] : has 2 state below:
    // None -> This ID is not available now
    //     It means that this ID has already been used
    // Some(id) -> This ID is available
    //      'id' is the next available ID
    entity_flags : Vec<Option<EntityId>>,
    // Box<SparseSet<EntityId,Component>>
    components: HashMap<TypeId,RefCell<Box<dyn ComponentStorage>>>,
}

impl World {
    pub fn new() -> World {
        World {
            entity_flags : vec![None],
            components: Default::default()
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
        if let Some(last_id) = self.entity_flags.first().unwrap() {
            let last_id = *last_id;
            //we got an id can be reused
            let new_id = self.entity_flags[last_id.get()];
            self.entity_flags[last_id.get()] = None;
            self.entity_flags[0] = new_id;
            EntityRef::new(self,last_id)
        }else{
            //full
            let id = self.entity_flags.len();
            self.entity_flags.push(None);
            //safe here because this id can't be 0
            EntityRef::new(self, unsafe { EntityId::new_unchecked(id) })
        }
    }

    pub fn remove_entity(&mut self,entity_id : EntityId){
        let _entity_id = entity_id.get();
        assert!(self.exist(entity_id),"Entity is not existence");
        self.entity_flags[_entity_id] = self.entity_flags[0];
        self.entity_flags[0] = Some(entity_id);
        //remove all components of this entity
        for (_,storage) in &mut self.components{
            let mut storage = storage.borrow_mut();
            storage.remove(entity_id);
        }
    }

    pub fn attach_component<T : Component>(&mut self,entity_id : EntityId,component : T){
        debug_assert!(self.components.contains_key(&TypeId::of::<T>()),
            "Component has not been registered.");
        let mut components = self.components
            .get(&TypeId::of::<T>())
            .unwrap()
            .borrow_mut();
        unsafe {
            components.downcast_mut::<SparseSet<EntityId,T>>()
        }.add(entity_id,component);
    }

    pub fn detach_component<T : Component>(&mut self,entity_id : EntityId) -> Option<T>{
        debug_assert!(self.components.contains_key(&TypeId::of::<T>()),
                      "Component has not been registered.");
        let mut components = self.components
            .get(&TypeId::of::<T>())
            .unwrap()
            .borrow_mut();
        unsafe {
            components.downcast_mut::<SparseSet<EntityId,T>>()
        }.remove(entity_id)
    }

    pub fn components_ref<T : Component>(&self) -> Ref<'_,[T]>{
        debug_assert!(self.components.contains_key(&TypeId::of::<T>()),
                      "Component has not been registered.");
        let components = self.components
            .get(&TypeId::of::<T>())
            .unwrap()
            .borrow();
        Ref::map(components,|raw|{
            unsafe{
                raw.downcast_ref::<SparseSet<EntityId,T>>()
            }.data()
        })
    }

    pub fn components_mut<T : Component>(&mut self) -> RefMut<'_,[T]>{
        debug_assert!(self.components.contains_key(&TypeId::of::<T>()),
                      "Component has not been registered.");
        let components = self.components
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .borrow_mut();
        RefMut::map(components,|raw|{
            unsafe{
                raw.downcast_mut::<SparseSet<EntityId,T>>()
            }.data_mut()
        })
    }

    pub fn exist(&mut self,entity_id : EntityId) -> bool {
        let entity_id = entity_id.get();
        entity_id < self.entity_flags.len() && self.entity_flags[entity_id].is_none()
    }

    pub fn entity(&mut self,entity_id : EntityId) -> Option<EntityRef<'_>> {
        if self.exist(entity_id) {
            Some(EntityRef::new(self, entity_id))
        }else{
            None
        }
    }
}

#[cfg(test)]
mod tests{
    use crate::world::World;
    use crate::{Component, EntityId};

    #[test]
    fn flags_test(){
        let mut world = World::new();
        world.create_entity(); // 1
        world.create_entity(); // 2
        world.create_entity(); // 3
        world.create_entity(); // 4
        world.create_entity(); // 5
        assert_eq!(world.entity_flags.as_slice(),
                   &[None,None,None,None,None,None]);
        world.remove_entity(EntityId::new(3).unwrap());
        assert_eq!(world.entity_flags.as_slice(),
                   &[EntityId::new(3),None,None,None,None,None]);
        world.remove_entity(EntityId::new(5).unwrap());
        assert_eq!(world.entity_flags.as_slice(),
                   &[EntityId::new(5),None,None,None,None,EntityId::new(3)]);
        world.remove_entity(EntityId::new(1).unwrap());
        assert_eq!(world.entity_flags.as_slice(),
                   &[EntityId::new(1),EntityId::new(5),None,None,None,EntityId::new(3)]);
        assert_eq!(world.create_entity().into_id(),EntityId::new(1).unwrap());
        assert_eq!(world.entity_flags.as_slice(),
                   &[EntityId::new(5),None,None,None,None,EntityId::new(3)]);
        assert_eq!(world.create_entity().into_id(),EntityId::new(5).unwrap());
        assert_eq!(world.entity_flags.as_slice(),
                   &[EntityId::new(3),None,None,None,None,None]);
        assert_eq!(world.create_entity().into_id(),EntityId::new(3).unwrap());
        assert_eq!(world.entity_flags.as_slice(),
                   &[None,None,None,None,None,None]);
        assert_eq!(world.create_entity().into_id(),EntityId::new(6).unwrap());
        assert_eq!(world.entity_flags.as_slice(),
                   &[None,None,None,None,None,None,None]);

    }

    #[test]
    fn component_test(){
        let mut world = World::new();
        world.register::<char>();
        let id1 = world.create_entity().into_id();
        let id2 = world.create_entity().into_id();
        let id3 = world.create_entity().into_id();

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
        // let mut world = World::new();
        //
        // world
        //     .register::<u32>()
        //     .register::<char>();
        //
        // world.create_entity(1u32);
        // world.create_entity(2u32);
        // world.create_entity(3u32)
        //     .with('a');
        // world.create_entity(4u32)
        //     .with('b');
        // world.create_entity(5u32)
        //     .with('c');
        // world.create_entity(6u32);
        // world.create_entity('d');
        // println!("u32 :{:?}",world.components::<u32>().unwrap().entities());
        // println!("char:{:?}",world.components::<char>().unwrap().entities());
        //
        // println!();
        //
        // world.make_group::<u32,char>();
        // println!("u32 :{:?}",world.components::<u32>().unwrap().entities());
        // println!("char:{:?}",world.components::<char>().unwrap().entities());
        //
        // let mut iter = world.group_filter_iter::<char>();
        // if let Some(group) = iter.next() {
        //     let group = group.borrow();
        //     println!("Group len:{}",group.range.len());
        // }else{
        //     panic!("Cannot find any group has <{}>",std::any::type_name::<char>())
        // }
    }
}