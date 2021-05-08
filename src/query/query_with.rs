use crate::{World, Component};
use std::marker::PhantomData;

pub struct QueryWith<'a,T> {
    pub (in crate::query) world : &'a mut World,
    pub (in crate::query) _marker : PhantomData<T>,
}

pub struct QueryEntitiesWith<'a,T> {
    pub (in crate::query) world : &'a mut World,
    pub (in crate::query) _marker : PhantomData<T>,
}

impl<'a,A : Component,B : Component> QueryWith<'a,(A,B)> {
    pub fn query(self) -> impl Iterator<Item=(&'a A,&'a B)>{
        //has group
        todo!()
    }

    pub fn query_mut(self) -> impl Iterator<Item=(&'a mut A,&'a mut B)> {
        todo!()
    }
}