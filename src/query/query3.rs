use crate::{Component, World};
use std::marker::PhantomData;

pub struct Query2<'a,A : Component,B : Component,C : Component >{
    pub(in crate::query) world : &'a mut World,
    pub(in crate::query) _marker : PhantomData<(A,B,C)>
}

pub struct QueryEntity2<'a,A : Component,B : Component,C : Component>{
    pub(in crate::query) world : &'a mut World,
    pub(in crate::query) _marker : PhantomData<(A,B,C)>
}


