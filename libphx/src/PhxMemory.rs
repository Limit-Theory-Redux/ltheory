use ::libc;
extern "C" {
    fn memmove(
        _: *mut libc::c_void,
        _: *const libc::c_void,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn memcpy(
        _: *mut libc::c_void,
        _: *const libc::c_void,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn calloc(_: libc::c_ulong, _: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn realloc(_: *mut libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
}
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
#[no_mangle]
pub unsafe extern "C" fn Memory_Alloc(mut size: size_t) -> *mut libc::c_void {
    return malloc(size);
}
#[no_mangle]
pub unsafe extern "C" fn Memory_Calloc(
    mut n: size_t,
    mut size: size_t,
) -> *mut libc::c_void {
    return calloc(n, size);
}
#[no_mangle]
pub unsafe extern "C" fn Memory_Free(mut ptr: *mut libc::c_void) {
    free(ptr);
}
#[no_mangle]
pub unsafe extern "C" fn Memory_MemCopy(
    mut dst: *mut libc::c_void,
    mut src: *const libc::c_void,
    mut size: size_t,
) {
    memcpy(dst, src, size);
}
#[no_mangle]
pub unsafe extern "C" fn Memory_MemMove(
    mut dst: *mut libc::c_void,
    mut src: *const libc::c_void,
    mut size: size_t,
) {
    memmove(dst, src, size);
}
#[no_mangle]
pub unsafe extern "C" fn Memory_Realloc(
    mut ptr: *mut libc::c_void,
    mut newSize: size_t,
) -> *mut libc::c_void {
    return realloc(ptr, newSize);
}
