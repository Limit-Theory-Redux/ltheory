use super::gl;
use super::*;
use crate::logging::warn;
use crate::math::*;
use crate::system::*;

/* TODO JP : Replace all immediates with static VBO/IBOs & glDraw*. */

const MAX_STACK_DEPTH: usize = 16;

static mut alphaStack: [f32; MAX_STACK_DEPTH] = [0.; MAX_STACK_DEPTH];
static mut alphaIndex: i32 = -1;
static mut color: Color = Color::WHITE;

#[no_mangle]
pub unsafe extern "C" fn Draw_PushAlpha(a: f32) {
    if alphaIndex + 1 >= 16 {
        panic!("Draw_PushAlpha: Maximum alpha stack depth exceeded");
    }

    let prevAlpha: f32 = if alphaIndex >= 0 {
        alphaStack[alphaIndex as usize]
    } else {
        1.0f32
    };
    let alpha: f32 = a * prevAlpha;

    alphaIndex += 1;
    alphaStack[alphaIndex as usize] = alpha;

    gl_color4f(color.r, color.g, color.b, color.a * alpha);
}

#[no_mangle]
pub unsafe extern "C" fn Draw_PopAlpha() {
    if alphaIndex < 0 {
        panic!("Draw_PopAlpha Attempting to pop an empty alpha stack");
    }

    alphaIndex -= 1;
    let alpha: f32 = if alphaIndex >= 0 {
        alphaStack[alphaIndex as usize]
    } else {
        1.0f32
    };

    gl_color4f(color.r, color.g, color.b, color.a * alpha);
}

#[no_mangle]
pub extern "C" fn Draw_Axes(pos: &Vec3, x: &Vec3, y: &Vec3, z: &Vec3, scale: f32, _alpha: f32) {
    let left: Vec3 = *pos + (*x) * scale;
    let up: Vec3 = *pos + (*y) * scale;
    let forward: Vec3 = *pos + (*z) * scale;

    gl_begin(gl::LINES);
    gl_color4f(1.0f32, 0.25f32, 0.25f32, _alpha);
    gl_vertex3f((*pos).x, (*pos).y, (*pos).z);
    gl_vertex3f(left.x, left.y, left.z);
    gl_color4f(0.25f32, 1.0f32, 0.25f32, _alpha);
    gl_vertex3f((*pos).x, (*pos).y, (*pos).z);
    gl_vertex3f(up.x, up.y, up.z);
    gl_color4f(0.25f32, 0.25f32, 1.0f32, _alpha);
    gl_vertex3f((*pos).x, (*pos).y, (*pos).z);
    gl_vertex3f(forward.x, forward.y, forward.z);
    gl_end();

    gl_begin(gl::POINTS);
    gl_color4f(1.0f32, 1.0f32, 1.0f32, _alpha);
    gl_vertex3f((*pos).x, (*pos).y, (*pos).z);
    gl_end();
}

#[no_mangle]
pub extern "C" fn Draw_Border(s: f32, x: f32, y: f32, w: f32, h: f32) {
    Draw_Rect(x, y, w, s);
    Draw_Rect(x, y + h - s, w, s);
    Draw_Rect(x, y + s, s, h - 2.0f32 * s);
    Draw_Rect(x + w - s, y + s, s, h - 2.0f32 * s);
}

