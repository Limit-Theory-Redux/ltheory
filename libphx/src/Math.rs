use crate::internal::Memory::*;
use crate::Common::*;
use libc;

// Re-export glam types.
pub use glam::{DVec2, DVec3, DVec4};
pub use glam::{IVec2, IVec3, IVec4};
pub use glam::{Vec2, Vec3, Vec4};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Box3 {
    pub lower: Vec3,
    pub upper: Vec3,
}

impl Box3 {
    pub fn new(lower: Vec3, upper: Vec3) -> Box3 {
        Box3 { lower, upper }
    }

    pub fn union(a: Box3, b: Box3) -> Box3 {
        Box3 {
            lower: Vec3 {
                x: f32::min(a.lower.x, b.lower.x),
                y: f32::min(a.lower.y, b.lower.y),
                z: f32::min(a.lower.z, b.lower.z),
            },
            upper: Vec3 {
                x: f32::max(a.upper.x, b.upper.x),
                y: f32::max(a.upper.y, b.upper.y),
                z: f32::max(a.upper.z, b.upper.z),
            },
        }
    }

    pub fn intersection(a: Box3, b: Box3) -> Box3 {
        Box3 {
            lower: Vec3 {
                x: f32::max(a.lower.x, b.lower.x),
                y: f32::max(a.lower.y, b.lower.y),
                z: f32::max(a.lower.z, b.lower.z),
            },
            upper: Vec3 {
                x: f32::min(a.upper.x, b.upper.x),
                y: f32::min(a.upper.y, b.upper.y),
                z: f32::min(a.upper.z, b.upper.z),
            },
        }
    }

    pub fn center(&self) -> Vec3 {
        Vec3::new(
            (self.lower.x + self.upper.x) / 2.0f32,
            (self.lower.y + self.upper.y) / 2.0f32,
            (self.lower.z + self.upper.z) / 2.0f32,
        )
    }

    pub fn add(&mut self, point: Vec3) {
        self.lower = Vec3::min(self.lower, point);
        self.upper = Vec3::max(self.upper, point);
    }

    pub fn volume(&self) -> f32 {
        (self.upper.x - self.lower.x)
            * (self.upper.y - self.lower.y)
            * (self.upper.z - self.lower.z)
    }

    pub fn contains(a: Box3, b: Box3) -> bool {
        a.lower.x <= b.lower.x
            && a.upper.x >= b.upper.x
            && a.lower.y <= b.lower.y
            && a.upper.y >= b.upper.y
            && a.lower.z <= b.lower.z
            && a.upper.z >= b.upper.z
    }

    pub fn intersects_ray(&self, ro: Vec3, rdi: Vec3) -> bool {
        let mut t1: f64 = (rdi.x * (self.lower.x - ro.x)) as f64;
        let mut t2: f64 = (rdi.x * (self.upper.x - ro.x)) as f64;
        let mut tMin: f64 = f64::min(t1, t2);
        let mut tMax: f64 = f64::max(t1, t2);
        t1 = (rdi.y * (self.lower.y - ro.y)) as f64;
        t2 = (rdi.y * (self.upper.y - ro.y)) as f64;
        tMin = f64::max(tMin, f64::min(t1, t2));
        tMax = f64::min(tMax, f64::max(t1, t2));
        t1 = (rdi.z * (self.lower.z - ro.z)) as f64;
        t2 = (rdi.z * (self.upper.z - ro.z)) as f64;
        tMin = f64::max(tMin, f64::min(t1, t2));
        tMax = f64::min(tMax, f64::max(t1, t2));
        tMax >= tMin && tMax > 0 as f64
    }

    pub fn intersects_box(a: Box3, b: Box3) -> bool {
        if a.lower.x > b.upper.x || a.upper.x < b.lower.x {
            return false;
        }
        if a.lower.y > b.upper.y || a.upper.y < b.lower.y {
            return false;
        }
        if a.lower.z > b.upper.z || a.upper.z < b.lower.z {
            return false;
        }
        true
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Sphere {
    pub p: Vec3,
    pub r: f32,
}

#[inline]
pub unsafe extern "C" fn Lerp(mut a: f64, mut b: f64, mut t: f64) -> f64 {
    a + t * (b - a)
}

#[inline]
pub unsafe extern "C" fn Saturate(mut t: f64) -> f64 {
    f64::clamp(t, 0.0f64, 1.0f64)
}

#[inline]
pub unsafe extern "C" fn Float_Validatef(mut x: f32) -> Error {
    let mut classification: i32 = if std::mem::size_of::<f32>() == std::mem::size_of::<f32>() {
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
            Fatal(
                b"Float_Validate: Unhandled case: %i\0" as *const u8 as *const libc::c_char,
                classification,
            );
        }
    }
    0 as Error
}

#[inline]
pub unsafe extern "C" fn Float_Validate(mut x: f64) -> Error {
    let mut classification: i32 = if std::mem::size_of::<f64>() as libc::c_ulong
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
        2 => return 0x4 as Error,
        5 => return 0x8 as Error,
        1 => return 0x20 as Error,
        3 | 4 => return 0 as Error,
        _ => {
            Fatal(
                b"Float_Validate: Unhandled case: %i\0" as *const u8 as *const libc::c_char,
                classification,
            );
        }
    }
}

