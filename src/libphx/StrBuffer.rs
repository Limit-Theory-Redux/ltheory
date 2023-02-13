use std::ffi::VaListImpl;

use ::libc;
use super::internal::Memory::*;
use internal::Memory::*;
extern "C" {
    fn memcpy(
        _: *mut libc::c_void,
        _: *const libc::c_void,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn memset(
        _: *mut libc::c_void,
        _: libc::c_int,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn realloc(_: *mut libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn calloc(_: libc::c_ulong, _: libc::c_ulong) -> *mut libc::c_void;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn vsnprintf(
        _: *mut libc::c_char,
        _: libc::c_ulong,
        _: *const libc::c_char,
        _: __builtin_va_list,
    ) -> libc::c_int;
}
pub type __builtin_va_list = *mut libc::c_char;
pub type int32_t = libc::c_int;
pub type uint32_t = libc::c_uint;
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint32 = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StrBuffer {
    pub data: *mut libc::c_char,
    pub size: uint32,
    pub capacity: uint32,
}
pub type va_list = __builtin_va_list;
#[inline]
unsafe extern "C" fn MemSet(
    mut dst: *mut libc::c_void,
    mut value: libc::c_int,
    mut size: size_t,
) {
    memset(dst, value, size);
}


#[inline]
unsafe extern "C" fn MemCpy(
    mut dst: *mut libc::c_void,
    mut src: *const libc::c_void,
    mut size: size_t,
) {
    memcpy(dst, src, size);
}
#[inline]
unsafe extern "C" fn MemAllocZero(mut size: size_t) -> *mut libc::c_void {
    return calloc(1 as libc::c_int as libc::c_ulong, size);
}
#[inline]
unsafe extern "C" fn StrLen(mut s: cstr) -> size_t {
    if s.is_null() {
        return 0 as libc::c_int as size_t;
    }
    let mut begin: cstr = s;
    while *s != 0 {
        s = s.offset(1);
    }
    return s.offset_from(begin) as libc::c_long as size_t;
}
#[inline]
unsafe extern "C" fn StrBuffer_GrowTo(mut self_0: *mut StrBuffer, mut newSize: uint32) {
    if (newSize > (*self_0).capacity) as libc::c_int as libc::c_long != 0 {
        while (*self_0).capacity < newSize {
            (*self_0)
                .capacity = ((*self_0).capacity as libc::c_uint)
                .wrapping_mul(2 as libc::c_int as libc::c_uint) as uint32 as uint32;
        }
        (*self_0)
            .data = MemRealloc(
            (*self_0).data as *mut libc::c_void,
            ((*self_0).capacity).wrapping_add(1 as libc::c_int as libc::c_uint) as size_t,
        ) as *mut libc::c_char;
        MemSet(
            ((*self_0).data).offset((*self_0).size as isize) as *mut libc::c_void,
            0 as libc::c_int,
            ((*self_0).capacity)
                .wrapping_add(1 as libc::c_int as libc::c_uint)
                .wrapping_sub((*self_0).size) as size_t,
        );
    }
}
#[inline]
unsafe extern "C" fn StrBuffer_AppendData(
    mut self_0: *mut StrBuffer,
    mut data: *const libc::c_void,
    mut len: uint32,
) {
    StrBuffer_GrowTo(self_0, ((*self_0).size).wrapping_add(len));
    MemCpy(
        ((*self_0).data).offset((*self_0).size as isize) as *mut libc::c_void,
        data,
        len as size_t,
    );
    (*self_0)
        .size = ((*self_0).size as libc::c_uint).wrapping_add(len) as uint32 as uint32;
}
#[no_mangle]
pub unsafe extern "C" fn StrBuffer_Create(mut capacity: uint32) -> *mut StrBuffer {
    let mut self_0: *mut StrBuffer = MemAlloc(
        ::core::mem::size_of::<StrBuffer>() as libc::c_ulong,
    ) as *mut StrBuffer;
    (*self_0)
        .data = MemAllocZero(
        capacity.wrapping_add(1 as libc::c_int as libc::c_uint) as size_t,
    ) as *mut libc::c_char;
    (*self_0).size = 0 as libc::c_int as uint32;
    (*self_0).capacity = capacity;
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn StrBuffer_FromStr(
    mut s: *const libc::c_char,
) -> *mut StrBuffer {
    let mut len: uint32 = StrLen(s) as uint32;
    let mut self_0: *mut StrBuffer = StrBuffer_Create(len);
    (*self_0).size = len;
    MemCpy((*self_0).data as *mut libc::c_void, s as *const libc::c_void, len as size_t);
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn StrBuffer_Free(mut self_0: *mut StrBuffer) {
    MemFree((*self_0).data as *const libc::c_void);
    MemFree(self_0 as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn StrBuffer_Append(
    mut self_0: *mut StrBuffer,
    mut other: *mut StrBuffer,
) {
    StrBuffer_AppendData(self_0, (*other).data as *const libc::c_void, (*other).size);
}
#[no_mangle]
pub unsafe extern "C" fn StrBuffer_AppendStr(
    mut self_0: *mut StrBuffer,
    mut other: *const libc::c_char,
) {
    StrBuffer_AppendData(self_0, other as *const libc::c_void, StrLen(other) as uint32);
}
#[inline]
unsafe extern "C" fn StrBuffer_SetImpl(
    mut self_0: *mut StrBuffer,
    mut format: cstr,
    mut args: va_list,
) -> int32 {
    let mut newSize: int32 = vsnprintf(
        (*self_0).data,
        ((*self_0).capacity).wrapping_add(1 as libc::c_int as libc::c_uint)
            as libc::c_ulong,
        format,
        args,
    );
    if (newSize as uint32 <= (*self_0).capacity) as libc::c_int as libc::c_long != 0 {
        (*self_0).size = newSize as uint32;
        return 0 as libc::c_int;
    } else {
        return (newSize as libc::c_uint).wrapping_sub((*self_0).capacity) as int32
    };
}
#[no_mangle]
pub unsafe extern "C" fn StrBuffer_Set(
    mut self_0: *mut StrBuffer,
    mut format: cstr,
    mut args: ...
) {
    let mut args_0: va_list = 0 as *mut libc::c_char;
    args_0 = &args as *const VaListImpl as va_list;
    let mut neededSpace: int32 = StrBuffer_SetImpl(self_0, format, args_0);
    if (neededSpace > 0 as libc::c_int) as libc::c_int as libc::c_long != 0 {
        StrBuffer_GrowTo(
            self_0,
            ((*self_0).capacity).wrapping_add(neededSpace as libc::c_uint),
        );
        let mut args2: va_list = 0 as *mut libc::c_char;
        args2 = &args as *const VaListImpl as va_list;
        neededSpace = StrBuffer_SetImpl(self_0, format, args_0);
    }
}
#[no_mangle]
pub unsafe extern "C" fn StrBuffer_Clone(mut other: *mut StrBuffer) -> *mut StrBuffer {
    let mut self_0: *mut StrBuffer = StrBuffer_Create((*other).size);
    MemCpy(
        (*self_0).data as *mut libc::c_void,
        (*other).data as *const libc::c_void,
        (*other).size as size_t,
    );
    (*self_0).size = (*other).size;
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn StrBuffer_GetData(mut self_0: *mut StrBuffer) -> cstr {
    return (*self_0).data as cstr;
}
