use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
extern "C" {
}
pub type uint16_t = libc::c_ushort;
pub type uint32_t = libc::c_uint;
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


unsafe extern "C" fn MemPool_Grow(mut this: *mut MemPool) {
    let fresh0 = (*this).blockCount;
    (*this).blockCount = ((*this).blockCount).wrapping_add(1);
    let mut newBlockIndex: uint16 = fresh0;
    (*this)
        .capacity = ((*this).capacity as libc::c_uint)
        .wrapping_add((*this).blockSize) as uint32 as uint32;
    (*this)
        .blocks = MemRealloc(
        (*this).blocks as *mut libc::c_void,
        ((*this).blockCount as usize).wrapping_mul(::core::mem::size_of::<*mut libc::c_void>()),
    ) as *mut *mut libc::c_void;
    let mut newBlock: *mut libc::c_void = MemAlloc(
        ((*this).cellSize).wrapping_mul((*this).blockSize) as libc::size_t,
    );
    let ref mut fresh1 = *((*this).blocks).offset(newBlockIndex as isize);
    *fresh1 = newBlock;
    let mut prev: *mut *mut libc::c_void = &mut (*this).freeList;
    let mut pCurr: *mut libc::c_char = newBlock as *mut libc::c_char;
    let mut i: uint32 = 0 as libc::c_int as uint32;
    while i < (*this).blockSize {
        *prev = pCurr as *mut libc::c_void;
        prev = pCurr as *mut *mut libc::c_void;
        pCurr = pCurr.offset((*this).cellSize as isize);
        i = i.wrapping_add(1);
    }
    *prev = 0 as *mut libc::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn MemPool_Create(
    mut cellSize: uint32,
    mut blockSize: uint32,
) -> *mut MemPool {
    let mut this: *mut MemPool = MemAlloc(
        ::core::mem::size_of::<MemPool>() as usize,
    ) as *mut MemPool;
    (*this).size = 0 as libc::c_int as uint32;
    (*this).capacity = 0 as libc::c_int as uint32;
    (*this).freeList = 0 as *mut libc::c_void;
    (*this).cellSize = cellSize;
    (*this).blockSize = blockSize;
    (*this).blockCount = 0 as libc::c_int as uint16;
    (*this).blocks = 0 as *mut *mut libc::c_void;
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn MemPool_CreateAuto(mut elemSize: uint32) -> *mut MemPool {
    return MemPool_Create(
        elemSize,
        (0x1000 as libc::c_int as libc::c_uint).wrapping_div(elemSize),
    );
}
#[no_mangle]
pub unsafe extern "C" fn MemPool_Free(mut this: *mut MemPool) {
    let mut i: uint16 = 0 as libc::c_int as uint16;
    while (i as libc::c_int) < (*this).blockCount as libc::c_int {
        MemFree(*((*this).blocks).offset(i as isize));
        i = i.wrapping_add(1);
    }
    MemFree((*this).blocks as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn MemPool_Alloc(mut this: *mut MemPool) -> *mut libc::c_void {
    if ((*this).size == (*this).capacity) as libc::c_long != 0 {
        MemPool_Grow(this);
    }
    let mut freeCell: *mut libc::c_void = (*this).freeList;
    (*this).freeList = *(freeCell as *mut *mut libc::c_void);
    (*this).size = ((*this).size).wrapping_add(1);
    MemZero(freeCell, (*this).cellSize as libc::size_t);
    return freeCell;
}
#[no_mangle]
pub unsafe extern "C" fn MemPool_Clear(mut this: *mut MemPool) {
    (*this).size = 0 as libc::c_int as uint32;
    (*this).freeList = 0 as *mut libc::c_void;
    let mut prev: *mut *mut libc::c_void = &mut (*this).freeList;
    let mut i: uint32 = 0 as libc::c_int as uint32;
    while i < (*this).blockCount as libc::c_uint {
        let mut pCurr: *mut libc::c_char = *((*this).blocks).offset(i as isize)
            as *mut libc::c_char;
        let mut j: uint32 = 0 as libc::c_int as uint32;
        while j < (*this).blockSize {
            *prev = pCurr as *mut libc::c_void;
            prev = pCurr as *mut *mut libc::c_void;
            pCurr = pCurr.offset((*this).cellSize as isize);
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    *prev = 0 as *mut libc::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn MemPool_Dealloc(
    mut this: *mut MemPool,
    mut ptr: *mut libc::c_void,
) {
    let ref mut fresh2 = *(ptr as *mut *mut libc::c_void);
    *fresh2 = (*this).freeList;
    (*this).freeList = ptr;
    (*this).size = ((*this).size).wrapping_sub(1);
}
#[no_mangle]
pub unsafe extern "C" fn MemPool_GetCapacity(mut this: *mut MemPool) -> uint32 {
    return (*this).capacity;
}
#[no_mangle]
pub unsafe extern "C" fn MemPool_GetSize(mut this: *mut MemPool) -> uint32 {
    return (*this).size;
}
