#![allow(clippy::missing_safety_doc)] // This file will be removed after refactoring

#[inline]
pub unsafe extern "C" fn mem_alloc(size: usize) -> *mut libc::c_void {
    libc::malloc(size)
}

#[inline]
pub unsafe extern "C" fn mem_free(ptr: *const libc::c_void) {
    libc::free(ptr as *mut _);
}

#[inline]
pub unsafe extern "C" fn mem_realloc(ptr: *mut libc::c_void, new_size: usize) -> *mut libc::c_void {
    libc::realloc(ptr, new_size)
}

#[inline]
pub unsafe extern "C" fn mem_zero(dst: *mut libc::c_void, size: usize) {
    libc::memset(dst, 0, size);
}
