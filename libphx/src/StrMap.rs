use crate::internal::Memory::*;
use crate::Hash::*;
use glam::Vec3;
use libc;

extern "C" {
    fn Fatal(_: *const libc::c_char, _: ...);
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct StrMap {
    pub capacity: u32,
    pub size: u32,
    pub data: *mut Node,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Node {
    pub key: *const libc::c_char,
    pub next: *mut Node,
    pub value: *mut libc::c_void,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct StrMapIter {
    pub map: *mut StrMap,
    pub node: *mut Node,
    pub slot: u32,
}

#[inline]
unsafe extern "C" fn Hash(mut key: *const libc::c_char) -> u64 {
    return Hash_XX64(key as *const libc::c_void, StrLen(key) as i32, 0_u64);
}

#[inline]
unsafe extern "C" fn StrMap_GetBucket(mut this: *mut StrMap, mut key: *const libc::c_char) -> *mut Node {
    return ((*this).data).offset((Hash(key)).wrapping_rem((*this).capacity as u64) as isize);
}

unsafe extern "C" fn StrMap_Grow(mut this: *mut StrMap) {
    let mut newMap: StrMap = StrMap {
        capacity: ((*this).capacity).wrapping_mul(2_i32 as u32),
        size: 0_i32 as u32,
        data: std::ptr::null_mut(),
    };
    newMap.data =
        MemAllocZero((::core::mem::size_of::<Node>()).wrapping_mul(newMap.capacity as usize))
            as *mut Node;
    let mut i: u32 = 0_i32 as u32;
    while i < (*this).capacity {
        let mut node: *mut Node = ((*this).data).offset(i as isize);
        if !((*node).key).is_null() {
            StrMap_Set(&mut newMap, (*node).key, (*node).value);
            StrFree((*node).key);
            node = (*node).next;
            while !node.is_null() {
                let mut next: *mut Node = (*node).next;
                StrMap_Set(&mut newMap, (*node).key, (*node).value);
                StrFree((*node).key);
                MemFree(node as *const libc::c_void);
                node = next;
            }
        }
        i = i.wrapping_add(1);
    }
    MemFree((*this).data as *const libc::c_void);
    *this = newMap;
}

#[no_mangle]
pub unsafe extern "C" fn StrMap_Create(mut capacity: u32) -> *mut StrMap {
    let mut this: *mut StrMap =
        MemAllocZero(::core::mem::size_of::<StrMap>()) as *mut StrMap;
    (*this).capacity = capacity;
    (*this).data =
        MemAllocZero((::core::mem::size_of::<Node>()).wrapping_mul(capacity as usize)) as *mut Node;
    return this;
}

#[no_mangle]
pub unsafe extern "C" fn StrMap_Free(mut this: *mut StrMap) {
    let mut i: u32 = 0_i32 as u32;
    while i < (*this).capacity {
        let mut node: *mut Node = ((*this).data).offset(i as isize);
        if !((*node).key).is_null() {
            StrFree((*node).key);
            node = (*node).next;
            while !node.is_null() {
                let mut next: *mut Node = (*node).next;
                StrFree((*node).key);
                MemFree(node as *const libc::c_void);
                node = next;
            }
        }
        i = i.wrapping_add(1);
    }
    MemFree((*this).data as *const libc::c_void);
    MemFree(this as *const libc::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn StrMap_FreeEx(
    mut this: *mut StrMap,
    mut freeFn: Option<unsafe extern "C" fn(*const libc::c_char, *mut libc::c_void) -> ()>,
) {
    let mut i: u32 = 0_i32 as u32;
    while i < (*this).capacity {
        let mut node: *mut Node = ((*this).data).offset(i as isize);
        if !((*node).key).is_null() {
            freeFn.expect("non-null function pointer")((*node).key, (*node).value);
            StrFree((*node).key);
            node = (*node).next;
            while !node.is_null() {
                let mut next: *mut Node = (*node).next;
                freeFn.expect("non-null function pointer")((*node).key, (*node).value);
                StrFree((*node).key);
                MemFree(node as *const libc::c_void);
                node = next;
            }
        }
        i = i.wrapping_add(1);
    }
    MemFree((*this).data as *const libc::c_void);
    MemFree(this as *const libc::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn StrMap_Get(mut this: *mut StrMap, mut key: *const libc::c_char) -> *mut libc::c_void {
    let mut node: *mut Node = StrMap_GetBucket(this, key);
    if ((*node).key).is_null() {
        return std::ptr::null_mut();
    }
    while !node.is_null() {
        if StrEqual((*node).key, key) {
            return (*node).value;
        }
        node = (*node).next;
    }
    return std::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "C" fn StrMap_GetSize(mut this: *mut StrMap) -> u32 {
    return (*this).size;
}

#[no_mangle]
pub unsafe extern "C" fn StrMap_Remove(mut this: *mut StrMap, mut key: *const libc::c_char) {
    let mut prev: *mut *mut Node = std::ptr::null_mut();
    let mut node: *mut Node = StrMap_GetBucket(this, key);
    while !node.is_null() && !((*node).key).is_null() {
        if StrEqual((*node).key, key) {
            StrFree((*node).key);
            let mut next: *mut Node = (*node).next;
            if !next.is_null() {
                (*node).key = (*next).key;
                (*node).next = (*next).next;
                (*node).value = (*next).value;
                MemFree(next as *const libc::c_void);
            } else {
                (*node).key = std::ptr::null();
                (*node).value = std::ptr::null_mut();
            }
            if !prev.is_null() {
                *prev = next;
            }
            return;
        }
        prev = &mut (*node).next;
        node = (*node).next;
    }
    Fatal(
        b"StrMap_Remove: Map does not contain key <%s>\0" as *const u8 as *const libc::c_char,
        key,
    );
}

#[no_mangle]
pub unsafe extern "C" fn StrMap_Set(
    mut this: *mut StrMap,
    mut key: *const libc::c_char,
    mut value: *mut libc::c_void,
) {
    (*this).size = ((*this).size).wrapping_add(1);
    if (3_i32 as u32).wrapping_mul((*this).capacity)
        < (4_i32 as u32).wrapping_mul((*this).size)
    {
        StrMap_Grow(this);
    }
    let mut node: *mut Node = StrMap_GetBucket(this, key);
    if ((*node).key).is_null() {
        (*node).key = StrDup(key);
        (*node).value = value;
        return;
    }
    let mut prev: *mut *mut Node = std::ptr::null_mut();
    while !node.is_null() {
        if StrEqual((*node).key, key) {
            (*node).value = value;
            return;
        }
        prev = &mut (*node).next;
        node = (*node).next;
    }
    node = MemAlloc(::core::mem::size_of::<Node>()) as *mut Node;
    (*node).key = StrDup(key);
    (*node).value = value;
    (*node).next = std::ptr::null_mut();
    *prev = node;
}

#[no_mangle]
pub unsafe extern "C" fn StrMap_Dump(mut this: *mut StrMap) {
    libc::printf(
        b"StrMap @ %p:\n\0" as *const u8 as *const libc::c_char,
        this,
    );
    libc::printf(
        b"      size: %d\n\0" as *const u8 as *const libc::c_char,
        (*this).size,
    );
    libc::printf(
        b"  capacity: %d\n\0" as *const u8 as *const libc::c_char,
        (*this).capacity,
    );
    libc::printf(
        b"      load: %f\n\0" as *const u8 as *const libc::c_char,
        ((*this).size as f32 / (*this).capacity as f32) as f64,
    );
    libc::puts(b"\0" as *const u8 as *const libc::c_char);
    let mut i: u32 = 0_i32 as u32;
    while i < (*this).capacity {
        let mut node: *mut Node = ((*this).data).offset(i as isize);
        if !((*node).key).is_null() {
            libc::printf(b"  [%03i]:\n\0" as *const u8 as *const libc::c_char, i);
            while !node.is_null() {
                libc::printf(
                    b"    (%lx) %s -> %p\n\0" as *const u8 as *const libc::c_char,
                    Hash((*node).key),
                    (*node).key,
                    (*node).value,
                );
                node = (*node).next;
            }
        }
        i = i.wrapping_add(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn StrMap_Iterate(mut this: *mut StrMap) -> *mut StrMapIter {
    let mut it: *mut StrMapIter =
        MemAlloc(::core::mem::size_of::<StrMapIter>()) as *mut StrMapIter;
    (*it).map = this;
    (*it).slot = 0_i32 as u32;
    (*it).node = std::ptr::null_mut();
    let mut i: u32 = 0_i32 as u32;
    while i < (*this).capacity {
        let mut node: *mut Node = ((*this).data).offset(i as isize);
        if ((*node).key).is_null() {
            i = i.wrapping_add(1);
        } else {
            (*it).slot = i;
            (*it).node = node;
            break;
        }
    }
    return it;
}

#[no_mangle]
pub unsafe extern "C" fn StrMapIter_Free(mut this: *mut StrMapIter) {
    MemFree(this as *const libc::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn StrMapIter_Advance(mut it: *mut StrMapIter) {
    let mut this: *mut StrMap = (*it).map;
    (*it).node = (*(*it).node).next;
    if !((*it).node).is_null() {
        return;
    }
    (*it).slot = ((*it).slot).wrapping_add(1);
    let mut i: u32 = (*it).slot;
    while i < (*this).capacity {
        let mut node: *mut Node = ((*this).data).offset(i as isize);
        if ((*node).key).is_null() {
            i = i.wrapping_add(1);
        } else {
            (*it).slot = i;
            (*it).node = node;
            break;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn StrMapIter_HasMore(mut it: *mut StrMapIter) -> bool {
    return !((*it).node).is_null();
}

#[no_mangle]
pub unsafe extern "C" fn StrMapIter_GetKey(mut it: *mut StrMapIter) -> *const libc::c_char {
    return (*(*it).node).key;
}

#[no_mangle]
pub unsafe extern "C" fn StrMapIter_GetValue(mut it: *mut StrMapIter) -> *mut libc::c_void {
    return (*(*it).node).value;
}
