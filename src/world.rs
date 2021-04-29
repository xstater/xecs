use std::any::{TypeId, Any};
use crate::component::{self, Component};
use crate::EntityId;
use crate::entity::Entity;
use crate::query::Query;

pub struct World {
    destroyed_count : usize,
    destroyed_id : Option<EntityId>,
    components_count: Vec<usize>,
    components_managers: Vec<(TypeId, Box<dyn Any>)>,//Box<component::Manager<T>>
}

impl World {
    pub fn new() -> World {
        World {
            destroyed_count : 0,
            destroyed_id : None,
            components_count: Vec::new(),
            components_managers: Vec::new(),
        }
    }

    fn create_entity_without_component(&mut self) -> Entity<'_> {
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
    }

    // you cannot create a entity without a component!!!!
    pub fn create_entity<T : Component>(&mut self,component : T) -> Entity<'_> {
        self.create_entity_without_component()
            .with(component)
    }

    pub fn register<T : Component>(&mut self) -> &mut Self{
        //registered
        for (type_id,_) in &self.components_managers {
            if *type_id == TypeId::of::<component::Manager<T>>() {
                //do nothing
                return self;
            }
        }
        self.components_managers.push(
            (
                TypeId::of::<component::Manager<T>>(),
                Box::new(component::Manager::<T>::new())));
        self
    }

    pub fn add_component<T : Component>(&mut self, entity_id : EntityId, component : T){
        for (type_id,ptr) in &mut self.components_managers {
            if *type_id == TypeId::of::<component::Manager<T>>() {
                let manager = ptr.downcast_mut::<component::Manager<T>>().unwrap();
                if !manager.exists(entity_id) {
                    self.components_count[entity_id] += 1;
                }
                manager.new_component(entity_id,component);
                return;
            }
        }
        panic!("Type <{}> have not been registered !",std::any::type_name::<T>());
    }

    pub fn remove_component<T : Component>(&mut self, entity_id : EntityId) -> Option<T> {
        for (type_id,ptr) in &mut self.components_managers {
            if *type_id == TypeId::of::<component::Manager<T>>() {
                let manager = ptr.downcast_mut::<component::Manager<T>>().unwrap();
                if manager.exists(entity_id) {
                    self.components_count[entity_id] -= 1;
                    if self.components_count[entity_id] == 0 {
                        self.destroyed_count += 1;
                        if let Some(prev_id) = self.destroyed_id {
                            self.components_count[entity_id] = prev_id;
                        }
                        self.destroyed_id = Some(entity_id);
                    }
                }
                return manager.remove_component(entity_id);
            }
        }
        panic!("Type <{}> have not been registered !",std::any::type_name::<T>());
    }

    pub fn components<T : Component>(&self) -> &[T] {
        for (type_id,ptr) in &self.components_managers {
            if *type_id == TypeId::of::<component::Manager<T>>(){
                let manager = ptr.downcast_ref::<component::Manager<T>>().unwrap();
                return manager.components();
            }
        }
        panic!("Type <{}> have not been registered !",std::any::type_name::<T>());
    }


    pub fn components_mut<T : Component>(&mut self) -> &mut [T] {
        for (type_id,ptr) in &mut self.components_managers {
            if *type_id == TypeId::of::<component::Manager<T>>(){
                let manager = ptr.downcast_mut::<component::Manager<T>>().unwrap();
                return manager.components_mut();
            }
        }
        panic!("Type <{}> have not been registered !",std::any::type_name::<T>());
    }

    pub fn entities_count(&self) -> usize {
        self.components_count.len() - self.destroyed_count
    }

    pub fn make_query<T>(&mut self) -> Query<'_,T>{
        Query::from_world(self)
    }

}

#[cfg(test)]
mod tests{
    use crate::world::World;
    use crate::component::Component;

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
        println!("Shit:{:?}",world.components::<Shit>());
        println!("Fuck:{:?}",world.components::<Fuck>());

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
}