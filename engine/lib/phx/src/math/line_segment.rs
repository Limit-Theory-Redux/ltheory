#![allow(unsafe_code)] // TODO: remove

use crate::math::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct LineSegment {
    pub p0: Position,
    pub p1: Position,
}

#[luajit_ffi_gen::luajit_ffi(clone = true, opaque = false)]
impl LineSegment {
    pub fn to_ray(&self, out: &mut Ray) {
        out.p = self.p0;
        out.dir = self.p1.as_dvec3() - self.p0.as_dvec3();
        out.t_min = 0.0;
        out.t_max = 1.0;
    }

    pub fn from_ray(ray: &Ray, out: &mut LineSegment) {
        unsafe { Ray_ToLineSegment(ray, out) };
    }

    #[bind(role = "to_string")]
    pub fn get_string(&self) -> String {
        format!("p0:{} p1:{}", self.p0, self.p1)
    }
}
