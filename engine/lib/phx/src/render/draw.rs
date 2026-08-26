use glam::{Vec2, Vec3};

use super::{Color, PrimitiveType};
use crate::math::{Box3, reject_vec3};
use crate::render::Renderer;
use crate::system::Metric;

/// CPU-side state for `Draw`'s color/alpha stack (was part of the `Draw`
/// singleton, alongside the `PrimitiveBuilder` that's now `Renderer.imm`).
pub struct DrawState {
    alpha_stack: Vec<f32>,
    color: Color,
}

impl DrawState {
    pub fn new() -> Self {
        Self {
            alpha_stack: vec![],
            color: Color::WHITE,
        }
    }
}

impl Default for DrawState {
    fn default() -> Self {
        Self::new()
    }
}

/// Submit whatever `r.imm` accumulated since the matching `begin()`, if
/// anything was drawn.
fn end_and_submit(r: &mut Renderer) {
    if let Some((primitive, vertices)) = r.data.imm.end() {
        r.draw_immediate(primitive, vertices);
    }
}

pub struct Draw;

fn spherical(r: f32, yaw: f32, pitch: f32) -> Vec3 {
    Vec3::new(
        (r as f64 * f64::sin(pitch as f64) * f64::cos(yaw as f64)) as f32,
        (r as f64 * f64::cos(pitch as f64)) as f32,
        (r as f64 * f64::sin(pitch as f64) * f64::sin(yaw as f64)) as f32,
    )
}

#[luajit_ffi_gen::luajit_ffi]
impl Draw {
    pub fn clear(r: &mut Renderer, red: f32, green: f32, blue: f32, alpha: f32) {
        r.clear_intern(Some([red, green, blue, alpha]), None);
    }

    pub fn clear_depth(r: &mut Renderer, d: f32) {
        r.clear_intern(None, Some(d));
    }

    pub fn color(r: &mut Renderer, red: f32, green: f32, blue: f32, alpha: f32) {
        let last_alpha = r.data.draw_state.alpha_stack.last().copied().unwrap_or(1.0);

        r.data.draw_state.color = Color::new(red, green, blue, alpha);
        r.data.imm.color4(red, green, blue, alpha * last_alpha);
    }

    pub fn flush(r: &mut Renderer) {
        Metric::Flush.inc();
        r.gl_finish();
    }

    pub fn push_alpha(r: &mut Renderer, a: f32) {
        let prev_alpha = r.data.draw_state.alpha_stack.last().copied().unwrap_or(1.0);

        let alpha = a * prev_alpha;
        r.data.draw_state.alpha_stack.push(alpha);

        let mut color = r.data.draw_state.color;
        color.a *= alpha;
        r.data.imm.color4(color.r, color.g, color.b, color.a);
    }

    pub fn pop_alpha(r: &mut Renderer) {
        if r.data.draw_state.alpha_stack.is_empty() {
            panic!("attempting to pop an empty alpha stack");
        }

        r.data.draw_state.alpha_stack.pop();
        let alpha = r.data.draw_state.alpha_stack.last().copied().unwrap_or(1.0);

        let mut color = r.data.draw_state.color;
        color.a *= alpha;
        r.data.imm.color4(color.r, color.g, color.b, color.a);
    }

    pub fn line_width(r: &mut Renderer, width: f32) {
        r.set_line_width(width);
    }

    pub fn point_size(r: &mut Renderer, size: f32) {
        r.set_point_size(size);
    }

    pub fn axes(
        r: &mut Renderer,
        pos: &Vec3,
        x: &Vec3,
        y: &Vec3,
        z: &Vec3,
        scale: f32,
        alpha: f32,
    ) {
        let left: Vec3 = *pos + (*x) * scale;
        let up: Vec3 = *pos + (*y) * scale;
        let forward: Vec3 = *pos + (*z) * scale;

        r.data.imm.begin(PrimitiveType::Lines);
        r.data.imm.color4(1.0, 0.25, 0.25, alpha);
        r.data.imm.vertex3(pos.x, pos.y, pos.z);
        r.data.imm.vertex3(left.x, left.y, left.z);
        r.data.imm.color4(0.25, 1.0, 0.25, alpha);
        r.data.imm.vertex3(pos.x, pos.y, pos.z);
        r.data.imm.vertex3(up.x, up.y, up.z);
        r.data.imm.color4(0.25, 0.25, 1.0, alpha);
        r.data.imm.vertex3(pos.x, pos.y, pos.z);
        r.data.imm.vertex3(forward.x, forward.y, forward.z);
        end_and_submit(r);

        r.data.imm.begin(PrimitiveType::Points);
        r.data.imm.color4(1.0, 1.0, 1.0, alpha);
        r.data.imm.vertex3(pos.x, pos.y, pos.z);
        end_and_submit(r);
    }

