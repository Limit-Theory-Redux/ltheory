//! Immediate batching (doc/engine/render-api-v2.md, section 3d).
//!
//! `Imm` is a batcher on the open pass. Primitives append to a run of CPU
//! vertices; a run ends when the `(layout, pipeline, texture, scissor)` key
//! changes or when anything else is recorded into the pass, and then becomes
//! one `PassCmd::DrawImm` (its vertices are copied into the vertex ring).
//! A quad is two triangles in the order of the old triangle fan, so geometry
//! rasterizes exactly as the per-primitive `DrawImmediate` did.
//!
//! Two vertex layouts exist. `Imm2D` (pixel coordinates, UI projection) carries
//! the shape parameters as flat attributes: each `ui/*` fragment shader reads
//! them from varyings instead of uniforms. `Imm3D` is a plain position, uv and
//! color vertex for debug geometry and for the backdrop box.

use std::collections::HashMap;

use glam::{Mat4, Vec3};

use super::{
    DepthState, PASS_FLUSH_LIMIT, PassCmd, PipelineDesc, PipelineId, PolygonMode, SamplerId,
    Samplers, TexView, Topology, VertexLayout,
};
use crate::math::Box3;
use crate::render::{BlendMode, Color, Renderer, ResourceId, ScissorUpdate, Shader, Tex2D};
use crate::system::Metric;

/// Vertices per `DrawImm` at most (a multiple of 6 and of 3, and the bytes fit
/// one vertex-ring chunk).
pub const IMM_MAX_RUN_VERTS: usize = 8190;

/// UI vertex: pixel position, uv, color and the two parameter vectors of the
/// shape (see `Shape::pack`).
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Imm2DVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub color: [f32; 4],
    pub p: [f32; 4],
    pub q: [f32; 4],
}

/// Debug and backdrop vertex.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Imm3DVertex {
    pub pos: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

/// Which vertex layout a `DrawImm` reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImmLayout {
    D2,
    D3,
}

impl ImmLayout {
    pub fn stride(self) -> usize {
        match self {
            ImmLayout::D2 => std::mem::size_of::<Imm2DVertex>(),
            ImmLayout::D3 => std::mem::size_of::<Imm3DVertex>(),
        }
    }
}

/// The 2D primitives, each with its own pipeline (fragment shader, blend mode
/// baked in). Parameters (`a..d` of `Imm.Shape`):
///
/// | shape | a | b | c | d |
/// |---|---|---|---|---|
/// | Circle, Hex, Ring, RingGlow, RingDim | radius | | | |
/// | Panel | inner alpha | bevel | | |
/// | Wedge | r1 | r2 | to | tw |
/// | Annulus | inner radius | outer radius | | |
/// | Box, Grid, PanelGlow, Point, PointGlow | none | | | |
///
/// `Triangle` and `LineGlow*` have their own entry points (`Imm.TriGlow`,
/// `Imm.LineGlow`).
#[luajit_ffi_gen::luajit_ffi(repr = "u32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Shape {
    Solid,
    Image,
    Text,
    TextAdditive,
    Box,
    Circle,
    Grid,
    Hex,
    Icon,
    Panel,
    PanelGlow,
    Point,
    PointGlow,
    Ring,
    RingGlow,
    RingDim,
    Triangle,
    Wedge,
    Annulus,
    LineGlow,
}

impl Shape {
    const COUNT: usize = 20;

