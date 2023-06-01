use crate::*;

use crate::math::Float_Validate;
use crate::math::Vec3;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub fn to_string(&self) -> String {
        format!(
            "({:.4}, {:.4}, {:.4}, {:.4})",
            self.x, self.y, self.z, self.w
        )
    }
}

#[inline]
extern "C" fn Float_ApproximatelyEqual(x: f64, y: f64) -> bool {
    f64::abs(x - y) < 1e-3f64
}

#[no_mangle]
pub extern "C" fn Quat_Create(x: f32, y: f32, z: f32, w: f32) -> Quat {
    Quat { x, y, z, w }
}

#[no_mangle]
pub extern "C" fn Quat_GetAxisX(q: &Quat, out: &mut Vec3) {
    // out = q.
    out.x = 1.0f32 - 2.0f32 * (q.y * q.y + q.z * q.z);
    out.y = 2.0f32 * (q.x * q.y + q.z * q.w);
    out.z = 2.0f32 * (q.x * q.z - q.y * q.w);
}

#[no_mangle]
pub extern "C" fn Quat_GetAxisY(q: &Quat, out: &mut Vec3) {
    out.x = 2.0f32 * (q.x * q.y - q.z * q.w);
    out.y = 1.0f32 - 2.0f32 * (q.x * q.x + q.z * q.z);
    out.z = 2.0f32 * (q.y * q.z + q.x * q.w);
}

#[no_mangle]
pub extern "C" fn Quat_GetAxisZ(q: &Quat, out: &mut Vec3) {
    out.x = 2.0f32 * (q.x * q.z + q.y * q.w);
    out.y = 2.0f32 * (q.y * q.z - q.x * q.w);
    out.z = 1.0f32 - 2.0f32 * (q.x * q.x + q.y * q.y);
}

#[no_mangle]
pub extern "C" fn Quat_GetForward(q: &Quat, out: &mut Vec3) {
    Quat_GetAxisZ(q, out);
    out.x = -out.x;
    out.y = -out.y;
    out.z = -out.z;
}

#[no_mangle]
pub extern "C" fn Quat_GetRight(q: &Quat, out: &mut Vec3) {
    Quat_GetAxisX(q, out);
}

#[no_mangle]
pub extern "C" fn Quat_GetUp(q: &Quat, out: &mut Vec3) {
    Quat_GetAxisY(q, out);
}

#[no_mangle]
pub extern "C" fn Quat_Identity(out: &mut Quat) {
    out.x = 0.0f32;
    out.y = 0.0f32;
    out.z = 0.0f32;
    out.w = 1.0f32;
}

#[no_mangle]
pub extern "C" fn Quat_Canonicalize(q: &Quat, out: &mut Quat) {
    let value: f32 = if !Float_ApproximatelyEqual(q.w as f64, 0.0f64) {
        q.w
    } else if !Float_ApproximatelyEqual(q.z as f64, 0.0f64) {
        q.z
    } else if !Float_ApproximatelyEqual(q.y as f64, 0.0f64) {
        q.y
    } else if !Float_ApproximatelyEqual(q.x as f64, 0.0f64) {
        q.x
    } else {
        0.0f32
    };
    if value < 0.0f32 {
        out.x = -q.x;
        out.y = -q.y;
        out.z = -q.z;
        out.w = -q.w;
    } else {
        out.x = q.x;
        out.y = q.y;
        out.z = q.z;
        out.w = q.w;
    };
}

#[no_mangle]
pub extern "C" fn Quat_ICanonicalize(q: &mut Quat) {
    let value: f32 = if !Float_ApproximatelyEqual(q.w as f64, 0.0f64) {
        q.w
    } else if !Float_ApproximatelyEqual(q.z as f64, 0.0f64) {
        q.z
    } else if !Float_ApproximatelyEqual(q.y as f64, 0.0f64) {
        q.y
    } else if !Float_ApproximatelyEqual(q.x as f64, 0.0f64) {
        q.x
    } else {
        0.0f32
    };
    if value < 0.0f32 {
        q.x = -q.x;
        q.y = -q.y;
        q.z = -q.z;
        q.w = -q.w;
    }
}

