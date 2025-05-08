#![allow(non_snake_case)] // TODO: remove this and fix all warnings
#![allow(unsafe_code)] // TODO: refactor

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Memory_Alloc(size: usize) -> *mut libc::c_void {
    unsafe { libc::malloc(size) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Memory_Calloc(n: usize, size: usize) -> *mut libc::c_void {
    unsafe { libc::calloc(n, size) }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Memory_Free(ptr: *mut libc::c_void) {
    unsafe { libc::free(ptr) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Memory_MemCopy(
    dst: *mut libc::c_void,
    src: *const libc::c_void,
    size: usize,
) {
    unsafe { libc::memcpy(dst, src, size) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Memory_MemMove(
    dst: *mut libc::c_void,
    src: *const libc::c_void,
    size: usize,
) {
    unsafe { libc::memmove(dst, src, size) };
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn Memory_Realloc(
    ptr: *mut libc::c_void,
    new_size: usize,
) -> *mut libc::c_void {
    unsafe { libc::realloc(ptr, new_size) }
}
