extern crate xsparseset;

use std::any::{TypeId, Any};
use crate::Component;
use crate::EntityId;
use crate::entity::Entity;
use crate::query::{Query};
use xsparseset::SparseSet;
use std::ops::Range;
use std::collections::{HashMap, HashSet};
use std::cell::{RefCell, Ref, RefMut};

#[derive(Debug)]
pub(in crate) struct Group{
    pub(in crate) types : HashSet<TypeId>,
    pub(in crate) range : Range<usize>,
    pub(in crate) need_update : bool
}

pub struct World {
    pub(in crate) destroyed_count : usize,
    pub(in crate) destroyed_id : Option<EntityId>,
    pub(in crate) components_count: Vec<usize>,
    pub(in crate) components: HashMap<TypeId,RefCell<Box<dyn Any>>>,//Box<SparseSet<EntityId,Component>>
    pub(in crate) groups : Vec<RefCell<Group>>
}

impl World {
    pub fn new() -> World {
        World {
            destroyed_count : 0,
            destroyed_id : None,
            components_count: Vec::new(),
            components: HashMap::new(),
            groups : Vec::new()
        }
    }

    pub fn create_entity<T : Component>(&mut self,component : T) -> Entity<'_> {
        let id = if let Some(id) = self.destroyed_id {
            self.destroyed_count -= 1;
            if self.destroyed_count == 0 {
                self.destroyed_id = None;
            }else{
                self.destroyed_id = Some(self.components_count[id]);
            }
            self.components_count[id] = 0;
            id
        }else {
            self.components_count.push(0);
            self.components_count.len() - 1
        };
        Entity::new(self, id)
            .with(component)
    }

    pub fn register<T : Component>(&mut self) -> &mut Self{
        //registered
        if self.components.contains_key(&TypeId::of::<T>()) {
            panic!("Cannot register component {} as twice",std::any::type_name::<T>());
        }else{
            self.components.insert(
                TypeId::of::<T>(),
                RefCell::new(Box::new(SparseSet::<EntityId, T>::new())));
        }
        self
    }

    pub fn make_group<A : Component,B : Component>(&mut self){
        let tid_a = TypeId::of::<A>();
        let tid_b = TypeId::of::<B>();
        //panic if A or B already in group
        for group in &self.groups {
            let group = group.borrow();
            if group.types.contains(&tid_a) {
                panic!("Component {} is already in group!",std::any::type_name::<A>())
            }
            if group.types.contains(&tid_b) {
                panic!("Component {} is already in group!",std::any::type_name::<B>())
            }
        }
        let mut group = Group{
            types: {
                let mut set = HashSet::new();
                set.insert(tid_a);
                set.insert(tid_b);
                set
            },
            range: (0..0),
            need_update: false
        };
        // re-order by the shorter one
        let len = {
            let mut component_a = self.components_mut::<A>().unwrap();
            let mut component_b = self.components_mut::<B>().unwrap();
            component_a.make_group_in(&mut component_b,0)
        };
        group.range.end = len;
        self.groups.push(RefCell::new(group));
    }

    pub fn remove_group<A : Component,B : Component>(&mut self){
        let mut index = None;
        for (i,group) in self.groups.iter().enumerate() {
            let group = group.borrow();
            if group.types.len() == 2
            && group.types.contains(&TypeId::of::<A>())
            && group.types.contains(&TypeId::of::<B>()) {
                index = Some(i);
                break;
            }
        };
        if let Some(index) = index {
            self.groups.remove(index);
        }else{
            panic!("Group<{},{}> does not exist",std::any::type_name::<A>(),std::any::type_name::<B>());
        }
    }

    pub(in crate) fn group_filter_iter<T : Component>(&self) -> impl Iterator<Item = &RefCell<Group>>{
        self.groups
            .iter()
            .filter(|group|{
                let group = (*group).borrow();
                group.types.contains(&TypeId::of::<T>())
            })
    }

    pub fn add_component<T : Component>(&mut self, entity_id : EntityId, component : T){
        for group in self.group_filter_iter::<T>() {
            let mut group = group.borrow_mut();
            group.need_update = true;
        }
        if let Some(ptr) = self.components.get_mut(&TypeId::of::<T>()) {
            let mut ptr = ptr.borrow_mut();
            let manager = ptr.downcast_mut::<SparseSet<EntityId,T>>().unwrap();
            if !manager.exist(entity_id) {
                self.components_count[entity_id] += 1;
            }
            manager.add(entity_id,component);
            return;
        }
        panic!("Component {} have not been registered !",std::any::type_name::<T>());
    }

    pub fn remove_component<T : Component>(&mut self, entity_id : EntityId) -> Option<T> {
        for group in self.group_filter_iter::<T>() {
            let mut group = group.borrow_mut();
            group.need_update = true;
        }
        if let Some(ptr) = self.components.get_mut(&TypeId::of::<T>()) {
            let mut ptr = ptr.borrow_mut();
            let manager = ptr.downcast_mut::<SparseSet<EntityId,T>>().unwrap();
            if manager.exist(entity_id) {
                self.components_count[entity_id] -= 1;
                if self.components_count[entity_id] == 0 {
                    self.destroyed_count += 1;
                    if let Some(prev_id) = self.destroyed_id {
                        self.components_count[entity_id] = prev_id;
                    }
                    self.destroyed_id = Some(entity_id);
                }
            }
            return manager.remove(entity_id);
        }
        panic!("Type <{}> have not been registered !",std::any::type_name::<T>());
    }

    pub(in crate) fn components<T : Component>(&self) -> Option<Ref<'_,SparseSet<EntityId,T>>> {
        let ref_box = self.components
            .get(&TypeId::of::<T>())?
            .borrow();
        Some(Ref::map(ref_box,|component|{
            component.downcast_ref::<SparseSet<EntityId,T>>().unwrap()
        }))
    }
    pub(in crate) fn components_mut<T : Component>(&self) -> Option<RefMut<'_,SparseSet<EntityId,T>>> {
        let ref_box = self.components
            .get(&TypeId::of::<T>())?
            .borrow_mut();
        Some(RefMut::map(ref_box,|component|{
            component.downcast_mut::<SparseSet<EntityId,T>>().unwrap()
        }))
    }

    pub(in crate) fn group<A : Component,B : Component>(&self) -> Option<Ref<'_,Group>> {
        self.groups.iter().find(|group|{
            let group = (*group).borrow();
            group.types.len() == 2 &&
            group.types.contains(&TypeId::of::<A>()) &&
            group.types.contains(&TypeId::of::<B>())
        }).map(|group|{
            group.borrow()
        })
    }

    pub(in crate) fn group_mut<A : Component,B : Component>(&self) -> Option<RefMut<'_,Group>> {
        self.groups.iter().find(|group|{
            let group = (*group).borrow();
            group.types.len() == 2 &&
                group.types.contains(&TypeId::of::<A>()) &&
                group.types.contains(&TypeId::of::<B>())
        }).map(|group|{
            group.borrow_mut()
        })
    }
    pub fn entities_count(&self) -> usize {
        self.components_count.len() - self.destroyed_count
    }

    pub fn make_query<T : Component>(&mut self) -> Query<'_,T>{
        Query{
            world: self,
            _marker: Default::default()
        }
    }

}

