use super::*;
use crate::common::*;
use crate::logging::warn;
use crate::math::*;
use crate::system::*;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;

pub struct Draw {
    alpha_stack: Vec<f32>,
    color: Vec4,
    draw_color: Vec4,

    renderer: Rc<RefCell<Renderer>>,
    ids: ImmediateDrawSet,
}

static mut INSTANCE: Option<*mut Draw> = None;
impl Draw {
    pub fn new(renderer: Rc<RefCell<Renderer>>) -> Draw {
        Draw {
            alpha_stack: vec![],
            color: Vec4::ONE,
            draw_color: Vec4::ONE,
            renderer,
            ids: ImmediateDrawSet::new(),
        }
    }

    // TODO: This will eventually get refactored into a singleton inside the Lua code itself.
    pub(crate) fn set_instance(instance: *mut Draw) {
        unsafe {
            INSTANCE = Some(instance);
        }
    }

    fn inst() -> &'static Draw {
        unsafe { &*INSTANCE.expect("Draw module not initialized") }
    }

    fn inst_mut() -> &'static mut Draw {
        unsafe { &mut *INSTANCE.expect("Draw module not initialized") }
    }

    fn spherical(r: f32, yaw: f32, pitch: f32) -> Vec3 {
        Vec3::new(
            (r as f64 * f64::sin(pitch as f64) * f64::cos(yaw as f64)) as f32,
            (r as f64 * f64::cos(pitch as f64)) as f32,
            (r as f64 * f64::sin(pitch as f64) * f64::sin(yaw as f64)) as f32,
        )
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl Draw {
    pub fn push_alpha(a: f32) {
        let this = Draw::inst_mut();

        let prev_alpha = if this.alpha_stack.is_empty() {
            1.0
        } else {
            this.alpha_stack[this.alpha_stack.len() - 1]
        };

        let alpha = a * prev_alpha;
        this.alpha_stack.push(alpha);

        this.draw_color = this.color;
        this.draw_color.w *= alpha;
    }

    pub fn pop_alpha() {
        let this = Draw::inst_mut();

        if this.alpha_stack.is_empty() {
            panic!("attempting to pop an empty alpha stack");
        }

        this.alpha_stack.pop();
        let alpha: f32 = if this.alpha_stack.is_empty() {
            1.0
        } else {
            this.alpha_stack[this.alpha_stack.len() - 1]
        };

        this.draw_color = this.color;
        this.draw_color.w *= alpha;
    }

    pub fn axes(pos: &Vec3, x: &Vec3, y: &Vec3, z: &Vec3, scale: f32, alpha: f32) {
        let left: Vec3 = *pos + (*x) * scale;
        let up: Vec3 = *pos + (*y) * scale;
        let forward: Vec3 = *pos + (*z) * scale;

        let this = Draw::inst_mut();

        this.ids.begin(PrimitiveType::Lines);
        this.ids.color(1.0, 0.25, 0.25, alpha);
        this.ids.vertex3(*pos);
        this.ids.vertex3(left);
        this.ids.color(0.25, 1.0, 0.25, alpha);
        this.ids.vertex3(*pos);
        this.ids.vertex3(up);
        this.ids.color(0.25, 0.25, 1.0, alpha);
        this.ids.vertex3(*pos);
        this.ids.vertex3(forward);
        this.ids.end();

        this.ids.begin(PrimitiveType::Points);
        this.ids.color(1.0, 1.0, 1.0, alpha);
        this.ids.vertex3(*pos);
        this.ids.end();
    }

    pub fn border(s: f32, x: f32, y: f32, w: f32, h: f32) {
        Draw::rect(x, y, w, s);
        Draw::rect(x, y + h - s, w, s);
        Draw::rect(x, y + s, s, h - 2.0f32 * s);
        Draw::rect(x + w - s, y + s, s, h - 2.0f32 * s);
    }

    pub fn box3(b: &Box3) {
        unsafe { Metric_AddDrawImm(6, 12, 24) };

        let this = Draw::inst_mut();

        this.ids.begin(PrimitiveType::Quads);

        /* Left. */
        this.ids.vertex3(Vec3::new(b.lower.x, b.lower.y, b.lower.z));
        this.ids.vertex3(Vec3::new(b.lower.x, b.lower.y, b.upper.z));
        this.ids.vertex3(Vec3::new(b.lower.x, b.upper.y, b.upper.z));
        this.ids.vertex3(Vec3::new(b.lower.x, b.upper.y, b.lower.z));

        /* Right. */
        this.ids.vertex3(Vec3::new(b.upper.x, b.lower.y, b.lower.z));
        this.ids.vertex3(Vec3::new(b.upper.x, b.upper.y, b.lower.z));
        this.ids.vertex3(Vec3::new(b.upper.x, b.upper.y, b.upper.z));
        this.ids.vertex3(Vec3::new(b.upper.x, b.lower.y, b.upper.z));

        /* Front. */
        this.ids.vertex3(Vec3::new(b.lower.x, b.lower.y, b.upper.z));
        this.ids.vertex3(Vec3::new(b.upper.x, b.lower.y, b.upper.z));
        this.ids.vertex3(Vec3::new(b.upper.x, b.upper.y, b.upper.z));
        this.ids.vertex3(Vec3::new(b.lower.x, b.upper.y, b.upper.z));

        /* Back. */
        this.ids.vertex3(Vec3::new(b.lower.x, b.lower.y, b.lower.z));
        this.ids.vertex3(Vec3::new(b.lower.x, b.upper.y, b.lower.z));
        this.ids.vertex3(Vec3::new(b.upper.x, b.upper.y, b.lower.z));
        this.ids.vertex3(Vec3::new(b.upper.x, b.lower.y, b.lower.z));

        /* Top. */
        this.ids.vertex3(Vec3::new(b.lower.x, b.upper.y, b.lower.z));
        this.ids.vertex3(Vec3::new(b.lower.x, b.upper.y, b.upper.z));
        this.ids.vertex3(Vec3::new(b.upper.x, b.upper.y, b.upper.z));
        this.ids.vertex3(Vec3::new(b.upper.x, b.upper.y, b.lower.z));

        /* Bottom. */
        this.ids.vertex3(Vec3::new(b.lower.x, b.lower.y, b.lower.z));
        this.ids.vertex3(Vec3::new(b.upper.x, b.lower.y, b.lower.z));
        this.ids.vertex3(Vec3::new(b.upper.x, b.lower.y, b.upper.z));
        this.ids.vertex3(Vec3::new(b.lower.x, b.lower.y, b.upper.z));

        this.ids.end();
    }

    pub fn clear(r: f32, g: f32, b: f32, a: f32) {
        let this = Draw::inst_mut();

        let mut renderer = this.renderer.borrow_mut();
        let frame = renderer.get_frame();

        frame.start_render_pass(wgpu::Color {
            r: r as f64,
            g: g as f64,
            b: b as f64,
            a: a as f64,
        });

        // let status = gl_check_framebuffer_status(gl::FRAMEBUFFER);
        // if status != gl::FRAMEBUFFER_COMPLETE {
        //     warn!(
        //         "Framebuffer is incomplete, skipping clear. Status[{status}]: {}",
        //         framebuffer_status_to_str(status)
        //     );
        // } else {
        //     gl_clear_color(r, g, b, a);
        //     gl_clear(gl::COLOR_BUFFER_BIT);
        // };
    }

    pub fn clear_depth(d: f32) {
        // gl_clear_depth(d as f64);
        // gl_clear(gl::DEPTH_BUFFER_BIT);
    }

    pub fn color(r: f32, g: f32, b: f32, a: f32) {
        let this = Draw::inst_mut();

        let alpha = if this.alpha_stack.is_empty() {
            1.0
        } else {
            this.alpha_stack[this.alpha_stack.len() - 1]
        };

        this.color = Vec4::new(r, g, b, a);
        this.draw_color = this.color;
        this.draw_color.w *= alpha;
    }

    pub fn flush() {
        unsafe { Metric_Inc(0x6) };
        // gl_finish();
    }

    pub fn line(x1: f32, y1: f32, x2: f32, y2: f32) {
        let this = Draw::inst_mut();

        this.ids.begin(PrimitiveType::Lines);
        this.ids.vertex2(Vec2::new(x1, y1));
        this.ids.vertex2(Vec2::new(x2, y2));
        this.ids.end();
    }

    pub fn line3(p1: &Vec3, p2: &Vec3) {
        let this = Draw::inst_mut();

        this.ids.begin(PrimitiveType::Lines);
        this.ids.vertex3(*p1);
        this.ids.vertex3(*p2);
        this.ids.end();
    }

    pub fn line_width(width: f32) {
        // gl_line_width(width);
    }

    pub fn plane(p: &Vec3, n: &Vec3, scale: f32) {
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

        let this = Draw::inst_mut();

        this.ids.begin(PrimitiveType::Quads);
        this.ids.vertex3(p0);
        this.ids.vertex3(p1);
        this.ids.vertex3(p2);
        this.ids.vertex3(p3);
        this.ids.end();
    }

    pub fn point(x: f32, y: f32) {
        let this = Draw::inst_mut();
        this.ids.begin(PrimitiveType::Points);
        this.ids.vertex2(Vec2::new(x, y));
        this.ids.end();
    }

    pub fn point3(x: f32, y: f32, z: f32) {
        let this = Draw::inst_mut();
        this.ids.begin(PrimitiveType::Points);
        this.ids.vertex3(Vec3::new(x, y, z));
        this.ids.end();
    }

    pub fn point_size(size: f32) {
        // gl_point_size(size);
    }

    pub fn quad(p1: &Vec2, p2: &Vec2, p3: &Vec2, p4: &Vec2) {
        unsafe { Metric_AddDrawImm(1, 2, 4) };

        let this = Draw::inst_mut();

        this.ids.begin(PrimitiveType::Quads);
        this.ids.texcoord(Vec2::new(0.0, 0.0));
        this.ids.vertex2(*p1);
        this.ids.texcoord(Vec2::new(0.0, 1.0));
        this.ids.vertex2(*p2);
        this.ids.texcoord(Vec2::new(1.0, 1.0));
        this.ids.vertex2(*p3);
        this.ids.texcoord(Vec2::new(1.0, 0.0));
        this.ids.vertex2(*p4);
        this.ids.end();
    }

    pub fn quad3(p1: &Vec3, p2: &Vec3, p3: &Vec3, p4: &Vec3) {
        unsafe { Metric_AddDrawImm(1, 2, 4) };

        let this = Draw::inst_mut();

        this.ids.begin(PrimitiveType::Quads);
        this.ids.texcoord(Vec2::new(0.0, 0.0));
        this.ids.vertex3(*p1);
        this.ids.texcoord(Vec2::new(0.0, 1.0));
        this.ids.vertex3(*p2);
        this.ids.texcoord(Vec2::new(1.0, 1.0));
        this.ids.vertex3(*p3);
        this.ids.texcoord(Vec2::new(1.0, 0.0));
        this.ids.vertex3(*p4);
        this.ids.end();
    }

    pub fn rect(x1: f32, y1: f32, xs: f32, ys: f32) {
        let x2: f32 = x1 + xs;
        let y2: f32 = y1 + ys;

        unsafe { Metric_AddDrawImm(1, 2, 4) };

        let this = Draw::inst_mut();

        this.ids.begin(PrimitiveType::Quads);
        this.ids.texcoord(Vec2::new(0.0, 0.0));
        this.ids.vertex2(Vec2::new(x1, y1));
        this.ids.texcoord(Vec2::new(0.0, 1.0));
        this.ids.vertex2(Vec2::new(x1, y2));
        this.ids.texcoord(Vec2::new(1.0, 1.0));
        this.ids.vertex2(Vec2::new(x2, y2));
        this.ids.texcoord(Vec2::new(1.0, 0.0));
        this.ids.vertex2(Vec2::new(x2, y1));
        this.ids.end();
    }

    pub fn smooth_lines(enabled: bool) {
        // if enabled {
        //     gl_enable(gl::LINE_SMOOTH);
        //     gl_hint(gl::LINE_SMOOTH_HINT, gl::NICEST);
        // } else {
        //     gl_disable(gl::LINE_SMOOTH);
        //     gl_hint(gl::LINE_SMOOTH_HINT, gl::FASTEST);
        // };
    }

    pub fn smooth_points(enabled: bool) {
        // if enabled {
        //     gl_enable(gl::POINT_SMOOTH);
        //     gl_hint(gl::POINT_SMOOTH_HINT, gl::NICEST);
        // } else {
        //     gl_disable(gl::POINT_SMOOTH);
        //     gl_hint(gl::POINT_SMOOTH_HINT, gl::FASTEST);
        // };
    }

    /* TODO JP : Lazy creation of VBO / IBO & glDraw instead of immediate. */
    pub fn sphere(p: &Vec3, r: f32) {
        let res: usize = 7;
        let f_res: f32 = res as f32;

        let this = Draw::inst_mut();

        // First Row
        unsafe { Metric_AddDrawImm(res as i32, res as i32, res.wrapping_mul(3) as i32) };

        let mut last_theta: f32 = res.wrapping_sub(1) as f32 / f_res * std::f32::consts::TAU;
        let phi: f32 = 1.0f32 / f_res * std::f32::consts::PI;
        let tc: Vec3 = *p + Self::spherical(r, 0.0f32, 0.0f32);

        this.ids.begin(PrimitiveType::Triangles);
        for i_theta in 0..res {
            let theta: f32 = i_theta as f32 / f_res * std::f32::consts::TAU;
            let br: Vec3 = *p + Self::spherical(r, last_theta, phi);
            let bl: Vec3 = *p + Self::spherical(r, theta, phi);

            this.ids.vertex3(Vec3::new(br.x, br.y, br.z));
            this.ids.vertex3(Vec3::new(tc.x, tc.y, tc.z));
            this.ids.vertex3(Vec3::new(bl.x, bl.y, bl.z));

            last_theta = theta;
        }
        this.ids.end();

        // Middle Rows
        unsafe {
            Metric_AddDrawImm(
                res.wrapping_sub(2) as i32,
                2_usize.wrapping_mul(res.wrapping_sub(2)) as i32,
                4_usize.wrapping_mul(res.wrapping_sub(2)) as i32,
            )
        };

        let mut last_phi: f32 = 1.0f32 / f_res * std::f32::consts::PI;
        let mut last_theta: f32 = res.wrapping_sub(1) as f32 / f_res * std::f32::consts::TAU;

        this.ids.begin(PrimitiveType::Quads);
        for iPhi in 2..res {
            let phi: f32 = iPhi as f32 / f_res * std::f32::consts::PI;
            for i_theta in 0..res {
                let theta: f32 = i_theta as f32 / f_res * std::f32::consts::TAU;
                let br: Vec3 = *p + Self::spherical(r, last_theta, phi);
                let tr: Vec3 = *p + Self::spherical(r, last_theta, last_phi);
                let tl: Vec3 = *p + Self::spherical(r, theta, last_phi);
                let bl: Vec3 = *p + Self::spherical(r, theta, phi);

                this.ids.vertex3(Vec3::new(br.x, br.y, br.z));
                this.ids.vertex3(Vec3::new(tr.x, tr.y, tr.z));
                this.ids.vertex3(Vec3::new(tl.x, tl.y, tl.z));
                this.ids.vertex3(Vec3::new(bl.x, bl.y, bl.z));

                last_theta = theta;
            }
            last_phi = phi;
        }
        this.ids.end();

        // Bottom Row
        unsafe { Metric_AddDrawImm(res as i32, res as i32, res.wrapping_mul(3) as i32) };

        let mut last_theta: f32 = res.wrapping_sub(1) as f32 / f_res * std::f32::consts::TAU;
        let phi: f32 = res.wrapping_sub(1) as f32 / f_res * std::f32::consts::PI;
        let bc: Vec3 = *p + Self::spherical(r, 0.0f32, std::f32::consts::PI);

        this.ids.begin(PrimitiveType::Triangles);
        for i_theta in 0..res {
            let theta: f32 = i_theta as f32 / f_res * std::f32::consts::TAU;
            let tr: Vec3 = *p + Self::spherical(r, last_theta, phi);
            let tl: Vec3 = *p + Self::spherical(r, theta, phi);

            this.ids.vertex3(Vec3::new(tr.x, tr.y, tr.z));
            this.ids.vertex3(Vec3::new(tl.x, tl.y, tl.z));
            this.ids.vertex3(Vec3::new(bc.x, bc.y, bc.z));

            last_theta = theta;
        }
        this.ids.end();
    }

    pub fn tri(v1: &Vec2, v2: &Vec2, v3: &Vec2) {
        unsafe { Metric_AddDrawImm(1, 1, 3) };

        let this = Draw::inst_mut();

        this.ids.begin(PrimitiveType::Triangles);
        this.ids.texcoord(Vec2::new(0.0, 0.0));
        this.ids.vertex2(*v1);
        this.ids.texcoord(Vec2::new(0.0, 1.0));
        this.ids.vertex2(*v2);
        this.ids.texcoord(Vec2::new(1.0, 1.0));
        this.ids.vertex2(*v3);
        this.ids.end();
    }

    pub fn tri3(v1: &Vec3, v2: &Vec3, v3: &Vec3) {
        unsafe { Metric_AddDrawImm(1, 1, 3) };

        let this = Draw::inst_mut();

        this.ids.begin(PrimitiveType::Triangles);
        this.ids.texcoord(Vec2::new(0.0, 0.0));
        this.ids.vertex3(*v1);
        this.ids.texcoord(Vec2::new(0.0, 1.0));
        this.ids.vertex3(*v2);
        this.ids.texcoord(Vec2::new(1.0, 1.0));
        this.ids.vertex3(*v3);
        this.ids.end();
    }
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Poly(points: *const Vec2, count: i32) {
    Metric_AddDrawImm(1, count - 2, count);

    let this = Draw::inst_mut();

    this.ids.begin(PrimitiveType::Polygon);
    for i in 0..(count as isize) {
        this.ids.vertex2(*points.offset(i));
    }
    this.ids.end();
}

#[no_mangle]
pub unsafe extern "C" fn Draw_Poly3(points: *const Vec3, count: i32) {
    Metric_AddDrawImm(1, count - 2, count);

    let this = Draw::inst_mut();

    this.ids.begin(PrimitiveType::Polygon);
    for i in 0..(count as isize) {
        this.ids.vertex3(*points.offset(i));
    }
    this.ids.end();
}

// fn framebuffer_status_to_str(status: gl::types::GLenum) -> &'static str {
//     match status {
//         gl::FRAMEBUFFER_COMPLETE => "framebuffer is complete",
//         gl::FRAMEBUFFER_UNDEFINED => "the specified framebuffer is the default read or draw framebuffer, but the default framebuffer does not exist",
//         gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => "any of the framebuffer attachment points are framebuffer incomplete",
//         gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => "the framebuffer does not have at least one image attached to it",
//         gl::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => "the value of GL_FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE is GL_NONE for any color attachment point(s) named by GL_DRAW_BUFFERi",
//         gl::FRAMEBUFFER_INCOMPLETE_READ_BUFFER => "GL_READ_BUFFER is not GL_NONE and the value of GL_FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE is GL_NONE for the color attachment point named by GL_READ_BUFFER",
//         gl::FRAMEBUFFER_UNSUPPORTED => "the combination of internal formats of the attached images violates an implementation-dependent set of restrictions",
//         gl::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => "the value of GL_RENDERBUFFER_SAMPLES is not the same for all attached renderbuffers; if the value of GL_TEXTURE_SAMPLES is the not same for all attached textures; or, if the attached images are a mix of renderbuffers and textures, the value of GL_RENDERBUFFER_SAMPLES does not match the value of GL_TEXTURE_SAMPLES. Also returned if the value of GL_TEXTURE_FIXED_SAMPLE_LOCATIONS is not the same for all attached textures; or, if the attached images are a mix of renderbuffers and textures, the value of GL_TEXTURE_FIXED_SAMPLE_LOCATIONS is not GL_TRUE for all attached textures",
//         // gl::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS => "any framebuffer attachment is layered, and any populated attachment is not layered, or if all populated color attachments are not from textures of the same target",
//         _ => "Unknown",
//     }
// }
