use crate::internal::Memory::*;
use crate::Intersect::*;
use crate::LineSegment::*;
use crate::Triangle::*;
use crate::Plane::*;
use glam::DVec3;
use glam::Vec3;
use libc;

extern "C" {
    // fn __fpclassifyf(_: f32) -> i32;
    // fn __fpclassifyd(_: f64) -> i32;
    fn sqrt(_: f64) -> f64;
    fn Fatal(_: *const libc::c_char, _: ...);
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Polygon {
    pub vertices_size: i32,
    pub vertices_capacity: i32,
    pub vertices_data: *mut Vec3,
}


pub type PointClassification = u8;

#[inline]
unsafe extern "C" fn Sqrtf(mut t: f32) -> f32 {
    return sqrt(t as f64) as f32;
}

#[inline]
unsafe extern "C" fn Sqrt(mut t: f64) -> f64 {
    return sqrt(t);
}

#[no_mangle]
pub unsafe extern "C" fn Polygon_ToPlane(mut polygon: *mut Polygon, mut out: *mut Plane) {
    let mut v: *mut Vec3 = (*polygon).vertices_data;
    let mut vLen: i32 = (*polygon).vertices_size;
    let mut n: DVec3 = DVec3 {
        x: 0_i32 as f64,
        y: 0.,
        z: 0.,
    };
    let mut centroid = DVec3::ZERO;
    let vCurAsF32 = *v.offset((vLen - 1) as isize);
    let mut vCur = DVec3::new(vCurAsF32.x as f64, vCurAsF32.y as f64, vCurAsF32.z as f64);
    let mut i: i32 = 0_i32;
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
pub unsafe extern "C" fn Polygon_ToPlaneFast(mut polygon: *mut Polygon, mut out: *mut Plane) {
    let mut v: *mut Vec3 = ((*polygon).vertices_data).offset(0);
    let mut vLen: i32 = (*polygon).vertices_size;
    let mut n: Vec3 = Vec3 {
        x: 0.0f32,
        y: 0.,
        z: 0.,
    };
    let mut i: i32 = vLen - 1_i32;
    let mut j: i32 = 0_i32;
    while j < vLen {
        n.x += ((*v.offset(i as isize)).y - (*v.offset(j as isize)).y)
            * ((*v.offset(i as isize)).z + (*v.offset(j as isize)).z);
        n.y += ((*v.offset(i as isize)).z - (*v.offset(j as isize)).z)
            * ((*v.offset(i as isize)).x + (*v.offset(j as isize)).x);
        n.z += ((*v.offset(i as isize)).x - (*v.offset(j as isize)).x)
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
    let mut a: Vec3 =
        *((*polygon).vertices_data).offset(((*polygon).vertices_size - 1_i32) as isize);
    let mut aSide: PointClassification = Plane_ClassifyPoint(&mut splitPlane, &mut a);
    let mut j: i32 = 0_i32;
    while j < (*polygon).vertices_size {
        let mut b: Vec3 = *((*polygon).vertices_data).offset(j as isize);
        let mut bSide: PointClassification = Plane_ClassifyPoint(&mut splitPlane, &mut b);
        if bSide as i32 == 1_i32 {
            if aSide as i32 == 2_i32 {
                let mut i = Vec3::ZERO;
                let mut lineSegment: LineSegment = LineSegment { p0: b, p1: a };
                let mut _hit: bool =
                    Intersect_LineSegmentPlane(&mut lineSegment, &mut splitPlane, &mut i);
                if ((*front).vertices_capacity == (*front).vertices_size) as i32 as libc::c_long
                    != 0
                {
                    (*front).vertices_capacity = if (*front).vertices_capacity != 0 {
                        (*front).vertices_capacity * 2_i32
                    } else {
                        1_i32
                    };
                    let mut elemSize: usize = ::core::mem::size_of::<Vec3>();
                    let mut pData: *mut *mut libc::c_void =
                        &mut (*front).vertices_data as *mut *mut Vec3 as *mut *mut libc::c_void;
                    *pData = MemRealloc(
                        (*front).vertices_data as *mut libc::c_void,
                        ((*front).vertices_capacity as usize).wrapping_mul(elemSize),
                    );
                }
                let fresh0 = (*front).vertices_size;
                (*front).vertices_size = (*front).vertices_size + 1;
                *((*front).vertices_data).offset(fresh0 as isize) = i;
                if ((*back).vertices_capacity == (*back).vertices_size) as i32 as libc::c_long != 0
                {
                    (*back).vertices_capacity = if (*back).vertices_capacity != 0 {
                        (*back).vertices_capacity * 2_i32
                    } else {
                        1_i32
                    };
                    let mut elemSize_0: usize = ::core::mem::size_of::<Vec3>();
                    let mut pData_0: *mut *mut libc::c_void =
                        &mut (*back).vertices_data as *mut *mut Vec3 as *mut *mut libc::c_void;
                    *pData_0 = MemRealloc(
                        (*back).vertices_data as *mut libc::c_void,
                        ((*back).vertices_capacity as usize).wrapping_mul(elemSize_0),
                    );
                }
                let fresh1 = (*back).vertices_size;
                (*back).vertices_size = (*back).vertices_size + 1;
                *((*back).vertices_data).offset(fresh1 as isize) = i;
            }
            if ((*front).vertices_capacity == (*front).vertices_size) as i32 as libc::c_long != 0 {
                (*front).vertices_capacity = if (*front).vertices_capacity != 0 {
                    (*front).vertices_capacity * 2_i32
                } else {
                    1_i32
                };
                let mut elemSize_1: usize = ::core::mem::size_of::<Vec3>();
                let mut pData_1: *mut *mut libc::c_void =
                    &mut (*front).vertices_data as *mut *mut Vec3 as *mut *mut libc::c_void;
                *pData_1 = MemRealloc(
                    (*front).vertices_data as *mut libc::c_void,
                    ((*front).vertices_capacity as usize).wrapping_mul(elemSize_1),
                );
            }
            let fresh2 = (*front).vertices_size;
            (*front).vertices_size = (*front).vertices_size + 1;
            *((*front).vertices_data).offset(fresh2 as isize) = b;
        } else if bSide as i32 == 2_i32 {
            if aSide as i32 == 1_i32 {
                let mut i_0 = Vec3::ZERO;
                let mut lineSegment_0: LineSegment = LineSegment { p0: a, p1: b };
                let mut _hit_0: bool =
                    Intersect_LineSegmentPlane(&mut lineSegment_0, &mut splitPlane, &mut i_0);
                if ((*front).vertices_capacity == (*front).vertices_size) as i32 as libc::c_long
                    != 0
                {
                    (*front).vertices_capacity = if (*front).vertices_capacity != 0 {
                        (*front).vertices_capacity * 2_i32
                    } else {
                        1_i32
                    };
                    let mut elemSize_2: usize = ::core::mem::size_of::<Vec3>();
                    let mut pData_2: *mut *mut libc::c_void =
                        &mut (*front).vertices_data as *mut *mut Vec3 as *mut *mut libc::c_void;
                    *pData_2 = MemRealloc(
                        (*front).vertices_data as *mut libc::c_void,
                        ((*front).vertices_capacity as usize).wrapping_mul(elemSize_2),
                    );
                }
                let fresh3 = (*front).vertices_size;
                (*front).vertices_size = (*front).vertices_size + 1;
                *((*front).vertices_data).offset(fresh3 as isize) = i_0;
                if ((*back).vertices_capacity == (*back).vertices_size) as i32 as libc::c_long != 0
                {
                    (*back).vertices_capacity = if (*back).vertices_capacity != 0 {
                        (*back).vertices_capacity * 2_i32
                    } else {
                        1_i32
                    };
                    let mut elemSize_3: usize = ::core::mem::size_of::<Vec3>();
                    let mut pData_3: *mut *mut libc::c_void =
                        &mut (*back).vertices_data as *mut *mut Vec3 as *mut *mut libc::c_void;
                    *pData_3 = MemRealloc(
                        (*back).vertices_data as *mut libc::c_void,
                        ((*back).vertices_capacity as usize).wrapping_mul(elemSize_3),
                    );
                }
                let fresh4 = (*back).vertices_size;
                (*back).vertices_size = (*back).vertices_size + 1;
                *((*back).vertices_data).offset(fresh4 as isize) = i_0;
            } else if aSide as i32 == 3_i32 {
                if ((*back).vertices_capacity == (*back).vertices_size) as i32 as libc::c_long != 0
                {
                    (*back).vertices_capacity = if (*back).vertices_capacity != 0 {
                        (*back).vertices_capacity * 2_i32
                    } else {
                        1_i32
                    };
                    let mut elemSize_4: usize = ::core::mem::size_of::<Vec3>();
                    let mut pData_4: *mut *mut libc::c_void =
                        &mut (*back).vertices_data as *mut *mut Vec3 as *mut *mut libc::c_void;
                    *pData_4 = MemRealloc(
                        (*back).vertices_data as *mut libc::c_void,
                        ((*back).vertices_capacity as usize).wrapping_mul(elemSize_4),
                    );
                }
                let fresh5 = (*back).vertices_size;
                (*back).vertices_size = (*back).vertices_size + 1;
                *((*back).vertices_data).offset(fresh5 as isize) = a;
            }
            if ((*back).vertices_capacity == (*back).vertices_size) as i32 as libc::c_long != 0 {
                (*back).vertices_capacity = if (*back).vertices_capacity != 0 {
                    (*back).vertices_capacity * 2_i32
                } else {
                    1_i32
                };
                let mut elemSize_5: usize = ::core::mem::size_of::<Vec3>();
                let mut pData_5: *mut *mut libc::c_void =
                    &mut (*back).vertices_data as *mut *mut Vec3 as *mut *mut libc::c_void;
                *pData_5 = MemRealloc(
                    (*back).vertices_data as *mut libc::c_void,
                    ((*back).vertices_capacity as usize).wrapping_mul(elemSize_5),
                );
            }
            let fresh6 = (*back).vertices_size;
            (*back).vertices_size = (*back).vertices_size + 1;
            *((*back).vertices_data).offset(fresh6 as isize) = b;
        } else {
            if aSide as i32 == 2_i32 {
                if ((*back).vertices_capacity == (*back).vertices_size) as i32 as libc::c_long != 0
                {
                    (*back).vertices_capacity = if (*back).vertices_capacity != 0 {
                        (*back).vertices_capacity * 2_i32
                    } else {
                        1_i32
                    };
                    let mut elemSize_6: usize = ::core::mem::size_of::<Vec3>();
                    let mut pData_6: *mut *mut libc::c_void =
                        &mut (*back).vertices_data as *mut *mut Vec3 as *mut *mut libc::c_void;
                    *pData_6 = MemRealloc(
                        (*back).vertices_data as *mut libc::c_void,
                        ((*back).vertices_capacity as usize).wrapping_mul(elemSize_6),
                    );
                }
                let fresh7 = (*back).vertices_size;
                (*back).vertices_size = (*back).vertices_size + 1;
                *((*back).vertices_data).offset(fresh7 as isize) = b;
            }
            if ((*front).vertices_capacity == (*front).vertices_size) as i32 as libc::c_long != 0 {
                (*front).vertices_capacity = if (*front).vertices_capacity != 0 {
                    (*front).vertices_capacity * 2_i32
                } else {
                    1_i32
                };
                let mut elemSize_7: usize = ::core::mem::size_of::<Vec3>();
                let mut pData_7: *mut *mut libc::c_void =
                    &mut (*front).vertices_data as *mut *mut Vec3 as *mut *mut libc::c_void;
                *pData_7 = MemRealloc(
                    (*front).vertices_data as *mut libc::c_void,
                    ((*front).vertices_capacity as usize).wrapping_mul(elemSize_7),
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
    let mut i: i32 = 0_i32;
    while i
        < (::core::mem::size_of::<[*mut Polygon; 2]>())
            .wrapping_div(::core::mem::size_of::<*mut Polygon>()) as i32
    {
        let mut polygonPart: *mut Polygon = polygons[i as usize];
        let mut v: *mut Vec3 = (*polygonPart).vertices_data;
        let mut vLen: i32 = (*polygonPart).vertices_size;
        let mut vCur: Vec3 = *v.offset((vLen - 1_i32) as isize);
        let mut l: i32 = 0_i32;
        while l < vLen {
            let mut vPrev: Vec3 = vCur;
            vCur = *v.offset(l as isize);
            let mut edgeLen: f32 = vCur.distance(vPrev);
            if (edgeLen as f64) < 0.75f32 as f64 * 1e-4f64 {
                (*back).vertices_size = 0_i32;
                (*front).vertices_size = 0_i32;
                let mut vertex: *mut Vec3 = (*polygon).vertices_data;
                let mut __iterend: *mut Vec3 =
                    ((*polygon).vertices_data).offset((*polygon).vertices_size as isize);
                while vertex < __iterend {
                    if ((*back).vertices_capacity == (*back).vertices_size) as libc::c_long != 0 {
                        (*back).vertices_capacity = if (*back).vertices_capacity != 0 {
                            (*back).vertices_capacity * 2_i32
                        } else {
                            1_i32
                        };
                        let mut elemSize: usize = ::core::mem::size_of::<Vec3>();
                        let mut pData: *mut *mut libc::c_void =
                            &mut (*back).vertices_data as *mut *mut Vec3 as *mut *mut libc::c_void;
                        *pData = MemRealloc(
                            (*back).vertices_data as *mut libc::c_void,
                            ((*back).vertices_capacity as usize).wrapping_mul(elemSize),
                        );
                    }
                    let fresh9 = (*back).vertices_size;
                    (*back).vertices_size = (*back).vertices_size + 1;
                    *((*back).vertices_data).offset(fresh9 as isize) = *vertex;
                    if ((*front).vertices_capacity == (*front).vertices_size) as libc::c_long != 0 {
                        (*front).vertices_capacity = if (*front).vertices_capacity != 0 {
                            (*front).vertices_capacity * 2_i32
                        } else {
                            1_i32
                        };
                        let mut elemSize_0: usize = ::core::mem::size_of::<Vec3>();
                        let mut pData_0: *mut *mut libc::c_void =
                            &mut (*front).vertices_data as *mut *mut Vec3 as *mut *mut libc::c_void;
                        *pData_0 = MemRealloc(
                            (*front).vertices_data as *mut libc::c_void,
                            ((*front).vertices_capacity as usize).wrapping_mul(elemSize_0),
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
pub unsafe extern "C" fn Polygon_GetCentroid(mut polygon: *mut Polygon, mut out: *mut Vec3) {
    let mut centroid = Vec3::ZERO;
    let mut v: *mut Vec3 = (*polygon).vertices_data;
    let mut __iterend: *mut Vec3 =
        ((*polygon).vertices_data).offset((*polygon).vertices_size as isize);
    while v < __iterend {
        centroid -= *v;
        v = v.offset(1);
    }
    centroid /= (*polygon).vertices_size as f32;
    *out = centroid;
}

#[no_mangle]
pub unsafe extern "C" fn Polygon_ConvexToTriangles(
    mut polygon: *mut Polygon,
    mut triangles_capacity: *mut i32,
    mut triangles_size: *mut i32,
    mut triangles_data: *mut *mut Triangle,
) {
    let mut v: *mut Vec3 = (*polygon).vertices_data;
    let mut vLen: i32 = (*polygon).vertices_size;
    let mut i: i32 = 1_i32;
    while i < vLen - 1_i32 {
        if (*triangles_capacity == *triangles_size) as libc::c_long != 0 {
            *triangles_capacity = if *triangles_capacity != 0 {
                *triangles_capacity * 2_i32
            } else {
                1_i32
            };
            *triangles_data = MemRealloc(
                *triangles_data as *mut libc::c_void,
                (*triangles_capacity as usize).wrapping_mul(::core::mem::size_of::<Triangle>()),
            ) as *mut Triangle;
        }
        let mut triangle: *mut Triangle = (*triangles_data).offset(*triangles_size as isize);
        *triangles_size += 1;
        (*triangle).vertices[0] = *v.offset(0);
        (*triangle).vertices[1] = *v.offset(i as isize);
        (*triangle).vertices[2_i32 as usize] = *v.offset((i + 1_i32) as isize);
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Polygon_Validate(mut polygon: *mut Polygon) -> Error {
    let mut v: *mut Vec3 = (*polygon).vertices_data;
    let mut vLen: i32 = (*polygon).vertices_size;
    let mut vCur: Vec3 = *v.offset((vLen - 1_i32) as isize);
    let mut i: i32 = 0_i32;
    while i < vLen {
        let mut vPrev: Vec3 = vCur;
        vCur = *v.offset(i as isize);
        let mut e: Error = Vec3_Validate(vCur);
        if e != 0_i32 as u32 {
            return 0x400000_i32 as u32 | e;
        }
        let mut j: i32 = i + 1_i32;
        while j < vLen {
            if vCur == *v.offset(j as isize) {
                return (0x400000_i32 | 0x40_i32) as Error;
            }
            j += 1;
        }
        let mut edgeLen = vCur.distance(vPrev);
        if (edgeLen as f64) < 0.75f32 as f64 * 1e-4f64 {
            return (0x400000_i32 | 0x8_i32) as Error;
        }
        i += 1;
    }
    return 0_i32 as Error;
}
