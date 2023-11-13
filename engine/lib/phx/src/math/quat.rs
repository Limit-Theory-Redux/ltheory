use crate::error::Error;
use crate::math::*;
use crate::*;

pub use glam::Quat;

pub trait QuatExtensions {
    fn canonicalize(&self) -> Quat;
    fn get_axis_x(&self) -> Vec3;
    fn get_axis_y(&self) -> Vec3;
    fn get_axis_z(&self) -> Vec3;
}

impl QuatExtensions for Quat {
    fn canonicalize(&self) -> Quat {
        let value: f32 = if !Float_ApproximatelyEqualf(self.w, 0.0) {
            self.w
        } else if !Float_ApproximatelyEqualf(self.z, 0.0) {
            self.z
        } else if !Float_ApproximatelyEqualf(self.y, 0.0) {
            self.y
        } else if !Float_ApproximatelyEqualf(self.x, 0.0) {
            self.x
        } else {
            0.0
        };
        if value < 0.0 {
            -*self
        } else {
            *self
        }
    }

    fn get_axis_x(&self) -> Vec3 {
        Vec3 {
            x: 1.0 - 2.0 * (self.y * self.y + self.z * self.z),
            y: 2.0 * (self.x * self.y + self.z * self.w),
            z: 2.0 * (self.x * self.z - self.y * self.w),
        }
    }

    fn get_axis_y(&self) -> Vec3 {
        Vec3 {
            x: 2.0 * (self.x * self.y - self.z * self.w),
            y: 1.0 - 2.0 * (self.x * self.x + self.z * self.z),
            z: 2.0 * (self.y * self.z + self.x * self.w),
        }
    }

    fn get_axis_z(&self) -> Vec3 {
        Vec3 {
            x: 2.0 * (self.x * self.z + self.y * self.w),
            y: 2.0 * (self.y * self.z - self.x * self.w),
            z: 1.0 - 2.0 * (self.x * self.x + self.y * self.y),
        }
    }
}

#[no_mangle]
pub extern "C" fn Quat_Create(x: f32, y: f32, z: f32, w: f32) -> Quat {
    Quat { x, y, z, w }
}

#[no_mangle]
pub extern "C" fn Quat_GetAxisX(q: &Quat, out: &mut Vec3) {
    *out = q.get_axis_x()
}

#[no_mangle]
pub extern "C" fn Quat_GetAxisY(q: &Quat, out: &mut Vec3) {
    *out = q.get_axis_y()
}

#[no_mangle]
pub extern "C" fn Quat_GetAxisZ(q: &Quat, out: &mut Vec3) {
    *out = q.get_axis_z()
}

#[no_mangle]
pub extern "C" fn Quat_GetForward(q: &Quat, out: &mut Vec3) {
    *out = -q.get_axis_z();
}

#[no_mangle]
pub extern "C" fn Quat_GetRight(q: &Quat, out: &mut Vec3) {
    *out = q.get_axis_x();
}

#[no_mangle]
pub extern "C" fn Quat_GetUp(q: &Quat, out: &mut Vec3) {
    *out = q.get_axis_y();
}

#[no_mangle]
pub extern "C" fn Quat_Identity(out: &mut Quat) {
    *out = Quat::IDENTITY;
}

#[no_mangle]
pub extern "C" fn Quat_Canonicalize(q: &Quat, out: &mut Quat) {
    *out = q.canonicalize();
}

#[no_mangle]
pub extern "C" fn Quat_ICanonicalize(q: &mut Quat) {
    *q = q.canonicalize();
}

#[no_mangle]
pub extern "C" fn Quat_Dot(q: &Quat, p: &Quat) -> f32 {
    q.dot(*p)
}

#[no_mangle]
pub extern "C" fn Quat_Equal(q: &Quat, p: &Quat) -> bool {
    q.canonicalize() == p.canonicalize()
}

#[no_mangle]
pub extern "C" fn Quat_ApproximatelyEqual(q: &Quat, p: &Quat) -> bool {
    let cq = q.canonicalize();
    let cp = p.canonicalize();
    Float_ApproximatelyEqualf(cq.x, cp.x)
        && Float_ApproximatelyEqualf(cq.y, cp.y)
        && Float_ApproximatelyEqualf(cq.z, cp.z)
        && Float_ApproximatelyEqualf(cq.w, cp.w)
}

