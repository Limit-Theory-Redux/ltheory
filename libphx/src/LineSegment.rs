use ::libc;
extern "C" {
    fn snprintf(
        _: *mut libc::c_char,
        _: libc::c_ulong,
        _: *const libc::c_char,
        _: ...
    ) -> libc::c_int;
    fn Ray_ToLineSegment(_: *const Ray, _: *mut LineSegment);
}
pub type __darwin_size_t = libc::c_ulong;
pub type size_t = __darwin_size_t;
pub type cstr = *const libc::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct LineSegment {
    pub p0: Vec3f,
    pub p1: Vec3f,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec3f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Ray {
    pub p: Vec3f,
    pub dir: Vec3f,
    pub tMin: libc::c_float,
    pub tMax: libc::c_float,
}
#[inline]
unsafe extern "C" fn Vec3f_ToString(mut v: *mut Vec3f) -> cstr {
    static mut buffer: [libc::c_char; 512] = [0; 512];
    snprintf(
        buffer.as_mut_ptr(),
        (::core::mem::size_of::<[libc::c_char; 512]>() as libc::c_ulong)
            .wrapping_div(::core::mem::size_of::<libc::c_char>() as libc::c_ulong)
            as libc::c_int as size_t,
        b"(%.4f, %.4f, %.4f)\0" as *const u8 as *const libc::c_char,
        (*v).x as libc::c_double,
        (*v).y as libc::c_double,
        (*v).z as libc::c_double,
    );
    return buffer.as_mut_ptr() as cstr;
}
#[inline]
unsafe extern "C" fn Vec3f_Sub(mut a: Vec3f, mut b: Vec3f) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: a.x - b.x,
            y: a.y - b.y,
            z: a.z - b.z,
        };
        init
    };
    return self_0;
}
#[no_mangle]
pub unsafe extern "C" fn LineSegment_ToRay(
    mut self_0: *const LineSegment,
    mut out: *mut Ray,
) {
    (*out).p = (*self_0).p0;
    (*out).dir = Vec3f_Sub((*self_0).p1, (*self_0).p0);
    (*out).tMin = 0.0f32;
    (*out).tMax = 1.0f32;
}
#[no_mangle]
pub unsafe extern "C" fn LineSegment_FromRay(
    mut ray: *const Ray,
    mut out: *mut LineSegment,
) {
    Ray_ToLineSegment(ray, out);
}
#[no_mangle]
pub unsafe extern "C" fn LineSegment_ToString(mut self_0: *mut LineSegment) -> cstr {
    static mut buffer: [libc::c_char; 512] = [0; 512];
    snprintf(
        buffer.as_mut_ptr(),
        (::core::mem::size_of::<[libc::c_char; 512]>() as libc::c_ulong)
            .wrapping_div(::core::mem::size_of::<libc::c_char>() as libc::c_ulong)
            as libc::c_int as size_t,
        b"p0:%s p1:%s\0" as *const u8 as *const libc::c_char,
        Vec3f_ToString(&mut (*self_0).p0),
        Vec3f_ToString(&mut (*self_0).p1),
    );
    return buffer.as_mut_ptr() as cstr;
}
