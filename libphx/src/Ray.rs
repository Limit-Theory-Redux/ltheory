use crate::internal::Memory::*;
use crate::Intersect::*;
use crate::Plane::*;
use crate::Triangle::*;
use crate::LineSegment::*;
use glam::Vec3;
use libc;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Ray {
    pub p: Vec3,
    pub dir: Vec3,
    pub tMin: f32,
    pub tMax: f32,
}

#[no_mangle]
pub unsafe extern "C" fn Ray_GetPoint(mut this: *const Ray, mut t: f32, mut out: *mut Vec3) {
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
    mut tEpsilon: f32,
    mut tHit: *mut f32,
) -> bool {
    return Intersect_RayTriangle_Barycentric(this, tri, tEpsilon, tHit);
}

#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectTriangle_Moller1(
    mut this: *const Ray,
    mut tri: *const Triangle,
    mut tHit: *mut f32,
) -> bool {
    return Intersect_RayTriangle_Moller1(this, tri, tHit);
}

#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectTriangle_Moller2(
    mut this: *const Ray,
    mut tri: *const Triangle,
    mut tHit: *mut f32,
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
