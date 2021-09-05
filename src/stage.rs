//! Stage struct
use crate::World;
use crate::system::{System, Run, Dependencies, End};
use std::collections::HashMap;
use std::any::{TypeId};
use std::cell::{RefCell, Ref, RefMut};
use std::fmt::{Debug, Formatter};
use std::option::Option::Some;

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
    run_queue : Vec<TypeId>,
    need_init : Vec<TypeId>
}

impl Debug for Stage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f
            .debug_struct("Stage")
            .field("world",&self.world)
            .field("systems id",&self.run_queue)
            .finish()
    }
}

impl Stage {
    /// Create a stage with a empty world.
    pub fn new() -> Stage {
        Stage {
            world: RefCell::new(World::new()),
            systems: HashMap::new(),
            need_update: false,
            run_queue: vec![],
            need_init: vec![]
        }
    }

    /// Create a stage with determined world.
    pub fn from_world(world : World) -> Stage {
        Stage {
            world : RefCell::new(world),
            systems : HashMap::new(),
            need_update: false,
            run_queue: vec![],
            need_init: vec![]
        }
    }
    /// Add a normal system in stage.
    pub fn add_system<T : for<'a> System<'a>>(&mut self,system : T) -> &mut Self{
        self.need_update = true;
        self.need_init.push(TypeId::of::<T>());
        self.systems.insert(
            TypeId::of::<T>(),
            SystemInfo {
                dependencies: <<T as System>::Dependencies as Dependencies>::dependencies(),
                is_active: true,
                is_once : false,
                system : RefCell::new(Box::new(system))
            }
        );
        self
    }

    /// Add a system that run only once in stage.
    #[deprecated = "Use System::init() !"]
    pub fn add_once_system<T : for<'a> System<'a>>(&mut self,system : T) -> &mut Self{
        self.need_update = true;
        self.need_init.push(TypeId::of::<T>());
        self.systems.insert(
            TypeId::of::<T>(),
            SystemInfo {
                dependencies: <<T as System>::Dependencies as Dependencies>::dependencies(),
                is_active: true,
                is_once : true,
                system: RefCell::new(Box::new(system))
            }
        );
        self
    }

    /// Check if stage has such system.
    pub(in crate) fn has_system<T : for<'a> System<'a>>(&self) -> bool {
        self.has_system_dyn(TypeId::of::<T>())
    }


    /// Check if stage has such system from a dynamic TypeId
    pub(in crate) fn has_system_dyn(&self,system : TypeId) -> bool {
        self.systems.contains_key(&system)
    }

    /// Deactivate a system.
    /// ### Detail
    /// * A deactivated system will not be executed in stage run.
    /// * The depended systems also will not be executed too.
    pub fn deactivate<T : for<'a> System<'a>>(&mut self) -> &mut Self{
        self.deactivate_dyn(TypeId::of::<T>())
    }

    /// Same as ```deactivate```
    pub fn deactivate_dyn(&mut self,system : TypeId) -> &mut Self{
        debug_assert!(self.has_system_dyn(system),
                      "There is no such system in stage");
        self.systems
            .get_mut(&system)
            .unwrap()
            .is_active = false;
        self
    }

    /// Activate a system.
    /// ### Detail
    /// The system is activated by default.
    pub fn activate<T : for<'a> System<'a>>(&mut self) -> &mut Self {
        self.activate_dyn(TypeId::of::<T>())
    }

    /// Same as ```activate```
    pub fn activate_dyn(&mut self,system : TypeId) -> &mut Self {
        debug_assert!(self.has_system_dyn(system),
                      "There is no such system in stage");
        self.systems
            .get_mut(&system)
            .unwrap()
            .is_active = true;
        self
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
        self.update();
        // initialize all systems
        for system_type in self.need_init.iter().cloned() {
            self.systems
                .get(&system_type)
                .unwrap()
                .system
                .borrow_mut()
                .initialize(self);
        }
        self.need_init.clear();
        let mut remove_list = vec![];
        for type_id in &self.run_queue {
            let system = self.systems
                .get(type_id)
                .unwrap();
            if system.is_active {
                if system.is_once {
                    remove_list.push(*type_id);
                }
                system.system.borrow_mut().run(self);
            }
        }
        for type_id in remove_list {
            self.systems.remove(&type_id);
        }
    }

    fn update(&mut self) {
        if !self.need_update {
            return;
        }
        self.run_queue.clear();
        let mut inverse_map = HashMap::new();
        let mut enter_edges_count = HashMap::new();
        // initialization
        for (type_id,system_info) in &self.systems {
            inverse_map.insert(*type_id,vec![]);
            enter_edges_count.insert(*type_id,system_info.dependencies.len());
        }
        inverse_map.insert(TypeId::of::<End>(),vec![]);
        // build inverse map
        for (self_type,self_system_info) in &self.systems {
            for dep_sys in &self_system_info.dependencies {
                inverse_map.get_mut(dep_sys)
                    .unwrap()
                    .push(*self_type)
            }
        }
        // topological sort
        fn find_zero(map : &HashMap<TypeId,usize>) -> Option<TypeId> {
            for (type_id,count) in map {
                // ignore the End
                if *type_id == TypeId::of::<End>() {
                    continue
                }
                if *count == 0 {
                    return Some(*type_id);
                }
            }
            None
        }
        fn sort(inverse_map : &HashMap<TypeId,Vec<TypeId>>,
                enter_edges_count : &mut HashMap<TypeId,usize>,
                run_queue : &mut Vec<TypeId>) {
            while let Some(type_id) = find_zero(enter_edges_count) {
                enter_edges_count.remove(&type_id);
                run_queue.push(type_id);
                for system in inverse_map.get(&type_id).unwrap().iter() {
                    let count = enter_edges_count.get_mut(system).unwrap();
                    *count -= 1;
                }
            }
        }
        sort(&inverse_map,&mut enter_edges_count,&mut self.run_queue);
        // sort remain systems
        if let Some(systems) = inverse_map.get(&TypeId::of::<End>()) {
            for system in systems.iter() {
                let count = enter_edges_count.get_mut(system).unwrap();
                *count -= 1;
            }
            sort(&inverse_map, &mut enter_edges_count, &mut self.run_queue);
        }
    }

}

#[cfg(test)]
mod tests{
    use crate::World;
    use crate::stage::Stage;
    use crate::system::{System, End};
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
        struct LastOfEnd;

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

            fn init(&'a mut self, _ : ()) {
                println!("DataSystemName has been added to stage");
            }
        }
        impl<'a> System<'a> for DataSystemAge {
            type Resource = ();
            type Dependencies = StartSystem;

            fn init(&'a mut self, _ : ()) {
                println!("DataSystemAge has been added to stage");
            }
        }

        impl<'a> System<'a> for AfterAll {
            type Resource = ();
            type Dependencies = End;

            fn update(&'a mut self, _resource: <Self::Resource as Resource<'a>>::Type) {
                println!("Finished");
            }
        }
        impl<'a> System<'a> for LastOfEnd {
            type Resource = ();
            type Dependencies = End;

            fn update(&'a mut self, _resource: <Self::Resource as Resource<'a>>::Type) {
                println!("Finished!!!");
            }
        }

        stage
            .add_system(StartSystem)
            .add_system(PrintSystem)
            .add_system(DataSystemName("asda".to_string()))
            .add_system(DataSystemAge(13))
            .add_system(AfterAll)
            .add_system(LastOfEnd);

        stage.run();

        stage.run();

        stage.deactivate::<PrintSystem>();

        stage.run();

        stage.activate::<PrintSystem>();
        stage.run();
    }
}