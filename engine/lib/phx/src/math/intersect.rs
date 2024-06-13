use super::*;

/* --- Intersect ---------------------------------------------------------------
 *
 *   Intersect_RectRect     : Rect format is {x, y, sx, sy}
 *   Intersect_RectRectFast : Only works with non-inverted rects (sx, sy > 0)
 *
 * -------------------------------------------------------------------------- */

pub const PLANE_THICKNESS_EPSILON: f32 = 1e-4;
pub const POINT_INTERSECTION_EPSILON: f32 = 2.0f32 * PLANE_THICKNESS_EPSILON;
pub const RAY_INTERSECTION_EPSILON: f32 = 8.0f32 * PLANE_THICKNESS_EPSILON;
pub const SPHERE_INTERSECTION_EPSILON: f32 = 2.0f32 * PLANE_THICKNESS_EPSILON;

/* NOTE: On Epsilons
 *  - PLANE_THICKNESS_EPSILON
 *
 *    1) Limited by floating point precision.
 *    Before building a BSP we normalize the mesh by dividing by the bounds
 *    size such that vertex positions are between [-1, 1] on all axes. When
 *    choosing splitting planes we call Polygon_ToPlane for autopartitioning
 *    cuts. As the vertices and center of the triangle can be anywhere in that
 *    [-1, 1] range, our precision is capped around 1e-7. Even if all inputs
 *    are double precision, simply doing the vertex-plane distance calculation
 *    in float will end up rounding the distance to somewhere around 1e-8 in
 *    many cases.
 *
 *    2) Limited by overall BSP size (in bytes)
 *    The smaller the epsilon is, the more eagerly we end up cutting polygons.
 *    In order to make a cut, an edge must cross from 'in front' of the plane
 *    to 'behind' the plane. The thinner the plane is, the easier it is to
 *    satisfy this condition and we end up making a lot more cuts, which leads
 *    to more triangles, which leads to more memory usage. At last checl (r631)
 *    Dropping from 1e-4 to 1e-5 raised the overall memory usage by 25% while
 *    only gaining 0.1 us.
 */

/* TODO : Need to handle epsilons properly in these intersection tests */

