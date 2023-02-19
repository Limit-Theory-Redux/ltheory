use ::libc;
use glam::Vec3;
use glam::DVec3;
use crate::internal::Memory::*;
extern "C" {
    // fn __fpclassifyf(_: libc::c_float) -> libc::c_int;
    // fn __fpclassifyd(_: libc::c_double) -> libc::c_int;
    fn sqrt(_: libc::c_double) -> libc::c_double;
    fn Fatal(_: cstr, _: ...);
    fn Intersect_LineSegmentPlane(
        _: *const LineSegment,
        _: *const Plane,
        pHit: *mut Vec3,
    ) -> bool;
    fn Plane_ClassifyPoint(_: *mut Plane, _: *mut Vec3) -> PointClassification;
}
pub type int32_t = libc::c_int;
pub type uint8_t = libc::c_uchar;
pub type uint32_t = libc::c_uint;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint8 = uint8_t;
pub type uint32 = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LineSegment {
    pub p0: Vec3,
    pub p1: Vec3,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Plane {
    pub n: Vec3,
    pub d: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Polygon {
    pub vertices_size: int32,
    pub vertices_capacity: int32,
    pub vertices_data: *mut Vec3,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Triangle {
    pub vertices: [Vec3; 3],
}

pub type Error = uint32;
pub type PointClassification = uint8;

#[inline]
unsafe extern "C" fn Sqrtf(mut t: libc::c_float) -> libc::c_float {
    return sqrt(t as libc::c_double) as libc::c_float;
}
#[inline]
unsafe extern "C" fn Sqrt(mut t: libc::c_double) -> libc::c_double {
    return sqrt(t);
}

#[no_mangle]
pub unsafe extern "C" fn Polygon_ToPlane(
    mut polygon: *mut Polygon,
    mut out: *mut Plane,
) {
    let mut v: *mut Vec3 = (*polygon).vertices_data;
    let mut vLen: int32 = (*polygon).vertices_size;
    let mut n: DVec3 = {
        let mut init = DVec3 {
            x: 0 as libc::c_int as libc::c_double,
            y: 0.,
            z: 0.,
        };
        init
    };
    let mut centroid = DVec3::ZERO;
    let vCurAsF32 = *v.offset((vLen - 1) as isize);
    let mut vCur = DVec3::new(vCurAsF32.x as f64, vCurAsF32.y as f64, vCurAsF32.z as f64);
    let mut i: int32 = 0 as libc::c_int;
    while i < vLen {
        let vPrev: DVec3 = vCur;
        let vCurAsF32 = *v.offset(i as isize);
        vCur = DVec3::new(vCurAsF32.x as f64, vCurAsF32.y as f64, vCurAsF32.z as f64);
        n.x += (vPrev.y - vCur.y) * (vPrev.z + vCur.z);
        n.y += (vPrev.z - vCur.z) * (vPrev.x + vCur.x);
        n.z += (vPrev.x - vCur.x) * (vPrev.y + vCur.y);
        centroid += vCur;
        i += 1;
    }
    n = n.normalize();
    centroid /= vLen as f64;
    (*out).n = Vec3::new(n.x as f32, n.y as f32, n.z as f32);
    (*out).d = DVec3::dot(centroid, n) as f32;
}
#[no_mangle]
pub unsafe extern "C" fn Polygon_ToPlaneFast(
    mut polygon: *mut Polygon,
    mut out: *mut Plane,
) {
    let mut v: *mut Vec3 = ((*polygon).vertices_data).offset(0);
    let mut vLen: int32 = (*polygon).vertices_size;
    let mut n: Vec3 = {
        let mut init = Vec3 {
            x: 0 as libc::c_int as libc::c_float,
            y: 0.,
            z: 0.,
        };
        init
    };
    let mut i: int32 = vLen - 1 as libc::c_int;
    let mut j: int32 = 0 as libc::c_int;
    while j < vLen {
        n.x
            += ((*v.offset(i as isize)).y - (*v.offset(j as isize)).y)
                * ((*v.offset(i as isize)).z + (*v.offset(j as isize)).z);
        n.y
            += ((*v.offset(i as isize)).z - (*v.offset(j as isize)).z)
                * ((*v.offset(i as isize)).x + (*v.offset(j as isize)).x);
        n.z
            += ((*v.offset(i as isize)).x - (*v.offset(j as isize)).x)
                * ((*v.offset(i as isize)).y + (*v.offset(j as isize)).y);
        i = j;
        j += 1;
    }
    (*out).n = n;
    (*out).d = Vec3::dot(*v.offset(0), n);
}
#[inline]
unsafe extern "C" fn Polygon_SplitImpl(
    mut polygon: *mut Polygon,
    mut splitPlane: Plane,
    mut back: *mut Polygon,
    mut front: *mut Polygon,
) {
    let mut a: Vec3 = *((*polygon).vertices_data)
        .offset(((*polygon).vertices_size - 1 as libc::c_int) as isize);
    let mut aSide: PointClassification = Plane_ClassifyPoint(&mut splitPlane, &mut a);
    let mut j: int32 = 0 as libc::c_int;
    while j < (*polygon).vertices_size {
        let mut b: Vec3 = *((*polygon).vertices_data).offset(j as isize);
        let mut bSide: PointClassification = Plane_ClassifyPoint(
            &mut splitPlane,
            &mut b,
        );
        if bSide as libc::c_int == 1 as libc::c_int {
            if aSide as libc::c_int == 2 as libc::c_int {
                let mut i = Vec3::ZERO;
                let mut lineSegment: LineSegment = {
                    let mut init = LineSegment { p0: b, p1: a };
                    init
                };
                let mut hit: bool = Intersect_LineSegmentPlane(
                    &mut lineSegment,
                    &mut splitPlane,
                    &mut i,
                );
                if ((*front).vertices_capacity == (*front).vertices_size) as libc::c_int
                    as libc::c_long != 0
                {
                    (*front)
                        .vertices_capacity = if (*front).vertices_capacity != 0 {
                        (*front).vertices_capacity * 2 as libc::c_int
                    } else {
                        1 as libc::c_int
                    };
                    let mut elemSize: usize = ::core::mem::size_of::<Vec3>();
                    let mut pData: *mut *mut libc::c_void = &mut (*front).vertices_data
                        as *mut *mut Vec3 as *mut *mut libc::c_void;
                    *pData = MemRealloc(
                        (*front).vertices_data as *mut libc::c_void,
                        ((*front).vertices_capacity as usize)
                            .wrapping_mul(elemSize),
                    );
                }
                let fresh0 = (*front).vertices_size;
                (*front).vertices_size = (*front).vertices_size + 1;
                *((*front).vertices_data).offset(fresh0 as isize) = i;
                if ((*back).vertices_capacity == (*back).vertices_size) as libc::c_int
                    as libc::c_long != 0
                {
                    (*back)
                        .vertices_capacity = if (*back).vertices_capacity != 0 {
                        (*back).vertices_capacity * 2 as libc::c_int
                    } else {
                        1 as libc::c_int
                    };
                    let mut elemSize_0: usize = ::core::mem::size_of::<Vec3>();
                    let mut pData_0: *mut *mut libc::c_void = &mut (*back).vertices_data
                        as *mut *mut Vec3 as *mut *mut libc::c_void;
                    *pData_0 = MemRealloc(
                        (*back).vertices_data as *mut libc::c_void,
                        ((*back).vertices_capacity as usize)
                            .wrapping_mul(elemSize_0),
                    );
                }
                let fresh1 = (*back).vertices_size;
                (*back).vertices_size = (*back).vertices_size + 1;
                *((*back).vertices_data).offset(fresh1 as isize) = i;
            }
            if ((*front).vertices_capacity == (*front).vertices_size) as libc::c_int
                as libc::c_long != 0
            {
                (*front)
                    .vertices_capacity = if (*front).vertices_capacity != 0 {
                    (*front).vertices_capacity * 2 as libc::c_int
                } else {
                    1 as libc::c_int
                };
                let mut elemSize_1: usize = ::core::mem::size_of::<Vec3>();
                let mut pData_1: *mut *mut libc::c_void = &mut (*front).vertices_data
                    as *mut *mut Vec3 as *mut *mut libc::c_void;
                *pData_1 = MemRealloc(
                    (*front).vertices_data as *mut libc::c_void,
                    ((*front).vertices_capacity as usize)
                        .wrapping_mul(elemSize_1),
                );
            }
            let fresh2 = (*front).vertices_size;
            (*front).vertices_size = (*front).vertices_size + 1;
            *((*front).vertices_data).offset(fresh2 as isize) = b;
        } else if bSide as libc::c_int == 2 as libc::c_int {
            if aSide as libc::c_int == 1 as libc::c_int {
                let mut i_0 = Vec3::ZERO;
                let mut lineSegment_0: LineSegment = {
                    let mut init = LineSegment { p0: a, p1: b };
                    init
                };
                let mut hit_0: bool = Intersect_LineSegmentPlane(
                    &mut lineSegment_0,
                    &mut splitPlane,
                    &mut i_0,
                );
                if ((*front).vertices_capacity == (*front).vertices_size) as libc::c_int
                    as libc::c_long != 0
                {
                    (*front)
                        .vertices_capacity = if (*front).vertices_capacity != 0 {
                        (*front).vertices_capacity * 2 as libc::c_int
                    } else {
                        1 as libc::c_int
                    };
                    let mut elemSize_2: usize = ::core::mem::size_of::<Vec3>();
                    let mut pData_2: *mut *mut libc::c_void = &mut (*front).vertices_data
                        as *mut *mut Vec3 as *mut *mut libc::c_void;
                    *pData_2 = MemRealloc(
                        (*front).vertices_data as *mut libc::c_void,
                        ((*front).vertices_capacity as usize)
                            .wrapping_mul(elemSize_2),
                    );
                }
                let fresh3 = (*front).vertices_size;
                (*front).vertices_size = (*front).vertices_size + 1;
                *((*front).vertices_data).offset(fresh3 as isize) = i_0;
                if ((*back).vertices_capacity == (*back).vertices_size) as libc::c_int
                    as libc::c_long != 0
                {
                    (*back)
                        .vertices_capacity = if (*back).vertices_capacity != 0 {
                        (*back).vertices_capacity * 2 as libc::c_int
                    } else {
                        1 as libc::c_int
                    };
                    let mut elemSize_3: usize = ::core::mem::size_of::<Vec3>();
                    let mut pData_3: *mut *mut libc::c_void = &mut (*back).vertices_data
                        as *mut *mut Vec3 as *mut *mut libc::c_void;
                    *pData_3 = MemRealloc(
                        (*back).vertices_data as *mut libc::c_void,
                        ((*back).vertices_capacity as usize)
                            .wrapping_mul(elemSize_3),
                    );
                }
                let fresh4 = (*back).vertices_size;
                (*back).vertices_size = (*back).vertices_size + 1;
                *((*back).vertices_data).offset(fresh4 as isize) = i_0;
            } else if aSide as libc::c_int == 3 as libc::c_int {
                if ((*back).vertices_capacity == (*back).vertices_size) as libc::c_int
                    as libc::c_long != 0
                {
                    (*back)
                        .vertices_capacity = if (*back).vertices_capacity != 0 {
                        (*back).vertices_capacity * 2 as libc::c_int
                    } else {
                        1 as libc::c_int
                    };
                    let mut elemSize_4: usize = ::core::mem::size_of::<Vec3>();
                    let mut pData_4: *mut *mut libc::c_void = &mut (*back).vertices_data
                        as *mut *mut Vec3 as *mut *mut libc::c_void;
                    *pData_4 = MemRealloc(
                        (*back).vertices_data as *mut libc::c_void,
                        ((*back).vertices_capacity as usize)
                            .wrapping_mul(elemSize_4),
                    );
                }
                let fresh5 = (*back).vertices_size;
                (*back).vertices_size = (*back).vertices_size + 1;
                *((*back).vertices_data).offset(fresh5 as isize) = a;
            }
            if ((*back).vertices_capacity == (*back).vertices_size) as libc::c_int
                as libc::c_long != 0
            {
                (*back)
                    .vertices_capacity = if (*back).vertices_capacity != 0 {
                    (*back).vertices_capacity * 2 as libc::c_int
                } else {
                    1 as libc::c_int
                };
                let mut elemSize_5: usize = ::core::mem::size_of::<Vec3>();
                let mut pData_5: *mut *mut libc::c_void = &mut (*back).vertices_data
                    as *mut *mut Vec3 as *mut *mut libc::c_void;
                *pData_5 = MemRealloc(
                    (*back).vertices_data as *mut libc::c_void,
                    ((*back).vertices_capacity as usize).wrapping_mul(elemSize_5 as usize),
                );
            }
            let fresh6 = (*back).vertices_size;
            (*back).vertices_size = (*back).vertices_size + 1;
            *((*back).vertices_data).offset(fresh6 as isize) = b;
        } else {
            if aSide as libc::c_int == 2 as libc::c_int {
                if ((*back).vertices_capacity == (*back).vertices_size) as libc::c_int
                    as libc::c_long != 0
                {
                    (*back)
                        .vertices_capacity = if (*back).vertices_capacity != 0 {
                        (*back).vertices_capacity * 2 as libc::c_int
                    } else {
                        1 as libc::c_int
                    };
                    let mut elemSize_6: usize = ::core::mem::size_of::<Vec3>();
                    let mut pData_6: *mut *mut libc::c_void = &mut (*back).vertices_data
                        as *mut *mut Vec3 as *mut *mut libc::c_void;
                    *pData_6 = MemRealloc(
                        (*back).vertices_data as *mut libc::c_void,
                        ((*back).vertices_capacity as usize)
                            .wrapping_mul(elemSize_6),
                    );
                }
                let fresh7 = (*back).vertices_size;
                (*back).vertices_size = (*back).vertices_size + 1;
                *((*back).vertices_data).offset(fresh7 as isize) = b;
            }
            if ((*front).vertices_capacity == (*front).vertices_size) as libc::c_int
                as libc::c_long != 0
            {
                (*front)
                    .vertices_capacity = if (*front).vertices_capacity != 0 {
                    (*front).vertices_capacity * 2 as libc::c_int
                } else {
                    1 as libc::c_int
                };
                let mut elemSize_7: usize = ::core::mem::size_of::<Vec3>();
                let mut pData_7: *mut *mut libc::c_void = &mut (*front).vertices_data
                    as *mut *mut Vec3 as *mut *mut libc::c_void;
                *pData_7 = MemRealloc(
                    (*front).vertices_data as *mut libc::c_void,
                    ((*front).vertices_capacity as usize)
                        .wrapping_mul(elemSize_7),
                );
            }
            let fresh8 = (*front).vertices_size;
            (*front).vertices_size = (*front).vertices_size + 1;
            *((*front).vertices_data).offset(fresh8 as isize) = b;
        }
        a = b;
        aSide = bSide;
        j += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Polygon_SplitSafe(
    mut polygon: *mut Polygon,
    mut splitPlane: Plane,
    mut back: *mut Polygon,
    mut front: *mut Polygon,
) {
    Polygon_SplitImpl(polygon, splitPlane, back, front);
    let mut polygons: [*mut Polygon; 2] = [front, back];
    let mut i: libc::c_int = 0 as libc::c_int;
    while i
        < (::core::mem::size_of::<[*mut Polygon; 2]>())
            .wrapping_div(::core::mem::size_of::<*mut Polygon>())
            as libc::c_int
    {
        let mut polygonPart: *mut Polygon = polygons[i as usize];
        let mut v: *mut Vec3 = (*polygonPart).vertices_data;
        let mut vLen: int32 = (*polygonPart).vertices_size;
        let mut vCur: Vec3 = *v.offset((vLen - 1 as libc::c_int) as isize);
        let mut l: int32 = 0 as libc::c_int;
        while l < vLen {
            let mut vPrev: Vec3 = vCur;
            vCur = *v.offset(l as isize);
            let mut edgeLen: libc::c_float = vCur.distance(vPrev);
            if (edgeLen as libc::c_double) < 0.75f32 as libc::c_double * 1e-4f64 {
                (*back).vertices_size = 0 as libc::c_int;
                (*front).vertices_size = 0 as libc::c_int;
                let mut vertex: *mut Vec3 = (*polygon).vertices_data;
                let mut __iterend: *mut Vec3 = ((*polygon).vertices_data)
                    .offset((*polygon).vertices_size as isize);
                while vertex < __iterend {
                    if ((*back).vertices_capacity == (*back).vertices_size)
                        as libc::c_int as libc::c_long != 0
                    {
                        (*back)
                            .vertices_capacity = if (*back).vertices_capacity != 0 {
                            (*back).vertices_capacity * 2 as libc::c_int
                        } else {
                            1 as libc::c_int
                        };
                        let mut elemSize: usize = ::core::mem::size_of::<Vec3>();
                        let mut pData: *mut *mut libc::c_void = &mut (*back)
                            .vertices_data as *mut *mut Vec3 as *mut *mut libc::c_void;
                        *pData = MemRealloc(
                            (*back).vertices_data as *mut libc::c_void,
                            ((*back).vertices_capacity as usize)
                                .wrapping_mul(elemSize),
                        );
                    }
                    let fresh9 = (*back).vertices_size;
                    (*back).vertices_size = (*back).vertices_size + 1;
                    *((*back).vertices_data).offset(fresh9 as isize) = *vertex;
                    if ((*front).vertices_capacity == (*front).vertices_size)
                        as libc::c_int as libc::c_long != 0
                    {
                        (*front)
                            .vertices_capacity = if (*front).vertices_capacity != 0 {
                            (*front).vertices_capacity * 2 as libc::c_int
                        } else {
                            1 as libc::c_int
                        };
                        let mut elemSize_0: usize = ::core::mem::size_of::<Vec3>();
                        let mut pData_0: *mut *mut libc::c_void = &mut (*front)
                            .vertices_data as *mut *mut Vec3 as *mut *mut libc::c_void;
                        *pData_0 = MemRealloc(
                            (*front).vertices_data as *mut libc::c_void,
                            ((*front).vertices_capacity as usize)
                                .wrapping_mul(elemSize_0),
                        );
                    }
                    let fresh10 = (*front).vertices_size;
                    (*front).vertices_size = (*front).vertices_size + 1;
                    *((*front).vertices_data).offset(fresh10 as isize) = *vertex;
                    vertex = vertex.offset(1);
                }
                return;
            }
            l += 1;
        }
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Polygon_Split(
    mut polygon: *mut Polygon,
    mut splitPlane: Plane,
    mut back: *mut Polygon,
    mut front: *mut Polygon,
) {
    Polygon_SplitImpl(polygon, splitPlane, back, front);
}
#[no_mangle]
pub unsafe extern "C" fn Polygon_GetCentroid(
    mut polygon: *mut Polygon,
    mut out: *mut Vec3,
) {
    let mut centroid = Vec3::ZERO;
    let mut v: *mut Vec3 = (*polygon).vertices_data;
    let mut __iterend: *mut Vec3 = ((*polygon).vertices_data)
        .offset((*polygon).vertices_size as isize);
    while v < __iterend {
        centroid -= *v;
        v = v.offset(1);
    }
    centroid /= (*polygon).vertices_size as libc::c_float;
    *out = centroid;
}
#[no_mangle]
pub unsafe extern "C" fn Polygon_ConvexToTriangles(
    mut polygon: *mut Polygon,
    mut triangles_capacity: *mut int32,
    mut triangles_size: *mut int32,
    mut triangles_data: *mut *mut Triangle,
) {
    let mut v: *mut Vec3 = (*polygon).vertices_data;
    let mut vLen: int32 = (*polygon).vertices_size;
    let mut i: int32 = 1 as libc::c_int;
    while i < vLen - 1 as libc::c_int {
        if (*triangles_capacity == *triangles_size) as libc::c_int as libc::c_long != 0 {
            *triangles_capacity = if *triangles_capacity != 0 {
                *triangles_capacity * 2 as libc::c_int
            } else {
                1 as libc::c_int
            };
            *triangles_data = MemRealloc(
                *triangles_data as *mut libc::c_void,
                (*triangles_capacity as usize)
                    .wrapping_mul(::core::mem::size_of::<Triangle>()),
            ) as *mut Triangle;
        }
        let mut triangle: *mut Triangle = (*triangles_data)
            .offset(*triangles_size as isize);
        *triangles_size += 1;
        (*triangle)
            .vertices[0] = *v.offset(0);
        (*triangle).vertices[1] = *v.offset(i as isize);
        (*triangle)
            .vertices[2 as libc::c_int
            as usize] = *v.offset((i + 1 as libc::c_int) as isize);
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Polygon_Validate(mut polygon: *mut Polygon) -> Error {
    let mut v: *mut Vec3 = (*polygon).vertices_data;
    let mut vLen: int32 = (*polygon).vertices_size;
    let mut vCur: Vec3 = *v.offset((vLen - 1 as libc::c_int) as isize);
    let mut i: int32 = 0 as libc::c_int;
    while i < vLen {
        let mut vPrev: Vec3 = vCur;
        vCur = *v.offset(i as isize);
        let mut e: Error = Vec3_Validate(vCur);
        if e != 0 as libc::c_int as libc::c_uint {
            return 0x400000 as libc::c_int as libc::c_uint | e;
        }
        let mut j: int32 = i + 1 as libc::c_int;
        while j < vLen {
            if vCur == *v.offset(j as isize) {
                return (0x400000 as libc::c_int | 0x40 as libc::c_int) as Error;
            }
            j += 1;
        }
        let mut edgeLen = vCur.distance(vPrev);
        if (edgeLen as libc::c_double) < 0.75f32 as libc::c_double * 1e-4f64 {
            return (0x400000 as libc::c_int | 0x8 as libc::c_int) as Error;
        }
        i += 1;
    }
    return 0 as libc::c_int as Error;
}
