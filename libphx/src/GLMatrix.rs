use crate::internal::Memory::*;
use crate::Matrix::*;
use glam::DVec3;
use glam::Vec3;
use libc;

extern "C" {
    fn tan(_: f64) -> f64;
    fn sqrt(_: f64) -> f64;
    fn glGetFloatv(pname: GLenum, params: *mut GLfloat);
    fn glGetIntegerv(pname: GLenum, params: *mut GLint);
    fn glLoadIdentity();
    fn glLoadMatrixf(m: *const GLfloat);
    fn glMatrixMode(mode: GLenum);
    fn glMultMatrixd(m: *const GLdouble);
    fn glMultMatrixf(m: *const GLfloat);
    fn glPopMatrix();
    fn glPushMatrix();
    fn glRotated(angle: GLdouble, x: GLdouble, y: GLdouble, z: GLdouble);
    fn glScaled(x: GLdouble, y: GLdouble, z: GLdouble);
    fn glTranslated(x: GLdouble, y: GLdouble, z: GLdouble);
}

pub type GLdouble = f64;
pub type GLfloat = f32;
pub type GLenum = u32;
pub type GLint = i32;

#[inline]
unsafe extern "C" fn Sqrt(mut t: f64) -> f64 {
    return sqrt(t);
}
#[inline]
unsafe extern "C" fn Tan(mut t: f64) -> f64 {
    return tan(t);
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Clear() {
    glLoadIdentity();
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
    glLoadMatrixf(transpose.as_mut_ptr());
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
        0 as i32 as f64,
        x.y,
        y.y,
        -z.y,
        0 as i32 as f64,
        x.z,
        y.z,
        -z.z,
        0 as i32 as f64,
        0 as i32 as f64,
        0 as i32 as f64,
        0 as i32 as f64,
        1 as i32 as f64,
    ];
    glMultMatrixd(m.as_mut_ptr());
    glTranslated(-(*eye).x, -(*eye).y, -(*eye).z);
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_ModeP() {
    glMatrixMode(0x1701 as i32 as GLenum);
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_ModeWV() {
    glMatrixMode(0x1700 as i32 as GLenum);
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
    glMultMatrixf(transpose.as_mut_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Perspective(
    mut fovy: f64,
    mut aspect: f64,
    mut z0: f64,
    mut z1: f64,
) {
    let mut rads: f64 = 3.14159265f32 as f64 * fovy / 360.0f64;
    let mut cot: f64 = 1.0f64 / Tan(rads);
    let mut dz: f64 = z1 - z0;
    let mut nf: f64 = -2.0f64 * (z0 * z1) / dz;
    let mut m: [f64; 16] = [
        cot / aspect,
        0 as i32 as f64,
        0 as i32 as f64,
        0 as i32 as f64,
        0 as i32 as f64,
        cot,
        0 as i32 as f64,
        0 as i32 as f64,
        0 as i32 as f64,
        0 as i32 as f64,
        -(z0 + z1) / dz,
        -1.0f64,
        0 as i32 as f64,
        0 as i32 as f64,
        nf,
        0 as i32 as f64,
    ];
    glMultMatrixd(m.as_mut_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Pop() {
    glPopMatrix();
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Push() {
    glPushMatrix();
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_PushClear() {
    glPushMatrix();
    glLoadIdentity();
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Get() -> *mut Matrix {
    let mut matrixMode: GLint = 0;
    glGetIntegerv(0xba0 as i32 as GLenum, &mut matrixMode);
    match matrixMode {
        5888 => {
            matrixMode = 0xba6 as i32;
        }
        5889 => {
            matrixMode = 0xba7 as i32;
        }
        6144 | 5890 | _ => return 0 as *mut Matrix,
    }
    let mut matrix: *mut Matrix = Matrix_Identity();
    glGetFloatv(matrixMode as GLenum, matrix as *mut f32);
    return matrix;
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_RotateX(mut angle: f64) {
    glRotated(
        angle,
        1 as i32 as GLdouble,
        0 as i32 as GLdouble,
        0 as i32 as GLdouble,
    );
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_RotateY(mut angle: f64) {
    glRotated(
        angle,
        0 as i32 as GLdouble,
        1 as i32 as GLdouble,
        0 as i32 as GLdouble,
    );
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_RotateZ(mut angle: f64) {
    glRotated(
        angle,
        0 as i32 as GLdouble,
        0 as i32 as GLdouble,
        1 as i32 as GLdouble,
    );
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Scale(mut x: f64, mut y: f64, mut z: f64) {
    glScaled(x, y, z);
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Translate(mut x: f64, mut y: f64, mut z: f64) {
    glTranslated(x, y, z);
}
