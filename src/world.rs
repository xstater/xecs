use std::num::NonZeroU32;
use std::ffi::c_void;
use std::any::TypeId;
use std::alloc::Layout;
use crate::component::{self,Component};
use crate::EntityId;
use crate::entity::Entity;

#[derive(Debug)]
pub struct World {
    next_id : EntityId,
    component_manager : Vec<(TypeId,Layout,*mut u8)>,
}

impl World {
    pub fn new() -> World {
        World {
            next_id : 0,
            component_manager: vec![]
        }
    }

    pub fn create_entity(&mut self) -> Entity<'_> {
        self.next_id += 1;
        Entity::new(self,self.next_id - 1)
    }

    pub fn register<T : Component>(&mut self) -> &mut Self{
        let layout = Layout::new::<component::Manager<T>>();
        let tid = TypeId::of::<T>();
        let ptr = unsafe { std::alloc::alloc(layout) };
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        unsafe {
            *(ptr as *mut component::Manager<T>) = component::Manager::<T>::new();
        };
        self.component_manager.push((tid,layout,ptr));
        self
    }

    pub fn add_component_for_entity<T : Component>(&mut self, entity_id : EntityId,component : T){
        for (type_id,_,ptr) in &self.component_manager {
            if *type_id == TypeId::of::<T>(){
                let ptr = *ptr as *mut component::Manager<T>;
                let manager = unsafe { &mut *ptr };
                manager.new_component(entity_id,component);
                return;
            }
        }
        panic!("Type <{}> have not been registered !",std::any::type_name::<T>());
    }

    pub fn components<T : Component>(&self) -> &[T] {
        for (type_id,_,ptr) in &self.component_manager {
            if *type_id == TypeId::of::<T>(){
                let ptr = *ptr as *mut component::Manager<T>;
                let manager = unsafe { &*ptr };
                return manager.components()
            }
        }
        panic!("Type <{}> have not been registered !",std::any::type_name::<T>());
    }

}

impl Drop for World {
    fn drop(&mut self) {
        for (_,layout,ptr) in self.component_manager.iter() {
            unsafe { std::alloc::dealloc(*ptr,*layout) };
        }
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

        println!("here");

        let mut world = World::new();
        println!("{:?}",world);
        world
            .register::<Fuck>()
            .register::<Shit>()
        ;
        println!("{:?}",world);

        // world.create_entity()
        //     .with(Shit('a'));
        // world.create_entity()
        //     .with(Fuck(2))
        //     .with(Shit('c'));
        // for c in 'a'..='z' {
        //     world.create_entity()
        //         .with(Shit(c));
        // }
        //
        // println!("{:?}\n{:?}",world.components::<Shit>(),world.components::<Fuck>());

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