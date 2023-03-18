use crate::internal::Memory::*;
use crate::Metric::*;
use glam::Vec2;
use glam::Vec3;
use libc;

extern "C" {
    fn Fatal(_: *const libc::c_char, _: ...);
    fn Warn(_: *const libc::c_char, _: ...);
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

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Vec4f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
pub type Metric = i32;
pub type GLbitfield = u32;
pub type GLclampf = f32;
pub type GLenum = u32;
pub type PFNGLCHECKFRAMEBUFFERSTATUSPROC = Option<unsafe extern "C" fn(GLenum) -> GLenum>;
pub type GLclampd = f64;
pub type GLfloat = f32;

#[inline]
unsafe extern "C" fn Vec3_Reject(mut a: Vec3, mut b: Vec3) -> Vec3 {
    let mut d: f32 = Vec3::dot(a, b);
    let mut this: Vec3 = Vec3 {
        x: a.x - d * b.x,
        y: a.y - d * b.y,
        z: a.z - d * b.z,
    };
    this
}

#[inline]
unsafe extern "C" fn Vec4f_Create(mut x: f32, mut y: f32, mut z: f32, mut w: f32) -> Vec4f {
    let mut this: Vec4f = Vec4f {
        x: x,
        y: y,
        z: z,
        w: w,
    };
    this
}
static mut alphaStack: [f32; 16] = [0.; 16];

static mut alphaIndex: i32 = -1_i32;

static mut color: Vec4f = Vec4f {
    x: 1.0f32,
    y: 1.0f32,
    z: 1.0f32,
    w: 1.0f32,
};

#[no_mangle]
pub unsafe extern "C" fn Draw_PushAlpha(mut a: f32) {
    if alphaIndex + 1_i32 >= 16_i32 {
        Fatal(
            b"Draw_PushAlpha: Maximum alpha stack depth exceeded\0" as *const u8
                as *const libc::c_char,
        );
    }
    let mut prevAlpha: f32 = if alphaIndex >= 0_i32 {
        alphaStack[alphaIndex as usize]
    } else {
        1.0f32
    };
    let mut alpha: f32 = a * prevAlpha;
    alphaIndex += 1;
    alphaStack[alphaIndex as usize] = alpha;
    glColor4f(color.x, color.y, color.z, color.w * alpha);
}

#[no_mangle]
pub unsafe extern "C" fn Draw_PopAlpha() {
    if alphaIndex < 0_i32 {
        Fatal(
            b"Draw_PopAlpha Attempting to pop an empty alpha stack\0" as *const u8
                as *const libc::c_char,
        );
    }
    alphaIndex -= 1;
    let mut alpha: f32 = if alphaIndex >= 0_i32 {
        alphaStack[alphaIndex as usize]
    } else {
        1.0f32
    };
    glColor4f(color.x, color.y, color.z, color.w * alpha);
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Axes(
    mut pos: *const Vec3,
    mut x: *const Vec3,
    mut y: *const Vec3,
    mut z: *const Vec3,
    mut scale: f32,
    mut _alpha: f32,
) {
    let mut left: Vec3 = *pos + (*x) * scale;
    let mut up: Vec3 = *pos + (*y) * scale;
    let mut forward: Vec3 = *pos + (*z) * scale;
    glBegin(0x1_i32 as GLenum);
    glColor4f(1_i32 as GLfloat, 0.25f32, 0.25f32, _alpha);
    glVertex3f((*pos).x, (*pos).y, (*pos).z);
    glVertex3f(left.x, left.y, left.z);
    glColor4f(0.25f32, 1_i32 as GLfloat, 0.25f32, _alpha);
    glVertex3f((*pos).x, (*pos).y, (*pos).z);
    glVertex3f(up.x, up.y, up.z);
    glColor4f(0.25f32, 0.25f32, 1_i32 as GLfloat, _alpha);
    glVertex3f((*pos).x, (*pos).y, (*pos).z);
    glVertex3f(forward.x, forward.y, forward.z);
    glEnd();
    glBegin(0_i32 as GLenum);
    glColor4f(1_i32 as GLfloat, 1_i32 as GLfloat, 1_i32 as GLfloat, _alpha);
    glVertex3f((*pos).x, (*pos).y, (*pos).z);
    glEnd();
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Border(mut s: f32, mut x: f32, mut y: f32, mut w: f32, mut h: f32) {
    Draw_Rect(x, y, w, s);
    Draw_Rect(x, y + h - s, w, s);
    Draw_Rect(x, y + s, s, h - 2.0f32 * s);
    Draw_Rect(x + w - s, y + s, s, h - 2.0f32 * s);
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Box3(mut this: *const Box3f) {
    Metric_AddDrawImm(6_i32, 12_i32, 24_i32);
    glBegin(0x7_i32 as GLenum);
    glVertex3f((*this).lower.x, (*this).lower.y, (*this).lower.z);
    glVertex3f((*this).lower.x, (*this).lower.y, (*this).upper.z);
    glVertex3f((*this).lower.x, (*this).upper.y, (*this).upper.z);
    glVertex3f((*this).lower.x, (*this).upper.y, (*this).lower.z);
    glVertex3f((*this).upper.x, (*this).lower.y, (*this).lower.z);
    glVertex3f((*this).upper.x, (*this).upper.y, (*this).lower.z);
    glVertex3f((*this).upper.x, (*this).upper.y, (*this).upper.z);
    glVertex3f((*this).upper.x, (*this).lower.y, (*this).upper.z);
    glVertex3f((*this).lower.x, (*this).lower.y, (*this).upper.z);
    glVertex3f((*this).upper.x, (*this).lower.y, (*this).upper.z);
    glVertex3f((*this).upper.x, (*this).upper.y, (*this).upper.z);
    glVertex3f((*this).lower.x, (*this).upper.y, (*this).upper.z);
    glVertex3f((*this).lower.x, (*this).lower.y, (*this).lower.z);
    glVertex3f((*this).lower.x, (*this).upper.y, (*this).lower.z);
    glVertex3f((*this).upper.x, (*this).upper.y, (*this).lower.z);
    glVertex3f((*this).upper.x, (*this).lower.y, (*this).lower.z);
    glVertex3f((*this).lower.x, (*this).upper.y, (*this).lower.z);
    glVertex3f((*this).lower.x, (*this).upper.y, (*this).upper.z);
    glVertex3f((*this).upper.x, (*this).upper.y, (*this).upper.z);
    glVertex3f((*this).upper.x, (*this).upper.y, (*this).lower.z);
    glVertex3f((*this).lower.x, (*this).lower.y, (*this).lower.z);
    glVertex3f((*this).upper.x, (*this).lower.y, (*this).lower.z);
    glVertex3f((*this).upper.x, (*this).lower.y, (*this).upper.z);
    glVertex3f((*this).lower.x, (*this).lower.y, (*this).upper.z);
    glEnd();
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Clear(mut r: f32, mut g: f32, mut b: f32, mut a: f32) {
    let mut status: i32 =
        __glewCheckFramebufferStatus.expect("non-null function pointer")(0x8d40_i32 as GLenum)
            as i32;
    if status != 0x8cd5_i32 {
        Warn(
            b"Framebuffer is incomplete, skipping clear: %d\0" as *const u8 as *const libc::c_char,
            status,
        );
    } else {
        glClearColor(r, g, b, a);
        glClear(0x4000_i32 as GLbitfield);
    };
}

#[no_mangle]
pub unsafe extern "C" fn Draw_ClearDepth(mut d: f32) {
    glClearDepth(d as GLclampd);
    glClear(0x100_i32 as GLbitfield);
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Color(mut r: f32, mut g: f32, mut b: f32, mut a: f32) {
    let mut alpha: f32 = if alphaIndex >= 0_i32 {
        alphaStack[alphaIndex as usize]
    } else {
        1.0f32
    };
    color = Vec4f_Create(r, g, b, a);
    glColor4f(r, g, b, a * alpha);
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Flush() {
    Metric_Inc(0x6_i32);
    glFinish();
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Line(mut x1: f32, mut y1: f32, mut x2: f32, mut y2: f32) {
    glBegin(0x1_i32 as GLenum);
    glVertex2f(x1, y1);
    glVertex2f(x2, y2);
    glEnd();
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Line3(mut p1: *const Vec3, mut p2: *const Vec3) {
    glBegin(0x1_i32 as GLenum);
    glVertex3f((*p1).x, (*p1).y, (*p1).z);
    glVertex3f((*p2).x, (*p2).y, (*p2).z);
    glEnd();
}

#[no_mangle]
pub unsafe extern "C" fn Draw_LineWidth(mut width: f32) {
    glLineWidth(width);
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Plane(mut p: *const Vec3, mut n: *const Vec3, mut scale: f32) {
    let mut e1: Vec3 = if f64::abs((*n).x as f64) < 0.7f32 as f64 {
        Vec3::new(1.0f32, 0.0f32, 0.0f32)
    } else {
        Vec3::new(0.0f32, 1.0f32, 0.0f32)
    };
    e1 = Vec3_Reject(e1, *n).normalize();
    let mut e2: Vec3 = Vec3::cross(*n, e1);
    let mut p0: Vec3 = *p + (e1 * -scale) + (e2 * -scale);
    let mut p1: Vec3 = *p + (e1 * scale) + (e2 * -scale);
    let mut p2: Vec3 = *p + (e1 * scale) + (e2 * scale);
    let mut p3: Vec3 = *p + (e1 * -scale) + (e2 * scale);
    Metric_AddDrawImm(1_i32, 2_i32, 4_i32);
    glBegin(0x7_i32 as GLenum);
    glVertex3f(p0.x, p0.y, p0.z);
    glVertex3f(p1.x, p1.y, p1.z);
    glVertex3f(p2.x, p2.y, p2.z);
    glVertex3f(p3.x, p3.y, p3.z);
    glEnd();
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Point(mut x: f32, mut y: f32) {
    glBegin(0_i32 as GLenum);
    glVertex2f(x, y);
    glEnd();
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Point3(mut x: f32, mut y: f32, mut z: f32) {
    glBegin(0_i32 as GLenum);
    glVertex3f(x, y, z);
    glEnd();
}

#[no_mangle]
pub unsafe extern "C" fn Draw_PointSize(mut size: f32) {
    glPointSize(size);
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Poly(mut points: *const Vec2, mut count: i32) {
    Metric_AddDrawImm(1_i32, count - 2_i32, count);
    glBegin(0x9_i32 as GLenum);
    let mut i: i32 = 0_i32;
    while i < count {
        glVertex2f(
            (*points.offset(i as isize)).x,
            (*points.offset(i as isize)).y,
        );
        i += 1;
    }
    glEnd();
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Poly3(mut points: *const Vec3, mut count: i32) {
    Metric_AddDrawImm(1_i32, count - 2_i32, count);
    glBegin(0x9_i32 as GLenum);
    let mut i: i32 = 0_i32;
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
    mut p1: *const Vec2,
    mut p2: *const Vec2,
    mut p3: *const Vec2,
    mut p4: *const Vec2,
) {
    Metric_AddDrawImm(1_i32, 2_i32, 4_i32);
    glBegin(0x7_i32 as GLenum);
    glTexCoord2f(0_i32 as GLfloat, 0_i32 as GLfloat);
    glVertex2f((*p1).x, (*p1).y);
    glTexCoord2f(0_i32 as GLfloat, 1_i32 as GLfloat);
    glVertex2f((*p2).x, (*p2).y);
    glTexCoord2f(1_i32 as GLfloat, 1_i32 as GLfloat);
    glVertex2f((*p3).x, (*p3).y);
    glTexCoord2f(1_i32 as GLfloat, 0_i32 as GLfloat);
    glVertex2f((*p4).x, (*p4).y);
    glEnd();
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Quad3(
    mut p1: *const Vec3,
    mut p2: *const Vec3,
    mut p3: *const Vec3,
    mut p4: *const Vec3,
) {
    Metric_AddDrawImm(1_i32, 2_i32, 4_i32);
    glBegin(0x7_i32 as GLenum);
    glTexCoord2f(0_i32 as GLfloat, 0_i32 as GLfloat);
    glVertex3f((*p1).x, (*p1).y, (*p1).z);
    glTexCoord2f(0_i32 as GLfloat, 1_i32 as GLfloat);
    glVertex3f((*p2).x, (*p2).y, (*p2).z);
    glTexCoord2f(1_i32 as GLfloat, 1_i32 as GLfloat);
    glVertex3f((*p3).x, (*p3).y, (*p3).z);
    glTexCoord2f(1_i32 as GLfloat, 0_i32 as GLfloat);
    glVertex3f((*p4).x, (*p4).y, (*p4).z);
    glEnd();
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Rect(mut x1: f32, mut y1: f32, mut xs: f32, mut ys: f32) {
    let mut x2: f32 = x1 + xs;
    let mut y2: f32 = y1 + ys;
    Metric_AddDrawImm(1_i32, 2_i32, 4_i32);
    glBegin(0x7_i32 as GLenum);
    glTexCoord2f(0_i32 as GLfloat, 0_i32 as GLfloat);
    glVertex2f(x1, y1);
    glTexCoord2f(0_i32 as GLfloat, 1_i32 as GLfloat);
    glVertex2f(x1, y2);
    glTexCoord2f(1_i32 as GLfloat, 1_i32 as GLfloat);
    glVertex2f(x2, y2);
    glTexCoord2f(1_i32 as GLfloat, 0_i32 as GLfloat);
    glVertex2f(x2, y1);
    glEnd();
}

#[no_mangle]
pub unsafe extern "C" fn Draw_SmoothLines(mut enabled: bool) {
    if enabled {
        glEnable(0xb20_i32 as GLenum);
        glHint(0xc52_i32 as GLenum, 0x1102_i32 as GLenum);
    } else {
        glDisable(0xb20_i32 as GLenum);
        glHint(0xc52_i32 as GLenum, 0x1101_i32 as GLenum);
    };
}

#[no_mangle]
pub unsafe extern "C" fn Draw_SmoothPoints(mut enabled: bool) {
    if enabled {
        glEnable(0xb10_i32 as GLenum);
        glHint(0xc51_i32 as GLenum, 0x1102_i32 as GLenum);
    } else {
        glDisable(0xb10_i32 as GLenum);
        glHint(0xc51_i32 as GLenum, 0x1101_i32 as GLenum);
    };
}

#[inline]
unsafe extern "C" fn Spherical(mut r: f32, mut yaw: f32, mut pitch: f32) -> Vec3 {
    Vec3::new(
        (r as f64 * f64::sin(pitch as f64) * f64::cos(yaw as f64)) as f32,
        (r as f64 * f64::cos(pitch as f64)) as f32,
        (r as f64 * f64::sin(pitch as f64) * f64::sin(yaw as f64)) as f32,
    )
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Sphere(mut p: *const Vec3, mut r: f32) {
    let res: usize = 7_i32 as usize;
    let fRes: f32 = res as f32;
    Metric_AddDrawImm(res as i32, res as i32, res.wrapping_mul(3_usize) as i32);
    glBegin(0x4_i32 as GLenum);
    let mut lastTheta: f32 = res.wrapping_sub(1_usize) as f32 / fRes * std::f32::consts::TAU;
    let mut phi: f32 = 1.0f32 / fRes * std::f32::consts::PI;
    let mut tc: Vec3 = *p + Spherical(r, 0.0f32, 0.0f32);
    let mut iTheta: usize = 0_i32 as usize;
    while iTheta < res {
        let mut theta: f32 = iTheta as f32 / fRes * std::f32::consts::TAU;
        let mut br: Vec3 = *p + Spherical(r, lastTheta, phi);
        let mut bl: Vec3 = *p + Spherical(r, theta, phi);
        glVertex3f(br.x, br.y, br.z);
        glVertex3f(tc.x, tc.y, tc.z);
        glVertex3f(bl.x, bl.y, bl.z);
        lastTheta = theta;
        iTheta = iTheta.wrapping_add(1);
    }
    glEnd();
    Metric_AddDrawImm(
        res.wrapping_sub(2_usize) as i32,
        2_usize.wrapping_mul(res.wrapping_sub(2_usize)) as i32,
        4_usize.wrapping_mul(res.wrapping_sub(2_usize)) as i32,
    );
    glBegin(0x7_i32 as GLenum);
    let mut lastPhi: f32 = 1.0f32 / fRes * std::f32::consts::PI;
    let mut lastTheta_0: f32 = res.wrapping_sub(1_usize) as f32 / fRes * std::f32::consts::TAU;
    let mut iPhi: usize = 2_i32 as usize;
    while iPhi < res {
        let mut phi_0: f32 = iPhi as f32 / fRes * std::f32::consts::PI;
        let mut iTheta_0: usize = 0_i32 as usize;
        while iTheta_0 < res {
            let mut theta_0: f32 = iTheta_0 as f32 / fRes * std::f32::consts::TAU;
            let mut br_0: Vec3 = *p + Spherical(r, lastTheta_0, phi_0);
            let mut tr: Vec3 = *p + Spherical(r, lastTheta_0, lastPhi);
            let mut tl: Vec3 = *p + Spherical(r, theta_0, lastPhi);
            let mut bl_0: Vec3 = *p + Spherical(r, theta_0, phi_0);
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
    Metric_AddDrawImm(res as i32, res as i32, res.wrapping_mul(3_usize) as i32);
    glBegin(0x4_i32 as GLenum);
    let mut lastTheta_1: f32 = res.wrapping_sub(1_usize) as f32 / fRes * std::f32::consts::TAU;
    let mut phi_1: f32 = res.wrapping_sub(1_usize) as f32 / fRes * std::f32::consts::PI;
    let mut bc: Vec3 = *p + Spherical(r, 0.0f32, std::f32::consts::PI);
    let mut iTheta_1: usize = 0_i32 as usize;
    while iTheta_1 < res {
        let mut theta_1: f32 = iTheta_1 as f32 / fRes * std::f32::consts::TAU;
        let mut tr_0: Vec3 = *p + Spherical(r, lastTheta_1, phi_1);
        let mut tl_0: Vec3 = *p + Spherical(r, theta_1, phi_1);
        glVertex3f(tr_0.x, tr_0.y, tr_0.z);
        glVertex3f(tl_0.x, tl_0.y, tl_0.z);
        glVertex3f(bc.x, bc.y, bc.z);
        lastTheta_1 = theta_1;
        iTheta_1 = iTheta_1.wrapping_add(1);
    }
    glEnd();
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Tri(mut v1: *const Vec2, mut v2: *const Vec2, mut v3: *const Vec2) {
    Metric_AddDrawImm(1_i32, 1_i32, 3_i32);
    glBegin(0x4_i32 as GLenum);
    glTexCoord2f(0_i32 as GLfloat, 0_i32 as GLfloat);
    glVertex2f((*v1).x, (*v1).y);
    glTexCoord2f(0_i32 as GLfloat, 1_i32 as GLfloat);
    glVertex2f((*v2).x, (*v2).y);
    glTexCoord2f(1_i32 as GLfloat, 1_i32 as GLfloat);
    glVertex2f((*v3).x, (*v3).y);
    glEnd();
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Tri3(mut v1: *const Vec3, mut v2: *const Vec3, mut v3: *const Vec3) {
    Metric_AddDrawImm(1_i32, 1_i32, 3_i32);
    glBegin(0x4_i32 as GLenum);
    glTexCoord2f(0_i32 as GLfloat, 0_i32 as GLfloat);
    glVertex3f((*v1).x, (*v1).y, (*v1).z);
    glTexCoord2f(0_i32 as GLfloat, 1_i32 as GLfloat);
    glVertex3f((*v2).x, (*v2).y, (*v2).z);
    glTexCoord2f(1_i32 as GLfloat, 1_i32 as GLfloat);
    glVertex3f((*v3).x, (*v3).y, (*v3).z);
    glEnd();
}