#[no_mangle]
pub extern "C" fn Draw_Box3(this: &Box3) {
    unsafe { Metric_AddDrawImm(6, 12, 24) };

    gl_begin(gl::QUADS);

    /* Left. */
    gl_vertex3f(this.lower.x, this.lower.y, this.lower.z);
    gl_vertex3f(this.lower.x, this.lower.y, this.upper.z);
    gl_vertex3f(this.lower.x, this.upper.y, this.upper.z);
    gl_vertex3f(this.lower.x, this.upper.y, this.lower.z);

    /* Right. */
    gl_vertex3f(this.upper.x, this.lower.y, this.lower.z);
    gl_vertex3f(this.upper.x, this.upper.y, this.lower.z);
    gl_vertex3f(this.upper.x, this.upper.y, this.upper.z);
    gl_vertex3f(this.upper.x, this.lower.y, this.upper.z);

    /* Front. */
    gl_vertex3f(this.lower.x, this.lower.y, this.upper.z);
    gl_vertex3f(this.upper.x, this.lower.y, this.upper.z);
    gl_vertex3f(this.upper.x, this.upper.y, this.upper.z);
    gl_vertex3f(this.lower.x, this.upper.y, this.upper.z);

    /* Back. */
    gl_vertex3f(this.lower.x, this.lower.y, this.lower.z);
    gl_vertex3f(this.lower.x, this.upper.y, this.lower.z);
    gl_vertex3f(this.upper.x, this.upper.y, this.lower.z);
    gl_vertex3f(this.upper.x, this.lower.y, this.lower.z);

    /* Top. */
    gl_vertex3f(this.lower.x, this.upper.y, this.lower.z);
    gl_vertex3f(this.lower.x, this.upper.y, this.upper.z);
    gl_vertex3f(this.upper.x, this.upper.y, this.upper.z);
    gl_vertex3f(this.upper.x, this.upper.y, this.lower.z);

    /* Bottom. */
    gl_vertex3f(this.lower.x, this.lower.y, this.lower.z);
    gl_vertex3f(this.upper.x, this.lower.y, this.lower.z);
    gl_vertex3f(this.upper.x, this.lower.y, this.upper.z);
    gl_vertex3f(this.lower.x, this.lower.y, this.upper.z);

    gl_end();
}

#[no_mangle]
pub extern "C" fn Draw_Clear(r: f32, g: f32, b: f32, a: f32) {
    let status = gl_check_framebuffer_status(gl::FRAMEBUFFER);
    if status != gl::FRAMEBUFFER_COMPLETE {
        warn!(
            "Framebuffer is incomplete, skipping clear. Status[{status}]: {}",
            framebuffer_status_to_str(status)
        );
    } else {
        gl_clear_color(r, g, b, a);
        gl_clear(gl::COLOR_BUFFER_BIT);
    };
}

