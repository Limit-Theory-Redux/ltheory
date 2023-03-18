use crate::internal::Memory::*;
use glam::Vec3;
use libc;

extern "C" {
    fn exp(_: f64) -> f64;
    fn fabs(_: f64) -> f64;
    fn pow(_: f64, _: f64) -> f64;
    fn floor(_: f64) -> f64;
}

#[inline]
unsafe extern "C" fn Floor(mut t: f64) -> f64 {
    floor(t)
}

#[inline]
unsafe extern "C" fn Abs(mut t: f64) -> f64 {
    fabs(t)
}

#[inline]
unsafe extern "C" fn Pow(mut t: f64, mut p: f64) -> f64 {
    pow(t, p)
}

#[inline]
unsafe extern "C" fn Lerp(mut a: f64, mut b: f64, mut t: f64) -> f64 {
    a + t * (b - a)
}

#[inline]
unsafe extern "C" fn Exp(mut t: f64) -> f64 {
    exp(t)
}

#[inline]
unsafe extern "C" fn Round(mut t: f64) -> f64 {
    Floor(t + 0.5f64)
}

#[inline]
unsafe extern "C" fn Sign2(mut x: f64) -> f64 {
    if x > 0.0f64 {
        1.0f64
    } else {
        -1.0f64
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
            ::core::mem::size_of::<f64>(),
        );
        MemCpy(
            &mut b as *mut f64 as *mut libc::c_void,
            &mut a as *mut f64 as *const libc::c_void,
            ::core::mem::size_of::<f64>(),
        );
        MemCpy(
            &mut a as *mut f64 as *mut libc::c_void,
            swap_temp.as_mut_ptr() as *const libc::c_void,
            ::core::mem::size_of::<f64>(),
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
    if x < -1.0f64 {
        -1.0f64
    } else if x > 1.0f64 {
        1.0f64
    } else {
        x
    }
}

#[no_mangle]
pub unsafe extern "C" fn Math_ExpMap(mut x: f64, mut p: f64) -> f64 {
    1.0f64 - Exp(-Pow(Abs(x), p))
}

#[no_mangle]
pub unsafe extern "C" fn Math_ExpMapSigned(mut x: f64, mut p: f64) -> f64 {
    Sign2(x) * (1.0f64 - Exp(-Pow(Abs(x), p)))
}

#[no_mangle]
pub unsafe extern "C" fn Math_ExpMap1(mut x: f64) -> f64 {
    1.0f64 - Exp(-Abs(x))
}

#[no_mangle]
pub unsafe extern "C" fn Math_ExpMap1Signed(mut x: f64) -> f64 {
    Sign2(x) * (1.0f64 - Exp(-Abs(x)))
}

#[no_mangle]
pub unsafe extern "C" fn Math_ExpMap2(mut x: f64) -> f64 {
    1.0f64 - Exp(-x * x)
}

#[no_mangle]
pub unsafe extern "C" fn Math_ExpMap2Signed(mut x: f64) -> f64 {
    Sign2(x) * (1.0f64 - Exp(-x * x))
}

#[no_mangle]
pub unsafe extern "C" fn Math_PowSigned(mut x: f64, mut p: f64) -> f64 {
    Sign2(x) * Pow(Abs(x), p)
}

#[no_mangle]
pub unsafe extern "C" fn Math_Round(mut x: f64) -> f64 {
    Round(x)
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
