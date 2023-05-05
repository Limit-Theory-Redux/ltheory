use crate::phx::internal::Memory::*;
use crate::phx::Common::*;
use crate::phx::Math::Vec3;
use crate::phx::Mesh::*;

/* --- LodMesh -----------------------------------------------------------------
 *
 *   A basic container for abstracting LOD rendering behavior. LodMesh consists
 *   of any number of (Mesh, distMin, distMax) tuples. Drawing a requires
 *   passing a *distance squared* argument that is used to determine which
 *   component(s) of the LodMesh to draw.
 *
 *   This type is REFERENCE-COUNTED. See ../doc/RefCounted.txt for details.
 *
 * -------------------------------------------------------------------------- */

/* TODO : Merge meshes into single IBO/VBO so that we can skip all the rebinds
 *        (profiling shows that they are a huge perf drain in the rendering
 *         pipeline) */

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
    let this = MemNew!(LodMesh);
    (*this)._refCount = 1;
    (*this).head = std::ptr::null_mut();
    this
}

#[no_mangle]
pub extern "C" fn LodMesh_Acquire(this: &mut LodMesh) {
    this._refCount = (this._refCount).wrapping_add(1);
}

#[no_mangle]
pub unsafe extern "C" fn LodMesh_Free(this: *mut LodMesh) {
    if !this.is_null() && {
        (*this)._refCount = ((*this)._refCount).wrapping_sub(1);
        (*this)._refCount <= 0
    } {
        let mut e: *mut LodMeshEntry = (*this).head;
        while !e.is_null() {
            let next: *mut LodMeshEntry = (*e).next;
            Mesh_Free(&mut *(*e).mesh);
            MemFree(e as *const _);
            e = next;
        }
        MemFree(this as *const _);
    }
}

#[no_mangle]
pub unsafe extern "C" fn LodMesh_Add(this: &mut LodMesh, mesh: *mut Mesh, dMin: f32, dMax: f32) {
    let e = MemNew!(LodMeshEntry);
    (*e).mesh = mesh;
    (*e).dMin = dMin * dMin;
    (*e).dMax = dMax * dMax;
    (*e).next = this.head;
    this.head = e;
}

#[no_mangle]
pub unsafe extern "C" fn LodMesh_Draw(this: &mut LodMesh, d2: f32) {
    let mut e: *mut LodMeshEntry = this.head;
    while !e.is_null() {
        if (*e).dMin <= d2 && d2 <= (*e).dMax {
            Mesh_Draw(&mut *(*e).mesh);
        }
        e = (*e).next;
    }
}

#[no_mangle]
pub unsafe extern "C" fn LodMesh_Get(this: &mut LodMesh, d2: f32) -> *mut Mesh {
    let mut e: *mut LodMeshEntry = this.head;
    while !e.is_null() {
        if (*e).dMin <= d2 && d2 <= (*e).dMax {
            return (*e).mesh;
        }
        e = (*e).next;
    }
    std::ptr::null_mut()
}
