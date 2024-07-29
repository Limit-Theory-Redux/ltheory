// Re-export glam types.
pub use glam::{
    dvec2, dvec3, dvec4, ivec2, ivec3, ivec4, vec2, vec3, vec4, DVec2, DVec3, DVec4, IVec2, IVec3,
    IVec4, Vec2, Vec3, Vec4,
};

use crate::error::Error;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Sphere {
    pub p: Vec3,
    pub r: f32,
}

#[inline]
pub fn Lerp(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

#[inline]
pub fn Saturate(t: f64) -> f64 {
    f64::clamp(t, 0.0f64, 1.0f64)
}

#[inline]
pub extern "C" fn Float_Validatef(x: f32) -> Error {
    let classification: i32 = if std::mem::size_of::<f32>() == std::mem::size_of::<f32>() {
        f32::classify(x) as i32
    } else if std::mem::size_of::<f32>() == std::mem::size_of::<f64>() {
        f64::classify(x as f64) as i32
    } else {
        3
    };
    match classification {
        2 => return 0x4 as Error,
        5 => {}
        1 => return 0x20 as Error,
        3 | 4 => return 0 as Error,
        _ => {
            panic!("Float_Validate: Unhandled case: {classification}");
        }
    }
    0 as Error
}

#[inline]
pub extern "C" fn Float_ApproximatelyEqual(x: f64, y: f64) -> bool {
    f64::abs(x - y) < 1e-3
}

#[inline]
pub extern "C" fn Float_ApproximatelyEqualf(x: f32, y: f32) -> bool {
    f32::abs(x - y) < 1e-3
}

#[inline]
pub extern "C" fn Float_Validate(x: f64) -> Error {
    let classification: i32 = if std::mem::size_of::<f64>() as libc::c_ulong
        == std::mem::size_of::<f32>() as libc::c_ulong
    {
        f32::classify(x as f32) as i32
    } else if std::mem::size_of::<f64>() as libc::c_ulong
        == std::mem::size_of::<f64>() as libc::c_ulong
    {
        f64::classify(x) as i32
    } else {
        3
    };
    match classification {
        2 => 0x4 as Error,
        5 => 0x8 as Error,
        1 => 0x20 as Error,
        3 | 4 => 0 as Error,
        _ => {
            panic!("Float_Validate: Unhandled case: {classification}");
        }
    }
}

#[inline]
pub extern "C" fn Vec3_Validate(v: Vec3) -> Error {
    let mut e = 0 as Error;
    e |= Float_Validatef(v.x);
    e |= Float_Validatef(v.y);
    e |= Float_Validatef(v.z);
    e
}

#[inline]
pub extern "C" fn Vec3_Reject(a: Vec3, b: Vec3) -> Vec3 {
    let d: f32 = Vec3::dot(a, b);
    Vec3 {
        x: a.x - d * b.x,
        y: a.y - d * b.y,
        z: a.z - d * b.z,
    }
}

#[no_mangle]
pub extern "C" fn Math_Bezier3(x: f64, y1: f64, y2: f64, y3: f64) -> f64 {
    let y12: f64 = Lerp(y1, y2, x);
    let y23: f64 = Lerp(y2, y3, x);
    Lerp(y12, y23, x)
}

#[no_mangle]
pub extern "C" fn Math_Bezier4(x: f64, y1: f64, y2: f64, y3: f64, y4: f64) -> f64 {
    let y12: f64 = Lerp(y1, y2, x);
    let y23: f64 = Lerp(y2, y3, x);
    let y34: f64 = Lerp(y3, y4, x);
    let y123: f64 = Lerp(y12, y23, x);
    let y234: f64 = Lerp(y23, y34, x);
    Lerp(y123, y234, x)
}

#[no_mangle]
pub extern "C" fn Math_Clamp(x: f64, a: f64, b: f64) -> f64 {
    x.clamp(a, b)
}

#[no_mangle]
pub extern "C" fn Math_Clamp01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

#[no_mangle]
pub extern "C" fn Math_ClampSafe(x: f64, a: f64, b: f64) -> f64 {
    if b < a {
        x.clamp(b, a)
    } else {
        x.clamp(a, b)
    }
}

#[no_mangle]
pub extern "C" fn Math_ClampUnit(x: f64) -> f64 {
    f64::clamp(x, -1.0f64, 1.0f64)
}

#[no_mangle]
pub extern "C" fn Math_ExpMap(x: f64, p: f64) -> f64 {
    1.0f64 - f64::exp(-f64::powf(f64::abs(x), p))
}

#[no_mangle]
pub extern "C" fn Math_ExpMapSigned(x: f64, p: f64) -> f64 {
    f64::signum(x) * (1.0f64 - f64::exp(-f64::powf(f64::abs(x), p)))
}

#[no_mangle]
pub extern "C" fn Math_ExpMap1(x: f64) -> f64 {
    1.0f64 - f64::exp(-f64::abs(x))
}

#[no_mangle]
pub extern "C" fn Math_ExpMap1Signed(x: f64) -> f64 {
    f64::signum(x) * (1.0f64 - f64::exp(-f64::abs(x)))
}

#[no_mangle]
pub extern "C" fn Math_ExpMap2(x: f64) -> f64 {
    1.0f64 - f64::exp(-x * x)
}

#[no_mangle]
pub extern "C" fn Math_ExpMap2Signed(x: f64) -> f64 {
    f64::signum(x) * (1.0f64 - f64::exp(-x * x))
}

#[no_mangle]
pub extern "C" fn Math_PowSigned(x: f64, p: f64) -> f64 {
    f64::signum(x) * f64::powf(f64::abs(x), p)
}

#[no_mangle]
pub extern "C" fn Math_Round(x: f64) -> f64 {
    f64::round(x)
}

#[no_mangle]
pub extern "C" fn Math_Sign(x: f64) -> f64 {
    if x > 0.0f64 {
        1.0f64
    } else if x < 0.0f64 {
        -1.0f64
    } else {
        0.0f64
    }
}
