use glam::{Mat3, Mat4, Quat, Vec3, Vec4};
use tracing::info;

use super::{approximately_equal, Box3};

// glam::Mat4 is a column-major matrix.
#[derive(Clone)]
#[repr(C)]
pub struct Matrix(Mat4);

impl From<Mat4> for Matrix {
    fn from(value: Mat4) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for Matrix {
    type Target = Mat4;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[luajit_ffi_gen::luajit_ffi(clone = true, typedef = "float m[16];")]
impl Matrix {
    pub fn equal(&self, other: &Matrix) -> bool {
        self.0 == other.0
    }

    pub fn approximately_equal(&self, other: &Matrix) -> bool {
        for row in 0..3 {
            for col in 0..3 {
                if !approximately_equal(self.0.col(col)[row], other.0.col(col)[row]) {
                    return false;
                }
            }
        }
        true
    }

    pub fn inverse(&self) -> Self {
        Self(self.0.inverse())
    }

    pub fn inverse_transpose(&self) -> Self {
        Self(self.0.inverse().transpose())
    }

    pub fn sum(&self, other: &Matrix) -> Self {
        Self(self.0 + other.0)
    }

    pub fn transpose(&self) -> Self {
        Self(self.0.transpose())
    }

    pub fn i_inverse(&mut self) {
        self.0 = self.0.inverse();
    }

    pub fn i_scale(&mut self, scale: f32) {
        self.0 *= Mat4::from_scale(Vec3::splat(scale))
    }

    pub fn i_transpose(&mut self) {
        self.0 = self.0.transpose();
    }

    pub fn identity() -> Self {
        Self(Mat4::IDENTITY)
    }

    pub fn look_at(pos: &Vec3, at: &Vec3, up: &Vec3) -> Self {
        Self(Mat4::look_at_rh(*pos, *at, *up))
    }

    pub fn look_up(pos: &Vec3, look: &Vec3, up: &Vec3) -> Self {
        // The equvalent function in glam would be:
        // Matrix::look_to_rh(*pos, *look, *up).inverse()
        //
        // but as inversing a matrix is expensive, compute the "look to" camera matrix directly.
        let f = look.normalize();
        let s = Vec3::cross(f, *up).normalize();
        let u = Vec3::cross(s, f);

        Self(Mat4::from_cols(
            s.extend(0.0),
            u.extend(0.0),
            -f.extend(0.0),
            pos.extend(1.0),
        ))
    }

    pub fn perspective(degrees_fovy: f32, aspect: f32, n: f32, f: f32) -> Self {
        Self(Mat4::perspective_rh_gl(
            f32::to_radians(degrees_fovy),
            aspect,
            n,
            f,
        ))
    }

    pub fn product(&self, other: &Matrix) -> Self {
        Self(self.0.mul_mat4(&other.0))
    }

    pub fn rotation_x(rads: f32) -> Self {
        Self(Mat4::from_rotation_x(rads))
    }

    pub fn rotation_y(rads: f32) -> Self {
        Self(Mat4::from_rotation_y(rads))
    }

    pub fn rotation_z(rads: f32) -> Self {
        Self(Mat4::from_rotation_z(rads))
    }

    pub fn scaling(sx: f32, sy: f32, sz: f32) -> Self {
        Self(Mat4::from_scale(Vec3::new(sx, sy, sz)))
    }

    #[bind(name = "SRT")]
    pub fn srt(
        sx: f32,
        sy: f32,
        sz: f32,
        ry: f32,
        rp: f32,
        rr: f32,
        tx: f32,
        ty: f32,
        tz: f32,
    ) -> Self {
        Self(Mat4::from_scale_rotation_translation(
            Vec3::new(sx, sy, sz),
            Quat::from_euler(glam::EulerRot::ZYX, rr, ry, rp),
            Vec3::new(tx, ty, tz),
        ))
    }

    pub fn translation(tx: f32, ty: f32, tz: f32) -> Self {
        Self(Mat4::from_translation(Vec3::new(tx, ty, tz)))
    }

    pub fn yaw_pitch_roll(yaw: f32, pitch: f32, roll: f32) -> Self {
        Self(Mat4::from_quat(Quat::from_euler(
            glam::EulerRot::ZYX,
            roll,
            yaw,
            pitch,
        )))
    }

    #[bind(out_param = true)]
    pub fn mul_box(&self, in_0: &Box3) -> Box3 {
        let corners = [
            Vec3::new(in_0.lower.x, in_0.lower.y, in_0.lower.z),
            Vec3::new(in_0.upper.x, in_0.lower.y, in_0.lower.z),
            Vec3::new(in_0.lower.x, in_0.upper.y, in_0.lower.z),
            Vec3::new(in_0.upper.x, in_0.upper.y, in_0.lower.z),
            Vec3::new(in_0.lower.x, in_0.lower.y, in_0.upper.z),
            Vec3::new(in_0.upper.x, in_0.lower.y, in_0.upper.z),
            Vec3::new(in_0.lower.x, in_0.upper.y, in_0.upper.z),
            Vec3::new(in_0.upper.x, in_0.upper.y, in_0.upper.z),
        ];
        let lower = self.0.transform_point3(corners[0]);
        let mut out = Box3 {
            lower,
            upper: lower,
        };

        corners.iter().skip(1).for_each(|corner| {
            let result = self.0.transform_point3(*corner);
            out.lower = Vec3::min(out.lower, result);
            out.upper = Vec3::max(out.upper, result);
        });

        out
    }

    #[bind(out_param = true)]
    pub fn mul_dir(&self, d: &Vec3) -> Vec3 {
        self.0.transform_vector3(*d)
    }

    #[bind(out_param = true)]
    pub fn mul_point(&self, p: &Vec3) -> Vec3 {
        self.0.transform_point3(*p)
    }

    #[bind(out_param = true)]
    pub fn mul_vec(&self, v: &Vec4) -> Vec4 {
        self.0.mul_vec4(*v)
    }

    #[bind(out_param = true)]
    pub fn get_forward(&self) -> Vec3 {
        -self.0.z_axis.truncate()
    }

    #[bind(out_param = true)]
    pub fn get_right(&self) -> Vec3 {
        self.0.x_axis.truncate()
    }

    #[bind(out_param = true)]
    pub fn get_up(&self) -> Vec3 {
        self.0.y_axis.truncate()
    }

    #[bind(out_param = true)]
    pub fn get_pos(&self) -> Vec3 {
        self.0.w_axis.truncate()
    }

    #[bind(out_param = true)]
    pub fn get_row(&self, row: i32) -> Vec4 {
        self.0.row(row as usize)
    }

    pub fn from_basis(x: &Vec3, y: &Vec3, z: &Vec3) -> Self {
        Self(Mat4::from_mat3(Mat3::from_cols(*x, *y, *z)))
    }

    pub fn from_pos_rot(pos: &Vec3, rot: &Quat) -> Self {
        Self(Mat4::from_rotation_translation(*rot, *pos))
    }

    pub fn from_pos_rot_scale(pos: &Vec3, rot: &Quat, scale: f32) -> Self {
        Self(Mat4::from_scale_rotation_translation(
            Vec3::splat(scale),
            *rot,
            *pos,
        ))
    }

    pub fn from_pos_basis(pos: &Vec3, x: &Vec3, y: &Vec3, z: &Vec3) -> Self {
        let mut mat_from_basis = Self(Mat4::from_mat3(Mat3::from_cols(*x, *y, *z)));
        mat_from_basis.0.w_axis.x = pos.x;
        mat_from_basis.0.w_axis.y = pos.y;
        mat_from_basis.0.w_axis.z = pos.z;
        mat_from_basis
    }

    pub fn from_quat(q: &Quat) -> Self {
        Self(Mat4::from_quat(*q))
    }

    pub fn to_quat(&self) -> Quat {
        Quat::from_mat4(&self.0)
    }

    pub fn print(&self) {
        for r in 0..4 {
            let row = self.0.row(r);
            let s = row.as_ref().map(|elem| format!("{elem}"));
            info!("{}", s.join(" "));
        }
    }

    #[bind(role = "to_string")]
    pub fn get_string(&self) -> String {
        self.0.to_string()
    }
}
