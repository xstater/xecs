//! # System trait and Dependencies trait
use crate::stage::Stage;
use std::any::{TypeId};
use crate::resource::Resource;

/// ## System trait
/// * System can has it owm data like ```struct Event(u32)```
/// * System can get other systems' data in Resource
pub trait System<'a> : 'static{
    /// Required data while system running.
    type Resource : Resource<'a>;
    /// The Dependencies of system
    type Dependencies : Dependencies;
    /// initialize the data
    #[allow(unused_variables)]
    fn init(&'a mut self,resource : <Self::Resource as Resource<'a>>::Type) {}
    /// update the states
    #[allow(unused_variables)]
    fn update(&'a mut self,resource : <Self::Resource as Resource<'a>>::Type){}
}

/// Something can be dependencies of systems
pub trait Dependencies {
    fn dependencies() -> Vec<TypeId>;
}
impl Dependencies for () {
    fn dependencies() -> Vec<TypeId> {
        vec![]
    }
}
impl<T : for<'a> System<'a>> Dependencies for T {
    fn dependencies() -> Vec<TypeId> {
        vec![TypeId::of::<T>()]
    }
}
impl<A> Dependencies for (A,)
    where A : Dependencies{
    fn dependencies() -> Vec<TypeId> {
        <A as Dependencies>::dependencies()
    }
}
impl<A,B> Dependencies for (A,B)
    where A : Dependencies,
          B : Dependencies{
    fn dependencies() -> Vec<TypeId> {
        let a = <A as Dependencies>::dependencies();
        let b = <B as Dependencies>::dependencies();
        [a,b].concat()
    }
}
impl<A,B,C> Dependencies for (A,B,C)
    where A : Dependencies,
          B : Dependencies,
          C : Dependencies{
    fn dependencies() -> Vec<TypeId> {
        let a = <A as Dependencies>::dependencies();
        let b = <B as Dependencies>::dependencies();
        let c = <C as Dependencies>::dependencies();
        [a,b,c].concat()
    }
}
impl<A,B,C,D> Dependencies for (A,B,C,D)
    where A : Dependencies,
          B : Dependencies,
          C : Dependencies,
          D : Dependencies{
    fn dependencies() -> Vec<TypeId> {
        let a = <A as Dependencies>::dependencies();
        let b = <B as Dependencies>::dependencies();
        let c = <C as Dependencies>::dependencies();
        let d = <D as Dependencies>::dependencies();
        [a,b,c,d].concat()
    }
}
impl<A,B,C,D,E> Dependencies for (A,B,C,D,E)
    where A : Dependencies,
          B : Dependencies,
          C : Dependencies,
          D : Dependencies,
          E : Dependencies{
    fn dependencies() -> Vec<TypeId> {
        let a = <A as Dependencies>::dependencies();
        let b = <B as Dependencies>::dependencies();
        let c = <C as Dependencies>::dependencies();
        let d = <D as Dependencies>::dependencies();
        let e = <E as Dependencies>::dependencies();
        [a,b,c,d,e].concat()
    }
}
impl<A,B,C,D,E,F> Dependencies for (A,B,C,D,E,F)
    where A : Dependencies,
          B : Dependencies,
          C : Dependencies,
          D : Dependencies,
          E : Dependencies,
          F : Dependencies{
    fn dependencies() -> Vec<TypeId> {
        let a = <A as Dependencies>::dependencies();
        let b = <B as Dependencies>::dependencies();
        let c = <C as Dependencies>::dependencies();
        let d = <D as Dependencies>::dependencies();
        let e = <E as Dependencies>::dependencies();
        let f = <F as Dependencies>::dependencies();
        [a,b,c,d,e,f].concat()
    }
}

pub(in crate) trait Run{
    fn initialize(&mut self,stage : &Stage);
    fn run(&mut self,stage : &Stage);
}

impl<T : for<'a> System<'a>> Run for T {
    fn initialize(&mut self, stage: &Stage) {
        //self has type Self:System
        let resource = <T as System>::Resource::resource(stage);
        self.init(resource);
    }

    fn run(&mut self, stage: &Stage) {
        //self has type Self:System
        let resource = <T as System>::Resource::resource(stage);
        self.update(resource);
    }
}

impl<'a> dyn 'static + Run {
    pub(in crate) unsafe fn downcast_ref<T : Run>(&self) -> &T {
        &*(self as *const dyn Run as *const T)
    }
    pub(in crate) unsafe fn downcast_mut<T : Run>(&mut self) -> &mut T {
        &mut *(self as *mut dyn Run as *mut T)
    }
}

/// ### A special Dependent struct
/// if a system depends on this struct ,
/// this system will run in the end.
/// ## Example
/// ```
/// # use xecs::System;
/// use xecs::resource::Resource;
/// use xecs::system::End;
/// struct Clear;
/// impl<'a> System<'a> for Clear {
///     type Resource = ();
///     type Dependencies = End;
///
///     fn init(&'a mut self, resource: <Self::Resource as Resource<'a>>::Type) {
///         // initialize data
///         // or register components
///     }
///
///     fn update(&'a mut self, resource: <Self::Resource as Resource<'a>>::Type) {
///         // DO STH WORK
///     }
/// }
/// ```
#[derive(Debug,Default,Copy,Clone)]
pub struct End;

impl Dependencies for End {
    fn dependencies() -> Vec<TypeId> {
        vec![TypeId::of::<End>()]
    }
}


#[cfg(test)]
mod tests{
    use crate::system::{System, Dependencies};
    use std::any::TypeId;

    #[test]
    fn dependencies_trait_test() {
        impl<'a> System<'a> for u32{
            type Resource = ();
            type Dependencies = ();
        }
        impl<'a> System<'a> for i32{
            type Resource = ();
            type Dependencies = ();
        }
        impl<'a> System<'a> for char{
            type Resource = ();
            type Dependencies = ();
        }

        assert_eq!(
            &<(u32,i32,char,char,char) as Dependencies>::dependencies(),
            &[  TypeId::of::<u32>(),
                TypeId::of::<i32>(),
                TypeId::of::<char>(),
                TypeId::of::<char>(),
                TypeId::of::<char>()]
        )
    }
}