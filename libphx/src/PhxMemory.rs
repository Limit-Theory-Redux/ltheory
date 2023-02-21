use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;

#[no_mangle]
pub unsafe extern "C" fn Memory_Alloc(mut size: libc::size_t) -> *mut libc::c_void {
    return libc::malloc(size);
}

#[no_mangle]
pub unsafe extern "C" fn Memory_Calloc(
    mut n: libc::size_t,
    mut size: libc::size_t,
) -> *mut libc::c_void {
    return libc::calloc(n, size);
}

#[no_mangle]
pub unsafe extern "C" fn Memory_Free(mut ptr: *mut libc::c_void) {
    libc::free(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn Memory_MemCopy(
    mut dst: *mut libc::c_void,
    mut src: *const libc::c_void,
    mut size: libc::size_t,
) {
    libc::memcpy(dst, src, size);
}

#[no_mangle]
pub unsafe extern "C" fn Memory_MemMove(
    mut dst: *mut libc::c_void,
    mut src: *const libc::c_void,
    mut size: libc::size_t,
) {
    libc::memmove(dst, src, size);
}

#[no_mangle]
pub unsafe extern "C" fn Memory_Realloc(
    mut ptr: *mut libc::c_void,
    mut newSize: libc::size_t,
) -> *mut libc::c_void {
    return libc::realloc(ptr, newSize);
}
