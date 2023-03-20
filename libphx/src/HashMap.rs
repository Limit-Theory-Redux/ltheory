use crate::internal::Memory::*;
use crate::Common::*;
use crate::Hash::*;
use crate::Math::Vec3;
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
pub type ValueForeach = Option<unsafe extern "C" fn(*mut libc::c_void, *mut libc::c_void) -> ()>;

#[inline]
unsafe extern "C" fn Hash(mut key: *const libc::c_void, mut len: u32) -> u64 {
    Hash_XX64(key, len as i32, 0_u64)
}

#[no_mangle]
pub unsafe extern "C" fn HashMap_Create(mut keySize: u32, mut capacity: u32) -> *mut HashMap {
    let mut logCapacity: u32 = 0_u32;
    while capacity > 1_u32 {
        capacity = capacity.wrapping_div(2_u32);
        logCapacity = logCapacity.wrapping_add(1);
    }
    capacity = (1_i32 << logCapacity) as u32;
    let mut this = MemNew!(HashMap);
    (*this).elems = MemNewArrayZero!(Node, capacity);
    (*this).size = 0_u32;
    (*this).capacity = capacity;
    (*this).mask = ((1_i32 << logCapacity) - 1_i32) as u32;
    (*this).keySize = keySize;
    (*this).maxProbe = logCapacity;
    this
}

#[no_mangle]
pub unsafe extern "C" fn HashMap_Free(mut this: *mut HashMap) {
    MemFree((*this).elems as *const libc::c_void);
    MemFree(this as *const libc::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn HashMap_Foreach(
    mut this: *mut HashMap,
    mut fn_0: ValueForeach,
    mut userData: *mut libc::c_void,
) {
    let mut i: u32 = 0_u32;
    while i < (*this).capacity {
        let mut node: *mut Node = ((*this).elems).offset(i as isize);
        if !((*node).value).is_null() {
            fn_0.expect("non-null function pointer")((*node).value, userData);
        }
        i = i.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn HashMap_Get(
    mut this: *mut HashMap,
    mut key: *const libc::c_void,
) -> *mut libc::c_void {
    HashMap_GetRaw(this, Hash(key, (*this).keySize))
}

#[no_mangle]
pub unsafe extern "C" fn HashMap_GetRaw(
    mut this: *mut HashMap,
    mut hash: u64,
) -> *mut libc::c_void {
    let mut index: u32 = 0_u32;
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
pub unsafe extern "C" fn HashMap_Resize(mut this: *mut HashMap, mut capacity: u32) {
    let mut other: *mut HashMap = HashMap_Create((*this).keySize, capacity);
    let mut i: u32 = 0_u32;
    while i < (*this).capacity {
        let mut node: *mut Node = ((*this).elems).offset(i as isize);
        if !((*node).value).is_null() {
            HashMap_SetRaw(other, (*node).hash, (*node).value);
        }
        i = i.wrapping_add(1);
    }
    MemFree((*this).elems as *const libc::c_void);
    *this = *other;
}

#[no_mangle]
pub unsafe extern "C" fn HashMap_Set(
    mut this: *mut HashMap,
    mut key: *const libc::c_void,
    mut value: *mut libc::c_void,
) {
    HashMap_SetRaw(this, Hash(key, (*this).keySize), value);
}

#[no_mangle]
pub unsafe extern "C" fn HashMap_SetRaw(
    mut this: *mut HashMap,
    mut hash: u64,
    mut value: *mut libc::c_void,
) {
    let mut index: u32 = 0_u32;
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
        HashMap_Resize(this, ((*this).capacity).wrapping_mul(2_u32));
        HashMap_SetRaw(this, hash, value);
    } else {
        (*node).hash = hash;
        (*node).value = value;
        (*this).size = ((*this).size).wrapping_add(1);
    };
}
