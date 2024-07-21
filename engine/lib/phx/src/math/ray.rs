use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Ray {
    pub p: Position,
    pub dir: DVec3,
    pub tMin: f64,
    pub tMax: f64,
}

#[no_mangle]
pub extern "C" fn Ray_GetPoint(this: &Ray, t: f64, out: &mut Position) {
    *out = Position::from_dvec(this.p.v + this.dir * t);
}

#[no_mangle]
pub extern "C" fn Ray_IntersectPlane(this: &Ray, plane: &Plane, pHit: &mut Position) -> bool {
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
pub unsafe extern "C" fn Ray_ToLineSegment(this: &Ray, line_segment: &mut LineSegment) {
    Ray_GetPoint(this, this.tMin, &mut line_segment.p0);
    Ray_GetPoint(this, this.tMax, &mut line_segment.p1);
}

#[no_mangle]
pub unsafe extern "C" fn Ray_FromLineSegment(line_segment: &LineSegment, this: &mut Ray) {
    LineSegment_ToRay(line_segment, this);
}
