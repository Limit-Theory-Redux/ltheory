use crate::math::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct LineSegment {
    pub p0: Position,
    pub p1: Position,
}

#[luajit_ffi_gen::luajit_ffi(
    clone = true,
    typedef = "
        double p0x;
        double p0y;
        double p0z;
        double p1x;
        double p1y;
        double p1z;"
)]
impl LineSegment {
    pub fn to_ray(&self) -> Ray {
        Ray {
            p: self.p0,
            dir: self.p1.as_dvec3() - self.p0.as_dvec3(),
            t_min: 0.0,
            t_max: 1.0,
        }
    }

    pub fn from_ray(ray: &Ray) -> Self {
        ray.to_line_segment()
    }

    #[bind(role = "to_string")]
    pub fn get_string(&self) -> String {
        format!("p0:{} p1:{}", self.p0, self.p1)
    }
}
