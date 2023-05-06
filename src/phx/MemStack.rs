use crate::phx::internal::Memory::*;
use crate::phx::Common::*;


#[derive(Copy, Clone)]
#[repr(C)]
pub struct MemStack {
    pub size: u32,
    pub capacity: u32,
    pub data: *mut libc::c_void,
}

#[no_mangle]
pub unsafe extern "C" fn MemStack_Create(capacity: u32) -> *mut MemStack {
    let this = MemNew!(MemStack);
    (*this).size = 0;
    (*this).capacity = capacity;
    (*this).data = MemAlloc(capacity as usize);
    this
}

#[no_mangle]
pub unsafe extern "C" fn MemStack_Free(this: *mut MemStack) {
    MemFree((*this).data);
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn MemStack_Alloc(this: &mut MemStack, size: u32) -> *mut libc::c_void {
    if (this.size).wrapping_add(size) > this.capacity {
        CFatal!("MemStack_Alloc: Allocation request exceeds remaining capacity");
    }
    let p: *mut libc::c_void =
        (this.data as *mut libc::c_char).offset(this.size as isize) as *mut _;
    this.size = this.size.wrapping_add(size);
    p
}

#[no_mangle]
pub extern "C" fn MemStack_Clear(this: &mut MemStack) {
    this.size = 0;
}

#[no_mangle]
pub unsafe extern "C" fn MemStack_Dealloc(this: &mut MemStack, size: u32) {
    if this.size < size {
        CFatal!("MemStack_Dealloc: Attempt to dealloc more memory than is allocated");
    }
    this.size = this.size.wrapping_sub(size);
}

#[no_mangle]
pub extern "C" fn MemStack_CanAlloc(this: &mut MemStack, size: u32) -> bool {
    (this.size).wrapping_add(size) <= this.capacity
}

#[no_mangle]
pub extern "C" fn MemStack_GetSize(this: &mut MemStack) -> u32 {
    this.size
}

#[no_mangle]
pub extern "C" fn MemStack_GetCapacity(this: &mut MemStack) -> u32 {
    this.capacity
}

#[no_mangle]
pub extern "C" fn MemStack_GetRemaining(this: &mut MemStack) -> u32 {
    (this.capacity).wrapping_sub(this.size)
}
