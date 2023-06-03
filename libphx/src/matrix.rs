use crate::common::*;
use crate::internal::*;
use crate::math::Box3;
use crate::math::Vec3;
use crate::math::Vec4;
use crate::quat::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Matrix {
    pub m: [f32; 16],
}

impl Matrix {
    /// All zeroes.
    pub const ZERO: Self = Matrix { m: [0.; 16] };

    /// Identity.
    pub const IDENTITY: Self = Matrix {
        m: [
            1.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 1.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 1.0f32,
            0.0f32, 0.0f32, 0.0f32, 0.0f32, 1.0f32,
        ],
    };

    pub fn from_slice(slice: &[f32]) -> Matrix {
        Matrix {
            m: slice.try_into().expect("slice with incorrect length"),
        }
    }

    pub fn look_at(pos: &Vec3, at: &Vec3, up: &Vec3) -> Matrix {
        let z: Vec3 = (*pos - *at).normalize();
        let x: Vec3 = Vec3::cross(*up, z).normalize();
        let y: Vec3 = Vec3::cross(z, x);
        Matrix {
            m: [
                x.x, y.x, z.x, pos.x, x.y, y.y, z.y, pos.y, x.z, y.z, z.z, pos.z, 0.0f32, 0.0f32,
                0.0f32, 1.0f32,
            ],
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "[{:+.2}, {:+.2}, {:+.2}, {:+.2}]\n\
                [{:+.2}, {:+.2}, {:+.2}, {:+.2}]\n\
                [{:+.2}, {:+.2}, {:+.2}, {:+.2}]\n\
                [{:+.2}, {:+.2}, {:+.2}, {:+.2}]",
            self.m[0],
            self.m[1],
            self.m[2],
            self.m[3],
            self.m[4],
            self.m[5],
            self.m[6],
            self.m[7],
            self.m[8],
            self.m[9],
            self.m[10],
            self.m[11],
            self.m[12],
            self.m[13],
            self.m[14],
            self.m[15],
        )
    }