    pub fn border(r: &mut Renderer, s: f32, x: f32, y: f32, w: f32, h: f32) {
        Draw::rect(r, x, y, w, s);
        Draw::rect(r, x, y + h - s, w, s);
        Draw::rect(r, x, y + s, s, h - 2.0 * s);
        Draw::rect(r, x + w - s, y + s, s, h - 2.0 * s);
    }

    pub fn box3(r: &mut Renderer, b: &Box3) {
        Metric::add_draw_imm(6, 12, 24);

        r.data.imm.begin(PrimitiveType::Quads);

        /* Left. */
        r.data.imm.vertex3(b.lower.x, b.lower.y, b.lower.z);
        r.data.imm.vertex3(b.lower.x, b.lower.y, b.upper.z);
        r.data.imm.vertex3(b.lower.x, b.upper.y, b.upper.z);
        r.data.imm.vertex3(b.lower.x, b.upper.y, b.lower.z);

        /* Right. */
        r.data.imm.vertex3(b.upper.x, b.lower.y, b.lower.z);
        r.data.imm.vertex3(b.upper.x, b.upper.y, b.lower.z);
        r.data.imm.vertex3(b.upper.x, b.upper.y, b.upper.z);
        r.data.imm.vertex3(b.upper.x, b.lower.y, b.upper.z);

        /* Front. */
        r.data.imm.vertex3(b.lower.x, b.lower.y, b.upper.z);
        r.data.imm.vertex3(b.upper.x, b.lower.y, b.upper.z);
        r.data.imm.vertex3(b.upper.x, b.upper.y, b.upper.z);
        r.data.imm.vertex3(b.lower.x, b.upper.y, b.upper.z);

        /* Back. */
        r.data.imm.vertex3(b.lower.x, b.lower.y, b.lower.z);
        r.data.imm.vertex3(b.lower.x, b.upper.y, b.lower.z);
        r.data.imm.vertex3(b.upper.x, b.upper.y, b.lower.z);
        r.data.imm.vertex3(b.upper.x, b.lower.y, b.lower.z);

        /* Top. */
        r.data.imm.vertex3(b.lower.x, b.upper.y, b.lower.z);
        r.data.imm.vertex3(b.lower.x, b.upper.y, b.upper.z);
        r.data.imm.vertex3(b.upper.x, b.upper.y, b.upper.z);
        r.data.imm.vertex3(b.upper.x, b.upper.y, b.lower.z);

        /* Bottom. */
        r.data.imm.vertex3(b.lower.x, b.lower.y, b.lower.z);
        r.data.imm.vertex3(b.upper.x, b.lower.y, b.lower.z);
        r.data.imm.vertex3(b.upper.x, b.lower.y, b.upper.z);
        r.data.imm.vertex3(b.lower.x, b.lower.y, b.upper.z);

        end_and_submit(r);
    }

    pub fn line(r: &mut Renderer, x1: f32, y1: f32, x2: f32, y2: f32) {
        r.data.imm.begin(PrimitiveType::Lines);
        r.data.imm.vertex2(x1, y1);
        r.data.imm.vertex2(x2, y2);
        end_and_submit(r);
    }

    pub fn line3(r: &mut Renderer, p1: &Vec3, p2: &Vec3) {
        r.data.imm.begin(PrimitiveType::Lines);
        r.data.imm.vertex3(p1.x, p1.y, p1.z);
        r.data.imm.vertex3(p2.x, p2.y, p2.z);
        end_and_submit(r);
    }

    pub fn plane(r: &mut Renderer, p: &Vec3, n: &Vec3, scale: f32) {
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

        r.data.imm.begin(PrimitiveType::Quads);
        r.data.imm.vertex3(p0.x, p0.y, p0.z);
        r.data.imm.vertex3(p1.x, p1.y, p1.z);
        r.data.imm.vertex3(p2.x, p2.y, p2.z);
        r.data.imm.vertex3(p3.x, p3.y, p3.z);
        end_and_submit(r);
    }

