use super::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Ray {
    pub p: Position,
    pub dir: DVec3,
    pub t_min: f64,
    pub t_max: f64,
}

#[luajit_ffi_gen::luajit_ffi(
    clone = true,
    typedef = "
        double px;
        double py;
        double pz;
        double dirx;
        double diry;
        double dirz;
        double tMin;
        double tMax;"
)]
impl Ray {
    pub fn get_point(&self, t: f64) -> Position {
        Position::from_dvec(self.p.v + self.dir * t)
    }

    pub fn intersect_plane(&self, plane: &Plane, p_hit: &mut Position) -> bool {
        Intersect::ray_plane(self, plane, p_hit)
    }

    pub fn intersect_triangle_barycentric(
        &self,
        tri: &Triangle,
        t_epsilon: f32,
        t_hit: &mut f32,
    ) -> bool {
        Intersect::ray_triangle_barycentric(self, tri, t_epsilon, t_hit)
    }

    pub fn intersect_triangle_moller1(&self, tri: &Triangle, t_hit: &mut f32) -> bool {
        Intersect::ray_triangle_moller1(self, tri, t_hit)
    }

    pub fn intersect_triangle_moller2(&self, tri: &Triangle, t_hit: &mut f32) -> bool {
        Intersect::ray_triangle_moller2(self, tri, t_hit)
    }

    pub fn to_line_segment(&self) -> LineSegment {
        LineSegment {
            p0: self.get_point(self.t_min),
            p1: self.get_point(self.t_max),
        }
    }

    pub fn from_line_segment(line_segment: &LineSegment) -> Self {
        line_segment.to_ray()
    }
}
