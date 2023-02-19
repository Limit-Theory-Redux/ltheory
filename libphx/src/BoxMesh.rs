use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;


extern "C" {
    pub type Mesh;
    pub type Matrix;
    
    fn Matrix_Free(_: *mut Matrix);
    fn Matrix_YawPitchRoll(
        yaw: libc::c_float,
        pitch: libc::c_float,
        roll: libc::c_float,
    ) -> *mut Matrix;
    fn Matrix_MulPoint(
        _: *const Matrix,
        out: *mut Vec3,
        x: libc::c_float,
        y: libc::c_float,
        z: libc::c_float,
    );
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
    fn Mesh_ReserveIndexData(_: *mut Mesh, capacity: libc::c_int);
    fn Mesh_ReserveVertexData(_: *mut Mesh, capacity: libc::c_int);
    fn sqrt(_: libc::c_double) -> libc::c_double;
}
pub type int32_t = libc::c_int;
pub type int32 = int32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct BoxMesh {
    pub elem_size: int32,
    pub elem_capacity: int32,
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


#[inline]
unsafe extern "C" fn Sqrtf(mut t: libc::c_float) -> libc::c_float {
    return sqrt(t as libc::c_double) as libc::c_float;
}
#[inline]
unsafe extern "C" fn Clampf(
    mut t: libc::c_float,
    mut lower: libc::c_float,
    mut upper: libc::c_float,
) -> libc::c_float {
    t = if t > upper { upper } else { t };
    t = if t < lower { lower } else { t };
    return t;
}
static mut kFaceOrigin: [Vec3; 6] = [
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
static mut kFaceU: [Vec3; 6] = [
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
static mut kFaceV: [Vec3; 6] = [
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
#[no_mangle]
pub unsafe extern "C" fn BoxMesh_Create() -> *mut BoxMesh {
    let mut self_0: *mut BoxMesh = MemAlloc(
        ::core::mem::size_of::<BoxMesh>() as usize,
    ) as *mut BoxMesh;
    (*self_0).elem_capacity = 0 as libc::c_int;
    (*self_0).elem_size = 0 as libc::c_int;
    (*self_0).elem_data = 0 as *mut Box_0;
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn BoxMesh_Free(mut self_0: *mut BoxMesh) {
    MemFree((*self_0).elem_data as *const libc::c_void);
    MemFree(self_0 as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn BoxMesh_Add(
    mut self_0: *mut BoxMesh,
    mut p: *const Vec3,
    mut s: *const Vec3,
    mut r: *const Vec3,
    mut b: *const Vec3,
) {
    if ((*self_0).elem_capacity == (*self_0).elem_size) as libc::c_int as libc::c_long
        != 0
    {
        (*self_0)
            .elem_capacity = if (*self_0).elem_capacity != 0 {
            (*self_0).elem_capacity * 2 as libc::c_int
        } else {
            1 as libc::c_int
        };
        let mut elemSize: usize = ::core::mem::size_of::<Box_0>();
        let mut pData: *mut *mut libc::c_void = &mut (*self_0).elem_data
            as *mut *mut Box_0 as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*self_0).elem_data as *mut libc::c_void,
            ((*self_0).elem_capacity as usize).wrapping_mul(elemSize as usize),
        );
    }
    let fresh0 = (*self_0).elem_size;
    (*self_0).elem_size = (*self_0).elem_size + 1;
    let mut box_0: *mut Box_0 = ((*self_0).elem_data).offset(fresh0 as isize);
    (*box_0).p = *p;
    (*box_0).s = *s;
    (*box_0).r = *r;
    (*box_0).b = *b;
}
#[no_mangle]
pub unsafe extern "C" fn BoxMesh_GetMesh(
    mut self_0: *mut BoxMesh,
    mut res: libc::c_int,
) -> *mut Mesh {
    let mut mesh: *mut Mesh = Mesh_Create();
    Mesh_ReserveVertexData(mesh, 6 as libc::c_int * res * res * (*self_0).elem_size);
    Mesh_ReserveIndexData(
        mesh,
        12 as libc::c_int * (res - 1 as libc::c_int) * (res - 1 as libc::c_int),
    );
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < (*self_0).elem_size {
        let mut box_0: *mut Box_0 = ((*self_0).elem_data).offset(i as isize);
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
        let mut rot: *mut Matrix = Matrix_YawPitchRoll(
            (*box_0).r.x,
            (*box_0).r.y,
            (*box_0).r.z,
        );
        let mut face: libc::c_int = 0 as libc::c_int;
        while face < 6 as libc::c_int {
            let mut o: Vec3 = kFaceOrigin[face as usize];
            let mut du: Vec3 = kFaceU[face as usize];
            let mut dv: Vec3 = kFaceV[face as usize];
            let mut n: Vec3 = Vec3::cross(du, dv).normalize();
            let mut iu: libc::c_int = 0 as libc::c_int;
            while iu < res {
                let mut u: libc::c_float = iu as libc::c_float
                    / (res - 1 as libc::c_int) as libc::c_float;
                let mut iv: libc::c_int = 0 as libc::c_int;
                while iv < res {
                    let mut v: libc::c_float = iv as libc::c_float
                        / (res - 1 as libc::c_int) as libc::c_float;
                    let mut p: Vec3 = o + (du * u) + (dv * v);
                    let mut clamped: Vec3 = Vec3::clamp(p, lower, upper);
                    let mut proj: Vec3 = p - clamped;
                    p = clamped + (proj.normalize() * (*box_0).b);
                    p *= (*box_0).s;
                    let mut rp = Vec3::ZERO;
                    Matrix_MulPoint(rot, &mut rp, p.x, p.y, p.z);
                    p = rp + (*box_0).p;
                    if iu != 0 && iv != 0 {
                        let mut off: libc::c_int = Mesh_GetVertexCount(mesh);
                        Mesh_AddQuad(
                            mesh,
                            off,
                            off - res,
                            off - res - 1 as libc::c_int,
                            off - 1 as libc::c_int,
                        );
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
    return mesh;
}
