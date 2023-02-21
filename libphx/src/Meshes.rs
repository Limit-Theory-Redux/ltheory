use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
use glam::Vec2;

extern "C" {
    pub type Mesh;
    fn Mesh_Create() -> *mut Mesh;
    fn Mesh_AddQuad(
        _: *mut Mesh,
        _: libc::c_int,
        _: libc::c_int,
        _: libc::c_int,
        _: libc::c_int,
    );
    fn Mesh_AddVertex(
        _: *mut Mesh,
        px: libc::c_float,
        py: libc::c_float,
        pz: libc::c_float,
        nx: libc::c_float,
        ny: libc::c_float,
        nz: libc::c_float,
        u: libc::c_float,
        v: libc::c_float,
    );
    fn Mesh_GetVertexCount(_: *mut Mesh) -> libc::c_int;
    fn Mesh_GetVertexData(_: *mut Mesh) -> *mut Vertex;
    fn sqrt(_: libc::c_double) -> libc::c_double;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub p: Vec3,
    pub n: Vec3,
    pub uv: Vec2,
}
#[inline]
unsafe extern "C" fn Sqrtf(mut t: libc::c_float) -> libc::c_float {
    return sqrt(t as libc::c_double) as libc::c_float;
}
#[inline]
unsafe extern "C" fn Mesh_AddPlane(
    mut this: *mut Mesh,
    mut origin: Vec3,
    mut du: Vec3,
    mut dv: Vec3,
    mut resU: libc::c_int,
    mut resV: libc::c_int,
) {
    let mut n: Vec3 = Vec3::cross(du, dv).normalize();
    let mut iu: libc::c_int = 0 as libc::c_int;
    while iu < resU {
        let mut u: libc::c_float = iu as libc::c_float
            / (resU - 1 as libc::c_int) as libc::c_float;
        let mut iv: libc::c_int = 0 as libc::c_int;
        while iv < resV {
            let mut v: libc::c_float = iv as libc::c_float
                / (resV - 1 as libc::c_int) as libc::c_float;
            let mut p: Vec3 = origin + du * u + dv * v;
            if iu != 0 && iv != 0 {
                let mut vc: libc::c_int = Mesh_GetVertexCount(this);
                Mesh_AddQuad(
                    this,
                    vc,
                    vc - resV,
                    vc - resV - 1 as libc::c_int,
                    vc - 1 as libc::c_int,
                );
            }
            Mesh_AddVertex(this, p.x, p.y, p.z, n.x, n.y, n.z, u, v);
            iv += 1;
        }
        iu += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Box(mut res: libc::c_int) -> *mut Mesh {
    let origin: [Vec3; 6] = [
        {
            let mut init = Vec3 {
                x: -(1 as libc::c_int) as libc::c_float,
                y: -(1 as libc::c_int) as libc::c_float,
                z: 1 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: -(1 as libc::c_int) as libc::c_float,
                y: -(1 as libc::c_int) as libc::c_float,
                z: -(1 as libc::c_int) as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 1 as libc::c_int as libc::c_float,
                y: -(1 as libc::c_int) as libc::c_float,
                z: -(1 as libc::c_int) as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: -(1 as libc::c_int) as libc::c_float,
                y: -(1 as libc::c_int) as libc::c_float,
                z: -(1 as libc::c_int) as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: -(1 as libc::c_int) as libc::c_float,
                y: 1 as libc::c_int as libc::c_float,
                z: -(1 as libc::c_int) as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: -(1 as libc::c_int) as libc::c_float,
                y: -(1 as libc::c_int) as libc::c_float,
                z: -(1 as libc::c_int) as libc::c_float,
            };
            init
        },
    ];
    let du: [Vec3; 6] = [
        {
            let mut init = Vec3 {
                x: 2 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 0 as libc::c_int as libc::c_float,
                y: 2 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 0 as libc::c_int as libc::c_float,
                y: 2 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 0 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 2 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 0 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 2 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 2 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
    ];
    let dv: [Vec3; 6] = [
        {
            let mut init = Vec3 {
                x: 0 as libc::c_int as libc::c_float,
                y: 2 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 2 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 0 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 2 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 0 as libc::c_int as libc::c_float,
                y: 2 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 2 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: 0 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 2 as libc::c_int as libc::c_float,
            };
            init
        },
    ];
    let mut this: *mut Mesh = Mesh_Create();
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 6 as libc::c_int {
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
pub unsafe extern "C" fn Mesh_BoxSphere(mut res: libc::c_int) -> *mut Mesh {
    let mut this: *mut Mesh = Mesh_Box(res);
    let mut vertexCount: libc::c_int = Mesh_GetVertexCount(this);
    let mut vertexData: *mut Vertex = Mesh_GetVertexData(this);
    let mut i: libc::c_int = 0 as libc::c_int;
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
    mut resU: libc::c_int,
    mut resV: libc::c_int,
) -> *mut Mesh {
    let mut this: *mut Mesh = Mesh_Create();
    Mesh_AddPlane(this, origin, du, dv, resU, resV);
    return this;
}