#[inline]
pub unsafe extern "C" fn Vec3_Validate(mut v: Vec3) -> Error {
    let mut e: Error = 0 as Error;
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
pub unsafe extern "C" fn Math_Bezier3(mut x: f64, mut y1: f64, mut y2: f64, mut y3: f64) -> f64 {
    let mut y12: f64 = Lerp(y1, y2, x);
    let mut y23: f64 = Lerp(y2, y3, x);
    Lerp(y12, y23, x)
}

#[no_mangle]
pub unsafe extern "C" fn Math_Bezier4(
    mut x: f64,
    mut y1: f64,
    mut y2: f64,
    mut y3: f64,
    mut y4: f64,
) -> f64 {
    let mut y12: f64 = Lerp(y1, y2, x);
    let mut y23: f64 = Lerp(y2, y3, x);
    let mut y34: f64 = Lerp(y3, y4, x);
    let mut y123: f64 = Lerp(y12, y23, x);
    let mut y234: f64 = Lerp(y23, y34, x);
    Lerp(y123, y234, x)
}

#[no_mangle]
pub unsafe extern "C" fn Math_Clamp(mut x: f64, mut a: f64, mut b: f64) -> f64 {
    if x < a {
        a
    } else if x > b {
        b
    } else {
        x
    }
}

#[no_mangle]
pub unsafe extern "C" fn Math_Clamp01(mut x: f64) -> f64 {
    if x < 0.0f64 {
        0.0f64
    } else if x > 1.0f64 {
        1.0f64
    } else {
        x
    }
}

#[no_mangle]
pub unsafe extern "C" fn Math_ClampSafe(mut x: f64, mut a: f64, mut b: f64) -> f64 {
    if b < a {
        let mut swap_temp: [libc::c_uchar; 8] = [0; 8];
        MemCpy(
            swap_temp.as_mut_ptr() as *mut libc::c_void,
            &mut b as *mut f64 as *const libc::c_void,
            std::mem::size_of::<f64>(),
        );
        MemCpy(
            &mut b as *mut f64 as *mut libc::c_void,
            &mut a as *mut f64 as *const libc::c_void,
            std::mem::size_of::<f64>(),
        );
        MemCpy(
            &mut a as *mut f64 as *mut libc::c_void,
            swap_temp.as_mut_ptr() as *const libc::c_void,
            std::mem::size_of::<f64>(),
        );
    }
    if x < a {
        a
    } else if x > b {
        b
    } else {
        x
    }
}

#[no_mangle]
pub unsafe extern "C" fn Math_ClampUnit(mut x: f64) -> f64 {
    f64::clamp(x, -1.0f64, 1.0f64)
}

#[no_mangle]
pub unsafe extern "C" fn Math_ExpMap(mut x: f64, mut p: f64) -> f64 {
    1.0f64 - f64::exp(-f64::powf(f64::abs(x), p))
}

#[no_mangle]
pub unsafe extern "C" fn Math_ExpMapSigned(mut x: f64, mut p: f64) -> f64 {
    f64::signum(x) * (1.0f64 - f64::exp(-f64::powf(f64::abs(x), p)))
}

#[no_mangle]
pub unsafe extern "C" fn Math_ExpMap1(mut x: f64) -> f64 {
    1.0f64 - f64::exp(-f64::abs(x))
}

#[no_mangle]
pub unsafe extern "C" fn Math_ExpMap1Signed(mut x: f64) -> f64 {
    f64::signum(x) * (1.0f64 - f64::exp(-f64::abs(x)))
}

#[no_mangle]
pub unsafe extern "C" fn Math_ExpMap2(mut x: f64) -> f64 {
    1.0f64 - f64::exp(-x * x)
}

#[no_mangle]
pub unsafe extern "C" fn Math_ExpMap2Signed(mut x: f64) -> f64 {
    f64::signum(x) * (1.0f64 - f64::exp(-x * x))
}

#[no_mangle]
pub unsafe extern "C" fn Math_PowSigned(mut x: f64, mut p: f64) -> f64 {
    f64::signum(x) * f64::powf(f64::abs(x), p)
}

#[no_mangle]
pub unsafe extern "C" fn Math_Round(mut x: f64) -> f64 {
    f64::round(x)
}

#[no_mangle]
pub unsafe extern "C" fn Math_Sign(mut x: f64) -> f64 {
    if x > 0.0f64 {
        1.0f64
    } else if x < 0.0f64 {
        -1.0f64
    } else {
        0.0f64
    }
}
