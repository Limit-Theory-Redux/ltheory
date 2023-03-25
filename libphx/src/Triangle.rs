use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::Vec3;
use crate::Math::Vec3_Validate;
use crate::Plane::*;
use libc;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Triangle {
    pub vertices: [Vec3; 3],
}

#[no_mangle]
pub unsafe extern "C" fn Triangle_ToPlane(tri: *const Triangle, plane: *mut Plane) {
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
pub unsafe extern "C" fn Triangle_ToPlaneFast(triangle: *const Triangle, plane: *mut Plane) {
    let mut v: *const Vec3 = ((*triangle).vertices).as_ptr();
    let mut e1: Vec3 = *v.offset(1) - *v.offset(0);
    let mut e2: Vec3 = *v.offset(2) - *v.offset(0);
    let mut n: Vec3 = Vec3::cross(e1, e2);
    (*plane).n = n;
    (*plane).d = Vec3::dot(*v.offset(0), n);
}

#[no_mangle]
pub unsafe extern "C" fn Triangle_GetArea(tri: *const Triangle) -> f32 {
    let mut e1 = (*tri).vertices[1] - (*tri).vertices[0];
    let mut e2 = (*tri).vertices[2] - (*tri).vertices[1];
    0.5f32 * Vec3::cross(e1, e2).length()
}

#[no_mangle]
pub unsafe extern "C" fn Triangle_Validate(tri: *const Triangle) -> Error {
    let mut v: *const Vec3 = ((*tri).vertices).as_ptr();
    let mut i: i32 = 0;
    while i < 3 {
        let mut e: Error = Vec3_Validate(*v.offset(i as isize));
        if e != 0 {
            return 0x400000 | e;
        }
        i += 1;
    }
    let mut eq01 = *v.offset(0) == *v.offset(1);
    let mut eq12 = *v.offset(1) == *v.offset(2);
    let mut eq20 = *v.offset(2) == *v.offset(0);
    if eq01 as i32 != 0 || eq12 as i32 != 0 || eq20 as i32 != 0 {
        return (0x400000 | 0x40) as Error;
    }
    let mut e01 = (*v.offset(0)).distance(*v.offset(1));
    let mut e12 = (*v.offset(1)).distance(*v.offset(2));
    let mut e20 = (*v.offset(2)).distance(*v.offset(0));
    let mut shortest: f32 = f64::min(f64::min(e01 as f64, e12 as f64), e20 as f64) as f32;
    if (shortest as f64) < 0.75f64 * 1e-4f64 {
        return (0x400000 | 0x8) as Error;
    }
    0 as Error
}
