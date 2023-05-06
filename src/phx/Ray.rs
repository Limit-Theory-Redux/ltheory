

use crate::phx::Intersect::*;
use crate::phx::LineSegment::*;
use crate::phx::Math::Vec3;
use crate::phx::Plane::*;
use crate::phx::Triangle::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Ray {
    pub p: Vec3,
    pub dir: Vec3,
    pub tMin: f32,
    pub tMax: f32,
}

#[no_mangle]
pub extern "C" fn Ray_GetPoint(this: &Ray, t: f32, out: &mut Vec3) {
    *out = this.p + (this.dir * t);
}

#[no_mangle]
pub extern "C" fn Ray_IntersectPlane(this: &Ray, plane: &Plane, pHit: &mut Vec3) -> bool {
    Intersect_RayPlane(this, plane, pHit)
}

#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectTriangle_Barycentric(
    this: &Ray,
    tri: &Triangle,
    tEpsilon: f32,
    tHit: &mut f32,
) -> bool {
    Intersect_RayTriangle_Barycentric(this, tri, tEpsilon, tHit)
}

#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectTriangle_Moller1(
    this: &Ray,
    tri: &Triangle,
    tHit: &mut f32,
) -> bool {
    Intersect_RayTriangle_Moller1(this, tri, tHit)
}

#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectTriangle_Moller2(
    this: &Ray,
    tri: &Triangle,
    tHit: &mut f32,
) -> bool {
    Intersect_RayTriangle_Moller2(this, tri, tHit)
}

#[no_mangle]
pub extern "C" fn Ray_ToLineSegment(this: &Ray, lineSegment: &mut LineSegment) {
    Ray_GetPoint(this, this.tMin, &mut lineSegment.p0);
    Ray_GetPoint(this, this.tMax, &mut lineSegment.p1);
}

#[no_mangle]
pub extern "C" fn Ray_FromLineSegment(lineSegment: &LineSegment, this: &mut Ray) {
    LineSegment_ToRay(lineSegment, this);
}
