use ::libc;
use super::internal::Memory::*;
use std::ffi::VaListImpl;
use super::internal::Memory::*;
extern "C" {
    pub type __sFILEX;
    fn vsnprintf(
        _: *mut libc::c_char,
        _: libc::c_ulong,
        _: *const libc::c_char,
        _: __builtin_va_list,
    ) -> libc::c_int;
    fn puts(_: *const libc::c_char) -> libc::c_int;
    static mut __stdoutp: *mut FILE;
    fn fflush(_: *mut FILE) -> libc::c_int;
    fn fprintf(_: *mut FILE, _: *const libc::c_char, _: ...) -> libc::c_int;
    fn free(_: *mut libc::c_void);
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn abort() -> !;
}
pub type __builtin_va_list = *mut libc::c_char;
pub type __int64_t = libc::c_longlong;
pub type __darwin_size_t = libc::c_ulong;
pub type __darwin_off_t = __int64_t;
pub type size_t = __darwin_size_t;
pub type cstr = *const libc::c_char;
pub type va_list = __builtin_va_list;
pub type FILE = __sFILE;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sFILE {
    pub _p: *mut libc::c_uchar,
    pub _r: libc::c_int,
    pub _w: libc::c_int,
    pub _flags: libc::c_short,
    pub _file: libc::c_short,
    pub _bf: __sbuf,
    pub _lbfsize: libc::c_int,
    pub _cookie: *mut libc::c_void,
    pub _close: Option::<unsafe extern "C" fn(*mut libc::c_void) -> libc::c_int>,
    pub _read: Option::<
        unsafe extern "C" fn(
            *mut libc::c_void,
            *mut libc::c_char,
            libc::c_int,
        ) -> libc::c_int,
    >,
    pub _seek: Option::<
        unsafe extern "C" fn(*mut libc::c_void, fpos_t, libc::c_int) -> fpos_t,
    >,
    pub _write: Option::<
        unsafe extern "C" fn(
            *mut libc::c_void,
            *const libc::c_char,
            libc::c_int,
        ) -> libc::c_int,
    >,
    pub _ub: __sbuf,
    pub _extra: *mut __sFILEX,
    pub _ur: libc::c_int,
    pub _ubuf: [libc::c_uchar; 3],
    pub _nbuf: [libc::c_uchar; 1],
    pub _lb: __sbuf,
    pub _blksize: libc::c_int,
    pub _offset: fpos_t,
}
pub type fpos_t = __darwin_off_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct __sbuf {
    pub _base: *mut libc::c_uchar,
    pub _size: libc::c_int,
}

unsafe extern "C" fn Fatal_Output(mut message: cstr) {
    puts(message);
    abort();
}

#[no_mangle]
pub unsafe extern "C" fn Fatal(mut format: cstr, mut args: ...) {
    let mut args_0: va_list = 0 as *mut libc::c_char;
    args_0 = &args as *const VaListImpl as va_list;
    let mut len: libc::c_int = vsnprintf(
        0 as *mut libc::c_char,
        0 as libc::c_int as libc::c_ulong,
        format,
        args_0,
    ) + 1 as libc::c_int;
    let mut message: *mut libc::c_char = MemAlloc(
        (::core::mem::size_of::<libc::c_char>())
            .wrapping_mul(len as usize),
    ) as *mut libc::c_char;
    args_0 = &args as *const VaListImpl as va_list;
    vsnprintf(message, len as libc::c_ulong, format, args_0);
    Fatal_Output(message as cstr);
}

#[no_mangle]
pub unsafe extern "C" fn Warn(mut format: cstr, mut args: ...) {
    let mut args_0: va_list = 0 as *mut libc::c_char;
    args_0 = &args as *const VaListImpl as va_list;
    let mut len: libc::c_int = vsnprintf(
        0 as *mut libc::c_char,
        0 as libc::c_int as libc::c_ulong,
        format,
        args_0,
    ) + 1 as libc::c_int;
    let mut message: *mut libc::c_char = MemAlloc(
        (::core::mem::size_of::<libc::c_char>())
            .wrapping_mul(len as usize),
    ) as *mut libc::c_char;
    args_0 = &args as *const VaListImpl as va_list;
    vsnprintf(message, len as libc::c_ulong, format, args_0);
    fprintf(__stdoutp, b"%s\n\0" as *const u8 as *const libc::c_char, message);
    fflush(__stdoutp);
    free(message as *mut libc::c_void);
}