    /// Fragment shader, blend mode and whether it samples a texture.
    fn info(self) -> (&'static str, BlendMode, bool) {
        use BlendMode::{Additive, Alpha};
        match self {
            Shape::Solid => ("fragment/ui/solidcolor", Alpha, false),
            Shape::Image => ("fragment/ui/image", Alpha, true),
            Shape::Text => ("fragment/ui/text", Alpha, true),
            Shape::TextAdditive => ("fragment/ui/text", Additive, true),
            Shape::Box => ("fragment/ui/box", Additive, false),
            Shape::Circle => ("fragment/ui/circle", Additive, false),
            Shape::Grid => ("fragment/ui/grid", Additive, false),
            Shape::Hex => ("fragment/ui/hex", Additive, false),
            Shape::Icon => ("fragment/ui/icon", Additive, true),
            Shape::Panel => ("fragment/ui/panel", Alpha, false),
            Shape::PanelGlow => ("fragment/ui/panelglow", Additive, false),
            Shape::Point => ("fragment/ui/point", Alpha, false),
            Shape::PointGlow => ("fragment/ui/point", Additive, false),
            Shape::Ring => ("fragment/ui/ring", Alpha, false),
            Shape::RingGlow => ("fragment/ui/ring", Additive, false),
            Shape::RingDim => ("fragment/ui/ringdim", Additive, false),
            Shape::Triangle => ("fragment/ui/triangle", Additive, false),
            Shape::Wedge => ("fragment/ui/wedge", Additive, false),
            Shape::Annulus => ("fragment/ui/annulus", Alpha, false),
            Shape::LineGlow => ("fragment/ui/line", Additive, false),
        }
    }

    /// The `p` and `q` vertex parameters for a quad of `w` x `h` pixels.
    fn pack(self, w: f32, h: f32, a: f32, b: f32, c: f32, d: f32) -> ([f32; 4], [f32; 4]) {
        match self {
            // radius, size
            Shape::Circle | Shape::Hex | Shape::RingDim => ([a, w, h, 0.0], [0.0; 4]),
            // radius, size, glow flag
            Shape::Ring => ([a, w, h, 0.0], [0.0; 4]),
            Shape::RingGlow => ([a, w, h, 0.0], [1.0, 0.0, 0.0, 0.0]),
            // innerAlpha, size, bevel
            Shape::Panel => ([a, w, h, b], [0.0; 4]),
            // r1, r2, to, tw; size
            Shape::Wedge => ([a, b, c, d], [w, h, 0.0, 0.0]),
            // innerRadius, outerRadius, size
            Shape::Annulus => ([a, b, w, h], [0.0; 4]),
            // size only
            Shape::Box | Shape::Grid | Shape::PanelGlow | Shape::Point | Shape::PointGlow => {
                ([w, h, 0.0, 0.0], [0.0; 4])
            }
            _ => ([0.0; 4], [0.0; 4]),
        }
    }
}

/// State of a debug (`Imm3D`) pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmDebugState {
    pub blend: BlendMode,
    pub depth_test: bool,
    pub wireframe: bool,
}

impl Default for ImmDebugState {
    fn default() -> Self {
        Self {
            blend: BlendMode::Alpha,
            depth_test: true,
            wireframe: false,
        }
    }
}

/// What a run of vertices shares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RunKey {
    layout: ImmLayout,
    pipeline: PipelineId,
    tex: Option<(TexView, SamplerId)>,
    scissor: ScissorUpdate,
}

/// Main-thread state of the batcher, owned by `RendererData`.
pub struct ImmBatcher {
    key: Option<RunKey>,
    v2: Vec<Imm2DVertex>,
    v3: Vec<Imm3DVertex>,
    /// Shape shaders (kept alive here) and the pipeline made from each, with
    /// the shader resource it was made for (a hot reload changes it).
    shaders: Vec<Option<Shader>>,
    pipes: Vec<Option<(ResourceId, PipelineId)>>,
    debug_shader: Option<Shader>,
    debug_pipes: HashMap<ImmDebugState, (ResourceId, PipelineId)>,
}

impl ImmBatcher {
    pub fn new() -> Self {
        Self {
            key: None,
            v2: Vec::with_capacity(1024),
            v3: Vec::new(),
            shaders: vec![None; Shape::COUNT],
            pipes: vec![None; Shape::COUNT],
            debug_shader: None,
            debug_pipes: HashMap::new(),
        }
    }

    pub fn has_pending(&self) -> bool {
        self.key.is_some()
    }
}

