use crate::internal::Memory::*;
use crate::Common::*;
use crate::Hash;
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
    pub cells: Vec<HashGridCell>,
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

#[no_mangle]
pub unsafe extern "C" fn HashGrid_Create(cellSize: f32, mut cellCount: u32) -> *mut HashGrid {
    let mut logCount: u32 = 0;
    while cellCount > 1 {
        cellCount = cellCount.wrapping_div(2);
        logCount = logCount.wrapping_add(1);
    }
    cellCount = (1 << logCount) as u32;

    let this = MemNew!(HashGrid);
    (*this).version = 0;
    (*this).cells = Vec::new();
    (*this).cells.resize(
        cellCount as usize,
        HashGridCell {
            version: 0,
            elems: Vec::new(),
        },
    );
    (*this).elemPool = MemPool_Create(
        std::mem::size_of::<HashGridElem>() as u32,
        (0x1000_usize).wrapping_div(std::mem::size_of::<HashGridElem>()) as u32,
    );
    (*this).cellCount = cellCount;
    (*this).cellSize = cellSize;
    (*this).mask = ((1 << logCount) - 1) as u32;
    (*this).results = Vec::new();

    this
}

#[no_mangle]
pub unsafe extern "C" fn HashGrid_Free(this: *mut HashGrid) {
    (*this).cells.clear();
    MemPool_Free((*this).elemPool);
    MemDelete!(this);
}

#[inline]
unsafe extern "C" fn HashGrid_GetCell(
    this: *mut HashGrid,
    x: i32,
    y: i32,
    z: i32,
) -> *mut HashGridCell {
    let mut p: [i32; 3] = [x, y, z];
    let hash: u64 = Hash_XX64(
        p.as_mut_ptr() as *const _,
        std::mem::size_of::<[i32; 3]>() as i32,
        0,
    );
    (*this)
        .cells
        .get_unchecked_mut((hash & (*this).mask as u64) as usize) as *mut _
}

unsafe extern "C" fn HashGrid_AddElem(this: *mut HashGrid, elem: *mut HashGridElem) {
    (*this).version += 1;
    for x in (*elem).lower[0]..=(*elem).upper[0] {
        for y in (*elem).lower[1]..=(*elem).upper[1] {
            for z in (*elem).lower[2]..=(*elem).upper[2] {
                let cell: *mut HashGridCell = HashGrid_GetCell(this, x, y, z);

                /* If cell version is out-of-date, we need to insert. Otherwise, we have
                 * already inserted into this cell (e.g., we have encountered a modulus
                 * hash collision) and should not insert a duplicate. */
                if (*cell).version != (*this).version {
                    (*cell).version = (*this).version;
                    (*cell).elems.push(elem);
                }
            }
        }
    }
}

unsafe extern "C" fn HashGrid_RemoveElem(this: *mut HashGrid, elem: *mut HashGridElem) {
    (*this).version += 1;
    for x in (*elem).lower[0]..=(*elem).upper[0] {
        for y in (*elem).lower[1]..=(*elem).upper[1] {
            for z in (*elem).lower[2]..=(*elem).upper[2] {
                let cell: *mut HashGridCell = HashGrid_GetCell(this, x, y, z);
                if (*cell).version != (*this).version {
                    (*cell).version = (*this).version;
                    if let Some(index) = (*cell).elems.iter().position(|c| *c == elem) {
                        (*cell).elems.swap_remove(index);
                    }
                }
            }
        }
    }
}

#[inline]
unsafe extern "C" fn HashGrid_ToLocal(this: *const HashGrid, x: f32) -> i32 {
    f32::floor(x / (*this).cellSize) as i32
}

