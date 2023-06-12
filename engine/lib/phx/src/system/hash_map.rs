use super::*;
use crate::internal::*;
use crate::math::*;
use crate::*;

use libc;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HashMap {
    pub elems: *mut Node,
    pub size: u32,
    pub capacity: u32,
    pub mask: u32,
    pub keySize: u32,
    pub maxProbe: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Node {
    pub hash: u64,
    pub value: *mut libc::c_void,
}
pub type ValueForeach = Option<extern "C" fn(*mut libc::c_void, *mut libc::c_void) -> ()>;

#[inline]
unsafe extern "C" fn Hash(key: *const libc::c_void, len: u32) -> u64 {
    Hash_XX64(key, len as i32, 0)
}

#[no_mangle]
pub unsafe extern "C" fn HashMap_Create(keySize: u32, mut capacity: u32) -> *mut HashMap {
    let mut logCapacity: u32 = 0;
    while capacity > 1 {
        capacity = capacity.wrapping_div(2);
        logCapacity = logCapacity.wrapping_add(1);
    }
    capacity = (1 << logCapacity) as u32;
    let this = MemNew!(HashMap);
    (*this).elems = MemNewArrayZero!(Node, capacity);
    (*this).size = 0;
    (*this).capacity = capacity;
    (*this).mask = ((1 << logCapacity) - 1) as u32;
    (*this).keySize = keySize;
    (*this).maxProbe = logCapacity;
    this
}

#[no_mangle]
pub unsafe extern "C" fn HashMap_Free(this: *mut HashMap) {
    MemFree((*this).elems as *const _);
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn HashMap_Foreach(
    this: *mut HashMap,
    fn_0: ValueForeach,
    userData: *mut libc::c_void,
) {
    let mut i: u32 = 0;
    while i < (*this).capacity {
        let node: *mut Node = ((*this).elems).offset(i as isize);
        if !((*node).value).is_null() {
            fn_0.expect("non-null function pointer")((*node).value, userData);
        }
        i = i.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn HashMap_Get(
    this: *mut HashMap,
    key: *const libc::c_void,
) -> *mut libc::c_void {
    HashMap_GetRaw(this, Hash(key, (*this).keySize))
}

#[no_mangle]
pub unsafe extern "C" fn HashMap_GetRaw(this: *mut HashMap, hash: u64) -> *mut libc::c_void {
    let mut index: u32 = 0;
    let mut node: *mut Node =
        ((*this).elems).offset((hash.wrapping_add(index as u64) & (*this).mask as u64) as isize);
    while !((*node).value).is_null() && index < (*this).maxProbe {
        if (*node).hash == hash {
            return (*node).value;
        }
        index = index.wrapping_add(1);
        node = ((*this).elems)
            .offset((hash.wrapping_add(index as u64) & (*this).mask as u64) as isize);
    }
    std::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "C" fn HashMap_Resize(this: *mut HashMap, capacity: u32) {
    let other: *mut HashMap = HashMap_Create((*this).keySize, capacity);
    let mut i: u32 = 0;
    while i < (*this).capacity {
        let node: *mut Node = ((*this).elems).offset(i as isize);
        if !((*node).value).is_null() {
            HashMap_SetRaw(other, (*node).hash, (*node).value);
        }
        i = i.wrapping_add(1);
    }
    MemFree((*this).elems as *const _);
    *this = *other;
}

#[no_mangle]
pub unsafe extern "C" fn HashMap_Set(
    this: *mut HashMap,
    key: *const libc::c_void,
    value: *mut libc::c_void,
) {
    HashMap_SetRaw(this, Hash(key, (*this).keySize), value);
}

#[no_mangle]
pub unsafe extern "C" fn HashMap_SetRaw(this: *mut HashMap, hash: u64, value: *mut libc::c_void) {
    let mut index: u32 = 0;
    let mut node: *mut Node =
        ((*this).elems).offset((hash.wrapping_add(index as u64) & (*this).mask as u64) as isize);
    while !((*node).value).is_null() && index < (*this).maxProbe {
        if (*node).hash == hash {
            (*node).value = value;
            return;
        }
        index = index.wrapping_add(1);
        node = ((*this).elems)
            .offset((hash.wrapping_add(index as u64) & (*this).mask as u64) as isize);
    }
    if index >= (*this).maxProbe {
        HashMap_Resize(this, ((*this).capacity).wrapping_mul(2));
        HashMap_SetRaw(this, hash, value);
    } else {
        (*node).hash = hash;
        (*node).value = value;
        (*this).size = ((*this).size).wrapping_add(1);
    };
}
