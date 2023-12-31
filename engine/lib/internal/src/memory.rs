#![allow(non_snake_case)]

#[inline]
pub unsafe extern "C" fn MemAlloc(size: usize) -> *mut libc::c_void {
    libc::malloc(size)
}

#[inline]
pub unsafe extern "C" fn MemAllocZero(size: usize) -> *mut libc::c_void {
    libc::calloc(1, size)
}

#[inline]
pub unsafe extern "C" fn MemFree(ptr: *const libc::c_void) {
    libc::free(ptr as *mut _);
}

#[inline]
pub unsafe extern "C" fn MemRealloc(ptr: *mut libc::c_void, newSize: usize) -> *mut libc::c_void {
    libc::realloc(ptr, newSize)
}

#[inline]
pub unsafe extern "C" fn MemCpy(dst: *mut libc::c_void, src: *const libc::c_void, size: usize) {
    libc::memcpy(dst, src, size);
}

#[inline]
pub unsafe extern "C" fn MemMove(dst: *mut libc::c_void, src: *const libc::c_void, size: usize) {
    libc::memmove(dst, src, size);
}

#[inline]
pub unsafe extern "C" fn MemZero(dst: *mut libc::c_void, size: usize) {
    libc::memset(dst, 0, size);
}

#[inline]
pub unsafe extern "C" fn MemSet(dst: *mut libc::c_void, value: i32, size: usize) {
    libc::memset(dst, value, size);
}

#[inline]
pub unsafe extern "C" fn StrAlloc(len: usize) -> *mut libc::c_char {
    libc::malloc(len) as *mut libc::c_char
}

#[inline]
pub unsafe extern "C" fn StrFree(s: *const libc::c_char) {
    libc::free(s as *mut _);
}

#[inline]
pub unsafe extern "C" fn StrDup(s: *const libc::c_char) -> *const libc::c_char {
    if s.is_null() {
        return std::ptr::null();
    }
    let len: usize = (StrLen(s)).wrapping_add(1);
    let buf: *mut libc::c_char = StrAlloc(len);
    libc::memcpy(buf as *mut _, s as *const _, len);
    buf as *const libc::c_char
}

#[inline]
pub unsafe extern "C" fn StrLen(mut s: *const libc::c_char) -> usize {
    if s.is_null() {
        return 0;
    }
    let begin: *const libc::c_char = s;
    while *s != 0 {
        s = s.offset(1);
    }
    s.offset_from(begin) as libc::c_long as usize
}