    pub fn point(r: &mut Renderer, x: f32, y: f32) {
        r.data.imm.begin(PrimitiveType::Points);
        r.data.imm.vertex2(x, y);
        end_and_submit(r);
    }

    pub fn point3(r: &mut Renderer, x: f32, y: f32, z: f32) {
        r.data.imm.begin(PrimitiveType::Points);
        r.data.imm.vertex3(x, y, z);
        end_and_submit(r);
    }

    pub fn quad(r: &mut Renderer, p1: &Vec2, p2: &Vec2, p3: &Vec2, p4: &Vec2) {
        Metric::add_draw_imm(1, 2, 4);

        r.data.imm.begin(PrimitiveType::Quads);
        r.data.imm.texcoord2(0.0, 0.0);
        r.data.imm.vertex2(p1.x, p1.y);
        r.data.imm.texcoord2(0.0, 1.0);
        r.data.imm.vertex2(p2.x, p2.y);
        r.data.imm.texcoord2(1.0, 1.0);
        r.data.imm.vertex2(p3.x, p3.y);
        r.data.imm.texcoord2(1.0, 0.0);
        r.data.imm.vertex2(p4.x, p4.y);
        end_and_submit(r);
    }

    pub fn quad3(r: &mut Renderer, p1: &Vec3, p2: &Vec3, p3: &Vec3, p4: &Vec3) {
        Metric::add_draw_imm(1, 2, 4);

        r.data.imm.begin(PrimitiveType::Quads);
        r.data.imm.texcoord2(0.0, 0.0);
        r.data.imm.vertex3(p1.x, p1.y, p1.z);
        r.data.imm.texcoord2(0.0, 1.0);
        r.data.imm.vertex3(p2.x, p2.y, p2.z);
        r.data.imm.texcoord2(1.0, 1.0);
        r.data.imm.vertex3(p3.x, p3.y, p3.z);
        r.data.imm.texcoord2(1.0, 0.0);
        r.data.imm.vertex3(p4.x, p4.y, p4.z);
        end_and_submit(r);
    }

