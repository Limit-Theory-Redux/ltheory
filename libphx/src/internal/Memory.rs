use crate::Math::Vec3;
use libc;

pub use crate::Error::Error;

macro_rules! MemNew {
    ($x:ty) => {
        MemAlloc(std::mem::size_of::<$x>()) as *mut $x
    };
}
macro_rules! MemNewZero {
    ($x:ty) => {
        MemAllocZero(std::mem::size_of::<$x>()) as *mut $x
    };
}
macro_rules! MemNewArray {
    ($x:ty, $s:expr) => {
        MemAlloc(std::mem::size_of::<$x>().wrapping_mul($s as usize)) as *mut $x
    };
}
macro_rules! MemNewArrayZero {
    ($x:ty, $s:expr) => {
        MemAllocZero(std::mem::size_of::<$x>().wrapping_mul($s as usize)) as *mut $x
    };
}
macro_rules! MemDelete {
    ($v:ident) => {
        $v.drop_in_place();
        MemFree($v as *mut _)
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

#[inline]
pub unsafe extern "C" fn StrEqual(a: *const libc::c_char, b: *const libc::c_char) -> bool {
    libc::strcmp(a, b) == 0
}

#[inline]
pub unsafe extern "C" fn StrFormat(fmt: *const libc::c_char, mut args: ...) -> *const libc::c_char {
    let mut s = String::new();
    let _ = printf_compat::format(
        fmt,
        args.as_va_list(),
        printf_compat::output::fmt_write(&mut s),
    );
    let mut mem = libc::malloc(s.len() + 1) as *mut libc::c_char;
    libc::memcpy(mem as *mut _, s.as_bytes().as_ptr() as *const _, s.len());
    *mem.add(s.len()) = 0;
    mem
}

#[inline]
pub unsafe extern "C" fn StrReplace(
    mut s: *const libc::c_char,
    search: *const libc::c_char,
    mut replace: *const libc::c_char,
) -> *const libc::c_char {
    let mut result: *mut libc::c_char = std::ptr::null_mut();
    let mut ins: *mut libc::c_char = std::ptr::null_mut();
    let mut tmp: *mut libc::c_char = std::ptr::null_mut();
    let mut len_search: usize = 0;
    let mut len_replace: usize = 0;
    let mut len_front: usize = 0;
    let mut count: usize = 0;
    if s.is_null() || search.is_null() {
        return std::ptr::null();
    }
    len_search = StrLen(search);
    if len_search == 0 {
        return std::ptr::null();
    }
    if replace.is_null() {
        replace = b"\0" as *const u8 as *const libc::c_char;
    }
    len_replace = StrLen(replace);
    ins = s as *mut libc::c_char;
    count = 0;
    loop {
        tmp = libc::strstr(ins, search);
        if tmp.is_null() {
            break;
        }
        ins = tmp.add(len_search);
        count = count.wrapping_add(1);
    }
    result = StrAlloc(
        (StrLen(s))
            .wrapping_add(len_replace.wrapping_sub(len_search).wrapping_mul(count))
            .wrapping_add(1),
    );
    tmp = result;
    loop {
        let fresh0 = count;
        count = count.wrapping_sub(1);
        if !(fresh0 != 0) {
            break;
        }
        ins = libc::strstr(s, search);
        len_front = ins.offset_from(s) as libc::c_long as usize;
        tmp = (libc::strncpy(tmp, s, len_front)).add(len_front);
        tmp = (libc::strcpy(tmp, replace)).add(len_replace);
        s = s.add(len_front.wrapping_add(len_search));
    }
    libc::strcpy(tmp, s);
    result as *const libc::c_char
}

#[inline]
pub unsafe extern "C" fn StrFind(
    s: *const libc::c_char,
    sub: *const libc::c_char,
) -> *const libc::c_char {
    libc::strstr(s, sub) as *const libc::c_char
}

#[inline]
pub unsafe extern "C" fn StrSubStr(
    mut begin: *const libc::c_char,
    end: *const libc::c_char,
) -> *const libc::c_char {
    let mut len: usize = end.offset_from(begin) as libc::c_long as usize;
    let mut result: *mut libc::c_char = StrAlloc(len.wrapping_add(1));
    let mut pResult: *mut libc::c_char = result;
    while begin != end {
        let fresh1 = begin;
        begin = begin.offset(1);
        let fresh2 = pResult;
        pResult = pResult.offset(1);
        *fresh2 = *fresh1;
    }
    *result.add(len) = 0 as libc::c_char;
    result as *const libc::c_char
}

#[inline]
pub unsafe extern "C" fn StrSub(
    mut s: *const libc::c_char,
    begin: *const libc::c_char,
    mut end: *const libc::c_char,
    mut replace: *const libc::c_char,
) -> *const libc::c_char {
    let mut len: usize = begin
        .add((StrLen(s)).wrapping_add(StrLen(replace)))
        .offset_from(end) as libc::c_long as usize;
    let mut result: *mut libc::c_char = StrAlloc(len.wrapping_add(1));
    let mut pResult: *mut libc::c_char = result;
    while s != begin {
        let fresh3 = s;
        s = s.offset(1);
        let fresh4 = pResult;
        pResult = pResult.offset(1);
        *fresh4 = *fresh3;
    }
    while *replace != 0 {
        let fresh5 = replace;
        replace = replace.offset(1);
        let fresh6 = pResult;
        pResult = pResult.offset(1);
        *fresh6 = *fresh5;
    }
    while *end != 0 {
        let fresh7 = end;
        end = end.offset(1);
        let fresh8 = pResult;
        pResult = pResult.offset(1);
        *fresh8 = *fresh7;
    }
    *pResult = 0 as libc::c_char;
    result as *const libc::c_char
}

#[inline]
pub unsafe extern "C" fn StrAdd(
    mut a: *const libc::c_char,
    mut b: *const libc::c_char,
) -> *const libc::c_char {
    let mut buf: *mut libc::c_char = StrAlloc((StrLen(a)).wrapping_add(StrLen(b)).wrapping_add(1));
    let mut cur: *mut libc::c_char = buf;
    while *a != 0 {
        let fresh9 = a;
        a = a.offset(1);
        let fresh10 = cur;
        cur = cur.offset(1);
        *fresh10 = *fresh9;
    }
    while *b != 0 {
        let fresh11 = b;
        b = b.offset(1);
        let fresh12 = cur;
        cur = cur.offset(1);
        *fresh12 = *fresh11;
    }
    *cur = 0 as libc::c_char;
    buf as *const libc::c_char
}

#[inline]
pub unsafe extern "C" fn StrAdd3(
    mut a: *const libc::c_char,
    mut b: *const libc::c_char,
    mut c: *const libc::c_char,
) -> *const libc::c_char {
    let mut buf: *mut libc::c_char = StrAlloc(
        (StrLen(a))
            .wrapping_add(StrLen(b))
            .wrapping_add(StrLen(c))
            .wrapping_add(1),
    );
    let mut cur: *mut libc::c_char = buf;
    while *a != 0 {
        let fresh0 = a;
        a = a.offset(1);
        let fresh1 = cur;
        cur = cur.offset(1);
        *fresh1 = *fresh0;
    }
    while *b != 0 {
        let fresh2 = b;
        b = b.offset(1);
        let fresh3 = cur;
        cur = cur.offset(1);
        *fresh3 = *fresh2;
    }
    while *c != 0 {
        let fresh4 = c;
        c = c.offset(1);
        let fresh5 = cur;
        cur = cur.offset(1);
        *fresh5 = *fresh4;
    }
    *cur = 0 as libc::c_char;
    buf as *const libc::c_char
}
