use ::libc;
use crate::internal::Memory::*;
extern "C" {
    fn Fatal(_: cstr, _: ...);
    fn Warn(_: cstr, _: ...);
    fn cos(_: libc::c_double) -> libc::c_double;
    fn sin(_: libc::c_double) -> libc::c_double;
    fn fabs(_: libc::c_double) -> libc::c_double;
    fn sqrt(_: libc::c_double) -> libc::c_double;
    fn Metric_Inc(_: Metric);
    fn Metric_AddDrawImm(polys: int32, tris: int32, verts: int32);
    fn glBegin(mode: GLenum);
    fn glClear(mask: GLbitfield);
    fn glClearColor(red: GLclampf, green: GLclampf, blue: GLclampf, alpha: GLclampf);
    fn glClearDepth(depth: GLclampd);
    fn glColor4f(red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat);
    fn glDisable(cap: GLenum);
    fn glEnable(cap: GLenum);
    fn glEnd();
    fn glFinish();
    fn glHint(target: GLenum, mode: GLenum);
    fn glLineWidth(width: GLfloat);
    fn glPointSize(size: GLfloat);
    fn glTexCoord2f(s: GLfloat, t: GLfloat);
    fn glVertex2f(x: GLfloat, y: GLfloat);
    fn glVertex3f(x: GLfloat, y: GLfloat, z: GLfloat);
    static mut __glewCheckFramebufferStatus: PFNGLCHECKFRAMEBUFFERSTATUSPROC;
}
pub type int32_t = libc::c_int;
pub type cstr = *const libc::c_char;
pub type int32 = int32_t;
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Box3f {
    pub lower: Vec3f,
    pub upper: Vec3f,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec3f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec2f {
    pub x: libc::c_float,
    pub y: libc::c_float,
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec4f {
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub z: libc::c_float,
    pub w: libc::c_float,
}
pub type Metric = int32;
pub type GLbitfield = libc::c_uint;
pub type GLclampf = libc::c_float;
pub type GLenum = libc::c_uint;
pub type PFNGLCHECKFRAMEBUFFERSTATUSPROC = Option::<
    unsafe extern "C" fn(GLenum) -> GLenum,
>;
pub type GLclampd = libc::c_double;
pub type GLfloat = libc::c_float;
#[inline]
unsafe extern "C" fn Abs(mut t: libc::c_double) -> libc::c_double {
    return fabs(t);
}
#[inline]
unsafe extern "C" fn Sqrtf(mut t: libc::c_float) -> libc::c_float {
    return sqrt(t as libc::c_double) as libc::c_float;
}
#[inline]
unsafe extern "C" fn Cos(mut t: libc::c_double) -> libc::c_double {
    return cos(t);
}
#[inline]
unsafe extern "C" fn Sin(mut t: libc::c_double) -> libc::c_double {
    return sin(t);
}
#[inline]
unsafe extern "C" fn Vec3f_Create(
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut z: libc::c_float,
) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f { x: x, y: y, z: z };
        init
    };
    return self_0;
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
unsafe extern "C" fn Vec3f_Muls(mut a: Vec3f, mut b: libc::c_float) -> Vec3f {
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: a.x * b,
            y: a.y * b,
            z: a.z * b,
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
unsafe extern "C" fn Vec3f_Reject(mut a: Vec3f, mut b: Vec3f) -> Vec3f {
    let mut d: libc::c_float = Vec3f_Dot(a, b);
    let mut self_0: Vec3f = {
        let mut init = Vec3f {
            x: a.x - d * b.x,
            y: a.y - d * b.y,
            z: a.z - d * b.z,
        };
        init
    };
    return self_0;
}
#[inline]
unsafe extern "C" fn Vec4f_Create(
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut z: libc::c_float,
    mut w: libc::c_float,
) -> Vec4f {
    let mut self_0: Vec4f = {
        let mut init = Vec4f { x: x, y: y, z: z, w: w };
        init
    };
    return self_0;
}
static mut alphaStack: [libc::c_float; 16] = [0.; 16];
static mut alphaIndex: libc::c_int = -(1 as libc::c_int);
static mut color: Vec4f = {
    let mut init = Vec4f {
        x: 1 as libc::c_int as libc::c_float,
        y: 1 as libc::c_int as libc::c_float,
        z: 1 as libc::c_int as libc::c_float,
        w: 1 as libc::c_int as libc::c_float,
    };
    init
};
#[no_mangle]
pub unsafe extern "C" fn Draw_PushAlpha(mut a: libc::c_float) {
    if alphaIndex + 1 as libc::c_int >= 16 as libc::c_int {
        Fatal(
            b"Draw_PushAlpha: Maximum alpha stack depth exceeded\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut prevAlpha: libc::c_float = if alphaIndex >= 0 as libc::c_int {
        alphaStack[alphaIndex as usize]
    } else {
        1 as libc::c_int as libc::c_float
    };
    let mut alpha: libc::c_float = a * prevAlpha;
    alphaIndex += 1;
    alphaStack[alphaIndex as usize] = alpha;
    glColor4f(color.x, color.y, color.z, color.w * alpha);
}
#[no_mangle]
pub unsafe extern "C" fn Draw_PopAlpha() {
    if alphaIndex < 0 as libc::c_int {
        Fatal(
            b"Draw_PopAlpha Attempting to pop an empty alpha stack\0" as *const u8
                as *const libc::c_char,
        );
    }
    alphaIndex -= 1;
    let mut alpha: libc::c_float = if alphaIndex >= 0 as libc::c_int {
        alphaStack[alphaIndex as usize]
    } else {
        1 as libc::c_int as libc::c_float
    };
    glColor4f(color.x, color.y, color.z, color.w * alpha);
}
#[no_mangle]
pub unsafe extern "C" fn Draw_Axes(
    mut pos: *const Vec3f,
    mut x: *const Vec3f,
    mut y: *const Vec3f,
    mut z: *const Vec3f,
    mut scale: libc::c_float,
    mut _alpha: libc::c_float,
) {
    let mut left: Vec3f = Vec3f_Add(*pos, Vec3f_Muls(*x, scale));
    let mut up: Vec3f = Vec3f_Add(*pos, Vec3f_Muls(*y, scale));
    let mut forward: Vec3f = Vec3f_Add(*pos, Vec3f_Muls(*z, scale));
    glBegin(0x1 as libc::c_int as GLenum);
    glColor4f(1 as libc::c_int as GLfloat, 0.25f32, 0.25f32, _alpha);
    glVertex3f((*pos).x, (*pos).y, (*pos).z);
    glVertex3f(left.x, left.y, left.z);
    glColor4f(0.25f32, 1 as libc::c_int as GLfloat, 0.25f32, _alpha);
    glVertex3f((*pos).x, (*pos).y, (*pos).z);
    glVertex3f(up.x, up.y, up.z);
    glColor4f(0.25f32, 0.25f32, 1 as libc::c_int as GLfloat, _alpha);
    glVertex3f((*pos).x, (*pos).y, (*pos).z);
    glVertex3f(forward.x, forward.y, forward.z);
    glEnd();
    glBegin(0 as libc::c_int as GLenum);
    glColor4f(
        1 as libc::c_int as GLfloat,
        1 as libc::c_int as GLfloat,
        1 as libc::c_int as GLfloat,
        _alpha,
    );
    glVertex3f((*pos).x, (*pos).y, (*pos).z);
    glEnd();
}
#[no_mangle]
pub unsafe extern "C" fn Draw_Border(
    mut s: libc::c_float,
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut w: libc::c_float,
    mut h: libc::c_float,
) {
    Draw_Rect(x, y, w, s);
    Draw_Rect(x, y + h - s, w, s);
    Draw_Rect(x, y + s, s, h - 2 as libc::c_int as libc::c_float * s);
    Draw_Rect(x + w - s, y + s, s, h - 2 as libc::c_int as libc::c_float * s);
}
#[no_mangle]
pub unsafe extern "C" fn Draw_Box3(mut self_0: *const Box3f) {
    Metric_AddDrawImm(6 as libc::c_int, 12 as libc::c_int, 24 as libc::c_int);
    glBegin(0x7 as libc::c_int as GLenum);
    glVertex3f((*self_0).lower.x, (*self_0).lower.y, (*self_0).lower.z);
    glVertex3f((*self_0).lower.x, (*self_0).lower.y, (*self_0).upper.z);
    glVertex3f((*self_0).lower.x, (*self_0).upper.y, (*self_0).upper.z);
    glVertex3f((*self_0).lower.x, (*self_0).upper.y, (*self_0).lower.z);
    glVertex3f((*self_0).upper.x, (*self_0).lower.y, (*self_0).lower.z);
    glVertex3f((*self_0).upper.x, (*self_0).upper.y, (*self_0).lower.z);
    glVertex3f((*self_0).upper.x, (*self_0).upper.y, (*self_0).upper.z);
    glVertex3f((*self_0).upper.x, (*self_0).lower.y, (*self_0).upper.z);
    glVertex3f((*self_0).lower.x, (*self_0).lower.y, (*self_0).upper.z);
    glVertex3f((*self_0).upper.x, (*self_0).lower.y, (*self_0).upper.z);
    glVertex3f((*self_0).upper.x, (*self_0).upper.y, (*self_0).upper.z);
    glVertex3f((*self_0).lower.x, (*self_0).upper.y, (*self_0).upper.z);
    glVertex3f((*self_0).lower.x, (*self_0).lower.y, (*self_0).lower.z);
    glVertex3f((*self_0).lower.x, (*self_0).upper.y, (*self_0).lower.z);
    glVertex3f((*self_0).upper.x, (*self_0).upper.y, (*self_0).lower.z);
    glVertex3f((*self_0).upper.x, (*self_0).lower.y, (*self_0).lower.z);
    glVertex3f((*self_0).lower.x, (*self_0).upper.y, (*self_0).lower.z);
    glVertex3f((*self_0).lower.x, (*self_0).upper.y, (*self_0).upper.z);
    glVertex3f((*self_0).upper.x, (*self_0).upper.y, (*self_0).upper.z);
    glVertex3f((*self_0).upper.x, (*self_0).upper.y, (*self_0).lower.z);
    glVertex3f((*self_0).lower.x, (*self_0).lower.y, (*self_0).lower.z);
    glVertex3f((*self_0).upper.x, (*self_0).lower.y, (*self_0).lower.z);
    glVertex3f((*self_0).upper.x, (*self_0).lower.y, (*self_0).upper.z);
    glVertex3f((*self_0).lower.x, (*self_0).lower.y, (*self_0).upper.z);
    glEnd();
}
#[no_mangle]
pub unsafe extern "C" fn Draw_Clear(
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
) {
    let mut status: libc::c_int = __glewCheckFramebufferStatus
        .expect("non-null function pointer")(0x8d40 as libc::c_int as GLenum)
        as libc::c_int;
    if status != 0x8cd5 as libc::c_int {
        Warn(
            b"Framebuffer is incomplete, skipping clear: %d\0" as *const u8
                as *const libc::c_char,
            status,
        );
    } else {
        glClearColor(r, g, b, a);
        glClear(0x4000 as libc::c_int as GLbitfield);
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draw_ClearDepth(mut d: libc::c_float) {
    glClearDepth(d as GLclampd);
    glClear(0x100 as libc::c_int as GLbitfield);
}
#[no_mangle]
pub unsafe extern "C" fn Draw_Color(
    mut r: libc::c_float,
    mut g: libc::c_float,
    mut b: libc::c_float,
    mut a: libc::c_float,
) {
    let mut alpha: libc::c_float = if alphaIndex >= 0 as libc::c_int {
        alphaStack[alphaIndex as usize]
    } else {
        1 as libc::c_int as libc::c_float
    };
    color = Vec4f_Create(r, g, b, a);
    glColor4f(r, g, b, a * alpha);
}
#[no_mangle]
pub unsafe extern "C" fn Draw_Flush() {
    Metric_Inc(0x6 as libc::c_int);
    glFinish();
}
#[no_mangle]
pub unsafe extern "C" fn Draw_Line(
    mut x1: libc::c_float,
    mut y1: libc::c_float,
    mut x2: libc::c_float,
    mut y2: libc::c_float,
) {
    glBegin(0x1 as libc::c_int as GLenum);
    glVertex2f(x1, y1);
    glVertex2f(x2, y2);
    glEnd();
}
#[no_mangle]
pub unsafe extern "C" fn Draw_Line3(mut p1: *const Vec3f, mut p2: *const Vec3f) {
    glBegin(0x1 as libc::c_int as GLenum);
    glVertex3f((*p1).x, (*p1).y, (*p1).z);
    glVertex3f((*p2).x, (*p2).y, (*p2).z);
    glEnd();
}
#[no_mangle]
pub unsafe extern "C" fn Draw_LineWidth(mut width: libc::c_float) {
    glLineWidth(width);
}
#[no_mangle]
pub unsafe extern "C" fn Draw_Plane(
    mut p: *const Vec3f,
    mut n: *const Vec3f,
    mut scale: libc::c_float,
) {
    let mut e1: Vec3f = if Abs((*n).x as libc::c_double) < 0.7f32 as libc::c_double {
        Vec3f_Create(
            1 as libc::c_int as libc::c_float,
            0 as libc::c_int as libc::c_float,
            0 as libc::c_int as libc::c_float,
        )
    } else {
        Vec3f_Create(
            0 as libc::c_int as libc::c_float,
            1 as libc::c_int as libc::c_float,
            0 as libc::c_int as libc::c_float,
        )
    };
    e1 = Vec3f_Normalize(Vec3f_Reject(e1, *n));
    let mut e2: Vec3f = Vec3f_Cross(*n, e1);
    let mut p0: Vec3f = Vec3f_Add(
        *p,
        Vec3f_Add(Vec3f_Muls(e1, -scale), Vec3f_Muls(e2, -scale)),
    );
    let mut p1: Vec3f = Vec3f_Add(
        *p,
        Vec3f_Add(Vec3f_Muls(e1, scale), Vec3f_Muls(e2, -scale)),
    );
    let mut p2: Vec3f = Vec3f_Add(
        *p,
        Vec3f_Add(Vec3f_Muls(e1, scale), Vec3f_Muls(e2, scale)),
    );
    let mut p3: Vec3f = Vec3f_Add(
        *p,
        Vec3f_Add(Vec3f_Muls(e1, -scale), Vec3f_Muls(e2, scale)),
    );
    Metric_AddDrawImm(1 as libc::c_int, 2 as libc::c_int, 4 as libc::c_int);
    glBegin(0x7 as libc::c_int as GLenum);
    glVertex3f(p0.x, p0.y, p0.z);
    glVertex3f(p1.x, p1.y, p1.z);
    glVertex3f(p2.x, p2.y, p2.z);
    glVertex3f(p3.x, p3.y, p3.z);
    glEnd();
}
#[no_mangle]
pub unsafe extern "C" fn Draw_Point(mut x: libc::c_float, mut y: libc::c_float) {
    glBegin(0 as libc::c_int as GLenum);
    glVertex2f(x, y);
    glEnd();
}
#[no_mangle]
pub unsafe extern "C" fn Draw_Point3(
    mut x: libc::c_float,
    mut y: libc::c_float,
    mut z: libc::c_float,
) {
    glBegin(0 as libc::c_int as GLenum);
    glVertex3f(x, y, z);
    glEnd();
}
#[no_mangle]
pub unsafe extern "C" fn Draw_PointSize(mut size: libc::c_float) {
    glPointSize(size);
}
#[no_mangle]
pub unsafe extern "C" fn Draw_Poly(mut points: *const Vec2f, mut count: libc::c_int) {
    Metric_AddDrawImm(1 as libc::c_int, count - 2 as libc::c_int, count);
    glBegin(0x9 as libc::c_int as GLenum);
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < count {
        glVertex2f((*points.offset(i as isize)).x, (*points.offset(i as isize)).y);
        i += 1;
    }
    glEnd();
}
#[no_mangle]
pub unsafe extern "C" fn Draw_Poly3(mut points: *const Vec3f, mut count: libc::c_int) {
    Metric_AddDrawImm(1 as libc::c_int, count - 2 as libc::c_int, count);
    glBegin(0x9 as libc::c_int as GLenum);
    let mut i: libc::c_int = 0 as libc::c_int;
    while i < count {
        glVertex3f(
            (*points.offset(i as isize)).x,
            (*points.offset(i as isize)).y,
            (*points.offset(i as isize)).z,
        );
        i += 1;
    }
    glEnd();
}
#[no_mangle]
pub unsafe extern "C" fn Draw_Quad(
    mut p1: *const Vec2f,
    mut p2: *const Vec2f,
    mut p3: *const Vec2f,
    mut p4: *const Vec2f,
) {
    Metric_AddDrawImm(1 as libc::c_int, 2 as libc::c_int, 4 as libc::c_int);
    glBegin(0x7 as libc::c_int as GLenum);
    glTexCoord2f(0 as libc::c_int as GLfloat, 0 as libc::c_int as GLfloat);
    glVertex2f((*p1).x, (*p1).y);
    glTexCoord2f(0 as libc::c_int as GLfloat, 1 as libc::c_int as GLfloat);
    glVertex2f((*p2).x, (*p2).y);
    glTexCoord2f(1 as libc::c_int as GLfloat, 1 as libc::c_int as GLfloat);
    glVertex2f((*p3).x, (*p3).y);
    glTexCoord2f(1 as libc::c_int as GLfloat, 0 as libc::c_int as GLfloat);
    glVertex2f((*p4).x, (*p4).y);
    glEnd();
}
#[no_mangle]
pub unsafe extern "C" fn Draw_Quad3(
    mut p1: *const Vec3f,
    mut p2: *const Vec3f,
    mut p3: *const Vec3f,
    mut p4: *const Vec3f,
) {
    Metric_AddDrawImm(1 as libc::c_int, 2 as libc::c_int, 4 as libc::c_int);
    glBegin(0x7 as libc::c_int as GLenum);
    glTexCoord2f(0 as libc::c_int as GLfloat, 0 as libc::c_int as GLfloat);
    glVertex3f((*p1).x, (*p1).y, (*p1).z);
    glTexCoord2f(0 as libc::c_int as GLfloat, 1 as libc::c_int as GLfloat);
    glVertex3f((*p2).x, (*p2).y, (*p2).z);
    glTexCoord2f(1 as libc::c_int as GLfloat, 1 as libc::c_int as GLfloat);
    glVertex3f((*p3).x, (*p3).y, (*p3).z);
    glTexCoord2f(1 as libc::c_int as GLfloat, 0 as libc::c_int as GLfloat);
    glVertex3f((*p4).x, (*p4).y, (*p4).z);
    glEnd();
}
#[no_mangle]
pub unsafe extern "C" fn Draw_Rect(
    mut x1: libc::c_float,
    mut y1: libc::c_float,
    mut xs: libc::c_float,
    mut ys: libc::c_float,
) {
    let mut x2: libc::c_float = x1 + xs;
    let mut y2: libc::c_float = y1 + ys;
    Metric_AddDrawImm(1 as libc::c_int, 2 as libc::c_int, 4 as libc::c_int);
    glBegin(0x7 as libc::c_int as GLenum);
    glTexCoord2f(0 as libc::c_int as GLfloat, 0 as libc::c_int as GLfloat);
    glVertex2f(x1, y1);
    glTexCoord2f(0 as libc::c_int as GLfloat, 1 as libc::c_int as GLfloat);
    glVertex2f(x1, y2);
    glTexCoord2f(1 as libc::c_int as GLfloat, 1 as libc::c_int as GLfloat);
    glVertex2f(x2, y2);
    glTexCoord2f(1 as libc::c_int as GLfloat, 0 as libc::c_int as GLfloat);
    glVertex2f(x2, y1);
    glEnd();
}
#[no_mangle]
pub unsafe extern "C" fn Draw_SmoothLines(mut enabled: bool) {
    if enabled {
        glEnable(0xb20 as libc::c_int as GLenum);
        glHint(0xc52 as libc::c_int as GLenum, 0x1102 as libc::c_int as GLenum);
    } else {
        glDisable(0xb20 as libc::c_int as GLenum);
        glHint(0xc52 as libc::c_int as GLenum, 0x1101 as libc::c_int as GLenum);
    };
}
#[no_mangle]
pub unsafe extern "C" fn Draw_SmoothPoints(mut enabled: bool) {
    if enabled {
        glEnable(0xb10 as libc::c_int as GLenum);
        glHint(0xc51 as libc::c_int as GLenum, 0x1102 as libc::c_int as GLenum);
    } else {
        glDisable(0xb10 as libc::c_int as GLenum);
        glHint(0xc51 as libc::c_int as GLenum, 0x1101 as libc::c_int as GLenum);
    };
}
#[inline]
unsafe extern "C" fn Spherical(
    mut r: libc::c_float,
    mut yaw: libc::c_float,
    mut pitch: libc::c_float,
) -> Vec3f {
    return Vec3f_Create(
        (r as libc::c_double * Sin(pitch as libc::c_double) * Cos(yaw as libc::c_double))
            as libc::c_float,
        (r as libc::c_double * Cos(pitch as libc::c_double)) as libc::c_float,
        (r as libc::c_double * Sin(pitch as libc::c_double) * Sin(yaw as libc::c_double))
            as libc::c_float,
    );
}
#[no_mangle]
pub unsafe extern "C" fn Draw_Sphere(mut p: *const Vec3f, mut r: libc::c_float) {
    let res: libc::size_t = 7 as libc::c_int as libc::size_t;
    let fRes: libc::c_float = res as libc::c_float;
    Metric_AddDrawImm(
        res as int32,
        res as int32,
        res.wrapping_mul(3 as libc::size_t) as int32,
    );
    glBegin(0x4 as libc::c_int as GLenum);
    let mut lastTheta: libc::c_float = res
        .wrapping_sub(1 as libc::size_t) as libc::c_float / fRes
        * 6.28318531f32;
    let mut phi: libc::c_float = 1.0f32 / fRes * 3.14159265f32;
    let mut tc: Vec3f = Vec3f_Add(
        *p,
        Spherical(
            r,
            0 as libc::c_int as libc::c_float,
            0 as libc::c_int as libc::c_float,
        ),
    );
    let mut iTheta: libc::size_t = 0 as libc::c_int as libc::size_t;
    while iTheta < res {
        let mut theta: libc::c_float = iTheta as libc::c_float / fRes * 6.28318531f32;
        let mut br: Vec3f = Vec3f_Add(*p, Spherical(r, lastTheta, phi));
        let mut bl: Vec3f = Vec3f_Add(*p, Spherical(r, theta, phi));
        glVertex3f(br.x, br.y, br.z);
        glVertex3f(tc.x, tc.y, tc.z);
        glVertex3f(bl.x, bl.y, bl.z);
        lastTheta = theta;
        iTheta = iTheta.wrapping_add(1);
    }
    glEnd();
    Metric_AddDrawImm(
        res.wrapping_sub(2 as libc::size_t) as int32,
        (2 as libc::c_int as usize).wrapping_mul(res.wrapping_sub(2 as libc::size_t) as usize) as int32,
        (4 as libc::c_int as usize).wrapping_mul(res.wrapping_sub(2 as libc::size_t) as usize) as int32,
    );
    glBegin(0x7 as libc::c_int as GLenum);
    let mut lastPhi: libc::c_float = 1.0f32 / fRes * 3.14159265f32;
    let mut lastTheta_0: libc::c_float = res
        .wrapping_sub(1 as libc::size_t) as libc::c_float / fRes
        * 6.28318531f32;
    let mut iPhi: libc::size_t = 2 as libc::c_int as libc::size_t;
    while iPhi < res {
        let mut phi_0: libc::c_float = iPhi as libc::c_float / fRes * 3.14159265f32;
        let mut iTheta_0: libc::size_t = 0 as libc::c_int as libc::size_t;
        while iTheta_0 < res {
            let mut theta_0: libc::c_float = iTheta_0 as libc::c_float / fRes
                * 6.28318531f32;
            let mut br_0: Vec3f = Vec3f_Add(*p, Spherical(r, lastTheta_0, phi_0));
            let mut tr: Vec3f = Vec3f_Add(*p, Spherical(r, lastTheta_0, lastPhi));
            let mut tl: Vec3f = Vec3f_Add(*p, Spherical(r, theta_0, lastPhi));
            let mut bl_0: Vec3f = Vec3f_Add(*p, Spherical(r, theta_0, phi_0));
            glVertex3f(br_0.x, br_0.y, br_0.z);
            glVertex3f(tr.x, tr.y, tr.z);
            glVertex3f(tl.x, tl.y, tl.z);
            glVertex3f(bl_0.x, bl_0.y, bl_0.z);
            lastTheta_0 = theta_0;
            iTheta_0 = iTheta_0.wrapping_add(1);
        }
        lastPhi = phi_0;
        iPhi = iPhi.wrapping_add(1);
    }
    glEnd();
    Metric_AddDrawImm(
        res as int32,
        res as int32,
        res.wrapping_mul(3 as libc::size_t) as int32,
    );
    glBegin(0x4 as libc::c_int as GLenum);
    let mut lastTheta_1: libc::c_float = res
        .wrapping_sub(1 as libc::size_t) as libc::c_float / fRes
        * 6.28318531f32;
    let mut phi_1: libc::c_float = res.wrapping_sub(1 as libc::size_t)
        as libc::c_float / fRes * 3.14159265f32;
    let mut bc: Vec3f = Vec3f_Add(
        *p,
        Spherical(r, 0 as libc::c_int as libc::c_float, 3.14159265f32),
    );
    let mut iTheta_1: libc::size_t = 0 as libc::c_int as libc::size_t;
    while iTheta_1 < res {
        let mut theta_1: libc::c_float = iTheta_1 as libc::c_float / fRes
            * 6.28318531f32;
        let mut tr_0: Vec3f = Vec3f_Add(*p, Spherical(r, lastTheta_1, phi_1));
        let mut tl_0: Vec3f = Vec3f_Add(*p, Spherical(r, theta_1, phi_1));
        glVertex3f(tr_0.x, tr_0.y, tr_0.z);
        glVertex3f(tl_0.x, tl_0.y, tl_0.z);
        glVertex3f(bc.x, bc.y, bc.z);
        lastTheta_1 = theta_1;
        iTheta_1 = iTheta_1.wrapping_add(1);
    }
    glEnd();
}
#[no_mangle]
pub unsafe extern "C" fn Draw_Tri(
    mut v1: *const Vec2f,
    mut v2: *const Vec2f,
    mut v3: *const Vec2f,
) {
    Metric_AddDrawImm(1 as libc::c_int, 1 as libc::c_int, 3 as libc::c_int);
    glBegin(0x4 as libc::c_int as GLenum);
    glTexCoord2f(0 as libc::c_int as GLfloat, 0 as libc::c_int as GLfloat);
    glVertex2f((*v1).x, (*v1).y);
    glTexCoord2f(0 as libc::c_int as GLfloat, 1 as libc::c_int as GLfloat);
    glVertex2f((*v2).x, (*v2).y);
    glTexCoord2f(1 as libc::c_int as GLfloat, 1 as libc::c_int as GLfloat);
    glVertex2f((*v3).x, (*v3).y);
    glEnd();
}
#[no_mangle]
pub unsafe extern "C" fn Draw_Tri3(
    mut v1: *const Vec3f,
    mut v2: *const Vec3f,
    mut v3: *const Vec3f,
) {
    Metric_AddDrawImm(1 as libc::c_int, 1 as libc::c_int, 3 as libc::c_int);
    glBegin(0x4 as libc::c_int as GLenum);
    glTexCoord2f(0 as libc::c_int as GLfloat, 0 as libc::c_int as GLfloat);
    glVertex3f((*v1).x, (*v1).y, (*v1).z);
    glTexCoord2f(0 as libc::c_int as GLfloat, 1 as libc::c_int as GLfloat);
    glVertex3f((*v2).x, (*v2).y, (*v2).z);
    glTexCoord2f(1 as libc::c_int as GLfloat, 1 as libc::c_int as GLfloat);
    glVertex3f((*v3).x, (*v3).y, (*v3).z);
    glEnd();
}
