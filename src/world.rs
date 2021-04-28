use std::num::NonZeroU32;
use std::ffi::c_void;
use std::any::TypeId;
use std::alloc::Layout;
use crate::component::{self,Component};
use crate::EntityId;

pub struct World {
    entities : Vec<EntityId>,
    component_manager : Vec<(TypeId,Layout,*mut u8)>,
}

impl World {
    pub fn new() -> World {
        World {
            entities : vec![],
            component_manager: vec![]
        }
    }

    pub fn register<T : Component>(&mut self) -> &mut Self{
        let layout = Layout::new::<component::Manager<T>>();
        let tid = TypeId::of::<T>();
        let ptr = unsafe { std::alloc::alloc(layout) };
        unsafe {
            *(ptr as * mut component::Manager<T>) = component::Manager::<T>::new();
        };
        self.component_manager.push((tid,layout,ptr));
        self
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
        struct Fuck(i32);
        struct Shit(char);

        let mut world = World::new();
        world
            .register::<Fuck>()
            .register::<Shit>();

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