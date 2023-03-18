use crate::internal::Memory::*;
use crate::Common::*;
use crate::Math::DVec3;
use crate::Math::Vec3;
use crate::Matrix::*;
use crate::GL::gl;
use libc;

pub type GLdouble = f64;
pub type GLfloat = f32;
pub type GLenum = u32;
pub type GLint = i32;

#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Clear() {
    gl::LoadIdentity();
}

#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Load(mut matrix: *mut Matrix) {
    let mut m: *mut f32 = matrix as *mut f32;
    let mut transpose: [f32; 16] = [
        *m.offset(0),
        *m.offset(4),
        *m.offset(8),
        *m.offset(12),
        *m.offset(1),
        *m.offset(5),
        *m.offset(9),
        *m.offset(13),
        *m.offset(2),
        *m.offset(6),
        *m.offset(10),
        *m.offset(14),
        *m.offset(3),
        *m.offset(7),
        *m.offset(11),
        *m.offset(15),
    ];
    gl::LoadMatrixf(transpose.as_mut_ptr());
}

#[no_mangle]
pub unsafe extern "C" fn GLMatrix_LookAt(
    mut eye: *const DVec3,
    mut at: *const DVec3,
    mut up: *const DVec3,
) {
    let mut z = (*at - *eye).normalize();
    let mut x = DVec3::cross(z, (*up).normalize()).normalize();
    let mut y = DVec3::cross(x, z);
    let mut m: [f64; 16] = [
        x.x,
        y.x,
        -z.x,
        0_i32 as f64,
        x.y,
        y.y,
        -z.y,
        0_i32 as f64,
        x.z,
        y.z,
        -z.z,
        0_i32 as f64,
        0_i32 as f64,
        0_i32 as f64,
        0_i32 as f64,
        1_f64,
    ];
    gl::MultMatrixd(m.as_mut_ptr());
    gl::Translated(-(*eye).x, -(*eye).y, -(*eye).z);
}

#[no_mangle]
pub unsafe extern "C" fn GLMatrix_ModeP() {
    gl::MatrixMode(0x1701_i32 as GLenum);
}

#[no_mangle]
pub unsafe extern "C" fn GLMatrix_ModeWV() {
    gl::MatrixMode(0x1700_i32 as GLenum);
}

#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Mult(mut matrix: *mut Matrix) {
    let mut m: *mut f32 = matrix as *mut f32;
    let mut transpose: [f32; 16] = [
        *m.offset(0),
        *m.offset(4),
        *m.offset(8),
        *m.offset(12),
        *m.offset(1),
        *m.offset(5),
        *m.offset(9),
        *m.offset(13),
        *m.offset(2),
        *m.offset(6),
        *m.offset(10),
        *m.offset(14),
        *m.offset(3),
        *m.offset(7),
        *m.offset(11),
        *m.offset(15),
    ];
    gl::MultMatrixf(transpose.as_mut_ptr());
}

#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Perspective(
    mut fovy: f64,
    mut aspect: f64,
    mut z0: f64,
    mut z1: f64,
) {
    let mut rads: f64 = std::f32::consts::PI as f64 * fovy / 360.0f64;
    let mut cot: f64 = 1.0f64 / f64::tan(rads);
    let mut dz: f64 = z1 - z0;
    let mut nf: f64 = -2.0f64 * (z0 * z1) / dz;
    let mut m: [f64; 16] = [
        cot / aspect,
        0_i32 as f64,
        0_i32 as f64,
        0_i32 as f64,
        0_i32 as f64,
        cot,
        0_i32 as f64,
        0_i32 as f64,
        0_i32 as f64,
        0_i32 as f64,
        -(z0 + z1) / dz,
        -1.0f64,
        0_i32 as f64,
        0_i32 as f64,
        nf,
        0_i32 as f64,
    ];
    gl::MultMatrixd(m.as_mut_ptr());
}

#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Pop() {
    gl::PopMatrix();
}

#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Push() {
    gl::PushMatrix();
}

#[no_mangle]
pub unsafe extern "C" fn GLMatrix_PushClear() {
    gl::PushMatrix();
    gl::LoadIdentity();
}

#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Get() -> *mut Matrix {
    let mut matrixMode: GLint = 0;
    gl::GetIntegerv(0xba0_i32 as GLenum, &mut matrixMode);
    match matrixMode {
        5888 => {
            matrixMode = 0xba6_i32;
        }
        5889 => {
            matrixMode = 0xba7_i32;
        }
        6144 | 5890 | _ => return std::ptr::null_mut(),
    }
    let mut matrix: *mut Matrix = Matrix_Identity();
    gl::GetFloatv(matrixMode as GLenum, matrix as *mut f32);
    matrix
}

#[no_mangle]
pub unsafe extern "C" fn GLMatrix_RotateX(mut angle: f64) {
    gl::Rotated(
        angle,
        1_i32 as GLdouble,
        0_i32 as GLdouble,
        0_i32 as GLdouble,
    );
}

#[no_mangle]
pub unsafe extern "C" fn GLMatrix_RotateY(mut angle: f64) {
    gl::Rotated(
        angle,
        0_i32 as GLdouble,
        1_i32 as GLdouble,
        0_i32 as GLdouble,
    );
}

#[no_mangle]
pub unsafe extern "C" fn GLMatrix_RotateZ(mut angle: f64) {
    gl::Rotated(
        angle,
        0_i32 as GLdouble,
        0_i32 as GLdouble,
        1_i32 as GLdouble,
    );
}

#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Scale(mut x: f64, mut y: f64, mut z: f64) {
    gl::Scaled(x, y, z);
}

#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Translate(mut x: f64, mut y: f64, mut z: f64) {
    gl::Translated(x, y, z);
}
