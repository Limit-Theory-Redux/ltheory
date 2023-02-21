use ::libc;
use glam::Vec3;
use crate::internal::Memory::*;
extern "C" {
    fn cos(_: f64) -> f64;
    fn sin(_: f64) -> f64;
    fn tan(_: f64) -> f64;
    fn fabs(_: f64) -> f64;
    fn sqrt(_: f64) -> f64;
    fn printf(_: *const libc::c_char, _: ...) -> libc::c_int;
    fn snprintf(
        _: *mut libc::c_char,
        _: libc::size_t,
        _: *const libc::c_char,
        _: ...
    ) -> libc::c_int;
    fn Quat_GetAxisX(_: *const Quat, _: *mut Vec3);
    fn Quat_GetAxisY(_: *const Quat, _: *mut Vec3);
    fn Quat_GetAxisZ(_: *const Quat, _: *mut Vec3);
    fn Quat_FromBasis(x: *const Vec3, y: *const Vec3, z: *const Vec3, _: *mut Quat);
}
pub type cstr = *const libc::c_char;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Box3f {
    pub lower: Vec3,
    pub upper: Vec3,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Matrix {
    pub m: [f32; 16],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec4f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
#[inline]
unsafe extern "C" fn Abs(mut t: f64) -> f64 {
    return fabs(t);
}
#[inline]
unsafe extern "C" fn Sqrtf(mut t: f32) -> f32 {
    return sqrt(t as f64) as f32;
}
#[inline]
unsafe extern "C" fn Maxf(mut a: f32, mut b: f32) -> f32 {
    return if a > b { a } else { b };
}
#[inline]
unsafe extern "C" fn Minf(mut a: f32, mut b: f32) -> f32 {
    return if a < b { a } else { b };
}
#[inline]
unsafe extern "C" fn Cos(mut t: f64) -> f64 {
    return cos(t);
}
#[inline]
unsafe extern "C" fn Sin(mut t: f64) -> f64 {
    return sin(t);
}
#[inline]
unsafe extern "C" fn Tan(mut t: f64) -> f64 {
    return tan(t);
}
#[inline]
unsafe extern "C" fn Float_ApproximatelyEqual(
    mut x: f64,
    mut y: f64,
) -> bool {
    return Abs(x - y) < 1e-3f64;
}

#[no_mangle]
pub unsafe extern "C" fn Matrix_Clone(mut this: *const Matrix) -> *mut Matrix {
    let mut clone: *mut Matrix = MemAlloc(
        ::core::mem::size_of::<Matrix>() as usize,
    ) as *mut Matrix;
    *clone = *this;
    return clone;
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_Free(mut this: *mut Matrix) {
    MemFree(this as *const libc::c_void);
}
unsafe extern "C" fn Matrix_IOInverse(mut in_0: *const Matrix, mut out: *mut Matrix) {
    let mut src: *const f32 = in_0 as *const f32;
    let mut dst: *mut f32 = out as *mut f32;
    *dst
        .offset(
            0 as libc::c_int as isize,
        ) = *src.offset(5)
        * *src.offset(10)
        * *src.offset(15)
        - *src.offset(5)
            * *src.offset(11)
            * *src.offset(14)
        - *src.offset(9) * *src.offset(6)
            * *src.offset(15)
        + *src.offset(9) * *src.offset(7)
            * *src.offset(14)
        + *src.offset(13)
            * *src.offset(6)
            * *src.offset(11)
        - *src.offset(13)
            * *src.offset(7)
            * *src.offset(10);
    *dst
        .offset(
            4 as libc::c_int as isize,
        ) = -*src.offset(4)
        * *src.offset(10)
        * *src.offset(15)
        + *src.offset(4)
            * *src.offset(11)
            * *src.offset(14)
        + *src.offset(8) * *src.offset(6)
            * *src.offset(15)
        - *src.offset(8) * *src.offset(7)
            * *src.offset(14)
        - *src.offset(12)
            * *src.offset(6)
            * *src.offset(11)
        + *src.offset(12)
            * *src.offset(7)
            * *src.offset(10);
    *dst
        .offset(
            8 as libc::c_int as isize,
        ) = *src.offset(4)
        * *src.offset(9)
        * *src.offset(15)
        - *src.offset(4)
            * *src.offset(11)
            * *src.offset(13)
        - *src.offset(8) * *src.offset(5)
            * *src.offset(15)
        + *src.offset(8) * *src.offset(7)
            * *src.offset(13)
        + *src.offset(12)
            * *src.offset(5)
            * *src.offset(11)
        - *src.offset(12)
            * *src.offset(7)
            * *src.offset(9);
    *dst
        .offset(
            12 as libc::c_int as isize,
        ) = -*src.offset(4)
        * *src.offset(9)
        * *src.offset(14)
        + *src.offset(4)
            * *src.offset(10)
            * *src.offset(13)
        + *src.offset(8) * *src.offset(5)
            * *src.offset(14)
        - *src.offset(8) * *src.offset(6)
            * *src.offset(13)
        - *src.offset(12)
            * *src.offset(5)
            * *src.offset(10)
        + *src.offset(12)
            * *src.offset(6)
            * *src.offset(9);
    *dst
        .offset(
            1 as libc::c_int as isize,
        ) = -*src.offset(1)
        * *src.offset(10)
        * *src.offset(15)
        + *src.offset(1)
            * *src.offset(11)
            * *src.offset(14)
        + *src.offset(9) * *src.offset(2)
            * *src.offset(15)
        - *src.offset(9) * *src.offset(3)
            * *src.offset(14)
        - *src.offset(13)
            * *src.offset(2)
            * *src.offset(11)
        + *src.offset(13)
            * *src.offset(3)
            * *src.offset(10);
    *dst
        .offset(
            5 as libc::c_int as isize,
        ) = *src.offset(0)
        * *src.offset(10)
        * *src.offset(15)
        - *src.offset(0)
            * *src.offset(11)
            * *src.offset(14)
        - *src.offset(8) * *src.offset(2)
            * *src.offset(15)
        + *src.offset(8) * *src.offset(3)
            * *src.offset(14)
        + *src.offset(12)
            * *src.offset(2)
            * *src.offset(11)
        - *src.offset(12)
            * *src.offset(3)
            * *src.offset(10);
    *dst
        .offset(
            9 as libc::c_int as isize,
        ) = -*src.offset(0)
        * *src.offset(9)
        * *src.offset(15)
        + *src.offset(0)
            * *src.offset(11)
            * *src.offset(13)
        + *src.offset(8) * *src.offset(1)
            * *src.offset(15)
        - *src.offset(8) * *src.offset(3)
            * *src.offset(13)
        - *src.offset(12)
            * *src.offset(1)
            * *src.offset(11)
        + *src.offset(12)
            * *src.offset(3)
            * *src.offset(9);
    *dst
        .offset(
            13 as libc::c_int as isize,
        ) = *src.offset(0)
        * *src.offset(9)
        * *src.offset(14)
        - *src.offset(0)
            * *src.offset(10)
            * *src.offset(13)
        - *src.offset(8) * *src.offset(1)
            * *src.offset(14)
        + *src.offset(8) * *src.offset(2)
            * *src.offset(13)
        + *src.offset(12)
            * *src.offset(1)
            * *src.offset(10)
        - *src.offset(12)
            * *src.offset(2)
            * *src.offset(9);
    *dst
        .offset(
            2 as libc::c_int as isize,
        ) = *src.offset(1)
        * *src.offset(6)
        * *src.offset(15)
        - *src.offset(1) * *src.offset(7)
            * *src.offset(14)
        - *src.offset(5) * *src.offset(2)
            * *src.offset(15)
        + *src.offset(5) * *src.offset(3)
            * *src.offset(14)
        + *src.offset(13)
            * *src.offset(2)
            * *src.offset(7)
        - *src.offset(13)
            * *src.offset(3)
            * *src.offset(6);
    *dst
        .offset(
            6 as libc::c_int as isize,
        ) = -*src.offset(0)
        * *src.offset(6)
        * *src.offset(15)
        + *src.offset(0) * *src.offset(7)
            * *src.offset(14)
        + *src.offset(4) * *src.offset(2)
            * *src.offset(15)
        - *src.offset(4) * *src.offset(3)
            * *src.offset(14)
        - *src.offset(12)
            * *src.offset(2)
            * *src.offset(7)
        + *src.offset(12)
            * *src.offset(3)
            * *src.offset(6);
    *dst
        .offset(
            10 as libc::c_int as isize,
        ) = *src.offset(0)
        * *src.offset(5)
        * *src.offset(15)
        - *src.offset(0) * *src.offset(7)
            * *src.offset(13)
        - *src.offset(4) * *src.offset(1)
            * *src.offset(15)
        + *src.offset(4) * *src.offset(3)
            * *src.offset(13)
        + *src.offset(12)
            * *src.offset(1)
            * *src.offset(7)
        - *src.offset(12)
            * *src.offset(3)
            * *src.offset(5);
    *dst
        .offset(
            14 as libc::c_int as isize,
        ) = -*src.offset(0)
        * *src.offset(5)
        * *src.offset(14)
        + *src.offset(0) * *src.offset(6)
            * *src.offset(13)
        + *src.offset(4) * *src.offset(1)
            * *src.offset(14)
        - *src.offset(4) * *src.offset(2)
            * *src.offset(13)
        - *src.offset(12)
            * *src.offset(1)
            * *src.offset(6)
        + *src.offset(12)
            * *src.offset(2)
            * *src.offset(5);
    *dst
        .offset(
            3 as libc::c_int as isize,
        ) = -*src.offset(1)
        * *src.offset(6)
        * *src.offset(11)
        + *src.offset(1) * *src.offset(7)
            * *src.offset(10)
        + *src.offset(5) * *src.offset(2)
            * *src.offset(11)
        - *src.offset(5) * *src.offset(3)
            * *src.offset(10)
        - *src.offset(9) * *src.offset(2)
            * *src.offset(7)
        + *src.offset(9) * *src.offset(3)
            * *src.offset(6);
    *dst
        .offset(
            7 as libc::c_int as isize,
        ) = *src.offset(0)
        * *src.offset(6)
        * *src.offset(11)
        - *src.offset(0) * *src.offset(7)
            * *src.offset(10)
        - *src.offset(4) * *src.offset(2)
            * *src.offset(11)
        + *src.offset(4) * *src.offset(3)
            * *src.offset(10)
        + *src.offset(8) * *src.offset(2)
            * *src.offset(7)
        - *src.offset(8) * *src.offset(3)
            * *src.offset(6);
    *dst
        .offset(
            11 as libc::c_int as isize,
        ) = -*src.offset(0)
        * *src.offset(5)
        * *src.offset(11)
        + *src.offset(0) * *src.offset(7)
            * *src.offset(9)
        + *src.offset(4) * *src.offset(1)
            * *src.offset(11)
        - *src.offset(4) * *src.offset(3)
            * *src.offset(9)
        - *src.offset(8) * *src.offset(1)
            * *src.offset(7)
        + *src.offset(8) * *src.offset(3)
            * *src.offset(5);
    *dst
        .offset(
            15 as libc::c_int as isize,
        ) = *src.offset(0)
        * *src.offset(5)
        * *src.offset(10)
        - *src.offset(0) * *src.offset(6)
            * *src.offset(9)
        - *src.offset(4) * *src.offset(1)
            * *src.offset(10)
        + *src.offset(4) * *src.offset(2)
            * *src.offset(9)
        + *src.offset(8) * *src.offset(1)
            * *src.offset(6)
        - *src.offset(8) * *src.offset(2)
            * *src.offset(5);
    let mut det: f32 = 1.0f32
        / (*src.offset(0)
            * *dst.offset(0)
            + *src.offset(1)
                * *dst.offset(4)
            + *src.offset(2)
                * *dst.offset(8)
            + *src.offset(3)
                * *dst.offset(12));
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 16 as libc::c_int {
        *dst.offset(i as isize) *= det;
        i += 1;
    }
}
unsafe extern "C" fn Matrix_IOTranspose(mut in_0: *const Matrix, mut out: *mut Matrix) {
    let mut src: *const f32 = in_0 as *const f32;
    let mut dst: *mut f32 = out as *mut f32;
    *dst.offset(0) = *src.offset(0);
    *dst.offset(1) = *src.offset(4);
    *dst.offset(2) = *src.offset(8);
    *dst.offset(3) = *src.offset(12);
    *dst.offset(4) = *src.offset(1);
    *dst.offset(5) = *src.offset(5);
    *dst.offset(6) = *src.offset(9);
    *dst.offset(7) = *src.offset(13);
    *dst.offset(8) = *src.offset(2);
    *dst.offset(9) = *src.offset(6);
    *dst.offset(10) = *src.offset(10);
    *dst.offset(11) = *src.offset(14);
    *dst.offset(12) = *src.offset(3);
    *dst.offset(13) = *src.offset(7);
    *dst.offset(14) = *src.offset(11);
    *dst.offset(15) = *src.offset(15);
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
            (*a).m[i as usize] as f64,
            (*b).m[i as usize] as f64,
        ) {
            return 0 as libc::c_int != 0;
        }
        i += 1;
    }
    return 1 as libc::c_int != 0;
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_Inverse(mut this: *const Matrix) -> *mut Matrix {
    let mut result: Matrix = Matrix { m: [0.; 16] };
    Matrix_IOInverse(this, &mut result);
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_InverseTranspose(
    mut this: *const Matrix,
) -> *mut Matrix {
    let mut inverse: Matrix = Matrix { m: [0.; 16] };
    let mut result: Matrix = Matrix { m: [0.; 16] };
    Matrix_IOInverse(this, &mut inverse);
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
pub unsafe extern "C" fn Matrix_Transpose(mut this: *const Matrix) -> *mut Matrix {
    let mut result: Matrix = Matrix { m: [0.; 16] };
    Matrix_IOTranspose(this, &mut result);
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_IInverse(mut this: *mut Matrix) {
    let mut result: Matrix = Matrix { m: [0.; 16] };
    Matrix_IOInverse(this, &mut result);
    *this = result;
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_IScale(
    mut this: *mut Matrix,
    mut scale: f32,
) {
    let mut m: *mut f32 = ((*this).m).as_mut_ptr();
    *m.offset(0) *= scale;
    *m.offset(1) *= scale;
    *m.offset(2) *= scale;
    *m.offset(4) *= scale;
    *m.offset(5) *= scale;
    *m.offset(6) *= scale;
    *m.offset(8) *= scale;
    *m.offset(9) *= scale;
    *m.offset(10) *= scale;
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_ITranspose(mut this: *mut Matrix) {
    let mut result: Matrix = Matrix { m: [0.; 16] };
    Matrix_IOTranspose(this, &mut result);
    *this = result;
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_Identity() -> *mut Matrix {
    let identity: Matrix = {
        let mut init = Matrix {
            m: [
                1.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                1.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                1.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                1.0f32,
            ],
        };
        init
    };
    return Matrix_Clone(&identity);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_LookAt(
    mut pos: *const Vec3,
    mut at: *const Vec3,
    mut up: *const Vec3,
) -> *mut Matrix {
    let mut z: Vec3 = (*pos - *at).normalize();
    let mut x: Vec3 = Vec3::cross(*up, z).normalize();
    let mut y: Vec3 = Vec3::cross(z, x);
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
                0.0f32,
                0.0f32,
                0.0f32,
                1.0f32,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_LookUp(
    mut pos: *const Vec3,
    mut look: *const Vec3,
    mut up: *const Vec3,
) -> *mut Matrix {
    let mut z: Vec3 = (*look * -1.0f32).normalize();
    let mut x: Vec3 = Vec3::cross(*up, z).normalize();
    let mut y: Vec3 = Vec3::cross(z, x);
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
                0.0f32,
                0.0f32,
                0.0f32,
                1.0f32,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_Perspective(
    mut degreesFovy: f32,
    mut aspect: f32,
    mut N: f32,
    mut F: f32,
) -> *mut Matrix {
    let mut rads: f64 = (3.14159265f32 * degreesFovy) as f64
        / 360.0f64;
    let mut cot: f64 = 1.0f64 / Tan(rads);
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                (cot / aspect as f64) as f32,
                0.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                cot as f32,
                0.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                (N + F) / (N - F),
                (2.0f64 * (F * N) as f64 / (N - F) as f64)
                    as f32,
                0.0f32,
                0.0f32,
                -1.0f32,
                0.0f32,
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
    let mut pResult: *mut f32 = (result.m).as_mut_ptr();
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 4 as libc::c_int {
        let mut j: libc::c_int = 0 as libc::c_int;
        while j < 4 as libc::c_int {
            let mut sum: f32 = 0.0f32;
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
pub unsafe extern "C" fn Matrix_RotationX(mut rads: f32) -> *mut Matrix {
    let mut c: f32 = Cos(rads as f64) as f32;
    let mut s: f32 = Sin(rads as f64) as f32;
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                1.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                c,
                -s,
                0.0f32,
                0.0f32,
                s,
                c,
                0.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                1.0f32,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_RotationY(mut rads: f32) -> *mut Matrix {
    let mut c: f32 = Cos(rads as f64) as f32;
    let mut s: f32 = Sin(rads as f64) as f32;
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                c,
                0.0f32,
                s,
                0.0f32,
                0.0f32,
                1.0f32,
                0.0f32,
                0.0f32,
                -s,
                0.0f32,
                c,
                0.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                1.0f32,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_RotationZ(mut rads: f32) -> *mut Matrix {
    let mut c: f32 = Cos(rads as f64) as f32;
    let mut s: f32 = Sin(rads as f64) as f32;
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                c,
                -s,
                0.0f32,
                0.0f32,
                s,
                c,
                0.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                1.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                1.0f32,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_Scaling(
    mut sx: f32,
    mut sy: f32,
    mut sz: f32,
) -> *mut Matrix {
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                sx,
                0.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                sy,
                0.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                sz,
                0.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                1.0f32,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_SRT(
    mut sx: f32,
    mut sy: f32,
    mut sz: f32,
    mut ry: f32,
    mut rp: f32,
    mut rr: f32,
    mut tx: f32,
    mut ty: f32,
    mut tz: f32,
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
    mut tx: f32,
    mut ty: f32,
    mut tz: f32,
) -> *mut Matrix {
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                1.0f32,
                0.0f32,
                0.0f32,
                tx,
                0.0f32,
                1.0f32,
                0.0f32,
                ty,
                0.0f32,
                0.0f32,
                1.0f32,
                tz,
                0.0f32,
                0.0f32,
                0.0f32,
                1.0f32,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_YawPitchRoll(
    mut yaw: f32,
    mut pitch: f32,
    mut roll: f32,
) -> *mut Matrix {
    let mut ca: f32 = Cos(roll as f64) as f32;
    let mut sa: f32 = Sin(roll as f64) as f32;
    let mut cb: f32 = Cos(yaw as f64) as f32;
    let mut sb: f32 = Sin(yaw as f64) as f32;
    let mut cy: f32 = Cos(pitch as f64) as f32;
    let mut sy: f32 = Sin(pitch as f64) as f32;
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                ca * cb,
                ca * sb * sy - sa * cy,
                ca * sb * cy + sa * sy,
                0.0f32,
                sa * cb,
                sa * sb * sy + ca * cy,
                sa * sb * cy - ca * sy,
                0.0f32,
                -sb,
                cb * sy,
                cb * cy,
                0.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                1.0f32,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_MulBox(
    mut this: *const Matrix,
    mut out: *mut Box3f,
    mut in_0: *const Box3f,
) {
    let corners: [Vec3; 8] = [
        {
            let mut init = Vec3 {
                x: (*in_0).lower.x,
                y: (*in_0).lower.y,
                z: (*in_0).lower.z,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: (*in_0).upper.x,
                y: (*in_0).lower.y,
                z: (*in_0).lower.z,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: (*in_0).lower.x,
                y: (*in_0).upper.y,
                z: (*in_0).lower.z,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: (*in_0).upper.x,
                y: (*in_0).upper.y,
                z: (*in_0).lower.z,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: (*in_0).lower.x,
                y: (*in_0).lower.y,
                z: (*in_0).upper.z,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: (*in_0).upper.x,
                y: (*in_0).lower.y,
                z: (*in_0).upper.z,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: (*in_0).lower.x,
                y: (*in_0).upper.y,
                z: (*in_0).upper.z,
            };
            init
        },
        {
            let mut init = Vec3 {
                x: (*in_0).upper.x,
                y: (*in_0).upper.y,
                z: (*in_0).upper.z,
            };
            init
        },
    ];
    let mut result = Vec3::ZERO;
    Matrix_MulPoint(
        this,
        &mut result,
        corners[0].x,
        corners[0].y,
        corners[0].z,
    );
    (*out).lower = result;
    (*out).upper = result;
    let mut i: libc::c_int = 1 as libc::c_int;
    while i < 8 as libc::c_int {
        Matrix_MulPoint(
            this,
            &mut result,
            corners[i as usize].x,
            corners[i as usize].y,
            corners[i as usize].z,
        );
        (*out).lower = Vec3::min((*out).lower, result);
        (*out).upper = Vec3::max((*out).upper, result);
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_MulDir(
    mut this: *const Matrix,
    mut out: *mut Vec3,
    mut x: f32,
    mut y: f32,
    mut z: f32,
) {
    let mut m: *const f32 = ((*this).m).as_ptr();
    (*out)
        .x = *m.offset(0) * x
        + *m.offset(1) * y
        + *m.offset(2) * z;
    (*out)
        .y = *m.offset(4) * x
        + *m.offset(5) * y
        + *m.offset(6) * z;
    (*out)
        .z = *m.offset(8) * x
        + *m.offset(9) * y
        + *m.offset(10) * z;
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_MulPoint(
    mut this: *const Matrix,
    mut out: *mut Vec3,
    mut x: f32,
    mut y: f32,
    mut z: f32,
) {
    let mut m: *const f32 = ((*this).m).as_ptr();
    (*out)
        .x = *m.offset(0) * x
        + *m.offset(1) * y
        + *m.offset(2) * z
        + *m.offset(3);
    (*out)
        .y = *m.offset(4) * x
        + *m.offset(5) * y
        + *m.offset(6) * z
        + *m.offset(7);
    (*out)
        .z = *m.offset(8) * x
        + *m.offset(9) * y
        + *m.offset(10) * z
        + *m.offset(11);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_MulVec(
    mut this: *const Matrix,
    mut out: *mut Vec4f,
    mut x: f32,
    mut y: f32,
    mut z: f32,
    mut w: f32,
) {
    let mut m: *const f32 = ((*this).m).as_ptr();
    (*out)
        .x = *m.offset(0) * x
        + *m.offset(1) * y
        + *m.offset(2) * z
        + *m.offset(3) * w;
    (*out)
        .y = *m.offset(4) * x
        + *m.offset(5) * y
        + *m.offset(6) * z
        + *m.offset(7) * w;
    (*out)
        .z = *m.offset(8) * x
        + *m.offset(9) * y
        + *m.offset(10) * z
        + *m.offset(11) * w;
    (*out)
        .w = *m.offset(12) * x
        + *m.offset(13) * y
        + *m.offset(14) * z
        + *m.offset(15) * w;
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_GetForward(
    mut this: *const Matrix,
    mut out: *mut Vec3,
) {
    (*out).x = -(*this).m[2];
    (*out).y = -(*this).m[6];
    (*out).z = -(*this).m[10];
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_GetRight(
    mut this: *const Matrix,
    mut out: *mut Vec3,
) {
    (*out).x = (*this).m[0];
    (*out).y = (*this).m[4];
    (*out).z = (*this).m[8];
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_GetUp(mut this: *const Matrix, mut out: *mut Vec3) {
    (*out).x = (*this).m[1];
    (*out).y = (*this).m[5];
    (*out).z = (*this).m[9];
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_GetPos(mut this: *const Matrix, mut out: *mut Vec3) {
    (*out).x = (*this).m[3];
    (*out).y = (*this).m[7];
    (*out).z = (*this).m[11];
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_GetRow(
    mut this: *const Matrix,
    mut out: *mut Vec4f,
    mut row: libc::c_int,
) {
    (*out).x = (*this).m[(4 as libc::c_int * row + 0 as libc::c_int) as usize];
    (*out).y = (*this).m[(4 as libc::c_int * row + 1 as libc::c_int) as usize];
    (*out).z = (*this).m[(4 as libc::c_int * row + 2 as libc::c_int) as usize];
    (*out).w = (*this).m[(4 as libc::c_int * row + 3 as libc::c_int) as usize];
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_FromBasis(
    mut x: *const Vec3,
    mut y: *const Vec3,
    mut z: *const Vec3,
) -> *mut Matrix {
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                (*x).x,
                (*y).x,
                (*z).x,
                0.0f32,
                (*x).y,
                (*y).y,
                (*z).y,
                0.0f32,
                (*x).z,
                (*y).z,
                (*z).z,
                0.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                1.0f32,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_FromPosRot(
    mut pos: *const Vec3,
    mut rot: *const Quat,
) -> *mut Matrix {
    let mut x = Vec3::ZERO;
    Quat_GetAxisX(rot, &mut x);
    let mut y = Vec3::ZERO;
    Quat_GetAxisY(rot, &mut y);
    let mut z = Vec3::ZERO;
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
                0.0f32,
                0.0f32,
                0.0f32,
                1.0f32,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_FromPosRotScale(
    mut pos: *const Vec3,
    mut rot: *const Quat,
    mut scale: f32,
) -> *mut Matrix {
    let mut x = Vec3::ZERO;
    Quat_GetAxisX(rot, &mut x);
    let mut y = Vec3::ZERO;
    Quat_GetAxisY(rot, &mut y);
    let mut z = Vec3::ZERO;
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
                0.0f32,
                0.0f32,
                0.0f32,
                1.0f32,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_FromPosBasis(
    mut pos: *const Vec3,
    mut x: *const Vec3,
    mut y: *const Vec3,
    mut z: *const Vec3,
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
                0.0f32,
                0.0f32,
                0.0f32,
                1.0f32,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_FromQuat(mut q: *const Quat) -> *mut Matrix {
    let mut x = Vec3::ZERO;
    Quat_GetAxisX(q, &mut x);
    let mut y = Vec3::ZERO;
    Quat_GetAxisY(q, &mut y);
    let mut z = Vec3::ZERO;
    Quat_GetAxisZ(q, &mut z);
    let mut result: Matrix = {
        let mut init = Matrix {
            m: [
                x.x,
                y.x,
                z.x,
                0.0f32,
                x.y,
                y.y,
                z.y,
                0.0f32,
                x.z,
                y.z,
                z.z,
                0.0f32,
                0.0f32,
                0.0f32,
                0.0f32,
                1.0f32,
            ],
        };
        init
    };
    return Matrix_Clone(&mut result);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_ToQuat(mut this: *const Matrix, mut q: *mut Quat) {
    let mut m: *const f32 = this as *const f32;
    let mut x: Vec3 = {
        let mut init = Vec3 {
            x: *m.offset(0),
            y: *m.offset(4),
            z: *m.offset(8),
        };
        init
    };
    let mut y: Vec3 = {
        let mut init = Vec3 {
            x: *m.offset(1),
            y: *m.offset(5),
            z: *m.offset(9),
        };
        init
    };
    let mut z: Vec3 = {
        let mut init = Vec3 {
            x: *m.offset(2),
            y: *m.offset(6),
            z: *m.offset(10),
        };
        init
    };
    Quat_FromBasis(&mut x, &mut y, &mut z, q);
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_Print(mut this: *const Matrix) {
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < 4 as libc::c_int {
        let mut j: libc::c_int = 0 as libc::c_int;
        while j < 4 as libc::c_int {
            printf(
                b"%f \0" as *const u8 as *const libc::c_char,
                (*this).m[(4 as libc::c_int * i + j) as usize] as f64,
            );
            j += 1;
        }
        printf(b"\n\0" as *const u8 as *const libc::c_char);
        i += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn Matrix_ToString(mut this: *const Matrix) -> cstr {
    static mut buffer: [libc::c_char; 512] = [0; 512];
    let mut m: *const f32 = ((*this).m).as_ptr();
    snprintf(
        buffer.as_mut_ptr(),
        (::core::mem::size_of::<[libc::c_char; 512]>())
            .wrapping_div(::core::mem::size_of::<libc::c_char>())
            as libc::c_int as libc::size_t,
        b"[%+.2f, %+.2f, %+.2f, %+.2f]\n[%+.2f, %+.2f, %+.2f, %+.2f]\n[%+.2f, %+.2f, %+.2f, %+.2f]\n[%+.2f, %+.2f, %+.2f, %+.2f]\0"
            as *const u8 as *const libc::c_char,
        *m.offset(0) as f64,
        *m.offset(1) as f64,
        *m.offset(2) as f64,
        *m.offset(3) as f64,
        *m.offset(4) as f64,
        *m.offset(5) as f64,
        *m.offset(6) as f64,
        *m.offset(7) as f64,
        *m.offset(8) as f64,
        *m.offset(9) as f64,
        *m.offset(10) as f64,
        *m.offset(11) as f64,
        *m.offset(12) as f64,
        *m.offset(13) as f64,
        *m.offset(14) as f64,
        *m.offset(15) as f64,
    );
    return buffer.as_mut_ptr() as cstr;
}
