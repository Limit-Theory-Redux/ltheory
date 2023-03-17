use crate::internal::Memory::*;
use crate::Ray::*;
use glam::Vec3;
use libc;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct LineSegment {
    pub p0: Vec3,
    pub p1: Vec3,
}

#[no_mangle]
pub unsafe extern "C" fn LineSegment_ToRay(mut this: *const LineSegment, mut out: *mut Ray) {
    (*out).p = (*this).p0;
    (*out).dir = (*this).p1 - (*this).p0;
    (*out).tMin = 0.0f32;
    (*out).tMax = 1.0f32;
}

#[no_mangle]
pub unsafe extern "C" fn LineSegment_FromRay(mut ray: *const Ray, mut out: *mut LineSegment) {
    Ray_ToLineSegment(ray, out);
}

#[no_mangle]
pub unsafe extern "C" fn LineSegment_ToString(mut this: *mut LineSegment) -> *const libc::c_char {
    static mut buffer: [libc::c_char; 512] = [0; 512];
    libc::snprintf(
        buffer.as_mut_ptr(),
        (::core::mem::size_of::<[libc::c_char; 512]>())
            .wrapping_div(::core::mem::size_of::<libc::c_char>()) as i32 as usize,
        b"p0:%s p1:%s\0" as *const u8 as *const libc::c_char,
        (*this).p0.to_string().as_mut_ptr(),
        (*this).p1.to_string().as_mut_ptr(),
    );
    return buffer.as_mut_ptr() as *const libc::c_char;
}