impl Default for ImmBatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Two triangles in the order of the quad's fan: (0, 1, 2) and (0, 2, 3).
fn triangulate<V: Copy>(q: [V; 4]) -> [V; 6] {
    [q[0], q[1], q[2], q[0], q[2], q[3]]
}

#[allow(clippy::too_many_arguments)]
fn quad2d(
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    uv: [f32; 4],
    color: [f32; 4],
    p: [f32; 4],
    q: [f32; 4],
) -> [Imm2DVertex; 6] {
    let v = |x: f32, y: f32, u: f32, w: f32| Imm2DVertex {
        pos: [x, y],
        uv: [u, w],
        color,
        p,
        q,
    };
    triangulate([
        v(x1, y1, uv[0], uv[1]),
        v(x1, y2, uv[0], uv[3]),
        v(x2, y2, uv[2], uv[3]),
        v(x2, y1, uv[2], uv[1]),
    ])
}

fn rgba(c: &Color) -> [f32; 4] {
    [c.r, c.g, c.b, c.a]
}

impl Renderer {
    fn imm_shape_pipeline(&mut self, shape: Shape) -> PipelineId {
        let i = shape as usize;
        if self.data.imm_batch.shaders[i].is_none() {
            let (fragment, _, _) = shape.info();
            let shader = Shader::load(self, "vertex/imm2d", fragment);
            self.data.imm_batch.shaders[i] = Some(shader);
        }
        let resource = self.data.imm_batch.shaders[i]
            .as_ref()
            .expect("shape shader")
            .resource();
        if let Some((res, id)) = self.data.imm_batch.pipes[i] {
            if res == resource {
                return id;
            }
        }
        let (_, blend, _) = shape.info();
        let mut desc = PipelineDesc::new(resource);
        desc.vertex = VertexLayout::Imm2D;
        desc.topology = Topology::Triangles;
        desc.blend = blend;
        let id = self.get_pipeline(&desc);
        self.data.imm_batch.pipes[i] = Some((resource, id));
        id
    }

    fn imm_debug_pipeline(&mut self, state: ImmDebugState) -> PipelineId {
        if self.data.imm_batch.debug_shader.is_none() {
            let shader = Shader::load(self, "vertex/imm3d", "fragment/imm3d");
            self.data.imm_batch.debug_shader = Some(shader);
        }
        let resource = self
            .data
            .imm_batch
            .debug_shader
            .as_ref()
            .expect("debug shader")
            .resource();
        if let Some(&(res, id)) = self.data.imm_batch.debug_pipes.get(&state) {
            if res == resource {
                return id;
            }
        }
        let mut desc = PipelineDesc::new(resource);
        desc.vertex = VertexLayout::Imm3D;
        desc.topology = Topology::Triangles;
        desc.blend = state.blend;
        desc.depth = DepthState {
            test: state.depth_test,
            write: false,
            compare: super::CompareFn::LessEqual,
        };
        if state.wireframe {
            desc.polygon = PolygonMode::Line;
        }
        let id = self.get_pipeline(&desc);
        self.data.imm_batch.debug_pipes.insert(state, (resource, id));
        id
    }

    /// The scissor the clip stack asks for now.
    fn imm_scissor(&mut self) -> ScissorUpdate {
        let size = self.target_size();
        self.data.clip_rect.desired(size)
    }

    /// Append `verts` to the current run, starting a new one if the key
    /// differs.
    fn imm_push2d(
        &mut self,
        pipeline: PipelineId,
        tex: Option<(TexView, SamplerId)>,
        verts: &[Imm2DVertex],
    ) {
        self.pass_require_open("imm");
        let key = RunKey {
            layout: ImmLayout::D2,
            pipeline,
            tex,
            scissor: self.imm_scissor(),
        };
        if self.data.imm_batch.key != Some(key) {
            self.imm_flush();
            self.data.imm_batch.key = Some(key);
        }
        self.data.imm_batch.v2.extend_from_slice(verts);
        if self.data.imm_batch.v2.len() >= IMM_MAX_RUN_VERTS {
            self.imm_flush();
        }
    }