    pub fn scale(&mut self, scale: f32) {
        let m: *mut f32 = (self.m).as_mut_ptr();
        unsafe {
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
    }

    pub fn scaled(&self, scale: f32) -> Matrix {
        let mut matrix = self.clone();
        matrix.scale(scale);
        matrix
    }

    pub fn inverted(&self) -> Matrix {
        let mut out = Matrix::ZERO;
        let src: *const f32 = self.m.as_ptr();
        let dst: *mut f32 = out.m.as_mut_ptr();
        unsafe {
            *dst.offset(0) = *src.offset(5) * *src.offset(10) * *src.offset(15)
                - *src.offset(5) * *src.offset(11) * *src.offset(14)
                - *src.offset(9) * *src.offset(6) * *src.offset(15)
                + *src.offset(9) * *src.offset(7) * *src.offset(14)
                + *src.offset(13) * *src.offset(6) * *src.offset(11)
                - *src.offset(13) * *src.offset(7) * *src.offset(10);
            *dst.offset(4) = -*src.offset(4) * *src.offset(10) * *src.offset(15)
                + *src.offset(4) * *src.offset(11) * *src.offset(14)
                + *src.offset(8) * *src.offset(6) * *src.offset(15)
                - *src.offset(8) * *src.offset(7) * *src.offset(14)
                - *src.offset(12) * *src.offset(6) * *src.offset(11)
                + *src.offset(12) * *src.offset(7) * *src.offset(10);
            *dst.offset(8) = *src.offset(4) * *src.offset(9) * *src.offset(15)
                - *src.offset(4) * *src.offset(11) * *src.offset(13)
                - *src.offset(8) * *src.offset(5) * *src.offset(15)
                + *src.offset(8) * *src.offset(7) * *src.offset(13)
                + *src.offset(12) * *src.offset(5) * *src.offset(11)
                - *src.offset(12) * *src.offset(7) * *src.offset(9);
            *dst.offset(12) = -*src.offset(4) * *src.offset(9) * *src.offset(14)
                + *src.offset(4) * *src.offset(10) * *src.offset(13)
                + *src.offset(8) * *src.offset(5) * *src.offset(14)
                - *src.offset(8) * *src.offset(6) * *src.offset(13)
                - *src.offset(12) * *src.offset(5) * *src.offset(10)
                + *src.offset(12) * *src.offset(6) * *src.offset(9);
            *dst.offset(1) = -*src.offset(1) * *src.offset(10) * *src.offset(15)
                + *src.offset(1) * *src.offset(11) * *src.offset(14)
                + *src.offset(9) * *src.offset(2) * *src.offset(15)
                - *src.offset(9) * *src.offset(3) * *src.offset(14)
                - *src.offset(13) * *src.offset(2) * *src.offset(11)
                + *src.offset(13) * *src.offset(3) * *src.offset(10);
            *dst.offset(5) = *src.offset(0) * *src.offset(10) * *src.offset(15)
                - *src.offset(0) * *src.offset(11) * *src.offset(14)
                - *src.offset(8) * *src.offset(2) * *src.offset(15)
                + *src.offset(8) * *src.offset(3) * *src.offset(14)
                + *src.offset(12) * *src.offset(2) * *src.offset(11)
                - *src.offset(12) * *src.offset(3) * *src.offset(10);
            *dst.offset(9) = -*src.offset(0) * *src.offset(9) * *src.offset(15)
                + *src.offset(0) * *src.offset(11) * *src.offset(13)
                + *src.offset(8) * *src.offset(1) * *src.offset(15)
                - *src.offset(8) * *src.offset(3) * *src.offset(13)
                - *src.offset(12) * *src.offset(1) * *src.offset(11)
                + *src.offset(12) * *src.offset(3) * *src.offset(9);
            *dst.offset(13) = *src.offset(0) * *src.offset(9) * *src.offset(14)
                - *src.offset(0) * *src.offset(10) * *src.offset(13)
                - *src.offset(8) * *src.offset(1) * *src.offset(14)
                + *src.offset(8) * *src.offset(2) * *src.offset(13)
                + *src.offset(12) * *src.offset(1) * *src.offset(10)
                - *src.offset(12) * *src.offset(2) * *src.offset(9);
            *dst.offset(2) = *src.offset(1) * *src.offset(6) * *src.offset(15)
                - *src.offset(1) * *src.offset(7) * *src.offset(14)
                - *src.offset(5) * *src.offset(2) * *src.offset(15)
                + *src.offset(5) * *src.offset(3) * *src.offset(14)
                + *src.offset(13) * *src.offset(2) * *src.offset(7)
                - *src.offset(13) * *src.offset(3) * *src.offset(6);
            *dst.offset(6) = -*src.offset(0) * *src.offset(6) * *src.offset(15)
                + *src.offset(0) * *src.offset(7) * *src.offset(14)
                + *src.offset(4) * *src.offset(2) * *src.offset(15)
                - *src.offset(4) * *src.offset(3) * *src.offset(14)
                - *src.offset(12) * *src.offset(2) * *src.offset(7)
                + *src.offset(12) * *src.offset(3) * *src.offset(6);
            *dst.offset(10) = *src.offset(0) * *src.offset(5) * *src.offset(15)
                - *src.offset(0) * *src.offset(7) * *src.offset(13)
                - *src.offset(4) * *src.offset(1) * *src.offset(15)
                + *src.offset(4) * *src.offset(3) * *src.offset(13)
                + *src.offset(12) * *src.offset(1) * *src.offset(7)
                - *src.offset(12) * *src.offset(3) * *src.offset(5);
            *dst.offset(14) = -*src.offset(0) * *src.offset(5) * *src.offset(14)
                + *src.offset(0) * *src.offset(6) * *src.offset(13)
                + *src.offset(4) * *src.offset(1) * *src.offset(14)
                - *src.offset(4) * *src.offset(2) * *src.offset(13)
                - *src.offset(12) * *src.offset(1) * *src.offset(6)
                + *src.offset(12) * *src.offset(2) * *src.offset(5);
            *dst.offset(3) = -*src.offset(1) * *src.offset(6) * *src.offset(11)
                + *src.offset(1) * *src.offset(7) * *src.offset(10)
                + *src.offset(5) * *src.offset(2) * *src.offset(11)
                - *src.offset(5) * *src.offset(3) * *src.offset(10)
                - *src.offset(9) * *src.offset(2) * *src.offset(7)
                + *src.offset(9) * *src.offset(3) * *src.offset(6);
            *dst.offset(7) = *src.offset(0) * *src.offset(6) * *src.offset(11)
                - *src.offset(0) * *src.offset(7) * *src.offset(10)
                - *src.offset(4) * *src.offset(2) * *src.offset(11)
                + *src.offset(4) * *src.offset(3) * *src.offset(10)
                + *src.offset(8) * *src.offset(2) * *src.offset(7)
                - *src.offset(8) * *src.offset(3) * *src.offset(6);
            *dst.offset(11) = -*src.offset(0) * *src.offset(5) * *src.offset(11)
                + *src.offset(0) * *src.offset(7) * *src.offset(9)
                + *src.offset(4) * *src.offset(1) * *src.offset(11)
                - *src.offset(4) * *src.offset(3) * *src.offset(9)
                - *src.offset(8) * *src.offset(1) * *src.offset(7)
                + *src.offset(8) * *src.offset(3) * *src.offset(5);
            *dst.offset(15) = *src.offset(0) * *src.offset(5) * *src.offset(10)
                - *src.offset(0) * *src.offset(6) * *src.offset(9)
                - *src.offset(4) * *src.offset(1) * *src.offset(10)
                + *src.offset(4) * *src.offset(2) * *src.offset(9)
                + *src.offset(8) * *src.offset(1) * *src.offset(6)
                - *src.offset(8) * *src.offset(2) * *src.offset(5);
            let det: f32 = 1.0f32
                / (*src.offset(0) * *dst.offset(0)
                    + *src.offset(1) * *dst.offset(4)
                    + *src.offset(2) * *dst.offset(8)
                    + *src.offset(3) * *dst.offset(12));
            let mut i: i32 = 0;
            while i < 16 {
                *dst.offset(i as isize) *= det;
                i += 1;
            }
        }
        out
    }

    pub fn transposed(&self) -> Matrix {
        let mut out = Matrix::ZERO;
        let src: *const f32 = self.m.as_ptr();
        let dst: *mut f32 = out.m.as_mut_ptr();
        unsafe {
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
        out
    }
}

#[inline]
extern "C" fn Float_ApproximatelyEqual(x: f64, y: f64) -> bool {
    f64::abs(x - y) < 1e-3f64
}

#[no_mangle]
pub extern "C" fn Matrix_Clone(this: &Matrix) -> Box<Matrix> {
    Box::new(this.clone())
}

#[no_mangle]
pub extern "C" fn Matrix_Free(_: Option<Box<Matrix>>) {}

#[no_mangle]
pub extern "C" fn Matrix_Equal(a: &Matrix, b: &Matrix) -> bool {
    let mut i: i32 = 0;
    while i < 16 {
        if (*a).m[i as usize] != (*b).m[i as usize] {
            return false;
        }
        i += 1;
    }
    true
}

#[no_mangle]
pub extern "C" fn Matrix_ApproximatelyEqual(a: &Matrix, b: &Matrix) -> bool {
    let mut i: i32 = 0;
    while i < 16 {
        if !Float_ApproximatelyEqual((*a).m[i as usize] as f64, (*b).m[i as usize] as f64) {
            return false;
        }
        i += 1;
    }
    true
}

#[no_mangle]
pub extern "C" fn Matrix_Inverse(this: &Matrix) -> Box<Matrix> {
    Box::new(this.inverted())
}

#[no_mangle]
pub extern "C" fn Matrix_InverseTranspose(this: &Matrix) -> Box<Matrix> {
    Box::new(this.inverted().transposed())
}

#[no_mangle]
pub extern "C" fn Matrix_Sum(a: &Matrix, b: &Matrix) -> Box<Matrix> {
    let mut result: Matrix = Matrix::ZERO;
    let mut i: i32 = 0;
    while i < 16 {
        result.m[i as usize] = (*a).m[i as usize] + (*b).m[i as usize];
        i += 1;
    }
    Box::new(result)
}

#[no_mangle]
pub extern "C" fn Matrix_Transpose(this: &Matrix) -> Box<Matrix> {
    Box::new(this.transposed())
}

#[no_mangle]
pub extern "C" fn Matrix_IInverse(this: &mut Matrix) {
    *this = this.inverted();
}

#[no_mangle]
pub extern "C" fn Matrix_IScale(this: &mut Matrix, scale: f32) {
    this.scale(scale)
}

#[no_mangle]
pub extern "C" fn Matrix_ITranspose(this: &mut Matrix) {
    *this = this.transposed();
}

#[no_mangle]
pub extern "C" fn Matrix_Identity() -> Box<Matrix> {
    Box::new(Matrix::IDENTITY)
}

#[no_mangle]
pub extern "C" fn Matrix_LookAt(pos: &Vec3, at: &Vec3, up: &Vec3) -> Box<Matrix> {
    let z: Vec3 = (*pos - *at).normalize();
    let x: Vec3 = Vec3::cross(*up, z).normalize();
    let y: Vec3 = Vec3::cross(z, x);
    let result: Matrix = Matrix {
        m: [
            x.x, y.x, z.x, pos.x, x.y, y.y, z.y, pos.y, x.z, y.z, z.z, pos.z, 0.0f32, 0.0f32,
            0.0f32, 1.0f32,
        ],
    };
    Box::new(result)
}

#[no_mangle]
pub extern "C" fn Matrix_LookUp(pos: &Vec3, look: &Vec3, up: &Vec3) -> Box<Matrix> {
    let z: Vec3 = (*look * -1.0f32).normalize();
    let x: Vec3 = Vec3::cross(*up, z).normalize();
    let y: Vec3 = Vec3::cross(z, x);
    let result: Matrix = Matrix {
        m: [
            x.x, y.x, z.x, pos.x, x.y, y.y, z.y, pos.y, x.z, y.z, z.z, pos.z, 0.0f32, 0.0f32,
            0.0f32, 1.0f32,
        ],
    };
    Box::new(result)
}

#[no_mangle]
pub extern "C" fn Matrix_Perspective(degreesFovy: f32, aspect: f32, N: f32, F: f32) -> Box<Matrix> {
    let rads: f64 = (std::f32::consts::PI * degreesFovy) as f64 / 360.0f64;
    let cot: f64 = 1.0f64 / f64::tan(rads);
    let result: Matrix = Matrix {
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
            (2.0f64 * (F * N) as f64 / (N - F) as f64) as f32,
            0.0f32,
            0.0f32,
            -1.0f32,
            0.0f32,
        ],
    };
    Box::new(result)
}

#[no_mangle]
pub extern "C" fn Matrix_Product(a: &Matrix, b: &Matrix) -> Box<Matrix> {
    let mut result: Matrix = Matrix::ZERO;
    let mut pResult: *mut f32 = (result.m).as_mut_ptr();
    let mut i: i32 = 0;
    while i < 4 {
        let mut j: i32 = 0;
        while j < 4 {
            let mut sum: f32 = 0.0f32;
            let mut k: i32 = 0;
            while k < 4 {
                sum += (*a).m[(4 * i + k) as usize] * (*b).m[(4 * k + j) as usize];
                k += 1;
            }
            unsafe {
                let fresh0 = pResult;
                pResult = pResult.offset(1);
                *fresh0 = sum;
            }
            j += 1;
        }
        i += 1;
    }
    Box::new(result)
}

#[no_mangle]
pub extern "C" fn Matrix_RotationX(rads: f32) -> Box<Matrix> {
    let c: f32 = f64::cos(rads as f64) as f32;
    let s: f32 = f64::sin(rads as f64) as f32;
    let result: Matrix = Matrix {
        m: [
            1.0f32, 0.0f32, 0.0f32, 0.0f32, 0.0f32, c, -s, 0.0f32, 0.0f32, s, c, 0.0f32, 0.0f32,
            0.0f32, 0.0f32, 1.0f32,
        ],
    };
    Box::new(result)
}

#[no_mangle]
pub extern "C" fn Matrix_RotationY(rads: f32) -> Box<Matrix> {
    let c: f32 = f64::cos(rads as f64) as f32;
    let s: f32 = f64::sin(rads as f64) as f32;
    let result: Matrix = Matrix {
        m: [
            c, 0.0f32, s, 0.0f32, 0.0f32, 1.0f32, 0.0f32, 0.0f32, -s, 0.0f32, c, 0.0f32, 0.0f32,
            0.0f32, 0.0f32, 1.0f32,
        ],
    };
    Box::new(result)
}

#[no_mangle]
pub extern "C" fn Matrix_RotationZ(rads: f32) -> Box<Matrix> {
    let c: f32 = f64::cos(rads as f64) as f32;
    let s: f32 = f64::sin(rads as f64) as f32;
    let result: Matrix = Matrix {
        m: [
            c, -s, 0.0f32, 0.0f32, s, c, 0.0f32, 0.0f32, 0.0f32, 0.0f32, 1.0f32, 0.0f32, 0.0f32,
            0.0f32, 0.0f32, 1.0f32,
        ],
    };
    Box::new(result)
}

#[no_mangle]
pub extern "C" fn Matrix_Scaling(sx: f32, sy: f32, sz: f32) -> Box<Matrix> {
    let result: Matrix = Matrix {
        m: [
            sx, 0.0f32, 0.0f32, 0.0f32, 0.0f32, sy, 0.0f32, 0.0f32, 0.0f32, 0.0f32, sz, 0.0f32,
            0.0f32, 0.0f32, 0.0f32, 1.0f32,
        ],
    };
    Box::new(result)
}

#[no_mangle]
pub extern "C" fn Matrix_SRT(
    sx: f32,
    sy: f32,
    sz: f32,
    ry: f32,
    rp: f32,
    rr: f32,
    tx: f32,
    ty: f32,
    tz: f32,
) -> Box<Matrix> {
    let S: Box<Matrix> = Matrix_Scaling(sx, sy, sz);
    let R: Box<Matrix> = Matrix_YawPitchRoll(ry, rp, rr);
    let T: Box<Matrix> = Matrix_Translation(tx, ty, tz);
    let TR: Box<Matrix> = Matrix_Product(T.as_ref(), R.as_ref());
    let TRS: Box<Matrix> = Matrix_Product(TR.as_ref(), S.as_ref());
    TRS
}

#[no_mangle]
pub extern "C" fn Matrix_Translation(tx: f32, ty: f32, tz: f32) -> Box<Matrix> {
    let result: Matrix = Matrix {
        m: [
            1.0f32, 0.0f32, 0.0f32, tx, 0.0f32, 1.0f32, 0.0f32, ty, 0.0f32, 0.0f32, 1.0f32, tz,
            0.0f32, 0.0f32, 0.0f32, 1.0f32,
        ],
    };
    Box::new(result)
}

#[no_mangle]
pub extern "C" fn Matrix_YawPitchRoll(yaw: f32, pitch: f32, roll: f32) -> Box<Matrix> {
    let ca: f32 = f64::cos(roll as f64) as f32;
    let sa: f32 = f64::sin(roll as f64) as f32;
    let cb: f32 = f64::cos(yaw as f64) as f32;
    let sb: f32 = f64::sin(yaw as f64) as f32;
    let cy: f32 = f64::cos(pitch as f64) as f32;
    let sy: f32 = f64::sin(pitch as f64) as f32;
    let result: Matrix = Matrix {
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
    Box::new(result)
}

#[no_mangle]
pub extern "C" fn Matrix_MulBox(this: &Matrix, out: &mut Box3, in_0: &Box3) {
    let corners: [Vec3; 8] = [
        Vec3 {
            x: in_0.lower.x,
            y: in_0.lower.y,
            z: in_0.lower.z,
        },
        Vec3 {
            x: in_0.upper.x,
            y: in_0.lower.y,
            z: in_0.lower.z,
        },
        Vec3 {
            x: in_0.lower.x,
            y: in_0.upper.y,
            z: in_0.lower.z,
        },
        Vec3 {
            x: in_0.upper.x,
            y: in_0.upper.y,
            z: in_0.lower.z,
        },
        Vec3 {
            x: in_0.lower.x,
            y: in_0.lower.y,
            z: in_0.upper.z,
        },
        Vec3 {
            x: in_0.upper.x,
            y: in_0.lower.y,
            z: in_0.upper.z,
        },
        Vec3 {
            x: in_0.lower.x,
            y: in_0.upper.y,
            z: in_0.upper.z,
        },
        Vec3 {
            x: in_0.upper.x,
            y: in_0.upper.y,
            z: in_0.upper.z,
        },
    ];
    let mut result = Vec3::ZERO;
    Matrix_MulPoint(this, &mut result, corners[0].x, corners[0].y, corners[0].z);
    out.lower = result;
    out.upper = result;
    let mut i: i32 = 1;
    while i < 8 {
        Matrix_MulPoint(
            this,
            &mut result,
            corners[i as usize].x,
            corners[i as usize].y,
            corners[i as usize].z,
        );
        out.lower = Vec3::min(out.lower, result);
        out.upper = Vec3::max(out.upper, result);
        i += 1;
    }
}

#[no_mangle]
pub extern "C" fn Matrix_MulDir(this: &Matrix, out: &mut Vec3, x: f32, y: f32, z: f32) {
    let m: *const f32 = (this.m).as_ptr();
    unsafe {
        out.x = *m.offset(0) * x + *m.offset(1) * y + *m.offset(2) * z;
        out.y = *m.offset(4) * x + *m.offset(5) * y + *m.offset(6) * z;
        out.z = *m.offset(8) * x + *m.offset(9) * y + *m.offset(10) * z;
    }
}

#[no_mangle]
pub extern "C" fn Matrix_MulPoint(this: &Matrix, out: &mut Vec3, x: f32, y: f32, z: f32) {
    let m: *const f32 = (this.m).as_ptr();
    unsafe {
        out.x = *m.offset(0) * x + *m.offset(1) * y + *m.offset(2) * z + *m.offset(3);
        out.y = *m.offset(4) * x + *m.offset(5) * y + *m.offset(6) * z + *m.offset(7);
        out.z = *m.offset(8) * x + *m.offset(9) * y + *m.offset(10) * z + *m.offset(11);
    }
}

#[no_mangle]
pub extern "C" fn Matrix_MulVec(this: &Matrix, out: &mut Vec4, x: f32, y: f32, z: f32, w: f32) {
    let m: *const f32 = (this.m).as_ptr();
    unsafe {
        out.x = *m.offset(0) * x + *m.offset(1) * y + *m.offset(2) * z + *m.offset(3) * w;
        out.y = *m.offset(4) * x + *m.offset(5) * y + *m.offset(6) * z + *m.offset(7) * w;
        out.z = *m.offset(8) * x + *m.offset(9) * y + *m.offset(10) * z + *m.offset(11) * w;
        out.w = *m.offset(12) * x + *m.offset(13) * y + *m.offset(14) * z + *m.offset(15) * w;
    }
}

#[no_mangle]
pub extern "C" fn Matrix_GetForward(this: &Matrix, out: &mut Vec3) {
    out.x = -this.m[2];
    out.y = -this.m[6];
    out.z = -this.m[10];
}

#[no_mangle]
pub extern "C" fn Matrix_GetRight(this: &Matrix, out: &mut Vec3) {
    out.x = this.m[0];
    out.y = this.m[4];
    out.z = this.m[8];
}

#[no_mangle]
pub extern "C" fn Matrix_GetUp(this: &Matrix, out: &mut Vec3) {
    out.x = this.m[1];
    out.y = this.m[5];
    out.z = this.m[9];
}

#[no_mangle]
pub extern "C" fn Matrix_GetPos(this: &Matrix, out: &mut Vec3) {
    out.x = this.m[3];
    out.y = this.m[7];
    out.z = this.m[11];
}

#[no_mangle]
pub extern "C" fn Matrix_GetRow(this: &Matrix, out: &mut Vec4, row: i32) {
    out.x = this.m[(4 * row + 0) as usize];
    out.y = this.m[(4 * row + 1) as usize];
    out.z = this.m[(4 * row + 2) as usize];
    out.w = this.m[(4 * row + 3) as usize];
}

#[no_mangle]
pub extern "C" fn Matrix_FromBasis(x: &Vec3, y: &Vec3, z: &Vec3) -> Box<Matrix> {
    let result: Matrix = Matrix {
        m: [
            x.x, y.x, z.x, 0.0f32, x.y, y.y, z.y, 0.0f32, x.z, y.z, z.z, 0.0f32, 0.0f32, 0.0f32,
            0.0f32, 1.0f32,
        ],
    };
    Box::new(result)
}

#[no_mangle]
pub extern "C" fn Matrix_FromPosRot(pos: &Vec3, rot: &Quat) -> Box<Matrix> {
    let mut x = Vec3::ZERO;
    Quat_GetAxisX(rot, &mut x);
    let mut y = Vec3::ZERO;
    Quat_GetAxisY(rot, &mut y);
    let mut z = Vec3::ZERO;
    Quat_GetAxisZ(rot, &mut z);
    let result: Matrix = Matrix {
        m: [
            x.x, y.x, z.x, pos.x, x.y, y.y, z.y, pos.y, x.z, y.z, z.z, pos.z, 0.0f32, 0.0f32,
            0.0f32, 1.0f32,
        ],
    };
    Box::new(result)
}

#[no_mangle]
pub extern "C" fn Matrix_FromPosRotScale(pos: &Vec3, rot: &Quat, scale: f32) -> Box<Matrix> {
    let mut x = Vec3::ZERO;
    Quat_GetAxisX(rot, &mut x);
    let mut y = Vec3::ZERO;
    Quat_GetAxisY(rot, &mut y);
    let mut z = Vec3::ZERO;
    Quat_GetAxisZ(rot, &mut z);
    let result: Matrix = Matrix {
        m: [
            scale * x.x,
            scale * y.x,
            scale * z.x,
            pos.x,
            scale * x.y,
            scale * y.y,
            scale * z.y,
            pos.y,
            scale * x.z,
            scale * y.z,
            scale * z.z,
            pos.z,
            0.0f32,
            0.0f32,
            0.0f32,
            1.0f32,
        ],
    };
    Box::new(result)
}

#[no_mangle]
pub extern "C" fn Matrix_FromPosBasis(pos: &Vec3, x: &Vec3, y: &Vec3, z: &Vec3) -> Box<Matrix> {
    let result: Matrix = Matrix {
        m: [
            x.x, y.x, z.x, pos.x, x.y, y.y, z.y, pos.y, x.z, y.z, z.z, pos.z, 0.0f32, 0.0f32,
            0.0f32, 1.0f32,
        ],
    };
    Box::new(result)
}

#[no_mangle]
pub extern "C" fn Matrix_FromQuat(q: &Quat) -> Box<Matrix> {
    let mut x = Vec3::ZERO;
    Quat_GetAxisX(q, &mut x);
    let mut y = Vec3::ZERO;
    Quat_GetAxisY(q, &mut y);
    let mut z = Vec3::ZERO;
    Quat_GetAxisZ(q, &mut z);
    let result: Matrix = Matrix {
        m: [
            x.x, y.x, z.x, 0.0f32, x.y, y.y, z.y, 0.0f32, x.z, y.z, z.z, 0.0f32, 0.0f32, 0.0f32,
            0.0f32, 1.0f32,
        ],
    };
    Box::new(result)
}

#[no_mangle]
pub extern "C" fn Matrix_ToQuat(this: &Matrix, q: &mut Quat) {
    let m: *const f32 = this as *const Matrix as *const f32;
    unsafe {
        let mut x: Vec3 = Vec3::new(*m.offset(0), *m.offset(4), *m.offset(8));
        let mut y: Vec3 = Vec3::new(*m.offset(1), *m.offset(5), *m.offset(9));
        let mut z: Vec3 = Vec3::new(*m.offset(2), *m.offset(6), *m.offset(10));
        Quat_FromBasis(&mut x, &mut y, &mut z, q);
    }
}

#[no_mangle]
pub extern "C" fn Matrix_Print(this: &Matrix) {
    let mut i: i32 = 0;
    while i < 4 {
        let mut j: i32 = 0;
        while j < 4 {
            CPrintf!("%f ", this.m[(4 * i + j) as usize] as f64);
            j += 1;
        }
        CPrintf!("\n");
        i += 1;
    }
}

#[no_mangle]
pub extern "C" fn Matrix_ToString(this: &Matrix) -> *const libc::c_char {
    static_string!(this.to_string())
}
