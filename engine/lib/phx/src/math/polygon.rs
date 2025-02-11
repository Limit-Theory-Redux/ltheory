#![allow(unsafe_code)] // TODO: remove

use super::*;
use crate::error::Error;

#[derive(Clone)]
#[repr(C)]
pub struct Polygon {
    pub vertices: Vec<Vec3>,
}

#[no_mangle]
pub unsafe extern "C" fn Polygon_ToPlane(polygon: *const Polygon, out: *mut Plane) {
    let v: &Vec<Vec3> = &(*polygon).vertices;
    let mut n: DVec3 = DVec3::ZERO;
    let mut centroid = DVec3::ZERO;

    let v_cur_as_f32 = v[v.len() - 1];
    let mut v_cur = DVec3::new(
        v_cur_as_f32.x as f64,
        v_cur_as_f32.y as f64,
        v_cur_as_f32.z as f64,
    );
    let mut i: usize = 0;
    while i < v.len() {
        let v_prev: DVec3 = v_cur;
        let v_cur_as_f32 = v[i];
        v_cur = DVec3::new(
            v_cur_as_f32.x as f64,
            v_cur_as_f32.y as f64,
            v_cur_as_f32.z as f64,
        );

        n.x += (v_prev.y - v_cur.y) * (v_prev.z + v_cur.z);
        n.y += (v_prev.z - v_cur.z) * (v_prev.x + v_cur.x);
        n.z += (v_prev.x - v_cur.x) * (v_prev.y + v_cur.y);
        centroid += v_cur;
        i += 1;
    }
    n = n.normalize();
    centroid /= v.len() as f64;

    (*out).n = Vec3::new(n.x as f32, n.y as f32, n.z as f32);
    (*out).d = DVec3::dot(centroid, n) as f32;

    // CHECK2(Assert(PointsInPlane(out, polygon)));
}

#[no_mangle]
pub unsafe extern "C" fn Polygon_ToPlaneFast(polygon: *const Polygon, out: *mut Plane) {
    // NOTE: Doesn't normalize n and uses v[0] as the center.

    let v: &Vec<Vec3> = &(*polygon).vertices;

    let mut n: Vec3 = Vec3::new(0.0f32, 0., 0.);
    let mut i: usize = v.len() - 1;
    let mut j: usize = 0;
    while j < v.len() {
        n.x += (v[i].y - v[j].y) * (v[i].z + v[j].z);
        n.y += (v[i].z - v[j].z) * (v[i].x + v[j].x);
        n.z += (v[i].x - v[j].x) * (v[i].y + v[j].y);
        i = j;
        j += 1;
    }

    (*out).n = n;
    (*out).d = Vec3::dot(v[0], n);

    // CHECK2(Assert(PointsInPlane(out, polygon)));
}