    fn imm_push3d(&mut self, pipeline: PipelineId, verts: &[Imm3DVertex]) {
        self.pass_require_open("imm");
        let key = RunKey {
            layout: ImmLayout::D3,
            pipeline,
            tex: None,
            scissor: self.imm_scissor(),
        };
        if self.data.imm_batch.key != Some(key) {
            self.imm_flush();
            self.data.imm_batch.key = Some(key);
        }
        self.data.imm_batch.v3.extend_from_slice(verts);
        if self.data.imm_batch.v3.len() >= IMM_MAX_RUN_VERTS {
            self.imm_flush();
        }
    }

    /// End the current run: copy its vertices into the vertex ring and record
    /// the draw. Called before anything else is recorded into the pass, so
    /// the batch never reorders against other commands.
    pub(crate) fn imm_flush(&mut self) {
        let Some(key) = self.data.imm_batch.key.take() else {
            return;
        };
        let (bytes, count): (&[u8], usize) = match key.layout {
            ImmLayout::D2 => {
                let v = &self.data.imm_batch.v2;
                (bytemuck_bytes(v), v.len())
            }
            ImmLayout::D3 => {
                let v = &self.data.imm_batch.v3;
                (bytemuck_bytes(v), v.len())
            }
        };
        if count == 0 {
            return;
        }
        // The ring is a separate field from the batcher, so the vertices are
        // copied straight out of the run's own buffer.
        let at = self.data.vertex_ring.alloc_copy(bytes);
        match key.layout {
            ImmLayout::D2 => self.data.imm_batch.v2.clear(),
            ImmLayout::D3 => self.data.imm_batch.v3.clear(),
        }

        if self.data.pass.bound_pipeline != Some(key.pipeline) {
            self.data.pass.bound_pipeline = Some(key.pipeline);
            self.data.encoder.push(PassCmd::SetPipeline(key.pipeline));
        }
        if let Some(tex) = key.tex {
            let mut inputs = [None; super::MAX_INPUTS];
            inputs[0] = Some(tex);
            self.data.encoder.push(PassCmd::SetInputs(Box::new(inputs)));
            // The executor now holds the run's texture in slot 0: whatever
            // the caller staged goes out again with their next draw.
            self.data.encoder.inputs_dirty = true;
        }
        self.data.encoder.push(PassCmd::DrawImm {
            layout: key.layout,
            vertices: at,
            count: count as u32,
        });
        Metric::add_draw_imm(count as u64 / 6, count as u64 / 3, count as u64);
        if self.data.encoder.len() >= PASS_FLUSH_LIMIT {
            self.flush_pass_encoder();
        }
    }

    // ---- 2D ------------------------------------------------------------

