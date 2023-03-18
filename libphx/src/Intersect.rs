use crate::internal::Memory::*;
use crate::Ray::*;
use crate::Triangle::*;
use crate::LineSegment::*;
use crate::Plane::*;
use crate::Matrix::*;
use glam::Vec3;
use libc;

extern "C" {
    fn fabs(_: f64) -> f64;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec4f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[inline]
unsafe extern "C" fn Abs(mut t: f64) -> f64 {
    return fabs(t);
}

#[inline]
unsafe extern "C" fn Absf(mut t: f32) -> f32 {
    return fabs(t as f64) as f32;
}

#[inline]
unsafe extern "C" fn Minf(mut a: f32, mut b: f32) -> f32 {
    return if a < b { a } else { b };
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_PointBox(mut src: *mut Matrix, mut dst: *mut Matrix) -> bool {
    let mut inv: *mut Matrix = Matrix_Inverse(dst);
    let mut srcPt = Vec3::ZERO;
    Matrix_GetPos(src, &mut srcPt);
    let mut dstPt = Vec3::ZERO;
    Matrix_MulPoint(inv, &mut dstPt, srcPt.x, srcPt.y, srcPt.z);
    Matrix_Free(inv);
    return -1.0f32 < dstPt.x
        && dstPt.x < 1.0f32
        && -1.0f32 < dstPt.y
        && dstPt.y < 1.0f32
        && -1.0f32 < dstPt.z
        && dstPt.z < 1.0f32;
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_PointTriangle_Barycentric(
    mut p: *const Vec3,
    mut tri: *const Triangle,
) -> bool {
    let mut v: *const Vec3 = ((*tri).vertices).as_ptr();
    let mut pv0: Vec3 = *v.offset(0) - *p;
    let mut pv1: Vec3 = *v.offset(1) - *p;
    let mut pv2: Vec3 = *v.offset(2) - *p;
    let mut plane: Plane = Plane {
        n: Vec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        },
        d: 0.,
    };
    Triangle_ToPlaneFast(tri, &mut plane);
    let mut areaABC: f32 = Vec3::dot(plane.n, plane.n);
    let mut areaPBC: f32 = Vec3::dot(plane.n, Vec3::cross(pv1, pv2));
    let mut areaPCA: f32 = Vec3::dot(plane.n, Vec3::cross(pv2, pv0));
    let mut A: f32 = areaPBC / areaABC;
    let mut B: f32 = areaPCA / areaABC;
    let mut C: f32 = 1.0f32 - A - B;
    let mut fuzzyMin: f32 = 0.0f32 - 0.01f32;
    let mut fuzzyMax: f32 = 1.0f32 + 0.01f32;
    return A > fuzzyMin
        && A < fuzzyMax
        && B > fuzzyMin
        && B < fuzzyMax
        && C > fuzzyMin
        && C < fuzzyMax;
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_RayPlane(
    mut ray: *const Ray,
    mut plane: *const Plane,
    mut pHit: *mut Vec3,
) -> bool {
    let mut dist: f32 = (*plane).d - Vec3::dot((*plane).n, (*ray).p);
    let mut denom: f32 = Vec3::dot((*plane).n, (*ray).dir);
    let mut t: f32 = dist / denom;
    if t >= (*ray).tMin && t <= (*ray).tMax {
        *pHit = (*ray).p + (*ray).dir * t;
        return 1_i32 != 0;
    }
    return 0_i32 != 0;
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_RayTriangle_Barycentric(
    mut ray: *const Ray,
    mut tri: *const Triangle,
    mut tEpsilon: f32,
    mut tHit: *mut f32,
) -> bool {
    let mut plane: Plane = Plane {
        n: Vec3 {
            x: 0.,
            y: 0.,
            z: 0.,
        },
        d: 0.,
    };
    Triangle_ToPlaneFast(tri, &mut plane);
    let mut dist: f32 = Vec3::dot(plane.n, (*ray).p) - plane.d;
    let mut denom: f32 = -Vec3::dot(plane.n, (*ray).dir);
    if denom != 0.0f32 {
        let mut t: f32 = dist / denom;
        if t > (*ray).tMin - tEpsilon && t < (*ray).tMax + tEpsilon {
            let mut v: *const Vec3 = ((*tri).vertices).as_ptr();
            let mut p = Vec3::ZERO;
            Ray_GetPoint(ray, t, &mut p);
            let mut pv0: Vec3 = *v.offset(0) - p;
            let mut pv1: Vec3 = *v.offset(1) - p;
            let mut pv2: Vec3 = *v.offset(2) - p;
            let mut areaABC: f32 = Vec3::dot(plane.n, plane.n);
            let mut areaPBC: f32 = Vec3::dot(plane.n, Vec3::cross(pv1, pv2));
            let mut areaPCA: f32 = Vec3::dot(plane.n, Vec3::cross(pv2, pv0));
            let mut A: f32 = areaPBC / areaABC;
            let mut B: f32 = areaPCA / areaABC;
            let mut C: f32 = 1.0f32 - A - B;
            let mut fuzzyMin: f32 = 0.0f32 - 0.01f32;
            let mut fuzzyMax: f32 = 1.0f32 + 0.01f32;
            if A > fuzzyMin
                && A < fuzzyMax
                && B > fuzzyMin
                && B < fuzzyMax
                && C > fuzzyMin
                && C < fuzzyMax
            {
                *tHit = t;
                return 1_i32 != 0;
            }
        }
    }
    return 0_i32 != 0;
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_RayTriangle_Moller1(
    mut ray: *const Ray,
    mut tri: *const Triangle,
    mut tHit: *mut f32,
) -> bool {
    let mut vt: *const Vec3 = ((*tri).vertices).as_ptr();
    let mut edge1: Vec3 = *vt.offset(1) - *vt.offset(0);
    let mut edge2: Vec3 = *vt.offset(2) - *vt.offset(0);
    let mut u: f32 = 0.;
    let mut v: f32 = 0.;
    let mut qvec = Vec3::ZERO;
    let mut pvec: Vec3 = Vec3::cross((*ray).dir, edge2);
    let epsilon: f32 = 0.000001f32;
    let mut det: f32 = Vec3::dot(edge1, pvec);
    if det > epsilon {
        let mut tvec: Vec3 = (*ray).p - *vt.offset(0);
        u = Vec3::dot(tvec, pvec);
        if (u as f64) < 0.0f64 || u > det {
            return 0_i32 != 0;
        }
        qvec = Vec3::cross(tvec, edge1);
        v = Vec3::dot((*ray).dir, qvec);
        if (v as f64) < 0.0f64 || u + v > det {
            return 0_i32 != 0;
        }
    } else if det < -epsilon {
        let mut tvec_0: Vec3 = (*ray).p - *vt.offset(0);
        u = Vec3::dot(tvec_0, pvec);
        if u as f64 > 0.0f64 || u < det {
            return 0_i32 != 0;
        }
        qvec = Vec3::cross(tvec_0, edge1);
        v = Vec3::dot((*ray).dir, qvec);
        if v as f64 > 0.0f64 || u + v < det {
            return 0_i32 != 0;
        }
    } else {
        return 0_i32 != 0;
    }
    let mut inv_det: f32 = 1.0f32 / det;
    *tHit = Vec3::dot(edge2, qvec) * inv_det;
    return 1_i32 != 0;
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_RayTriangle_Moller2(
    mut ray: *const Ray,
    mut tri: *const Triangle,
    mut tHit: *mut f32,
) -> bool {
    let mut vt: *const Vec3 = ((*tri).vertices).as_ptr();
    let mut edge1: Vec3 = *vt.offset(1) - *vt.offset(0);
    let mut edge2: Vec3 = *vt.offset(2) - *vt.offset(0);
    let mut pvec: Vec3 = Vec3::cross((*ray).dir, edge2);
    let mut det: f32 = Vec3::dot(edge1, pvec);
    if Abs(det as f64) < 0.000001f32 as f64 {
        return 0_i32 != 0;
    }
    let mut inv_det: f32 = 1.0f32 / det;
    let mut tvec: Vec3 = (*ray).p - *vt.offset(0);
    let mut fuzzyMin: f32 = 0.0f32 - 0.01f32;
    let mut fuzzyMax: f32 = 1.0f32 + 0.01f32;
    let mut u: f32 = Vec3::dot(tvec, pvec) * inv_det;
    if u < fuzzyMin || u > fuzzyMax {
        return 0_i32 != 0;
    }
    let mut qvec: Vec3 = Vec3::cross(tvec, edge1);
    let mut v: f32 = Vec3::dot((*ray).dir, qvec) * inv_det;
    if v < fuzzyMin || u + v > fuzzyMax {
        return 0_i32 != 0;
    }
    *tHit = Vec3::dot(edge2, qvec) * inv_det;
    return 1_i32 != 0;
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_LineSegmentPlane(
    mut lineSegment: *const LineSegment,
    mut plane: *const Plane,
    mut pHit: *mut Vec3,
) -> bool {
    let mut dir: Vec3 = (*lineSegment).p1 - (*lineSegment).p0;
    let mut ray: Ray = Ray {
        p: (*lineSegment).p0,
        dir: dir,
        tMin: 0.0f32,
        tMax: 1.0f32,
    };
    return Intersect_RayPlane(&mut ray, plane, pHit);
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_RectRect(mut a: *const Vec4f, mut b: *const Vec4f) -> bool {
    let mut a2: Vec4f = Vec4f {
        x: (*a).x + Minf((*a).z, 0.0f32),
        y: (*a).y + Minf((*a).w, 0.0f32),
        z: Absf((*a).z),
        w: Absf((*a).w),
    };
    let mut b2: Vec4f = Vec4f {
        x: (*b).x + Minf((*b).z, 0.0f32),
        y: (*b).y + Minf((*b).w, 0.0f32),
        z: Absf((*b).z),
        w: Absf((*b).w),
    };
    return Intersect_RectRectFast(&mut a2, &mut b2);
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_RectRectFast(mut a: *const Vec4f, mut b: *const Vec4f) -> bool {
    let mut result: bool = 1_i32 != 0;
    result = (result as i32 & ((*a).x < (*b).x + (*b).z) as i32) != 0;
    result = (result as i32 & ((*b).x < (*a).x + (*a).z) as i32) != 0;
    result = (result as i32 & ((*a).y < (*b).y + (*b).w) as i32) != 0;
    result = (result as i32 & ((*b).y < (*a).y + (*a).w) as i32) != 0;
    return result;
}

#[inline]
unsafe extern "C" fn ClosestPoint_PointToTriangle(
    mut p: *const Vec3,
    mut tri: *const Triangle,
) -> Vec3 {
    let mut a: Vec3 = (*tri).vertices[0];
    let mut b: Vec3 = (*tri).vertices[1];
    let mut c: Vec3 = (*tri).vertices[2];
    let mut ab: Vec3 = b - a;
    let mut ac: Vec3 = c - a;
    let mut ap: Vec3 = *p - a;
    let mut d1: f32 = Vec3::dot(ab, ap);
    let mut d2: f32 = Vec3::dot(ac, ap);
    if d1 <= 0.0f32 && d2 <= 0.0f32 {
        return a;
    }
    let mut bp: Vec3 = *p - b;
    let mut d3: f32 = Vec3::dot(ab, bp);
    let mut d4: f32 = Vec3::dot(ac, bp);
    if d3 >= 0.0f32 && d4 <= d3 {
        return b;
    }
    let mut vc: f32 = d1 * d4 - d3 * d2;
    if vc <= 0.0f32 && d1 >= 0.0f32 && d3 <= 0.0f32 {
        let mut v: f32 = d1 / (d1 - d3);
        return a + ab * v;
    }
    let mut cp: Vec3 = *p - c;
    let mut d5: f32 = Vec3::dot(ab, cp);
    let mut d6: f32 = Vec3::dot(ac, cp);
    if d6 >= 0.0f32 && d5 <= d6 {
        return c;
    }
    let mut vb: f32 = d5 * d2 - d1 * d6;
    if vb <= 0.0f32 && d2 >= 0.0f32 && d6 <= 0.0f32 {
        let mut w: f32 = d2 / (d2 - d6);
        return a + ac * w;
    }
    let mut va: f32 = d3 * d6 - d5 * d4;
    let mut d4m3: f32 = d4 - d3;
    let mut d5m6: f32 = d5 - d6;
    if va <= 0.0f32 && d4m3 >= 0.0f32 && d5m6 >= 0.0f32 {
        let mut w_0: f32 = d4m3 / (d4m3 + d5m6);
        let mut bc: Vec3 = c - b;
        return b + bc * w_0;
    }
    let mut denom: f32 = 1.0f32 / (va + vb + vc);
    let mut v_0: f32 = vb * denom;
    let mut w_1: f32 = vc * denom;
    return a + ab * v_0 + ac * w_1;
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_SphereTriangle(
    mut sphere: *const Sphere,
    mut triangle: *const Triangle,
    mut pHit: *mut Vec3,
) -> bool {
    let mut pClosest: Vec3 = ClosestPoint_PointToTriangle(&(*sphere).p, triangle);
    let mut distSq: f32 = (*sphere).p.distance_squared(pClosest);
    if distSq < (*sphere).r * (*sphere).r {
        *pHit = pClosest;
        return 1_i32 != 0;
    }
    return 0_i32 != 0;
}
