use crate::internal::Memory::*;
use crate::Common::*;
use crate::Hash::*;
use crate::Math::Box3;
use crate::Math::Vec3;
use crate::MemPool::*;
use crate::Profiler::*;
use libc;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HashGrid {
    pub version: u64,
    pub cells: *mut HashGridCell,
    pub elemPool: *mut MemPool,
    pub cellCount: u32,
    pub cellSize: f32,
    pub mask: u32,
    pub results_size: i32,
    pub results_capacity: i32,
    pub results_data: *mut *mut libc::c_void,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct HashGridCell {
    pub version: u64,
    pub elems_size: i32,
    pub elems_capacity: i32,
    pub elems_data: *mut *mut HashGridElem,
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
    let mut logCount: u32 = 0_u32;
    while cellCount > 1_u32 {
        cellCount = cellCount.wrapping_div(2_u32);
        logCount = logCount.wrapping_add(1);
    }
    cellCount = (1_i32 << logCount) as u32;
    let mut this = MemNew!(HashGrid);
    (*this).version = 0_i32 as u64;
    (*this).cells = MemNewArrayZero!(HashGridCell, cellCount);
    (*this).elemPool = MemPool_Create(
        std::mem::size_of::<HashGridElem>() as u32,
        (0x1000_u32 as usize).wrapping_div(std::mem::size_of::<HashGridElem>()) as u32,
    );
    (*this).cellCount = cellCount;
    (*this).cellSize = cellSize;
    (*this).mask = ((1_i32 << logCount) - 1_i32) as u32;
    (*this).results_capacity = 0_i32;
    (*this).results_size = 0_i32;
    (*this).results_data = std::ptr::null_mut();
    let mut i: u32 = 0_u32;
    while i < cellCount {
        (*((*this).cells).offset(i as isize)).elems_capacity = 0_i32;
        (*((*this).cells).offset(i as isize)).elems_size = 0_i32;
        let ref mut fresh0 = (*((*this).cells).offset(i as isize)).elems_data;
        *fresh0 = std::ptr::null_mut();
        i = i.wrapping_add(1);
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn HashGrid_Free(mut this: *mut HashGrid) {
    MemFree((*this).results_data as *const libc::c_void);
    let mut i: u32 = 0_u32;
    while i < (*this).cellCount {
        MemFree((*((*this).cells).offset(i as isize)).elems_data as *const libc::c_void);
        i = i.wrapping_add(1);
    }
    MemPool_Free((*this).elemPool);
    MemFree((*this).cells as *const libc::c_void);
    MemFree(this as *const libc::c_void);
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
        p.as_mut_ptr() as *const libc::c_void,
        std::mem::size_of::<[i32; 3]>() as libc::c_ulong as i32,
        0_u64,
    );
    ((*this).cells).offset((hash & (*this).mask as u64) as isize)
}

unsafe extern "C" fn HashGrid_AddElem(mut this: *mut HashGrid, mut elem: *mut HashGridElem) {
    (*this).version = ((*this).version).wrapping_add(1);
    let mut x: i32 = (*elem).lower[0];
    while x <= (*elem).upper[0] {
        let mut y: i32 = (*elem).lower[1];
        while y <= (*elem).upper[1] {
            let mut z: i32 = (*elem).lower[2];
            while z <= (*elem).upper[2] {
                let mut cell: *mut HashGridCell = HashGrid_GetCell(this, x, y, z);
                if (*cell).version != (*this).version {
                    (*cell).version = (*this).version;
                    if ((*cell).elems_capacity == (*cell).elems_size) as i32 as libc::c_long != 0 {
                        (*cell).elems_capacity = if (*cell).elems_capacity != 0 {
                            (*cell).elems_capacity * 2_i32
                        } else {
                            1_i32
                        };
                        let mut elemSize: usize = std::mem::size_of::<*mut HashGridElem>();
                        let mut pData: *mut *mut libc::c_void = &mut (*cell).elems_data
                            as *mut *mut *mut HashGridElem
                            as *mut *mut libc::c_void;
                        *pData = MemRealloc(
                            (*cell).elems_data as *mut libc::c_void,
                            ((*cell).elems_capacity as usize).wrapping_mul(elemSize),
                        );
                    }
                    let fresh1 = (*cell).elems_size;
                    (*cell).elems_size += 1;
                    let ref mut fresh2 = *((*cell).elems_data).offset(fresh1 as isize);
                    *fresh2 = elem;
                }
                z += 1;
            }
            y += 1;
        }
        x += 1;
    }
}

unsafe extern "C" fn HashGrid_RemoveElem(mut this: *mut HashGrid, mut elem: *mut HashGridElem) {
    (*this).version = ((*this).version).wrapping_add(1);
    let mut x: i32 = (*elem).lower[0];
    while x <= (*elem).upper[0] {
        let mut y: i32 = (*elem).lower[1];
        while y <= (*elem).upper[1] {
            let mut z: i32 = (*elem).lower[2];
            while z <= (*elem).upper[2] {
                let mut cell: *mut HashGridCell = HashGrid_GetCell(this, x, y, z);
                if (*cell).version != (*this).version {
                    (*cell).version = (*this).version;
                    let mut _i: i32 = 0_i32;
                    while _i < (*cell).elems_size {
                        if (*((*cell).elems_data).offset(_i as isize) == elem) as libc::c_long != 0
                        {
                            (*cell).elems_size -= 1;
                            let ref mut fresh3 = *((*cell).elems_data).offset(_i as isize);
                            *fresh3 = *((*cell).elems_data).offset((*cell).elems_size as isize);
                            break;
                        } else {
                            _i += 1;
                        }
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
    (*this).version = 0_i32 as u64;
    let mut i: u32 = 0_u32;
    while i < (*this).cellCount {
        (*((*this).cells).offset(i as isize)).elems_size = 0_i32;
        (*((*this).cells).offset(i as isize)).version = 0_i32 as u64;
        i = i.wrapping_add(1);
    }
    MemPool_Clear((*this).elemPool);
    (*this).results_size = 0_i32;
}

#[no_mangle]
pub unsafe extern "C" fn HashGrid_Remove(mut this: *mut HashGrid, mut elem: *mut HashGridElem) {
    HashGrid_RemoveElem(this, elem);
    MemPool_Dealloc((*this).elemPool, elem as *mut libc::c_void);
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
    (*this).version = ((*this).version).wrapping_add(1);
    let mut vRemove: u64 = (*this).version;
    (*this).version = ((*this).version).wrapping_add(1);
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
                                let mut _i: i32 = 0_i32;
                                while _i < (*cell).elems_size {
                                    if (*((*cell).elems_data).offset(_i as isize) == elem)
                                        as libc::c_long
                                        != 0
                                    {
                                        (*cell).elems_size -= 1;
                                        let ref mut fresh4 =
                                            *((*cell).elems_data).offset(_i as isize);
                                        *fresh4 = *((*cell).elems_data)
                                            .offset((*cell).elems_size as isize);
                                        break;
                                    } else {
                                        _i += 1;
                                    }
                                }
                                (*cell).version = vRemove;
                            } else {
                                if (*cell).version != vRemove {
                                    let mut _i_0: i32 = 0_i32;
                                    while _i_0 < (*cell).elems_size {
                                        if (*((*cell).elems_data).offset(_i_0 as isize) == elem)
                                            as libc::c_long
                                            != 0
                                        {
                                            (*cell).elems_size -= 1;
                                            let ref mut fresh5 =
                                                *((*cell).elems_data).offset(_i_0 as isize);
                                            *fresh5 = *((*cell).elems_data)
                                                .offset((*cell).elems_size as isize);
                                            break;
                                        } else {
                                            _i_0 += 1;
                                        }
                                    }
                                }
                                if ((*cell).elems_capacity == (*cell).elems_size) as libc::c_long
                                    != 0
                                {
                                    (*cell).elems_capacity = if (*cell).elems_capacity != 0 {
                                        (*cell).elems_capacity * 2_i32
                                    } else {
                                        1_i32
                                    };
                                    let mut elemSize: usize =
                                        std::mem::size_of::<*mut HashGridElem>();
                                    let mut pData: *mut *mut libc::c_void = &mut (*cell).elems_data
                                        as *mut *mut *mut HashGridElem
                                        as *mut *mut libc::c_void;
                                    *pData = MemRealloc(
                                        (*cell).elems_data as *mut libc::c_void,
                                        ((*cell).elems_capacity as usize).wrapping_mul(elemSize),
                                    );
                                }
                                let fresh6 = (*cell).elems_size;
                                (*cell).elems_size += 1;
                                let ref mut fresh7 = *((*cell).elems_data).offset(fresh6 as isize);
                                *fresh7 = elem;
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
    (*this).results_data
}

#[no_mangle]
pub unsafe extern "C" fn HashGrid_QueryBox(mut this: *mut HashGrid, mut box_0: *const Box3) -> i32 {
    (*this).results_size = 0_i32;
    (*this).version = ((*this).version).wrapping_add(1);
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
                    let mut i: i32 = 0_i32;
                    while i < (*cell).elems_size {
                        let mut elem: *mut HashGridElem = *((*cell).elems_data).offset(i as isize);
                        if (*elem).version != (*this).version {
                            (*elem).version = (*this).version;
                            if ((*this).results_capacity == (*this).results_size) as libc::c_long
                                != 0
                            {
                                (*this).results_capacity = if (*this).results_capacity != 0 {
                                    (*this).results_capacity * 2_i32
                                } else {
                                    1_i32
                                };
                                let mut elemSize: usize = std::mem::size_of::<*mut libc::c_void>();
                                let mut pData: *mut *mut libc::c_void = &mut (*this).results_data
                                    as *mut *mut *mut libc::c_void
                                    as *mut *mut libc::c_void;
                                *pData = MemRealloc(
                                    (*this).results_data as *mut libc::c_void,
                                    ((*this).results_capacity as usize).wrapping_mul(elemSize),
                                );
                            }
                            let fresh8 = (*this).results_size;
                            (*this).results_size += 1;
                            let ref mut fresh9 = *((*this).results_data).offset(fresh8 as isize);
                            *fresh9 = (*elem).object;
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
    (*this).results_size
}

#[no_mangle]
pub unsafe extern "C" fn HashGrid_QueryPoint(mut this: *mut HashGrid, mut p: *const Vec3) -> i32 {
    (*this).results_size = 0_i32;
    let mut cell: *mut HashGridCell = HashGrid_GetCell(
        this,
        HashGrid_ToLocal(this, (*p).x),
        HashGrid_ToLocal(this, (*p).y),
        HashGrid_ToLocal(this, (*p).z),
    );
    let mut i: i32 = 0_i32;
    while i < (*cell).elems_size {
        let mut elem: *mut HashGridElem = *((*cell).elems_data).offset(i as isize);
        if ((*this).results_capacity == (*this).results_size) as i32 as libc::c_long != 0 {
            (*this).results_capacity = if (*this).results_capacity != 0 {
                (*this).results_capacity * 2_i32
            } else {
                1_i32
            };
            let mut elemSize: usize = std::mem::size_of::<*mut libc::c_void>();
            let mut pData: *mut *mut libc::c_void =
                &mut (*this).results_data as *mut *mut *mut libc::c_void as *mut *mut libc::c_void;
            *pData = MemRealloc(
                (*this).results_data as *mut libc::c_void,
                ((*this).results_capacity as usize).wrapping_mul(elemSize),
            );
        }
        let fresh10 = (*this).results_size;
        (*this).results_size += 1;
        let ref mut fresh11 = *((*this).results_data).offset(fresh10 as isize);
        *fresh11 = (*elem).object;
        i += 1;
    }
    (*this).results_size
}
