use ::libc;
use crate::internal::Memory::*;
extern "C" {
    fn __fpclassifyf(_: libc::c_float) -> libc::c_int;
    fn __fpclassifyd(_: libc::c_double) -> libc::c_int;
    fn sqrt(_: libc::c_double) -> libc::c_double;
    fn Fatal(_: cstr, _: ...);
}
pub type int32_t = libc::c_int;
pub type uint32_t = libc::c_uint;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
pub type uint32 = uint32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec3f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Plane {
    pub n: Vec3f,
    pub d: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Triangle {
    pub vertices: [Vec3f; 3],
}
pub type Error = uint32;
#[inline]
unsafe extern "C" fn Sqrtf(mut t: libc::c_float) -> libc::c_float {
    return sqrt(t as libc::c_double) as libc::c_float;
}
#[inline]
unsafe extern "C" fn Min(
    mut a: libc::c_double,
    mut b: libc::c_double,
) -> libc::c_double {
    return if a < b { a } else { b };
}
#[inline]
unsafe extern "C" fn Float_Validatef(mut x: libc::c_float) -> Error {
    let mut classification: libc::c_int = if ::core::mem::size_of::<libc::c_float>()
        as libc::c_ulong == ::core::mem::size_of::<libc::c_float>() as libc::c_ulong
    {
        __fpclassifyf(x)
    } else if ::core::mem::size_of::<libc::c_float>() as libc::c_ulong
        == ::core::mem::size_of::<libc::c_double>() as libc::c_ulong
    {
        __fpclassifyd(x as libc::c_double)
    } else {3
    };
    match classification {
        2 => return 0x4 as libc::c_int as Error,
        5 => {}
        1 => return 0x20 as libc::c_int as Error,
        3 | 4 => return 0 as libc::c_int as Error,
        _ => {
            Fatal(
                b"Float_Validate: Unhandled case: %i\0" as *const u8
                    as *const libc::c_char,
                classification,
            );
        }
    }
    return 0 as libc::c_int as Error;
}
#[inline]
unsafe extern "C" fn Vec3f_Add(mut a: Vec3f, mut b: Vec3f) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: a.x + b.x,
            y: a.y + b.y,
            z: a.z + b.z,
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Vec3f_Sub(mut a: Vec3f, mut b: Vec3f) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: a.x - b.x,
            y: a.y - b.y,
            z: a.z - b.z,
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Vec3f_Divs(mut a: Vec3f, mut b: libc::c_float) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: a.x / b,
            y: a.y / b,
            z: a.z / b,
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Vec3f_Cross(mut a: Vec3f, mut b: Vec3f) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: b.z * a.y - b.y * a.z,
            y: b.x * a.z - b.z * a.x,
            z: b.y * a.x - b.x * a.y,
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Vec3f_Equal(mut a: Vec3f, mut b: Vec3f) -> bool {
    return a.x == b.x && a.y == b.y && a.z == b.z;
}
#[inline]
unsafe extern "C" fn Vec3f_Dot(mut a: Vec3f, mut b: Vec3f) -> libc::c_float {
    return a.x * b.x + a.y * b.y + a.z * b.z;
}
#[inline]
unsafe extern "C" fn Vec3f_Length(mut v: Vec3f) -> libc::c_float {
    return Sqrtf(v.x * v.x + v.y * v.y + v.z * v.z);
}
#[inline]
unsafe extern "C" fn Vec3f_Normalize(mut v: Vec3f) -> Vec3f {
    let mut l: libc::c_float = Vec3f_Length(v);
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: v.x / l,
            y: v.y / l,
            z: v.z / l,
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Vec3f_Validate(mut v: Vec3f) -> Error {
    let mut e: Error = 0 as libc::c_int as Error;
    e |= Float_Validatef(v.x);
    e |= Float_Validatef(v.y);
    e |= Float_Validatef(v.z);
    return e;
}
#[no_mangle]
pub unsafe extern "C" fn Triangle_ToPlane(
    mut tri: *const Triangle,
    mut plane: *mut Plane,
) {
    let mut v: *const Vec3f = ((*tri).vertices).as_ptr();
    let mut e1: Vec3f = Vec3f_Sub(
        *v.offset(1 as libc::c_int as isize),
        *v.offset(0 as libc::c_int as isize),
    );
    let mut e2: Vec3f = Vec3f_Sub(
        *v.offset(2 as libc::c_int as isize),
        *v.offset(0 as libc::c_int as isize),
    );
    let mut n: Vec3f = Vec3f_Cross(e1, e2);
    n = Vec3f_Normalize(n);
    let mut centroid: Vec3f = *v.offset(0 as libc::c_int as isize);
    centroid = Vec3f_Add(centroid, *v.offset(1 as libc::c_int as isize));
    centroid = Vec3f_Add(centroid, *v.offset(2 as libc::c_int as isize));
    centroid = Vec3f_Divs(centroid, 3.0f32);
    (*plane).n = n;
    (*plane).d = Vec3f_Dot(centroid, n);
}
#[no_mangle]
pub unsafe extern "C" fn Triangle_ToPlaneFast(
    mut triangle: *const Triangle,
    mut plane: *mut Plane,
) {
    let mut v: *const Vec3f = ((*triangle).vertices).as_ptr();
    let mut e1: Vec3f = Vec3f_Sub(
        *v.offset(1 as libc::c_int as isize),
        *v.offset(0 as libc::c_int as isize),
    );
    let mut e2: Vec3f = Vec3f_Sub(
        *v.offset(2 as libc::c_int as isize),
        *v.offset(0 as libc::c_int as isize),
    );
    let mut n: Vec3f = Vec3f_Cross(e1, e2);
    (*plane).n = n;
    (*plane).d = Vec3f_Dot(*v.offset(0 as libc::c_int as isize), n);
}
#[no_mangle]
pub unsafe extern "C" fn Triangle_GetArea(mut tri: *const Triangle) -> libc::c_float {
    let mut e1: Vec3f = Vec3f_Sub(
        (*tri).vertices[1 as libc::c_int as usize],
        (*tri).vertices[0 as libc::c_int as usize],
    );
    let mut e2: Vec3f = Vec3f_Sub(
        (*tri).vertices[2 as libc::c_int as usize],
        (*tri).vertices[1 as libc::c_int as usize],
    );
    return 0.5f32 * Vec3f_Length(Vec3f_Cross(e1, e2));
}
#[no_mangle]
pub unsafe extern "C" fn Triangle_Validate(mut tri: *const Triangle) -> Error {
    let mut v: *const Vec3f = ((*tri).vertices).as_ptr();
    let mut i: int32 = 0 as libc::c_int;
    while i < 3 as libc::c_int {
        let mut e: Error = Vec3f_Validate(*v.offset(i as isize));
        if e != 0 as libc::c_int as libc::c_uint {
            return 0x400000 as libc::c_int as libc::c_uint | e;
        }
        i += 1;
    }
    let mut eq01: bool = Vec3f_Equal(
        *v.offset(0 as libc::c_int as isize),
        *v.offset(1 as libc::c_int as isize),
    );
    let mut eq12: bool = Vec3f_Equal(
        *v.offset(1 as libc::c_int as isize),
        *v.offset(2 as libc::c_int as isize),
    );
    let mut eq20: bool = Vec3f_Equal(
        *v.offset(2 as libc::c_int as isize),
        *v.offset(0 as libc::c_int as isize),
    );
    if eq01 as libc::c_int != 0 || eq12 as libc::c_int != 0 || eq20 as libc::c_int != 0 {
        return (0x400000 as libc::c_int | 0x40 as libc::c_int) as Error;
    }
    let mut e01: libc::c_float = Vec3f_Length(
        Vec3f_Sub(
            *v.offset(0 as libc::c_int as isize),
            *v.offset(1 as libc::c_int as isize),
        ),
    );
    let mut e12: libc::c_float = Vec3f_Length(
        Vec3f_Sub(
            *v.offset(1 as libc::c_int as isize),
            *v.offset(2 as libc::c_int as isize),
        ),
    );
    let mut e20: libc::c_float = Vec3f_Length(
        Vec3f_Sub(
            *v.offset(2 as libc::c_int as isize),
            *v.offset(0 as libc::c_int as isize),
        ),
    );
    let mut shortest: libc::c_float = Min(
        Min(e01 as libc::c_double, e12 as libc::c_double),
        e20 as libc::c_double,
    ) as libc::c_float;
    if (shortest as libc::c_double) < 0.75f32 as libc::c_double * 1e-4f64 {
        return (0x400000 as libc::c_int | 0x8 as libc::c_int) as Error;
    }
    return 0 as libc::c_int as Error;
}
