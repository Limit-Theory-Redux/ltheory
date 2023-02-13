use ::libc;
use super::internal::Memory::*;
extern "C" {
    fn Fatal(_: cstr, _: ...);
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
}
pub type uint32_t = libc::c_uint;
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
pub type cstr = *const libc::c_char;
pub type uint32 = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MemStack {
    pub size: uint32,
    pub capacity: uint32,
    pub data: *mut libc::c_void,
}

#[no_mangle]
pub unsafe extern "C" fn MemStack_Create(mut capacity: uint32) -> *mut MemStack {
    let mut self_0: *mut MemStack = MemAlloc(
        ::core::mem::size_of::<MemStack>() as libc::c_ulong,
    ) as *mut MemStack;
    (*self_0).size = 0 as libc::c_int as uint32;
    (*self_0).capacity = capacity;
    (*self_0).data = MemAlloc(capacity as size_t);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn MemStack_Free(mut self_0: *mut MemStack) {
    MemFree((*self_0).data);
    MemFree(self_0 as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn MemStack_Alloc(
    mut self_0: *mut MemStack,
    mut size: uint32,
) -> *mut libc::c_void {
    if ((*self_0).size).wrapping_add(size) > (*self_0).capacity {
        Fatal(
            b"MemStack_Alloc: Allocation request exceeds remaining capacity\0"
                as *const u8 as *const libc::c_char,
        );
    }
    let mut p: *mut libc::c_void = ((*self_0).data as *mut libc::c_char)
        .offset((*self_0).size as isize) as *mut libc::c_void;
    (*self_0)
        .size = ((*self_0).size as libc::c_uint).wrapping_add(size) as uint32 as uint32;
    return p;
}
#[no_mangle]
pub unsafe extern "C" fn MemStack_Clear(mut self_0: *mut MemStack) {
    (*self_0).size = 0 as libc::c_int as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn MemStack_Dealloc(mut self_0: *mut MemStack, mut size: uint32) {
    if (*self_0).size < size {
        Fatal(
            b"MemStack_Dealloc: Attempt to dealloc more memory than is allocated\0"
                as *const u8 as *const libc::c_char,
        );
    }
    (*self_0)
        .size = ((*self_0).size as libc::c_uint).wrapping_sub(size) as uint32 as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn MemStack_CanAlloc(
    mut self_0: *mut MemStack,
    mut size: uint32,
) -> bool {
    return ((*self_0).size).wrapping_add(size) <= (*self_0).capacity;
}
#[no_mangle]
pub unsafe extern "C" fn MemStack_GetSize(mut self_0: *mut MemStack) -> uint32 {
    return (*self_0).size;
}
#[no_mangle]
pub unsafe extern "C" fn MemStack_GetCapacity(mut self_0: *mut MemStack) -> uint32 {
    return (*self_0).capacity;
}
#[no_mangle]
pub unsafe extern "C" fn MemStack_GetRemaining(mut self_0: *mut MemStack) -> uint32 {
    return ((*self_0).capacity).wrapping_sub((*self_0).size);
}
