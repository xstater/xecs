use crate::{Component};
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
    //// 替换数组中之前的值
    /// # Details
    /// * 之前的值会被`drop`
    /// # Safety
    /// * `index`必须在范围内
    /// * `data`实际类型必须和数组中元素类型相同
    /// * 调用该方法后data不能使用，因为所有权发生了转移
    unsafe fn replace_any_and_drop_unchecked(&mut self,index: usize,data: *mut u8);
    //// 替换数组中之前的值
    /// # Details
    /// * 之前的值会被`forget`
    /// # Safety
    /// * `index`必须在范围内
    /// * `data`实际类型必须和数组中元素类型相同
    /// * 调用该方法后data不能使用，因为所有权发生了转移
    unsafe fn replace_any_and_forget_unchecked(&mut self,index: usize,data: *mut u8);
    /// 弹出最后一个元素
    /// # Details
    /// * 如果`self为`空，则什么也不会做
    /// * 弹出的元素会被`drop`
    fn pop_and_drop(&mut self);
    /// 弹出最后一个元素
    /// # Details
    /// * 如果`self为`空，则什么也不会做
    /// * 弹出的元素会被`forget`
    fn pop_and_forget(&mut self);
    /// 获得单个元素的指针
    fn get_ptr(&self, index: usize) -> Option<*const u8>;
    /// 获得单个元素的指针
    fn get_mut_ptr(&mut self, index: usize) -> Option<*mut u8>;
    /// 获得单个元素的指针
    /// # Safety
    /// * index 必须在数组的范围内
    unsafe fn get_ptr_unchecked(&self, index: usize) -> *const u8;
    /// 获得单个元素的指针
    /// # Safety
    /// * index 必须在数组的范围内
    unsafe fn get_mut_ptr_unchecked(&mut self,index: usize) -> *mut u8;
    /// 获得整个数组的头指针
    fn data_ptr(&self) -> *const u8;
    /// 获得整个数组的头指针
    fn data_mut_ptr(&mut self) -> *mut u8;


    /// 与最后一个元素交换并删除
    fn swap_remove_and_drop(&mut self, index: usize){
        let last_index = self.len() - 1;
        self.swap(index,last_index);
        self.remove_and_drop(last_index);
    }
    /// 与最后一个元素交换并删除
    fn swap_remove_and_forget(&mut self, index: usize){
        let last_index = self.len() - 1;
        self.swap(index,last_index);
        self.remove_and_forget(last_index);
    }
    /// 获得第一个元素
    fn first_ptr(&self) -> Option<*const u8> {
        self.get_ptr(0)
    }
    /// 获得第一个元素
    fn first_mut_ptr(&mut self) -> Option<*mut u8> {
        self.get_mut_ptr(0)
    }
    /// 获得最后一个元素
    fn last_ptr(&self) -> Option<*const u8> {
         self.get_ptr(self.len() - 1)
    }
    /// 获得最后一个元素
    fn last_mut_ptr(&mut self) -> Option<*mut u8> {
        self.get_mut_ptr(self.len() - 1)
    }
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
        self.as_mut_slice().swap(index_a, index_b)
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

    unsafe fn replace_any_and_drop_unchecked(&mut self,index: usize,data: *mut u8) {
        let data = data as *mut T;
        let data = std::ptr::read(data);
        let replaced = std::mem::replace(self.get_unchecked_mut(index), data);
        std::mem::drop(replaced)
    }

    unsafe fn replace_any_and_forget_unchecked(&mut self,index: usize,data: *mut u8) {
        let data = data as *mut T;
        let data = std::ptr::read(data);
        let replaced = std::mem::replace(self.get_unchecked_mut(index), data);
        std::mem::forget(replaced)
    }

    fn pop_and_drop(&mut self) {
        self.pop().map(|x|std::mem::drop(x));
    }

    fn pop_and_forget(&mut self) {
        self.pop().map(|x|std::mem::forget(x));
    }

    fn get_ptr(&self, index: usize) -> Option<*const u8> {
        self.get(index).map(|data| data as *const T as *const _)
    }

    fn get_mut_ptr(&mut self, index: usize) -> Option<*mut u8> {
        self.get_mut(index).map(|data| data as *mut T as *mut _)
    }

    unsafe fn get_ptr_unchecked(&self, index: usize) -> *const u8 {
        self.get_unchecked(index) as *const T as *const _
    }

    unsafe fn get_mut_ptr_unchecked(&mut self,index: usize) -> *mut u8 {
        self.get_unchecked_mut(index) as *mut T as *mut _
    }

    fn data_ptr(&self) -> *const u8 {
        self.as_ptr() as *const _
    }

    fn data_mut_ptr(&mut self) -> *mut u8 {
        self.as_mut_ptr() as *mut _
    }

    fn swap_remove_and_drop(&mut self, index: usize) {
        self.swap_remove(index);
    }

    fn swap_remove_and_forget(&mut self, index: usize) {
        let removed = self.swap_remove(index);
        std::mem::forget(removed);
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
