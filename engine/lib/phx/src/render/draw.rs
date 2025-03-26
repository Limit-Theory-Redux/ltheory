use std::sync::{Mutex, MutexGuard, OnceLock};

use super::*;
use crate::logging::*;
use crate::math::*;
use crate::system::*;

pub struct Draw {
    alpha_stack: Vec<f32>,
    color: Color,

    pb: PrimitiveBuilder,
}

impl Draw {
    fn inst() -> MutexGuard<'static, Draw> {
        static INST: OnceLock<Mutex<Draw>> = OnceLock::new();
        INST.get_or_init(|| {
            Mutex::new(Draw {
                alpha_stack: vec![],
                color: Color::WHITE,
                pb: PrimitiveBuilder::new(),
            })
        })
        .lock()
        .unwrap()
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
    pub fn clear(r: f32, g: f32, b: f32, a: f32) {
        let status = unsafe { gl::CheckFramebufferStatus(gl::FRAMEBUFFER) };
        if status == gl::FRAMEBUFFER_COMPLETE {
            glcheck!(gl::ClearColor(r, g, b, a));
            glcheck!(gl::Clear(gl::COLOR_BUFFER_BIT));
        } else {
            warn!(
                "Framebuffer is incomplete, skipping clear. Status[{status}]: {}",
                framebuffer_status_to_str(status)
            );
        }
    }

    pub fn clear_depth(d: f32) {
        glcheck!(gl::ClearDepth(d as f64));
        glcheck!(gl::Clear(gl::DEPTH_BUFFER_BIT));
    }

    pub fn color(r: f32, g: f32, b: f32, a: f32) {
        let mut this = Self::inst();

        let alpha = if this.alpha_stack.is_empty() {
            1.0
        } else {
            this.alpha_stack[this.alpha_stack.len() - 1]
        };

        this.color = Color::new(r, g, b, a);
        this.pb.color4(r, g, b, a * alpha);
    }

    pub fn flush() {
        Metric::Flush.inc();
        glcheck!(gl::Finish());
    }

    pub fn push_alpha(a: f32) {
        let mut this = Self::inst();

        let prev_alpha = if this.alpha_stack.is_empty() {
            1.0
        } else {
            this.alpha_stack[this.alpha_stack.len() - 1]
        };

        let alpha = a * prev_alpha;
        this.alpha_stack.push(alpha);

        let mut color = this.color;
        color.a *= alpha;
        this.pb.color4(color.r, color.g, color.b, color.a);
    }

    pub fn pop_alpha() {
        let mut this = Self::inst();

        if this.alpha_stack.is_empty() {
            panic!("attempting to pop an empty alpha stack");
        }

        this.alpha_stack.pop();
        let alpha: f32 = if this.alpha_stack.is_empty() {
            1.0
        } else {
            this.alpha_stack[this.alpha_stack.len() - 1]
        };

        let mut color = this.color;
        color.a *= alpha;
        this.pb.color4(color.r, color.g, color.b, color.a);
    }

    pub fn line_width(width: f32) {
        glcheck!(gl::LineWidth(width));
    }

    pub fn point_size(size: f32) {
        glcheck!(gl::PointSize(size));
    }

    pub fn axes(pos: &Vec3, x: &Vec3, y: &Vec3, z: &Vec3, scale: f32, alpha: f32) {
        let left: Vec3 = *pos + (*x) * scale;
        let up: Vec3 = *pos + (*y) * scale;
        let forward: Vec3 = *pos + (*z) * scale;

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Lines);
        this.pb.color4(1.0, 0.25, 0.25, alpha);
        this.pb.vertex3(pos.x, pos.y, pos.z);
        this.pb.vertex3(left.x, left.y, left.z);
        this.pb.color4(0.25, 1.0, 0.25, alpha);
        this.pb.vertex3(pos.x, pos.y, pos.z);
        this.pb.vertex3(up.x, up.y, up.z);
        this.pb.color4(0.25, 0.25, 1.0, alpha);
        this.pb.vertex3(pos.x, pos.y, pos.z);
        this.pb.vertex3(forward.x, forward.y, forward.z);
        this.pb.end();