/*
p: 0 1 2 3 4 5 6 7
m:         x x
A: 4 7 2 1 3 6
B: 4 7 8 5 3
g:-----^
updated ent:3 6
updated pos:4 5
*/
#[cfg(test)]
mod tests{
    use crate::world::World;
    use crate::{Component};

    #[test]
    fn test(){
        #[derive(Debug)]
        struct Fuck(i32);
        #[derive(Debug)]
        struct Shit(char);

        let mut world = World::new();
        world
            .register::<Fuck>()
            .register::<Shit>();

        world.create_entity(Shit('a'));
        world.create_entity(Fuck(2))
            .with(Shit('b'))
            .with(Shit('c'));
        for c in 'a'..='z' {
            world.create_entity(Shit(c));
        }
        world.remove_component::<Shit>(0);
        world.remove_component::<Shit>(3);
        world.remove_component::<Shit>(5);


        println!("destroyed:{},id:{:?},counts:{:?}",world.destroyed_count,world.destroyed_id,world.components_count);
        world.create_entity(Fuck(3));
        println!("destroyed:{},id:{:?},counts:{:?}",world.destroyed_count,world.destroyed_id,world.components_count);
        world.create_entity(Fuck(2));
        println!("destroyed:{},id:{:?},counts:{:?}",world.destroyed_count,world.destroyed_id,world.components_count);
        world.create_entity(Fuck(5));
        println!("destroyed:{},id:{:?},counts:{:?}",world.destroyed_count,world.destroyed_id,world.components_count);
        world.create_entity(Fuck(7));
        println!("destroyed:{},id:{:?},counts:{:?}",world.destroyed_count,world.destroyed_id,world.components_count);
        println!("Shit:{:?}",world.components::<Shit>().unwrap().data());
        println!("Fuck:{:?}",world.components::<Fuck>().unwrap().data());

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

        world
            .register::<u32>()
            .register::<char>();

        world.create_entity(1u32);
        world.create_entity(2u32);
        world.create_entity(3u32)
            .with('a');
        world.create_entity(4u32)
            .with('b');
        world.create_entity(5u32)
            .with('c');
        world.create_entity(6u32);
        world.create_entity('d');
        println!("u32 :{:?}",world.components::<u32>().unwrap().entities());
        println!("char:{:?}",world.components::<char>().unwrap().entities());

        println!();

        world.make_group::<u32,char>();
        println!("u32 :{:?}",world.components::<u32>().unwrap().entities());
        println!("char:{:?}",world.components::<char>().unwrap().entities());

        let mut iter = world.group_filter_iter::<char>();
        if let Some(group) = iter.next() {
            let group = group.borrow();
            println!("Group len:{}",group.range.len());
        }else{
            panic!("Cannot find any group has <{}>",std::any::type_name::<char>())
        }
    }
}