#[no_mangle]
pub unsafe extern "C" fn Intersect_PointBox(src: &mut Matrix, dst: &mut Matrix) -> bool {
    let inv = Matrix_Inverse(dst);
    let mut srcPt = Vec3::ZERO;
    Matrix_GetPos(src, &mut srcPt);
    let mut dstPt = Vec3::ZERO;
    Matrix_MulPoint(inv.as_ref(), &mut dstPt, srcPt.x, srcPt.y, srcPt.z);
    -1.0f32 < dstPt.x
        && dstPt.x < 1.0f32
        && -1.0f32 < dstPt.y
        && dstPt.y < 1.0f32
        && -1.0f32 < dstPt.z
        && dstPt.z < 1.0f32
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_PointTriangle_Barycentric(p: &Vec3, tri: &Triangle) -> bool {
    let v: &[Vec3; 3] = &tri.vertices;

    let pv0: Vec3 = v[0] - *p;
    let pv1: Vec3 = v[1] - *p;
    let pv2: Vec3 = v[2] - *p;

    /* NOTE: Any scale factor on plane.n will fall out of the calcuations for A and B. */
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

    /* TODO : Need a proper epsilon */
    let fuzzyMin: f32 = 0.0f32 - 0.01f32;
    let fuzzyMax: f32 = 1.0f32 + 0.01f32;

    A > fuzzyMin && A < fuzzyMax && B > fuzzyMin && B < fuzzyMax && C > fuzzyMin && C < fuzzyMax
}

#[no_mangle]
pub extern "C" fn Intersect_RayPlane(ray: &Ray, plane: &Plane, pHit: &mut Position) -> bool {
    /* TODO : Shouldn't we handle denom == 0? */
    let dist: f64 = (*plane).d as f64 - DVec3::dot((*plane).n.as_dvec3(), ray.p.v);
    let denom: f64 = DVec3::dot((*plane).n.as_dvec3(), ray.dir);
    let t: f64 = dist / denom;

    if t >= ray.tMin && t <= ray.tMax {
        *pHit = Position::from_dvec(ray.p.v + ray.dir * t);
        true
    } else {
        false
    }
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_RayTriangle_Barycentric(
    ray: &Ray,
    tri: &Triangle,
    tEpsilon: f32,
    tHit: *mut f32,
) -> bool {
    /* NOTE: Any scale factor on plane.n falls out of the calculation for t.
     * denom and dist are both off by the scale factor, but we don't need
     * them for anything other than t.
     */
    let mut plane: Plane = Plane {
        n: Vec3::ZERO,
        d: 0.,
    };
    Triangle_ToPlaneFast(tri, &mut plane);

    let dist: f32 = Vec3::dot(plane.n, ray.p.as_vec3()) - plane.d;
    let denom: f32 = -Vec3::dot(plane.n, ray.dir.as_vec3());

    if denom != 0.0f32 {
        let t: f32 = dist / denom;
        if t > ray.tMin as f32 - tEpsilon && t < ray.tMax as f32 + tEpsilon {
            let v: &[Vec3; 3] = &tri.vertices;
            let mut pp = Position::ZERO;
            Ray_GetPoint(ray, t as f64, &mut pp);
            let p = pp.as_vec3();

            let pv0: Vec3 = v[0] - p;
            let pv1: Vec3 = v[1] - p;
            let pv2: Vec3 = v[2] - p;

            let areaABC: f32 = Vec3::dot(plane.n, plane.n);
            let areaPBC: f32 = Vec3::dot(plane.n, Vec3::cross(pv1, pv2));
            let areaPCA: f32 = Vec3::dot(plane.n, Vec3::cross(pv2, pv0));

            let A: f32 = areaPBC / areaABC;
            let B: f32 = areaPCA / areaABC;
            let C: f32 = 1.0f32 - A - B;

            /* TODO : Need a proper epsilon */
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
        } else {
            /* TODO : Handle parallel but in triangle (or its thick plane) */
        }
    }
    false
}

/* http://fileadmin.cs.lth.se/cs/Personal/Tomas_Akenine-Moller/raytri/ */
#[no_mangle]
pub unsafe extern "C" fn Intersect_RayTriangle_Moller1(
    ray: &Ray,
    tri: &Triangle,
    tHit: *mut f32,
) -> bool {
    /* Rewritten test sign of determinant; division is at the end. */
    let vt: &[Vec3; 3] = &tri.vertices;
    let edge1: Vec3 = vt[1] - vt[0];
    let edge2: Vec3 = vt[2] - vt[0];

    /* Begin calculating determinant - also used to calculate U parameter. */
    let pvec: Vec3 = Vec3::cross(ray.dir.as_vec3(), edge2);

    /* TODO : Need a proper epsilon */
    let epsilon: f32 = 0.000001f32;

    /* If determinant is near zero, ray lies in plane of triangle. */
    let det: f32 = Vec3::dot(edge1, pvec);
    let qvec = if det > epsilon {
        /* Calculate distance from vert0 to ray origin. */
        let tvec: Vec3 = ray.p.as_vec3() - vt[0];

        /* Calculate U parameter and test bounds. */
        let u = Vec3::dot(tvec, pvec);
        if (u as f64) < 0.0f64 || u > det {
            return false;
        }

        /* Prepare to test V parameter. */
        let qvec = Vec3::cross(tvec, edge1);

        /* Calculate V parameter and test bounds. */
        let v = Vec3::dot(ray.dir.as_vec3(), qvec);

        if (v as f64) < 0.0f64 || u + v > det {
            return false;
        }

        qvec
    } else if det < -epsilon {
        /* Calculate distance from vert0 to ray origin. */
        let tvec: Vec3 = ray.p.as_vec3() - vt[0];

        /* Calculate U parameter and test bounds. */
        let u = Vec3::dot(tvec, pvec);
        if u as f64 > 0.0f64 || u < det {
            return false;
        }

        /* Prepare to test V parameter. */
        let qvec = Vec3::cross(tvec, edge1);

        /* Calculate V parameter and test bounds. */
        let v = Vec3::dot(ray.dir.as_vec3(), qvec);

        if v as f64 > 0.0f64 || u + v < det {
            return false;
        }

        qvec
    } else {
        /* Ray is parallel to the plane of the triangle */
        return false;
    };

    let inv_det: f32 = 1.0f32 / det;

    /* Ray intersects; calculate t. */
    *tHit = Vec3::dot(edge2, qvec) * inv_det;
    true
}

/* http://www.cs.virginia.edu/~gfx/courses/2003/ImageSynthesis/papers/Acceleration/Fast%20MinimumStorage%20RayTriangle%20Intersection.pdf */
#[no_mangle]
pub unsafe extern "C" fn Intersect_RayTriangle_Moller2(
    ray: &Ray,
    tri: &Triangle,
    tHit: *mut f32,
) -> bool {
    let vt: &[Vec3; 3] = &tri.vertices;

    /* Find vectors for two edges sharing vert. */
    let edge1: Vec3 = vt[1] - vt[0];
    let edge2: Vec3 = vt[2] - vt[0];

    /* Begin calculating determinant - also used to calculate U parameter. */
    let pvec: Vec3 = Vec3::cross(ray.dir.as_vec3(), edge2);

    /* If determinant is near zero ray lies in plane of triangle. */
    let det: f32 = Vec3::dot(edge1, pvec);

    /* TODO : Need a proper epsilon */
    if f32::abs(det) < 0.000001f32 {
        return false;
    }

    let inv_det: f32 = 1.0f32 / det;

    /* Calculate distance from vert to ray origin. */
    let tvec: Vec3 = ray.p.as_vec3() - vt[0];

    /* TODO : Need a proper epsilon */
    let fuzzyMin: f32 = 0.0f32 - 0.01f32;
    let fuzzyMax: f32 = 1.0f32 + 0.01f32;

    /* Calculate U and test bounds. */
    let u: f32 = Vec3::dot(tvec, pvec) * inv_det;
    if u < fuzzyMin || u > fuzzyMax {
        return false;
    }

    /* Prepare to test V. */
    let qvec: Vec3 = Vec3::cross(tvec, edge1);

    /* Calculate V and test bounds. */
    let v: f32 = Vec3::dot(ray.dir.as_vec3(), qvec) * inv_det;
    if v < fuzzyMin || u + v > fuzzyMax {
        return false;
    }

    /* Ray intersects; calculate t. */
    *tHit = Vec3::dot(edge2, qvec) * inv_det;
    true
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_LineSegmentPlane(
    lineSegment: *const LineSegment,
    plane: &Plane,
    pHit: &mut Position,
) -> bool {
    let mut ray: Ray = Ray {
        p: (*lineSegment).p0,
        dir: (*lineSegment).p1.as_dvec3() - (*lineSegment).p0.as_dvec3(),
        tMin: 0.0,
        tMax: 1.0,
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
    result = result && (*a).x < ((*b).x + (*b).z);
    result = result && (*b).x < ((*a).x + (*a).z);
    result = result && (*a).y < ((*b).y + (*b).w);
    result = result && (*b).y < ((*a).y + (*a).w);
    result
}

/* Realtime Collision Detection, pp 141-142 */
#[inline]
unsafe fn ClosestPoint_PointToTriangle(p: *const Vec3, tri: &Triangle) -> Vec3 {
    let a: Vec3 = tri.vertices[0];
    let b: Vec3 = tri.vertices[1];
    let c: Vec3 = tri.vertices[2];

    /* Check if P in vertex region outside A */
    let ab: Vec3 = b - a;
    let ac: Vec3 = c - a;
    let ap: Vec3 = *p - a;
    let d1: f32 = Vec3::dot(ab, ap);
    let d2: f32 = Vec3::dot(ac, ap);
    if d1 <= 0.0f32 && d2 <= 0.0f32 {
        return a; // (1, 0, 0)
    }

    /* Check if P in vertex region outside B. */
    let bp: Vec3 = *p - b;
    let d3: f32 = Vec3::dot(ab, bp);
    let d4: f32 = Vec3::dot(ac, bp);
    if d3 >= 0.0f32 && d4 <= d3 {
        return b; // (0, 1, 0)
    }

    /* Check if P in edge region of AB, if so return projection of P onto AB. */
    let vc: f32 = d1 * d4 - d3 * d2;
    if vc <= 0.0f32 && d1 >= 0.0f32 && d3 <= 0.0f32 {
        let v: f32 = d1 / (d1 - d3);
        return a + ab * v; // (1 - v, v, 0)
    }

    /* Check if P in vertex region outside C. */
    let cp: Vec3 = *p - c;
    let d5: f32 = Vec3::dot(ab, cp);
    let d6: f32 = Vec3::dot(ac, cp);
    if d6 >= 0.0f32 && d5 <= d6 {
        return c; // (0, 0, 1)
    }

    /* Check if P in edge region of AC, if so return projection of P onto AC. */
    let vb: f32 = d5 * d2 - d1 * d6;
    if vb <= 0.0f32 && d2 >= 0.0f32 && d6 <= 0.0f32 {
        let w: f32 = d2 / (d2 - d6);
        return a + ac * w; // (1 - w, 0, w)
    }

    /* Check if P in edge region of BC, if so return projection of P onto BC. */
    let va: f32 = d3 * d6 - d5 * d4;
    let d4m3: f32 = d4 - d3;
    let d5m6: f32 = d5 - d6;
    if va <= 0.0f32 && d4m3 >= 0.0f32 && d5m6 >= 0.0f32 {
        let w: f32 = d4m3 / (d4m3 + d5m6);
        let bc: Vec3 = c - b;
        return b + bc * w; // (0, 1 - w, w)
    }

    /* P inside face region. Compute barycentric coordinates (1 - v - w, v, w) */
    let denom: f32 = 1.0f32 / (va + vb + vc);
    let v: f32 = vb * denom;
    let w: f32 = vc * denom;
    a + ab * v + ac * w
}

#[no_mangle]
pub unsafe extern "C" fn Intersect_SphereTriangle(
    sphere: &Sphere,
    triangle: &Triangle,
    pHit: &mut Vec3,
) -> bool {
    let pClosest: Vec3 = ClosestPoint_PointToTriangle(&sphere.p, triangle);
    let distSq: f32 = sphere.p.distance_squared(pClosest);
    if distSq < sphere.r * sphere.r {
        *pHit = pClosest;
        return true;
    }
    false
}

// /* TODO : This is not yet working properly
//  * TODO : Need to precompute index of the largest normal component */
// bool Intersect_RayTriangle_Badouel (Ray* ray, Triangle* triangle, float tEpsilon, float* tHit) {
//   /* NOTE: There's some great information in the following reference, including
//    * comparisons of multiple algorithms and code for each.
//    * http://erich.realtimerendering.com/ptinpoly/
//    */
//   Plane plane = Triangle_ToPlaneFast(triangle);

//   float dist = Vec3f_Dot(plane.n, ray->p) - plane.d;
//   float denom = -Vec3f_Dot(plane.n, ray->dir);

//   if (denom != 0.0f) {
//     float t =  dist / denom;

//     if (t > ray->tMin - tEpsilon && t < ray->tMax + tEpsilon) {
//       Vec3f* pgon = triangle->vertices;
//       Vec3f point = Ray_GetPoint(ray, t);

//       Vec3f pg1, pg2;
//       float tx, ty, u0, u1, u2, v0, v1, vx0, vy0, alpha, beta, denom2;
//       int inside_flag;

//       tx = point.x;
//       ty = point.y;
//       vx0 = pgon[0].x;
//       vy0 = pgon[0].y;
//       u0 = tx - vx0;
//       v0 = ty - vy0;

//       inside_flag = 0;

//       /* TODO : v1 is uninitialized in the first if */
//       v1 = 0;

//       pg1 = pgon[1];
//       pg2 = pgon[2];
//       u1 = pg1.x - vx0;
//       if (u1 == 0.0) {
//         /* 0 and 1 vertices have same X value */
//         u2 = pg2.x - vx0;
//         if (
//           /* compute beta and check bounds */
//           ((beta = u0 / u2) < -0.01f) || (beta > 1.01f) ||

//           /* compute alpha and check bounds */
//           ((alpha = (v0 - beta * (pg2.y - vy0)) / v1) < 0.0)) {

//           /* whew! missed! */
//           return inside_flag != 0;
//         }
//       }
//       else {
//         /* 0 and 1 vertices have different X value */
//         /* compute denom2 */
//         u2 = pg2.x - vx0;
//         v1 = pg1.y - vy0;
//         denom2 = (pg2.y - vy0) * u1 - u2 * v1;
//         if (
//           /* compute beta and check bounds */
//           ((beta = (v0 * u1 - u0 * v1) / denom2) < -0.01f) || (beta > 1.01f) ||

//           /* compute alpha & check bounds */
//           ((alpha = (u0 - beta * u2) / u1) < 0.0)) {

//           /* whew! missed! */
//           return inside_flag != 0;
//         }
//       }

//       /* check gamma */
//       if (alpha + beta <= 1.01f) {
//         /* survived */
//         inside_flag = !inside_flag;
//       }

//       return inside_flag != 0;
//     }
//   }
//   else {
//     /* TODO : Handle parallel but in triangle (or its thick plane) */
//   }

//   /* TODO : Coalesce returns? */
//   return false;
// }
