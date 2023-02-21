use ::libc;
use glam::Vec3;
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
    fn Intersect_RayPlane(_: *const Ray, _: *const Plane, pHit: *mut Vec3) -> bool;
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
    pub p0: Vec3,
    pub p1: Vec3,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Ray {
    pub p: Vec3,
    pub dir: Vec3,
    pub tMin: libc::c_float,
    pub tMax: libc::c_float,
}
#[no_mangle]
pub unsafe extern "C" fn Ray_GetPoint(
    mut this: *const Ray,
    mut t: libc::c_float,
    mut out: *mut Vec3,
) {
    *out = (*this).p + ((*this).dir * t);
}
#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectPlane(
    mut this: *const Ray,
    mut plane: *const Plane,
    mut pHit: *mut Vec3,
) -> bool {
    return Intersect_RayPlane(this, plane, pHit);
}
#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectTriangle_Barycentric(
    mut this: *const Ray,
    mut tri: *const Triangle,
    mut tEpsilon: libc::c_float,
    mut tHit: *mut libc::c_float,
) -> bool {
    return Intersect_RayTriangle_Barycentric(this, tri, tEpsilon, tHit);
}
#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectTriangle_Moller1(
    mut this: *const Ray,
    mut tri: *const Triangle,
    mut tHit: *mut libc::c_float,
) -> bool {
    return Intersect_RayTriangle_Moller1(this, tri, tHit);
}
#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectTriangle_Moller2(
    mut this: *const Ray,
    mut tri: *const Triangle,
    mut tHit: *mut libc::c_float,
) -> bool {
    return Intersect_RayTriangle_Moller2(this, tri, tHit);
}
#[no_mangle]
pub unsafe extern "C" fn Ray_ToLineSegment(
    mut this: *const Ray,
    mut lineSegment: *mut LineSegment,
) {
    Ray_GetPoint(this, (*this).tMin, &mut (*lineSegment).p0);
    Ray_GetPoint(this, (*this).tMax, &mut (*lineSegment).p1);
}
#[no_mangle]
pub unsafe extern "C" fn Ray_FromLineSegment(
    mut lineSegment: *const LineSegment,
    mut this: *mut Ray,
) {
    LineSegment_ToRay(lineSegment, this);
}
