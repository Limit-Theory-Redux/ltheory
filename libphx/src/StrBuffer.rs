use crate::internal::Memory::*;
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
unsafe extern "C" fn StrBuffer_GrowTo(mut this: *mut StrBuffer, mut newSize: u32) {
    if (newSize > (*this).capacity) as libc::c_long != 0 {
        while (*this).capacity < newSize {
            (*this).capacity = (*this).capacity.wrapping_mul(2_i32 as u32);
        }
        (*this).data = MemRealloc(
            (*this).data as *mut libc::c_void,
            ((*this).capacity).wrapping_add(1_i32 as u32) as usize,
        ) as *mut libc::c_char;
        MemSet(
            ((*this).data).offset((*this).size as isize) as *mut libc::c_void,
            0_i32,
            ((*this).capacity)
                .wrapping_add(1_i32 as u32)
                .wrapping_sub((*this).size) as usize,
        );
    }
}

#[inline]
unsafe extern "C" fn StrBuffer_AppendData(
    mut this: *mut StrBuffer,
    mut data: *const libc::c_void,
    mut len: u32,
) {
    StrBuffer_GrowTo(this, ((*this).size).wrapping_add(len));
    MemCpy(
        ((*this).data).offset((*this).size as isize) as *mut libc::c_void,
        data,
        len as usize,
    );
    (*this).size = (*this).size.wrapping_add(len);
}

#[no_mangle]
pub unsafe extern "C" fn StrBuffer_Create(mut capacity: u32) -> *mut StrBuffer {
    let mut this: *mut StrBuffer = MemAlloc(::core::mem::size_of::<StrBuffer>()) as *mut StrBuffer;
    (*this).data = MemAllocZero(capacity.wrapping_add(1_i32 as u32) as usize) as *mut libc::c_char;
    (*this).size = 0_i32 as u32;
    (*this).capacity = capacity;
    this
}

#[no_mangle]
pub unsafe extern "C" fn StrBuffer_FromStr(mut s: *const libc::c_char) -> *mut StrBuffer {
    let mut len: u32 = StrLen(s) as u32;
    let mut this: *mut StrBuffer = StrBuffer_Create(len);
    (*this).size = len;
    MemCpy(
        (*this).data as *mut libc::c_void,
        s as *const libc::c_void,
        len as usize,
    );
    this
}

#[no_mangle]
pub unsafe extern "C" fn StrBuffer_Free(mut this: *mut StrBuffer) {
    MemFree((*this).data as *const libc::c_void);
    MemFree(this as *const libc::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn StrBuffer_Append(mut this: *mut StrBuffer, mut other: *mut StrBuffer) {
    StrBuffer_AppendData(this, (*other).data as *const libc::c_void, (*other).size);
}

#[no_mangle]
pub unsafe extern "C" fn StrBuffer_AppendStr(
    mut this: *mut StrBuffer,
    mut other: *const libc::c_char,
) {
    StrBuffer_AppendData(this, other as *const libc::c_void, StrLen(other) as u32);
}

#[inline]
unsafe extern "C" fn StrBuffer_SetImpl(
    mut this: *mut StrBuffer,
    mut format: *const libc::c_char,
    mut args: va_list,
) -> i32 {
    let mut newSize: i32 = libc::snprintf(
        (*this).data,
        ((*this).capacity).wrapping_add(1) as usize,
        format,
        args,
    );
    if (newSize as u32 <= (*this).capacity) as libc::c_long != 0 {
        (*this).size = newSize as u32;
        0_i32
    } else {
        (newSize as u32).wrapping_sub((*this).capacity) as i32
    }
}

#[no_mangle]
pub unsafe extern "C" fn StrBuffer_Set(
    mut this: *mut StrBuffer,
    mut format: *const libc::c_char,
    mut args: ...
) {
    let mut args_0: va_list = std::ptr::null_mut();
    args_0 = &args as *const VaListImpl as va_list;
    let mut neededSpace: i32 = StrBuffer_SetImpl(this, format, args_0);
    if (neededSpace > 0_i32) as libc::c_long != 0 {
        StrBuffer_GrowTo(this, ((*this).capacity).wrapping_add(neededSpace as u32));
        let mut args2: va_list = std::ptr::null_mut();
        args2 = &args as *const VaListImpl as va_list;
        neededSpace = StrBuffer_SetImpl(this, format, args_0);
    }
}

#[no_mangle]
pub unsafe extern "C" fn StrBuffer_Clone(mut other: *mut StrBuffer) -> *mut StrBuffer {
    let mut this: *mut StrBuffer = StrBuffer_Create((*other).size);
    MemCpy(
        (*this).data as *mut libc::c_void,
        (*other).data as *const libc::c_void,
        (*other).size as usize,
    );
    (*this).size = (*other).size;
    this
}

#[no_mangle]
pub unsafe extern "C" fn StrBuffer_GetData(mut this: *mut StrBuffer) -> *const libc::c_char {
    (*this).data as *const libc::c_char
}
