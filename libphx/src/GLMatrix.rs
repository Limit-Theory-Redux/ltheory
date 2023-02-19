use ::libc;
use glam::Vec3;
use glam::DVec3;
use crate::internal::Memory::*;
extern "C" {
    pub type Matrix;
    fn tan(_: libc::c_double) -> libc::c_double;
    fn sqrt(_: libc::c_double) -> libc::c_double;
    fn Matrix_Identity() -> *mut Matrix;
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

pub type GLdouble = libc::c_double;
pub type GLfloat = libc::c_float;
pub type GLenum = libc::c_uint;
pub type GLint = libc::c_int;

#[inline]
unsafe extern "C" fn Sqrt(mut t: libc::c_double) -> libc::c_double {
    return sqrt(t);
}
#[inline]
unsafe extern "C" fn Tan(mut t: libc::c_double) -> libc::c_double {
    return tan(t);
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Clear() {
    glLoadIdentity();
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Load(mut matrix: *mut Matrix) {
    let mut m: *mut libc::c_float = matrix as *mut libc::c_float;
    let mut transpose: [libc::c_float; 16] = [
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
    let mut m: [libc::c_double; 16] = [
        x.x,
        y.x,
        -z.x,
        0 as libc::c_int as libc::c_double,
        x.y,
        y.y,
        -z.y,
        0 as libc::c_int as libc::c_double,
        x.z,
        y.z,
        -z.z,
        0 as libc::c_int as libc::c_double,
        0 as libc::c_int as libc::c_double,
        0 as libc::c_int as libc::c_double,
        0 as libc::c_int as libc::c_double,
        1 as libc::c_int as libc::c_double,
    ];
    glMultMatrixd(m.as_mut_ptr());
    glTranslated(-(*eye).x, -(*eye).y, -(*eye).z);
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_ModeP() {
    glMatrixMode(0x1701 as libc::c_int as GLenum);
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_ModeWV() {
    glMatrixMode(0x1700 as libc::c_int as GLenum);
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Mult(mut matrix: *mut Matrix) {
    let mut m: *mut libc::c_float = matrix as *mut libc::c_float;
    let mut transpose: [libc::c_float; 16] = [
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
    mut fovy: libc::c_double,
    mut aspect: libc::c_double,
    mut z0: libc::c_double,
    mut z1: libc::c_double,
) {
    let mut rads: libc::c_double = 3.14159265f32 as libc::c_double * fovy / 360.0f64;
    let mut cot: libc::c_double = 1.0f64 / Tan(rads);
    let mut dz: libc::c_double = z1 - z0;
    let mut nf: libc::c_double = -2.0f64 * (z0 * z1) / dz;
    let mut m: [libc::c_double; 16] = [
        cot / aspect,
        0 as libc::c_int as libc::c_double,
        0 as libc::c_int as libc::c_double,
        0 as libc::c_int as libc::c_double,
        0 as libc::c_int as libc::c_double,
        cot,
        0 as libc::c_int as libc::c_double,
        0 as libc::c_int as libc::c_double,
        0 as libc::c_int as libc::c_double,
        0 as libc::c_int as libc::c_double,
        -(z0 + z1) / dz,
        -1.0f64,
        0 as libc::c_int as libc::c_double,
        0 as libc::c_int as libc::c_double,
        nf,
        0 as libc::c_int as libc::c_double,
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
    glGetIntegerv(0xba0 as libc::c_int as GLenum, &mut matrixMode);
    match matrixMode {
        5888 => {
            matrixMode = 0xba6 as libc::c_int;
        }
        5889 => {
            matrixMode = 0xba7 as libc::c_int;
        }
        6144 | 5890 | _ => return 0 as *mut Matrix,
    }
    let mut matrix: *mut Matrix = Matrix_Identity();
    glGetFloatv(matrixMode as GLenum, matrix as *mut libc::c_float);
    return matrix;
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_RotateX(mut angle: libc::c_double) {
    glRotated(
        angle,
        1 as libc::c_int as GLdouble,
        0 as libc::c_int as GLdouble,
        0 as libc::c_int as GLdouble,
    );
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_RotateY(mut angle: libc::c_double) {
    glRotated(
        angle,
        0 as libc::c_int as GLdouble,
        1 as libc::c_int as GLdouble,
        0 as libc::c_int as GLdouble,
    );
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_RotateZ(mut angle: libc::c_double) {
    glRotated(
        angle,
        0 as libc::c_int as GLdouble,
        0 as libc::c_int as GLdouble,
        1 as libc::c_int as GLdouble,
    );
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Scale(
    mut x: libc::c_double,
    mut y: libc::c_double,
    mut z: libc::c_double,
) {
    glScaled(x, y, z);
}
#[no_mangle]
pub unsafe extern "C" fn GLMatrix_Translate(
    mut x: libc::c_double,
    mut y: libc::c_double,
    mut z: libc::c_double,
) {
    glTranslated(x, y, z);
}
