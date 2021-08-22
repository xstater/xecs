//! Stage struct
use crate::World;
use crate::system::{System, Run, Dependencies};
use std::collections::HashMap;
use std::any::{TypeId};
use std::cell::{RefCell, Ref, RefMut};
use std::fmt::{Debug, Formatter};

struct SystemInfo {
    dependencies : Vec<TypeId>,
    is_active : bool,
    is_once : bool,
    system : RefCell<Box<dyn Run>>
}

/// Stage = World + Systems
pub struct Stage{
    world : RefCell<World>,
    systems : HashMap<TypeId,SystemInfo>,
    need_update : bool,
    run_sequence : Vec<TypeId>
}

impl Debug for Stage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f
            .debug_struct("Stage")
            .field("world",&self.world)
            .field("systems id",&self.run_sequence)
            .finish()
    }
}

impl Stage {
    /// Create a stage with a empty world.
    pub fn new() -> Stage {
        Stage {
            world: RefCell::new(World::new()),
            systems: HashMap::new(),
            need_update : false,
            run_sequence : vec![]
        }
    }

    /// Create a stage with determined world.
    pub fn from_world(world : World) -> Stage {
        Stage {
            world : RefCell::new(world),
            systems : HashMap::new(),
            need_update : false,
            run_sequence : vec![]
        }
    }
    /// Add a normal system in stage.
    pub fn add_system<T : for<'a> System<'a>>(&mut self,system : T){
        self.need_update = true;
        self.systems.insert(
            TypeId::of::<T>(),
            SystemInfo {
                dependencies: <<T as System>::Dependencies as Dependencies>::dependencies(),
                is_active: true,
                is_once : false,
                system : RefCell::new(Box::new(system))
            }
        );
    }

    /// Add a system that run only once in stage.
    pub fn add_once_system<T : for<'a> System<'a>>(&mut self,system : T){
        self.need_update = true;
        self.systems.insert(
            TypeId::of::<T>(),
            SystemInfo {
                dependencies: <<T as System>::Dependencies as Dependencies>::dependencies(),
                is_active: true,
                is_once : true,
                system: RefCell::new(Box::new(system))
            }
        );
    }

    /// Check if stage has such system.
    pub(in crate) fn has_system<T : for<'a> System<'a>>(&self) -> bool {
        self.systems.contains_key(&TypeId::of::<T>())
    }

    /// Deactivate a system.
    /// ### Detail
    /// * A deactivated system will not be executed in stage run.
    /// * The depended systems also will not be executed too.
    pub fn deactivate<T : for<'a> System<'a>>(&mut self) {
        debug_assert!(self.has_system::<T>(),
                      "There is no such system in stage");
        self.systems
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .is_active = false;
    }

    /// Activate a system.
    /// ### Detail
    /// The system is activated by default.
    pub fn activate<T : for<'a> System<'a>>(&mut self) {
        debug_assert!(self.has_system::<T>(),
                      "There is no such system in stage");
        self.systems
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .is_active = true;
    }

