#![allow(non_snake_case)] // TODO: remove this and fix all warnings
#![allow(unsafe_code)] // TODO: refactor

use internal::*;

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

unsafe extern "C" fn MemPool_Grow(this: &mut MemPool) {
    unsafe {
        let fresh0 = this.blockCount;
        this.blockCount = (this.blockCount).wrapping_add(1);
        let newBlockIndex: u16 = fresh0;
        this.capacity = this.capacity.wrapping_add(this.blockSize);
        this.blocks = mem_realloc(
            this.blocks as *mut _,
            (this.blockCount as usize).wrapping_mul(std::mem::size_of::<*mut libc::c_void>()),
        ) as *mut *mut libc::c_void;
        let newBlock: *mut libc::c_void =
            mem_alloc((this.cellSize).wrapping_mul(this.blockSize) as usize);
        let fresh1 = &mut (*(this.blocks).offset(newBlockIndex as isize));
        *fresh1 = newBlock;
        let mut prev: *mut *mut libc::c_void = &mut this.freeList;
        let mut pCurr: *mut libc::c_char = newBlock as *mut libc::c_char;
        let mut i: u32 = 0;
        while i < this.blockSize {
            *prev = pCurr as *mut _;
            prev = pCurr as *mut *mut libc::c_void;
            pCurr = pCurr.offset(this.cellSize as isize);
            i = i.wrapping_add(1);
        }
        *prev = std::ptr::null_mut();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn MemPool_Create(cellSize: u32, blockSize: u32) -> *mut MemPool {
    unsafe {
        let this = MemNew!(MemPool);
        (*this).size = 0;
        (*this).capacity = 0;
        (*this).freeList = std::ptr::null_mut();
        (*this).cellSize = cellSize;
        (*this).blockSize = blockSize;
        (*this).blockCount = 0;
        (*this).blocks = std::ptr::null_mut();
        this
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn MemPool_CreateAuto(elemSize: u32) -> *mut MemPool {
    unsafe { MemPool_Create(elemSize, (0x1000_u32).wrapping_div(elemSize)) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn MemPool_Free(this: *mut MemPool) {
    unsafe {
        let mut i: u16 = 0;
        while (i as i32) < (*this).blockCount as i32 {
            mem_free(*((*this).blocks).offset(i as isize));
            i = i.wrapping_add(1);
        }
        mem_free((*this).blocks as *const _);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn MemPool_Alloc(this: &mut MemPool) -> *mut libc::c_void {
    unsafe {
        if (this.size == this.capacity) as libc::c_long != 0 {
            MemPool_Grow(this);
        }
        let freeCell: *mut libc::c_void = this.freeList;
        this.freeList = *(freeCell as *mut *mut libc::c_void);
        this.size = (this.size).wrapping_add(1);
        mem_zero(freeCell, this.cellSize as usize);
        freeCell
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn MemPool_Clear(this: &mut MemPool) {
    this.size = 0;
    this.freeList = std::ptr::null_mut();
    let mut prev: *mut *mut libc::c_void = &mut this.freeList;
    let mut i: u32 = 0;
    unsafe {
        while i < this.blockCount as u32 {
            let mut pCurr: *mut libc::c_char =
                *(this.blocks).offset(i as isize) as *mut libc::c_char;
            let mut j: u32 = 0;
            while j < this.blockSize {
                *prev = pCurr as *mut _;
                prev = pCurr as *mut *mut libc::c_void;
                pCurr = pCurr.offset(this.cellSize as isize);
                j = j.wrapping_add(1);
            }
            i = i.wrapping_add(1);
        }
        *prev = std::ptr::null_mut();
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn MemPool_Dealloc(this: &mut MemPool, ptr: *mut libc::c_void) {
    let fresh2 = unsafe { &mut (*(ptr as *mut *mut libc::c_void)) };
    *fresh2 = this.freeList;
    this.freeList = ptr;
    this.size = (this.size).wrapping_sub(1);
}

#[unsafe(no_mangle)]
pub extern "C" fn MemPool_GetCapacity(this: &mut MemPool) -> u32 {
    this.capacity
}

#[unsafe(no_mangle)]
pub extern "C" fn MemPool_GetSize(this: &mut MemPool) -> u32 {
    this.size
}
