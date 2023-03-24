use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use crate::Matrix::*;
use crate::Mesh::*;
use libc;

#[derive(Clone)]
#[repr(C)]
pub struct BoxMesh {
    pub elem: Vec<Box_0>,
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
    let mut this = MemNew!(BoxMesh);
    (*this).elem = Vec::new();
    this
}

#[no_mangle]
pub unsafe extern "C" fn BoxMesh_Free(mut this: *mut BoxMesh) {
    MemFree(this as *const _);
}

#[no_mangle]
pub unsafe extern "C" fn BoxMesh_Add(
    mut this: *mut BoxMesh,
    p: *const Vec3,
    s: *const Vec3,
    r: *const Vec3,
    b: *const Vec3,
) {
    (*this).elem.push(Box_0 { p: *p, s: *s, r: *r, b: *b });
}

#[no_mangle]
pub unsafe extern "C" fn BoxMesh_GetMesh(mut this: *mut BoxMesh, mut res: i32) -> *mut Mesh {
    let mut mesh: *mut Mesh = Mesh_Create();
    Mesh_ReserveVertexData(mesh, 6 * res * res * (*this).elem.len() as i32);
    Mesh_ReserveIndexData(mesh, 12 * (res - 1) * (res - 1));
    for box_0 in (*this).elem.iter() {
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
        let mut face: i32 = 0;
        while face < 6 {
            let mut o: Vec3 = kFaceOrigin[face as usize];
            let mut du: Vec3 = kFaceU[face as usize];
            let mut dv: Vec3 = kFaceV[face as usize];
            let mut n: Vec3 = Vec3::cross(du, dv).normalize();
            let mut iu: i32 = 0;
            while iu < res {
                let mut u: f32 = iu as f32 / (res - 1) as f32;
                let mut iv: i32 = 0;
                while iv < res {
                    let mut v: f32 = iv as f32 / (res - 1) as f32;
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
                        Mesh_AddQuad(mesh, off, off - res, off - res - 1, off - 1);
                    }
                    Mesh_AddVertex(mesh, p.x, p.y, p.z, n.x, n.y, n.z, u, v);
                    iv += 1;
                }
                iu += 1;
            }
            face += 1;
        }
        Matrix_Free(rot);
    }
    mesh
}
