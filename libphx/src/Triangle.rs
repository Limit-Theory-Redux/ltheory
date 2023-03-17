use crate::internal::Memory::*;
use crate::Plane::*;
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
pub struct Triangle {
    pub vertices: [Vec3; 3],
}

#[inline]
unsafe extern "C" fn Sqrtf(mut t: f32) -> f32 {
    return sqrt(t as f64) as f32;
}
#[inline]
unsafe extern "C" fn Min(mut a: f64, mut b: f64) -> f64 {
    return if a < b { a } else { b };
}

#[no_mangle]
pub unsafe extern "C" fn Triangle_ToPlane(mut tri: *const Triangle, mut plane: *mut Plane) {
    let mut v: *const Vec3 = ((*tri).vertices).as_ptr();
    let mut e1: Vec3 = *v.offset(1) - *v.offset(0);
    let mut e2: Vec3 = *v.offset(2) - *v.offset(0);
    let mut n: Vec3 = Vec3::cross(e1, e2).normalize();
    let mut centroid: Vec3 = *v.offset(0);
    centroid += *v.offset(1);
    centroid += *v.offset(2);
    centroid /= 3.0f32;
    (*plane).n = n;
    (*plane).d = Vec3::dot(centroid, n);
}
#[no_mangle]
pub unsafe extern "C" fn Triangle_ToPlaneFast(
    mut triangle: *const Triangle,
    mut plane: *mut Plane,
) {
    let mut v: *const Vec3 = ((*triangle).vertices).as_ptr();
    let mut e1: Vec3 = *v.offset(1) - *v.offset(0);
    let mut e2: Vec3 = *v.offset(2) - *v.offset(0);
    let mut n: Vec3 = Vec3::cross(e1, e2);
    (*plane).n = n;
    (*plane).d = Vec3::dot(*v.offset(0), n);
}
#[no_mangle]
pub unsafe extern "C" fn Triangle_GetArea(mut tri: *const Triangle) -> f32 {
    let mut e1 = (*tri).vertices[1] - (*tri).vertices[0];
    let mut e2 = (*tri).vertices[2] - (*tri).vertices[1];
    return 0.5f32 * Vec3::cross(e1, e2).length();
}
#[no_mangle]
pub unsafe extern "C" fn Triangle_Validate(mut tri: *const Triangle) -> Error {
    let mut v: *const Vec3 = ((*tri).vertices).as_ptr();
    let mut i: i32 = 0 as i32;
    while i < 3 as i32 {
        let mut e: Error = Vec3_Validate(*v.offset(i as isize));
        if e != 0 as i32 as u32 {
            return 0x400000 as i32 as u32 | e;
        }
        i += 1;
    }
    let mut eq01 = *v.offset(0) == *v.offset(1);
    let mut eq12 = *v.offset(1) == *v.offset(2);
    let mut eq20 = *v.offset(2) == *v.offset(0);
    if eq01 as i32 != 0 || eq12 as i32 != 0 || eq20 as i32 != 0 {
        return (0x400000 as i32 | 0x40 as i32) as Error;
    }
    let mut e01 = (*v.offset(0)).distance(*v.offset(1));
    let mut e12 = (*v.offset(1)).distance(*v.offset(2));
    let mut e20 = (*v.offset(2)).distance(*v.offset(0));
    let mut shortest: f32 = Min(Min(e01 as f64, e12 as f64), e20 as f64) as f32;
    if (shortest as f64) < 0.75f32 as f64 * 1e-4f64 {
        return (0x400000 as i32 | 0x8 as i32) as Error;
    }
    return 0 as i32 as Error;
}
