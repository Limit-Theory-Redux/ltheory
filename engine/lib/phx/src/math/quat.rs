use glam::{Mat3, Vec3};

use super::{approximately_equal, Float_Validate};
use crate::error::Error;

#[derive(Clone, PartialEq)]
#[repr(C)]
pub struct Quat(glam::Quat);

impl Quat {
    fn _canonicalize(&self) -> glam::Quat {
        let value: f32 = if !approximately_equal(self.0.w, 0.0) {
            self.0.w
        } else if !approximately_equal(self.0.z, 0.0) {
            self.0.z
        } else if !approximately_equal(self.0.y, 0.0) {
            self.0.y
        } else if !approximately_equal(self.0.x, 0.0) {
            self.0.x
        } else {
            0.0
        };
        if value < 0.0 {
            -self.0
        } else {
            self.0
        }
    }

    fn _get_axis_x(&self) -> Vec3 {
        Vec3 {
            x: 1.0 - 2.0 * (self.0.y * self.0.y + self.0.z * self.0.z),
            y: 2.0 * (self.0.x * self.0.y + self.0.z * self.0.w),
            z: 2.0 * (self.0.x * self.0.z - self.0.y * self.0.w),
        }
    }

    fn _get_axis_y(&self) -> Vec3 {
        Vec3 {
            x: 2.0 * (self.0.x * self.0.y - self.0.z * self.0.w),
            y: 1.0 - 2.0 * (self.0.x * self.0.x + self.0.z * self.0.z),
            z: 2.0 * (self.0.y * self.0.z + self.0.x * self.0.w),
        }
    }

    fn _get_axis_z(&self) -> Vec3 {
        Vec3 {
            x: 2.0 * (self.0.x * self.0.z + self.0.y * self.0.w),
            y: 2.0 * (self.0.y * self.0.z - self.0.x * self.0.w),
            z: 1.0 - 2.0 * (self.0.x * self.0.x + self.0.y * self.0.y),
        }
    }
}

impl From<glam::Quat> for Quat {
    fn from(value: glam::Quat) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for Quat {
    type Target = glam::Quat;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[luajit_ffi_gen::luajit_ffi(typedef = "
    float x;
    float y;
    float z;
    float w;")]
impl Quat {
    #[bind(name = "Create")]
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self(glam::Quat { x, y, z, w })
    }

    pub fn get_axis_x(&self) -> Vec3 {
        self._get_axis_x()
    }

    pub fn get_axis_y(&self) -> Vec3 {
        self._get_axis_y()
    }

    pub fn get_axis_z(&self) -> Vec3 {
        self._get_axis_z()
    }

    pub fn get_forward(&self) -> Vec3 {
        -self._get_axis_z()
    }

    pub fn get_right(&self) -> Vec3 {
        self.get_axis_x()
    }

    pub fn get_up(&self) -> Vec3 {
        self._get_axis_y()
    }

    pub fn identity() -> Self {
        Self(glam::Quat::IDENTITY)
    }

    pub fn canonicalize(&self) -> Self {
        Self(self._canonicalize())
    }

    pub fn i_canonicalize(&mut self) {
        self.0 = self._canonicalize();
    }

    pub fn dot(&self, p: &Quat) -> f32 {
        self.0.dot(p.0)
    }

    pub fn equal(&self, p: &Quat) -> bool {
        self._canonicalize() == p._canonicalize()
    }

    pub fn approximately_equal(&self, p: &Quat) -> bool {
        let cq = self._canonicalize();
        let cp = p._canonicalize();
        approximately_equal(cq.x, cp.x)
            && approximately_equal(cq.y, cp.y)
            && approximately_equal(cq.z, cp.z)
            && approximately_equal(cq.w, cp.w)
    }

    pub fn inverse(&self) -> Self {
        Self(self.0.normalize().inverse())
    }

    pub fn i_inverse(&mut self) {
        self.0 = self.0.normalize().inverse();
    }

    pub fn lerp(&self, p: &Quat, t: f32) -> Self {
        Self(self.0.lerp(p.0, t))
    }

    pub fn i_lerp(&mut self, p: &Quat, t: f32) {
        self.0 = self.0.lerp(p.0, t);
    }

    pub fn mul(&self, p: &Quat) -> Self {
        Self(self.0.mul_quat(p.0))
    }

    pub fn i_mul(&mut self, p: &Quat) {
        self.0 = self.0.mul_quat(p.0);
    }

    pub fn mul_v(&self, v: &Vec3) -> Vec3 {
        self.0.mul_vec3(*v)
    }

    pub fn normalize(&self) -> Self {
        Self(self.0.normalize())
    }

    pub fn i_normalize(&mut self) {
        self.0 = self.0.normalize();
    }

    pub fn scale(&self, scale: f32) -> Self {
        Self::new(
            scale * self.0.x,
            scale * self.0.y,
            scale * self.0.z,
            scale * self.0.w,
        )
    }

    pub fn i_scale(&mut self, scale: f32) {
        self.0.x *= scale;
        self.0.y *= scale;
        self.0.z *= scale;
        self.0.w *= scale;
    }

    pub fn slerp(&self, p: &Quat, t: f32) -> Self {
        Self(self.0.slerp(p.0, t))
    }

    pub fn i_slerp(&mut self, p: &Quat, t: f32) {
        self.0 = self.0.slerp(p.0, t);
    }

    #[bind(role = "to_string")]
    pub fn get_string(&self) -> String {
        self.0.to_string()
    }

    pub fn validate(&self) -> Error {
        let mut e = 0 as Error;
        e |= Float_Validate(self.0.x as f64);
        e |= Float_Validate(self.0.y as f64);
        e |= Float_Validate(self.0.z as f64);
        e |= Float_Validate(self.0.w as f64);
        e
    }

    pub fn from_axis_angle(axis: &Vec3, radians: f32) -> Self {
        Self(glam::Quat::from_axis_angle(*axis, radians))
    }

    pub fn from_basis(x: &Vec3, y: &Vec3, z: &Vec3) -> Self {
        Self(glam::Quat::from_mat3(&Mat3::from_cols(*x, *y, *z)))
    }

    pub fn from_look_up(look: &Vec3, up: &Vec3) -> Self {
        let z = (*look * -1.0).normalize();
        let x = Vec3::cross(*up, z).normalize();
        let y = Vec3::cross(z, x);
        Self(glam::Quat::from_mat3(&Mat3::from_cols(x, y, z)))
    }

    pub fn from_rotate_to(from: &Vec3, to: &Vec3) -> Self {
        let axis = Vec3::cross(from.normalize(), to.normalize());
        let angle = f32::asin(axis.length());
        Self(glam::Quat::from_axis_angle(axis, angle))
    }
}
