use ::libc;
use super::internal::Memory::*;
extern "C" {
    fn __fpclassifyf(_: libc::c_float) -> libc::c_int;
    fn __fpclassifyd(_: libc::c_double) -> libc::c_int;
    fn fabs(_: libc::c_double) -> libc::c_double;
    fn sqrt(_: libc::c_double) -> libc::c_double;
    fn Fatal(_: cstr, _: ...);
    fn Polygon_ToPlane(_: *mut Polygon, _: *mut Plane);
    fn Polygon_ToPlaneFast(_: *mut Polygon, _: *mut Plane);
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
pub struct Vec3f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Plane {
    pub n: Vec3f,
    pub d: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Polygon {
    pub vertices_size: int32,
    pub vertices_capacity: int32,
    pub vertices_data: *mut Vec3f,
}
pub type Error = uint32;
pub type PointClassification = uint8;
pub type PolygonClassification = uint8;
#[inline]
unsafe extern "C" fn Abs(mut t: libc::c_double) -> libc::c_double {
    return fabs(t);
}
#[inline]
unsafe extern "C" fn Sqrtf(mut t: libc::c_float) -> libc::c_float {
    return sqrt(t as libc::c_double) as libc::c_float;
}
#[inline]
unsafe extern "C" fn Float_Validatef(mut x: libc::c_float) -> Error {
    let mut classification: libc::c_int = if ::core::mem::size_of::<libc::c_float>()
        as libc::c_ulong == ::core::mem::size_of::<libc::c_float>() as libc::c_ulong
    {
        __fpclassifyf(x)
    } else if ::core::mem::size_of::<libc::c_float>() as libc::c_ulong
        == ::core::mem::size_of::<libc::c_double>() as libc::c_ulong
    {
        __fpclassifyd(x as libc::c_double)
    } else {3
    };
    match classification {
        2 => return 0x4 as libc::c_int as Error,
        5 => {}
        1 => return 0x20 as libc::c_int as Error,
        3 | 4 => return 0 as libc::c_int as Error,
        _ => {
            Fatal(
                b"Float_Validate: Unhandled case: %i\0" as *const u8
                    as *const libc::c_char,
                classification,
            );
        }
    }
    return 0 as libc::c_int as Error;
}
#[inline]
unsafe extern "C" fn Float_Validate(mut x: libc::c_double) -> Error {
    let mut classification: libc::c_int = if ::core::mem::size_of::<libc::c_double>()
        as libc::c_ulong == ::core::mem::size_of::<libc::c_float>() as libc::c_ulong
    {
        __fpclassifyf(x as libc::c_float)
    } else if ::core::mem::size_of::<libc::c_double>() as libc::c_ulong
        == ::core::mem::size_of::<libc::c_double>() as libc::c_ulong
    {
        __fpclassifyd(x)
    } else {3
    };
    match classification {
        2 => return 0x4 as libc::c_int as Error,
        5 => return 0x8 as libc::c_int as Error,
        1 => return 0x20 as libc::c_int as Error,
        3 | 4 => return 0 as libc::c_int as Error,
        _ => {
            Fatal(
                b"Float_Validate: Unhandled case: %i\0" as *const u8
                    as *const libc::c_char,
                classification,
            );
        }
    }
    return 0 as libc::c_int as Error;
}
#[inline]
unsafe extern "C" fn Vec3f_Dot(mut a: Vec3f, mut b: Vec3f) -> libc::c_float {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}
#[inline]
unsafe extern "C" fn Vec3f_Length(mut v: Vec3f) -> libc::c_float {
    return Sqrtf(v.x * v.x + v.y * v.y + v.z * v.z);
}
#[inline]
unsafe extern "C" fn Vec3f_Validate(mut v: Vec3f) -> Error {
    let mut e: Error = 0 as libc::c_int as Error;
    e |= Float_Validatef(v.x);
    e |= Float_Validatef(v.y);
    e |= Float_Validatef(v.z);
    return e;
}
#[no_mangle]
pub unsafe extern "C" fn Plane_ClassifyPoint(
    mut plane: *mut Plane,
    mut p: *mut Vec3f,
) -> PointClassification {
    let mut magnitude: libc::c_float = Abs(
        (1.0f32 - Vec3f_Length((*plane).n)) as libc::c_double,
    ) as libc::c_float;
    let mut dist: libc::c_float = Vec3f_Dot((*plane).n, *p) - (*plane).d;
    if dist as libc::c_double > 1e-4f64 {
        return 1 as libc::c_int as PointClassification
    } else if (dist as libc::c_double) < -1e-4f64 {
        return 2 as libc::c_int as PointClassification
    } else {
        return 3 as libc::c_int as PointClassification
    };
}
#[no_mangle]
pub unsafe extern "C" fn Plane_ClassifyPolygon(
    mut plane: *mut Plane,
    mut polygon: *mut Polygon,
) -> PolygonClassification {
    let mut numInFront: int32 = 0 as libc::c_int;
    let mut numBehind: int32 = 0 as libc::c_int;
    let mut i: int32 = 0 as libc::c_int;
    while i < (*polygon).vertices_size {
        let mut vertex: Vec3f = *((*polygon).vertices_data).offset(i as isize);
        let mut classification: PointClassification = Plane_ClassifyPoint(
            plane,
            &mut vertex,
        );
        let mut current_block_2: u64;
        match classification as libc::c_int {
            1 => {
                current_block_2 = 18070553979786946493;
            }
            2 => {
                numBehind += 1;
                current_block_2 = 14523784380283086299;
            }
            3 => {
                current_block_2 = 14523784380283086299;
            }
            _ => {
                Fatal(
                    b"Plane_ClassifyPolygon: Unhandled case: %i\0" as *const u8
                        as *const libc::c_char,
                    classification as libc::c_int,
                );
                current_block_2 = 18070553979786946493;
            }
        }
        match current_block_2 {
            18070553979786946493 => {
                numInFront += 1;
            }
            _ => {}
        }
        if numInFront != 0 as libc::c_int && numBehind != 0 as libc::c_int {
            return 4 as libc::c_int as PolygonClassification;
        }
        i += 1;
    }
    if numInFront != 0 as libc::c_int {
        return 1 as libc::c_int as PolygonClassification;
    }
    if numBehind != 0 as libc::c_int {
        return 2 as libc::c_int as PolygonClassification;
    }
    return 3 as libc::c_int as PolygonClassification;
}
#[no_mangle]
pub unsafe extern "C" fn Plane_Validate(mut plane: *mut Plane) -> Error {
    let mut e: Error = 0 as libc::c_int as Error;
    e |= Float_Validate((*plane).d as libc::c_double);
    e |= Vec3f_Validate((*plane).n);
    return e;
}
#[no_mangle]
pub unsafe extern "C" fn Plane_FromPolygon(
    mut polygon: *mut Polygon,
    mut plane: *mut Plane,
) {
    Polygon_ToPlane(polygon, plane);
}
#[no_mangle]
pub unsafe extern "C" fn Plane_FromPolygonFast(
    mut polygon: *mut Polygon,
    mut plane: *mut Plane,
) {
    Polygon_ToPlaneFast(polygon, plane);
}
