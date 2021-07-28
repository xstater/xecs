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
    //     Unavailable(_)      -> there is no entityID for reuse
    //     Available(EntityID) -> the EntityID
    entity_flags : Vec<EntityFlag>,
    entities : Vec<EntityId>,
    // Box<SparseSet<EntityId,Component>>
    components: HashMap<TypeId,RefCell<Box<dyn ComponentStorage>>>,
    groups : Vec<Group>
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
        // todo: add support for group
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
        // todo: add support for group
    }

    fn components_storage_ref<T : Component>(&self) -> Ref<'_,SparseSet<EntityId,T>>{
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

    fn components_storage_mut<T : Component>(&self) -> RefMut<'_,SparseSet<EntityId,T>>{
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
        // todo: add support for group
    }

    pub fn detach_component<T : Component>(&mut self,entity_id : EntityId) -> Option<T>{
        self.components_storage_mut::<T>()
            .remove(entity_id)
        // todo: add support for group
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
                    if (owning_a && group.is_owned::<A>())
                    || (owning_b && group.is_owned::<B>()) {
                        ok = false;
                        break;
                    }
                }
                ok
            }
        ,"Component in group cannot be owned twice");
        if owning_a && owning_b {
            // full-owning group
            let mut group = FullOwningGroup {
                types: (TypeId::of::<A>(), TypeId::of::<B>()),
                length: 0
            };
            {
                let mut comp_a = self.components_storage_mut::<A>();
                let mut comp_b = self.components_storage_mut::<B>();
                let len_a = comp_a.len();
                let len_b = comp_b.len();
                if len_a < len_b {
                    for index_a in 0..len_a {
                        let entity_id = comp_a.entities()[index_a];
                        if let Some(index_b) = comp_b.get_index(entity_id) {
                            comp_a.swap_by_index(group.length, index_a);
                            comp_b.swap_by_index(group.length, index_b);
                            group.length += 1;
                        }
                    }
                } else {
                    for index_b in 0..len_b {
                        let entity_id = comp_b.entities()[index_b];
                        if let Some(index_a) = comp_a.get_index(entity_id) {
                            comp_a.swap_by_index(group.length,index_a);
                            comp_b.swap_by_index(group.length,index_b);
                            group.length += 1;
                        }
                    }
                }
            }
            self.groups.push(Group::Full(group));
            return;
        }

        if owning_a {
            //partial-owning group
            let mut group = PartialOwningGroup{
                owned_type: TypeId::of::<A>(),
                non_owned_type: TypeId::of::<B>(),
                length: 0
            };
            {
                //we owned A,so we just need to sort A
                let mut comp_a = self.components_storage_mut::<A>();
                let     comp_b = self.components_storage_ref::<B>();
                for index in 0..comp_a.len() {
                    let entity_id = comp_a.entities()[index];
                    if comp_b.exist(entity_id) {
                        comp_a.swap_by_index(group.length,index);
                        group.length += 1;
                    }
                }
            }
            self.groups.push(Group::Partial(group));
            return;
        }

        if owning_b {
            //partial-owning group
            let mut group = PartialOwningGroup{
                owned_type: TypeId::of::<B>(),
                non_owned_type: TypeId::of::<A>(),
                length: 0
            };
            {
                //we owned B,so we just need to sort B
                let     comp_a = self.components_storage_ref::<A>();
                let mut comp_b = self.components_storage_mut::<B>();
                for index in 0..comp_b.len() {
                    let entity_id = comp_b.entities()[index];
                    if comp_a.exist(entity_id) {
                        comp_b.swap_by_index(group.length,index);
                        group.length += 1;
                    }
                }
            }
            self.groups.push(Group::Partial(group));
            return;
        }

        //Non-Owning
        let mut group = NonOwningGroup{
            types: (TypeId::of::<A>(),TypeId::of::<B>()),
            entities: vec![],
            index_a: vec![],
            index_b: vec![]
        };
        {
            let comp_a = self.components_storage_ref::<A>();
            let comp_b = self.components_storage_ref::<B>();
            let len_a = comp_a.len();
            let len_b = comp_b.len();
            if len_a < len_b {
                for index_a in 0..len_a {
                    let entity_id = comp_a.entities()[index_a];
                    if let Some(index_b) = comp_b.get_index(entity_id){
                        group.entities.push(entity_id);
                        group.index_a.push(index_a);
                        group.index_b.push(index_b);
                    }
                }
            } else {
                for index_b in 0..len_b {
                    let entity_id = comp_b.entities()[index_b];
                    if let Some(index_a) = comp_a.get_index(entity_id) {
                        group.entities.push(entity_id);
                        group.index_a.push(index_a);
                        group.index_b.push(index_b)
                    }
                }
            }
        }
        self.groups.push(Group::Non(group));
    }
}

