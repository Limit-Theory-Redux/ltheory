use tracing::info;

use crate::math::*;

pub use glam::{Mat3, Mat4};

// glam::Mat4 is a column-major matrix.
pub type Matrix = Mat4;

pub trait MatrixExtensions {
    fn get_forward(&self) -> Vec3;
    fn get_right(&self) -> Vec3;
    fn get_up(&self) -> Vec3;
    fn get_translation(&self) -> Vec3;
}

impl MatrixExtensions for Matrix {
    fn get_forward(&self) -> Vec3 {
        -self.z_axis.truncate()
    }

    fn get_right(&self) -> Vec3 {
        self.x_axis.truncate()
    }

    fn get_up(&self) -> Vec3 {
        self.y_axis.truncate()
    }

    fn get_translation(&self) -> Vec3 {
        self.w_axis.truncate()
    }
}

#[no_mangle]
pub extern "C" fn Matrix_Clone(this: &Matrix) -> Box<Matrix> {
    Box::new(*this)
}

#[no_mangle]
pub extern "C" fn Matrix_Free(_: Option<Box<Matrix>>) {}

#[no_mangle]
pub extern "C" fn Matrix_Equal(a: &Matrix, b: &Matrix) -> bool {
    *a == *b
}

#[no_mangle]
pub extern "C" fn Matrix_ApproximatelyEqual(a: &Matrix, b: &Matrix) -> bool {
    for row in 0..3 {
        for col in 0..3 {
            if !Float_ApproximatelyEqualf(a.col(col)[row], b.col(col)[row]) {
                return false;
            }
        }
    }
    true
}

#[no_mangle]
pub extern "C" fn Matrix_Inverse(this: &Matrix) -> Box<Matrix> {
    Box::new(this.inverse())
}

#[no_mangle]
pub extern "C" fn Matrix_InverseTranspose(this: &Matrix) -> Box<Matrix> {
    Box::new(this.inverse().transpose())
}

#[no_mangle]
pub extern "C" fn Matrix_Sum(a: &Matrix, b: &Matrix) -> Box<Matrix> {
    Box::new(*a + *b)
}

#[no_mangle]
pub extern "C" fn Matrix_Transpose(this: &Matrix) -> Box<Matrix> {
    Box::new(this.transpose())
}

#[no_mangle]
pub extern "C" fn Matrix_IInverse(this: &mut Matrix) {
    *this = this.inverse();
}

#[no_mangle]
pub extern "C" fn Matrix_IScale(this: &mut Matrix, scale: f32) {
    *this *= Mat4::from_scale(Vec3::splat(scale))
}

#[no_mangle]
pub extern "C" fn Matrix_ITranspose(this: &mut Matrix) {
    *this = this.transpose();
}

#[no_mangle]
pub extern "C" fn Matrix_Identity() -> Box<Matrix> {
    Box::new(Matrix::IDENTITY)
}

#[no_mangle]
pub extern "C" fn Matrix_LookAt(pos: &Vec3, at: &Vec3, up: &Vec3) -> Box<Matrix> {
    Box::new(Matrix::look_at_rh(*pos, *at, *up))
}

#[no_mangle]
pub extern "C" fn Matrix_LookUp(pos: &Vec3, look: &Vec3, up: &Vec3) -> Box<Matrix> {
    // The equvalent function in glam would be:
    // Matrix::look_to_rh(*pos, *look, *up).inverse()
    //
    // but as inversing a matrix is expensive, compute the "look to" camera matrix directly.
    let f: Vec3 = look.normalize();
    let s: Vec3 = Vec3::cross(f, *up).normalize();
    let u: Vec3 = Vec3::cross(s, f);
    Box::new(Matrix::from_cols(
        s.extend(0.0),
        u.extend(0.0),
        -f.extend(0.0),
        pos.extend(1.0),
    ))
}

#[no_mangle]
pub extern "C" fn Matrix_Perspective(
    degrees_fovy: f32,
    aspect: f32,
    n: f32,
    f: f32,
) -> Box<Matrix> {
    Box::new(Matrix::perspective_rh_gl(
        f32::to_radians(degrees_fovy),
        aspect,
        n,
        f,
    ))
}

#[no_mangle]
pub extern "C" fn Matrix_Product(a: &Matrix, b: &Matrix) -> Box<Matrix> {
    Box::new(a.mul_mat4(b))
}

#[no_mangle]
pub extern "C" fn Matrix_RotationX(rads: f32) -> Box<Matrix> {
    Box::new(Matrix::from_rotation_x(rads))
}

#[no_mangle]
pub extern "C" fn Matrix_RotationY(rads: f32) -> Box<Matrix> {
    Box::new(Matrix::from_rotation_y(rads))
}

#[no_mangle]
pub extern "C" fn Matrix_RotationZ(rads: f32) -> Box<Matrix> {
    Box::new(Matrix::from_rotation_z(rads))
}

#[no_mangle]
pub extern "C" fn Matrix_Scaling(sx: f32, sy: f32, sz: f32) -> Box<Matrix> {
    Box::new(Matrix::from_scale(Vec3::new(sx, sy, sz)))
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
    Box::new(Matrix::from_scale_rotation_translation(
        Vec3::new(sx, sy, sz),
        Quat::from_euler(glam::EulerRot::ZYX, rr, ry, rp),
        Vec3::new(tx, ty, tz),
    ))
}

