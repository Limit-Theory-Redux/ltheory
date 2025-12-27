use std::sync::{Mutex, MutexGuard, OnceLock};

use glam::{Vec2, Vec3};
use tracing::warn;

use super::{Color, PrimitiveBuilder, PrimitiveType, RenderCommand, gl, is_command_mode, submit_command};
use crate::math::{Box3, reject_vec3};
use crate::render::glcheck;
use crate::system::Metric;

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
        if is_command_mode() {
            // In command mode, emit Clear command
            // Note: Framebuffer status check will be handled on render thread
            submit_command(RenderCommand::Clear {
                color: Some([r, g, b, a]),
                depth: None,
            });
        } else {
            // Direct GL mode
            let status = glcheck!(gl::CheckFramebufferStatus(gl::FRAMEBUFFER));
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
    }

    pub fn clear_depth(d: f32) {
        if is_command_mode() {
            submit_command(RenderCommand::Clear {
                color: None,
                depth: Some(d),
            });
        } else {
            glcheck!(gl::ClearDepth(d as f64));
            glcheck!(gl::Clear(gl::DEPTH_BUFFER_BIT));
        }
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
        if is_command_mode() {
            submit_command(RenderCommand::Flush);
        } else {
            glcheck!(gl::Finish());
        }
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
        if is_command_mode() {
            submit_command(RenderCommand::SetLineWidth(width));
        } else {
            glcheck!(gl::LineWidth(width));
        }
    }

    pub fn point_size(size: f32) {
        if is_command_mode() {
            submit_command(RenderCommand::SetPointSize(size));
        } else {
            glcheck!(gl::PointSize(size));
        }
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
        for i_phi in 2..res {
            let phi: f32 = i_phi as f32 / f_res * std::f32::consts::PI;
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

    /// Draw a 2D circle outline at (x, y) with radius r using `segments` line segments
    pub fn circle(x: f32, y: f32, r: f32, segments: i32) {
        let segments = segments.max(3) as usize;
        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Lines);
        for i in 0..segments {
            let a1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let a2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
            let x1 = x + r * a1.cos();
            let y1 = y + r * a1.sin();
            let x2 = x + r * a2.cos();
            let y2 = y + r * a2.sin();
            this.pb.vertex2(x1, y1);
            this.pb.vertex2(x2, y2);
        }
        this.pb.end();
    }

    /// Draw a filled 2D circle at (x, y) with radius r
    pub fn circle_filled(x: f32, y: f32, r: f32, segments: i32) {
        let segments = segments.max(3) as usize;
        Metric::add_draw_imm(1, segments as u64, segments as u64 + 1);

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Polygon);
        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            this.pb.vertex2(x + r * angle.cos(), y + r * angle.sin());
        }
        this.pb.end();
    }

    /// Draw a 3D circle outline centered at `center` with normal `normal` and radius `r`
    pub fn circle3(center: &Vec3, normal: &Vec3, r: f32, segments: i32) {
        let segments = segments.max(3) as usize;

        // Build orthonormal basis from normal
        let n = normal.normalize();
        let up = if n.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
        let tangent = n.cross(up).normalize();
        let bitangent = tangent.cross(n);

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Lines);
        for i in 0..segments {
            let a1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let a2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let p1 = *center + (tangent * a1.cos() + bitangent * a1.sin()) * r;
            let p2 = *center + (tangent * a2.cos() + bitangent * a2.sin()) * r;

            this.pb.vertex3(p1.x, p1.y, p1.z);
            this.pb.vertex3(p2.x, p2.y, p2.z);
        }
        this.pb.end();
    }

    /// Draw an arc from angle `start` to `end` (in radians) at (x, y) with radius r
    pub fn arc(x: f32, y: f32, r: f32, start: f32, end: f32, segments: i32) {
        let segments = segments.max(1) as usize;
        let angle_span = end - start;

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Lines);
        for i in 0..segments {
            let a1 = start + (i as f32 / segments as f32) * angle_span;
            let a2 = start + ((i + 1) as f32 / segments as f32) * angle_span;
            let x1 = x + r * a1.cos();
            let y1 = y + r * a1.sin();
            let x2 = x + r * a2.cos();
            let y2 = y + r * a2.sin();
            this.pb.vertex2(x1, y1);
            this.pb.vertex2(x2, y2);
        }
        this.pb.end();
    }

    /// Draw connected line segments through 2D points
    pub fn line_strip(points: &[Vec2]) {
        if points.len() < 2 {
            return;
        }

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Lines);
        for i in 0..points.len() - 1 {
            this.pb.vertex2(points[i].x, points[i].y);
            this.pb.vertex2(points[i + 1].x, points[i + 1].y);
        }
        this.pb.end();
    }

    /// Draw connected line segments through 3D points
    pub fn line_strip3(points: &[Vec3]) {
        if points.len() < 2 {
            return;
        }

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Lines);
        for i in 0..points.len() - 1 {
            this.pb.vertex3(points[i].x, points[i].y, points[i].z);
            this.pb.vertex3(points[i + 1].x, points[i + 1].y, points[i + 1].z);
        }
        this.pb.end();
    }

    /// Draw a 2D arrow from (x1, y1) to (x2, y2) with arrowhead of given size
    pub fn arrow(x1: f32, y1: f32, x2: f32, y2: f32, head_size: f32) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.0001 {
            return;
        }

        // Normalized direction
        let nx = dx / len;
        let ny = dy / len;

        // Perpendicular
        let px = -ny;
        let py = nx;

        // Arrowhead points
        let hx1 = x2 - nx * head_size + px * head_size * 0.5;
        let hy1 = y2 - ny * head_size + py * head_size * 0.5;
        let hx2 = x2 - nx * head_size - px * head_size * 0.5;
        let hy2 = y2 - ny * head_size - py * head_size * 0.5;

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Lines);
        // Main line
        this.pb.vertex2(x1, y1);
        this.pb.vertex2(x2, y2);
        // Arrowhead
        this.pb.vertex2(x2, y2);
        this.pb.vertex2(hx1, hy1);
        this.pb.vertex2(x2, y2);
        this.pb.vertex2(hx2, hy2);
        this.pb.end();
    }

    /// Draw a 3D arrow from p1 to p2 with arrowhead of given size
    pub fn arrow3(p1: &Vec3, p2: &Vec3, head_size: f32) {
        let dir = *p2 - *p1;
        let len = dir.length();
        if len < 0.0001 {
            return;
        }

        let dir_n = dir / len;

        // Find perpendicular vectors for arrowhead
        let up = if dir_n.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
        let perp1 = dir_n.cross(up).normalize();
        let perp2 = dir_n.cross(perp1);

        // Arrowhead base point
        let base = *p2 - dir_n * head_size;
        let h1 = base + perp1 * head_size * 0.5;
        let h2 = base - perp1 * head_size * 0.5;
        let h3 = base + perp2 * head_size * 0.5;
        let h4 = base - perp2 * head_size * 0.5;

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Lines);
        // Main line
        this.pb.vertex3(p1.x, p1.y, p1.z);
        this.pb.vertex3(p2.x, p2.y, p2.z);
        // Arrowhead (4 lines for 3D visibility)
        this.pb.vertex3(p2.x, p2.y, p2.z);
        this.pb.vertex3(h1.x, h1.y, h1.z);
        this.pb.vertex3(p2.x, p2.y, p2.z);
        this.pb.vertex3(h2.x, h2.y, h2.z);
        this.pb.vertex3(p2.x, p2.y, p2.z);
        this.pb.vertex3(h3.x, h3.y, h3.z);
        this.pb.vertex3(p2.x, p2.y, p2.z);
        this.pb.vertex3(h4.x, h4.y, h4.z);
        this.pb.end();
    }

    /// Draw a 2D grid centered at (x, y) with given size and cell count
    pub fn grid(x: f32, y: f32, width: f32, height: f32, cells_x: i32, cells_y: i32) {
        let cells_x = cells_x.max(1);
        let cells_y = cells_y.max(1);
        let cell_w = width / cells_x as f32;
        let cell_h = height / cells_y as f32;

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Lines);
        // Vertical lines
        for i in 0..=cells_x {
            let lx = x + i as f32 * cell_w;
            this.pb.vertex2(lx, y);
            this.pb.vertex2(lx, y + height);
        }
        // Horizontal lines
        for i in 0..=cells_y {
            let ly = y + i as f32 * cell_h;
            this.pb.vertex2(x, ly);
            this.pb.vertex2(x + width, ly);
        }
        this.pb.end();
    }

    /// Draw a 3D grid on the XZ plane centered at origin with given size and cell count
    pub fn grid3(center: &Vec3, size: f32, cells: i32) {
        let cells = cells.max(1);
        let half = size * 0.5;
        let cell_size = size / cells as f32;

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Lines);
        // Lines along X axis
        for i in 0..=cells {
            let z = center.z - half + i as f32 * cell_size;
            this.pb.vertex3(center.x - half, center.y, z);
            this.pb.vertex3(center.x + half, center.y, z);
        }
        // Lines along Z axis
        for i in 0..=cells {
            let x = center.x - half + i as f32 * cell_size;
            this.pb.vertex3(x, center.y, center.z - half);
            this.pb.vertex3(x, center.y, center.z + half);
        }
        this.pb.end();
    }

    /// Draw a 3D cylinder wireframe from `base` along `axis` with given radius and height
    pub fn cylinder(base: &Vec3, axis: &Vec3, radius: f32, height: f32, segments: i32) {
        let segments = segments.max(3) as usize;
        let axis_n = axis.normalize();

        // Build orthonormal basis
        let up = if axis_n.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
        let tangent = axis_n.cross(up).normalize();
        let bitangent = tangent.cross(axis_n);

        let top = *base + axis_n * height;

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Lines);
        for i in 0..segments {
            let a1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let a2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let offset1 = (tangent * a1.cos() + bitangent * a1.sin()) * radius;
            let offset2 = (tangent * a2.cos() + bitangent * a2.sin()) * radius;

            let b1 = *base + offset1;
            let b2 = *base + offset2;
            let t1 = top + offset1;
            let t2 = top + offset2;

            // Bottom circle
            this.pb.vertex3(b1.x, b1.y, b1.z);
            this.pb.vertex3(b2.x, b2.y, b2.z);
            // Top circle
            this.pb.vertex3(t1.x, t1.y, t1.z);
            this.pb.vertex3(t2.x, t2.y, t2.z);
            // Vertical lines (every other segment for clarity)
            if i % 2 == 0 {
                this.pb.vertex3(b1.x, b1.y, b1.z);
                this.pb.vertex3(t1.x, t1.y, t1.z);
            }
        }
        this.pb.end();
    }

    /// Draw a 3D cone wireframe from `base` along `axis` with given radius and height
    pub fn cone(base: &Vec3, axis: &Vec3, radius: f32, height: f32, segments: i32) {
        let segments = segments.max(3) as usize;
        let axis_n = axis.normalize();

        // Build orthonormal basis
        let up = if axis_n.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
        let tangent = axis_n.cross(up).normalize();
        let bitangent = tangent.cross(axis_n);

        let tip = *base + axis_n * height;

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Lines);
        for i in 0..segments {
            let a1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let a2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let offset1 = (tangent * a1.cos() + bitangent * a1.sin()) * radius;
            let offset2 = (tangent * a2.cos() + bitangent * a2.sin()) * radius;

            let b1 = *base + offset1;
            let b2 = *base + offset2;

            // Base circle
            this.pb.vertex3(b1.x, b1.y, b1.z);
            this.pb.vertex3(b2.x, b2.y, b2.z);
            // Lines to tip (every other segment)
            if i % 2 == 0 {
                this.pb.vertex3(b1.x, b1.y, b1.z);
                this.pb.vertex3(tip.x, tip.y, tip.z);
            }
        }
        this.pb.end();
    }

    /// Draw a 3D capsule wireframe (cylinder with hemispherical caps)
    pub fn capsule(p1: &Vec3, p2: &Vec3, radius: f32, segments: i32) {
        let segments = segments.max(4) as usize;
        let axis = *p2 - *p1;
        let height = axis.length();
        if height < 0.0001 {
            // Degenerate case - just draw a sphere
            Draw::sphere(p1, radius);
            return;
        }

        let axis_n = axis / height;

        // Build orthonormal basis
        let up = if axis_n.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
        let tangent = axis_n.cross(up).normalize();
        let bitangent = tangent.cross(axis_n);

        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Lines);

        // Cylinder body
        for i in 0..segments {
            let a1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let a2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let offset1 = (tangent * a1.cos() + bitangent * a1.sin()) * radius;
            let offset2 = (tangent * a2.cos() + bitangent * a2.sin()) * radius;

            let b1 = *p1 + offset1;
            let b2 = *p1 + offset2;
            let t1 = *p2 + offset1;
            let t2 = *p2 + offset2;

            // Bottom ring
            this.pb.vertex3(b1.x, b1.y, b1.z);
            this.pb.vertex3(b2.x, b2.y, b2.z);
            // Top ring
            this.pb.vertex3(t1.x, t1.y, t1.z);
            this.pb.vertex3(t2.x, t2.y, t2.z);
            // Vertical lines
            if i % 2 == 0 {
                this.pb.vertex3(b1.x, b1.y, b1.z);
                this.pb.vertex3(t1.x, t1.y, t1.z);
            }
        }

        // Hemisphere caps (simplified as arcs in two perpendicular planes)
        let half_segs = segments / 2;
        for i in 0..half_segs {
            let a1 = (i as f32 / half_segs as f32) * std::f32::consts::PI;
            let a2 = ((i + 1) as f32 / half_segs as f32) * std::f32::consts::PI;

            // Bottom cap (facing -axis)
            let bc1 = *p1 - axis_n * (radius * a1.cos()) + tangent * (radius * a1.sin());
            let bc2 = *p1 - axis_n * (radius * a2.cos()) + tangent * (radius * a2.sin());
            this.pb.vertex3(bc1.x, bc1.y, bc1.z);
            this.pb.vertex3(bc2.x, bc2.y, bc2.z);

            let bc3 = *p1 - axis_n * (radius * a1.cos()) + bitangent * (radius * a1.sin());
            let bc4 = *p1 - axis_n * (radius * a2.cos()) + bitangent * (radius * a2.sin());
            this.pb.vertex3(bc3.x, bc3.y, bc3.z);
            this.pb.vertex3(bc4.x, bc4.y, bc4.z);

            // Top cap (facing +axis)
            let tc1 = *p2 + axis_n * (radius * a1.cos()) + tangent * (radius * a1.sin());
            let tc2 = *p2 + axis_n * (radius * a2.cos()) + tangent * (radius * a2.sin());
            this.pb.vertex3(tc1.x, tc1.y, tc1.z);
            this.pb.vertex3(tc2.x, tc2.y, tc2.z);

            let tc3 = *p2 + axis_n * (radius * a1.cos()) + bitangent * (radius * a1.sin());
            let tc4 = *p2 + axis_n * (radius * a2.cos()) + bitangent * (radius * a2.sin());
            this.pb.vertex3(tc3.x, tc3.y, tc3.z);
            this.pb.vertex3(tc4.x, tc4.y, tc4.z);
        }

        this.pb.end();
    }

    /// Draw a wireframe AABB (axis-aligned bounding box)
    pub fn wire_box3(b: &Box3) {
        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Lines);
        // Bottom face
        this.pb.vertex3(b.lower.x, b.lower.y, b.lower.z);
        this.pb.vertex3(b.upper.x, b.lower.y, b.lower.z);
        this.pb.vertex3(b.upper.x, b.lower.y, b.lower.z);
        this.pb.vertex3(b.upper.x, b.lower.y, b.upper.z);
        this.pb.vertex3(b.upper.x, b.lower.y, b.upper.z);
        this.pb.vertex3(b.lower.x, b.lower.y, b.upper.z);
        this.pb.vertex3(b.lower.x, b.lower.y, b.upper.z);
        this.pb.vertex3(b.lower.x, b.lower.y, b.lower.z);
        // Top face
        this.pb.vertex3(b.lower.x, b.upper.y, b.lower.z);
        this.pb.vertex3(b.upper.x, b.upper.y, b.lower.z);
        this.pb.vertex3(b.upper.x, b.upper.y, b.lower.z);
        this.pb.vertex3(b.upper.x, b.upper.y, b.upper.z);
        this.pb.vertex3(b.upper.x, b.upper.y, b.upper.z);
        this.pb.vertex3(b.lower.x, b.upper.y, b.upper.z);
        this.pb.vertex3(b.lower.x, b.upper.y, b.upper.z);
        this.pb.vertex3(b.lower.x, b.upper.y, b.lower.z);
        // Vertical edges
        this.pb.vertex3(b.lower.x, b.lower.y, b.lower.z);
        this.pb.vertex3(b.lower.x, b.upper.y, b.lower.z);
        this.pb.vertex3(b.upper.x, b.lower.y, b.lower.z);
        this.pb.vertex3(b.upper.x, b.upper.y, b.lower.z);
        this.pb.vertex3(b.upper.x, b.lower.y, b.upper.z);
        this.pb.vertex3(b.upper.x, b.upper.y, b.upper.z);
        this.pb.vertex3(b.lower.x, b.lower.y, b.upper.z);
        this.pb.vertex3(b.lower.x, b.upper.y, b.upper.z);
        this.pb.end();
    }

    /// Draw a crosshair at the given 3D position
    pub fn crosshair3(pos: &Vec3, size: f32) {
        let half = size * 0.5;
        let mut this = Self::inst();

        this.pb.begin(PrimitiveType::Lines);
        this.pb.vertex3(pos.x - half, pos.y, pos.z);
        this.pb.vertex3(pos.x + half, pos.y, pos.z);
        this.pb.vertex3(pos.x, pos.y - half, pos.z);
        this.pb.vertex3(pos.x, pos.y + half, pos.z);
        this.pb.vertex3(pos.x, pos.y, pos.z - half);
        this.pb.vertex3(pos.x, pos.y, pos.z + half);
        this.pb.end();
    }
}

