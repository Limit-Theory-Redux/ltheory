use ::libc;
use super::internal::Memory::*;

extern "C" {
    fn Hash_XX64(buf: *const libc::c_void, len: libc::c_int, seed: uint64) -> uint64;
}

pub type uint32_t = libc::c_uint;
pub type uint64_t = libc::c_ulonglong;
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
pub type uint32 = uint32_t;
pub type uint64 = uint64_t;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HashMap {
    pub elems: *mut Node,
    pub size: uint32,
    pub capacity: uint32,
    pub mask: uint32,
    pub keySize: uint32,
    pub maxProbe: uint32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Node {
    pub hash: uint64,
    pub value: *mut libc::c_void,
}
pub type ValueForeach = Option::<
    unsafe extern "C" fn(*mut libc::c_void, *mut libc::c_void) -> (),
>;
#[inline]
unsafe extern "C" fn Hash(mut key: *const libc::c_void, mut len: uint32) -> uint64 {
    return Hash_XX64(key, len as libc::c_int, 0 as libc::c_ulonglong);
}
#[no_mangle]
pub unsafe extern "C" fn HashMap_Create(
    mut keySize: uint32,
    mut capacity: uint32,
) -> *mut HashMap {
    let mut logCapacity: uint32 = 0 as libc::c_int as uint32;
    while capacity > 1 as libc::c_int as libc::c_uint {
        capacity = (capacity as libc::c_uint)
            .wrapping_div(2 as libc::c_int as libc::c_uint) as uint32 as uint32;
        logCapacity = logCapacity.wrapping_add(1);
    }
    capacity = ((1 as libc::c_int) << logCapacity) as uint32;
    let mut self_0: *mut HashMap = MemAlloc(
        ::core::mem::size_of::<HashMap>() as usize,
    ) as *mut HashMap;
    (*self_0)
        .elems = MemAllocZero(
        (::core::mem::size_of::<Node>())
            .wrapping_mul(capacity as usize),
    ) as *mut Node;
    (*self_0).size = 0 as libc::c_int as uint32;
    (*self_0).capacity = capacity;
    (*self_0).mask = (((1 as libc::c_int) << logCapacity) - 1 as libc::c_int) as uint32;
    (*self_0).keySize = keySize;
    (*self_0).maxProbe = logCapacity;
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn HashMap_Free(mut self_0: *mut HashMap) {
    MemFree((*self_0).elems as *const libc::c_void);
    MemFree(self_0 as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn HashMap_Foreach(
    mut self_0: *mut HashMap,
    mut fn_0: ValueForeach,
    mut userData: *mut libc::c_void,
) {
    let mut i: uint32 = 0 as libc::c_int as uint32;
    while i < (*self_0).capacity {
        let mut node: *mut Node = ((*self_0).elems).offset(i as isize);
        if !((*node).value).is_null() {
            fn_0.expect("non-null function pointer")((*node).value, userData);
        }
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn HashMap_Get(
    mut self_0: *mut HashMap,
    mut key: *const libc::c_void,
) -> *mut libc::c_void {
    return HashMap_GetRaw(self_0, Hash(key, (*self_0).keySize));
}
#[no_mangle]
pub unsafe extern "C" fn HashMap_GetRaw(
    mut self_0: *mut HashMap,
    mut hash: uint64,
) -> *mut libc::c_void {
    let mut index: uint32 = 0 as libc::c_int as uint32;
    let mut node: *mut Node = ((*self_0).elems)
        .offset(
            (hash.wrapping_add(index as libc::c_ulonglong)
                & (*self_0).mask as libc::c_ulonglong) as isize,
        );
    while !((*node).value).is_null() && index < (*self_0).maxProbe {
        if (*node).hash == hash {
            return (*node).value;
        }
        index = index.wrapping_add(1);
        node = ((*self_0).elems)
            .offset(
                (hash.wrapping_add(index as libc::c_ulonglong)
                    & (*self_0).mask as libc::c_ulonglong) as isize,
            );
    }
    return 0 as *mut libc::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn HashMap_Resize(mut self_0: *mut HashMap, mut capacity: uint32) {
    let mut other: *mut HashMap = HashMap_Create((*self_0).keySize, capacity);
    let mut i: uint32 = 0 as libc::c_int as uint32;
    while i < (*self_0).capacity {
        let mut node: *mut Node = ((*self_0).elems).offset(i as isize);
        if !((*node).value).is_null() {
            HashMap_SetRaw(other, (*node).hash, (*node).value);
        }
        i = i.wrapping_add(1);
    }
    MemFree((*self_0).elems as *const libc::c_void);
    *self_0 = *other;
}
#[no_mangle]
pub unsafe extern "C" fn HashMap_Set(
    mut self_0: *mut HashMap,
    mut key: *const libc::c_void,
    mut value: *mut libc::c_void,
) {
    HashMap_SetRaw(self_0, Hash(key, (*self_0).keySize), value);
}
#[no_mangle]
pub unsafe extern "C" fn HashMap_SetRaw(
    mut self_0: *mut HashMap,
    mut hash: uint64,
    mut value: *mut libc::c_void,
) {
    let mut index: uint32 = 0 as libc::c_int as uint32;
    let mut node: *mut Node = ((*self_0).elems)
        .offset(
            (hash.wrapping_add(index as libc::c_ulonglong)
                & (*self_0).mask as libc::c_ulonglong) as isize,
        );
    while !((*node).value).is_null() && index < (*self_0).maxProbe {
        if (*node).hash == hash {
            (*node).value = value;
            return;
        }
        index = index.wrapping_add(1);
        node = ((*self_0).elems)
            .offset(
                (hash.wrapping_add(index as libc::c_ulonglong)
                    & (*self_0).mask as libc::c_ulonglong) as isize,
            );
    }
    if index >= (*self_0).maxProbe {
        HashMap_Resize(
            self_0,
            ((*self_0).capacity).wrapping_mul(2 as libc::c_int as libc::c_uint),
        );
        HashMap_SetRaw(self_0, hash, value);
    } else {
        (*node).hash = hash;
        (*node).value = value;
        (*self_0).size = ((*self_0).size).wrapping_add(1);
    };
}