    pub fn rect(r: &mut Renderer, x1: f32, y1: f32, xs: f32, ys: f32) {
        Self::rect_ex(r, x1, y1, xs, ys, 0.0, 0.0, 1.0, 1.0);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn rect_ex(
        r: &mut Renderer,
        x1: f32,
        y1: f32,
        xs: f32,
        ys: f32,
        u1: f32,
        v1: f32,
        u2: f32,
        v2: f32,
    ) {
        let x2: f32 = x1 + xs;
        let y2: f32 = y1 + ys;

        Metric::add_draw_imm(1, 2, 4);

        r.data.imm.begin(PrimitiveType::Quads);
        r.data.imm.texcoord2(u1, v1);
        r.data.imm.vertex2(x1, y1);
        r.data.imm.texcoord2(u1, v2);
        r.data.imm.vertex2(x1, y2);
        r.data.imm.texcoord2(u2, v2);
        r.data.imm.vertex2(x2, y2);
        r.data.imm.texcoord2(u2, v1);
        r.data.imm.vertex2(x2, y1);
        end_and_submit(r);
    }

    pub fn smooth_points(_enable: bool) {
        // TODO: Create rounded points.
    }

    /* TODO JP : Lazy creation of VBO / IBO & glDraw instead of immediate. */
    pub fn sphere(r: &mut Renderer, p: &Vec3, radius: f32) {
        let res = 7;
        let f_res = res as f32;

        // First Row
        Metric::add_draw_imm(res, res, res.wrapping_mul(3));

        let mut last_theta: f32 = res.wrapping_sub(1) as f32 / f_res * std::f32::consts::TAU;
        let phi: f32 = 1.0 / f_res * std::f32::consts::PI;
        let tc: Vec3 = *p + spherical(radius, 0.0, 0.0);

        r.data.imm.begin(PrimitiveType::Triangles);
        for i_theta in 0..res {
            let theta: f32 = i_theta as f32 / f_res * std::f32::consts::TAU;
            let br: Vec3 = *p + spherical(radius, last_theta, phi);
            let bl: Vec3 = *p + spherical(radius, theta, phi);

            r.data.imm.vertex3(br.x, br.y, br.z);
            r.data.imm.vertex3(tc.x, tc.y, tc.z);
            r.data.imm.vertex3(bl.x, bl.y, bl.z);

            last_theta = theta;
        }
        end_and_submit(r);

        // Middle Rows
        Metric::add_draw_imm(
            res.wrapping_sub(2),
            2u64.wrapping_mul(res.wrapping_sub(2)),
            4u64.wrapping_mul(res.wrapping_sub(2)),
        );

        let mut last_phi: f32 = 1.0 / f_res * std::f32::consts::PI;
        let mut last_theta: f32 = res.wrapping_sub(1) as f32 / f_res * std::f32::consts::TAU;

        r.data.imm.begin(PrimitiveType::Quads);
        for i_phi in 2..res {
            let phi: f32 = i_phi as f32 / f_res * std::f32::consts::PI;
            for i_theta in 0..res {
                let theta: f32 = i_theta as f32 / f_res * std::f32::consts::TAU;
                let br: Vec3 = *p + spherical(radius, last_theta, phi);
                let tr: Vec3 = *p + spherical(radius, last_theta, last_phi);
                let tl: Vec3 = *p + spherical(radius, theta, last_phi);
                let bl: Vec3 = *p + spherical(radius, theta, phi);

                r.data.imm.vertex3(br.x, br.y, br.z);
                r.data.imm.vertex3(tr.x, tr.y, tr.z);
                r.data.imm.vertex3(tl.x, tl.y, tl.z);
                r.data.imm.vertex3(bl.x, bl.y, bl.z);

                last_theta = theta;
            }
            last_phi = phi;
        }
        end_and_submit(r);

        // Bottom Row
        Metric::add_draw_imm(res, res, res.wrapping_mul(3));

        let mut last_theta: f32 = res.wrapping_sub(1) as f32 / f_res * std::f32::consts::TAU;
        let phi: f32 = res.wrapping_sub(1) as f32 / f_res * std::f32::consts::PI;
        let bc: Vec3 = *p + spherical(radius, 0.0, std::f32::consts::PI);

        r.data.imm.begin(PrimitiveType::Triangles);
        for i_theta in 0..res {
            let theta: f32 = i_theta as f32 / f_res * std::f32::consts::TAU;
            let tr: Vec3 = *p + spherical(radius, last_theta, phi);
            let tl: Vec3 = *p + spherical(radius, theta, phi);

            r.data.imm.vertex3(tr.x, tr.y, tr.z);
            r.data.imm.vertex3(tl.x, tl.y, tl.z);
            r.data.imm.vertex3(bc.x, bc.y, bc.z);

            last_theta = theta;
        }
        end_and_submit(r);
    }

    pub fn tri(r: &mut Renderer, v1: &Vec2, v2: &Vec2, v3: &Vec2) {
        Metric::add_draw_imm(1, 1, 3);

        r.data.imm.begin(PrimitiveType::Triangles);
        r.data.imm.texcoord2(0.0, 0.0);
        r.data.imm.vertex2(v1.x, v1.y);
        r.data.imm.texcoord2(0.0, 1.0);
        r.data.imm.vertex2(v2.x, v2.y);
        r.data.imm.texcoord2(1.0, 1.0);
        r.data.imm.vertex2(v3.x, v3.y);
        end_and_submit(r);
    }

    pub fn tri3(r: &mut Renderer, v1: &Vec3, v2: &Vec3, v3: &Vec3) {
        Metric::add_draw_imm(1, 1, 3);

        r.data.imm.begin(PrimitiveType::Triangles);
        r.data.imm.texcoord2(0.0, 0.0);
        r.data.imm.vertex3(v1.x, v1.y, v1.z);
        r.data.imm.texcoord2(0.0, 1.0);
        r.data.imm.vertex3(v2.x, v2.y, v2.z);
        r.data.imm.texcoord2(1.0, 1.0);
        r.data.imm.vertex3(v3.x, v3.y, v3.z);
        end_and_submit(r);
    }

    pub fn poly(r: &mut Renderer, points: &[Vec2]) {
        let count = points.len() as u64;

        Metric::add_draw_imm(1, count - 2, count);

        r.data.imm.begin(PrimitiveType::Polygon);
        for p in points {
            r.data.imm.vertex2(p.x, p.y);
        }
        end_and_submit(r);
    }

    pub fn poly3(r: &mut Renderer, points: &[Vec3]) {
        let count = points.len() as u64;

        Metric::add_draw_imm(1, count - 2, count);

        r.data.imm.begin(PrimitiveType::Polygon);
        for p in points {
            r.data.imm.vertex3(p.x, p.y, p.z);
        }
        end_and_submit(r);
    }
}