#[no_mangle]
pub extern "C" fn Matrix_Translation(tx: f32, ty: f32, tz: f32) -> Box<Matrix> {
    Box::new(Matrix::from_translation(Vec3::new(tx, ty, tz)))
}

#[no_mangle]
pub extern "C" fn Matrix_YawPitchRoll(yaw: f32, pitch: f32, roll: f32) -> Box<Matrix> {
    Box::new(Matrix::from_quat(Quat::from_euler(
        glam::EulerRot::ZYX,
        roll,
        yaw,
        pitch,
    )))
}

#[no_mangle]
pub extern "C" fn Matrix_MulBox(this: &Matrix, out: &mut Box3, in_0: &Box3) {
    let corners: [Vec3; 8] = [
        Vec3::new(in_0.lower.x, in_0.lower.y, in_0.lower.z),
        Vec3::new(in_0.upper.x, in_0.lower.y, in_0.lower.z),
        Vec3::new(in_0.lower.x, in_0.upper.y, in_0.lower.z),
        Vec3::new(in_0.upper.x, in_0.upper.y, in_0.lower.z),
        Vec3::new(in_0.lower.x, in_0.lower.y, in_0.upper.z),
        Vec3::new(in_0.upper.x, in_0.lower.y, in_0.upper.z),
        Vec3::new(in_0.lower.x, in_0.upper.y, in_0.upper.z),
        Vec3::new(in_0.upper.x, in_0.upper.y, in_0.upper.z),
    ];

    out.lower = this.transform_point3(corners[0]);
    out.upper = out.lower;
    corners.iter().skip(1).for_each(|corner| {
        let result = this.transform_point3(*corner);
        out.lower = Vec3::min(out.lower, result);
        out.upper = Vec3::max(out.upper, result);
    });
}

#[no_mangle]
pub extern "C" fn Matrix_MulDir(this: &Matrix, out: &mut Vec3, x: f32, y: f32, z: f32) {
    *out = this.transform_vector3(Vec3::new(x, y, z));
}

#[no_mangle]
pub extern "C" fn Matrix_MulPoint(this: &Matrix, out: &mut Vec3, x: f32, y: f32, z: f32) {
    *out = this.transform_point3(Vec3::new(x, y, z));
}

#[no_mangle]
pub extern "C" fn Matrix_MulVec(this: &Matrix, out: &mut Vec4, x: f32, y: f32, z: f32, w: f32) {
    *out = this.mul_vec4(Vec4::new(x, y, z, w));
}

#[no_mangle]
pub extern "C" fn Matrix_GetForward(this: &Matrix, out: &mut Vec3) {
    *out = this.get_forward();
}

#[no_mangle]
pub extern "C" fn Matrix_GetRight(this: &Matrix, out: &mut Vec3) {
    *out = this.get_right();
}

#[no_mangle]
pub extern "C" fn Matrix_GetUp(this: &Matrix, out: &mut Vec3) {
    *out = this.get_up();
}

#[no_mangle]
pub extern "C" fn Matrix_GetPos(this: &Matrix, out: &mut Vec3) {
    *out = this.get_translation();
}

#[no_mangle]
pub extern "C" fn Matrix_GetRow(this: &Matrix, out: &mut Vec4, row: i32) {
    *out = this.row(row as usize);
}

#[no_mangle]
pub extern "C" fn Matrix_FromBasis(x: &Vec3, y: &Vec3, z: &Vec3) -> Box<Matrix> {
    Box::new(Matrix::from_mat3(Mat3::from_cols(*x, *y, *z)))
}

#[no_mangle]
pub extern "C" fn Matrix_FromPosRot(pos: &Vec3, rot: &Quat) -> Box<Matrix> {
    Box::new(Matrix::from_rotation_translation(*rot, *pos))
}

#[no_mangle]
pub extern "C" fn Matrix_FromPosRotScale(pos: &Vec3, rot: &Quat, scale: f32) -> Box<Matrix> {
    Box::new(Matrix::from_scale_rotation_translation(
        Vec3::splat(scale),
        *rot,
        *pos,
    ))
}

#[no_mangle]
pub extern "C" fn Matrix_FromPosBasis(pos: &Vec3, x: &Vec3, y: &Vec3, z: &Vec3) -> Box<Matrix> {
    let mut mat_from_basis = Box::new(Matrix::from_mat3(Mat3::from_cols(*x, *y, *z)));
    mat_from_basis.w_axis.x = pos.x;
    mat_from_basis.w_axis.y = pos.y;
    mat_from_basis.w_axis.z = pos.z;
    mat_from_basis
}

#[no_mangle]
pub extern "C" fn Matrix_FromQuat(q: &Quat) -> Box<Matrix> {
    Box::new(Matrix::from_quat(*q))
}

#[no_mangle]
pub extern "C" fn Matrix_ToQuat(this: &Matrix, q: &mut Quat) {
    *q = Quat::from_mat4(this);
}

#[no_mangle]
pub extern "C" fn Matrix_Print(this: &Matrix) {
    for r in 0..4 {
        let row = this.row(r);
        let s = row.as_ref().map(|elem| format!("{elem}"));
        info!("{}", s.join(" "));
    }
}

#[no_mangle]
pub extern "C" fn Matrix_ToString(this: &Matrix) -> *const libc::c_char {
    internal::static_string!(this.to_string())
}