        this.pb.begin(PrimitiveType::Points);
        this.pb.color4(1.0, 1.0, 1.0, alpha);
        this.pb.vertex3(pos.x, pos.y, pos.z);
        this.pb.end();
    }

    pub fn border(s: f32, x: f32, y: f32, w: f32, h: f32) {
        Draw::rect(x, y, w, s);
        Draw::rect(x, y + h - s, w, s);
        Draw::rect(x, y + s, s, h - 2.0 * s);
        Draw::rect(x + w - s, y + s, s, h - 2.0 * s);
    }

    pub fn box3(b: &Box3) {
        Metric::add_draw_imm(6, 12, 24);

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Quads);

        /* Left. */
        this.pb.vertex3(b.lower.x, b.lower.y, b.lower.z);
        this.pb.vertex3(b.lower.x, b.lower.y, b.upper.z);
        this.pb.vertex3(b.lower.x, b.upper.y, b.upper.z);
        this.pb.vertex3(b.lower.x, b.upper.y, b.lower.z);

        /* Right. */
        this.pb.vertex3(b.upper.x, b.lower.y, b.lower.z);
        this.pb.vertex3(b.upper.x, b.upper.y, b.lower.z);
        this.pb.vertex3(b.upper.x, b.upper.y, b.upper.z);
        this.pb.vertex3(b.upper.x, b.lower.y, b.upper.z);

        /* Front. */
        this.pb.vertex3(b.lower.x, b.lower.y, b.upper.z);
        this.pb.vertex3(b.upper.x, b.lower.y, b.upper.z);
        this.pb.vertex3(b.upper.x, b.upper.y, b.upper.z);
        this.pb.vertex3(b.lower.x, b.upper.y, b.upper.z);

        /* Back. */
        this.pb.vertex3(b.lower.x, b.lower.y, b.lower.z);
        this.pb.vertex3(b.lower.x, b.upper.y, b.lower.z);
        this.pb.vertex3(b.upper.x, b.upper.y, b.lower.z);
        this.pb.vertex3(b.upper.x, b.lower.y, b.lower.z);

        /* Top. */
        this.pb.vertex3(b.lower.x, b.upper.y, b.lower.z);
        this.pb.vertex3(b.lower.x, b.upper.y, b.upper.z);
        this.pb.vertex3(b.upper.x, b.upper.y, b.upper.z);
        this.pb.vertex3(b.upper.x, b.upper.y, b.lower.z);

        /* Bottom. */
        this.pb.vertex3(b.lower.x, b.lower.y, b.lower.z);
        this.pb.vertex3(b.upper.x, b.lower.y, b.lower.z);
        this.pb.vertex3(b.upper.x, b.lower.y, b.upper.z);
        this.pb.vertex3(b.lower.x, b.lower.y, b.upper.z);

        this.pb.end();
    }

    pub fn line(x1: f32, y1: f32, x2: f32, y2: f32) {
        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Lines);
        this.pb.vertex2(x1, y1);
        this.pb.vertex2(x2, y2);
        this.pb.end();
    }

    pub fn line3(p1: &Vec3, p2: &Vec3) {
        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Lines);
        this.pb.vertex3(p1.x, p1.y, p1.z);
        this.pb.vertex3(p2.x, p2.y, p2.z);
        this.pb.end();
    }

    pub fn plane(p: &Vec3, n: &Vec3, scale: f32) {
        const THRESHOLD: f32 = 0.7;
        let mut e1: Vec3 = if f32::abs(n.x) < THRESHOLD {
            Vec3::X
        } else {
            Vec3::Y
        };
        e1 = reject_vec3(e1, *n).normalize();
        let e2: Vec3 = Vec3::cross(*n, e1);

        let p0: Vec3 = *p + (e1 * -scale) + (e2 * -scale);
        let p1: Vec3 = *p + (e1 * scale) + (e2 * -scale);
        let p2: Vec3 = *p + (e1 * scale) + (e2 * scale);
        let p3: Vec3 = *p + (e1 * -scale) + (e2 * scale);

        Metric::add_draw_imm(1, 2, 4);

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Quads);
        this.pb.vertex3(p0.x, p0.y, p0.z);
        this.pb.vertex3(p1.x, p1.y, p1.z);
        this.pb.vertex3(p2.x, p2.y, p2.z);
        this.pb.vertex3(p3.x, p3.y, p3.z);
        this.pb.end();
    }

    pub fn point(x: f32, y: f32) {
        let mut this = Self::inst();
        this.pb.begin(PrimitiveType::Points);
        this.pb.vertex2(x, y);
        this.pb.end();
    }

    pub fn point3(x: f32, y: f32, z: f32) {
        let mut this = Self::inst();
        this.pb.begin(PrimitiveType::Points);
        this.pb.vertex3(x, y, z);
        this.pb.end();
    }

    pub fn quad(p1: &Vec2, p2: &Vec2, p3: &Vec2, p4: &Vec2) {
        Metric::add_draw_imm(1, 2, 4);

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Quads);
        this.pb.texcoord2(0.0, 0.0);
        this.pb.vertex2(p1.x, p1.y);
        this.pb.texcoord2(0.0, 1.0);
        this.pb.vertex2(p2.x, p2.y);
        this.pb.texcoord2(1.0, 1.0);
        this.pb.vertex2(p3.x, p3.y);
        this.pb.texcoord2(1.0, 0.0);
        this.pb.vertex2(p4.x, p4.y);
        this.pb.end();
    }

    pub fn quad3(p1: &Vec3, p2: &Vec3, p3: &Vec3, p4: &Vec3) {
        Metric::add_draw_imm(1, 2, 4);

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Quads);
        this.pb.texcoord2(0.0, 0.0);
        this.pb.vertex3(p1.x, p1.y, p1.z);
        this.pb.texcoord2(0.0, 1.0);
        this.pb.vertex3(p2.x, p2.y, p2.z);
        this.pb.texcoord2(1.0, 1.0);
        this.pb.vertex3(p3.x, p3.y, p3.z);
        this.pb.texcoord2(1.0, 0.0);
        this.pb.vertex3(p4.x, p4.y, p4.z);
        this.pb.end();
    }

    pub fn rect(x1: f32, y1: f32, xs: f32, ys: f32) {
        Self::rect_ex(x1, y1, xs, ys, 0.0, 0.0, 1.0, 1.0);
    }

    pub fn rect_ex(x1: f32, y1: f32, xs: f32, ys: f32, u1: f32, v1: f32, u2: f32, v2: f32) {
        let x2: f32 = x1 + xs;
        let y2: f32 = y1 + ys;

        Metric::add_draw_imm(1, 2, 4);

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Quads);
        this.pb.texcoord2(u1, v1);
        this.pb.vertex2(x1, y1);
        this.pb.texcoord2(u1, v2);
        this.pb.vertex2(x1, y2);
        this.pb.texcoord2(u2, v2);
        this.pb.vertex2(x2, y2);
        this.pb.texcoord2(u2, v1);
        this.pb.vertex2(x2, y1);
        this.pb.end();
    }

    pub fn smooth_points(_enable: bool) {
        // TODO: Create rounded points.
    }

    /* TODO JP : Lazy creation of VBO / IBO & glDraw instead of immediate. */
    pub fn sphere(p: &Vec3, r: f32) {
        let res = 7;
        let f_res = res as f32;

        let mut this = Self::inst();

        // First Row
        Metric::add_draw_imm(res, res, res.wrapping_mul(3));

        let mut last_theta: f32 = res.wrapping_sub(1) as f32 / f_res * std::f32::consts::TAU;
        let phi: f32 = 1.0 / f_res * std::f32::consts::PI;
        let tc: Vec3 = *p + Self::spherical(r, 0.0, 0.0);

        this.pb.begin(PrimitiveType::Triangles);
        for i_theta in 0..res {
            let theta: f32 = i_theta as f32 / f_res * std::f32::consts::TAU;
            let br: Vec3 = *p + Self::spherical(r, last_theta, phi);
            let bl: Vec3 = *p + Self::spherical(r, theta, phi);

            this.pb.vertex3(br.x, br.y, br.z);
            this.pb.vertex3(tc.x, tc.y, tc.z);
            this.pb.vertex3(bl.x, bl.y, bl.z);

            last_theta = theta;
        }
        this.pb.end();

        // Middle Rows
        Metric::add_draw_imm(
            res.wrapping_sub(2),
            2u64.wrapping_mul(res.wrapping_sub(2)),
            4u64.wrapping_mul(res.wrapping_sub(2)),
        );

        let mut last_phi: f32 = 1.0 / f_res * std::f32::consts::PI;
        let mut last_theta: f32 = res.wrapping_sub(1) as f32 / f_res * std::f32::consts::TAU;

        this.pb.begin(PrimitiveType::Quads);
        for iPhi in 2..res {
            let phi: f32 = iPhi as f32 / f_res * std::f32::consts::PI;
            for i_theta in 0..res {
                let theta: f32 = i_theta as f32 / f_res * std::f32::consts::TAU;
                let br: Vec3 = *p + Self::spherical(r, last_theta, phi);
                let tr: Vec3 = *p + Self::spherical(r, last_theta, last_phi);
                let tl: Vec3 = *p + Self::spherical(r, theta, last_phi);
                let bl: Vec3 = *p + Self::spherical(r, theta, phi);

                this.pb.vertex3(br.x, br.y, br.z);
                this.pb.vertex3(tr.x, tr.y, tr.z);
                this.pb.vertex3(tl.x, tl.y, tl.z);
                this.pb.vertex3(bl.x, bl.y, bl.z);

                last_theta = theta;
            }
            last_phi = phi;
        }
        this.pb.end();

        // Bottom Row
        Metric::add_draw_imm(res, res, res.wrapping_mul(3));

        let mut last_theta: f32 = res.wrapping_sub(1) as f32 / f_res * std::f32::consts::TAU;
        let phi: f32 = res.wrapping_sub(1) as f32 / f_res * std::f32::consts::PI;
        let bc: Vec3 = *p + Self::spherical(r, 0.0, std::f32::consts::PI);

        this.pb.begin(PrimitiveType::Triangles);
        for i_theta in 0..res {
            let theta: f32 = i_theta as f32 / f_res * std::f32::consts::TAU;
            let tr: Vec3 = *p + Self::spherical(r, last_theta, phi);
            let tl: Vec3 = *p + Self::spherical(r, theta, phi);

            this.pb.vertex3(tr.x, tr.y, tr.z);
            this.pb.vertex3(tl.x, tl.y, tl.z);
            this.pb.vertex3(bc.x, bc.y, bc.z);

            last_theta = theta;
        }
        this.pb.end();
    }

    pub fn tri(v1: &Vec2, v2: &Vec2, v3: &Vec2) {
        Metric::add_draw_imm(1, 1, 3);

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Triangles);
        this.pb.texcoord2(0.0, 0.0);
        this.pb.vertex2(v1.x, v1.y);
        this.pb.texcoord2(0.0, 1.0);
        this.pb.vertex2(v2.x, v2.y);
        this.pb.texcoord2(1.0, 1.0);
        this.pb.vertex2(v3.x, v3.y);
        this.pb.end();
    }

    pub fn tri3(v1: &Vec3, v2: &Vec3, v3: &Vec3) {
        Metric::add_draw_imm(1, 1, 3);

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Triangles);
        this.pb.texcoord2(0.0, 0.0);
        this.pb.vertex3(v1.x, v1.y, v1.z);
        this.pb.texcoord2(0.0, 1.0);
        this.pb.vertex3(v2.x, v2.y, v2.z);
        this.pb.texcoord2(1.0, 1.0);
        this.pb.vertex3(v3.x, v3.y, v3.z);
        this.pb.end();
    }

    pub fn poly(points: &[Vec2]) {
        let count = points.len() as u64;

        Metric::add_draw_imm(1, count - 2, count);

        let mut this = Draw::inst();

        this.pb.begin(PrimitiveType::Polygon);
        for p in points {
            this.pb.vertex2(p.x, p.y);
        }
        this.pb.end();
    }

    pub fn poly3(points: &[Vec3]) {
        let count = points.len() as u64;

        Metric::add_draw_imm(1, count - 2, count);

        let mut this = Draw::inst();

        this.pb.begin(PrimitiveType::Polygon);
        for p in points {
            this.pb.vertex3(p.x, p.y, p.z);
        }
        this.pb.end();
    }
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
