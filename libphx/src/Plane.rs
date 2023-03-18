use crate::internal::Memory::*;
use crate::Polygon::*;
use glam::Vec3;
use libc;

extern "C" {
    // fn __fpclassifyf(_: f32) -> i32;
    // fn __fpclassifyd(_: f64) -> i32;
    fn Fatal(_: *const libc::c_char, _: ...);
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Plane {
    pub n: Vec3,
    pub d: f32,
}

pub type PointClassification = u8;
pub type PolygonClassification = u8;

#[inline]
unsafe extern "C" fn Float_Validate(mut x: f64) -> Error {
    let mut classification: i32 = if ::core::mem::size_of::<f64>() as libc::c_ulong
        == ::core::mem::size_of::<f32>() as libc::c_ulong
    {
        f32::classify(x as f32) as i32
    } else if ::core::mem::size_of::<f64>() as libc::c_ulong
        == ::core::mem::size_of::<f64>() as libc::c_ulong
    {
        f64::classify(x) as i32
    } else {
        3
    };
    match classification {
        2 => return 0x4_i32 as Error,
        5 => return 0x8_i32 as Error,
        1 => return 0x20_i32 as Error,
        3 | 4 => return 0_i32 as Error,
        _ => {
            Fatal(
                b"Float_Validate: Unhandled case: %i\0" as *const u8 as *const libc::c_char,
                classification,
            );
        }
    }
    0_i32 as Error
}

#[no_mangle]
pub unsafe extern "C" fn Plane_ClassifyPoint(
    mut plane: *mut Plane,
    mut p: *mut Vec3,
) -> PointClassification {
    let mut _magnitude: f32 = f64::abs((1.0f32 - (*plane).n.length()) as f64) as f32;
    let mut dist: f32 = Vec3::dot((*plane).n, *p) - (*plane).d;
    if dist as f64 > 1e-4f64 {
        1_i32 as PointClassification
    } else if (dist as f64) < -1e-4f64 {
        return 2_i32 as PointClassification;
    } else {
        return 3_i32 as PointClassification;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Plane_ClassifyPolygon(
    mut plane: *mut Plane,
    mut polygon: *mut Polygon,
) -> PolygonClassification {
    let mut numInFront: i32 = 0_i32;
    let mut numBehind: i32 = 0_i32;
    let mut i: i32 = 0_i32;
    while i < (*polygon).vertices_size {
        let mut vertex: Vec3 = *((*polygon).vertices_data).offset(i as isize);
        let mut classification: PointClassification = Plane_ClassifyPoint(plane, &mut vertex);
        let mut current_block_2: u64;
        match classification as i32 {
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
                    classification as i32,
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
        if numInFront != 0_i32 && numBehind != 0_i32 {
            return 4_i32 as PolygonClassification;
        }
        i += 1;
    }
    if numInFront != 0_i32 {
        return 1_i32 as PolygonClassification;
    }
    if numBehind != 0_i32 {
        return 2_i32 as PolygonClassification;
    }
    3_i32 as PolygonClassification
}

#[no_mangle]
pub unsafe extern "C" fn Plane_Validate(mut plane: *mut Plane) -> Error {
    let mut e: Error = 0_i32 as Error;
    e |= Float_Validate((*plane).d as f64);
    e |= Vec3_Validate((*plane).n);
    e
}

#[no_mangle]
pub unsafe extern "C" fn Plane_FromPolygon(mut polygon: *mut Polygon, mut plane: *mut Plane) {
    Polygon_ToPlane(polygon, plane);
}

#[no_mangle]
pub unsafe extern "C" fn Plane_FromPolygonFast(mut polygon: *mut Polygon, mut plane: *mut Plane) {
    Polygon_ToPlaneFast(polygon, plane);
}