#[no_mangle]
pub extern "C" fn Quat_Inverse(q: &Quat, out: &mut Quat) {
    *out = q.normalize().inverse();
}

#[no_mangle]
pub extern "C" fn Quat_IInverse(q: &mut Quat) {
    *q = q.normalize().inverse();
}

#[no_mangle]
pub extern "C" fn Quat_Lerp(q: &Quat, p: &Quat, t: f32, out: &mut Quat) {
    *out = q.lerp(*p, t);
}

#[no_mangle]
pub extern "C" fn Quat_ILerp(q: &mut Quat, p: &Quat, t: f32) {
    *q = q.lerp(*p, t);
}

#[no_mangle]
pub extern "C" fn Quat_Mul(q: &Quat, p: &Quat, out: &mut Quat) {
    *out = q.mul_quat(*p);
}

#[no_mangle]
pub extern "C" fn Quat_IMul(q: &mut Quat, p: &Quat) {
    *q = q.mul_quat(*p);
}

#[no_mangle]
pub extern "C" fn Quat_MulV(q: &Quat, v: &Vec3, out: &mut Vec3) {
    *out = q.mul_vec3(*v);
}

#[no_mangle]
pub extern "C" fn Quat_Normalize(q: &Quat, out: &mut Quat) {
    *out = q.normalize();
}

#[no_mangle]
pub extern "C" fn Quat_INormalize(q: &mut Quat) {
    *q = q.normalize();
}

#[no_mangle]
pub extern "C" fn Quat_Scale(q: &Quat, scale: f32, out: &mut Quat) {
    out.x = scale * q.x;
    out.y = scale * q.y;
    out.z = scale * q.z;
    out.w = scale * q.w;
}

#[no_mangle]
pub extern "C" fn Quat_IScale(q: &mut Quat, scale: f32) {
    q.x *= scale;
    q.y *= scale;
    q.z *= scale;
    q.w *= scale;
}

#[no_mangle]
pub extern "C" fn Quat_Slerp(q: &Quat, p: &Quat, t: f32, out: &mut Quat) {
    *out = q.slerp(*p, t);
}

#[no_mangle]
pub extern "C" fn Quat_ISlerp(q: &mut Quat, p: &Quat, t: f32) {
    *q = q.slerp(*p, t);
}

#[no_mangle]
pub extern "C" fn Quat_ToString(q: &Quat) -> *const libc::c_char {
    internal::static_string!(q.to_string())
}

#[no_mangle]
pub extern "C" fn Quat_Validate(q: &Quat) -> Error {
    let mut e = 0 as Error;
    e |= Float_Validate(q.x as f64);
    e |= Float_Validate(q.y as f64);
    e |= Float_Validate(q.z as f64);
    e |= Float_Validate(q.w as f64);
    e
}

#[no_mangle]
pub extern "C" fn Quat_FromAxisAngle(axis: &Vec3, radians: f32, out: &mut Quat) {
    *out = Quat::from_axis_angle(*axis, radians);
}

#[no_mangle]
pub extern "C" fn Quat_FromBasis(x: &Vec3, y: &Vec3, z: &Vec3, out: &mut Quat) {
    *out = Quat::from_mat3(&Mat3::from_cols(*x, *y, *z));
}

#[no_mangle]
pub extern "C" fn Quat_FromLookUp(look: &Vec3, up: &Vec3, out: &mut Quat) {
    let z = (*look * -1.0).normalize();
    let x = Vec3::cross(*up, z).normalize();
    let y = Vec3::cross(z, x);
    *out = Quat::from_mat3(&Mat3::from_cols(x, y, z));
}

#[no_mangle]
pub extern "C" fn Quat_FromRotateTo(from: &Vec3, to: &Vec3, out: &mut Quat) {
    let axis = Vec3::cross(from.normalize(), to.normalize());
    let angle = f32::asin(axis.length());
    *out = Quat::from_axis_angle(axis, angle);
}