    /// Get a reference of System data.
    pub fn system_data_ref<T : for<'a> System<'a>>(&self) -> Ref<'_,T> {
        debug_assert!(self.has_system::<T>(),
                    "There is no such system in stage");
        let any = &self.systems
            .get(&TypeId::of::<T>())
            .unwrap()
            .system;
        let any = any.borrow();
        Ref::map(any,|any| unsafe {
            any.downcast_ref::<T>()
        })
    }


    /// Get a mutable reference of System data.
    pub fn system_data_mut<T : for<'a> System<'a>>(&self) -> RefMut<'_,T> {
        debug_assert!(self.has_system::<T>(),
                      "There is no such system in stage");
        let any = &self.systems
            .get(&TypeId::of::<T>())
            .unwrap()
            .system;
        let any = any.borrow_mut();
        RefMut::map(any,|any| unsafe {
            any.downcast_mut::<T>()
        })
    }

    /// Get a reference of world in stage.
    pub fn world_ref(&self) -> Ref<'_,World> {
        self.world.borrow()
    }

    /// Get a mutable reference of world in stage.
    pub fn world_mut(&self) -> RefMut<'_,World> {
        self.world.borrow_mut()
    }

    /// Execute all systems in stage.
    /// ### Details
    /// * Once Systems will be removed after ran.
    /// * System will be ran with topological order
    pub fn run(&mut self) {
        //topological sort
        if self.need_update {
            self.run_sequence.clear();
            let mut deps = HashMap::new();
            for (type_id,_) in &self.systems {
                deps.insert(*type_id, (0_usize, vec![]));
            }
            for (type_id,system_info) in &self.systems {
                {
                    let (enter_count, _) = deps.get_mut(&type_id).unwrap();
                    *enter_count = system_info.dependencies.len();
                }
                for dep in &system_info.dependencies {
                    if let Some((_,leave_edges)) = deps.get_mut(dep) {
                        leave_edges.push(*type_id);
                    } else {
                        panic!("No such depend system");
                    }
                }
            }
            fn get_zero(deps : &HashMap<TypeId,(usize,Vec<TypeId>)>) -> Option<TypeId> {
                for (tid,(enter_count,_)) in deps {
                    if *enter_count == 0 {
                        return Some(*tid);
                    }
                }
                None
            }
            while let Some(type_id) = get_zero(&deps) {
                self.run_sequence.push(type_id);
                let (_,leave_edges) = deps.remove(&type_id).unwrap();
                for edge in leave_edges {
                    let (enter_count,_) = deps.get_mut(&edge).unwrap();
                    *enter_count -= 1;
                }
            }
        }
        self.need_update = false;

        let mut remove_list = vec![];
        for type_id in &self.run_sequence {
            let system_info = self.systems.get(type_id).unwrap();
            if system_info.is_active {
                let mut system = system_info.system.borrow_mut();
                system.run(&self);
                if system_info.is_once {
                    remove_list.push(*type_id);
                }
            }
        }
        if !remove_list.is_empty() {
            self.need_update = true;
            for type_id in remove_list {
                self.systems.remove(&type_id);
            }
        }
    }

}

#[cfg(test)]
mod tests{
    use crate::World;
    use crate::stage::Stage;
    use crate::system::{System};
    use crate::resource::Resource;

    #[test]
    fn test_run() {
        let mut world = World::new();

        world.register::<char>();

        world.create_entity().attach('c');
        world.create_entity().attach('a');
        world.create_entity().attach('f');

        let mut stage = Stage::from_world(world);

        struct StartSystem;
        struct PrintSystem;
        #[derive(Debug)]
        struct DataSystemName(String);
        #[derive(Debug)]
        struct DataSystemAge(u32);
        struct AfterAll;

        impl<'a> System<'a> for StartSystem {
            type Resource = ();
            type Dependencies = ();

            fn update(&'a mut self, _resource: <Self::Resource as Resource<'a>>::Type) {
                println!("Start");
            }
        }

        impl<'a> System<'a> for PrintSystem {
            type Resource = (&'a World,&'a DataSystemName,&'a mut DataSystemAge);
            type Dependencies = StartSystem;

            fn update(&mut self, (world,name,age) : <Self::Resource as Resource<'a>>::Type) {
                let v = world.query::<&char>().cloned().collect::<Vec<_>>();
                dbg!(&v);
                dbg!(&name.0);
                dbg!(&age.0);
            }
        }

        impl<'a> System<'a> for DataSystemName{
            type Resource = ();
            type Dependencies = StartSystem;
        }
        impl<'a> System<'a> for DataSystemAge {
            type Resource = ();
            type Dependencies = StartSystem;
        }

        impl<'a> System<'a> for AfterAll {
            type Resource = ();
            type Dependencies = (PrintSystem,DataSystemName,DataSystemAge);

            fn update(&'a mut self, _resource: <Self::Resource as Resource<'a>>::Type) {
                println!("Finished");
            }
        }

        stage.add_system(StartSystem);
        stage.add_system(PrintSystem);
        stage.add_system(DataSystemName("asda".to_string()));
        stage.add_system(DataSystemAge(13));
        stage.add_once_system(AfterAll);

        stage.run();

        stage.run();

        stage.inactive::<PrintSystem>();

        stage.run();
    }
}