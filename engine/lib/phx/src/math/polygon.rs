use super::*;
use crate::error::Error;

#[derive(Clone)]
#[repr(C)]
pub struct Polygon {
    pub vertices: Vec<Vec3>,
}

#[luajit_ffi_gen::luajit_ffi(
    clone = true,
    typedef = "
        int32         vertices_size;
        int32         vertices_capacity;
        struct Vec3f* vertices_data;"
)]
impl Polygon {
    pub fn to_plane(&self) -> Plane {
        let mut v_cur = self.vertices[self.vertices.len() - 1].as_dvec3();

        let mut n = DVec3::ZERO;
        let mut centroid = DVec3::ZERO;
        for v_cur_as_f32 in &self.vertices {
            let v_prev = v_cur;
            v_cur = v_cur_as_f32.as_dvec3();

            n.x += (v_prev.y - v_cur.y) * (v_prev.z + v_cur.z);
            n.y += (v_prev.z - v_cur.z) * (v_prev.x + v_cur.x);
            n.z += (v_prev.x - v_cur.x) * (v_prev.y + v_cur.y);
            centroid += v_cur;
        }
        n = n.normalize();
        centroid /= self.vertices.len() as f64;

        Plane {
            n: n.as_vec3(),
            d: centroid.dot(n) as f32,
        }

        // CHECK2(Assert(PointsInPlane(out, polygon)));
    }

    pub fn to_plane_fast(&self) -> Plane {
        // NOTE: Doesn't normalize n and uses v[0] as the center.

        let v = &self.vertices;

        let mut n = Vec3::ZERO;
        let mut i = v.len() - 1;
        let mut j = 0;
        while j < v.len() {
            n.x += (v[i].y - v[j].y) * (v[i].z + v[j].z);
            n.y += (v[i].z - v[j].z) * (v[i].x + v[j].x);
            n.z += (v[i].x - v[j].x) * (v[i].y + v[j].y);
            i = j;
            j += 1;
        }

        Plane { n, d: v[0].dot(n) }

        // CHECK2(Assert(PointsInPlane(out, polygon)));
    }

    pub fn split_safe(&self, split_plane: &Plane, back: &mut Polygon, front: &mut Polygon) {
        self.split_impl(split_plane, back, front);

        let mut split = false;
        let polygons = [&*front, &*back];
        'exit: for polygon_part in polygons {
            let mut v_prev = polygon_part.vertices[polygon_part.vertices.len() - 1];
            for v_cur in &polygon_part.vertices {
                let edge_len = v_cur.distance(v_prev);
                if edge_len < 0.75 * 1e-4 {
                    split = true;
                    break 'exit;
                }
                v_prev = *v_cur;
            }
        }

        if split {
            front.vertices.clear();
            back.vertices.clear();
            for vertex in &self.vertices {
                back.vertices.push(*vertex);
                front.vertices.push(*vertex);
            }
        }
    }

    pub fn split(&mut self, split_plane: &Plane, back: &mut Polygon, front: &mut Polygon) {
        self.split_impl(split_plane, back, front);
    }

    pub fn get_centroid(&mut self) -> Vec3 {
        let mut centroid = Vec3::ZERO;

        for v in &self.vertices {
            centroid += *v;
        }
        centroid /= self.vertices.len() as f32;

        centroid
    }

    pub fn validate(&mut self) -> Error {
        let v = &self.vertices;
        let mut v_prev = v[v.len() - 1];
        for (i, v_cur) in v.iter().enumerate() {
            // NaN or Inf
            let e = validate_vec3(*v_cur);
            if e != 0 {
                return 0x400000 | e;
            }

            // Degenerate
            let mut j = i + 1;
            while j < v.len() {
                if *v_cur == v[j] {
                    return (0x400000 | 0x40) as Error;
                }
                j += 1;
            }

            // Sliver
            /* TODO : See comment on slivers in Triangle_Validate */
            let edge_len = v_cur.distance(v_prev);
            if edge_len < 0.75 * 1e-4 {
                return (0x400000 | 0x8) as Error;
            }
            v_prev = *v_cur;
        }
        0 as Error
    }
}

impl Polygon {
    pub fn convex_to_triangles(&self) -> Vec<Triangle> {
        let mut triangles = vec![];
        let v = &self.vertices;
        for i in 1..(v.len() - 1) {
            triangles.push(Triangle {
                vertices: [v[0], v[i], v[i + 1]],
            });
        }
        triangles
    }

    #[inline]
    fn split_impl(&self, split_plane: &Plane, back: &mut Polygon, front: &mut Polygon) {
        if self.vertices.is_empty() {
            return;
        }

        let mut a = *self.vertices.last().unwrap();
        let mut a_side = split_plane.classify_point(&a);
        for b in &self.vertices {
            let b_side = split_plane.classify_point(b);

            if b_side == PointClassification::InFront {
                if a_side == PointClassification::Behind {
                    // let _lineSegment: LineSegment = LineSegment { p0: b, p1: a };
                    front.vertices.push(Vec3::ZERO);
                    back.vertices.push(Vec3::ZERO);

                    // let hit: bool = Intersect::line_segment_plane(&mut lineSegment, &splitPlane, &mut i);
                    // Assert(hit); UNUSED(hit);
                    // Assert(Plane_ClassifyPoint(&splitPlane, &i) == PointClassification_Coplanar);
                }
                front.vertices.push(*b)
            } else if b_side == PointClassification::Behind {
                if a_side == PointClassification::InFront {
                    // let _lineSegment: LineSegment = LineSegment { p0: a, p1: b };
                    front.vertices.push(Vec3::ZERO);
                    back.vertices.push(Vec3::ZERO);

                    // let hit: bool = Intersect::line_segment_plane(&mut lineSegment, &splitPlane, &mut i);
                    // Assert(hit); UNUSED(hit);
                    // Assert(Plane_ClassifyPoint(&splitPlane, &i) == PointClassification_Coplanar);
                } else if a_side == PointClassification::Coplanar {
                    back.vertices.push(a);
                }
                back.vertices.push(*b);
            } else {
                if a_side == PointClassification::Behind {
                    back.vertices.push(*b);
                }
                front.vertices.push(*b);
            }

            a = *b;
            a_side = b_side;
        }
    }
}
