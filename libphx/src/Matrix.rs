use ::libc;
use crate::internal::Memory::*;
extern "C" {
    fn cos(_: libc::c_double) -> libc::c_double;
    fn sin(_: libc::c_double) -> libc::c_double;
    fn tan(_: libc::c_double) -> libc::c_double;
    fn fabs(_: libc::c_double) -> libc::c_double;
    fn sqrt(_: libc::c_double) -> libc::c_double;
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
    fn snprintf(
        _: *mut libc::c_char,
        _: libc::size_t,
        _: *const libc::c_char,
        _: ...
    ) -> libc::c_int;
    fn Quat_GetAxisX(_: *const Quat, _: *mut Vec3f);
    fn Quat_GetAxisY(_: *const Quat, _: *mut Vec3f);
    fn Quat_GetAxisZ(_: *const Quat, _: *mut Vec3f);
    fn Quat_FromBasis(x: *const Vec3f, y: *const Vec3f, z: *const Vec3f, _: *mut Quat);
}
pub type cstr = *const libc::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Box3f {
    pub lower: Vec3f,
    pub upper: Vec3f,
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
pub struct Matrix {
    pub m: [libc::c_float; 16],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Quat {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
    pub w: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec4f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
    pub w: libc::c_float,
}
#[inline]
unsafe extern "C" fn Abs(mut t: libc::c_double) -> libc::c_double {
    return fabs(t);
}
#[inline]
unsafe extern "C" fn Sqrtf(mut t: libc::c_float) -> libc::c_float {
    return sqrt(t as libc::c_double) as libc::c_float;
}
#[inline]
unsafe extern "C" fn Maxf(mut a: libc::c_float, mut b: libc::c_float) -> libc::c_float {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Minf(mut a: libc::c_float, mut b: libc::c_float) -> libc::c_float {
    return if a < b { a } else { b };
}
#[inline]
unsafe extern "C" fn Cos(mut t: libc::c_double) -> libc::c_double {
    return cos(t);
}
#[inline]
unsafe extern "C" fn Sin(mut t: libc::c_double) -> libc::c_double {
    return sin(t);
}
#[inline]
unsafe extern "C" fn Tan(mut t: libc::c_double) -> libc::c_double {
    return tan(t);
}
#[inline]
unsafe extern "C" fn Float_ApproximatelyEqual(
    mut x: libc::c_double,
    mut y: libc::c_double,
) -> bool {
    return Abs(x - y) < 1e-3f64;
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
#[inline]
unsafe extern "C" fn Vec3f_Muls(mut a: Vec3f, mut b: libc::c_float) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: a.x * b,
            y: a.y * b,
            z: a.z * b,
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Vec3f_Cross(mut a: Vec3f, mut b: Vec3f) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: b.z * a.y - b.y * a.z,
            y: b.x * a.z - b.z * a.x,
            z: b.y * a.x - b.x * a.y,
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Vec3f_Length(mut v: Vec3f) -> libc::c_float {
    return Sqrtf(v.x * v.x + v.y * v.y + v.z * v.z);
}
#[inline]
unsafe extern "C" fn Vec3f_Max(mut a: Vec3f, mut b: Vec3f) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: Maxf(a.x, b.x),
            y: Maxf(a.y, b.y),
            z: Maxf(a.z, b.z),
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Vec3f_Min(mut a: Vec3f, mut b: Vec3f) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: Minf(a.x, b.x),
            y: Minf(a.y, b.y),
            z: Minf(a.z, b.z),
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Vec3f_Normalize(mut v: Vec3f) -> Vec3f {
    let mut l: libc::c_float = Vec3f_Length(v);
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: v.x / l,
            y: v.y / l,
            z: v.z / l,
        };
        init
    };
    return self_0;
}

#[no_mangle]
pub unsafe extern "C" fn Matrix_Clone(mut self_0: *const Matrix) -> *mut Matrix {
    let mut clone: *mut Matrix = MemAlloc(
        ::core::mem::size_of::<Matrix>() as usize,
    ) as *mut Matrix;
    *clone = *self_0;
    return clone;
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_Free(mut self_0: *mut Matrix) {
    MemFree(self_0 as *const libc::c_void);
}
unsafe extern "C" fn Matrix_IOInverse(mut in_0: *const Matrix, mut out: *mut Matrix) {
    let mut src: *const libc::c_float = in_0 as *const libc::c_float;
    let mut dst: *mut libc::c_float = out as *mut libc::c_float;
    *dst
        .offset(
            0 as libc::c_int as isize,
        ) = *src.offset(5 as libc::c_int as isize)
        * *src.offset(10 as libc::c_int as isize)
        * *src.offset(15 as libc::c_int as isize)
        - *src.offset(5 as libc::c_int as isize)
            * *src.offset(11 as libc::c_int as isize)
            * *src.offset(14 as libc::c_int as isize)
        - *src.offset(9 as libc::c_int as isize) * *src.offset(6 as libc::c_int as isize)
            * *src.offset(15 as libc::c_int as isize)
        + *src.offset(9 as libc::c_int as isize) * *src.offset(7 as libc::c_int as isize)
            * *src.offset(14 as libc::c_int as isize)
        + *src.offset(13 as libc::c_int as isize)
            * *src.offset(6 as libc::c_int as isize)
            * *src.offset(11 as libc::c_int as isize)
        - *src.offset(13 as libc::c_int as isize)
            * *src.offset(7 as libc::c_int as isize)
            * *src.offset(10 as libc::c_int as isize);
    *dst
        .offset(
            4 as libc::c_int as isize,
        ) = -*src.offset(4 as libc::c_int as isize)
        * *src.offset(10 as libc::c_int as isize)
        * *src.offset(15 as libc::c_int as isize)
        + *src.offset(4 as libc::c_int as isize)
            * *src.offset(11 as libc::c_int as isize)
            * *src.offset(14 as libc::c_int as isize)
        + *src.offset(8 as libc::c_int as isize) * *src.offset(6 as libc::c_int as isize)
            * *src.offset(15 as libc::c_int as isize)
        - *src.offset(8 as libc::c_int as isize) * *src.offset(7 as libc::c_int as isize)
            * *src.offset(14 as libc::c_int as isize)
        - *src.offset(12 as libc::c_int as isize)
            * *src.offset(6 as libc::c_int as isize)
            * *src.offset(11 as libc::c_int as isize)
        + *src.offset(12 as libc::c_int as isize)
            * *src.offset(7 as libc::c_int as isize)
            * *src.offset(10 as libc::c_int as isize);
    *dst
        .offset(
            8 as libc::c_int as isize,
        ) = *src.offset(4 as libc::c_int as isize)
        * *src.offset(9 as libc::c_int as isize)
        * *src.offset(15 as libc::c_int as isize)
        - *src.offset(4 as libc::c_int as isize)
            * *src.offset(11 as libc::c_int as isize)
            * *src.offset(13 as libc::c_int as isize)
        - *src.offset(8 as libc::c_int as isize) * *src.offset(5 as libc::c_int as isize)
            * *src.offset(15 as libc::c_int as isize)
        + *src.offset(8 as libc::c_int as isize) * *src.offset(7 as libc::c_int as isize)
            * *src.offset(13 as libc::c_int as isize)
        + *src.offset(12 as libc::c_int as isize)
            * *src.offset(5 as libc::c_int as isize)
            * *src.offset(11 as libc::c_int as isize)
        - *src.offset(12 as libc::c_int as isize)
            * *src.offset(7 as libc::c_int as isize)
            * *src.offset(9 as libc::c_int as isize);
    *dst
        .offset(
            12 as libc::c_int as isize,
        ) = -*src.offset(4 as libc::c_int as isize)
        * *src.offset(9 as libc::c_int as isize)
        * *src.offset(14 as libc::c_int as isize)
        + *src.offset(4 as libc::c_int as isize)
            * *src.offset(10 as libc::c_int as isize)
            * *src.offset(13 as libc::c_int as isize)
        + *src.offset(8 as libc::c_int as isize) * *src.offset(5 as libc::c_int as isize)
            * *src.offset(14 as libc::c_int as isize)
        - *src.offset(8 as libc::c_int as isize) * *src.offset(6 as libc::c_int as isize)
            * *src.offset(13 as libc::c_int as isize)
        - *src.offset(12 as libc::c_int as isize)
            * *src.offset(5 as libc::c_int as isize)
            * *src.offset(10 as libc::c_int as isize)
        + *src.offset(12 as libc::c_int as isize)
            * *src.offset(6 as libc::c_int as isize)
            * *src.offset(9 as libc::c_int as isize);
    *dst
        .offset(
            1 as libc::c_int as isize,
        ) = -*src.offset(1 as libc::c_int as isize)
        * *src.offset(10 as libc::c_int as isize)
        * *src.offset(15 as libc::c_int as isize)
        + *src.offset(1 as libc::c_int as isize)
            * *src.offset(11 as libc::c_int as isize)
            * *src.offset(14 as libc::c_int as isize)
        + *src.offset(9 as libc::c_int as isize) * *src.offset(2 as libc::c_int as isize)
            * *src.offset(15 as libc::c_int as isize)
        - *src.offset(9 as libc::c_int as isize) * *src.offset(3 as libc::c_int as isize)
            * *src.offset(14 as libc::c_int as isize)
        - *src.offset(13 as libc::c_int as isize)
            * *src.offset(2 as libc::c_int as isize)
            * *src.offset(11 as libc::c_int as isize)
        + *src.offset(13 as libc::c_int as isize)
            * *src.offset(3 as libc::c_int as isize)
            * *src.offset(10 as libc::c_int as isize);
    *dst
        .offset(
            5 as libc::c_int as isize,
        ) = *src.offset(0 as libc::c_int as isize)
        * *src.offset(10 as libc::c_int as isize)
        * *src.offset(15 as libc::c_int as isize)
        - *src.offset(0 as libc::c_int as isize)
            * *src.offset(11 as libc::c_int as isize)
            * *src.offset(14 as libc::c_int as isize)
        - *src.offset(8 as libc::c_int as isize) * *src.offset(2 as libc::c_int as isize)
            * *src.offset(15 as libc::c_int as isize)
        + *src.offset(8 as libc::c_int as isize) * *src.offset(3 as libc::c_int as isize)
            * *src.offset(14 as libc::c_int as isize)
        + *src.offset(12 as libc::c_int as isize)
            * *src.offset(2 as libc::c_int as isize)
            * *src.offset(11 as libc::c_int as isize)
        - *src.offset(12 as libc::c_int as isize)
            * *src.offset(3 as libc::c_int as isize)
            * *src.offset(10 as libc::c_int as isize);
    *dst
        .offset(
            9 as libc::c_int as isize,
        ) = -*src.offset(0 as libc::c_int as isize)
        * *src.offset(9 as libc::c_int as isize)
        * *src.offset(15 as libc::c_int as isize)
        + *src.offset(0 as libc::c_int as isize)
            * *src.offset(11 as libc::c_int as isize)
            * *src.offset(13 as libc::c_int as isize)
        + *src.offset(8 as libc::c_int as isize) * *src.offset(1 as libc::c_int as isize)
            * *src.offset(15 as libc::c_int as isize)
        - *src.offset(8 as libc::c_int as isize) * *src.offset(3 as libc::c_int as isize)
            * *src.offset(13 as libc::c_int as isize)
        - *src.offset(12 as libc::c_int as isize)
            * *src.offset(1 as libc::c_int as isize)
            * *src.offset(11 as libc::c_int as isize)
        + *src.offset(12 as libc::c_int as isize)
            * *src.offset(3 as libc::c_int as isize)
            * *src.offset(9 as libc::c_int as isize);
    *dst
        .offset(
            13 as libc::c_int as isize,
        ) = *src.offset(0 as libc::c_int as isize)
        * *src.offset(9 as libc::c_int as isize)
        * *src.offset(14 as libc::c_int as isize)
        - *src.offset(0 as libc::c_int as isize)
            * *src.offset(10 as libc::c_int as isize)
            * *src.offset(13 as libc::c_int as isize)
        - *src.offset(8 as libc::c_int as isize) * *src.offset(1 as libc::c_int as isize)
            * *src.offset(14 as libc::c_int as isize)
        + *src.offset(8 as libc::c_int as isize) * *src.offset(2 as libc::c_int as isize)
            * *src.offset(13 as libc::c_int as isize)
        + *src.offset(12 as libc::c_int as isize)
            * *src.offset(1 as libc::c_int as isize)
            * *src.offset(10 as libc::c_int as isize)
        - *src.offset(12 as libc::c_int as isize)
            * *src.offset(2 as libc::c_int as isize)
            * *src.offset(9 as libc::c_int as isize);
    *dst
        .offset(
            2 as libc::c_int as isize,
        ) = *src.offset(1 as libc::c_int as isize)
        * *src.offset(6 as libc::c_int as isize)
        * *src.offset(15 as libc::c_int as isize)
        - *src.offset(1 as libc::c_int as isize) * *src.offset(7 as libc::c_int as isize)
            * *src.offset(14 as libc::c_int as isize)
        - *src.offset(5 as libc::c_int as isize) * *src.offset(2 as libc::c_int as isize)
            * *src.offset(15 as libc::c_int as isize)
        + *src.offset(5 as libc::c_int as isize) * *src.offset(3 as libc::c_int as isize)
            * *src.offset(14 as libc::c_int as isize)
        + *src.offset(13 as libc::c_int as isize)
            * *src.offset(2 as libc::c_int as isize)
            * *src.offset(7 as libc::c_int as isize)
        - *src.offset(13 as libc::c_int as isize)
            * *src.offset(3 as libc::c_int as isize)
            * *src.offset(6 as libc::c_int as isize);
    *dst
        .offset(
            6 as libc::c_int as isize,
        ) = -*src.offset(0 as libc::c_int as isize)
        * *src.offset(6 as libc::c_int as isize)
        * *src.offset(15 as libc::c_int as isize)
        + *src.offset(0 as libc::c_int as isize) * *src.offset(7 as libc::c_int as isize)
            * *src.offset(14 as libc::c_int as isize)
        + *src.offset(4 as libc::c_int as isize) * *src.offset(2 as libc::c_int as isize)
            * *src.offset(15 as libc::c_int as isize)
        - *src.offset(4 as libc::c_int as isize) * *src.offset(3 as libc::c_int as isize)
            * *src.offset(14 as libc::c_int as isize)
        - *src.offset(12 as libc::c_int as isize)
            * *src.offset(2 as libc::c_int as isize)
            * *src.offset(7 as libc::c_int as isize)
        + *src.offset(12 as libc::c_int as isize)
            * *src.offset(3 as libc::c_int as isize)
            * *src.offset(6 as libc::c_int as isize);
    *dst
        .offset(
            10 as libc::c_int as isize,
        ) = *src.offset(0 as libc::c_int as isize)
        * *src.offset(5 as libc::c_int as isize)
        * *src.offset(15 as libc::c_int as isize)
        - *src.offset(0 as libc::c_int as isize) * *src.offset(7 as libc::c_int as isize)
            * *src.offset(13 as libc::c_int as isize)
        - *src.offset(4 as libc::c_int as isize) * *src.offset(1 as libc::c_int as isize)
            * *src.offset(15 as libc::c_int as isize)
        + *src.offset(4 as libc::c_int as isize) * *src.offset(3 as libc::c_int as isize)
            * *src.offset(13 as libc::c_int as isize)
        + *src.offset(12 as libc::c_int as isize)
            * *src.offset(1 as libc::c_int as isize)
            * *src.offset(7 as libc::c_int as isize)
        - *src.offset(12 as libc::c_int as isize)
            * *src.offset(3 as libc::c_int as isize)
            * *src.offset(5 as libc::c_int as isize);
    *dst
        .offset(
            14 as libc::c_int as isize,
        ) = -*src.offset(0 as libc::c_int as isize)
        * *src.offset(5 as libc::c_int as isize)
        * *src.offset(14 as libc::c_int as isize)
        + *src.offset(0 as libc::c_int as isize) * *src.offset(6 as libc::c_int as isize)
            * *src.offset(13 as libc::c_int as isize)
        + *src.offset(4 as libc::c_int as isize) * *src.offset(1 as libc::c_int as isize)
            * *src.offset(14 as libc::c_int as isize)
        - *src.offset(4 as libc::c_int as isize) * *src.offset(2 as libc::c_int as isize)
            * *src.offset(13 as libc::c_int as isize)
        - *src.offset(12 as libc::c_int as isize)
            * *src.offset(1 as libc::c_int as isize)
            * *src.offset(6 as libc::c_int as isize)
        + *src.offset(12 as libc::c_int as isize)
            * *src.offset(2 as libc::c_int as isize)
            * *src.offset(5 as libc::c_int as isize);
    *dst
        .offset(
            3 as libc::c_int as isize,
        ) = -*src.offset(1 as libc::c_int as isize)
        * *src.offset(6 as libc::c_int as isize)
        * *src.offset(11 as libc::c_int as isize)
        + *src.offset(1 as libc::c_int as isize) * *src.offset(7 as libc::c_int as isize)
            * *src.offset(10 as libc::c_int as isize)
        + *src.offset(5 as libc::c_int as isize) * *src.offset(2 as libc::c_int as isize)
            * *src.offset(11 as libc::c_int as isize)
        - *src.offset(5 as libc::c_int as isize) * *src.offset(3 as libc::c_int as isize)
            * *src.offset(10 as libc::c_int as isize)
        - *src.offset(9 as libc::c_int as isize) * *src.offset(2 as libc::c_int as isize)
            * *src.offset(7 as libc::c_int as isize)
        + *src.offset(9 as libc::c_int as isize) * *src.offset(3 as libc::c_int as isize)
            * *src.offset(6 as libc::c_int as isize);
    *dst
        .offset(
            7 as libc::c_int as isize,
        ) = *src.offset(0 as libc::c_int as isize)
        * *src.offset(6 as libc::c_int as isize)
        * *src.offset(11 as libc::c_int as isize)
        - *src.offset(0 as libc::c_int as isize) * *src.offset(7 as libc::c_int as isize)
            * *src.offset(10 as libc::c_int as isize)
        - *src.offset(4 as libc::c_int as isize) * *src.offset(2 as libc::c_int as isize)
            * *src.offset(11 as libc::c_int as isize)
        + *src.offset(4 as libc::c_int as isize) * *src.offset(3 as libc::c_int as isize)
            * *src.offset(10 as libc::c_int as isize)
        + *src.offset(8 as libc::c_int as isize) * *src.offset(2 as libc::c_int as isize)
            * *src.offset(7 as libc::c_int as isize)
        - *src.offset(8 as libc::c_int as isize) * *src.offset(3 as libc::c_int as isize)
            * *src.offset(6 as libc::c_int as isize);
    *dst
        .offset(
            11 as libc::c_int as isize,
        ) = -*src.offset(0 as libc::c_int as isize)
        * *src.offset(5 as libc::c_int as isize)
        * *src.offset(11 as libc::c_int as isize)
        + *src.offset(0 as libc::c_int as isize) * *src.offset(7 as libc::c_int as isize)
            * *src.offset(9 as libc::c_int as isize)
        + *src.offset(4 as libc::c_int as isize) * *src.offset(1 as libc::c_int as isize)
            * *src.offset(11 as libc::c_int as isize)
        - *src.offset(4 as libc::c_int as isize) * *src.offset(3 as libc::c_int as isize)
            * *src.offset(9 as libc::c_int as isize)
        - *src.offset(8 as libc::c_int as isize) * *src.offset(1 as libc::c_int as isize)
            * *src.offset(7 as libc::c_int as isize)
        + *src.offset(8 as libc::c_int as isize) * *src.offset(3 as libc::c_int as isize)
            * *src.offset(5 as libc::c_int as isize);
    *dst
        .offset(
            15 as libc::c_int as isize,
        ) = *src.offset(0 as libc::c_int as isize)
        * *src.offset(5 as libc::c_int as isize)
        * *src.offset(10 as libc::c_int as isize)
        - *src.offset(0 as libc::c_int as isize) * *src.offset(6 as libc::c_int as isize)
            * *src.offset(9 as libc::c_int as isize)
        - *src.offset(4 as libc::c_int as isize) * *src.offset(1 as libc::c_int as isize)
            * *src.offset(10 as libc::c_int as isize)
        + *src.offset(4 as libc::c_int as isize) * *src.offset(2 as libc::c_int as isize)
            * *src.offset(9 as libc::c_int as isize)
        + *src.offset(8 as libc::c_int as isize) * *src.offset(1 as libc::c_int as isize)
            * *src.offset(6 as libc::c_int as isize)
        - *src.offset(8 as libc::c_int as isize) * *src.offset(2 as libc::c_int as isize)
            * *src.offset(5 as libc::c_int as isize);
    let mut det: libc::c_float = 1.0f32
        / (*src.offset(0 as libc::c_int as isize)
            * *dst.offset(0 as libc::c_int as isize)
            + *src.offset(1 as libc::c_int as isize)
                * *dst.offset(4 as libc::c_int as isize)
            + *src.offset(2 as libc::c_int as isize)
                * *dst.offset(8 as libc::c_int as isize)
            + *src.offset(3 as libc::c_int as isize)
                * *dst.offset(12 as libc::c_int as isize));
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 16 as libc::c_int {
        *dst.offset(i as isize) *= det;
        i += 1;
    }
}
unsafe extern "C" fn Matrix_IOTranspose(mut in_0: *const Matrix, mut out: *mut Matrix) {
    let mut src: *const libc::c_float = in_0 as *const libc::c_float;
    let mut dst: *mut libc::c_float = out as *mut libc::c_float;
    *dst.offset(0 as libc::c_int as isize) = *src.offset(0 as libc::c_int as isize);
    *dst.offset(1 as libc::c_int as isize) = *src.offset(4 as libc::c_int as isize);
    *dst.offset(2 as libc::c_int as isize) = *src.offset(8 as libc::c_int as isize);
    *dst.offset(3 as libc::c_int as isize) = *src.offset(12 as libc::c_int as isize);
    *dst.offset(4 as libc::c_int as isize) = *src.offset(1 as libc::c_int as isize);
    *dst.offset(5 as libc::c_int as isize) = *src.offset(5 as libc::c_int as isize);
    *dst.offset(6 as libc::c_int as isize) = *src.offset(9 as libc::c_int as isize);
    *dst.offset(7 as libc::c_int as isize) = *src.offset(13 as libc::c_int as isize);
    *dst.offset(8 as libc::c_int as isize) = *src.offset(2 as libc::c_int as isize);
    *dst.offset(9 as libc::c_int as isize) = *src.offset(6 as libc::c_int as isize);
    *dst.offset(10 as libc::c_int as isize) = *src.offset(10 as libc::c_int as isize);
    *dst.offset(11 as libc::c_int as isize) = *src.offset(14 as libc::c_int as isize);
    *dst.offset(12 as libc::c_int as isize) = *src.offset(3 as libc::c_int as isize);
    *dst.offset(13 as libc::c_int as isize) = *src.offset(7 as libc::c_int as isize);
    *dst.offset(14 as libc::c_int as isize) = *src.offset(11 as libc::c_int as isize);
    *dst.offset(15 as libc::c_int as isize) = *src.offset(15 as libc::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_Equal(
    mut a: *const Matrix,
    mut b: *const Matrix,
) -> bool {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 16 as libc::c_int {
        if (*a).m[i as usize] != (*b).m[i as usize] {
            return 0 as libc::c_int != 0;
        }
        i += 1;
    }
    return 1 as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_ApproximatelyEqual(
    mut a: *const Matrix,
    mut b: *const Matrix,
) -> bool {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 16 as libc::c_int {
        if !Float_ApproximatelyEqual(
            (*a).m[i as usize] as libc::c_double,
            (*b).m[i as usize] as libc::c_double,
        ) {
            return 0 as libc::c_int != 0;
        }
        i += 1;
    }
    return 1 as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_Inverse(mut self_0: *const Matrix) -> *mut Matrix {
    let mut result: Matrix = Matrix { m: [0.; 16] };
    Matrix_IOInverse(self_0, &mut result);
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_InverseTranspose(
    mut self_0: *const Matrix,
) -> *mut Matrix {
    let mut inverse: Matrix = Matrix { m: [0.; 16] };
    let mut result: Matrix = Matrix { m: [0.; 16] };
    Matrix_IOInverse(self_0, &mut inverse);
    Matrix_IOTranspose(&mut inverse, &mut result);
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_Sum(
    mut a: *const Matrix,
    mut b: *const Matrix,
) -> *mut Matrix {
    let mut result: Matrix = Matrix { m: [0.; 16] };
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 16 as libc::c_int {
        result.m[i as usize] = (*a).m[i as usize] + (*b).m[i as usize];
        i += 1;
    }
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_Transpose(mut self_0: *const Matrix) -> *mut Matrix {
    let mut result: Matrix = Matrix { m: [0.; 16] };
    Matrix_IOTranspose(self_0, &mut result);
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_IInverse(mut self_0: *mut Matrix) {
    let mut result: Matrix = Matrix { m: [0.; 16] };
    Matrix_IOInverse(self_0, &mut result);
    *self_0 = result;
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_IScale(
    mut self_0: *mut Matrix,
    mut scale: libc::c_float,
) {
    let mut m: *mut libc::c_float = ((*self_0).m).as_mut_ptr();
    *m.offset(0 as libc::c_int as isize) *= scale;
    *m.offset(1 as libc::c_int as isize) *= scale;
    *m.offset(2 as libc::c_int as isize) *= scale;
    *m.offset(4 as libc::c_int as isize) *= scale;
    *m.offset(5 as libc::c_int as isize) *= scale;
    *m.offset(6 as libc::c_int as isize) *= scale;
    *m.offset(8 as libc::c_int as isize) *= scale;
    *m.offset(9 as libc::c_int as isize) *= scale;
    *m.offset(10 as libc::c_int as isize) *= scale;
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_ITranspose(mut self_0: *mut Matrix) {
    let mut result: Matrix = Matrix { m: [0.; 16] };
    Matrix_IOTranspose(self_0, &mut result);
    *self_0 = result;
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_Identity() -> *mut Matrix {
    let identity: Matrix = {
        let mut init = Matrix {
            m: [
                1 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            ],
        };
        init
    };
    return Matrix_Clone(&identity);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_LookAt(
    mut pos: *const Vec3f,
    mut at: *const Vec3f,
    mut up: *const Vec3f,
) -> *mut Matrix {
    let mut z: Vec3f = Vec3f_Normalize(Vec3f_Sub(*pos, *at));
    let mut x: Vec3f = Vec3f_Normalize(Vec3f_Cross(*up, z));
    let mut y: Vec3f = Vec3f_Cross(z, x);
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                x.x,
                y.x,
                z.x,
                (*pos).x,
                x.y,
                y.y,
                z.y,
                (*pos).y,
                x.z,
                y.z,
                z.z,
                (*pos).z,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_LookUp(
    mut pos: *const Vec3f,
    mut look: *const Vec3f,
    mut up: *const Vec3f,
) -> *mut Matrix {
    let mut z: Vec3f = Vec3f_Normalize(Vec3f_Muls(*look, -1.0f32));
    let mut x: Vec3f = Vec3f_Normalize(Vec3f_Cross(*up, z));
    let mut y: Vec3f = Vec3f_Cross(z, x);
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                x.x,
                y.x,
                z.x,
                (*pos).x,
                x.y,
                y.y,
                z.y,
                (*pos).y,
                x.z,
                y.z,
                z.z,
                (*pos).z,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_Perspective(
    mut degreesFovy: libc::c_float,
    mut aspect: libc::c_float,
    mut N: libc::c_float,
    mut F: libc::c_float,
) -> *mut Matrix {
    let mut rads: libc::c_double = (3.14159265f32 * degreesFovy) as libc::c_double
        / 360.0f64;
    let mut cot: libc::c_double = 1.0f64 / Tan(rads);
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                (cot / aspect as libc::c_double) as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                cot as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                (N + F) / (N - F),
                (2.0f64 * (F * N) as libc::c_double / (N - F) as libc::c_double)
                    as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                -1.0f32,
                0 as libc::c_int as libc::c_float,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_Product(
    mut a: *const Matrix,
    mut b: *const Matrix,
) -> *mut Matrix {
    let mut result: Matrix = Matrix { m: [0.; 16] };
    let mut pResult: *mut libc::c_float = (result.m).as_mut_ptr();
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 4 as libc::c_int {
        let mut j: libc::c_int = 0 as libc::c_int;
        while j < 4 as libc::c_int {
            let mut sum: libc::c_float = 0.0f32;
            let mut k: libc::c_int = 0 as libc::c_int;
            while k < 4 as libc::c_int {
                sum
                    += (*a).m[(4 as libc::c_int * i + k) as usize]
                        * (*b).m[(4 as libc::c_int * k + j) as usize];
                k += 1;
            }
            let fresh0 = pResult;
            pResult = pResult.offset(1);
            *fresh0 = sum;
            j += 1;
        }
        i += 1;
    }
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_RotationX(mut rads: libc::c_float) -> *mut Matrix {
    let mut c: libc::c_float = Cos(rads as libc::c_double) as libc::c_float;
    let mut s: libc::c_float = Sin(rads as libc::c_double) as libc::c_float;
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                1 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                c,
                -s,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                s,
                c,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_RotationY(mut rads: libc::c_float) -> *mut Matrix {
    let mut c: libc::c_float = Cos(rads as libc::c_double) as libc::c_float;
    let mut s: libc::c_float = Sin(rads as libc::c_double) as libc::c_float;
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                c,
                0 as libc::c_int as libc::c_float,
                s,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                -s,
                0 as libc::c_int as libc::c_float,
                c,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_RotationZ(mut rads: libc::c_float) -> *mut Matrix {
    let mut c: libc::c_float = Cos(rads as libc::c_double) as libc::c_float;
    let mut s: libc::c_float = Sin(rads as libc::c_double) as libc::c_float;
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                c,
                -s,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                s,
                c,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_Scaling(
    mut sx: libc::c_float,
    mut sy: libc::c_float,
    mut sz: libc::c_float,
) -> *mut Matrix {
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                sx,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                sy,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                sz,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_SRT(
    mut sx: libc::c_float,
    mut sy: libc::c_float,
    mut sz: libc::c_float,
    mut ry: libc::c_float,
    mut rp: libc::c_float,
    mut rr: libc::c_float,
    mut tx: libc::c_float,
    mut ty: libc::c_float,
    mut tz: libc::c_float,
) -> *mut Matrix {
    let mut S: *mut Matrix = Matrix_Scaling(sx, sy, sz);
    let mut R: *mut Matrix = Matrix_YawPitchRoll(ry, rp, rr);
    let mut T: *mut Matrix = Matrix_Translation(tx, ty, tz);
    let mut TR: *mut Matrix = Matrix_Product(T, R);
    let mut TRS: *mut Matrix = Matrix_Product(TR, S);
    Matrix_Free(S);
    Matrix_Free(R);
    Matrix_Free(T);
    Matrix_Free(TR);
    return TRS;
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_Translation(
    mut tx: libc::c_float,
    mut ty: libc::c_float,
    mut tz: libc::c_float,
) -> *mut Matrix {
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                1 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                tx,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                ty,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
                tz,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_YawPitchRoll(
    mut yaw: libc::c_float,
    mut pitch: libc::c_float,
    mut roll: libc::c_float,
) -> *mut Matrix {
    let mut ca: libc::c_float = Cos(roll as libc::c_double) as libc::c_float;
    let mut sa: libc::c_float = Sin(roll as libc::c_double) as libc::c_float;
    let mut cb: libc::c_float = Cos(yaw as libc::c_double) as libc::c_float;
    let mut sb: libc::c_float = Sin(yaw as libc::c_double) as libc::c_float;
    let mut cy: libc::c_float = Cos(pitch as libc::c_double) as libc::c_float;
    let mut sy: libc::c_float = Sin(pitch as libc::c_double) as libc::c_float;
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                ca * cb,
                ca * sb * sy - sa * cy,
                ca * sb * cy + sa * sy,
                0 as libc::c_int as libc::c_float,
                sa * cb,
                sa * sb * sy + ca * cy,
                sa * sb * cy - ca * sy,
                0 as libc::c_int as libc::c_float,
                -sb,
                cb * sy,
                cb * cy,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_MulBox(
    mut self_0: *const Matrix,
    mut out: *mut Box3f,
    mut in_0: *const Box3f,
) {
    let corners: [Vec3f; 8] = [
        {
            let mut init = Vec3f {
                x: (*in_0).lower.x,
                y: (*in_0).lower.y,
                z: (*in_0).lower.z,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: (*in_0).upper.x,
                y: (*in_0).lower.y,
                z: (*in_0).lower.z,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: (*in_0).lower.x,
                y: (*in_0).upper.y,
                z: (*in_0).lower.z,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: (*in_0).upper.x,
                y: (*in_0).upper.y,
                z: (*in_0).lower.z,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: (*in_0).lower.x,
                y: (*in_0).lower.y,
                z: (*in_0).upper.z,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: (*in_0).upper.x,
                y: (*in_0).lower.y,
                z: (*in_0).upper.z,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: (*in_0).lower.x,
                y: (*in_0).upper.y,
                z: (*in_0).upper.z,
            };
            init
        },
        {
            let mut init = Vec3f {
                x: (*in_0).upper.x,
                y: (*in_0).upper.y,
                z: (*in_0).upper.z,
            };
            init
        },
    ];
    let mut result: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
    Matrix_MulPoint(
        self_0,
        &mut result,
        corners[0 as libc::c_int as usize].x,
        corners[0 as libc::c_int as usize].y,
        corners[0 as libc::c_int as usize].z,
    );
    (*out).lower = result;
    (*out).upper = result;
    let mut i: libc::c_int = 1 as libc::c_int;
    while i < 8 as libc::c_int {
        Matrix_MulPoint(
            self_0,
            &mut result,
            corners[i as usize].x,
            corners[i as usize].y,
            corners[i as usize].z,
        );
        (*out).lower = Vec3f_Min((*out).lower, result);
        (*out).upper = Vec3f_Max((*out).upper, result);
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_MulDir(
    mut self_0: *const Matrix,
    mut out: *mut Vec3f,
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut z: libc::c_float,
) {
    let mut m: *const libc::c_float = ((*self_0).m).as_ptr();
    (*out)
        .x = *m.offset(0 as libc::c_int as isize) * x
        + *m.offset(1 as libc::c_int as isize) * y
        + *m.offset(2 as libc::c_int as isize) * z;
    (*out)
        .y = *m.offset(4 as libc::c_int as isize) * x
        + *m.offset(5 as libc::c_int as isize) * y
        + *m.offset(6 as libc::c_int as isize) * z;
    (*out)
        .z = *m.offset(8 as libc::c_int as isize) * x
        + *m.offset(9 as libc::c_int as isize) * y
        + *m.offset(10 as libc::c_int as isize) * z;
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_MulPoint(
    mut self_0: *const Matrix,
    mut out: *mut Vec3f,
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut z: libc::c_float,
) {
    let mut m: *const libc::c_float = ((*self_0).m).as_ptr();
    (*out)
        .x = *m.offset(0 as libc::c_int as isize) * x
        + *m.offset(1 as libc::c_int as isize) * y
        + *m.offset(2 as libc::c_int as isize) * z
        + *m.offset(3 as libc::c_int as isize);
    (*out)
        .y = *m.offset(4 as libc::c_int as isize) * x
        + *m.offset(5 as libc::c_int as isize) * y
        + *m.offset(6 as libc::c_int as isize) * z
        + *m.offset(7 as libc::c_int as isize);
    (*out)
        .z = *m.offset(8 as libc::c_int as isize) * x
        + *m.offset(9 as libc::c_int as isize) * y
        + *m.offset(10 as libc::c_int as isize) * z
        + *m.offset(11 as libc::c_int as isize);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_MulVec(
    mut self_0: *const Matrix,
    mut out: *mut Vec4f,
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut z: libc::c_float,
    mut w: libc::c_float,
) {
    let mut m: *const libc::c_float = ((*self_0).m).as_ptr();
    (*out)
        .x = *m.offset(0 as libc::c_int as isize) * x
        + *m.offset(1 as libc::c_int as isize) * y
        + *m.offset(2 as libc::c_int as isize) * z
        + *m.offset(3 as libc::c_int as isize) * w;
    (*out)
        .y = *m.offset(4 as libc::c_int as isize) * x
        + *m.offset(5 as libc::c_int as isize) * y
        + *m.offset(6 as libc::c_int as isize) * z
        + *m.offset(7 as libc::c_int as isize) * w;
    (*out)
        .z = *m.offset(8 as libc::c_int as isize) * x
        + *m.offset(9 as libc::c_int as isize) * y
        + *m.offset(10 as libc::c_int as isize) * z
        + *m.offset(11 as libc::c_int as isize) * w;
    (*out)
        .w = *m.offset(12 as libc::c_int as isize) * x
        + *m.offset(13 as libc::c_int as isize) * y
        + *m.offset(14 as libc::c_int as isize) * z
        + *m.offset(15 as libc::c_int as isize) * w;
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_GetForward(
    mut self_0: *const Matrix,
    mut out: *mut Vec3f,
) {
    (*out).x = -(*self_0).m[2 as libc::c_int as usize];
    (*out).y = -(*self_0).m[6 as libc::c_int as usize];
    (*out).z = -(*self_0).m[10 as libc::c_int as usize];
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_GetRight(
    mut self_0: *const Matrix,
    mut out: *mut Vec3f,
) {
    (*out).x = (*self_0).m[0 as libc::c_int as usize];
    (*out).y = (*self_0).m[4 as libc::c_int as usize];
    (*out).z = (*self_0).m[8 as libc::c_int as usize];
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_GetUp(mut self_0: *const Matrix, mut out: *mut Vec3f) {
    (*out).x = (*self_0).m[1 as libc::c_int as usize];
    (*out).y = (*self_0).m[5 as libc::c_int as usize];
    (*out).z = (*self_0).m[9 as libc::c_int as usize];
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_GetPos(mut self_0: *const Matrix, mut out: *mut Vec3f) {
    (*out).x = (*self_0).m[3 as libc::c_int as usize];
    (*out).y = (*self_0).m[7 as libc::c_int as usize];
    (*out).z = (*self_0).m[11 as libc::c_int as usize];
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_GetRow(
    mut self_0: *const Matrix,
    mut out: *mut Vec4f,
    mut row: libc::c_int,
) {
    (*out).x = (*self_0).m[(4 as libc::c_int * row + 0 as libc::c_int) as usize];
    (*out).y = (*self_0).m[(4 as libc::c_int * row + 1 as libc::c_int) as usize];
    (*out).z = (*self_0).m[(4 as libc::c_int * row + 2 as libc::c_int) as usize];
    (*out).w = (*self_0).m[(4 as libc::c_int * row + 3 as libc::c_int) as usize];
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_FromBasis(
    mut x: *const Vec3f,
    mut y: *const Vec3f,
    mut z: *const Vec3f,
) -> *mut Matrix {
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                (*x).x,
                (*y).x,
                (*z).x,
                0 as libc::c_int as libc::c_float,
                (*x).y,
                (*y).y,
                (*z).y,
                0 as libc::c_int as libc::c_float,
                (*x).z,
                (*y).z,
                (*z).z,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_FromPosRot(
    mut pos: *const Vec3f,
    mut rot: *const Quat,
) -> *mut Matrix {
    let mut x: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
    Quat_GetAxisX(rot, &mut x);
    let mut y: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
    Quat_GetAxisY(rot, &mut y);
    let mut z: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
    Quat_GetAxisZ(rot, &mut z);
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                x.x,
                y.x,
                z.x,
                (*pos).x,
                x.y,
                y.y,
                z.y,
                (*pos).y,
                x.z,
                y.z,
                z.z,
                (*pos).z,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_FromPosRotScale(
    mut pos: *const Vec3f,
    mut rot: *const Quat,
    mut scale: libc::c_float,
) -> *mut Matrix {
    let mut x: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
    Quat_GetAxisX(rot, &mut x);
    let mut y: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
    Quat_GetAxisY(rot, &mut y);
    let mut z: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
    Quat_GetAxisZ(rot, &mut z);
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                scale * x.x,
                scale * y.x,
                scale * z.x,
                (*pos).x,
                scale * x.y,
                scale * y.y,
                scale * z.y,
                (*pos).y,
                scale * x.z,
                scale * y.z,
                scale * z.z,
                (*pos).z,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_FromPosBasis(
    mut pos: *const Vec3f,
    mut x: *const Vec3f,
    mut y: *const Vec3f,
    mut z: *const Vec3f,
) -> *mut Matrix {
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                (*x).x,
                (*y).x,
                (*z).x,
                (*pos).x,
                (*x).y,
                (*y).y,
                (*z).y,
                (*pos).y,
                (*x).z,
                (*y).z,
                (*z).z,
                (*pos).z,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_FromQuat(mut q: *const Quat) -> *mut Matrix {
    let mut x: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
    Quat_GetAxisX(q, &mut x);
    let mut y: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
    Quat_GetAxisY(q, &mut y);
    let mut z: Vec3f = Vec3f { x: 0., y: 0., z: 0. };
    Quat_GetAxisZ(q, &mut z);
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                x.x,
                y.x,
                z.x,
                0 as libc::c_int as libc::c_float,
                x.y,
                y.y,
                z.y,
                0 as libc::c_int as libc::c_float,
                x.z,
                y.z,
                z.z,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                0 as libc::c_int as libc::c_float,
                1 as libc::c_int as libc::c_float,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_ToQuat(mut self_0: *const Matrix, mut q: *mut Quat) {
    let mut m: *const libc::c_float = self_0 as *const libc::c_float;
    let mut x: Vec3f = {
        let mut init = Vec3f {
            x: *m.offset(0 as libc::c_int as isize),
            y: *m.offset(4 as libc::c_int as isize),
            z: *m.offset(8 as libc::c_int as isize),
        };
        init
    };
    let mut y: Vec3f = {
        let mut init = Vec3f {
            x: *m.offset(1 as libc::c_int as isize),
            y: *m.offset(5 as libc::c_int as isize),
            z: *m.offset(9 as libc::c_int as isize),
        };
        init
    };
    let mut z: Vec3f = {
        let mut init = Vec3f {
            x: *m.offset(2 as libc::c_int as isize),
            y: *m.offset(6 as libc::c_int as isize),
            z: *m.offset(10 as libc::c_int as isize),
        };
        init
    };
    Quat_FromBasis(&mut x, &mut y, &mut z, q);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_Print(mut self_0: *const Matrix) {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 4 as libc::c_int {
        let mut j: libc::c_int = 0 as libc::c_int;
        while j < 4 as libc::c_int {
            printf(
                b"%f \0" as *const u8 as *const libc::c_char,
                (*self_0).m[(4 as libc::c_int * i + j) as usize] as libc::c_double,
            );
            j += 1;
        }
        printf(b"\n\0" as *const u8 as *const libc::c_char);
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_ToString(mut self_0: *const Matrix) -> cstr {
    static mut buffer: [libc::c_char; 512] = [0; 512];
    let mut m: *const libc::c_float = ((*self_0).m).as_ptr();
    snprintf(
        buffer.as_mut_ptr(),
        (::core::mem::size_of::<[libc::c_char; 512]>())
            .wrapping_div(::core::mem::size_of::<libc::c_char>())
            as libc::c_int as libc::size_t,
        b"[%+.2f, %+.2f, %+.2f, %+.2f]\n[%+.2f, %+.2f, %+.2f, %+.2f]\n[%+.2f, %+.2f, %+.2f, %+.2f]\n[%+.2f, %+.2f, %+.2f, %+.2f]\0"
            as *const u8 as *const libc::c_char,
        *m.offset(0 as libc::c_int as isize) as libc::c_double,
        *m.offset(1 as libc::c_int as isize) as libc::c_double,
        *m.offset(2 as libc::c_int as isize) as libc::c_double,
        *m.offset(3 as libc::c_int as isize) as libc::c_double,
        *m.offset(4 as libc::c_int as isize) as libc::c_double,
        *m.offset(5 as libc::c_int as isize) as libc::c_double,
        *m.offset(6 as libc::c_int as isize) as libc::c_double,
        *m.offset(7 as libc::c_int as isize) as libc::c_double,
        *m.offset(8 as libc::c_int as isize) as libc::c_double,
        *m.offset(9 as libc::c_int as isize) as libc::c_double,
        *m.offset(10 as libc::c_int as isize) as libc::c_double,
        *m.offset(11 as libc::c_int as isize) as libc::c_double,
        *m.offset(12 as libc::c_int as isize) as libc::c_double,
        *m.offset(13 as libc::c_int as isize) as libc::c_double,
        *m.offset(14 as libc::c_int as isize) as libc::c_double,
        *m.offset(15 as libc::c_int as isize) as libc::c_double,
    );
    return buffer.as_mut_ptr() as cstr;
}
