use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
extern "C" {
    pub type Mesh;
    fn Mesh_Free(_: *mut Mesh);
    fn Mesh_Draw(_: *mut Mesh);
}
pub type uint32_t = libc::c_uint;
pub type uint32 = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LodMesh {
    pub _refCount: uint32,
    pub head: *mut LodMeshEntry,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LodMeshEntry {
    pub next: *mut LodMeshEntry,
    pub mesh: *mut Mesh,
    pub dMin: libc::c_float,
    pub dMax: libc::c_float,
}

#[no_mangle]
pub unsafe extern "C" fn LodMesh_Create() -> *mut LodMesh {
    let mut self_0: *mut LodMesh = MemAlloc(
        ::core::mem::size_of::<LodMesh>() as usize,
    ) as *mut LodMesh;
    (*self_0)._refCount = 1 as libc::c_int as uint32;
    (*self_0).head = 0 as *mut LodMeshEntry;
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn LodMesh_Acquire(mut self_0: *mut LodMesh) {
    (*self_0)._refCount = ((*self_0)._refCount).wrapping_add(1);
}
#[no_mangle]
pub unsafe extern "C" fn LodMesh_Free(mut self_0: *mut LodMesh) {
    if !self_0.is_null()
        && {
            (*self_0)._refCount = ((*self_0)._refCount).wrapping_sub(1);
            (*self_0)._refCount <= 0 as libc::c_int as libc::c_uint
        }
    {
        let mut e: *mut LodMeshEntry = (*self_0).head;
        while !e.is_null() {
            let mut next: *mut LodMeshEntry = (*e).next;
            Mesh_Free((*e).mesh);
            MemFree(e as *const libc::c_void);
            e = next;
        }
        MemFree(self_0 as *const libc::c_void);
    }
}
#[no_mangle]
pub unsafe extern "C" fn LodMesh_Add(
    mut self_0: *mut LodMesh,
    mut mesh: *mut Mesh,
    mut dMin: libc::c_float,
    mut dMax: libc::c_float,
) {
    let mut e: *mut LodMeshEntry = MemAlloc(
        ::core::mem::size_of::<LodMeshEntry>() as usize,
    ) as *mut LodMeshEntry;
    (*e).mesh = mesh;
    (*e).dMin = dMin * dMin;
    (*e).dMax = dMax * dMax;
    (*e).next = (*self_0).head;
    (*self_0).head = e;
}
#[no_mangle]
pub unsafe extern "C" fn LodMesh_Draw(mut self_0: *mut LodMesh, mut d2: libc::c_float) {
    let mut e: *mut LodMeshEntry = (*self_0).head;
    while !e.is_null() {
        if (*e).dMin <= d2 && d2 <= (*e).dMax {
            Mesh_Draw((*e).mesh);
        }
        e = (*e).next;
    }
}
#[no_mangle]
pub unsafe extern "C" fn LodMesh_Get(
    mut self_0: *mut LodMesh,
    mut d2: libc::c_float,
) -> *mut Mesh {
    let mut e: *mut LodMeshEntry = (*self_0).head;
    while !e.is_null() {
        if (*e).dMin <= d2 && d2 <= (*e).dMax {
            return (*e).mesh;
        }
        e = (*e).next;
    }
    return 0 as *mut Mesh;
}