    #[allow(clippy::too_many_arguments)]
    pub fn imm_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: &Color) {
        let pipeline = self.imm_shape_pipeline(Shape::Solid);
        let quad = quad2d(
            x,
            y,
            x + w,
            y + h,
            [0.0, 0.0, 1.0, 1.0],
            rgba(color),
            [0.0; 4],
            [0.0; 4],
        );
        self.imm_push2d(pipeline, None, &quad);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn imm_shape(
        &mut self,
        shape: Shape,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: &Color,
        params: [f32; 4],
    ) {
        let pipeline = self.imm_shape_pipeline(shape);
        let (p, q) = shape.pack(w, h, params[0], params[1], params[2], params[3]);
        let quad = quad2d(x, y, x + w, y + h, [0.0, 0.0, 1.0, 1.0], rgba(color), p, q);
        self.imm_push2d(pipeline, None, &quad);
    }

    /// A textured quad with `shape` (`Image`, `Icon`, `Text`).
    #[allow(clippy::too_many_arguments)]
    pub fn imm_textured(
        &mut self,
        shape: Shape,
        view: TexView,
        sampler: SamplerId,
        rect: [f32; 4],
        uv: [f32; 4],
        color: &Color,
    ) {
        let pipeline = self.imm_shape_pipeline(shape);
        let quad = quad2d(
            rect[0],
            rect[1],
            rect[0] + rect[2],
            rect[1] + rect[3],
            uv,
            rgba(color),
            [0.0; 4],
            [0.0; 4],
        );
        self.imm_push2d(pipeline, Some((view, sampler)), &quad);
    }

    /// Append already built vertices of a textured shape (glyph runs).
    pub fn imm_textured_vertices(
        &mut self,
        shape: Shape,
        view: TexView,
        sampler: SamplerId,
        verts: &[Imm2DVertex],
    ) {
        let pipeline = self.imm_shape_pipeline(shape);
        self.imm_push2d(pipeline, Some((view, sampler)), verts);
    }

    /// A solid triangle.
    #[allow(clippy::too_many_arguments)]
    pub fn imm_tri(&mut self, p: [[f32; 2]; 3], color: &Color) {
        let pipeline = self.imm_shape_pipeline(Shape::Solid);
        let v = |p: [f32; 2], u: f32, w: f32| Imm2DVertex {
            pos: p,
            uv: [u, w],
            color: rgba(color),
            p: [0.0; 4],
            q: [0.0; 4],
        };
        let tri = [v(p[0], 0.0, 0.0), v(p[1], 0.0, 1.0), v(p[2], 1.0, 1.0)];
        self.imm_push2d(pipeline, None, &tri);
    }

    /// The soft-edged triangle (`ui/triangle`): its vertices ride in the
    /// parameters, the quad is the padded bounding box.
    pub fn imm_tri_glow(&mut self, p: [[f32; 2]; 3], color: &Color, pad: f32) {
        let pipeline = self.imm_shape_pipeline(Shape::Triangle);
        let min_x = p.iter().map(|v| v[0]).fold(f32::MAX, f32::min) - pad;
        let min_y = p.iter().map(|v| v[1]).fold(f32::MAX, f32::min) - pad;
        let max_x = p.iter().map(|v| v[0]).fold(f32::MIN, f32::max) + pad;
        let max_y = p.iter().map(|v| v[1]).fold(f32::MIN, f32::max) + pad;
        let quad = quad2d(
            min_x,
            min_y,
            max_x,
            max_y,
            [0.0, 0.0, 1.0, 1.0],
            rgba(color),
            [p[0][0], p[0][1], p[1][0], p[1][1]],
            [p[2][0], p[2][1], 0.0, 0.0],
        );
        self.imm_push2d(pipeline, None, &quad);
    }

    /// A solid line of `width` pixels, as a quad.
    pub fn imm_line(&mut self, a: [f32; 2], b: [f32; 2], color: &Color, width: f32) {
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-9 {
            return;
        }
        let h = 0.5 * width.max(0.0);
        let (nx, ny) = (-dy / len * h, dx / len * h);
        let pipeline = self.imm_shape_pipeline(Shape::Solid);
        let c = rgba(color);
        let v = |x: f32, y: f32, u: f32, w: f32| Imm2DVertex {
            pos: [x, y],
            uv: [u, w],
            color: c,
            p: [0.0; 4],
            q: [0.0; 4],
        };
        let quad = triangulate([
            v(a[0] + nx, a[1] + ny, 0.0, 0.0),
            v(a[0] - nx, a[1] - ny, 0.0, 1.0),
            v(b[0] - nx, b[1] - ny, 1.0, 1.0),
            v(b[0] + nx, b[1] + ny, 1.0, 0.0),
        ]);
        self.imm_push2d(pipeline, None, &quad);
    }

    /// The soft-edged line of `ui/line` (`fade` dims it towards its end).
    pub fn imm_line_glow(&mut self, a: [f32; 2], b: [f32; 2], color: &Color, fade: bool, pad: f32) {
        let pipeline = self.imm_shape_pipeline(Shape::LineGlow);
        let quad = quad2d(
            a[0].min(b[0]) - pad,
            a[1].min(b[1]) - pad,
            a[0].max(b[0]) + pad,
            a[1].max(b[1]) + pad,
            [0.0, 0.0, 1.0, 1.0],
            rgba(color),
            [a[0], a[1], b[0], b[1]],
            [if fade { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0],
        );
        self.imm_push2d(pipeline, None, &quad);
    }

    /// A convex polygon as a triangle fan from its first point.
    pub fn imm_poly(&mut self, points: &[[f32; 2]], color: &Color) {
        if points.len() < 3 {
            return;
        }
        let pipeline = self.imm_shape_pipeline(Shape::Solid);
        let c = rgba(color);
        let v = |p: [f32; 2]| Imm2DVertex {
            pos: p,
            uv: [0.0; 2],
            color: c,
            p: [0.0; 4],
            q: [0.0; 4],
        };
        let mut verts = Vec::with_capacity((points.len() - 2) * 3);
        for i in 1..points.len() - 1 {
            verts.extend_from_slice(&[v(points[0]), v(points[i]), v(points[i + 1])]);
        }
        self.imm_push2d(pipeline, None, &verts);
    }

    // ---- 3D ------------------------------------------------------------

    /// The six faces of `b`, drawn with the pass's current pipeline (which
    /// must use `VertexLayout::Imm3D`), as quads in the order of the old
    /// `Draw.Box3`.
    pub fn imm_box3(&mut self, b: &Box3) {
        let pipeline = self
            .data
            .pass
            .user_pipeline
            .expect("Imm.Box3: no pipeline set (pass:setPipeline with a VertexLayout.Imm3D pipeline)");
        let (l, u) = (b.lower, b.upper);
        let v = |x: f32, y: f32, z: f32| Imm3DVertex {
            pos: [x, y, z],
            uv: [0.0; 2],
            color: [1.0; 4],
        };
        let faces = [
            // Left.
            [v(l.x, l.y, l.z), v(l.x, l.y, u.z), v(l.x, u.y, u.z), v(l.x, u.y, l.z)],
            // Right.
            [v(u.x, l.y, l.z), v(u.x, u.y, l.z), v(u.x, u.y, u.z), v(u.x, l.y, u.z)],
            // Front.
            [v(l.x, l.y, u.z), v(u.x, l.y, u.z), v(u.x, u.y, u.z), v(l.x, u.y, u.z)],
            // Back.
            [v(l.x, l.y, l.z), v(l.x, u.y, l.z), v(u.x, u.y, l.z), v(u.x, l.y, l.z)],
            // Top.
            [v(l.x, u.y, l.z), v(l.x, u.y, u.z), v(u.x, u.y, u.z), v(u.x, u.y, l.z)],
            // Bottom.
            [v(l.x, l.y, l.z), v(u.x, l.y, l.z), v(u.x, l.y, u.z), v(l.x, l.y, u.z)],
        ];
        let mut verts = Vec::with_capacity(36);
        for f in faces {
            verts.extend_from_slice(&triangulate(f));
        }
        self.imm_push3d(pipeline, &verts);
    }

    fn debug_color(c: &Color) -> [f32; 4] {
        rgba(c)
    }

    /// A debug triangle (camera-relative world positions).
    pub fn imm_debug_tri3(&mut self, state: ImmDebugState, p: [Vec3; 3], color: &Color) {
        let pipeline = self.imm_debug_pipeline(state);
        let c = Self::debug_color(color);
        let v = |p: Vec3| Imm3DVertex {
            pos: p.to_array(),
            uv: [0.0; 2],
            color: c,
        };
        self.imm_push3d(pipeline, &[v(p[0]), v(p[1]), v(p[2])]);
    }

    /// A debug quad (camera-relative world positions, in fan order).
    pub fn imm_debug_quad3(&mut self, state: ImmDebugState, p: [Vec3; 4], color: &Color) {
        let pipeline = self.imm_debug_pipeline(state);
        let c = Self::debug_color(color);
        let v = |p: Vec3| Imm3DVertex {
            pos: p.to_array(),
            uv: [0.0; 2],
            color: c,
        };
        self.imm_push3d(pipeline, &triangulate([v(p[0]), v(p[1]), v(p[2]), v(p[3])]));
    }

    fn pixel_scale(&self) -> (Mat4, f32) {
        let camera = &self.data.camera;
        let vp_h = self.data.pass.viewport[3].max(1) as f32;
        // View-space units per pixel at depth 1.
        let k = 1.0 / (0.5 * vp_h * camera.proj.y_axis.y.abs().max(1e-6));
        (camera.view, k)
    }

    /// A debug line of `width` pixels: a camera-facing quad (wide lines are
    /// not a thing on core GL or wgpu).
    pub fn imm_debug_line3(
        &mut self,
        state: ImmDebugState,
        a: Vec3,
        b: Vec3,
        color: &Color,
        width: f32,
    ) {
        let (view, k) = self.pixel_scale();
        let (va, vb) = (view.transform_point3(a), view.transform_point3(b));
        let mid = (va + vb) * 0.5;
        let mut side = (vb - va).cross(mid).normalize_or_zero();
        if side == Vec3::ZERO {
            side = Vec3::X;
        }
        let half = 0.5 * width.max(0.0) * k;
        let oa = side * half * (-va.z).max(1e-3);
        let ob = side * half * (-vb.z).max(1e-3);
        let inv = view.inverse();
        let w = |p: Vec3| inv.transform_point3(p);
        self.imm_debug_quad3(state, [w(va + oa), w(va - oa), w(vb - ob), w(vb + ob)], color);
    }

    /// A debug point of `size` pixels: a camera-facing square.
    pub fn imm_debug_point3(&mut self, state: ImmDebugState, p: Vec3, color: &Color, size: f32) {
        let (view, k) = self.pixel_scale();
        let vp = view.transform_point3(p);
        let h = 0.5 * size.max(0.0) * k * (-vp.z).max(1e-3);
        let inv = view.inverse();
        let w = |d: Vec3| inv.transform_point3(vp + d);
        self.imm_debug_quad3(
            state,
            [
                w(Vec3::new(-h, -h, 0.0)),
                w(Vec3::new(h, -h, 0.0)),
                w(Vec3::new(h, h, 0.0)),
                w(Vec3::new(-h, h, 0.0)),
            ],
            color,
        );
    }
}

/// `v` as bytes (the vertex types are `repr(C)` floats without padding).
fn bytemuck_bytes<T: Copy>(v: &[T]) -> &[u8] {
    #[allow(unsafe_code)]
    // SAFETY: `Imm2DVertex`/`Imm3DVertex` are `repr(C)` structs of `f32`.
    unsafe {
        std::slice::from_raw_parts(v.as_ptr() as *const u8, std::mem::size_of_val(v))
    }
}

/// Lua namespace of the immediate batcher (`Imm.Rect`, ...). Everything
/// draws into the open pass; ClipRect sets the scissor and
/// `pass:setUiTransform` the transform.
pub struct Imm;

#[luajit_ffi_gen::luajit_ffi]
impl Imm {
    /// A solid rectangle in pixel coordinates (y down).
    pub fn rect(r: &mut Renderer, x: f32, y: f32, w: f32, h: f32, color: &Color) {
        r.imm_rect(x, y, w, h, color);
    }

    /// A solid rectangle outline of thickness `s` inside the rectangle.
    pub fn border(r: &mut Renderer, s: f32, x: f32, y: f32, w: f32, h: f32, color: &Color) {
        r.imm_rect(x, y, w, s, color);
        r.imm_rect(x, y + h - s, w, s, color);
        r.imm_rect(x, y + s, s, h - 2.0 * s, color);
        r.imm_rect(x + w - s, y + s, s, h - 2.0 * s, color);
    }

    /// A textured rectangle, multiplied by `color`. `sampler` is a
    /// `Samplers.*` value.
    #[allow(clippy::too_many_arguments)]
    pub fn image(
        r: &mut Renderer,
        tex: &Tex2D,
        sampler: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        u0: f32,
        v0: f32,
        u1: f32,
        v1: f32,
        color: &Color,
    ) {
        r.imm_textured(
            Shape::Image,
            tex.view(),
            SamplerId(sampler as u16),
            [x, y, w, h],
            [u0, v0, u1, v1],
            color,
        );
    }

    /// An icon: the texture's alpha, tinted by `color`, added to the target.
    #[allow(clippy::too_many_arguments)]
    pub fn icon(
        r: &mut Renderer,
        tex: &Tex2D,
        sampler: u32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: &Color,
    ) {
        r.imm_textured(
            Shape::Icon,
            tex.view(),
            SamplerId(sampler as u16),
            [x, y, w, h],
            [0.0, 0.0, 1.0, 1.0],
            color,
        );
    }

    /// One of the `ui/*` shapes in the rectangle `x, y, w, h` (which includes
    /// the shape's own padding); `a..d` are the shape's parameters (see
    /// `Shape`).
    #[allow(clippy::too_many_arguments)]
    pub fn shape(
        r: &mut Renderer,
        shape: Shape,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: &Color,
        a: f32,
        b: f32,
        c: f32,
        d: f32,
    ) {
        r.imm_shape(shape, x, y, w, h, color, [a, b, c, d]);
    }

    /// A solid triangle.
    #[allow(clippy::too_many_arguments)]
    pub fn tri(
        r: &mut Renderer,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        color: &Color,
    ) {
        r.imm_tri([[x1, y1], [x2, y2], [x3, y3]], color);
    }

    /// The soft-edged triangle (`ui/triangle`), additive.
    #[allow(clippy::too_many_arguments)]
    pub fn tri_glow(
        r: &mut Renderer,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        color: &Color,
        pad: f32,
    ) {
        r.imm_tri_glow([[x1, y1], [x2, y2], [x3, y3]], color, pad);
    }

    /// A solid line of `width` pixels.
    pub fn line(r: &mut Renderer, x1: f32, y1: f32, x2: f32, y2: f32, color: &Color, width: f32) {
        r.imm_line([x1, y1], [x2, y2], color, width);
    }

    /// The soft-edged line of `ui/line`, additive.
    #[allow(clippy::too_many_arguments)]
    pub fn line_glow(
        r: &mut Renderer,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: &Color,
        fade: bool,
        pad: f32,
    ) {
        r.imm_line_glow([x1, y1], [x2, y2], color, fade, pad);
    }

    /// A solid square point of `size` pixels centered on `x, y`.
    pub fn point(r: &mut Renderer, x: f32, y: f32, size: f32, color: &Color) {
        let h = 0.5 * size;
        r.imm_rect(x - h, y - h, size, size, color);
    }

    /// The faces of `b` with the pass's current pipeline (it must use
    /// `VertexLayout.Imm3D`).
    pub fn box3(r: &mut Renderer, b: &Box3) {
        r.imm_box3(b);
    }

    /// A debug line (camera-relative positions, depth tested, alpha blended).
    pub fn line3(r: &mut Renderer, p1: &Vec3, p2: &Vec3, color: &Color, width: f32) {
        r.imm_debug_line3(ImmDebugState::default(), *p1, *p2, color, width);
    }

    /// A debug point of `size` pixels.
    pub fn point3(r: &mut Renderer, p: &Vec3, color: &Color, size: f32) {
        r.imm_debug_point3(ImmDebugState::default(), *p, color, size);
    }
}

/// The sampler of an image: `Samplers.Point` unless the caller says more.
pub const IMM_DEFAULT_SAMPLER: Samplers = Samplers::Point;
