#![allow(non_snake_case)] // TODO: remove this and fix all warnings
#![allow(unsafe_code)] // TODO: refactor

#[no_mangle]
pub unsafe extern "C" fn Memory_Alloc(size: usize) -> *mut libc::c_void {
    libc::malloc(size)
}

#[no_mangle]
pub unsafe extern "C" fn Memory_Calloc(n: usize, size: usize) -> *mut libc::c_void {
    libc::calloc(n, size)
}

#[no_mangle]
pub unsafe extern "C" fn Memory_Free(ptr: *mut libc::c_void) {
    libc::free(ptr);
}

#[no_mangle]
pub unsafe extern "C" fn Memory_MemCopy(
    dst: *mut libc::c_void,
    src: *const libc::c_void,
    size: usize,
) {
    libc::memcpy(dst, src, size);
}

#[no_mangle]
pub unsafe extern "C" fn Memory_MemMove(
    dst: *mut libc::c_void,
    src: *const libc::c_void,
    size: usize,
) {
    libc::memmove(dst, src, size);
}

#[no_mangle]
pub unsafe extern "C" fn Memory_Realloc(
    ptr: *mut libc::c_void,
    new_size: usize,
) -> *mut libc::c_void {
    libc::realloc(ptr, new_size)
}
