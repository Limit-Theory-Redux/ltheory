use crate::common::*;
use crate::hash::*;
use crate::*;

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
unsafe extern "C" fn Hash(key: *const libc::c_char) -> u64 {
    Hash_XX64(key as *const _, key.convert().len() as i32, 0)
}

#[inline]
unsafe extern "C" fn StrMap_GetBucket(this: &mut StrMap, key: *const libc::c_char) -> *mut Node {
    (this.data).offset((Hash(key)).wrapping_rem(this.capacity as u64) as isize)
}

unsafe extern "C" fn StrMap_Grow(this: &mut StrMap) {
    let mut newMap: StrMap = StrMap {
        capacity: (this.capacity).wrapping_mul(2),
        size: 0,
        data: std::ptr::null_mut(),
    };
    newMap.data = MemNewArrayZero!(Node, newMap.capacity);
    let mut i: u32 = 0;
    while i < this.capacity {
        let mut node: *mut Node = (this.data).offset(i as isize);
        if !((*node).key).is_null() {
            StrMap_Set(&mut newMap, (*node).key, (*node).value);
            // FIXME: memory leak
            // StrFree((*node).key);
            node = (*node).next;
            while !node.is_null() {
                let next: *mut Node = (*node).next;
                StrMap_Set(&mut newMap, (*node).key, (*node).value);
                // FIXME: memory leak
                // StrFree((*node).key);
                MemFree(node as *const _);
                node = next;
            }
        }
        i = i.wrapping_add(1);
    }
    MemFree(this.data as *const _);
    *this = newMap;
}

#[no_mangle]
pub unsafe extern "C" fn StrMap_Create(capacity: u32) -> *mut StrMap {
    let this = MemNewZero!(StrMap);
    (*this).capacity = capacity;
    (*this).data = MemNewArrayZero!(Node, capacity);
    this
}

