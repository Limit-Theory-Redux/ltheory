use ::libc;
use super::internal::Memory::*;
extern "C" {
    pub type Matrix;
    fn fabs(_: libc::c_double) -> libc::c_double;
    fn Matrix_Free(_: *mut Matrix);
    fn Matrix_Inverse(_: *const Matrix) -> *mut Matrix;
    fn Matrix_MulPoint(
        _: *const Matrix,
        out: *mut Vec3f,
        x: libc::c_float,
        y: libc::c_float,
        z: libc::c_float,
    );
    fn Matrix_GetPos(_: *const Matrix, out: *mut Vec3f);
    fn Ray_GetPoint(_: *const Ray, t: libc::c_float, out: *mut Vec3f);
    fn Triangle_ToPlaneFast(_: *const Triangle, _: *mut Plane);
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
pub struct Plane {
    pub n: Vec3f,
    pub d: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Ray {
    pub p: Vec3f,
    pub dir: Vec3f,
    pub tMin: libc::c_float,
    pub tMax: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Sphere {
    pub p: Vec3f,
    pub r: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Triangle {
    pub vertices: [Vec3f; 3],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec4f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
    pub w: libc::c_float,
}
#[inline]
unsafe extern "C" fn Abs(mut t: libc::c_double) -> libc::c_double {
    return fabs(t);
}
#[inline]
unsafe extern "C" fn Absf(mut t: libc::c_float) -> libc::c_float {
    return fabs(t as libc::c_double) as libc::c_float;
}
#[inline]
unsafe extern "C" fn Vec3f_Dot(mut a: Vec3f, mut b: Vec3f) -> libc::c_float {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}
#[inline]
unsafe extern "C" fn Vec3f_DistanceSquared(mut a: Vec3f, mut b: Vec3f) -> libc::c_float {
    let mut dx: libc::c_float = a.x - b.x;
    let mut dy: libc::c_float = a.y - b.y;
    let mut dz: libc::c_float = a.z - b.z;
    return dx * dx + dy * dy + dz * dz;
}
#[inline]
unsafe extern "C" fn Vec3f_Cross(mut a: Vec3f, mut b: Vec3f) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: b.z * a.y - b.y * a.z,
            y: b.x * a.z - b.z * a.x,
            z: b.y * a.x - b.x * a.y,
        };
        init
    };
    return self_0;
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
unsafe extern "C" fn Vec3f_Sub(mut a: Vec3f, mut b: Vec3f) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: a.x - b.x,
            y: a.y - b.y,
            z: a.z - b.z,
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
#[inline]
unsafe extern "C" fn Minf(mut a: libc::c_float, mut b: libc::c_float) -> libc::c_float {
    return if a < b { a } else { b };
}
#[no_mangle]
pub unsafe extern "C" fn Intersect_PointBox(
    mut src: *mut Matrix,
    mut dst: *mut Matrix,
) -> bool {
    let mut inv: *mut Matrix = Matrix_Inverse(dst);
    let mut srcPt: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
    Matrix_GetPos(src, &mut srcPt);
    let mut dstPt: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
    Matrix_MulPoint(inv, &mut dstPt, srcPt.x, srcPt.y, srcPt.z);
    Matrix_Free(inv);
    return -1.0f32 < dstPt.x && dstPt.x < 1.0f32 && -1.0f32 < dstPt.y && dstPt.y < 1.0f32
        && -1.0f32 < dstPt.z && dstPt.z < 1.0f32;
}
#[no_mangle]
pub unsafe extern "C" fn Intersect_PointTriangle_Barycentric(
    mut p: *const Vec3f,
    mut tri: *const Triangle,
) -> bool {
    let mut v: *const Vec3f = ((*tri).vertices).as_ptr();
    let mut pv0: Vec3f = Vec3f_Sub(*v.offset(0 as libc::c_int as isize), *p);
    let mut pv1: Vec3f = Vec3f_Sub(*v.offset(1 as libc::c_int as isize), *p);
    let mut pv2: Vec3f = Vec3f_Sub(*v.offset(2 as libc::c_int as isize), *p);
    let mut plane: Plane = Plane {
        n: Vec3f { x: 0., y: 0., z: 0. },
        d: 0.,
    };
    Triangle_ToPlaneFast(tri, &mut plane);
    let mut areaABC: libc::c_float = Vec3f_Dot(plane.n, plane.n);
    let mut areaPBC: libc::c_float = Vec3f_Dot(plane.n, Vec3f_Cross(pv1, pv2));
    let mut areaPCA: libc::c_float = Vec3f_Dot(plane.n, Vec3f_Cross(pv2, pv0));
    let mut A: libc::c_float = areaPBC / areaABC;
    let mut B: libc::c_float = areaPCA / areaABC;
    let mut C: libc::c_float = 1.0f32 - A - B;
    let mut fuzzyMin: libc::c_float = 0.0f32 - 0.01f32;
    let mut fuzzyMax: libc::c_float = 1.0f32 + 0.01f32;
    return A > fuzzyMin && A < fuzzyMax && B > fuzzyMin && B < fuzzyMax && C > fuzzyMin
        && C < fuzzyMax;
}
#[no_mangle]
pub unsafe extern "C" fn Intersect_RayPlane(
    mut ray: *const Ray,
    mut plane: *const Plane,
    mut pHit: *mut Vec3f,
) -> bool {
    let mut dist: libc::c_float = (*plane).d - Vec3f_Dot((*plane).n, (*ray).p);
    let mut denom: libc::c_float = Vec3f_Dot((*plane).n, (*ray).dir);
    let mut t: libc::c_float = dist / denom;
    if t >= (*ray).tMin && t <= (*ray).tMax {
        *pHit = Vec3f_Add((*ray).p, Vec3f_Muls((*ray).dir, t));
        return 1 as libc::c_int != 0;
    }
    return 0 as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn Intersect_RayTriangle_Barycentric(
    mut ray: *const Ray,
    mut tri: *const Triangle,
    mut tEpsilon: libc::c_float,
    mut tHit: *mut libc::c_float,
) -> bool {
    let mut plane: Plane = Plane {
        n: Vec3f { x: 0., y: 0., z: 0. },
        d: 0.,
    };
    Triangle_ToPlaneFast(tri, &mut plane);
    let mut dist: libc::c_float = Vec3f_Dot(plane.n, (*ray).p) - plane.d;
    let mut denom: libc::c_float = -Vec3f_Dot(plane.n, (*ray).dir);
    if denom != 0.0f32 {
        let mut t: libc::c_float = dist / denom;
        if t > (*ray).tMin - tEpsilon && t < (*ray).tMax + tEpsilon {
            let mut v: *const Vec3f = ((*tri).vertices).as_ptr();
            let mut p: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
            Ray_GetPoint(ray, t, &mut p);
            let mut pv0: Vec3f = Vec3f_Sub(*v.offset(0 as libc::c_int as isize), p);
            let mut pv1: Vec3f = Vec3f_Sub(*v.offset(1 as libc::c_int as isize), p);
            let mut pv2: Vec3f = Vec3f_Sub(*v.offset(2 as libc::c_int as isize), p);
            let mut areaABC: libc::c_float = Vec3f_Dot(plane.n, plane.n);
            let mut areaPBC: libc::c_float = Vec3f_Dot(plane.n, Vec3f_Cross(pv1, pv2));
            let mut areaPCA: libc::c_float = Vec3f_Dot(plane.n, Vec3f_Cross(pv2, pv0));
            let mut A: libc::c_float = areaPBC / areaABC;
            let mut B: libc::c_float = areaPCA / areaABC;
            let mut C: libc::c_float = 1.0f32 - A - B;
            let mut fuzzyMin: libc::c_float = 0.0f32 - 0.01f32;
            let mut fuzzyMax: libc::c_float = 1.0f32 + 0.01f32;
            if A > fuzzyMin && A < fuzzyMax && B > fuzzyMin && B < fuzzyMax
                && C > fuzzyMin && C < fuzzyMax
            {
                *tHit = t;
                return 1 as libc::c_int != 0;
            }
        }
    }
    return 0 as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn Intersect_RayTriangle_Moller1(
    mut ray: *const Ray,
    mut tri: *const Triangle,
    mut tHit: *mut libc::c_float,
) -> bool {
    let mut vt: *const Vec3f = ((*tri).vertices).as_ptr();
    let mut edge1: Vec3f = Vec3f_Sub(
        *vt.offset(1 as libc::c_int as isize),
        *vt.offset(0 as libc::c_int as isize),
    );
    let mut edge2: Vec3f = Vec3f_Sub(
        *vt.offset(2 as libc::c_int as isize),
        *vt.offset(0 as libc::c_int as isize),
    );
    let mut u: libc::c_float = 0.;
    let mut v: libc::c_float = 0.;
    let mut qvec: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
    let mut pvec: Vec3f = Vec3f_Cross((*ray).dir, edge2);
    let epsilon: libc::c_float = 0.000001f32;
    let mut det: libc::c_float = Vec3f_Dot(edge1, pvec);
    if det > epsilon {
        let mut tvec: Vec3f = Vec3f_Sub((*ray).p, *vt.offset(0 as libc::c_int as isize));
        u = Vec3f_Dot(tvec, pvec);
        if (u as libc::c_double) < 0.0f64 || u > det {
            return 0 as libc::c_int != 0;
        }
        qvec = Vec3f_Cross(tvec, edge1);
        v = Vec3f_Dot((*ray).dir, qvec);
        if (v as libc::c_double) < 0.0f64 || u + v > det {
            return 0 as libc::c_int != 0;
        }
    } else if det < -epsilon {
        let mut tvec_0: Vec3f = Vec3f_Sub(
            (*ray).p,
            *vt.offset(0 as libc::c_int as isize),
        );
        u = Vec3f_Dot(tvec_0, pvec);
        if u as libc::c_double > 0.0f64 || u < det {
            return 0 as libc::c_int != 0;
        }
        qvec = Vec3f_Cross(tvec_0, edge1);
        v = Vec3f_Dot((*ray).dir, qvec);
        if v as libc::c_double > 0.0f64 || u + v < det {
            return 0 as libc::c_int != 0;
        }
    } else {
        return 0 as libc::c_int != 0
    }
    let mut inv_det: libc::c_float = 1.0f32 / det;
    *tHit = Vec3f_Dot(edge2, qvec) * inv_det;
    return 1 as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn Intersect_RayTriangle_Moller2(
    mut ray: *const Ray,
    mut tri: *const Triangle,
    mut tHit: *mut libc::c_float,
) -> bool {
    let mut vt: *const Vec3f = ((*tri).vertices).as_ptr();
    let mut edge1: Vec3f = Vec3f_Sub(
        *vt.offset(1 as libc::c_int as isize),
        *vt.offset(0 as libc::c_int as isize),
    );
    let mut edge2: Vec3f = Vec3f_Sub(
        *vt.offset(2 as libc::c_int as isize),
        *vt.offset(0 as libc::c_int as isize),
    );
    let mut pvec: Vec3f = Vec3f_Cross((*ray).dir, edge2);
    let mut det: libc::c_float = Vec3f_Dot(edge1, pvec);
    if Abs(det as libc::c_double) < 0.000001f32 as libc::c_double {
        return 0 as libc::c_int != 0;
    }
    let mut inv_det: libc::c_float = 1.0f32 / det;
    let mut tvec: Vec3f = Vec3f_Sub((*ray).p, *vt.offset(0 as libc::c_int as isize));
    let mut fuzzyMin: libc::c_float = 0.0f32 - 0.01f32;
    let mut fuzzyMax: libc::c_float = 1.0f32 + 0.01f32;
    let mut u: libc::c_float = Vec3f_Dot(tvec, pvec) * inv_det;
    if u < fuzzyMin || u > fuzzyMax {
        return 0 as libc::c_int != 0;
    }
    let mut qvec: Vec3f = Vec3f_Cross(tvec, edge1);
    let mut v: libc::c_float = Vec3f_Dot((*ray).dir, qvec) * inv_det;
    if v < fuzzyMin || u + v > fuzzyMax {
        return 0 as libc::c_int != 0;
    }
    *tHit = Vec3f_Dot(edge2, qvec) * inv_det;
    return 1 as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn Intersect_LineSegmentPlane(
    mut lineSegment: *const LineSegment,
    mut plane: *const Plane,
    mut pHit: *mut Vec3f,
) -> bool {
    let mut dir: Vec3f = Vec3f_Sub((*lineSegment).p1, (*lineSegment).p0);
    let mut ray: Ray = {
        let mut init = Ray {
            p: (*lineSegment).p0,
            dir: dir,
            tMin: 0.0f32,
            tMax: 1.0f32,
        };
        init
    };
    return Intersect_RayPlane(&mut ray, plane, pHit);
}
#[no_mangle]
pub unsafe extern "C" fn Intersect_RectRect(
    mut a: *const Vec4f,
    mut b: *const Vec4f,
) -> bool {
    let mut a2: Vec4f = {
        let mut init = Vec4f {
            x: (*a).x + Minf((*a).z, 0.0f32),
            y: (*a).y + Minf((*a).w, 0.0f32),
            z: Absf((*a).z),
            w: Absf((*a).w),
        };
        init
    };
    let mut b2: Vec4f = {
        let mut init = Vec4f {
            x: (*b).x + Minf((*b).z, 0.0f32),
            y: (*b).y + Minf((*b).w, 0.0f32),
            z: Absf((*b).z),
            w: Absf((*b).w),
        };
        init
    };
    return Intersect_RectRectFast(&mut a2, &mut b2);
}
#[no_mangle]
pub unsafe extern "C" fn Intersect_RectRectFast(
    mut a: *const Vec4f,
    mut b: *const Vec4f,
) -> bool {
    let mut result: bool = 1 as libc::c_int != 0;
    result = (result as libc::c_int & ((*a).x < (*b).x + (*b).z) as libc::c_int) as bool;
    result = (result as libc::c_int & ((*b).x < (*a).x + (*a).z) as libc::c_int) as bool;
    result = (result as libc::c_int & ((*a).y < (*b).y + (*b).w) as libc::c_int) as bool;
    result = (result as libc::c_int & ((*b).y < (*a).y + (*a).w) as libc::c_int) as bool;
    return result;
}
#[inline]
unsafe extern "C" fn ClosestPoint_PointToTriangle(
    mut p: *const Vec3f,
    mut tri: *const Triangle,
) -> Vec3f {
    let mut a: Vec3f = (*tri).vertices[0 as libc::c_int as usize];
    let mut b: Vec3f = (*tri).vertices[1 as libc::c_int as usize];
    let mut c: Vec3f = (*tri).vertices[2 as libc::c_int as usize];
    let mut ab: Vec3f = Vec3f_Sub(b, a);
    let mut ac: Vec3f = Vec3f_Sub(c, a);
    let mut ap: Vec3f = Vec3f_Sub(*p, a);
    let mut d1: libc::c_float = Vec3f_Dot(ab, ap);
    let mut d2: libc::c_float = Vec3f_Dot(ac, ap);
    if d1 <= 0.0f32 && d2 <= 0.0f32 {
        return a;
    }
    let mut bp: Vec3f = Vec3f_Sub(*p, b);
    let mut d3: libc::c_float = Vec3f_Dot(ab, bp);
    let mut d4: libc::c_float = Vec3f_Dot(ac, bp);
    if d3 >= 0.0f32 && d4 <= d3 {
        return b;
    }
    let mut vc: libc::c_float = d1 * d4 - d3 * d2;
    if vc <= 0.0f32 && d1 >= 0.0f32 && d3 <= 0.0f32 {
        let mut v: libc::c_float = d1 / (d1 - d3);
        return Vec3f_Add(a, Vec3f_Muls(ab, v));
    }
    let mut cp: Vec3f = Vec3f_Sub(*p, c);
    let mut d5: libc::c_float = Vec3f_Dot(ab, cp);
    let mut d6: libc::c_float = Vec3f_Dot(ac, cp);
    if d6 >= 0.0f32 && d5 <= d6 {
        return c;
    }
    let mut vb: libc::c_float = d5 * d2 - d1 * d6;
    if vb <= 0.0f32 && d2 >= 0.0f32 && d6 <= 0.0f32 {
        let mut w: libc::c_float = d2 / (d2 - d6);
        return Vec3f_Add(a, Vec3f_Muls(ac, w));
    }
    let mut va: libc::c_float = d3 * d6 - d5 * d4;
    let mut d4m3: libc::c_float = d4 - d3;
    let mut d5m6: libc::c_float = d5 - d6;
    if va <= 0.0f32 && d4m3 >= 0.0f32 && d5m6 >= 0.0f32 {
        let mut w_0: libc::c_float = d4m3 / (d4m3 + d5m6);
        let mut bc: Vec3f = Vec3f_Sub(c, b);
        return Vec3f_Add(b, Vec3f_Muls(bc, w_0));
    }
    let mut denom: libc::c_float = 1.0f32 / (va + vb + vc);
    let mut v_0: libc::c_float = vb * denom;
    let mut w_1: libc::c_float = vc * denom;
    return Vec3f_Add(Vec3f_Add(a, Vec3f_Muls(ab, v_0)), Vec3f_Muls(ac, w_1));
}
#[no_mangle]
pub unsafe extern "C" fn Intersect_SphereTriangle(
    mut sphere: *const Sphere,
    mut triangle: *const Triangle,
    mut pHit: *mut Vec3f,
) -> bool {
    let mut pClosest: Vec3f = ClosestPoint_PointToTriangle(&(*sphere).p, triangle);
    let mut distSq: libc::c_float = Vec3f_DistanceSquared((*sphere).p, pClosest);
    if distSq < (*sphere).r * (*sphere).r {
        *pHit = pClosest;
        return 1 as libc::c_int != 0;
    }
    return 0 as libc::c_int != 0;
}
