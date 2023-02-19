use ::libc;
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
pub struct Vec3f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vertex {
    pub p: Vec3f,
    pub n: Vec3f,
    pub uv: Vec2,
}
#[inline]
unsafe extern "C" fn Vec3f_Normalize(mut v: Vec3f) -> Vec3f {
    let mut l: libc::c_float = Vec3f_Length(v);
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: v.x / l,
            y: v.y / l,
            z: v.z / l,
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Vec3f_Length(mut v: Vec3f) -> libc::c_float {
    return Sqrtf(v.x * v.x + v.y * v.y + v.z * v.z);
}
#[inline]
unsafe extern "C" fn Vec3f_Cross(mut a: Vec3f, mut b: Vec3f) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: b.z * a.y - b.y * a.z,
            y: b.x * a.z - b.z * a.x,
            z: b.y * a.x - b.x * a.y,
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Vec3f_Add(mut a: Vec3f, mut b: Vec3f) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: a.x + b.x,
            y: a.y + b.y,
            z: a.z + b.z,
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Vec3f_Muls(mut a: Vec3f, mut b: libc::c_float) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: a.x * b,
            y: a.y * b,
            z: a.z * b,
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Sqrtf(mut t: libc::c_float) -> libc::c_float {
    return sqrt(t as libc::c_double) as libc::c_float;
}
#[inline]
unsafe extern "C" fn Mesh_AddPlane(
    mut self_0: *mut Mesh,
    mut origin: Vec3f,
    mut du: Vec3f,
    mut dv: Vec3f,
    mut resU: libc::c_int,
    mut resV: libc::c_int,
) {
    let mut n: Vec3f = Vec3f_Normalize(Vec3f_Cross(du, dv));
    let mut iu: libc::c_int = 0 as libc::c_int;
    while iu < resU {
        let mut u: libc::c_float = iu as libc::c_float
            / (resU - 1 as libc::c_int) as libc::c_float;
        let mut iv: libc::c_int = 0 as libc::c_int;
        while iv < resV {
            let mut v: libc::c_float = iv as libc::c_float
                / (resV - 1 as libc::c_int) as libc::c_float;
            let mut p: Vec3f = Vec3f_Add(
                origin,
                Vec3f_Add(Vec3f_Muls(du, u), Vec3f_Muls(dv, v)),
            );
            if iu != 0 && iv != 0 {
                let mut vc: libc::c_int = Mesh_GetVertexCount(self_0);
                Mesh_AddQuad(
                    self_0,
                    vc,
                    vc - resV,
                    vc - resV - 1 as libc::c_int,
                    vc - 1 as libc::c_int,
                );
            }
            Mesh_AddVertex(self_0, p.x, p.y, p.z, n.x, n.y, n.z, u, v);
            iv += 1;
        }
        iu += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Box(mut res: libc::c_int) -> *mut Mesh {
    let origin: [Vec3f; 6] = [
        {
            let mut init = Vec3f {
                x: -(1 as libc::c_int) as libc::c_float,
                y: -(1 as libc::c_int) as libc::c_float,
                z: 1 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: -(1 as libc::c_int) as libc::c_float,
                y: -(1 as libc::c_int) as libc::c_float,
                z: -(1 as libc::c_int) as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: 1 as libc::c_int as libc::c_float,
                y: -(1 as libc::c_int) as libc::c_float,
                z: -(1 as libc::c_int) as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: -(1 as libc::c_int) as libc::c_float,
                y: -(1 as libc::c_int) as libc::c_float,
                z: -(1 as libc::c_int) as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: -(1 as libc::c_int) as libc::c_float,
                y: 1 as libc::c_int as libc::c_float,
                z: -(1 as libc::c_int) as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: -(1 as libc::c_int) as libc::c_float,
                y: -(1 as libc::c_int) as libc::c_float,
                z: -(1 as libc::c_int) as libc::c_float,
            };
            init
        },
    ];
    let du: [Vec3f; 6] = [
        {
            let mut init = Vec3f {
                x: 2 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: 0 as libc::c_int as libc::c_float,
                y: 2 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: 0 as libc::c_int as libc::c_float,
                y: 2 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: 0 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 2 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: 0 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 2 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: 2 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
    ];
    let dv: [Vec3f; 6] = [
        {
            let mut init = Vec3f {
                x: 0 as libc::c_int as libc::c_float,
                y: 2 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: 2 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: 0 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 2 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: 0 as libc::c_int as libc::c_float,
                y: 2 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: 2 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: 0 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 2 as libc::c_int as libc::c_float,
            };
            init
        },
    ];
    let mut self_0: *mut Mesh = Mesh_Create();
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 6 as libc::c_int {
        Mesh_AddPlane(
            self_0,
            origin[i as usize],
            du[i as usize],
            dv[i as usize],
            res,
            res,
        );
        i += 1;
    }
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_BoxSphere(mut res: libc::c_int) -> *mut Mesh {
    let mut self_0: *mut Mesh = Mesh_Box(res);
    let mut vertexCount: libc::c_int = Mesh_GetVertexCount(self_0);
    let mut vertexData: *mut Vertex = Mesh_GetVertexData(self_0);
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < vertexCount {
        let mut vertex: *mut Vertex = vertexData.offset(i as isize);
        (*vertex).p = Vec3f_Normalize((*vertex).p);
        i += 1;
    }
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn Mesh_Plane(
    mut origin: Vec3f,
    mut du: Vec3f,
    mut dv: Vec3f,
    mut resU: libc::c_int,
    mut resV: libc::c_int,
) -> *mut Mesh {
    let mut self_0: *mut Mesh = Mesh_Create();
    Mesh_AddPlane(self_0, origin, du, dv, resU, resV);
    return self_0;
}
