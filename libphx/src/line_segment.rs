use crate::math::Vec3;
use crate::{ray::*, Convert};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct LineSegment {
    pub p0: Vec3,
    pub p1: Vec3,
}

impl LineSegment {
    pub fn to_string(&self) -> String {
        format!("p0:{} p1:{}", self.p0.to_string(), self.p1.to_string(),)
    }
}

#[no_mangle]
pub extern "C" fn LineSegment_ToRay(this: &LineSegment, out: &mut Ray) {
    out.p = this.p0;
    out.dir = this.p1 - this.p0;
    out.tMin = 0.0f32;
    out.tMax = 1.0f32;
}

#[no_mangle]
pub extern "C" fn LineSegment_FromRay(ray: &Ray, out: &mut LineSegment) {
    Ray_ToLineSegment(ray, out);
}

#[no_mangle]
pub extern "C" fn LineSegment_ToString(this: &mut LineSegment) -> *const libc::c_char {
    this.to_string().convert()
}
