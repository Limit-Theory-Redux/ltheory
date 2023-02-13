use ::libc;
use super::internal::Memory::*;
extern "C" {
    fn memcpy(
        _: *mut libc::c_void,
        _: *const libc::c_void,
        _: libc::c_ulong,
    ) -> *mut libc::c_void;
    fn exp(_: libc::c_double) -> libc::c_double;
    fn fabs(_: libc::c_double) -> libc::c_double;
    fn pow(_: libc::c_double, _: libc::c_double) -> libc::c_double;
    fn floor(_: libc::c_double) -> libc::c_double;
}
#[inline]
unsafe extern "C" fn Floor(mut t: libc::c_double) -> libc::c_double {
    return floor(t);
}
#[inline]
unsafe extern "C" fn Abs(mut t: libc::c_double) -> libc::c_double {
    return fabs(t);
}
#[inline]
unsafe extern "C" fn Pow(
    mut t: libc::c_double,
    mut p: libc::c_double,
) -> libc::c_double {
    return pow(t, p);
}
#[inline]
unsafe extern "C" fn Lerp(
    mut a: libc::c_double,
    mut b: libc::c_double,
    mut t: libc::c_double,
) -> libc::c_double {
    return a + t * (b - a);
}
#[inline]
unsafe extern "C" fn Exp(mut t: libc::c_double) -> libc::c_double {
    return exp(t);
}
#[inline]
unsafe extern "C" fn Round(mut t: libc::c_double) -> libc::c_double {
    return Floor(t + 0.5f64);
}
#[inline]
unsafe extern "C" fn Sign2(mut x: libc::c_double) -> libc::c_double {
    return if x > 0.0f64 { 1.0f64 } else { -1.0f64 };
}
#[no_mangle]
pub unsafe extern "C" fn Math_Bezier3(
    mut x: libc::c_double,
    mut y1: libc::c_double,
    mut y2: libc::c_double,
    mut y3: libc::c_double,
) -> libc::c_double {
    let mut y12: libc::c_double = Lerp(y1, y2, x);
    let mut y23: libc::c_double = Lerp(y2, y3, x);
    return Lerp(y12, y23, x);
}
#[no_mangle]
pub unsafe extern "C" fn Math_Bezier4(
    mut x: libc::c_double,
    mut y1: libc::c_double,
    mut y2: libc::c_double,
    mut y3: libc::c_double,
    mut y4: libc::c_double,
) -> libc::c_double {
    let mut y12: libc::c_double = Lerp(y1, y2, x);
    let mut y23: libc::c_double = Lerp(y2, y3, x);
    let mut y34: libc::c_double = Lerp(y3, y4, x);
    let mut y123: libc::c_double = Lerp(y12, y23, x);
    let mut y234: libc::c_double = Lerp(y23, y34, x);
    return Lerp(y123, y234, x);
}
#[no_mangle]
pub unsafe extern "C" fn Math_Clamp(
    mut x: libc::c_double,
    mut a: libc::c_double,
    mut b: libc::c_double,
) -> libc::c_double {
    return if x < a { a } else if x > b { b } else { x };
}
#[no_mangle]
pub unsafe extern "C" fn Math_Clamp01(mut x: libc::c_double) -> libc::c_double {
    return if x < 0.0f64 { 0.0f64 } else if x > 1.0f64 { 1.0f64 } else { x };
}
#[no_mangle]
pub unsafe extern "C" fn Math_ClampSafe(
    mut x: libc::c_double,
    mut a: libc::c_double,
    mut b: libc::c_double,
) -> libc::c_double {
    if b < a {
        let mut swap_temp: [libc::c_uchar; 8] = [0; 8];
        memcpy(
            swap_temp.as_mut_ptr() as *mut libc::c_void,
            &mut b as *mut libc::c_double as *const libc::c_void,
            ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
        );
        memcpy(
            &mut b as *mut libc::c_double as *mut libc::c_void,
            &mut a as *mut libc::c_double as *const libc::c_void,
            ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
        );
        memcpy(
            &mut a as *mut libc::c_double as *mut libc::c_void,
            swap_temp.as_mut_ptr() as *const libc::c_void,
            ::core::mem::size_of::<libc::c_double>() as libc::c_ulong,
        );
    }
    return if x < a { a } else if x > b { b } else { x };
}
#[no_mangle]
pub unsafe extern "C" fn Math_ClampUnit(mut x: libc::c_double) -> libc::c_double {
    return if x < -1.0f64 { -1.0f64 } else if x > 1.0f64 { 1.0f64 } else { x };
}
#[no_mangle]
pub unsafe extern "C" fn Math_ExpMap(
    mut x: libc::c_double,
    mut p: libc::c_double,
) -> libc::c_double {
    return 1.0f64 - Exp(-Pow(Abs(x), p));
}
#[no_mangle]
pub unsafe extern "C" fn Math_ExpMapSigned(
    mut x: libc::c_double,
    mut p: libc::c_double,
) -> libc::c_double {
    return Sign2(x) * (1.0f64 - Exp(-Pow(Abs(x), p)));
}
#[no_mangle]
pub unsafe extern "C" fn Math_ExpMap1(mut x: libc::c_double) -> libc::c_double {
    return 1.0f64 - Exp(-Abs(x));
}
#[no_mangle]
pub unsafe extern "C" fn Math_ExpMap1Signed(mut x: libc::c_double) -> libc::c_double {
    return Sign2(x) * (1.0f64 - Exp(-Abs(x)));
}
#[no_mangle]
pub unsafe extern "C" fn Math_ExpMap2(mut x: libc::c_double) -> libc::c_double {
    return 1.0f64 - Exp(-x * x);
}
#[no_mangle]
pub unsafe extern "C" fn Math_ExpMap2Signed(mut x: libc::c_double) -> libc::c_double {
    return Sign2(x) * (1.0f64 - Exp(-x * x));
}
#[no_mangle]
pub unsafe extern "C" fn Math_PowSigned(
    mut x: libc::c_double,
    mut p: libc::c_double,
) -> libc::c_double {
    return Sign2(x) * Pow(Abs(x), p);
}
#[no_mangle]
pub unsafe extern "C" fn Math_Round(mut x: libc::c_double) -> libc::c_double {
    return Round(x);
}
#[no_mangle]
pub unsafe extern "C" fn Math_Sign(mut x: libc::c_double) -> libc::c_double {
    return if x > 0.0f64 { 1.0f64 } else if x < 0.0f64 { -1.0f64 } else { 0.0f64 };
}
