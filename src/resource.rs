use crate::stage::Stage;
use crate::systems::System;
use std::cell::{Ref, RefMut};
use crate::World;

pub trait Resource<'a> {
    type Type;
    fn resource(stage : &'a Stage) -> Self::Type;
}

impl<'a,T : for<'b> System<'b>> Resource<'a> for &'a T {
    type Type = Ref<'a,T>;

    fn resource(stage: &'a Stage) -> Self::Type {
        stage.system_data_ref::<T>()
    }
}
impl<'a,T : for<'b> System<'b>> Resource<'a> for &'a mut T{
    type Type = RefMut<'a,T>;

    fn resource(stage: &'a Stage) -> Self::Type {
        stage.system_data_mut::<T>()
    }
}
impl<'a> Resource<'a> for &'a World {
    type Type = Ref<'a,World>;

    fn resource(stage: &'a Stage) -> Self::Type {
        stage.world_ref()
    }
}
impl<'a> Resource<'a> for &'a mut World {
    type Type = RefMut<'a,World>;

    fn resource(stage: &'a Stage) -> Self::Type {
        stage.world_mut()
    }
}
impl<'a> Resource<'a> for () {
    type Type = ();

    fn resource(_: &'a Stage) -> Self::Type {
        ()
    }
}
impl<'a,A> Resource<'a> for (A,)
    where A : Resource<'a>{
    type Type = (<A as Resource<'a>>::Type,);

    fn resource(stage: &'a Stage) -> Self::Type {
        (<A as Resource>::resource(stage),)
    }
}
impl<'a,A,B> Resource<'a> for (A,B)
    where A : Resource<'a>,
          B : Resource<'a>{
    type Type = (<A as Resource<'a>>::Type,
                 <B as Resource<'a>>::Type);

    fn resource(stage: &'a Stage) -> Self::Type {
        (<A as Resource>::resource(stage),
         <B as Resource>::resource(stage))
    }
}
impl<'a,A,B,C> Resource<'a> for (A,B,C)
    where A : Resource<'a>,
          B : Resource<'a>,
          C : Resource<'a>{
    type Type = (<A as Resource<'a>>::Type,
                 <B as Resource<'a>>::Type,
                 <C as Resource<'a>>::Type);

    fn resource(stage: &'a Stage) -> Self::Type {
        (<A as Resource>::resource(stage),
         <B as Resource>::resource(stage),
         <C as Resource>::resource(stage))
    }
}
impl<'a,A,B,C,D> Resource<'a> for (A,B,C,D)
    where A : Resource<'a>,
          B : Resource<'a>,
          C : Resource<'a>,
          D : Resource<'a>{
    type Type = (<A as Resource<'a>>::Type,
                 <B as Resource<'a>>::Type,
                 <C as Resource<'a>>::Type,
                 <D as Resource<'a>>::Type);

    fn resource(stage: &'a Stage) -> Self::Type {
        (<A as Resource>::resource(stage),
         <B as Resource>::resource(stage),
         <C as Resource>::resource(stage),
         <D as Resource>::resource(stage))
    }
}
impl<'a,A,B,C,D,E> Resource<'a> for (A,B,C,D,E)
    where A : Resource<'a>,
          B : Resource<'a>,
          C : Resource<'a>,
          D : Resource<'a>,
          E : Resource<'a>{
    type Type = (<A as Resource<'a>>::Type,
                 <B as Resource<'a>>::Type,
                 <C as Resource<'a>>::Type,
                 <D as Resource<'a>>::Type,
                 <E as Resource<'a>>::Type);

    fn resource(stage: &'a Stage) -> Self::Type {
        (<A as Resource>::resource(stage),
         <B as Resource>::resource(stage),
         <C as Resource>::resource(stage),
         <D as Resource>::resource(stage),
         <E as Resource>::resource(stage))
    }
}
