use crate::internal::Memory::*;
use crate::Common::*;
use crate::LineSegment::*;
use crate::Math::Sphere;
use crate::Math::Vec3;
use crate::Math::Vec4;
use crate::Matrix::*;
use crate::Plane::*;
use crate::Ray::*;
use crate::Triangle::*;
use libc;

#[no_mangle]
pub unsafe extern "C" fn Intersect_PointBox(src: *mut Matrix, dst: *mut Matrix) -> bool {
    let inv: *mut Matrix = Matrix_Inverse(dst);
    let mut srcPt = Vec3::ZERO;
    Matrix_GetPos(src, &mut srcPt);
    let mut dstPt = Vec3::ZERO;
    Matrix_MulPoint(inv, &mut dstPt, srcPt.x, srcPt.y, srcPt.z);
    Matrix_Free(inv);
    -1.0f32 < dstPt.x
        && dstPt.x < 1.0f32
        && -1.0f32 < dstPt.y
        && dstPt.y < 1.0f32
        && -1.0f32 < dstPt.z
        && dstPt.z < 1.0f32
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_PointTriangle_Barycentric(
    p: *const Vec3,
    tri: *const Triangle,
) -> bool {
    let v: *const Vec3 = ((*tri).vertices).as_ptr();
    let pv0: Vec3 = *v.offset(0) - *p;
    let pv1: Vec3 = *v.offset(1) - *p;
    let pv2: Vec3 = *v.offset(2) - *p;
    let mut plane: Plane = Plane {
        n: Vec3::ZERO,
        d: 0.,
    };
    Triangle_ToPlaneFast(tri, &mut plane);
    let areaABC: f32 = Vec3::dot(plane.n, plane.n);
    let areaPBC: f32 = Vec3::dot(plane.n, Vec3::cross(pv1, pv2));
    let areaPCA: f32 = Vec3::dot(plane.n, Vec3::cross(pv2, pv0));
    let A: f32 = areaPBC / areaABC;
    let B: f32 = areaPCA / areaABC;
    let C: f32 = 1.0f32 - A - B;
    let fuzzyMin: f32 = 0.0f32 - 0.01f32;
    let fuzzyMax: f32 = 1.0f32 + 0.01f32;
    A > fuzzyMin && A < fuzzyMax && B > fuzzyMin && B < fuzzyMax && C > fuzzyMin && C < fuzzyMax
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_RayPlane(
    ray: *const Ray,
    plane: *const Plane,
    pHit: *mut Vec3,
) -> bool {
    let dist: f32 = (*plane).d - Vec3::dot((*plane).n, (*ray).p);
    let denom: f32 = Vec3::dot((*plane).n, (*ray).dir);
    let t: f32 = dist / denom;
    if t >= (*ray).tMin && t <= (*ray).tMax {
        *pHit = (*ray).p + (*ray).dir * t;
        return true;
    }
    false
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_RayTriangle_Barycentric(
    ray: *const Ray,
    tri: *const Triangle,
    tEpsilon: f32,
    tHit: *mut f32,
) -> bool {
    let mut plane: Plane = Plane {
        n: Vec3::ZERO,
        d: 0.,
    };
    Triangle_ToPlaneFast(tri, &mut plane);
    let dist: f32 = Vec3::dot(plane.n, (*ray).p) - plane.d;
    let denom: f32 = -Vec3::dot(plane.n, (*ray).dir);
    if denom != 0.0f32 {
        let t: f32 = dist / denom;
        if t > (*ray).tMin - tEpsilon && t < (*ray).tMax + tEpsilon {
            let v: *const Vec3 = ((*tri).vertices).as_ptr();
            let mut p = Vec3::ZERO;
            Ray_GetPoint(ray, t, &mut p);
            let pv0: Vec3 = *v.offset(0) - p;
            let pv1: Vec3 = *v.offset(1) - p;
            let pv2: Vec3 = *v.offset(2) - p;
            let areaABC: f32 = Vec3::dot(plane.n, plane.n);
            let areaPBC: f32 = Vec3::dot(plane.n, Vec3::cross(pv1, pv2));
            let areaPCA: f32 = Vec3::dot(plane.n, Vec3::cross(pv2, pv0));
            let A: f32 = areaPBC / areaABC;
            let B: f32 = areaPCA / areaABC;
            let C: f32 = 1.0f32 - A - B;
            let fuzzyMin: f32 = 0.0f32 - 0.01f32;
            let fuzzyMax: f32 = 1.0f32 + 0.01f32;
            if A > fuzzyMin
                && A < fuzzyMax
                && B > fuzzyMin
                && B < fuzzyMax
                && C > fuzzyMin
                && C < fuzzyMax
            {
                *tHit = t;
                return true;
            }
        }
    }
    false
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_RayTriangle_Moller1(
    ray: *const Ray,
    tri: *const Triangle,
    tHit: *mut f32,
) -> bool {
    let vt: *const Vec3 = ((*tri).vertices).as_ptr();
    let edge1: Vec3 = *vt.offset(1) - *vt.offset(0);
    let edge2: Vec3 = *vt.offset(2) - *vt.offset(0);
    let mut u: f32 = 0.;
    let mut v: f32 = 0.;
    let mut qvec = Vec3::ZERO;
    let pvec: Vec3 = Vec3::cross((*ray).dir, edge2);
    let epsilon: f32 = 0.000001f32;
    let det: f32 = Vec3::dot(edge1, pvec);
    if det > epsilon {
        let tvec: Vec3 = (*ray).p - *vt.offset(0);
        u = Vec3::dot(tvec, pvec);
        if (u as f64) < 0.0f64 || u > det {
            return false;
        }
        qvec = Vec3::cross(tvec, edge1);
        v = Vec3::dot((*ray).dir, qvec);
        if (v as f64) < 0.0f64 || u + v > det {
            return false;
        }
    } else if det < -epsilon {
        let tvec_0: Vec3 = (*ray).p - *vt.offset(0);
        u = Vec3::dot(tvec_0, pvec);
        if u as f64 > 0.0f64 || u < det {
            return false;
        }
        qvec = Vec3::cross(tvec_0, edge1);
        v = Vec3::dot((*ray).dir, qvec);
        if v as f64 > 0.0f64 || u + v < det {
            return false;
        }
    } else {
        return false;
    }
    let inv_det: f32 = 1.0f32 / det;
    *tHit = Vec3::dot(edge2, qvec) * inv_det;
    true
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_RayTriangle_Moller2(
    ray: *const Ray,
    tri: *const Triangle,
    tHit: *mut f32,
) -> bool {
    let vt: *const Vec3 = ((*tri).vertices).as_ptr();
    let edge1: Vec3 = *vt.offset(1) - *vt.offset(0);
    let edge2: Vec3 = *vt.offset(2) - *vt.offset(0);
    let pvec: Vec3 = Vec3::cross((*ray).dir, edge2);
    let det: f32 = Vec3::dot(edge1, pvec);
    if f32::abs(det) < 0.000001f32 {
        return false;
    }
    let inv_det: f32 = 1.0f32 / det;
    let tvec: Vec3 = (*ray).p - *vt.offset(0);
    let fuzzyMin: f32 = 0.0f32 - 0.01f32;
    let fuzzyMax: f32 = 1.0f32 + 0.01f32;
    let u: f32 = Vec3::dot(tvec, pvec) * inv_det;
    if u < fuzzyMin || u > fuzzyMax {
        return false;
    }
    let qvec: Vec3 = Vec3::cross(tvec, edge1);
    let v: f32 = Vec3::dot((*ray).dir, qvec) * inv_det;
    if v < fuzzyMin || u + v > fuzzyMax {
        return false;
    }
    *tHit = Vec3::dot(edge2, qvec) * inv_det;
    true
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_LineSegmentPlane(
    lineSegment: *const LineSegment,
    plane: *const Plane,
    pHit: *mut Vec3,
) -> bool {
    let dir: Vec3 = (*lineSegment).p1 - (*lineSegment).p0;
    let mut ray: Ray = Ray {
        p: (*lineSegment).p0,
        dir: dir,
        tMin: 0.0f32,
        tMax: 1.0f32,
    };
    Intersect_RayPlane(&mut ray, plane, pHit)
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_RectRect(a: *const Vec4, b: *const Vec4) -> bool {
    let mut a2: Vec4 = Vec4::new(
        (*a).x + f32::min((*a).z, 0.0f32),
        (*a).y + f32::min((*a).w, 0.0f32),
        f32::abs((*a).z),
        f32::abs((*a).w),
    );
    let mut b2: Vec4 = Vec4::new(
        (*b).x + f32::min((*b).z, 0.0f32),
        (*b).y + f32::min((*b).w, 0.0f32),
        f32::abs((*b).z),
        f32::abs((*b).w),
    );
    Intersect_RectRectFast(&mut a2, &mut b2)
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_RectRectFast(a: *const Vec4, b: *const Vec4) -> bool {
    let mut result: bool = true;
    result = (result as i32 & ((*a).x < (*b).x + (*b).z) as i32) != 0;
    result = (result as i32 & ((*b).x < (*a).x + (*a).z) as i32) != 0;
    result = (result as i32 & ((*a).y < (*b).y + (*b).w) as i32) != 0;
    result = (result as i32 & ((*b).y < (*a).y + (*a).w) as i32) != 0;
    result
}

#[inline]
unsafe extern "C" fn ClosestPoint_PointToTriangle(p: *const Vec3, tri: *const Triangle) -> Vec3 {
    let a: Vec3 = (*tri).vertices[0];
    let b: Vec3 = (*tri).vertices[1];
    let c: Vec3 = (*tri).vertices[2];
    let ab: Vec3 = b - a;
    let ac: Vec3 = c - a;
    let ap: Vec3 = *p - a;
    let d1: f32 = Vec3::dot(ab, ap);
    let d2: f32 = Vec3::dot(ac, ap);
    if d1 <= 0.0f32 && d2 <= 0.0f32 {
        return a;
    }
    let bp: Vec3 = *p - b;
    let d3: f32 = Vec3::dot(ab, bp);
    let d4: f32 = Vec3::dot(ac, bp);
    if d3 >= 0.0f32 && d4 <= d3 {
        return b;
    }
    let vc: f32 = d1 * d4 - d3 * d2;
    if vc <= 0.0f32 && d1 >= 0.0f32 && d3 <= 0.0f32 {
        let v: f32 = d1 / (d1 - d3);
        return a + ab * v;
    }
    let cp: Vec3 = *p - c;
    let d5: f32 = Vec3::dot(ab, cp);
    let d6: f32 = Vec3::dot(ac, cp);
    if d6 >= 0.0f32 && d5 <= d6 {
        return c;
    }
    let vb: f32 = d5 * d2 - d1 * d6;
    if vb <= 0.0f32 && d2 >= 0.0f32 && d6 <= 0.0f32 {
        let w: f32 = d2 / (d2 - d6);
        return a + ac * w;
    }
    let va: f32 = d3 * d6 - d5 * d4;
    let d4m3: f32 = d4 - d3;
    let d5m6: f32 = d5 - d6;
    if va <= 0.0f32 && d4m3 >= 0.0f32 && d5m6 >= 0.0f32 {
        let w_0: f32 = d4m3 / (d4m3 + d5m6);
        let bc: Vec3 = c - b;
        return b + bc * w_0;
    }
    let denom: f32 = 1.0f32 / (va + vb + vc);
    let v_0: f32 = vb * denom;
    let w_1: f32 = vc * denom;
    a + ab * v_0 + ac * w_1
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_SphereTriangle(
    sphere: *const Sphere,
    triangle: *const Triangle,
    pHit: *mut Vec3,
) -> bool {
    let pClosest: Vec3 = ClosestPoint_PointToTriangle(&(*sphere).p, triangle);
    let distSq: f32 = (*sphere).p.distance_squared(pClosest);
    if distSq < (*sphere).r * (*sphere).r {
        *pHit = pClosest;
        return true;
    }
    false
}