#[no_mangle]
pub extern "C" fn Quat_Dot(q: &Quat, p: &Quat) -> f32 {
    q.x * p.x + q.y * p.y + q.z * p.z + q.w * p.w
}

#[no_mangle]
pub extern "C" fn Quat_Equal(q: &Quat, p: &Quat) -> bool {
    let mut cq = Quat_Create(0.0f32, 0.0f32, 0.0f32, 0.0f32);
    Quat_Canonicalize(q, &mut cq);
    let mut cp = Quat_Create(0.0f32, 0.0f32, 0.0f32, 0.0f32);
    Quat_Canonicalize(p, &mut cp);
    cq.x == cp.x && cq.y == cp.y && cq.z == cp.z && cq.w == cp.w
}

#[no_mangle]
pub extern "C" fn Quat_ApproximatelyEqual(q: &Quat, p: &Quat) -> bool {
    let mut cq = Quat_Create(0.0f32, 0.0f32, 0.0f32, 0.0f32);
    Quat_Canonicalize(q, &mut cq);
    let mut cp = Quat_Create(0.0f32, 0.0f32, 0.0f32, 0.0f32);
    Quat_Canonicalize(p, &mut cp);
    f64::abs((cq.x - cp.x) as f64) < 1e-3f64
        && f64::abs((cq.y - cp.y) as f64) < 1e-3f64
        && f64::abs((cq.z - cp.z) as f64) < 1e-3f64
        && f64::abs((cq.w - cp.w) as f64) < 1e-3f64
}

#[no_mangle]
pub extern "C" fn Quat_Inverse(q: &Quat, out: &mut Quat) {
    let magSq: f32 = q.x * q.x + q.y * q.y + q.z * q.z + q.w * q.w;
    out.x = -q.x / magSq;
    out.y = -q.y / magSq;
    out.z = -q.z / magSq;
    out.w = q.w / magSq;
}

#[no_mangle]
pub extern "C" fn Quat_IInverse(q: &mut Quat) {
    let magSq: f32 = q.x * q.x + q.y * q.y + q.z * q.z + q.w * q.w;
    q.x = -q.x / magSq;
    q.y = -q.y / magSq;
    q.z = -q.z / magSq;
    q.w /= magSq;
}

#[no_mangle]
pub extern "C" fn Quat_Lerp(q: &Quat, p: &Quat, t: f32, out: &mut Quat) {
    let d: f32 = Quat_Dot(p, q);
    let mut dp = Quat_Create(0.0f32, 0.0f32, 0.0f32, 0.0f32);
    if d < 0.0f32 {
        dp.x = -p.x;
        dp.y = -p.y;
        dp.z = -p.z;
        dp.w = -p.w;
    } else {
        dp = *p;
    }
    let x: f32 = q.x + (dp.x - q.x) * t;
    let y: f32 = q.y + (dp.y - q.y) * t;
    let z: f32 = q.z + (dp.z - q.z) * t;
    let w: f32 = q.w + (dp.w - q.w) * t;
    let rcpMag: f32 = (1.0f64 / f64::sqrt((x * x + y * y + z * z + w * w) as f64)) as f32;
    out.x = x * rcpMag;
    out.y = y * rcpMag;
    out.z = z * rcpMag;
    out.w = w * rcpMag;
}

