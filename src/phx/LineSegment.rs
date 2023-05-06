use crate::phx::internal::ffi;

use crate::phx::Math::Vec3;
use crate::phx::Ray::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct LineSegment {
    pub p0: Vec3,
    pub p1: Vec3,
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
    ffi::StaticString!(format!(
        "p0:{} p1:{}",
        this.p0.to_string(),
        this.p1.to_string(),
    ))
}
