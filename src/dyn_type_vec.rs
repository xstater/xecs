use crate::{ComponentAny, Component};
use std::any::TypeId;

/// 一个可以把Vec变成动态类型的Trait
pub trait DynTypeVec {
    /// 获得Vec实际储存数据类型ID
    fn type_id(&self) -> TypeId;
    /// 移除指定位置上的元素
    /// # Details
    /// * 这个方法会drop掉被移除的元素
    /// # Panics
    /// * index越界
    fn remove_and_drop(&mut self, index: usize);
    /// 移除指定位置上的元素
    /// # Details
    /// * 这个方法不会drop掉被移除的元素
    /// # Panics
    /// * index越界
    fn remove_and_forget(&mut self, index: usize);
    /// 交换两个位置上的元素
    /// # Panics
    /// * index越界
    fn swap(&mut self, index_a: usize, index_b: usize);
    /// 获得数组的长度
    fn len(&self) -> usize;
    /// 判断数组是否为空
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// 加一个元素到数组最后
    /// # Safety
    /// * data必须指向有效值
    /// * data的实际类型必须和数组类型一致
    /// * 调用这个函数之后不能使用data,因为所有权发生了转移
    unsafe fn push_any_unchecked(&mut self, data: *mut u8);
    /// 加一堆元素到数组最后
    /// # Details
    /// * 这个data指针类型应该是`Vec<T>`,因为内部是调用`Vec::append`实现的
    /// # Safety
    /// * data必须指向有效值
    /// * data必须是Vec<T>的指针
    /// * data的指向的数组实际类型必须和当前数组类型一致
    /// * 调用这个函数之后不能使用data,因为所有权发生了转移
    unsafe fn push_any_batch_unchecked(&mut self, data: *mut u8);
    /// 获得单个元素的指针
    fn get_ptr(&self, index: usize) -> Option<*const u8>;
    /// 获得单个元素的指针
    fn get_mut_ptr(&mut self, index: usize) -> Option<*mut u8>;
    /// 获得整个数组的头指针
    fn data_ptr(&self) -> *const u8;
    /// 获得整个数组的头指针
    fn data_mut_ptr(&mut self) -> *mut u8;
}

impl<T> DynTypeVec for Vec<T>
where
    T: Component,
{
    fn type_id(&self) -> TypeId {
        TypeId::of::<T>()
    }

    fn remove_and_drop(&mut self, index: usize) {
        Vec::remove(self, index);
    }

    fn remove_and_forget(&mut self, index: usize) {
        let result = Vec::remove(self, index);
        std::mem::forget(result);
    }

    fn swap(&mut self, index_a: usize, index_b: usize) {
        Vec::swap(self, index_a, index_b)
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn is_empty(&self) -> bool {
        Vec::is_empty(self)
    }

    unsafe fn push_any_unchecked(&mut self, data: *mut u8) {
        let data = data as *mut T;
        let data = std::ptr::read(data);
        Vec::push(self, data)
    }

    unsafe fn push_any_batch_unchecked(&mut self, data: *mut u8) {
        let data = data as *mut Vec<T>;
        let mut data = std::ptr::read(data);
        Vec::append(self,&mut data)
    }

    fn get_ptr(&self, index: usize) -> Option<*const u8> {
        self.get(index).map(|data| data as *const T as *const _)
    }

    fn get_mut_ptr(&mut self, index: usize) -> Option<*mut u8> {
        self.get_mut(index).map(|data| data as *mut T as *mut _)
    }

    fn data_ptr(&self) -> *const u8 {
        self.as_ptr() as *const _
    }

    fn data_mut_ptr(&mut self) -> *mut u8 {
        self.as_mut_ptr() as *mut _
    }
}

impl dyn 'static + DynTypeVec {
    /// Downcast `&dyn ComponentStorage` to `&T`
    /// # Safety
    /// * Safe when `self` has type `T`
    pub unsafe fn downcast_ref<T: DynTypeVec>(&self) -> &T {
        &*(self as *const dyn DynTypeVec as *const T)
    }

    /// Downcast `&mut dyn ComponentStorage` to `&mut T`
    /// # Safety
    /// * Safe when `self` has type `T`
    pub unsafe fn downcast_mut<T: DynTypeVec>(&mut self) -> &mut T {
        &mut *(self as *mut dyn DynTypeVec as *mut T)
    }
}
