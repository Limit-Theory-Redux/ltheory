use ::libc;
use super::internal::Memory::*;
extern "C" {
    fn memset(
        _: *mut libc::c_void,
        _: libc::c_int,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn realloc(_: *mut libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
}
pub type uint16_t = libc::c_ushort;
pub type uint32_t = libc::c_uint;
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
pub type uint16 = uint16_t;
pub type uint32 = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MemPool {
    pub size: uint32,
    pub capacity: uint32,
    pub freeList: *mut libc::c_void,
    pub cellSize: uint32,
    pub blockSize: uint32,
    pub blockCount: uint16,
    pub blocks: *mut *mut libc::c_void,
}


#[inline]
unsafe extern "C" fn MemZero(mut dst: *mut libc::c_void, mut size: size_t) {
    memset(dst, 0 as libc::c_int, size);
}
unsafe extern "C" fn MemPool_Grow(mut self_0: *mut MemPool) {
    let fresh0 = (*self_0).blockCount;
    (*self_0).blockCount = ((*self_0).blockCount).wrapping_add(1);
    let mut newBlockIndex: uint16 = fresh0;
    (*self_0)
        .capacity = ((*self_0).capacity as libc::c_uint)
        .wrapping_add((*self_0).blockSize) as uint32 as uint32;
    (*self_0)
        .blocks = MemRealloc(
        (*self_0).blocks as *mut libc::c_void,
        ((*self_0).blockCount as libc::c_ulong)
            .wrapping_mul(::core::mem::size_of::<*mut libc::c_void>()),
    ) as *mut *mut libc::c_void;
    let mut newBlock: *mut libc::c_void = MemAlloc(
        ((*self_0).cellSize).wrapping_mul((*self_0).blockSize) as size_t,
    );
    let ref mut fresh1 = *((*self_0).blocks).offset(newBlockIndex as isize);
    *fresh1 = newBlock;
    let mut prev: *mut *mut libc::c_void = &mut (*self_0).freeList;
    let mut pCurr: *mut libc::c_char = newBlock as *mut libc::c_char;
    let mut i: uint32 = 0 as libc::c_int as uint32;
    while i < (*self_0).blockSize {
        *prev = pCurr as *mut libc::c_void;
        prev = pCurr as *mut *mut libc::c_void;
        pCurr = pCurr.offset((*self_0).cellSize as isize);
        i = i.wrapping_add(1);
    }
    *prev = 0 as *mut libc::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn MemPool_Create(
    mut cellSize: uint32,
    mut blockSize: uint32,
) -> *mut MemPool {
    let mut self_0: *mut MemPool = MemAlloc(
        ::core::mem::size_of::<MemPool>() as libc::c_ulong,
    ) as *mut MemPool;
    (*self_0).size = 0 as libc::c_int as uint32;
    (*self_0).capacity = 0 as libc::c_int as uint32;
    (*self_0).freeList = 0 as *mut libc::c_void;
    (*self_0).cellSize = cellSize;
    (*self_0).blockSize = blockSize;
    (*self_0).blockCount = 0 as libc::c_int as uint16;
    (*self_0).blocks = 0 as *mut *mut libc::c_void;
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn MemPool_CreateAuto(mut elemSize: uint32) -> *mut MemPool {
    return MemPool_Create(
        elemSize,
        (0x1000 as libc::c_int as libc::c_uint).wrapping_div(elemSize),
    );
}
#[no_mangle]
pub unsafe extern "C" fn MemPool_Free(mut self_0: *mut MemPool) {
    let mut i: uint16 = 0 as libc::c_int as uint16;
    while (i as libc::c_int) < (*self_0).blockCount as libc::c_int {
        MemFree(*((*self_0).blocks).offset(i as isize));
        i = i.wrapping_add(1);
    }
    MemFree((*self_0).blocks as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn MemPool_Alloc(mut self_0: *mut MemPool) -> *mut libc::c_void {
    if ((*self_0).size == (*self_0).capacity) as libc::c_int as libc::c_long != 0 {
        MemPool_Grow(self_0);
    }
    let mut freeCell: *mut libc::c_void = (*self_0).freeList;
    (*self_0).freeList = *(freeCell as *mut *mut libc::c_void);
    (*self_0).size = ((*self_0).size).wrapping_add(1);
    MemZero(freeCell, (*self_0).cellSize as size_t);
    return freeCell;
}
#[no_mangle]
pub unsafe extern "C" fn MemPool_Clear(mut self_0: *mut MemPool) {
    (*self_0).size = 0 as libc::c_int as uint32;
    (*self_0).freeList = 0 as *mut libc::c_void;
    let mut prev: *mut *mut libc::c_void = &mut (*self_0).freeList;
    let mut i: uint32 = 0 as libc::c_int as uint32;
    while i < (*self_0).blockCount as libc::c_uint {
        let mut pCurr: *mut libc::c_char = *((*self_0).blocks).offset(i as isize)
            as *mut libc::c_char;
        let mut j: uint32 = 0 as libc::c_int as uint32;
        while j < (*self_0).blockSize {
            *prev = pCurr as *mut libc::c_void;
            prev = pCurr as *mut *mut libc::c_void;
            pCurr = pCurr.offset((*self_0).cellSize as isize);
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    *prev = 0 as *mut libc::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn MemPool_Dealloc(
    mut self_0: *mut MemPool,
    mut ptr: *mut libc::c_void,
) {
    let ref mut fresh2 = *(ptr as *mut *mut libc::c_void);
    *fresh2 = (*self_0).freeList;
    (*self_0).freeList = ptr;
    (*self_0).size = ((*self_0).size).wrapping_sub(1);
}
#[no_mangle]
pub unsafe extern "C" fn MemPool_GetCapacity(mut self_0: *mut MemPool) -> uint32 {
    return (*self_0).capacity;
}
#[no_mangle]
pub unsafe extern "C" fn MemPool_GetSize(mut self_0: *mut MemPool) -> uint32 {
    return (*self_0).size;
}
