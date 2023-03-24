use crate::internal::Memory::*;
use crate::Common::*;
use crate::Hash::*;
use crate::Math::Box3;
use crate::Math::Vec3;
use crate::MemPool::*;
use crate::Profiler::*;
use libc;

#[derive(Clone)]
#[repr(C)]
pub struct HashGrid {
    pub version: u64,
    pub cells: *mut HashGridCell,
    pub elemPool: *mut MemPool,
    pub cellCount: u32,
    pub cellSize: f32,
    pub mask: u32,
    pub results: Vec<*mut libc::c_void>,
}

#[derive(Clone)]
#[repr(C)]
pub struct HashGridCell {
    pub version: u64,
    pub elems: Vec<*mut HashGridElem>,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HashGridElem {
    pub version: u64,
    pub object: *mut libc::c_void,
    pub lower: [i32; 3],
    pub upper: [i32; 3],
}

#[inline]
unsafe extern "C" fn Maxi(mut a: i32, mut b: i32) -> i32 {
    if a > b {
        a
    } else {
        b
    }
}

#[inline]
unsafe extern "C" fn Mini(mut a: i32, mut b: i32) -> i32 {
    if a < b {
        a
    } else {
        b
    }
}

#[no_mangle]
pub unsafe extern "C" fn HashGrid_Create(mut cellSize: f32, mut cellCount: u32) -> *mut HashGrid {
    let mut logCount: u32 = 0;
    while cellCount > 1 {
        cellCount = cellCount.wrapping_div(2);
        logCount = logCount.wrapping_add(1);
    }
    cellCount = (1 << logCount) as u32;
    let mut this = MemNew!(HashGrid);
    (*this).version = 0;
    (*this).cells = MemNewArrayZero!(HashGridCell, cellCount);
    (*this).elemPool = MemPool_Create(
        std::mem::size_of::<HashGridElem>() as u32,
        (0x1000_usize).wrapping_div(std::mem::size_of::<HashGridElem>()) as u32,
    );
    (*this).cellCount = cellCount;
    (*this).cellSize = cellSize;
    (*this).mask = ((1 << logCount) - 1) as u32;
    (*this).results = Vec::new();
    let mut i: u32 = 0;
    while i < cellCount {
        (*((*this).cells).offset(i as isize)).elems = Vec::new();
        i = i.wrapping_add(1);
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn HashGrid_Free(mut this: *mut HashGrid) {
    let mut i: u32 = 0;
    while i < (*this).cellCount {
        ((*this).cells).offset(i as isize).drop_in_place();
        i += 1;
    }
    MemPool_Free((*this).elemPool);
    MemFree((*this).cells as *const _);
    MemDelete!(this);
}

#[inline]
unsafe extern "C" fn HashGrid_GetCell(
    mut this: *mut HashGrid,
    mut x: i32,
    mut y: i32,
    mut z: i32,
) -> *mut HashGridCell {
    let mut p: [i32; 3] = [x, y, z];
    let mut hash: u64 = Hash_XX64(
        p.as_mut_ptr() as *const _,
        std::mem::size_of::<[i32; 3]>() as libc::c_ulong as i32,
        0,
    );
    ((*this).cells).offset((hash & (*this).mask as u64) as isize)
}

unsafe extern "C" fn HashGrid_AddElem(mut this: *mut HashGrid, mut elem: *mut HashGridElem) {
    (*this).version += 1;
    let mut x: i32 = (*elem).lower[0];
    while x <= (*elem).upper[0] {
        let mut y: i32 = (*elem).lower[1];
        while y <= (*elem).upper[1] {
            let mut z: i32 = (*elem).lower[2];
            while z <= (*elem).upper[2] {
                let mut cell: *mut HashGridCell = HashGrid_GetCell(this, x, y, z);
                if (*cell).version != (*this).version {
                    (*cell).version = (*this).version;
                    (*cell).elems.push(elem);
                }
                z += 1;
            }
            y += 1;
        }
        x += 1;
    }
}

unsafe extern "C" fn HashGrid_RemoveElem(mut this: *mut HashGrid, mut elem: *mut HashGridElem) {
    (*this).version += 1;
    let mut x: i32 = (*elem).lower[0];
    while x <= (*elem).upper[0] {
        let mut y: i32 = (*elem).lower[1];
        while y <= (*elem).upper[1] {
            let mut z: i32 = (*elem).lower[2];
            while z <= (*elem).upper[2] {
                let mut cell: *mut HashGridCell = HashGrid_GetCell(this, x, y, z);
                if (*cell).version != (*this).version {
                    (*cell).version = (*this).version;
                    if let Some(index) = (*cell).elems.iter().position(|c| *c == elem) {
                        (*cell).elems.swap_remove(index);
                    }
                }
                z += 1;
            }
            y += 1;
        }
        x += 1;
    }
}

#[inline]
unsafe extern "C" fn HashGrid_ToLocal(mut this: *mut HashGrid, mut x: f32) -> i32 {
    f64::floor((x / (*this).cellSize) as f64) as i32
}

#[no_mangle]
pub unsafe extern "C" fn HashGrid_Add(
    mut this: *mut HashGrid,
    mut object: *mut libc::c_void,
    mut box_0: *const Box3,
) -> *mut HashGridElem {
    let mut elem: *mut HashGridElem = MemPool_Alloc((*this).elemPool) as *mut HashGridElem;
    (*elem).object = object;
    (*elem).lower[0] = HashGrid_ToLocal(this, (*box_0).lower.x);
    (*elem).lower[1] = HashGrid_ToLocal(this, (*box_0).lower.y);
    (*elem).lower[2] = HashGrid_ToLocal(this, (*box_0).lower.z);
    (*elem).upper[0] = HashGrid_ToLocal(this, (*box_0).upper.x);
    (*elem).upper[1] = HashGrid_ToLocal(this, (*box_0).upper.y);
    (*elem).upper[2] = HashGrid_ToLocal(this, (*box_0).upper.z);
    HashGrid_AddElem(this, elem);
    elem
}

#[no_mangle]
pub unsafe extern "C" fn HashGrid_Clear(mut this: *mut HashGrid) {
    (*this).version = 0;
    let mut i: u32 = 0;
    while i < (*this).cellCount {
        (*((*this).cells).offset(i as isize)).elems.clear();
        (*((*this).cells).offset(i as isize)).version = 0;
        i = i.wrapping_add(1);
    }
    MemPool_Clear((*this).elemPool);
    (*this).results.clear();
}

#[no_mangle]
pub unsafe extern "C" fn HashGrid_Remove(mut this: *mut HashGrid, mut elem: *mut HashGridElem) {
    HashGrid_RemoveElem(this, elem);
    MemPool_Dealloc((*this).elemPool, elem as *mut _);
}

#[no_mangle]
pub unsafe extern "C" fn HashGrid_Update(
    mut this: *mut HashGrid,
    mut elem: *mut HashGridElem,
    mut box_0: *const Box3,
) {
    Profiler_Begin(
        (*std::mem::transmute::<&[u8; 16], &[libc::c_char; 16]>(b"HashGrid_Update\0")).as_ptr(),
    );
    let mut lower: [i32; 3] = [
        HashGrid_ToLocal(this, (*box_0).lower.x),
        HashGrid_ToLocal(this, (*box_0).lower.y),
        HashGrid_ToLocal(this, (*box_0).lower.z),
    ];
    let mut upper: [i32; 3] = [
        HashGrid_ToLocal(this, (*box_0).upper.x),
        HashGrid_ToLocal(this, (*box_0).upper.y),
        HashGrid_ToLocal(this, (*box_0).upper.z),
    ];
    if lower[0] == (*elem).lower[0]
        && upper[0] == (*elem).upper[0]
        && lower[1] == (*elem).lower[1]
        && upper[1] == (*elem).upper[1]
        && lower[2] == (*elem).lower[2]
        && upper[2] == (*elem).upper[2]
    {
        Profiler_End();
        return;
    }
    let mut lowerUnion: [i32; 3] = [
        Mini(lower[0], (*elem).lower[0]),
        Mini(lower[1], (*elem).lower[1]),
        Mini(lower[2], (*elem).lower[2]),
    ];
    let mut upperUnion: [i32; 3] = [
        Maxi(upper[0], (*elem).upper[0]),
        Maxi(upper[1], (*elem).upper[1]),
        Maxi(upper[2], (*elem).upper[2]),
    ];
    (*this).version += 1;
    let mut vRemove: u64 = (*this).version;
    (*this).version += 1;
    let mut vAdd: u64 = (*this).version;
    let mut x: i32 = lowerUnion[0];
    while x <= upperUnion[0] {
        let mut y: i32 = lowerUnion[1];
        while y <= upperUnion[1] {
            let mut z: i32 = lowerUnion[2];
            while z <= upperUnion[2] {
                let mut inPrev: bool = (*elem).lower[0] <= x
                    && (*elem).lower[1] <= y
                    && (*elem).lower[2] <= z
                    && x <= (*elem).upper[0]
                    && y <= (*elem).upper[1]
                    && z <= (*elem).upper[2];
                let mut inCurr: bool = lower[0] <= x
                    && lower[1] <= y
                    && lower[2] <= z
                    && x <= upper[0]
                    && y <= upper[1]
                    && z <= upper[2];
                if !(inPrev as i32 != 0 && inCurr as i32 != 0) {
                    let mut cell: *mut HashGridCell = HashGrid_GetCell(this, x, y, z);
                    if !((*cell).version == vAdd) {
                        if !((*cell).version == vRemove && inPrev as i32 != 0) {
                            if inPrev {
                                if let Some(index) = (*cell).elems.iter().position(|c| *c == elem) {
                                    (*cell).elems.swap_remove(index);
                                }
                                (*cell).version = vRemove;
                            } else {
                                if (*cell).version != vRemove {
                                    if let Some(index) =
                                        (*cell).elems.iter().position(|c| *c == elem)
                                    {
                                        (*cell).elems.swap_remove(index);
                                    }
                                }
                                (*cell).elems.push(elem);
                                (*cell).version = vAdd;
                            }
                        }
                    }
                }
                z += 1;
            }
            y += 1;
        }
        x += 1;
    }
    (*elem).lower[0] = lower[0];
    (*elem).lower[1] = lower[1];
    (*elem).lower[2] = lower[2];
    (*elem).upper[0] = upper[0];
    (*elem).upper[1] = upper[1];
    (*elem).upper[2] = upper[2];
    Profiler_End();
}

#[no_mangle]
pub unsafe extern "C" fn HashGrid_GetResults(mut this: *mut HashGrid) -> *mut *mut libc::c_void {
    (*this).results.as_mut_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn HashGrid_QueryBox(mut this: *mut HashGrid, mut box_0: *const Box3) -> i32 {
    (*this).results.clear();
    (*this).version += 1;
    let mut lower: [i32; 3] = [
        HashGrid_ToLocal(this, (*box_0).lower.x),
        HashGrid_ToLocal(this, (*box_0).lower.y),
        HashGrid_ToLocal(this, (*box_0).lower.z),
    ];
    let mut upper: [i32; 3] = [
        HashGrid_ToLocal(this, (*box_0).upper.x),
        HashGrid_ToLocal(this, (*box_0).upper.y),
        HashGrid_ToLocal(this, (*box_0).upper.z),
    ];
    let mut x: i32 = lower[0];
    while x <= upper[0] {
        let mut y: i32 = lower[1];
        while y <= upper[1] {
            let mut z: i32 = lower[2];
            while z <= upper[2] {
                let mut cell: *mut HashGridCell = HashGrid_GetCell(this, x, y, z);
                if (*cell).version != (*this).version {
                    (*cell).version = (*this).version;
                    let mut i: i32 = 0;
                    while i < (*cell).elems.len() as i32 {
                        let mut elem: *mut HashGridElem = (*cell).elems[i as usize];
                        if (*elem).version != (*this).version {
                            (*elem).version = (*this).version;
                            (*this).results.push((*elem).object);
                        }
                        i += 1;
                    }
                }
                z += 1;
            }
            y += 1;
        }
        x += 1;
    }
    (*this).results.len() as i32
}

#[no_mangle]
pub unsafe extern "C" fn HashGrid_QueryPoint(mut this: *mut HashGrid, mut p: *const Vec3) -> i32 {
    (*this).results.clear();
    let mut cell: *mut HashGridCell = HashGrid_GetCell(
        this,
        HashGrid_ToLocal(this, (*p).x),
        HashGrid_ToLocal(this, (*p).y),
        HashGrid_ToLocal(this, (*p).z),
    );
    let mut i: i32 = 0;
    while i < (*cell).elems.len() as i32 {
        let elem: *mut HashGridElem = (*cell).elems[i as usize];
        (*this).results.push((*elem).object);
        i += 1;
    }
    (*this).results.len() as i32
}
