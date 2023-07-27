use crate::internal::*;
use crate::math::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct LineSegment {
    pub p0: Vec3,
    pub p1: Vec3,
}

#[luajit_ffi_gen::luajit_ffi(clone = true, opaque = false)]
impl LineSegment {
    pub fn to_ray(&self, out: &mut Ray) {
        out.p = self.p0;
        out.dir = self.p1 - self.p0;
        out.tMin = 0.0f32;
        out.tMax = 1.0f32;
    }

    pub fn from_ray(ray: &Ray, out: &mut LineSegment) {
        Ray_ToLineSegment(ray, out);
    }

    #[bind(role = "to_string")]
    pub fn to_string(&self) -> String {
        format!("p0:{} p1:{}", self.p0.to_string(), self.p1.to_string())
    }
}
