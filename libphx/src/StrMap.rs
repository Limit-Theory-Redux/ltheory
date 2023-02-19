use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
extern "C" {
    fn strcmp(_: *const libc::c_char, _: *const libc::c_char) -> libc::c_int;
    fn Hash_XX64(buf: *const libc::c_void, len: libc::c_int, seed: uint64) -> uint64;
    fn Fatal(_: cstr, _: ...);
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
    fn puts(_: *const libc::c_char) -> libc::c_int;
}
pub type uint32_t = libc::c_uint;
pub type uint64_t = libc::c_ulonglong;
pub type cstr = *const libc::c_char;
pub type uint32 = uint32_t;
pub type uint64 = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StrMap {
    pub capacity: uint32,
    pub size: uint32,
    pub data: *mut Node,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Node {
    pub key: cstr,
    pub next: *mut Node,
    pub value: *mut libc::c_void,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct StrMapIter {
    pub map: *mut StrMap,
    pub node: *mut Node,
    pub slot: uint32,
}




#[inline]
unsafe extern "C" fn Hash(mut key: cstr) -> uint64 {
    return Hash_XX64(
        key as *const libc::c_void,
        StrLen(key) as libc::c_int,
        0 as libc::c_ulonglong,
    );
}
#[inline]
unsafe extern "C" fn StrMap_GetBucket(
    mut self_0: *mut StrMap,
    mut key: cstr,
) -> *mut Node {
    return ((*self_0).data)
        .offset(
            (Hash(key)).wrapping_rem((*self_0).capacity as libc::c_ulonglong) as isize,
        );
}
unsafe extern "C" fn StrMap_Grow(mut self_0: *mut StrMap) {
    let mut newMap: StrMap = {
        let mut init = StrMap {
            capacity: ((*self_0).capacity)
                .wrapping_mul(2 as libc::c_int as libc::c_uint),
            size: 0 as libc::c_int as uint32,
            data: 0 as *mut Node,
        };
        init
    };
    newMap
        .data = MemAllocZero(
        (::core::mem::size_of::<Node>())
            .wrapping_mul(newMap.capacity as usize),
    ) as *mut Node;
    let mut i: uint32 = 0 as libc::c_int as uint32;
    while i < (*self_0).capacity {
        let mut node: *mut Node = ((*self_0).data).offset(i as isize);
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
    MemFree((*self_0).data as *const libc::c_void);
    *self_0 = newMap;
}
#[no_mangle]
pub unsafe extern "C" fn StrMap_Create(mut capacity: uint32) -> *mut StrMap {
    let mut self_0: *mut StrMap = MemAllocZero(
        ::core::mem::size_of::<StrMap>() as usize,
    ) as *mut StrMap;
    (*self_0).capacity = capacity;
    (*self_0)
        .data = MemAllocZero(
        (::core::mem::size_of::<Node>())
            .wrapping_mul(capacity as usize),
    ) as *mut Node;
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn StrMap_Free(mut self_0: *mut StrMap) {
    let mut i: uint32 = 0 as libc::c_int as uint32;
    while i < (*self_0).capacity {
        let mut node: *mut Node = ((*self_0).data).offset(i as isize);
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
    MemFree((*self_0).data as *const libc::c_void);
    MemFree(self_0 as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn StrMap_FreeEx(
    mut self_0: *mut StrMap,
    mut freeFn: Option::<unsafe extern "C" fn(cstr, *mut libc::c_void) -> ()>,
) {
    let mut i: uint32 = 0 as libc::c_int as uint32;
    while i < (*self_0).capacity {
        let mut node: *mut Node = ((*self_0).data).offset(i as isize);
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
    MemFree((*self_0).data as *const libc::c_void);
    MemFree(self_0 as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn StrMap_Get(
    mut self_0: *mut StrMap,
    mut key: cstr,
) -> *mut libc::c_void {
    let mut node: *mut Node = StrMap_GetBucket(self_0, key);
    if ((*node).key).is_null() {
        return 0 as *mut libc::c_void;
    }
    while !node.is_null() {
        if StrEqual((*node).key, key) {
            return (*node).value;
        }
        node = (*node).next;
    }
    return 0 as *mut libc::c_void;
}
#[no_mangle]
pub unsafe extern "C" fn StrMap_GetSize(mut self_0: *mut StrMap) -> uint32 {
    return (*self_0).size;
}
#[no_mangle]
pub unsafe extern "C" fn StrMap_Remove(mut self_0: *mut StrMap, mut key: cstr) {
    let mut prev: *mut *mut Node = 0 as *mut *mut Node;
    let mut node: *mut Node = StrMap_GetBucket(self_0, key);
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
                (*node).key = 0 as cstr;
                (*node).value = 0 as *mut libc::c_void;
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
        b"StrMap_Remove: Map does not contain key <%s>\0" as *const u8
            as *const libc::c_char,
        key,
    );
}
#[no_mangle]
pub unsafe extern "C" fn StrMap_Set(
    mut self_0: *mut StrMap,
    mut key: cstr,
    mut value: *mut libc::c_void,
) {
    (*self_0).size = ((*self_0).size).wrapping_add(1);
    if (3 as libc::c_int as libc::c_uint).wrapping_mul((*self_0).capacity)
        < (4 as libc::c_int as libc::c_uint).wrapping_mul((*self_0).size)
    {
        StrMap_Grow(self_0);
    }
    let mut node: *mut Node = StrMap_GetBucket(self_0, key);
    if ((*node).key).is_null() {
        (*node).key = StrDup(key);
        (*node).value = value;
        return;
    }
    let mut prev: *mut *mut Node = 0 as *mut *mut Node;
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
    (*node).next = 0 as *mut Node;
    *prev = node;
}
#[no_mangle]
pub unsafe extern "C" fn StrMap_Dump(mut self_0: *mut StrMap) {
    printf(b"StrMap @ %p:\n\0" as *const u8 as *const libc::c_char, self_0);
    printf(b"      size: %d\n\0" as *const u8 as *const libc::c_char, (*self_0).size);
    printf(
        b"  capacity: %d\n\0" as *const u8 as *const libc::c_char,
        (*self_0).capacity,
    );
    printf(
        b"      load: %f\n\0" as *const u8 as *const libc::c_char,
        ((*self_0).size as libc::c_float / (*self_0).capacity as libc::c_float)
            as libc::c_double,
    );
    puts(b"\0" as *const u8 as *const libc::c_char);
    let mut i: uint32 = 0 as libc::c_int as uint32;
    while i < (*self_0).capacity {
        let mut node: *mut Node = ((*self_0).data).offset(i as isize);
        if !((*node).key).is_null() {
            printf(b"  [%03i]:\n\0" as *const u8 as *const libc::c_char, i);
            while !node.is_null() {
                printf(
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
pub unsafe extern "C" fn StrMap_Iterate(mut self_0: *mut StrMap) -> *mut StrMapIter {
    let mut it: *mut StrMapIter = MemAlloc(
        ::core::mem::size_of::<StrMapIter>() as usize,
    ) as *mut StrMapIter;
    (*it).map = self_0;
    (*it).slot = 0 as libc::c_int as uint32;
    (*it).node = 0 as *mut Node;
    let mut i: uint32 = 0 as libc::c_int as uint32;
    while i < (*self_0).capacity {
        let mut node: *mut Node = ((*self_0).data).offset(i as isize);
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
pub unsafe extern "C" fn StrMapIter_Free(mut self_0: *mut StrMapIter) {
    MemFree(self_0 as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn StrMapIter_Advance(mut it: *mut StrMapIter) {
    let mut self_0: *mut StrMap = (*it).map;
    (*it).node = (*(*it).node).next;
    if !((*it).node).is_null() {
        return;
    }
    (*it).slot = ((*it).slot).wrapping_add(1);
    let mut i: uint32 = (*it).slot;
    while i < (*self_0).capacity {
        let mut node: *mut Node = ((*self_0).data).offset(i as isize);
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
pub unsafe extern "C" fn StrMapIter_GetKey(mut it: *mut StrMapIter) -> cstr {
    return (*(*it).node).key;
}
#[no_mangle]
pub unsafe extern "C" fn StrMapIter_GetValue(
    mut it: *mut StrMapIter,
) -> *mut libc::c_void {
    return (*(*it).node).value;
}
