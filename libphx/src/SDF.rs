use ::libc;
use glam::Vec3;
use glam::IVec3;
use crate::internal::Memory::*;
use crate::DataFormat::*;
use crate::PixelFormat::*;
extern "C" {
    pub type Mesh;
    pub type Tex3D;
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
        px: f32,
        py: f32,
        pz: f32,
        nx: f32,
        ny: f32,
        nz: f32,
        u: f32,
        v: f32,
    );
    fn Mesh_GetVertexCount(_: *mut Mesh) -> libc::c_int;
    fn Tex3D_GetData(_: *mut Tex3D, _: *mut libc::c_void, _: PixelFormat, _: DataFormat);
    fn Tex3D_GetSize(_: *mut Tex3D, out: *mut IVec3);
    fn sqrt(_: f64) -> f64;
}
pub type int32_t = libc::c_int;
pub type uint64_t = libc::c_ulonglong;
pub type int32 = int32_t;
pub type uint64 = uint64_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct SDF {
    pub size: IVec3,
    pub data: *mut Cell,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Cell {
    pub value: f32,
    pub normal: Vec3,
}


#[inline]
unsafe extern "C" fn Saturate(mut t: f64) -> f64 {
    return if t < 0.0f64 { 0.0f64 } else if t > 1.0f64 { 1.0f64 } else { t };
}
#[inline]
unsafe extern "C" fn Sqrtf(mut t: f32) -> f32 {
    return sqrt(t as f64) as f32;
}


#[no_mangle]
pub unsafe extern "C" fn SDF_Create(
    mut sx: libc::c_int,
    mut sy: libc::c_int,
    mut sz: libc::c_int,
) -> *mut SDF {
    let mut this: *mut SDF = MemAlloc(::core::mem::size_of::<SDF>())
        as *mut SDF;
    (*this).size = IVec3::new(sx, sy, sz);
    (*this)
        .data = MemAlloc(
        (::core::mem::size_of::<Cell>())
            .wrapping_mul((sx * sy * sz) as usize),
    ) as *mut Cell;
    MemZero(
        (*this).data as *mut libc::c_void,
        (::core::mem::size_of::<Cell>())
            .wrapping_mul(sx as usize).wrapping_mul(sy as usize).wrapping_mul(sz as usize),
    );
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn SDF_FromTex3D(mut tex: *mut Tex3D) -> *mut SDF {
    let mut this: *mut SDF = MemAlloc(::core::mem::size_of::<SDF>())
        as *mut SDF;
    Tex3D_GetSize(tex, &mut (*this).size);
    (*this)
        .data = MemAlloc(
        (::core::mem::size_of::<Cell>())
            .wrapping_mul(
                ((*this).size.x * (*this).size.y * (*this).size.z) as usize,
            ),
    ) as *mut Cell;
    Tex3D_GetData(
        tex,
        (*this).data as *mut libc::c_void,
        PixelFormat_RGBA,
        DataFormat_Float,
    );
    return this;
}
#[no_mangle]
pub unsafe extern "C" fn SDF_Free(mut this: *mut SDF) {
    MemFree((*this).data as *const libc::c_void);
    MemFree(this as *const libc::c_void);
}
#[no_mangle]
pub unsafe extern "C" fn SDF_ToMesh(mut this: *mut SDF) -> *mut Mesh {
    let mut mesh: *mut Mesh = Mesh_Create();
    let cells: IVec3 = {
        let mut init = IVec3 {
            x: (*this).size.x - 1 as libc::c_int,
            y: (*this).size.y - 1 as libc::c_int,
            z: (*this).size.z - 1 as libc::c_int,
        };
        init
    };
    let cellsF: Vec3 = {
        let mut init = Vec3 {
            x: cells.x as f32,
            y: cells.y as f32,
            z: cells.z as f32,
        };
        init
    };
    let stride: IVec3 = {
        let mut init = IVec3 {
            x: 1 as libc::c_int,
            y: (*this).size.x,
            z: (*this).size.x * (*this).size.y,
        };
        init
    };
    let cellStride: IVec3 = {
        let mut init = IVec3 {
            x: 1 as libc::c_int,
            y: cells.x,
            z: cells.x * cells.y,
        };
        init
    };
    let mut indices: *mut libc::c_int = MemAlloc(
        (::core::mem::size_of::<libc::c_int>())
            .wrapping_mul((cells.x * cells.y * cells.z) as usize),
    ) as *mut libc::c_int;
    let vp: [Vec3; 8] = [
        Vec3::new(0.0f32, 0.0f32, 0.0f32),
        Vec3::new(1.0f32, 0.0f32, 0.0f32),
        Vec3::new(0.0f32, 1.0f32, 0.0f32),
        Vec3::new(1.0f32, 1.0f32, 0.0f32),
        Vec3::new(0.0f32, 0.0f32, 1.0f32),
        Vec3::new(1.0f32, 0.0f32, 1.0f32),
        Vec3::new(0.0f32, 1.0f32, 1.0f32),
        Vec3::new(1.0f32, 1.0f32, 1.0f32),
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
        let mut z0: f32 = z as f32 / cells.z as f32;
        let mut y: libc::c_int = 0 as libc::c_int;
        while y < cells.y {
            let mut y0: f32 = y as f32 / cells.y as f32;
            let mut x: libc::c_int = 0 as libc::c_int;
            while x < cells.x {
                let mut x0: f32 = x as f32
                    / cells.x as f32;
                let mut cell: IVec3 = {
                    let mut init = IVec3 { x: x, y: y, z: z };
                    init
                };
                let cellIndex = IVec3::dot(cellStride, IVec3::new(x, y, z));
                let mut base: *const Cell = ((*this).data)
                    .offset(IVec3::dot(stride, IVec3::new(x, y, z)) as isize);
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
                    |= if (*v[0]).value
                        > 0.0f32
                    {
                        0x1 as libc::c_int
                    } else {
                        0 as libc::c_int
                    };
                mask
                    |= if (*v[1]).value
                        > 0.0f32
                    {
                        0x2 as libc::c_int
                    } else {
                        0 as libc::c_int
                    };
                mask
                    |= if (*v[2]).value
                        > 0.0f32
                    {
                        0x4 as libc::c_int
                    } else {
                        0 as libc::c_int
                    };
                mask
                    |= if (*v[3]).value
                        > 0.0f32
                    {
                        0x8 as libc::c_int
                    } else {
                        0 as libc::c_int
                    };
                mask
                    |= if (*v[4]).value
                        > 0.0f32
                    {
                        0x10 as libc::c_int
                    } else {
                        0 as libc::c_int
                    };
                mask
                    |= if (*v[5]).value
                        > 0.0f32
                    {
                        0x20 as libc::c_int
                    } else {
                        0 as libc::c_int
                    };
                mask
                    |= if (*v[6]).value
                        > 0.0f32
                    {
                        0x40 as libc::c_int
                    } else {
                        0 as libc::c_int
                    };
                mask
                    |= if (*v[7]).value
                        > 0.0f32
                    {
                        0x80 as libc::c_int
                    } else {
                        0 as libc::c_int
                    };
                if mask == 0 as libc::c_int || mask == 0xff as libc::c_int {
                    *indices.offset(cellIndex as isize) = -(1 as libc::c_int);
                } else {
                    let mut tw: f32 = 0.0f32;
                    let mut offset: Vec3 = {
                        let mut init = Vec3 {
                            x: 0.0f32,
                            y: 0.0f32,
                            z: 0.0f32,
                        };
                        init
                    };
                    let mut n: Vec3 = {
                        let mut init = Vec3 {
                            x: 0.0f32,
                            y: 0.0f32,
                            z: 0.0f32,
                        };
                        init
                    };
                    let mut i: libc::c_int = 0 as libc::c_int;
                    while i < 12 as libc::c_int {
                        let mut i0: libc::c_int = edgeTable[i
                            as usize][0];
                        let mut i1: libc::c_int = edgeTable[i
                            as usize][1];
                        let mut v0: *const Cell = v[i0 as usize];
                        let mut v1: *const Cell = v[i1 as usize];
                        if !(((*v0).value > 0.0f32)
                            as libc::c_int
                            == ((*v1).value > 0.0f32)
                                as libc::c_int)
                        {
                            let mut t: f32 = Saturate(
                                ((*v0).value / ((*v0).value - (*v1).value))
                                    as f64,
                            ) as f32;
                            offset += vp[i0 as usize].lerp(vp[i1 as usize], t);
                            n += (*v0).normal.lerp((*v1).normal, t);
                            tw += 1.0f32;
                        }
                        i += 1;
                    }
                    offset /= tw;
                    n = n.normalize();
                    let mut p: Vec3 = Vec3::new(x0, y0, z0) + (offset / cellsF);
                    p = p * 2.0f32 - 1.0f32;
                    *indices.offset(cellIndex as isize) = Mesh_GetVertexCount(mesh);
                    Mesh_AddVertex(
                        mesh,
                        p.x,
                        p.y,
                        p.z,
                        n.x,
                        n.y,
                        n.z,
                        1.0f32,
                        0.0f32,
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
                                if (*v[0]).value
                                    > 0.0f32
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
pub unsafe extern "C" fn SDF_Clear(mut this: *mut SDF, mut value: f32) {
    let mut size: uint64 = ((*this).size.x * (*this).size.y * (*this).size.z)
        as uint64;
    let mut pCell: *mut Cell = (*this).data;
    let mut i: uint64 = 0 as libc::c_int as uint64;
    while i < size {
        let fresh0 = pCell;
        pCell = pCell.offset(1);
        (*fresh0).value = value;
        i = i.wrapping_add(1);
    }
}
#[no_mangle]
pub unsafe extern "C" fn SDF_ComputeNormals(mut this: *mut SDF) {
    let stride: IVec3 = {
        let mut init = IVec3 {
            x: 1 as libc::c_int,
            y: (*this).size.x,
            z: (*this).size.x * (*this).size.y,
        };
        init
    };
    let mut z: libc::c_int = 1 as libc::c_int;
    while z < (*this).size.z - 1 as libc::c_int {
        let mut y: libc::c_int = 1 as libc::c_int;
        while y < (*this).size.y - 1 as libc::c_int {
            let mut x: libc::c_int = 1 as libc::c_int;
            while x < (*this).size.x - 1 as libc::c_int {
                let mut cell: *mut Cell = ((*this).data)
                    .offset((x * stride.x) as isize)
                    .offset((y * stride.y) as isize)
                    .offset((z * stride.z) as isize);
                let mut x0: *const Cell = cell.offset(-(stride.x as isize));
                let mut x1: *const Cell = cell.offset(stride.x as isize);
                let mut y0: *const Cell = cell.offset(-(stride.y as isize));
                let mut y1: *const Cell = cell.offset(stride.y as isize);
                let mut z0: *const Cell = cell.offset(-(stride.z as isize));
                let mut z1: *const Cell = cell.offset(stride.z as isize);
                (*cell).normal = Vec3::new(
                    (*x1).value - (*x0).value,
                    (*y1).value - (*y0).value,
                    (*z1).value - (*z0).value,
                ).normalize();
                x += 1;
            }
            y += 1;
        }
        z += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SDF_Set(
    mut this: *mut SDF,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut z: libc::c_int,
    mut value: f32,
) {
    (*((*this).data)
        .offset((x + (*this).size.x * (y + (*this).size.y * z)) as isize))
        .value = value;
}
#[no_mangle]
pub unsafe extern "C" fn SDF_SetNormal(
    mut this: *mut SDF,
    mut x: libc::c_int,
    mut y: libc::c_int,
    mut z: libc::c_int,
    mut normal: *const Vec3,
) {
    (*((*this).data)
        .offset((x + (*this).size.x * (y + (*this).size.y * z)) as isize))
        .normal = *normal;
}
