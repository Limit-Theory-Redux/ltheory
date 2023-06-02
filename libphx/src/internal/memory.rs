pub use crate::error::Error;

#[macro_export]
macro_rules! MemNew {
    ($x:ty) => {
        crate::internal::MemAlloc(std::mem::size_of::<$x>()) as *mut $x
    };
}

#[macro_export]
macro_rules! MemNewZero {
    ($x:ty) => {
        crate::internal::MemAllocZero(std::mem::size_of::<$x>()) as *mut $x
    };
}

#[macro_export]
macro_rules! MemNewArray {
    ($x:ty, $s:expr) => {
        crate::internal::MemAlloc(std::mem::size_of::<$x>().wrapping_mul($s as usize)) as *mut $x
    };
}

#[macro_export]
macro_rules! MemNewArrayZero {
    ($x:ty, $s:expr) => {
        crate::internal::MemAllocZero(std::mem::size_of::<$x>().wrapping_mul($s as usize))
            as *mut $x
    };
}

#[macro_export]
macro_rules! MemDelete {
    ($v:ident) => {
        $v.drop_in_place();
        crate::internal::MemFree($v as *mut _)
    };
}

pub(crate) use MemDelete;
pub(crate) use MemNew;
pub(crate) use MemNewArray;
pub(crate) use MemNewArrayZero;
pub(crate) use MemNewZero;

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
    let mut len: usize = (StrLen(s)).wrapping_add(1);
    let mut buf: *mut libc::c_char = StrAlloc(len);
    libc::memcpy(buf as *mut _, s as *const _, len);
    buf as *const libc::c_char
}

#[inline]
pub unsafe extern "C" fn StrLen(mut s: *const libc::c_char) -> usize {
    if s.is_null() {
        return 0;
    }
    let mut begin: *const libc::c_char = s;
    while *s != 0 {
        s = s.offset(1);
    }
    s.offset_from(begin) as libc::c_long as usize
}
