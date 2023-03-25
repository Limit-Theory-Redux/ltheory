use crate::internal::Memory::*;
use crate::Common::*;
use crate::Intersect::*;
use crate::LineSegment::*;
use crate::Math::Vec3;
use crate::Plane::*;
use crate::Triangle::*;
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
pub unsafe extern "C" fn Ray_GetPoint(this: *const Ray, t: f32, out: *mut Vec3) {
    *out = (*this).p + ((*this).dir * t);
}

#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectPlane(
    this: *const Ray,
    plane: *const Plane,
    pHit: *mut Vec3,
) -> bool {
    Intersect_RayPlane(this, plane, pHit)
}

#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectTriangle_Barycentric(
    this: *const Ray,
    tri: *const Triangle,
    tEpsilon: f32,
    tHit: *mut f32,
) -> bool {
    Intersect_RayTriangle_Barycentric(this, tri, tEpsilon, tHit)
}

#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectTriangle_Moller1(
    this: *const Ray,
    tri: *const Triangle,
    tHit: *mut f32,
) -> bool {
    Intersect_RayTriangle_Moller1(this, tri, tHit)
}

#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectTriangle_Moller2(
    this: *const Ray,
    tri: *const Triangle,
    tHit: *mut f32,
) -> bool {
    Intersect_RayTriangle_Moller2(this, tri, tHit)
}

#[no_mangle]
pub unsafe extern "C" fn Ray_ToLineSegment(this: *const Ray, lineSegment: *mut LineSegment) {
    Ray_GetPoint(this, (*this).tMin, &mut (*lineSegment).p0);
    Ray_GetPoint(this, (*this).tMax, &mut (*lineSegment).p1);
}

#[no_mangle]
pub unsafe extern "C" fn Ray_FromLineSegment(lineSegment: *const LineSegment, this: *mut Ray) {
    LineSegment_ToRay(lineSegment, this);
}
