use super::*;
use crate::error::Error;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Triangle {
    pub vertices: [Vec3; 3],
}

#[no_mangle]
pub unsafe extern "C" fn Triangle_ToPlane(tri: &Triangle, plane: &mut Plane) {
    let v: *const Vec3 = (tri.vertices).as_ptr();
    let e1: Vec3 = *v.offset(1) - *v.offset(0);
    let e2: Vec3 = *v.offset(2) - *v.offset(0);
    let n: Vec3 = Vec3::cross(e1, e2).normalize();
    let mut centroid: Vec3 = *v.offset(0);
    centroid += *v.offset(1);
    centroid += *v.offset(2);
    centroid /= 3.0f32;
    plane.n = n;
    plane.d = Vec3::dot(centroid, n);
}

#[no_mangle]
pub unsafe extern "C" fn Triangle_ToPlaneFast(triangle: &Triangle, plane: &mut Plane) {
    let v: *const Vec3 = (triangle.vertices).as_ptr();
    let e1: Vec3 = *v.offset(1) - *v.offset(0);
    let e2: Vec3 = *v.offset(2) - *v.offset(0);
    let n: Vec3 = Vec3::cross(e1, e2);
    plane.n = n;
    plane.d = Vec3::dot(*v.offset(0), n);
}

#[no_mangle]
pub extern "C" fn Triangle_GetArea(tri: &Triangle) -> f32 {
    let e1 = tri.vertices[1] - tri.vertices[0];
    let e2 = tri.vertices[2] - tri.vertices[1];
    0.5f32 * Vec3::cross(e1, e2).length()
}

#[no_mangle]
pub unsafe extern "C" fn Triangle_Validate(tri: &Triangle) -> Error {
    let v: *const Vec3 = (tri.vertices).as_ptr();
    let mut i: i32 = 0;
    while i < 3 {
        let e = Vec3_Validate(*v.offset(i as isize));
        if e != 0 {
            return 0x400000 | e;
        }
        i += 1;
    }
    let eq01 = *v.offset(0) == *v.offset(1);
    let eq12 = *v.offset(1) == *v.offset(2);
    let eq20 = *v.offset(2) == *v.offset(0);
    if eq01 as i32 != 0 || eq12 as i32 != 0 || eq20 as i32 != 0 {
        return (0x400000 | 0x40) as Error;
    }
    let e01 = (*v.offset(0)).distance(*v.offset(1));
    let e12 = (*v.offset(1)).distance(*v.offset(2));
    let e20 = (*v.offset(2)).distance(*v.offset(0));
    let shortest: f32 = f32::min(f32::min(e01, e12), e20);
    if (shortest as f64) < 0.75f64 * 1e-4f64 {
        return (0x400000 | 0x8) as Error;
    }
    0 as Error
}
