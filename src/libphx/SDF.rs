use ::libc;
use super::internal::Memory::*;
extern "C" {
    pub type Mesh;
    pub type Tex3D;
    fn memset(
        _: *mut libc::c_void,
        _: libc::c_int,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
    fn free(_: *mut libc::c_void);
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
    fn Tex3D_GetData(_: *mut Tex3D, _: *mut libc::c_void, _: PixelFormat, _: DataFormat);
    fn Tex3D_GetSize(_: *mut Tex3D, out: *mut Vec3i);
    fn sqrt(_: libc::c_double) -> libc::c_double;
}
pub type int32_t = libc::c_int;
pub type uint64_t = libc::c_ulonglong;
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
pub type int32 = int32_t;
pub type uint64 = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDF {
    pub size: Vec3i,
    pub data: *mut Cell,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Cell {
    pub value: libc::c_float,
    pub normal: Vec3f,
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
pub struct Vec3i {
    pub x: libc::c_int,
    pub y: libc::c_int,
    pub z: libc::c_int,
}
pub type DataFormat = int32;
pub type PixelFormat = int32;
#[no_mangle]
pub static mut DataFormat_Float: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_I32: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_U32: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_I16: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_U16: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_I8: DataFormat = 0;
#[no_mangle]
pub static mut DataFormat_U8: DataFormat = 0;
#[inline]
unsafe extern "C" fn MemZero(mut dst: *mut libc::c_void, mut size: size_t) {
    memset(dst, 0 as libc::c_int, size);
}

#[no_mangle]
pub static mut PixelFormat_Red: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_RG: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_RGB: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_BGR: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_RGBA: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_BGRA: PixelFormat = 0;
#[no_mangle]
pub static mut PixelFormat_Depth_Component: PixelFormat = 0;
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
unsafe extern "C" fn Vec3i_Create(
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut z: libc::c_int,
) -> Vec3i {
    let mut self_0: Vec3i = {
        let mut init = Vec3i { x: x, y: y, z: z };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Vec3f_IAdd(mut a: *mut Vec3f, mut b: Vec3f) {
    (*a).x += b.x;
    (*a).y += b.y;
    (*a).z += b.z;
}
#[inline]
unsafe extern "C" fn Vec3f_Lerp(
    mut a: Vec3f,
    mut b: Vec3f,
    mut t: libc::c_float,
) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: a.x + (b.x - a.x) * t,
            y: a.y + (b.y - a.y) * t,
            z: a.z + (b.z - a.z) * t,
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Saturate(mut t: libc::c_double) -> libc::c_double {
    return if t < 0.0f64 { 0.0f64 } else if t > 1.0f64 { 1.0f64 } else { t };
}
#[inline]
unsafe extern "C" fn Vec3f_IDivs(mut a: *mut Vec3f, mut b: libc::c_float) {
    (*a).x /= b;
    (*a).y /= b;
    (*a).z /= b;
}
#[inline]
unsafe extern "C" fn Sqrtf(mut t: libc::c_float) -> libc::c_float {
    return sqrt(t as libc::c_double) as libc::c_float;
}
#[inline]
unsafe extern "C" fn Vec3f_Length(mut v: Vec3f) -> libc::c_float {
    return Sqrtf(v.x * v.x + v.y * v.y + v.z * v.z);
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
unsafe extern "C" fn Vec3f_Subs(mut a: Vec3f, mut b: libc::c_float) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: a.x - b,
            y: a.y - b,
            z: a.z - b,
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
unsafe extern "C" fn Vec3f_Div(mut a: Vec3f, mut b: Vec3f) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: a.x / b.x,
            y: a.y / b.y,
            z: a.z / b.z,
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Vec3i_Dots(
    mut a: Vec3i,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut z: libc::c_int,
) -> libc::c_int {
    return a.x * x + a.y * y + a.z * z;
}
#[no_mangle]
pub unsafe extern "C" fn SDF_Create(
    mut sx: libc::c_int,
    mut sy: libc::c_int,
    mut sz: libc::c_int,
) -> *mut SDF {
    let mut self_0: *mut SDF = MemAlloc(::core::mem::size_of::<SDF>())
        as *mut SDF;
    (*self_0).size = Vec3i_Create(sx, sy, sz);
    (*self_0)
        .data = MemAlloc(
        (::core::mem::size_of::<Cell>())
            .wrapping_mul((sx * sy * sz) as libc::c_ulong),
    ) as *mut Cell;
    MemZero(
        (*self_0).data as *mut libc::c_void,
        (::core::mem::size_of::<Cell>())
            .wrapping_mul(sx as libc::c_ulong)
            .wrapping_mul(sy as libc::c_ulong)
            .wrapping_mul(sz as libc::c_ulong),
    );
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn SDF_FromTex3D(mut tex: *mut Tex3D) -> *mut SDF {
    let mut self_0: *mut SDF = MemAlloc(::core::mem::size_of::<SDF>())
        as *mut SDF;
    Tex3D_GetSize(tex, &mut (*self_0).size);
    (*self_0)
        .data = MemAlloc(
        (::core::mem::size_of::<Cell>())
            .wrapping_mul(
                ((*self_0).size.x * (*self_0).size.y * (*self_0).size.z) as libc::c_ulong,
            ),
    ) as *mut Cell;
    Tex3D_GetData(
        tex,
        (*self_0).data as *mut libc::c_void,
        PixelFormat_RGBA,
        DataFormat_Float,
    );
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn SDF_Free(mut self_0: *mut SDF) {
    MemFree((*self_0).data as *const libc::c_void);
    MemFree(self_0 as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn SDF_ToMesh(mut self_0: *mut SDF) -> *mut Mesh {
    let mut mesh: *mut Mesh = Mesh_Create();
    let cells: Vec3i = {
        let mut init = Vec3i {
            x: (*self_0).size.x - 1 as libc::c_int,
            y: (*self_0).size.y - 1 as libc::c_int,
            z: (*self_0).size.z - 1 as libc::c_int,
        };
        init
    };
    let cellsF: Vec3f = {
        let mut init = Vec3f {
            x: cells.x as libc::c_float,
            y: cells.y as libc::c_float,
            z: cells.z as libc::c_float,
        };
        init
    };
    let stride: Vec3i = {
        let mut init = Vec3i {
            x: 1 as libc::c_int,
            y: (*self_0).size.x,
            z: (*self_0).size.x * (*self_0).size.y,
        };
        init
    };
    let cellStride: Vec3i = {
        let mut init = Vec3i {
            x: 1 as libc::c_int,
            y: cells.x,
            z: cells.x * cells.y,
        };
        init
    };
    let mut indices: *mut libc::c_int = MemAlloc(
        (::core::mem::size_of::<libc::c_int>())
            .wrapping_mul((cells.x * cells.y * cells.z) as libc::c_ulong),
    ) as *mut libc::c_int;
    let vp: [Vec3f; 8] = [
        {
            let mut init = Vec3f {
                x: 0 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: 1 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: 0 as libc::c_int as libc::c_float,
                y: 1 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: 1 as libc::c_int as libc::c_float,
                y: 1 as libc::c_int as libc::c_float,
                z: 0 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: 0 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 1 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: 1 as libc::c_int as libc::c_float,
                y: 0 as libc::c_int as libc::c_float,
                z: 1 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: 0 as libc::c_int as libc::c_float,
                y: 1 as libc::c_int as libc::c_float,
                z: 1 as libc::c_int as libc::c_float,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: 1 as libc::c_int as libc::c_float,
                y: 1 as libc::c_int as libc::c_float,
                z: 1 as libc::c_int as libc::c_float,
            };
            init
        },
    ];
    let edgeTable: [[libc::c_int; 2]; 12] = [
        [0 as libc::c_int, 1 as libc::c_int],
        [2 as libc::c_int, 3 as libc::c_int],
        [4 as libc::c_int, 5 as libc::c_int],
        [6 as libc::c_int, 7 as libc::c_int],
        [0 as libc::c_int, 2 as libc::c_int],
        [1 as libc::c_int, 3 as libc::c_int],
        [4 as libc::c_int, 6 as libc::c_int],
        [5 as libc::c_int, 7 as libc::c_int],
        [0 as libc::c_int, 4 as libc::c_int],
        [1 as libc::c_int, 5 as libc::c_int],
        [2 as libc::c_int, 6 as libc::c_int],
        [3 as libc::c_int, 7 as libc::c_int],
    ];
    let mut z: libc::c_int = 0 as libc::c_int;
    while z < cells.z {
        let mut z0: libc::c_float = z as libc::c_float / cells.z as libc::c_float;
        let mut y: libc::c_int = 0 as libc::c_int;
        while y < cells.y {
            let mut y0: libc::c_float = y as libc::c_float / cells.y as libc::c_float;
            let mut x: libc::c_int = 0 as libc::c_int;
            while x < cells.x {
                let mut x0: libc::c_float = x as libc::c_float
                    / cells.x as libc::c_float;
                let mut cell: Vec3i = {
                    let mut init = Vec3i { x: x, y: y, z: z };
                    init
                };
                let mut cellIndex: libc::c_int = Vec3i_Dots(cellStride, x, y, z);
                let mut base: *const Cell = ((*self_0).data)
                    .offset(Vec3i_Dots(stride, x, y, z) as isize);
                let mut v: [*const Cell; 8] = [
                    base,
                    base.offset(stride.x as isize),
                    base.offset(stride.y as isize),
                    base.offset(stride.x as isize).offset(stride.y as isize),
                    base.offset(stride.z as isize),
                    base.offset(stride.z as isize).offset(stride.x as isize),
                    base.offset(stride.z as isize).offset(stride.y as isize),
                    base
                        .offset(stride.z as isize)
                        .offset(stride.y as isize)
                        .offset(stride.x as isize),
                ];
                let mut mask: libc::c_int = 0 as libc::c_int;
                mask
                    |= if (*v[0 as libc::c_int as usize]).value
                        > 0 as libc::c_int as libc::c_float
                    {
                        0x1 as libc::c_int
                    } else {
                        0 as libc::c_int
                    };
                mask
                    |= if (*v[1 as libc::c_int as usize]).value
                        > 0 as libc::c_int as libc::c_float
                    {
                        0x2 as libc::c_int
                    } else {
                        0 as libc::c_int
                    };
                mask
                    |= if (*v[2 as libc::c_int as usize]).value
                        > 0 as libc::c_int as libc::c_float
                    {
                        0x4 as libc::c_int
                    } else {
                        0 as libc::c_int
                    };
                mask
                    |= if (*v[3 as libc::c_int as usize]).value
                        > 0 as libc::c_int as libc::c_float
                    {
                        0x8 as libc::c_int
                    } else {
                        0 as libc::c_int
                    };
                mask
                    |= if (*v[4 as libc::c_int as usize]).value
                        > 0 as libc::c_int as libc::c_float
                    {
                        0x10 as libc::c_int
                    } else {
                        0 as libc::c_int
                    };
                mask
                    |= if (*v[5 as libc::c_int as usize]).value
                        > 0 as libc::c_int as libc::c_float
                    {
                        0x20 as libc::c_int
                    } else {
                        0 as libc::c_int
                    };
                mask
                    |= if (*v[6 as libc::c_int as usize]).value
                        > 0 as libc::c_int as libc::c_float
                    {
                        0x40 as libc::c_int
                    } else {
                        0 as libc::c_int
                    };
                mask
                    |= if (*v[7 as libc::c_int as usize]).value
                        > 0 as libc::c_int as libc::c_float
                    {
                        0x80 as libc::c_int
                    } else {
                        0 as libc::c_int
                    };
                if mask == 0 as libc::c_int || mask == 0xff as libc::c_int {
                    *indices.offset(cellIndex as isize) = -(1 as libc::c_int);
                } else {
                    let mut tw: libc::c_float = 0.0f32;
                    let mut offset: Vec3f = {
                        let mut init = Vec3f {
                            x: 0 as libc::c_int as libc::c_float,
                            y: 0 as libc::c_int as libc::c_float,
                            z: 0 as libc::c_int as libc::c_float,
                        };
                        init
                    };
                    let mut n: Vec3f = {
                        let mut init = Vec3f {
                            x: 0 as libc::c_int as libc::c_float,
                            y: 0 as libc::c_int as libc::c_float,
                            z: 0 as libc::c_int as libc::c_float,
                        };
                        init
                    };
                    let mut i: libc::c_int = 0 as libc::c_int;
                    while i < 12 as libc::c_int {
                        let mut i0: libc::c_int = edgeTable[i
                            as usize][0 as libc::c_int as usize];
                        let mut i1: libc::c_int = edgeTable[i
                            as usize][1 as libc::c_int as usize];
                        let mut v0: *const Cell = v[i0 as usize];
                        let mut v1: *const Cell = v[i1 as usize];
                        if !(((*v0).value > 0 as libc::c_int as libc::c_float)
                            as libc::c_int
                            == ((*v1).value > 0 as libc::c_int as libc::c_float)
                                as libc::c_int)
                        {
                            let mut t: libc::c_float = Saturate(
                                ((*v0).value / ((*v0).value - (*v1).value))
                                    as libc::c_double,
                            ) as libc::c_float;
                            Vec3f_IAdd(
                                &mut offset,
                                Vec3f_Lerp(vp[i0 as usize], vp[i1 as usize], t),
                            );
                            Vec3f_IAdd(
                                &mut n,
                                Vec3f_Lerp((*v0).normal, (*v1).normal, t),
                            );
                            tw += 1.0f32;
                        }
                        i += 1;
                    }
                    Vec3f_IDivs(&mut offset, tw);
                    n = Vec3f_SNormalize(n);
                    let mut p: Vec3f = Vec3f_Add(
                        Vec3f_Create(x0, y0, z0),
                        Vec3f_Div(offset, cellsF),
                    );
                    p = Vec3f_Subs(Vec3f_Muls(p, 2.0f32), 1.0f32);
                    *indices.offset(cellIndex as isize) = Mesh_GetVertexCount(mesh);
                    Mesh_AddVertex(
                        mesh,
                        p.x,
                        p.y,
                        p.z,
                        n.x,
                        n.y,
                        n.z,
                        1 as libc::c_int as libc::c_float,
                        0 as libc::c_int as libc::c_float,
                    );
                    let mut i_0: libc::c_int = 0 as libc::c_int;
                    while i_0 < 3 as libc::c_int {
                        let mut j: libc::c_int = (i_0 + 1 as libc::c_int)
                            % 3 as libc::c_int;
                        let mut k: libc::c_int = (i_0 + 2 as libc::c_int)
                            % 3 as libc::c_int;
                        if !(*(&mut cell.x as *mut libc::c_int).offset(j as isize)
                            == 0 as libc::c_int
                            || *(&mut cell.x as *mut libc::c_int).offset(k as isize)
                                == 0 as libc::c_int)
                        {
                            let mut du: libc::c_int = *(&cellStride.x
                                as *const libc::c_int)
                                .offset(j as isize);
                            let mut dv: libc::c_int = *(&cellStride.x
                                as *const libc::c_int)
                                .offset(k as isize);
                            let mut i0_0: libc::c_int = *indices
                                .offset(cellIndex as isize);
                            let mut i1_0: libc::c_int = *indices
                                .offset((cellIndex - du) as isize);
                            let mut i2: libc::c_int = *indices
                                .offset((cellIndex - du - dv) as isize);
                            let mut i3: libc::c_int = *indices
                                .offset((cellIndex - dv) as isize);
                            if !(i1_0 < 0 as libc::c_int || i2 < 0 as libc::c_int
                                || i3 < 0 as libc::c_int)
                            {
                                if (*v[0 as libc::c_int as usize]).value
                                    > 0 as libc::c_int as libc::c_float
                                {
                                    Mesh_AddQuad(mesh, i0_0, i3, i2, i1_0);
                                } else {
                                    Mesh_AddQuad(mesh, i0_0, i1_0, i2, i3);
                                }
                            }
                        }
                        i_0 += 1;
                    }
                }
                x += 1;
            }
            y += 1;
        }
        z += 1;
    }
    MemFree(indices as *const libc::c_void);
    return mesh;
}
#[no_mangle]
pub unsafe extern "C" fn SDF_Clear(mut self_0: *mut SDF, mut value: libc::c_float) {
    let mut size: uint64 = ((*self_0).size.x * (*self_0).size.y * (*self_0).size.z)
        as uint64;
    let mut pCell: *mut Cell = (*self_0).data;
    let mut i: uint64 = 0 as libc::c_int as uint64;
    while i < size {
        let fresh0 = pCell;
        pCell = pCell.offset(1);
        (*fresh0).value = value;
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn SDF_ComputeNormals(mut self_0: *mut SDF) {
    let stride: Vec3i = {
        let mut init = Vec3i {
            x: 1 as libc::c_int,
            y: (*self_0).size.x,
            z: (*self_0).size.x * (*self_0).size.y,
        };
        init
    };
    let mut z: libc::c_int = 1 as libc::c_int;
    while z < (*self_0).size.z - 1 as libc::c_int {
        let mut y: libc::c_int = 1 as libc::c_int;
        while y < (*self_0).size.y - 1 as libc::c_int {
            let mut x: libc::c_int = 1 as libc::c_int;
            while x < (*self_0).size.x - 1 as libc::c_int {
                let mut cell: *mut Cell = ((*self_0).data)
                    .offset((x * stride.x) as isize)
                    .offset((y * stride.y) as isize)
                    .offset((z * stride.z) as isize);
                let mut x0: *const Cell = cell.offset(-(stride.x as isize));
                let mut x1: *const Cell = cell.offset(stride.x as isize);
                let mut y0: *const Cell = cell.offset(-(stride.y as isize));
                let mut y1: *const Cell = cell.offset(stride.y as isize);
                let mut z0: *const Cell = cell.offset(-(stride.z as isize));
                let mut z1: *const Cell = cell.offset(stride.z as isize);
                (*cell)
                    .normal = Vec3f_SNormalize(
                    Vec3f_Create(
                        (*x1).value - (*x0).value,
                        (*y1).value - (*y0).value,
                        (*z1).value - (*z0).value,
                    ),
                );
                x += 1;
            }
            y += 1;
        }
        z += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SDF_Set(
    mut self_0: *mut SDF,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut z: libc::c_int,
    mut value: libc::c_float,
) {
    (*((*self_0).data)
        .offset((x + (*self_0).size.x * (y + (*self_0).size.y * z)) as isize))
        .value = value;
}
#[no_mangle]
pub unsafe extern "C" fn SDF_SetNormal(
    mut self_0: *mut SDF,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut z: libc::c_int,
    mut normal: *const Vec3f,
) {
    (*((*self_0).data)
        .offset((x + (*self_0).size.x * (y + (*self_0).size.y * z)) as isize))
        .normal = *normal;
}
