use ::libc;
use crate::internal::Memory::*;
extern "C" {
    pub type Plane;
    pub type Triangle;
    fn Intersect_RayTriangle_Moller2(
        _: *const Ray,
        _: *const Triangle,
        tHit: *mut libc::c_float,
    ) -> bool;
    fn Intersect_RayTriangle_Moller1(
        _: *const Ray,
        _: *const Triangle,
        tHit: *mut libc::c_float,
    ) -> bool;
    fn LineSegment_ToRay(_: *const LineSegment, _: *mut Ray);
    fn Intersect_RayPlane(_: *const Ray, _: *const Plane, pHit: *mut Vec3f) -> bool;
    fn Intersect_RayTriangle_Barycentric(
        _: *const Ray,
        _: *const Triangle,
        tEpsilon: libc::c_float,
        tHit: *mut libc::c_float,
    ) -> bool;
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LineSegment {
    pub p0: Vec3f,
    pub p1: Vec3f,
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
pub struct Ray {
    pub p: Vec3f,
    pub dir: Vec3f,
    pub tMin: libc::c_float,
    pub tMax: libc::c_float,
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
#[no_mangle]
pub unsafe extern "C" fn Ray_GetPoint(
    mut self_0: *const Ray,
    mut t: libc::c_float,
    mut out: *mut Vec3f,
) {
    *out = Vec3f_Add((*self_0).p, Vec3f_Muls((*self_0).dir, t));
}
#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectPlane(
    mut self_0: *const Ray,
    mut plane: *const Plane,
    mut pHit: *mut Vec3f,
) -> bool {
    return Intersect_RayPlane(self_0, plane, pHit);
}
#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectTriangle_Barycentric(
    mut self_0: *const Ray,
    mut tri: *const Triangle,
    mut tEpsilon: libc::c_float,
    mut tHit: *mut libc::c_float,
) -> bool {
    return Intersect_RayTriangle_Barycentric(self_0, tri, tEpsilon, tHit);
}
#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectTriangle_Moller1(
    mut self_0: *const Ray,
    mut tri: *const Triangle,
    mut tHit: *mut libc::c_float,
) -> bool {
    return Intersect_RayTriangle_Moller1(self_0, tri, tHit);
}
#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectTriangle_Moller2(
    mut self_0: *const Ray,
    mut tri: *const Triangle,
    mut tHit: *mut libc::c_float,
) -> bool {
    return Intersect_RayTriangle_Moller2(self_0, tri, tHit);
}
#[no_mangle]
pub unsafe extern "C" fn Ray_ToLineSegment(
    mut self_0: *const Ray,
    mut lineSegment: *mut LineSegment,
) {
    Ray_GetPoint(self_0, (*self_0).tMin, &mut (*lineSegment).p0);
    Ray_GetPoint(self_0, (*self_0).tMax, &mut (*lineSegment).p1);
}
#[no_mangle]
pub unsafe extern "C" fn Ray_FromLineSegment(
    mut lineSegment: *const LineSegment,
    mut self_0: *mut Ray,
) {
    LineSegment_ToRay(lineSegment, self_0);
}