#[no_mangle]
pub extern "C" fn Draw_ClearDepth(d: f32) {
    gl_clear_depth(d as f64);
    gl_clear(gl::DEPTH_BUFFER_BIT);
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Color(r: f32, g: f32, b: f32, a: f32) {
    let alpha: f32 = if alphaIndex >= 0 {
        alphaStack[alphaIndex as usize]
    } else {
        1.0f32
    };
    color = Color::new(r, g, b, a);

    gl_color4f(r, g, b, a * alpha);
}

#[no_mangle]
pub extern "C" fn Draw_Flush() {
    unsafe { Metric_Inc(0x6) };
    gl_finish();
}

#[no_mangle]
pub extern "C" fn Draw_Line(x1: f32, y1: f32, x2: f32, y2: f32) {
    gl_begin(gl::LINES);
    gl_vertex2f(x1, y1);
    gl_vertex2f(x2, y2);
    gl_end();
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Line3(p1: &Vec3, p2: &Vec3) {
    gl_begin(gl::LINES);
    gl_vertex3f(p1.x, p1.y, p1.z);
    gl_vertex3f(p2.x, p2.y, p2.z);
    gl_end();
}

#[no_mangle]
pub extern "C" fn Draw_LineWidth(width: f32) {
    gl_line_width(width);
}

#[no_mangle]
pub extern "C" fn Draw_Plane(p: &Vec3, n: &Vec3, scale: f32) {
    let mut e1: Vec3 = if f64::abs(n.x as f64) < 0.7f64 {
        Vec3::X
    } else {
        Vec3::Y
    };
    e1 = Vec3_Reject(e1, *n).normalize();
    let e2: Vec3 = Vec3::cross(*n, e1);

    let p0: Vec3 = *p + (e1 * -scale) + (e2 * -scale);
    let p1: Vec3 = *p + (e1 * scale) + (e2 * -scale);
    let p2: Vec3 = *p + (e1 * scale) + (e2 * scale);
    let p3: Vec3 = *p + (e1 * -scale) + (e2 * scale);

    unsafe { Metric_AddDrawImm(1, 2, 4) };

    gl_begin(gl::QUADS);
    gl_vertex3f(p0.x, p0.y, p0.z);
    gl_vertex3f(p1.x, p1.y, p1.z);
    gl_vertex3f(p2.x, p2.y, p2.z);
    gl_vertex3f(p3.x, p3.y, p3.z);
    gl_end();
}

#[no_mangle]
pub extern "C" fn Draw_Point(x: f32, y: f32) {
    gl_begin(gl::POINTS);
    gl_vertex2f(x, y);
    gl_end();
}

#[no_mangle]
pub extern "C" fn Draw_Point3(x: f32, y: f32, z: f32) {
    gl_begin(gl::POINTS);
    gl_vertex3f(x, y, z);
    gl_end();
}

#[no_mangle]
pub extern "C" fn Draw_PointSize(size: f32) {
    gl_point_size(size);
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Poly(points: *const Vec2, count: i32) {
    Metric_AddDrawImm(1, count - 2, count);

    gl_begin(gl::POLYGON);
    for i in 0..(count as isize) {
        gl_vertex2f((*points.offset(i)).x, (*points.offset(i)).y);
    }
    gl_end();
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Poly3(points: *const Vec3, count: i32) {
    Metric_AddDrawImm(1, count - 2, count);

    gl_begin(gl::POLYGON);
    for i in 0..(count as isize) {
        gl_vertex3f(
            (*points.offset(i)).x,
            (*points.offset(i)).y,
            (*points.offset(i)).z,
        );
    }
    gl_end();
}

#[no_mangle]
pub extern "C" fn Draw_Quad(p1: &Vec2, p2: &Vec2, p3: &Vec2, p4: &Vec2) {
    unsafe { Metric_AddDrawImm(1, 2, 4) };

    gl_begin(gl::QUADS);
    gl_tex_coord2f(0.0f32, 0.0f32);
    gl_vertex2f((*p1).x, (*p1).y);
    gl_tex_coord2f(0.0f32, 1.0f32);
    gl_vertex2f((*p2).x, (*p2).y);
    gl_tex_coord2f(1.0f32, 1.0f32);
    gl_vertex2f((*p3).x, (*p3).y);
    gl_tex_coord2f(1.0f32, 0.0f32);
    gl_vertex2f((*p4).x, (*p4).y);
    gl_end();
}

#[no_mangle]
pub extern "C" fn Draw_Quad3(p1: &Vec3, p2: &Vec3, p3: &Vec3, p4: &Vec3) {
    unsafe { Metric_AddDrawImm(1, 2, 4) };

    gl_begin(gl::QUADS);
    gl_tex_coord2f(0.0f32, 0.0f32);
    gl_vertex3f((*p1).x, (*p1).y, (*p1).z);
    gl_tex_coord2f(0.0f32, 1.0f32);
    gl_vertex3f((*p2).x, (*p2).y, (*p2).z);
    gl_tex_coord2f(1.0f32, 1.0f32);
    gl_vertex3f((*p3).x, (*p3).y, (*p3).z);
    gl_tex_coord2f(1.0f32, 0.0f32);
    gl_vertex3f((*p4).x, (*p4).y, (*p4).z);
    gl_end();
}

#[no_mangle]
pub extern "C" fn Draw_Rect(x1: f32, y1: f32, xs: f32, ys: f32) {
    let x2: f32 = x1 + xs;
    let y2: f32 = y1 + ys;

    unsafe { Metric_AddDrawImm(1, 2, 4) };

    gl_begin(gl::QUADS);
    gl_tex_coord2f(0.0f32, 0.0f32);
    gl_vertex2f(x1, y1);
    gl_tex_coord2f(0.0f32, 1.0f32);
    gl_vertex2f(x1, y2);
    gl_tex_coord2f(1.0f32, 1.0f32);
    gl_vertex2f(x2, y2);
    gl_tex_coord2f(1.0f32, 0.0f32);
    gl_vertex2f(x2, y1);
    gl_end();
}

#[no_mangle]
pub extern "C" fn Draw_SmoothLines(enabled: bool) {
    if enabled {
        gl_enable(gl::LINE_SMOOTH);
        gl_hint(gl::LINE_SMOOTH_HINT, gl::NICEST);
    } else {
        gl_disable(gl::LINE_SMOOTH);
        gl_hint(gl::LINE_SMOOTH_HINT, gl::FASTEST);
    };
}

#[no_mangle]
pub extern "C" fn Draw_SmoothPoints(enabled: bool) {
    if enabled {
        gl_enable(gl::POINT_SMOOTH);
        gl_hint(gl::POINT_SMOOTH_HINT, gl::NICEST);
    } else {
        gl_disable(gl::POINT_SMOOTH);
        gl_hint(gl::POINT_SMOOTH_HINT, gl::FASTEST);
    };
}

#[inline]
fn Spherical(r: f32, yaw: f32, pitch: f32) -> Vec3 {
    Vec3::new(
        (r as f64 * f64::sin(pitch as f64) * f64::cos(yaw as f64)) as f32,
        (r as f64 * f64::cos(pitch as f64)) as f32,
        (r as f64 * f64::sin(pitch as f64) * f64::sin(yaw as f64)) as f32,
    )
}

/* TODO JP : Lazy creation of VBO / IBO & glDraw instead of immediate. */
#[no_mangle]
pub unsafe extern "C" fn Draw_Sphere(p: *const Vec3, r: f32) {
    let res: usize = 7;
    let fRes: f32 = res as f32;

    // First Row
    Metric_AddDrawImm(res as i32, res as i32, res.wrapping_mul(3) as i32);

    let mut lastTheta: f32 = res.wrapping_sub(1) as f32 / fRes * std::f32::consts::TAU;
    let phi: f32 = 1.0f32 / fRes * std::f32::consts::PI;
    let tc: Vec3 = *p + Spherical(r, 0.0f32, 0.0f32);

    gl_begin(gl::TRIANGLES);
    for iTheta in 0..res {
        let theta: f32 = iTheta as f32 / fRes * std::f32::consts::TAU;
        let br: Vec3 = *p + Spherical(r, lastTheta, phi);
        let bl: Vec3 = *p + Spherical(r, theta, phi);

        gl_vertex3f(br.x, br.y, br.z);
        gl_vertex3f(tc.x, tc.y, tc.z);
        gl_vertex3f(bl.x, bl.y, bl.z);

        lastTheta = theta;
    }
    gl_end();

    // Middle Rows
    Metric_AddDrawImm(
        res.wrapping_sub(2) as i32,
        2_usize.wrapping_mul(res.wrapping_sub(2)) as i32,
        4_usize.wrapping_mul(res.wrapping_sub(2)) as i32,
    );

    let mut lastPhi: f32 = 1.0f32 / fRes * std::f32::consts::PI;
    let mut lastTheta: f32 = res.wrapping_sub(1) as f32 / fRes * std::f32::consts::TAU;

    gl_begin(gl::QUADS);
    for iPhi in 2..res {
        let phi: f32 = iPhi as f32 / fRes * std::f32::consts::PI;
        for iTheta in 0..res {
            let theta: f32 = iTheta as f32 / fRes * std::f32::consts::TAU;
            let br: Vec3 = *p + Spherical(r, lastTheta, phi);
            let tr: Vec3 = *p + Spherical(r, lastTheta, lastPhi);
            let tl: Vec3 = *p + Spherical(r, theta, lastPhi);
            let bl: Vec3 = *p + Spherical(r, theta, phi);

            gl_vertex3f(br.x, br.y, br.z);
            gl_vertex3f(tr.x, tr.y, tr.z);
            gl_vertex3f(tl.x, tl.y, tl.z);
            gl_vertex3f(bl.x, bl.y, bl.z);

            lastTheta = theta;
        }
        lastPhi = phi;
    }
    gl_end();

    // Bottom Row
    Metric_AddDrawImm(res as i32, res as i32, res.wrapping_mul(3) as i32);

    let mut lastTheta: f32 = res.wrapping_sub(1) as f32 / fRes * std::f32::consts::TAU;
    let phi: f32 = res.wrapping_sub(1) as f32 / fRes * std::f32::consts::PI;
    let bc: Vec3 = *p + Spherical(r, 0.0f32, std::f32::consts::PI);

    gl_begin(gl::TRIANGLES);
    for iTheta in 0..res {
        let theta: f32 = iTheta as f32 / fRes * std::f32::consts::TAU;
        let tr: Vec3 = *p + Spherical(r, lastTheta, phi);
        let tl: Vec3 = *p + Spherical(r, theta, phi);

        gl_vertex3f(tr.x, tr.y, tr.z);
        gl_vertex3f(tl.x, tl.y, tl.z);
        gl_vertex3f(bc.x, bc.y, bc.z);

        lastTheta = theta;
    }
    gl_end();
}

#[no_mangle]
pub extern "C" fn Draw_Tri(v1: &Vec2, v2: &Vec2, v3: &Vec2) {
    unsafe { Metric_AddDrawImm(1, 1, 3) };

    gl_begin(gl::TRIANGLES);
    gl_tex_coord2f(0.0f32, 0.0f32);
    gl_vertex2f((*v1).x, (*v1).y);
    gl_tex_coord2f(0.0f32, 1.0f32);
    gl_vertex2f((*v2).x, (*v2).y);
    gl_tex_coord2f(1.0f32, 1.0f32);
    gl_vertex2f((*v3).x, (*v3).y);
    gl_end();
}

#[no_mangle]
pub extern "C" fn Draw_Tri3(v1: &Vec3, v2: &Vec3, v3: &Vec3) {
    unsafe { Metric_AddDrawImm(1, 1, 3) };

    gl_begin(gl::TRIANGLES);
    gl_tex_coord2f(0.0f32, 0.0f32);
    gl_vertex3f((*v1).x, (*v1).y, (*v1).z);
    gl_tex_coord2f(0.0f32, 1.0f32);
    gl_vertex3f((*v2).x, (*v2).y, (*v2).z);
    gl_tex_coord2f(1.0f32, 1.0f32);
    gl_vertex3f((*v3).x, (*v3).y, (*v3).z);
    gl_end();
}

fn framebuffer_status_to_str(status: gl::types::GLenum) -> &'static str {
    match status {
        gl::FRAMEBUFFER_COMPLETE => "framebuffer is complete",
        gl::FRAMEBUFFER_UNDEFINED => "the specified framebuffer is the default read or draw framebuffer, but the default framebuffer does not exist",
        gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => "any of the framebuffer attachment points are framebuffer incomplete",
        gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => "the framebuffer does not have at least one image attached to it",
        gl::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => "the value of GL_FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE is GL_NONE for any color attachment point(s) named by GL_DRAW_BUFFERi",
        gl::FRAMEBUFFER_INCOMPLETE_READ_BUFFER => "GL_READ_BUFFER is not GL_NONE and the value of GL_FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE is GL_NONE for the color attachment point named by GL_READ_BUFFER",
        gl::FRAMEBUFFER_UNSUPPORTED => "the combination of internal formats of the attached images violates an implementation-dependent set of restrictions",
        gl::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => "the value of GL_RENDERBUFFER_SAMPLES is not the same for all attached renderbuffers; if the value of GL_TEXTURE_SAMPLES is the not same for all attached textures; or, if the attached images are a mix of renderbuffers and textures, the value of GL_RENDERBUFFER_SAMPLES does not match the value of GL_TEXTURE_SAMPLES. Also returned if the value of GL_TEXTURE_FIXED_SAMPLE_LOCATIONS is not the same for all attached textures; or, if the attached images are a mix of renderbuffers and textures, the value of GL_TEXTURE_FIXED_SAMPLE_LOCATIONS is not GL_TRUE for all attached textures",
        // gl::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS => "any framebuffer attachment is layered, and any populated attachment is not layered, or if all populated color attachments are not from textures of the same target",
        _ => "Unknown",
    }
}