#[no_mangle]
pub extern "C" fn Quat_ILerp(q: &mut Quat, p: &Quat, t: f32) {
    let d: f32 = Quat_Dot(p, q);
    let mut dp = Quat_Create(0.0f32, 0.0f32, 0.0f32, 0.0f32);
    if d < 0.0f32 {
        dp.x = -p.x;
        dp.y = -p.y;
        dp.z = -p.z;
        dp.w = -p.w;
    } else {
        dp = *p;
    }
    let x: f32 = q.x + (dp.x - q.x) * t;
    let y: f32 = q.y + (dp.y - q.y) * t;
    let z: f32 = q.z + (dp.z - q.z) * t;
    let w: f32 = q.w + (dp.w - q.w) * t;
    let rcpMag: f32 = (1.0f64 / f64::sqrt((x * x + y * y + z * z + w * w) as f64)) as f32;
    q.x = x * rcpMag;
    q.y = y * rcpMag;
    q.z = z * rcpMag;
    q.w = w * rcpMag;
}

#[no_mangle]
pub extern "C" fn Quat_Mul(q: &Quat, p: &Quat, out: &mut Quat) {
    let qv: Vec3 = Vec3::new(q.x, q.y, q.z);
    let pv: Vec3 = Vec3::new(p.x, p.y, p.z);
    let rv: Vec3 = (qv * p.w) + (pv * q.w) + Vec3::cross(qv, pv);
    out.x = rv.x;
    out.y = rv.y;
    out.z = rv.z;
    out.w = q.w * p.w - Vec3::dot(qv, pv);
}

#[no_mangle]
pub extern "C" fn Quat_IMul(q: &mut Quat, p: &Quat) {
    let qv: Vec3 = Vec3::new(q.x, q.y, q.z);
    let pv: Vec3 = Vec3::new(p.x, p.y, p.z);
    let rv: Vec3 = (qv * p.w) + (pv * q.w) + Vec3::cross(qv, pv);
    q.x = rv.x;
    q.y = rv.y;
    q.z = rv.z;
    q.w = q.w * p.w - Vec3::dot(qv, pv);
}

#[no_mangle]
pub extern "C" fn Quat_MulV(q: &Quat, v: &Vec3, out: &mut Vec3) {
    let u: Vec3 = Vec3::new(q.x, q.y, q.z);
    let w: f32 = q.w;
    let t: Vec3 = Vec3::cross(u, *v);
    *out = (u * 2.0f32 * Vec3::dot(u, *v)) + ((*v) * (2.0f32 * w * w - 1.0f32)) + (t * 2.0f32 * w);
}

#[no_mangle]
pub extern "C" fn Quat_Normalize(q: &Quat, out: &mut Quat) {
    let mag = f32::sqrt(q.x * q.x + q.y * q.y + q.z * q.z + q.w * q.w);
    out.x = q.x / mag;
    out.y = q.y / mag;
    out.z = q.z / mag;
    out.w = q.w / mag;
}

#[no_mangle]
pub extern "C" fn Quat_INormalize(q: &mut Quat) {
    let mag = f32::sqrt(q.x * q.x + q.y * q.y + q.z * q.z + q.w * q.w);
    q.x /= mag;
    q.y /= mag;
    q.z /= mag;
    q.w /= mag;
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
    let mut np = Quat_Create(0.0f32, 0.0f32, 0.0f32, 0.0f32);
    Quat_Normalize(p, &mut np);
    let mut d: f32 = Quat_Dot(q, p);
    if d < 0.0f32 {
        np.x = -np.x;
        np.y = -np.y;
        np.z = -np.z;
        np.w = -np.w;
        d = -d;
    }
    if d > 0.9995f32 {
        Quat_Lerp(q, p, t, out);
        return;
    }
    d = f32::clamp(d, -1.0f32, 1.0f32);
    let angle: f32 = t * f32::acos(d);
    let mut c = Quat_Create(p.x - d * q.x, p.y - d * q.y, p.z - d * q.z, p.w - d * q.w);
    Quat_INormalize(&mut c);
    let fa: f32 = f32::cos(angle);
    let fc: f32 = f32::sin(angle);
    out.x = fa * q.x + fc * c.x;
    out.y = fa * q.y + fc * c.y;
    out.z = fa * q.z + fc * c.z;
    out.w = fa * q.w + fc * c.w;
}

