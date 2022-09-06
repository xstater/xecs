use std::any::TypeId;

use crate::Component;

/// tuple的公有trait
pub trait Tuple {
    /// 获得Tuple中每个元素的类型ID
    fn types() -> Vec<TypeId>;
    /// 从指针数组构建Tuple
    /// # Safety:
    /// * `slice.len() <= types().len()`
    /// * `slice`中指针的实际类型与Tuple中元素类型一一对应
    /// * 调用该方法后`slice`中指针指向的数据应被`forget`
    unsafe fn from_ptr_slice(slice: &[*mut u8]) -> Self;
}

impl<A: Component,B: Component> Tuple for (A,B) {
    fn types() -> Vec<TypeId> {
        vec![TypeId::of::<A>(),TypeId::of::<B>()]
    }

    unsafe fn from_ptr_slice(slice: &[*mut u8]) -> Self {
        todo!()
    }
}
