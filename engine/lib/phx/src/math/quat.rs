use glam::{Mat3, Quat as GlamQuat, Vec3};

use super::{approximately_equal, validate_f64};
use crate::error::Error;

#[derive(Clone, PartialEq)]
#[repr(C)]
pub struct Quat(pub GlamQuat); // made field public for direct access if needed

impl Quat {
    // --- Private helpers (not FFI) ---
    fn _canonicalize(&self) -> GlamQuat {
        let q = self.0;
        if q.w < 0.0 { -q } else { q }
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

    // --- Game-dev helpers (non-FFI) ---
    pub fn to_euler(&self) -> (f32, f32, f32) {
        let q = self._canonicalize();
        let sinp = 2.0 * (q.w * q.y - q.z * q.x);
        let pitch = if sinp.abs() >= 1.0 {
            std::f32::consts::FRAC_PI_2.copysign(sinp)
        } else {
            sinp.asin()
        };
        let yaw = (2.0 * (q.w * q.z + q.x * q.y) / (1.0 - 2.0 * (q.y * q.y + q.z * q.z)))
            .atan2(1.0 - 2.0 * (q.x * q.x + q.y * q.y));
        let roll = (2.0 * (q.w * q.x + q.y * q.z) / (1.0 - 2.0 * (q.x * q.x + q.z * q.z)))
            .atan2(1.0 - 2.0 * (q.y * q.y + q.z * q.z));
        (yaw, pitch, roll)
    }

    pub fn conjugate(&self) -> Self {
        Self(GlamQuat::from_xyzw(self.0.x, self.0.y, self.0.z, -self.0.w))
    }

    pub fn i_conjugate(&mut self) {
        self.0.w = -self.0.w;
    }

    pub fn length_squared(&self) -> f32 {
        self.0.length_squared()
    }

    pub fn length(&self) -> f32 {
        self.0.length()
    }

    pub fn angle_between(&self, other: &Quat) -> f32 {
        self.0.dot(other.0).abs().acos() * 2.0
    }

    // Static helper (not instance)
    pub fn from_look_static(forward: &Vec3, up: &Vec3) -> Self {
        let z = -forward.normalize();
        let x = up.cross(z).normalize();
        let y = z.cross(x);
        Self(GlamQuat::from_mat3(&Mat3::from_cols(x, y, z)))
    }

    pub fn look_at_static(eye: &Vec3, target: &Vec3, up: &Vec3) -> Self {
        let forward = (*target - *eye).normalize();
        Self::from_look_static(&forward, up)
    }

    pub fn nlerp(&self, p: &Quat, t: f32) -> Self {
        Self((self.0 + (p.0 - self.0) * t).normalize())
    }

    pub fn i_nlerp(&mut self, p: &Quat, t: f32) {
        self.0 = (self.0 + (p.0 - self.0) * t).normalize();
    }
}

impl From<GlamQuat> for Quat {
    fn from(value: GlamQuat) -> Self {
        Self(value)
    }
}

impl std::ops::Deref for Quat {
    type Target = GlamQuat;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[luajit_ffi_gen::luajit_ffi(
    clone = true,
    typedef = "
        float x;
        float y;
        float z;
        float w;"
)]
impl Quat {
    #[bind(name = "Create")]
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self(GlamQuat { x, y, z, w })
    }

    #[bind(name = "Identity")]
    pub fn identity() -> Self {
        Self(GlamQuat::IDENTITY)
    }

    #[bind(out_param = true)]
    pub fn get_axis_x(&self) -> Vec3 {
        self._get_axis_x()
    }

    #[bind(out_param = true)]
    pub fn get_axis_y(&self) -> Vec3 {
        self._get_axis_y()
    }

    #[bind(out_param = true)]
    pub fn get_axis_z(&self) -> Vec3 {
        self._get_axis_z()
    }

    #[bind(out_param = true)]
    pub fn get_forward(&self) -> Vec3 {
        -self._get_axis_z()
    }

    #[bind(out_param = true)]
    pub fn get_right(&self) -> Vec3 {
        self.get_axis_x()
    }

    #[bind(out_param = true)]
    pub fn get_up(&self) -> Vec3 {
        self._get_axis_y()
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

    #[bind(out_param = true)]
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

    #[bind(name = "FromAxisAngle")]
    pub fn from_axis_angle(axis: &Vec3, radians: f32) -> Self {
        Self(GlamQuat::from_axis_angle(*axis, radians))
    }

    #[bind(name = "FromEuler")]
    pub fn from_euler(yaw: f32, pitch: f32, roll: f32) -> Self {
        Self(GlamQuat::from_euler(glam::EulerRot::YXZ, yaw, pitch, roll))
    }

    #[bind(name = "FromLook")]
    pub fn from_look(forward: &Vec3, up: &Vec3) -> Self {
        Self::from_look_static(forward, up)
    }

    #[bind(name = "FromLookUp")]
    pub fn from_look_up(look: &Vec3, up: &Vec3) -> Self {
        let z = (*look * -1.0).normalize();
        let x = Vec3::cross(*up, z).normalize();
        let y = Vec3::cross(z, x);
        Self(glam::Quat::from_mat3(&Mat3::from_cols(x, y, z)))
    }

    #[bind(name = "LookAt")]
    pub fn look_at(eye: &Vec3, target: &Vec3, up: &Vec3) -> Self {
        Self::look_at_static(eye, target, up)
    }

    #[bind(name = "FromRotateTo")]
    pub fn from_rotate_to(from: &Vec3, to: &Vec3) -> Self {
        let f = from.normalize();
        let t = to.normalize();
        let dot = f.dot(t);
        if dot > 0.9999 {
            return Self::identity();
        }
        if dot < -0.9999 {
            let axis = Vec3::cross(glam::Vec3::X, f);
            let axis = if axis.length_squared() < 1e-6 {
                Vec3::cross(glam::Vec3::Y, f)
            } else {
                axis
            }
            .normalize();
            return Self::from_axis_angle(&axis, std::f32::consts::PI);
        }
        let axis = Vec3::cross(f, t).normalize();
        let angle = dot.acos();
        Self::from_axis_angle(&axis, angle)
    }

    #[bind(role = "to_string")]
    pub fn get_string(&self) -> String {
        format!(
            "{:.6},{:.6},{:.6},{:.6}",
            self.0.x, self.0.y, self.0.z, self.0.w
        )
    }

    pub fn validate(&self) -> Error {
        let mut e = 0 as Error;
        e |= validate_f64(self.0.x as f64); // TODO: what is the reason to cast to f64 if there is validatef for f32?
        e |= validate_f64(self.0.y as f64);
        e |= validate_f64(self.0.z as f64);
        e |= validate_f64(self.0.w as f64);
        e
    }
}
