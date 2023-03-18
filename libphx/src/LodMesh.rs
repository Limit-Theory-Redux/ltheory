use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use crate::Mesh::*;
use libc;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct LodMesh {
    pub _refCount: u32,
    pub head: *mut LodMeshEntry,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct LodMeshEntry {
    pub next: *mut LodMeshEntry,
    pub mesh: *mut Mesh,
    pub dMin: f32,
    pub dMax: f32,
}

#[no_mangle]
pub unsafe extern "C" fn LodMesh_Create() -> *mut LodMesh {
    let mut this: *mut LodMesh = MemAlloc(::core::mem::size_of::<LodMesh>()) as *mut LodMesh;
    (*this)._refCount = 1_i32 as u32;
    (*this).head = std::ptr::null_mut();
    this
}

#[no_mangle]
pub unsafe extern "C" fn LodMesh_Acquire(mut this: *mut LodMesh) {
    (*this)._refCount = ((*this)._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn LodMesh_Free(mut this: *mut LodMesh) {
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount <= 0_i32 as u32
    } {
        let mut e: *mut LodMeshEntry = (*this).head;
        while !e.is_null() {
            let mut next: *mut LodMeshEntry = (*e).next;
            Mesh_Free((*e).mesh);
            MemFree(e as *const libc::c_void);
            e = next;
        }
        MemFree(this as *const libc::c_void);
    }
}

#[no_mangle]
pub unsafe extern "C" fn LodMesh_Add(
    mut this: *mut LodMesh,
    mut mesh: *mut Mesh,
    mut dMin: f32,
    mut dMax: f32,
) {
    let mut e: *mut LodMeshEntry =
        MemAlloc(::core::mem::size_of::<LodMeshEntry>()) as *mut LodMeshEntry;
    (*e).mesh = mesh;
    (*e).dMin = dMin * dMin;
    (*e).dMax = dMax * dMax;
    (*e).next = (*this).head;
    (*this).head = e;
}

#[no_mangle]
pub unsafe extern "C" fn LodMesh_Draw(mut this: *mut LodMesh, mut d2: f32) {
    let mut e: *mut LodMeshEntry = (*this).head;
    while !e.is_null() {
        if (*e).dMin <= d2 && d2 <= (*e).dMax {
            Mesh_Draw((*e).mesh);
        }
        e = (*e).next;
    }
}

#[no_mangle]
pub unsafe extern "C" fn LodMesh_Get(mut this: *mut LodMesh, mut d2: f32) -> *mut Mesh {
    let mut e: *mut LodMeshEntry = (*this).head;
    while !e.is_null() {
        if (*e).dMin <= d2 && d2 <= (*e).dMax {
            return (*e).mesh;
        }
        e = (*e).next;
    }
    std::ptr::null_mut()
}