#[allow(non_snake_case)] // TODO: remove this and fix all warnings
#[inline]
unsafe extern "C" fn Polygon_SplitImpl(
    polygon: *const Polygon,
    split_plane: Plane,
    back: *mut Polygon,
    front: *mut Polygon,
) {
    if (*polygon).vertices.is_empty() {
        return;
    }

    let mut a: Vec3 = *(*polygon).vertices.last().unwrap();
    let mut a_side = Plane_ClassifyPoint(&split_plane, &a);
    for j in 0..((*polygon).vertices.len() as i32) {
        let b: Vec3 = (*polygon).vertices[j as usize];
        let b_side = Plane_ClassifyPoint(&split_plane, &b);

        if b_side == PointClassification::InFront {
            if a_side == PointClassification::Behind {
                let i = Vec3::ZERO;
                // let _lineSegment: LineSegment = LineSegment { p0: b, p1: a };
                (*front).vertices.push(i);
                (*back).vertices.push(i);

                // let hit: bool = Intersect_LineSegmentPlane(&mut lineSegment, &splitPlane, &mut i);
                // Assert(hit); UNUSED(hit);
                // Assert(Plane_ClassifyPoint(&splitPlane, &i) == PointClassification_Coplanar);
            }
            (*front).vertices.push(b)
        } else if b_side == PointClassification::Behind {
            if a_side == PointClassification::InFront {
                let i = Vec3::ZERO;
                // let _lineSegment: LineSegment = LineSegment { p0: a, p1: b };
                (*front).vertices.push(i);
                (*back).vertices.push(i);

                // let hit: bool = Intersect_LineSegmentPlane(&mut lineSegment, &splitPlane, &mut i);
                // Assert(hit); UNUSED(hit);
                // Assert(Plane_ClassifyPoint(&splitPlane, &i) == PointClassification_Coplanar);
            } else if a_side == PointClassification::Coplanar {
                (*back).vertices.push(a);
            }
            (*back).vertices.push(b);
        } else {
            if a_side == PointClassification::Behind {
                (*back).vertices.push(b);
            }
            (*front).vertices.push(b);
        }

        a = b;
        a_side = b_side;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Polygon_SplitSafe(
    polygon: *const Polygon,
    split_plane: Plane,
    back: *mut Polygon,
    front: *mut Polygon,
) {
    Polygon_SplitImpl(polygon, split_plane, back, front);

    let polygons: [*mut Polygon; 2] = [front, back];
    let mut i: i32 = 0;
    while i < polygons.len() as i32 {
        let polygon_part: *mut Polygon = polygons[i as usize];
        let v: &Vec<Vec3> = &(*polygon_part).vertices;

        let mut v_cur: Vec3 = v[v.len() - 1];
        let mut l: usize = 0;
        while l < v.len() {
            let v_prev: Vec3 = v_cur;
            v_cur = v[l];

            let edge_len: f32 = v_cur.distance(v_prev);
            if (edge_len as f64) < 0.75f64 * 1e-4f64 {
                (*back).vertices.clear();
                (*front).vertices.clear();
                for vertex in (*polygon).vertices.iter() {
                    (*back).vertices.push(*vertex);
                    (*front).vertices.push(*vertex);
                }
                return;
            }
            l += 1;
        }
        i += 1;
    }
}

#[no_mangle]
pub unsafe extern "C" fn Polygon_Split(
    polygon: *mut Polygon,
    split_plane: Plane,
    back: *mut Polygon,
    front: *mut Polygon,
) {
    Polygon_SplitImpl(polygon, split_plane, back, front);
}

#[no_mangle]
pub unsafe extern "C" fn Polygon_GetCentroid(polygon: *mut Polygon, out: *mut Vec3) {
    let mut centroid = Vec3::ZERO;

    for v in (*polygon).vertices.iter() {
        centroid += *v;
    }
    centroid /= (*polygon).vertices.len() as f32;

    *out = centroid;
}

#[allow(non_snake_case)] // TODO: remove this and fix all warnings
pub fn Polygon_ConvexToTriangles(polygon: &Polygon, triangles: &mut Vec<Triangle>) {
    let v = &(*polygon).vertices;
    for i in 1..(v.len() - 1) {
        triangles.push(Triangle {
            vertices: [v[0], v[i], v[i + 1]],
        });
    }
}

#[no_mangle]
pub unsafe extern "C" fn Polygon_Validate(polygon: *mut Polygon) -> Error {
    let v: &Vec<Vec3> = &(*polygon).vertices;

    let mut v_cur: Vec3 = v[v.len() - 1];
    let mut i: usize = 0;
    while i < v.len() {
        let v_prev: Vec3 = v_cur;
        v_cur = v[i];

        // NaN or Inf
        let e = Vec3_Validate(v_cur);
        if e != 0 {
            return 0x400000 | e;
        }

        // Degenerate
        let mut j: usize = i + 1;
        while j < v.len() {
            if v_cur == v[j] {
                return (0x400000 | 0x40) as Error;
            }
            j += 1;
        }

        // Sliver
        /* TODO : See comment on slivers in Triangle_Validate */
        let edge_len = v_cur.distance(v_prev);
        if (edge_len as f64) < 0.75f64 * 1e-4f64 {
            return (0x400000 | 0x8) as Error;
        }
        i += 1;
    }
    0 as Error
}