#[no_mangle]
pub extern "C" fn Quat_ISlerp(q: &mut Quat, p: &Quat, t: f32) {
    let mut np = Quat_Create(0.0f32, 0.0f32, 0.0f32, 0.0f32);
    Quat_Normalize(p, &mut np);
    let mut d: f32 = Quat_Dot(q, p);
    if d < 0.0f32 {
        np.x = -np.x;
        np.y = -np.y;
        np.z = -np.z;
        np.w = -np.w;
        d = -d;
    }
    if d > 0.9995f32 {
        Quat_ILerp(q, p, t);
        return;
    }
    d = f32::clamp(d, -1.0f32, 1.0f32);
    let angle: f32 = t * f32::acos(d);
    let mut c = Quat_Create(p.x - d * q.x, p.y - d * q.y, p.z - d * q.z, p.w - d * q.w);
    Quat_INormalize(&mut c);
    let fa: f32 = f32::cos(angle);
    let fc: f32 = f32::sin(angle);
    q.x = fa * q.x + fc * c.x;
    q.y = fa * q.y + fc * c.y;
    q.z = fa * q.z + fc * c.z;
    q.w = fa * q.w + fc * c.w;
}

#[no_mangle]
pub extern "C" fn Quat_ToString(q: &Quat) -> *const libc::c_char {
    static_string!(q.to_string())
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
    let v: Vec3 = *axis * f32::sin(radians * 0.5);
    out.x = v.x;
    out.y = v.y;
    out.z = v.z;
    out.w = f32::cos(radians);
}

#[no_mangle]
pub extern "C" fn Quat_FromBasis(x: &Vec3, y: &Vec3, z: &Vec3, out: &mut Quat) {
    let r: f32 = x.x + y.y + z.z;
    if r > 0.0f32 {
        out.w = (f64::sqrt((r + 1.0f32) as f64) * 0.5f64) as f32;
        let w4: f32 = 1.0f32 / (4.0f32 * out.w);
        out.x = (y.z - z.y) * w4;
        out.y = (z.x - x.z) * w4;
        out.z = (x.y - y.x) * w4;
    } else if x.x > y.y && x.x > z.z {
        out.x = (f64::sqrt((1.0f32 + x.x - y.y - z.z) as f64) * 0.5f64) as f32;
        let x4: f32 = 1.0f32 / (4.0f32 * out.x);
        out.y = (y.x + x.y) * x4;
        out.z = (z.x + x.z) * x4;
        out.w = (y.z - z.y) * x4;
    } else if y.y > z.z {
        out.y = (f64::sqrt((1.0f32 + y.y - x.x - z.z) as f64) * 0.5f64) as f32;
        let y4: f32 = 1.0f32 / (4.0f32 * out.y);
        out.x = (y.x + x.y) * y4;
        out.z = (z.y + y.z) * y4;
        out.w = (z.x - x.z) * y4;
    } else {
        out.z = (f64::sqrt((1.0f32 + z.z - x.x - y.y) as f64) * 0.5f64) as f32;
        let z4: f32 = 1.0f32 / (4.0f32 * out.z);
        out.x = (z.x + x.z) * z4;
        out.y = (z.y + y.z) * z4;
        out.w = (x.y - y.x) * z4;
    };
}

#[no_mangle]
pub extern "C" fn Quat_FromLookUp(look: &Vec3, up: &Vec3, out: &mut Quat) {
    let mut z: Vec3 = (*look * -1.0f32).normalize();
    let mut x: Vec3 = Vec3::cross(*up, z).normalize();
    let mut y: Vec3 = Vec3::cross(z, x);
    Quat_FromBasis(&mut x, &mut y, &mut z, out);
}

#[no_mangle]
pub extern "C" fn Quat_FromRotateTo(from: &Vec3, to: &Vec3, out: &mut Quat) {
    let mut axis: Vec3 = Vec3::cross((*from).normalize(), (*to).normalize());
    let angle = f32::asin(axis.length());
    Quat_FromAxisAngle(&mut axis, angle, out);
}
