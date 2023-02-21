use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
extern "C" {
    fn Fatal(_: cstr, _: ...);
}
pub type cstr = *const libc::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MemStack {
    pub size: u32,
    pub capacity: u32,
    pub data: *mut libc::c_void,
}

#[no_mangle]
pub unsafe extern "C" fn MemStack_Create(mut capacity: u32) -> *mut MemStack {
    let mut this: *mut MemStack = MemAlloc(
        ::core::mem::size_of::<MemStack>() as usize,
    ) as *mut MemStack;
    (*this).size = 0 as i32 as u32;
    (*this).capacity = capacity;
    (*this).data = MemAlloc(capacity as usize);
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn MemStack_Free(mut this: *mut MemStack) {
    MemFree((*this).data);
    MemFree(this as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn MemStack_Alloc(
    mut this: *mut MemStack,
    mut size: u32,
) -> *mut libc::c_void {
    if ((*this).size).wrapping_add(size) > (*this).capacity {
        Fatal(
            b"MemStack_Alloc: Allocation request exceeds remaining capacity\0"
                as *const u8 as *const libc::c_char,
        );
    }
    let mut p: *mut libc::c_void = ((*this).data as *mut libc::c_char)
        .offset((*this).size as isize) as *mut libc::c_void;
    (*this)
        .size = ((*this).size as u32).wrapping_add(size) as u32;
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn MemStack_Clear(mut this: *mut MemStack) {
    (*this).size = 0 as i32 as u32;
}
#[no_mangle]
pub unsafe extern "C" fn MemStack_Dealloc(mut this: *mut MemStack, mut size: u32) {
    if (*this).size < size {
        Fatal(
            b"MemStack_Dealloc: Attempt to dealloc more memory than is allocated\0"
                as *const u8 as *const libc::c_char,
        );
    }
    (*this)
        .size = ((*this).size as u32).wrapping_sub(size) as u32;
}
#[no_mangle]
pub unsafe extern "C" fn MemStack_CanAlloc(
    mut this: *mut MemStack,
    mut size: u32,
) -> bool {
    return ((*this).size).wrapping_add(size) <= (*this).capacity;
}
#[no_mangle]
pub unsafe extern "C" fn MemStack_GetSize(mut this: *mut MemStack) -> u32 {
    return (*this).size;
}
#[no_mangle]
pub unsafe extern "C" fn MemStack_GetCapacity(mut this: *mut MemStack) -> u32 {
    return (*this).capacity;
}
#[no_mangle]
pub unsafe extern "C" fn MemStack_GetRemaining(mut this: *mut MemStack) -> u32 {
    return ((*this).capacity).wrapping_sub((*this).size);
}
