use std::any::TypeId;

use crate::Component;

/// A trait make tuple dynamic
pub trait Tuple {
    /// Get the count of tuple elements
    fn len(&self) -> usize; 
    /// Get type of data in tuple
    fn type_in(&self, index: usize) -> Option<TypeId>;
    /// Get pointer of data in tuple
    fn ptr_in(&self,index: usize) -> Option<*const u8>;

    fn get_ptrs(&self, ptrs: &mut [*mut u8]) {
        for i in 0..self.len() {
            ptrs[i] = self.ptr_in(i).unwrap_or_else(|| unreachable!());
        }
    }

    unsafe fn from_ptrs(ptrs: &[*mut u8]) -> Self;
}


impl Tuple for () {
    fn len(&self) -> usize {
        0
    }

    fn type_in(&self, index: usize) -> Option<TypeId> {
        None
    }

    fn ptr_in(&self,index: usize) -> Option<*const u8> {
        None
    }

    unsafe fn from_ptrs(ptrs: &[*mut u8]) -> Self {
        ()
    }
}

impl<A:Component, B: Component> Tuple for (A,B) {
    fn len(&self) -> usize {
        2
    }

    fn type_in(&self, index: usize) -> Option<TypeId> {
        if index == 0 {
            Some(TypeId::of::<A>())
        } else if index == 1 {
            Some(TypeId::of::<B>())
        } else {
            None
        }
    }

    fn ptr_in(&self,index: usize) -> Option<*const u8> {
        if index == 0 {
            Some(&self.0 as *const A as *const u8)
        } else if index == 1 {
            Some(&self.1 as *const B as *const u8)
        } else {
            None
        }
    }

    unsafe fn from_ptrs(ptrs: &[*mut u8]) -> Self {
        let a = (*ptrs.get_unchecked(0)) as *mut A;
        let b = (*ptrs.get_unchecked(1)) as *mut B;
        (std::ptr::read(a),std::ptr::read(b))
    }
}
