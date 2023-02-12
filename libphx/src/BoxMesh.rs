use ::libc;
extern "C" {
    pub type Mesh;
    pub type Matrix;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
    fn realloc(_: *mut libc::c_void, _: libc::c_ulong) -> *mut libc::c_void;
    fn Matrix_Free(_: *mut Matrix);
    fn Matrix_YawPitchRoll(
        yaw: libc::c_float,
        pitch: libc::c_float,
        roll: libc::c_float,
    ) -> *mut Matrix;
    fn Matrix_MulPoint(
        _: *const Matrix,
        out: *mut Vec3f,
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
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
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
    pub p: Vec3f,
    pub s: Vec3f,
    pub r: Vec3f,
    pub b: Vec3f,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec3f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
}
#[inline]
unsafe extern "C" fn MemRealloc(
    mut ptr: *mut libc::c_void,
    mut newSize: size_t,
) -> *mut libc::c_void {
    return realloc(ptr, newSize);
}
#[inline]
unsafe extern "C" fn MemFree(mut ptr: *const libc::c_void) {
    free(ptr as *mut libc::c_void);
}
#[inline]
unsafe extern "C" fn MemAlloc(mut size: size_t) -> *mut libc::c_void {
    return malloc(size);
}
#[inline]
unsafe extern "C" fn Vec3f_Mul(mut a: Vec3f, mut b: Vec3f) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: a.x * b.x,
            y: a.y * b.y,
            z: a.z * b.z,
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
unsafe extern "C" fn Vec3f_Create(
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut z: libc::c_float,
) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f { x: x, y: y, z: z };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Vec3f_Length(mut v: Vec3f) -> libc::c_float {
    return Sqrtf(v.x * v.x + v.y * v.y + v.z * v.z);
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
unsafe extern "C" fn Vec3f_Clamp(
    mut v: Vec3f,
    mut lower: Vec3f,
    mut upper: Vec3f,
) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: Clampf(v.x, lower.x, upper.x),
            y: Clampf(v.y, lower.y, upper.y),
            z: Clampf(v.z, lower.z, upper.z),
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Vec3f_SNormalize(mut v: Vec3f) -> Vec3f {
    let mut l: libc::c_float = Vec3f_Length(v);
    if l > 0 as libc::c_int as libc::c_float {
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
    return v;
}
#[inline]
unsafe extern "C" fn Vec3f_Sub(mut a: Vec3f, mut b: Vec3f) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: a.x - b.x,
            y: a.y - b.y,
            z: a.z - b.z,
        };
        init
    };
    return self_0;
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
static mut kFaceOrigin: [Vec3f; 6] = [
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
static mut kFaceU: [Vec3f; 6] = [
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
static mut kFaceV: [Vec3f; 6] = [
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
#[no_mangle]
pub unsafe extern "C" fn BoxMesh_Create() -> *mut BoxMesh {
    let mut self_0: *mut BoxMesh = MemAlloc(
        ::core::mem::size_of::<BoxMesh>() as libc::c_ulong,
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
    mut p: *const Vec3f,
    mut s: *const Vec3f,
    mut r: *const Vec3f,
    mut b: *const Vec3f,
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
        let mut elemSize: size_t = ::core::mem::size_of::<Box_0>() as libc::c_ulong;
        let mut pData: *mut *mut libc::c_void = &mut (*self_0).elem_data
            as *mut *mut Box_0 as *mut *mut libc::c_void;
        *pData = MemRealloc(
            (*self_0).elem_data as *mut libc::c_void,
            ((*self_0).elem_capacity as libc::c_ulong).wrapping_mul(elemSize),
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
        let mut lower: Vec3f = Vec3f_Create(
            (*box_0).b.x - 1.0f32,
            (*box_0).b.y - 1.0f32,
            (*box_0).b.z - 1.0f32,
        );
        let mut upper: Vec3f = Vec3f_Create(
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
            let mut o: Vec3f = kFaceOrigin[face as usize];
            let mut du: Vec3f = kFaceU[face as usize];
            let mut dv: Vec3f = kFaceV[face as usize];
            let mut n: Vec3f = Vec3f_Normalize(Vec3f_Cross(du, dv));
            let mut iu: libc::c_int = 0 as libc::c_int;
            while iu < res {
                let mut u: libc::c_float = iu as libc::c_float
                    / (res - 1 as libc::c_int) as libc::c_float;
                let mut iv: libc::c_int = 0 as libc::c_int;
                while iv < res {
                    let mut v: libc::c_float = iv as libc::c_float
                        / (res - 1 as libc::c_int) as libc::c_float;
                    let mut p: Vec3f = Vec3f_Add(
                        o,
                        Vec3f_Add(Vec3f_Muls(du, u), Vec3f_Muls(dv, v)),
                    );
                    let mut clamped: Vec3f = Vec3f_Clamp(p, lower, upper);
                    let mut proj: Vec3f = Vec3f_Sub(p, clamped);
                    p = Vec3f_Add(
                        clamped,
                        Vec3f_Mul(Vec3f_SNormalize(proj), (*box_0).b),
                    );
                    p = Vec3f_Mul(p, (*box_0).s);
                    let mut rp: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
                    Matrix_MulPoint(rot, &mut rp, p.x, p.y, p.z);
                    p = Vec3f_Add(rp, (*box_0).p);
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
