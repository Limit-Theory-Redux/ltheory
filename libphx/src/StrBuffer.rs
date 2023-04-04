use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use libc;
use std::ffi::VaListImpl;

pub type __builtin_va_list = *mut libc::c_char;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct StrBuffer {
    pub data: *mut libc::c_char,
    pub size: u32,
    pub capacity: u32,
}
pub type va_list = __builtin_va_list;

#[inline]
unsafe extern "C" fn StrBuffer_GrowTo(this: *mut StrBuffer, newSize: u32) {
    if (newSize > (*this).capacity) as libc::c_long != 0 {
        while (*this).capacity < newSize {
            (*this).capacity = (*this).capacity.wrapping_mul(2);
        }
        (*this).data = MemRealloc(
            (*this).data as *mut _,
            ((*this).capacity).wrapping_add(1) as usize,
        ) as *mut libc::c_char;
        MemSet(
            ((*this).data).offset((*this).size as isize) as *mut _,
            0,
            ((*this).capacity)
                .wrapping_add(1)
                .wrapping_sub((*this).size) as usize,
        );
    }
}

#[inline]
unsafe extern "C" fn StrBuffer_AppendData(
    this: *mut StrBuffer,
    data: *const libc::c_void,
    len: u32,
) {
    StrBuffer_GrowTo(this, ((*this).size).wrapping_add(len));
    MemCpy(
        ((*this).data).offset((*this).size as isize) as *mut _,
        data,
        len as usize,
    );
    (*this).size = (*this).size.wrapping_add(len);
}

#[no_mangle]
pub unsafe extern "C" fn StrBuffer_Create(capacity: u32) -> *mut StrBuffer {
    let this = MemNew!(StrBuffer);
    (*this).data = MemAllocZero(capacity.wrapping_add(1) as usize) as *mut libc::c_char;
    (*this).size = 0;
    (*this).capacity = capacity;
    this
}

#[no_mangle]
pub unsafe extern "C" fn StrBuffer_FromStr(s: *const libc::c_char) -> *mut StrBuffer {
    let len: u32 = StrLen(s) as u32;
    let this: *mut StrBuffer = StrBuffer_Create(len);
    (*this).size = len;
    MemCpy((*this).data as *mut _, s as *const _, len as usize);
    this
}

#[no_mangle]
pub unsafe extern "C" fn StrBuffer_Free(this: *mut StrBuffer) {
    MemFree((*this).data as *const _);
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn StrBuffer_Append(this: *mut StrBuffer, other: *mut StrBuffer) {
    StrBuffer_AppendData(this, (*other).data as *const _, (*other).size);
}

#[no_mangle]
pub unsafe extern "C" fn StrBuffer_AppendStr(this: *mut StrBuffer, other: *const libc::c_char) {
    StrBuffer_AppendData(this, other as *const _, StrLen(other) as u32);
}

#[inline]
unsafe extern "C" fn StrBuffer_SetImpl(
    this: *mut StrBuffer,
    format: *const libc::c_char,
    args: va_list,
) -> i32 {
    let newSize: i32 = libc::snprintf(
        (*this).data,
        ((*this).capacity).wrapping_add(1) as usize,
        format,
        args,
    );
    if (newSize as u32 <= (*this).capacity) as libc::c_long != 0 {
        (*this).size = newSize as u32;
        0
    } else {
        (newSize as u32).wrapping_sub((*this).capacity) as i32
    }
}

#[no_mangle]
pub unsafe extern "C" fn StrBuffer_Set(
    this: *mut StrBuffer,
    format: *const libc::c_char,
    args: ...
) {
    let mut args_0: va_list = std::ptr::null_mut();
    args_0 = &args as *const VaListImpl as va_list;
    let mut neededSpace: i32 = StrBuffer_SetImpl(this, format, args_0);
    if (neededSpace > 0) as libc::c_long != 0 {
        StrBuffer_GrowTo(this, ((*this).capacity).wrapping_add(neededSpace as u32));
        // let mut args2: va_list = std::ptr::null_mut();
        // args2 = &args as *const VaListImpl as va_list;
        neededSpace = StrBuffer_SetImpl(this, format, args_0);
    }
}

#[no_mangle]
pub unsafe extern "C" fn StrBuffer_Clone(other: *mut StrBuffer) -> *mut StrBuffer {
    let this: *mut StrBuffer = StrBuffer_Create((*other).size);
    MemCpy(
        (*this).data as *mut _,
        (*other).data as *const _,
        (*other).size as usize,
    );
    (*this).size = (*other).size;
    this
}

#[no_mangle]
pub unsafe extern "C" fn StrBuffer_GetData(this: *mut StrBuffer) -> *const libc::c_char {
    (*this).data as *const libc::c_char
}
