use crate::internal::Memory::*;
use glam::Vec3;
use libc;

extern "C" {}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MemPool {
    pub size: u32,
    pub capacity: u32,
    pub freeList: *mut libc::c_void,
    pub cellSize: u32,
    pub blockSize: u32,
    pub blockCount: u16,
    pub blocks: *mut *mut libc::c_void,
}

unsafe extern "C" fn MemPool_Grow(mut this: *mut MemPool) {
    let fresh0 = (*this).blockCount;
    (*this).blockCount = ((*this).blockCount).wrapping_add(1);
    let mut newBlockIndex: u16 = fresh0;
    (*this).capacity = (*this).capacity.wrapping_add((*this).blockSize);
    (*this).blocks = MemRealloc(
        (*this).blocks as *mut libc::c_void,
        ((*this).blockCount as usize).wrapping_mul(::core::mem::size_of::<*mut libc::c_void>()),
    ) as *mut *mut libc::c_void;
    let mut newBlock: *mut libc::c_void =
        MemAlloc(((*this).cellSize).wrapping_mul((*this).blockSize) as usize);
    let ref mut fresh1 = *((*this).blocks).offset(newBlockIndex as isize);
    *fresh1 = newBlock;
    let mut prev: *mut *mut libc::c_void = &mut (*this).freeList;
    let mut pCurr: *mut libc::c_char = newBlock as *mut libc::c_char;
    let mut i: u32 = 0_i32 as u32;
    while i < (*this).blockSize {
        *prev = pCurr as *mut libc::c_void;
        prev = pCurr as *mut *mut libc::c_void;
        pCurr = pCurr.offset((*this).cellSize as isize);
        i = i.wrapping_add(1);
    }
    *prev = std::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn MemPool_Create(mut cellSize: u32, mut blockSize: u32) -> *mut MemPool {
    let mut this: *mut MemPool =
        MemAlloc(::core::mem::size_of::<MemPool>()) as *mut MemPool;
    (*this).size = 0_i32 as u32;
    (*this).capacity = 0_i32 as u32;
    (*this).freeList = std::ptr::null_mut();
    (*this).cellSize = cellSize;
    (*this).blockSize = blockSize;
    (*this).blockCount = 0_i32 as u16;
    (*this).blocks = std::ptr::null_mut();
    return this;
}

#[no_mangle]
pub unsafe extern "C" fn MemPool_CreateAuto(mut elemSize: u32) -> *mut MemPool {
    return MemPool_Create(elemSize, (0x1000_i32 as u32).wrapping_div(elemSize));
}

#[no_mangle]
pub unsafe extern "C" fn MemPool_Free(mut this: *mut MemPool) {
    let mut i: u16 = 0_i32 as u16;
    while (i as i32) < (*this).blockCount as i32 {
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
    MemZero(freeCell, (*this).cellSize as usize);
    return freeCell;
}

#[no_mangle]
pub unsafe extern "C" fn MemPool_Clear(mut this: *mut MemPool) {
    (*this).size = 0_i32 as u32;
    (*this).freeList = std::ptr::null_mut();
    let mut prev: *mut *mut libc::c_void = &mut (*this).freeList;
    let mut i: u32 = 0_i32 as u32;
    while i < (*this).blockCount as u32 {
        let mut pCurr: *mut libc::c_char =
            *((*this).blocks).offset(i as isize) as *mut libc::c_char;
        let mut j: u32 = 0_i32 as u32;
        while j < (*this).blockSize {
            *prev = pCurr as *mut libc::c_void;
            prev = pCurr as *mut *mut libc::c_void;
            pCurr = pCurr.offset((*this).cellSize as isize);
            j = j.wrapping_add(1);
        }
        i = i.wrapping_add(1);
    }
    *prev = std::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn MemPool_Dealloc(mut this: *mut MemPool, mut ptr: *mut libc::c_void) {
    let ref mut fresh2 = *(ptr as *mut *mut libc::c_void);
    *fresh2 = (*this).freeList;
    (*this).freeList = ptr;
    (*this).size = ((*this).size).wrapping_sub(1);
}

#[no_mangle]
pub unsafe extern "C" fn MemPool_GetCapacity(mut this: *mut MemPool) -> u32 {
    return (*this).capacity;
}

#[no_mangle]
pub unsafe extern "C" fn MemPool_GetSize(mut this: *mut MemPool) -> u32 {
    return (*this).size;
}