#[no_mangle]
pub unsafe extern "C" fn HashGrid_Add(
    this: *mut HashGrid,
    object: *mut libc::c_void,
    box_0: *const Box3,
) -> *mut HashGridElem {
    let elem: *mut HashGridElem = MemPool_Alloc((*this).elemPool) as *mut HashGridElem;
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
pub unsafe extern "C" fn HashGrid_Clear(this: *mut HashGrid) {
    (*this).version = 0;
    for i in 0..(*this).cellCount {
        (*this).cells[i as usize].elems.clear();
        (*this).cells[i as usize].version = 0;
    }
    MemPool_Clear((*this).elemPool);
    (*this).results.clear();
}

#[no_mangle]
pub unsafe extern "C" fn HashGrid_Remove(this: *mut HashGrid, elem: *mut HashGridElem) {
    HashGrid_RemoveElem(this, elem);
    MemPool_Dealloc((*this).elemPool, elem as *mut _);
}

#[no_mangle]
pub unsafe extern "C" fn HashGrid_Update(
    this: *mut HashGrid,
    elem: *mut HashGridElem,
    box_0: *const Box3,
) {
    Profiler_Begin(c_str!("HashGrid_Update"));
    let lower: [i32; 3] = [
        HashGrid_ToLocal(this, (*box_0).lower.x),
        HashGrid_ToLocal(this, (*box_0).lower.y),
        HashGrid_ToLocal(this, (*box_0).lower.z),
    ];

    let upper: [i32; 3] = [
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

    let lowerUnion: [i32; 3] = [
        i32::min(lower[0], (*elem).lower[0]),
        i32::min(lower[1], (*elem).lower[1]),
        i32::min(lower[2], (*elem).lower[2]),
    ];

    let upperUnion: [i32; 3] = [
        i32::max(upper[0], (*elem).upper[0]),
        i32::max(upper[1], (*elem).upper[1]),
        i32::max(upper[2], (*elem).upper[2]),
    ];

    (*this).version += 1;
    let vRemove: u64 = (*this).version;
    (*this).version += 1;
    let vAdd: u64 = (*this).version;

    for x in lowerUnion[0]..=upperUnion[0] {
        for y in lowerUnion[1]..=upperUnion[1] {
            for z in lowerUnion[2]..=upperUnion[2] {
                let inPrev: bool = (*elem).lower[0] <= x
                    && (*elem).lower[1] <= y
                    && (*elem).lower[2] <= z
                    && x <= (*elem).upper[0]
                    && y <= (*elem).upper[1]
                    && z <= (*elem).upper[2];

                let inCurr: bool = lower[0] <= x
                    && lower[1] <= y
                    && lower[2] <= z
                    && x <= upper[0]
                    && y <= upper[1]
                    && z <= upper[2];

                /* Early out: cell is part of both previous and new bounding box, no change
                 * required. */
                if inPrev as i32 != 0 && inCurr as i32 != 0 {
                    continue;
                }

                let cell: *mut HashGridCell = HashGrid_GetCell(this, x, y, z);

                /* Early out: cell has already had elem inserted, no update required. */
                if (*cell).version == vAdd {
                    continue;
                }

                /* Early out: cell has already had elem removed and this is not part of the
                 * insertion frontier. */
                if (*cell).version == vRemove && inPrev as i32 != 0 {
                    continue;
                }

                /* inPrev but not inCurr -> remove elem and mark cell as having removed. */
                if inPrev {
                    if let Some(index) = (*cell).elems.iter().position(|c| *c == elem) {
                        (*cell).elems.swap_remove(index);
                    }
                    (*cell).version = vRemove;
                    continue;
                }

                /* Final case: insertion frontier. We must add the elem to this cell. */
                if (*cell).version != vRemove {
                    if let Some(index) = (*cell).elems.iter().position(|c| *c == elem) {
                        (*cell).elems.swap_remove(index);
                    }
                }
                (*cell).elems.push(elem);
                (*cell).version = vAdd;
            }
        }
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
pub unsafe extern "C" fn HashGrid_GetResults(this: *mut HashGrid) -> *mut *mut libc::c_void {
    (*this).results.as_mut_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn HashGrid_QueryBox(this: *mut HashGrid, box_0: *const Box3) -> i32 {
    (*this).results.clear();
    (*this).version += 1;

    let lower: [i32; 3] = [
        HashGrid_ToLocal(this, (*box_0).lower.x),
        HashGrid_ToLocal(this, (*box_0).lower.y),
        HashGrid_ToLocal(this, (*box_0).lower.z),
    ];

    let upper: [i32; 3] = [
        HashGrid_ToLocal(this, (*box_0).upper.x),
        HashGrid_ToLocal(this, (*box_0).upper.y),
        HashGrid_ToLocal(this, (*box_0).upper.z),
    ];

    for x in lower[0]..=upper[0] {
        for y in lower[1]..=upper[1] {
            for z in lower[2]..=upper[2] {
                let cell: *mut HashGridCell = HashGrid_GetCell(this, x, y, z);
                if (*cell).version != (*this).version {
                    (*cell).version = (*this).version;

                    for elem in (*cell).elems.iter() {
                        if (**elem).version != (*this).version {
                            (**elem).version = (*this).version;
                            (*this).results.push((**elem).object);
                        }
                    }
                }
            }
        }
    }

    (*this).results.len() as i32
}

#[no_mangle]
pub unsafe extern "C" fn HashGrid_QueryPoint(this: *mut HashGrid, p: *const Vec3) -> i32 {
    /* Since a point query is restricted to a single cell, we don't need to use
     * versioning here. */
    (*this).results.clear();

    let cell: *mut HashGridCell = HashGrid_GetCell(
        this,
        HashGrid_ToLocal(this, (*p).x),
        HashGrid_ToLocal(this, (*p).y),
        HashGrid_ToLocal(this, (*p).z),
    );

    for elem in (*cell).elems.iter() {
        (*this).results.push((**elem).object);
    }
    (*this).results.len() as i32
}
