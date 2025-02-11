#![allow(unsafe_code)] // TODO: remove

use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Ray {
    pub p: Position,
    pub dir: DVec3,
    pub t_min: f64,
    pub t_max: f64,
}

#[no_mangle]
pub extern "C" fn Ray_GetPoint(this: &Ray, t: f64, out: &mut Position) {
    *out = Position::from_dvec(this.p.v + this.dir * t);
}

#[no_mangle]
pub extern "C" fn Ray_IntersectPlane(this: &Ray, plane: &Plane, p_hit: &mut Position) -> bool {
    Intersect_RayPlane(this, plane, p_hit)
}

#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectTriangle_Barycentric(
    this: &Ray,
    tri: &Triangle,
    t_epsilon: f32,
    t_hit: &mut f32,
) -> bool {
    Intersect_RayTriangle_Barycentric(this, tri, t_epsilon, t_hit)
}

#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectTriangle_Moller1(
    this: &Ray,
    tri: &Triangle,
    t_hit: &mut f32,
) -> bool {
    Intersect_RayTriangle_Moller1(this, tri, t_hit)
}

#[no_mangle]
pub unsafe extern "C" fn Ray_IntersectTriangle_Moller2(
    this: &Ray,
    tri: &Triangle,
    t_hit: &mut f32,
) -> bool {
    Intersect_RayTriangle_Moller2(this, tri, t_hit)
}

#[no_mangle]
pub unsafe extern "C" fn Ray_ToLineSegment(this: &Ray, line_segment: &mut LineSegment) {
    Ray_GetPoint(this, this.t_min, &mut line_segment.p0);
    Ray_GetPoint(this, this.t_max, &mut line_segment.p1);
}

#[no_mangle]
pub unsafe extern "C" fn Ray_FromLineSegment(line_segment: &LineSegment, this: &mut Ray) {
    LineSegment_ToRay(line_segment, this);
}