#[derive(Debug,Copy,Clone)]
pub enum EntityFlag{
    /// store the next available EntityID
    Available(EntityId),
    /// store the index of EntityID in entities array
    Unavailable(usize)
}

#[derive(Debug)]
pub struct FullOwningGroup{
    types : (TypeId,TypeId),
    length : usize
}

impl FullOwningGroup{
    fn is_owned<T : Component>(&self) -> bool {
        let typeid = TypeId::of::<T>();
        typeid == self.types.0 || typeid == self.types.1
    }
}

#[derive(Debug)]
pub struct PartialOwningGroup {
    owned_type : TypeId,
    non_owned_type : TypeId,
    length : usize
}

impl PartialOwningGroup {
    fn is_owned<T : Component>(&self) -> bool {
        let typeid = TypeId::of::<T>();
        typeid == self.owned_type
    }
}

#[derive(Debug)]
pub struct NonOwningGroup {
    types : (TypeId,TypeId),
    entities : Vec<EntityId>,
    index_a : Vec<usize>,
    index_b : Vec<usize>
}

impl NonOwningGroup {
    fn is_owned<T : Component>(&self) -> bool {
        false
    }
}

#[derive(Debug)]
pub enum Group {
    Full(FullOwningGroup),
    Partial(PartialOwningGroup),
    Non(NonOwningGroup)
}

impl Group {
    fn is_owned<T : Component>(&self) -> bool {
        match &self {
            Group::Full(group) => group.is_owned::<T>(),
            Group::Partial(group) => group.is_owned::<T>(),
            Group::Non(group) => group.is_owned::<T>()
        }
    }
}

#[cfg(test)]
mod tests{
    use crate::world::{World, FullOwningGroup, PartialOwningGroup, NonOwningGroup, Group, EntityFlag};
    use crate::{ EntityId};

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
        let mut world = World::new();

        world.register::<u32>();
        world.register::<char>();
        world.register::<()>();

        world.create_entity().attach(1u32).attach(());
        world.create_entity().attach(2u32);
        world.create_entity().attach(3u32).attach('a').attach(());
        world.create_entity().attach(4u32).attach('b');
        world.create_entity().attach(5u32).attach('c');
        world.create_entity().attach(6u32);
        world.create_entity().attach('d').attach(());
        println!("u32 :{:?}",world.entities_in::<u32>());
        println!("char:{:?}",world.entities_in::<char>());
        println!("()  :{:?}",world.entities_in::<()>());

        println!();

        world.make_group::<u32,char>(true,true);
        world.make_group::<u32,char>(false,false);
        world.make_group::<u32,()>(false,true);
        println!("u32 :{:?}",world.entities_in::<u32>());
        println!("char:{:?}",world.entities_in::<char>());
        println!("()  :{:?}",world.entities_in::<()>());

        for group in &world.groups {
            println!("{:?}",group);
        }
    }

    #[test]
    fn size_test(){
        println!("Size of bool:{}Bytes",std::mem::size_of::<bool>());
        println!("Size of u64:{}Bytes",std::mem::size_of::<u64>());
        println!("Size of EntityFlag:{}Bytes",std::mem::size_of::<EntityFlag>());
        println!("Size of Full-Owning Group:{}Bytes",std::mem::size_of::<FullOwningGroup>());
        println!("Size of Partial-Owning Group:{}Bytes",std::mem::size_of::<PartialOwningGroup>());
        println!("Size of Non-Owning Group:{}Bytes",std::mem::size_of::<NonOwningGroup>());
        println!("Size of Group:{}Bytes",std::mem::size_of::<Group>());
    }
}