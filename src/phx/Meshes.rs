


use crate::phx::Math::Vec3;
use crate::phx::Mesh::*;

#[inline]
extern "C" fn Mesh_AddPlane(
    this: &mut Mesh,
    origin: Vec3,
    du: Vec3,
    dv: Vec3,
    resU: i32,
    resV: i32,
) {
    let n: Vec3 = Vec3::cross(du, dv).normalize();
    let mut iu: i32 = 0;
    while iu < resU {
        let u: f32 = iu as f32 / (resU - 1) as f32;
        let mut iv: i32 = 0;
        while iv < resV {
            let v: f32 = iv as f32 / (resV - 1) as f32;
            let p: Vec3 = origin + du * u + dv * v;
            if iu != 0 && iv != 0 {
                let vc: i32 = Mesh_GetVertexCount(this);
                Mesh_AddQuad(this, vc, vc - resV, vc - resV - 1, vc - 1);
            }
            Mesh_AddVertex(this, p.x, p.y, p.z, n.x, n.y, n.z, u, v);
            iv += 1;
        }
        iu += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Box(res: i32) -> *mut Mesh {
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
    let this: *mut Mesh = Mesh_Create();
    let mut i: i32 = 0;
    while i < 6 {
        Mesh_AddPlane(
            &mut *this,
            origin[i as usize],
            du[i as usize],
            dv[i as usize],
            res,
            res,
        );
        i += 1;
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_BoxSphere(res: i32) -> *mut Mesh {
    let this: *mut Mesh = Mesh_Box(res);
    let vertexCount: i32 = Mesh_GetVertexCount(&mut *this);
    let vertexData: *mut Vertex = Mesh_GetVertexData(&mut *this);
    let mut i: i32 = 0;
    while i < vertexCount {
        let vertex: *mut Vertex = vertexData.offset(i as isize);
        (*vertex).p = (*vertex).p.normalize();
        i += 1;
    }
    this
}

#[no_mangle]
pub unsafe extern "C" fn Mesh_Plane(
    origin: Vec3,
    du: Vec3,
    dv: Vec3,
    resU: i32,
    resV: i32,
) -> *mut Mesh {
    let this: *mut Mesh = Mesh_Create();
    Mesh_AddPlane(&mut *this, origin, du, dv, resU, resV);
    this
}
