//! # System trait and Dependencies trait
use crate::stage::Stage;
use std::any::{TypeId};
use crate::resource::Resource;
use std::error::Error;
use std::collections::HashMap;
use std::convert::Infallible;

/// ## System trait
/// * System can has it owm data like ```struct Event(u32)```
/// * System can get other systems' data through Resource
/// * A System has many parameters ,the macro or default parameter feature may simplify this work.
pub trait System<'a> : 'static{
    /// Required data while system initializing
    type InitResource : Resource<'a>;
    /// Required data while system running
    type Resource : Resource<'a>;
    /// The Dependencies of system
    type Dependencies : Dependencies;
    /// the error type
    type Error : Error + 'static;
    /// initialize the data
    #[allow(unused_variables)]
    fn init(&'a mut self,resource : <Self::InitResource as Resource<'a>>::Type) -> Result<(),Self::Error>{
        Ok(())
    }
    /// update the states
    #[allow(unused_variables)]
    fn update(&'a mut self,resource : <Self::Resource as Resource<'a>>::Type) -> Result<(),Self::Error>{
        Ok(())
    }
}


/// Something can be dependencies of systems
pub trait Dependencies : 'static {
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
        let resource = <T as System>::InitResource::resource(stage);
        if let Err(error) = self.init(resource) {
            stage.system_data_mut::<Errors>().store_error::<T>(error);
        }
    }

    fn run(&mut self, stage: &Stage) {
        //self has type Self:System
        let resource = <T as System>::Resource::resource(stage);
        if let Err(error) = self.update(resource) {
            stage.system_data_mut::<Errors>().store_error::<T>(error);
        }
    }
}

impl dyn 'static + Run {
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
/// use std::convert::Infallible;
/// struct Clear;
/// impl<'a> System<'a> for Clear {
///     type InitResource = ();
///     type Resource = ();
///     type Dependencies = End;
///     type Error = Infallible;
///
///
///     fn init(&'a mut self, resource: <Self::Resource as Resource<'a>>::Type) -> Result<(),Self::Error>{
///         // initialize data
///         // or register components
///     }
///
///     fn update(&'a mut self, resource: <Self::Resource as Resource<'a>>::Type) -> Result<(),Self::Error> {
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

/// A special System that store the errors
/// ## Examples
/// ```no_run
/// use xecs::System;
/// use std::convert::Infallible;
/// use xecs::system::Errors;
/// use std::cell::RefMut;
///
/// struct ErrorHandler;
/// impl<'a> System<'a> for ErrorHandler {
///     type InitResource = ();
///     type Resource = &'a mut Errors;
///     type Dependencies = ErrorSource;
///     type Error = Infallible;
///
///     fn update(&'a mut self,mut errors : RefMut<'a,Errors>) -> Result<(), Self::Error> {
///         if let Some(error) = errors.fetch_error::<ErrorSource>() {
///             println!("Catch error with value {}",error.as_ref().0);
///         }
///         Ok(())
///     }
/// }
/// ```
#[derive(Debug)]
pub struct Errors {
    errors : HashMap<TypeId,Option<Box<dyn Error>>>
}

impl<'a> System<'a> for Errors {
    type InitResource = ();
    type Resource = ();
    type Dependencies = ();
    type Error = Infallible;
}

impl Errors{
    pub(in crate) fn new() -> Errors {
        Errors {
            errors: HashMap::new()
        }
    }

    pub(in crate) fn register<S : for<'a> System<'a>>(&mut self) {
        let tid = TypeId::of::<S>();
        if !self.errors.contains_key(&tid) {
            self.errors.insert(tid,Option::None);
        }
    }

    pub(in crate) fn store_error<S>(&mut self,error : <S as System<'_>>::Error)
        where S : for<'a> System<'a>{
        let tid = TypeId::of::<S>();
        debug_assert!(self.errors.contains_key(&tid),
                      "Store error failed! No such system");
        self.errors.get_mut(&tid).unwrap()
            .replace(Box::new(error));
    }

    pub fn fetch_error<S : for<'a> System<'a>>(&mut self) -> Option<Box<<S as System<'_>>::Error>> {
        let tid = TypeId::of::<S>();
        debug_assert!(self.errors.contains_key(&tid),
            "Fetch error failed! No such system");
        self.errors.get_mut(&tid).unwrap()
            .take()
            .map(|error| {
                // must success!
                // because errors‘ Box<dyn Error> is S::Error !
                error
                    .downcast::<S::Error>()
                    .unwrap()
            })
    }
}

#[cfg(test)]
mod tests{
    use crate::system::{System, Dependencies};
    use std::any::TypeId;
    use std::convert::Infallible;

    #[test]
    fn dependencies_trait_test() {
        impl<'a> System<'a> for u32{
            type InitResource = ();
            type Resource = ();
            type Dependencies = ();
            type Error = Infallible;
        }
        impl<'a> System<'a> for i32{
            type InitResource = ();
            type Resource = ();
            type Dependencies = ();
            type Error = Infallible;
        }
        impl<'a> System<'a> for char{
            type InitResource = ();
            type Resource = ();
            type Dependencies = ();
            type Error = Infallible;
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