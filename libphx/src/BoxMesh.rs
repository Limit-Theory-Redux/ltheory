use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use crate::Matrix::*;
use crate::Mesh::*;
use libc;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct BoxMesh {
    pub elem_size: i32,
    pub elem_capacity: i32,
    pub elem_data: *mut Box_0,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Box_0 {
    pub p: Vec3,
    pub s: Vec3,
    pub r: Vec3,
    pub b: Vec3,
}

static mut kFaceOrigin: [Vec3; 6] = [
    Vec3::new(-1.0f32, -1.0f32, 1.0f32),
    Vec3::new(-1.0f32, -1.0f32, -1.0f32),
    Vec3::new(1.0f32, -1.0f32, -1.0f32),
    Vec3::new(-1.0f32, -1.0f32, -1.0f32),
    Vec3::new(-1.0f32, 1.0f32, -1.0f32),
    Vec3::new(-1.0f32, -1.0f32, -1.0f32),
];

static mut kFaceU: [Vec3; 6] = [
    Vec3::new(2.0f32, 0.0f32, 0.0f32),
    Vec3::new(0.0f32, 2.0f32, 0.0f32),
    Vec3::new(0.0f32, 2.0f32, 0.0f32),
    Vec3::new(0.0f32, 0.0f32, 2.0f32),
    Vec3::new(0.0f32, 0.0f32, 2.0f32),
    Vec3::new(2.0f32, 0.0f32, 0.0f32),
];

static mut kFaceV: [Vec3; 6] = [
    Vec3::new(0.0f32, 2.0f32, 0.0f32),
    Vec3::new(2.0f32, 0.0f32, 0.0f32),
    Vec3::new(0.0f32, 0.0f32, 2.0f32),
    Vec3::new(0.0f32, 2.0f32, 0.0f32),
    Vec3::new(2.0f32, 0.0f32, 0.0f32),
    Vec3::new(0.0f32, 0.0f32, 2.0f32),
];

#[no_mangle]
pub unsafe extern "C" fn BoxMesh_Create() -> *mut BoxMesh {
    let mut this: *mut BoxMesh = MemAlloc(std::mem::size_of::<BoxMesh>()) as *mut BoxMesh;
    (*this).elem_capacity = 0_i32;
    (*this).elem_size = 0_i32;
    (*this).elem_data = std::ptr::null_mut();
    this
}

#[no_mangle]
pub unsafe extern "C" fn BoxMesh_Free(mut this: *mut BoxMesh) {
    MemFree((*this).elem_data as *const libc::c_void);
    MemFree(this as *const libc::c_void);
}

#[no_mangle]
pub unsafe extern "C" fn BoxMesh_Add(
    mut this: *mut BoxMesh,
    mut p: *const Vec3,
    mut s: *const Vec3,
    mut r: *const Vec3,
    mut b: *const Vec3,
) {
    if ((*this).elem_capacity == (*this).elem_size) as libc::c_long != 0 {
        (*this).elem_capacity = if (*this).elem_capacity != 0 {
            (*this).elem_capacity * 2_i32
        } else {
            1_i32
        };
        let mut elemSize: usize = std::mem::size_of::<Box_0>();
        let mut pData: *mut *mut libc::c_void =
            &mut (*this).elem_data as *mut *mut Box_0 as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*this).elem_data as *mut libc::c_void,
            ((*this).elem_capacity as usize).wrapping_mul(elemSize),
        );
    }
    let fresh0 = (*this).elem_size;
    (*this).elem_size += 1;
    let mut box_0: *mut Box_0 = ((*this).elem_data).offset(fresh0 as isize);
    (*box_0).p = *p;
    (*box_0).s = *s;
    (*box_0).r = *r;
    (*box_0).b = *b;
}

#[no_mangle]
pub unsafe extern "C" fn BoxMesh_GetMesh(mut this: *mut BoxMesh, mut res: i32) -> *mut Mesh {
    let mut mesh: *mut Mesh = Mesh_Create();
    Mesh_ReserveVertexData(mesh, 6_i32 * res * res * (*this).elem_size);
    Mesh_ReserveIndexData(mesh, 12_i32 * (res - 1_i32) * (res - 1_i32));
    let mut i: i32 = 0_i32;
    while i < (*this).elem_size {
        let mut box_0: *mut Box_0 = ((*this).elem_data).offset(i as isize);
        let mut lower: Vec3 = Vec3::new(
            (*box_0).b.x - 1.0f32,
            (*box_0).b.y - 1.0f32,
            (*box_0).b.z - 1.0f32,
        );
        let mut upper: Vec3 = Vec3::new(
            1.0f32 - (*box_0).b.x,
            1.0f32 - (*box_0).b.y,
            1.0f32 - (*box_0).b.z,
        );
        let mut rot: *mut Matrix = Matrix_YawPitchRoll((*box_0).r.x, (*box_0).r.y, (*box_0).r.z);
        let mut face: i32 = 0_i32;
        while face < 6_i32 {
            let mut o: Vec3 = kFaceOrigin[face as usize];
            let mut du: Vec3 = kFaceU[face as usize];
            let mut dv: Vec3 = kFaceV[face as usize];
            let mut n: Vec3 = Vec3::cross(du, dv).normalize();
            let mut iu: i32 = 0_i32;
            while iu < res {
                let mut u: f32 = iu as f32 / (res - 1_i32) as f32;
                let mut iv: i32 = 0_i32;
                while iv < res {
                    let mut v: f32 = iv as f32 / (res - 1_i32) as f32;
                    let mut p: Vec3 = o + (du * u) + (dv * v);
                    let mut clamped: Vec3 = Vec3::clamp(p, lower, upper);
                    let mut proj: Vec3 = p - clamped;
                    p = clamped + (proj.normalize() * (*box_0).b);
                    p *= (*box_0).s;
                    let mut rp = Vec3::ZERO;
                    Matrix_MulPoint(rot, &mut rp, p.x, p.y, p.z);
                    p = rp + (*box_0).p;
                    if iu != 0 && iv != 0 {
                        let mut off: i32 = Mesh_GetVertexCount(mesh);
                        Mesh_AddQuad(mesh, off, off - res, off - res - 1_i32, off - 1_i32);
                    }
                    Mesh_AddVertex(mesh, p.x, p.y, p.z, n.x, n.y, n.z, u, v);
                    iv += 1;
                }
                iu += 1;
            }
            face += 1;
        }
        Matrix_Free(rot);
        i += 1;
    }
    mesh
}