fn framebuffer_status_to_str(status: gl::types::GLenum) -> &'static str {
    match status {
        gl::FRAMEBUFFER_COMPLETE => "framebuffer is complete",
        gl::FRAMEBUFFER_UNDEFINED => {
            "the specified framebuffer is the default read or draw framebuffer, but the default framebuffer does not exist"
        }
        gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => {
            "any of the framebuffer attachment points are framebuffer incomplete"
        }
        gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => {
            "the framebuffer does not have at least one image attached to it"
        }
        gl::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER => {
            "the value of GL_FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE is GL_NONE for any color attachment point(s) named by GL_DRAW_BUFFERi"
        }
        gl::FRAMEBUFFER_INCOMPLETE_READ_BUFFER => {
            "GL_READ_BUFFER is not GL_NONE and the value of GL_FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE is GL_NONE for the color attachment point named by GL_READ_BUFFER"
        }
        gl::FRAMEBUFFER_UNSUPPORTED => {
            "the combination of internal formats of the attached images violates an implementation-dependent set of restrictions"
        }
        gl::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => {
            "the value of GL_RENDERBUFFER_SAMPLES is not the same for all attached renderbuffers; if the value of GL_TEXTURE_SAMPLES is the not same for all attached textures; or, if the attached images are a mix of renderbuffers and textures, the value of GL_RENDERBUFFER_SAMPLES does not match the value of GL_TEXTURE_SAMPLES. Also returned if the value of GL_TEXTURE_FIXED_SAMPLE_LOCATIONS is not the same for all attached textures; or, if the attached images are a mix of renderbuffers and textures, the value of GL_TEXTURE_FIXED_SAMPLE_LOCATIONS is not GL_TRUE for all attached textures"
        }
        // gl::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS => "any framebuffer attachment is layered, and any populated attachment is not layered, or if all populated color attachments are not from textures of the same target",
        _ => "Unknown",
    }
}
