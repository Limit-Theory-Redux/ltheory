use libc;

extern "C" {
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn calloc(_: libc::c_ulong, _: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn realloc(_: *mut libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
}

#[inline]
pub unsafe extern "C" fn MemAlloc(mut size: libc::size_t) -> *mut libc::c_void {
    return malloc(size as libc::c_ulong);
}

#[inline]
pub unsafe extern "C" fn MemAllocZero(mut size: libc::size_t) -> *mut libc::c_void {
    return calloc(1 as libc::c_int as libc::c_ulong, size as libc::c_ulong);
}

#[inline]
pub unsafe extern "C" fn MemFree(mut ptr: *const libc::c_void) {
    free(ptr as *mut libc::c_void);
}

#[inline]
pub unsafe extern "C" fn MemRealloc(
    mut ptr: *mut libc::c_void,
    mut newSize: libc::size_t,
) -> *mut libc::c_void {
    return realloc(ptr, newSize as libc::c_ulong);
}