#[no_mangle]
pub unsafe extern "C" fn StrMap_Free(this: *mut StrMap) {
    let mut i: u32 = 0;
    while i < (*this).capacity {
        let mut node: *mut Node = ((*this).data).offset(i as isize);
        if !((*node).key).is_null() {
            // FIXME: memory leak
            // StrFree((*node).key);
            node = (*node).next;
            while !node.is_null() {
                let next: *mut Node = (*node).next;
                // FIXME: memory leak
                // StrFree((*node).key);
                MemFree(node as *const _);
                node = next;
            }
        }
        i = i.wrapping_add(1);
    }
    MemFree((*this).data as *const _);
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn StrMap_FreeEx(
    this: *mut StrMap,
    freeFn: Option<unsafe extern "C" fn(*const libc::c_char, *mut libc::c_void) -> ()>,
) {
    let mut i: u32 = 0;
    while i < (*this).capacity {
        let mut node: *mut Node = ((*this).data).offset(i as isize);
        if !((*node).key).is_null() {
            freeFn.expect("non-null function pointer")((*node).key, (*node).value);
            // FIXME: memory leak
            // StrFree((*node).key);
            node = (*node).next;
            while !node.is_null() {
                let next: *mut Node = (*node).next;
                freeFn.expect("non-null function pointer")((*node).key, (*node).value);
                // FIXME: memory leak
                // StrFree((*node).key);
                MemFree(node as *const _);
                node = next;
            }
        }
        i = i.wrapping_add(1);
    }
    MemFree((*this).data as *const _);
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn StrMap_Get(
    this: &mut StrMap,
    key: *const libc::c_char,
) -> *mut libc::c_void {
    let mut node: *mut Node = StrMap_GetBucket(this, key);
    if ((*node).key).is_null() {
        return std::ptr::null_mut();
    }
    while !node.is_null() {
        if (*node).key.convert() == key.convert() {
            return (*node).value;
        }
        node = (*node).next;
    }
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn StrMap_GetSize(this: &mut StrMap) -> u32 {
    this.size
}

#[no_mangle]
pub unsafe extern "C" fn StrMap_Remove(this: &mut StrMap, key: *const libc::c_char) {
    let mut prev: *mut *mut Node = std::ptr::null_mut();
    let mut node: *mut Node = StrMap_GetBucket(this, key);
    while !node.is_null() && !((*node).key).is_null() {
        if (*node).key.convert() == key.convert() {
            // FIXME: memory leak
            // StrFree((*node).key);
            let next: *mut Node = (*node).next;
            if !next.is_null() {
                (*node).key = (*next).key;
                (*node).next = (*next).next;
                (*node).value = (*next).value;
                MemFree(next as *const _);
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
    CFatal!("StrMap_Remove: Map does not contain key <%s>", key);
}

#[no_mangle]
pub unsafe extern "C" fn StrMap_Set(
    this: &mut StrMap,
    key: *const libc::c_char,
    value: *mut libc::c_void,
) {
    this.size = (this.size).wrapping_add(1);
    if (3_u32).wrapping_mul(this.capacity) < (4_u32).wrapping_mul(this.size) {
        StrMap_Grow(this);
    }
    let mut node: *mut Node = StrMap_GetBucket(this, key);
    if ((*node).key).is_null() {
        (*node).key = key.convert().convert();
        (*node).value = value;
        return;
    }
    let mut prev: *mut *mut Node = std::ptr::null_mut();
    while !node.is_null() {
        if (*node).key.convert() == key.convert() {
            (*node).value = value;
            return;
        }
        prev = &mut (*node).next;
        node = (*node).next;
    }
    node = MemNew!(Node);
    (*node).key = key.convert().convert();
    (*node).value = value;
    (*node).next = std::ptr::null_mut();
    *prev = node;
}

#[no_mangle]
pub unsafe extern "C" fn StrMap_Dump(this: &mut StrMap) {
    CPrintf!("StrMap @ %p:\n", &this);
    CPrintf!("      size: %d\n", this.size);
    CPrintf!("  capacity: %d\n", this.capacity);
    CPrintf!(
        "      load: %f\n",
        (this.size as f32 / this.capacity as f32) as f64,
    );
    println!("");
    let mut i: u32 = 0;
    while i < this.capacity {
        let mut node: *mut Node = (this.data).offset(i as isize);
        if !((*node).key).is_null() {
            CPrintf!("  [%03i]:\n", i);
            while !node.is_null() {
                CPrintf!(
                    "    (%lx) %s -> %p\n",
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
pub unsafe extern "C" fn StrMap_Iterate(this: &mut StrMap) -> *mut StrMapIter {
    let it = MemNew!(StrMapIter);
    (*it).map = this;
    (*it).slot = 0;
    (*it).node = std::ptr::null_mut();
    let mut i: u32 = 0;
    while i < this.capacity {
        let node: *mut Node = (this.data).offset(i as isize);
        if ((*node).key).is_null() {
            i = i.wrapping_add(1);
        } else {
            (*it).slot = i;
            (*it).node = node;
            break;
        }
    }
    it
}

#[no_mangle]
pub unsafe extern "C" fn StrMapIter_Free(this: *mut StrMapIter) {
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn StrMapIter_Advance(it: *mut StrMapIter) {
    let this: *mut StrMap = (*it).map;
    (*it).node = (*(*it).node).next;
    if !((*it).node).is_null() {
        return;
    }
    (*it).slot = ((*it).slot).wrapping_add(1);
    let mut i: u32 = (*it).slot;
    while i < (*this).capacity {
        let node: *mut Node = ((*this).data).offset(i as isize);
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
pub unsafe extern "C" fn StrMapIter_HasMore(it: *mut StrMapIter) -> bool {
    !((*it).node).is_null()
}

#[no_mangle]
pub unsafe extern "C" fn StrMapIter_GetKey(it: *mut StrMapIter) -> *const libc::c_char {
    (*(*it).node).key
}

#[no_mangle]
pub unsafe extern "C" fn StrMapIter_GetValue(it: *mut StrMapIter) -> *mut libc::c_void {
    (*(*it).node).value
}
