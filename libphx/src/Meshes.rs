use crate::internal::Memory::*;
use crate::Mesh::*;
use glam::Vec2;
use glam::Vec3;
use libc;

extern "C" {
    fn sqrt(_: f64) -> f64;
}

#[inline]
unsafe extern "C" fn Sqrtf(mut t: f32) -> f32 {
    return sqrt(t as f64) as f32;
}

#[inline]
unsafe extern "C" fn Mesh_AddPlane(
    mut this: *mut Mesh,
    mut origin: Vec3,
    mut du: Vec3,
    mut dv: Vec3,
    mut resU: i32,
    mut resV: i32,
) {
    let mut n: Vec3 = Vec3::cross(du, dv).normalize();
    let mut iu: i32 = 0_i32;
    while iu < resU {
        let mut u: f32 = iu as f32 / (resU - 1_i32) as f32;
        let mut iv: i32 = 0_i32;
        while iv < resV {
            let mut v: f32 = iv as f32 / (resV - 1_i32) as f32;
            let mut p: Vec3 = origin + du * u + dv * v;
            if iu != 0 && iv != 0 {
                let mut vc: i32 = Mesh_GetVertexCount(this);
                Mesh_AddQuad(this, vc, vc - resV, vc - resV - 1_i32, vc - 1_i32);
            }
            Mesh_AddVertex(this, p.x, p.y, p.z, n.x, n.y, n.z, u, v);
            iv += 1;
        }
        iu += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Box(mut res: i32) -> *mut Mesh {
    let origin: [Vec3; 6] = [
        Vec3::new(-1.0f32, -1.0f32, 1.0f32),
        Vec3::new(-1.0f32, -1.0f32, -1.0f32),
        Vec3::new(1.0f32, -1.0f32, -1.0f32),
        Vec3::new(-1.0f32, -1.0f32, -1.0f32),
        Vec3::new(-1.0f32, 1.0f32, -1.0f32),
        Vec3::new(-1.0f32, -1.0f32, -1.0f32),
    ];
    let du: [Vec3; 6] = [
        Vec3::new(2.0f32, 0.0f32, 0.0f32),
        Vec3::new(0.0f32, 2.0f32, 0.0f32),
        Vec3::new(0.0f32, 2.0f32, 0.0f32),
        Vec3::new(0.0f32, 0.0f32, 2.0f32),
        Vec3::new(0.0f32, 0.0f32, 2.0f32),
        Vec3::new(2.0f32, 0.0f32, 0.0f32),
    ];
    let dv: [Vec3; 6] = [
        Vec3::new(0.0f32, 2.0f32, 0.0f32),
        Vec3::new(2.0f32, 0.0f32, 0.0f32),
        Vec3::new(0.0f32, 0.0f32, 2.0f32),
        Vec3::new(0.0f32, 2.0f32, 0.0f32),
        Vec3::new(2.0f32, 0.0f32, 0.0f32),
        Vec3::new(0.0f32, 0.0f32, 2.0f32),
    ];
    let mut this: *mut Mesh = Mesh_Create();
    let mut i: i32 = 0_i32;
    while i < 6_i32 {
        Mesh_AddPlane(
            this,
            origin[i as usize],
            du[i as usize],
            dv[i as usize],
            res,
            res,
        );
        i += 1;
    }
    return this;
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_BoxSphere(mut res: i32) -> *mut Mesh {
    let mut this: *mut Mesh = Mesh_Box(res);
    let mut vertexCount: i32 = Mesh_GetVertexCount(this);
    let mut vertexData: *mut Vertex = Mesh_GetVertexData(this);
    let mut i: i32 = 0_i32;
    while i < vertexCount {
        let mut vertex: *mut Vertex = vertexData.offset(i as isize);
        (*vertex).p = (*vertex).p.normalize();
        i += 1;
    }
    return this;
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Plane(
    mut origin: Vec3,
    mut du: Vec3,
    mut dv: Vec3,
    mut resU: i32,
    mut resV: i32,
) -> *mut Mesh {
    let mut this: *mut Mesh = Mesh_Create();
    Mesh_AddPlane(this, origin, du, dv, resU, resV);
    return this;
}
