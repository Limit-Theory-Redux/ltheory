//! Pipelines: shader + fixed-function state, hashed and cached
//! (doc/engine/render-api-v2.md, sections 1.2 and 4).

use std::collections::HashMap;

use super::MAX_COLOR_ATTACHMENTS;
use crate::render::{
    BlendMode, CmdPrimitiveType, CullFace, Renderer, ResourceId, Shader, TexFormat,
};

/// Index of a created pipeline. Allocated on the main thread; the executor
/// maps it to its own state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PipelineId(pub u32);

#[luajit_ffi_gen::luajit_ffi(repr = "u32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompareFn {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}

impl CompareFn {
    pub fn to_gl(self) -> u32 {
        use crate::render::gl;
        match self {
            CompareFn::Never => gl::NEVER,
            CompareFn::Less => gl::LESS,
            CompareFn::Equal => gl::EQUAL,
            CompareFn::LessEqual => gl::LEQUAL,
            CompareFn::Greater => gl::GREATER,
            CompareFn::NotEqual => gl::NOTEQUAL,
            CompareFn::GreaterEqual => gl::GEQUAL,
            CompareFn::Always => gl::ALWAYS,
        }
    }
}

/// Primitive topology of indexed mesh draws (`CmdPrimitiveType` minus the
/// quad emulation).
#[luajit_ffi_gen::luajit_ffi(repr = "u32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Topology {
    Points,
    Lines,
    LineStrip,
    Triangles,
    TriangleStrip,
}

impl Topology {
    pub fn primitive(self) -> CmdPrimitiveType {
        match self {
            Topology::Points => CmdPrimitiveType::Points,
            Topology::Lines => CmdPrimitiveType::Lines,
            Topology::LineStrip => CmdPrimitiveType::LineStrip,
            Topology::Triangles => CmdPrimitiveType::Triangles,
            Topology::TriangleStrip => CmdPrimitiveType::TriangleStrip,
        }
    }
}

/// Where a pipeline's vertices come from. The mesh layouts are fixed by
/// `VertexFormat`; the wgpu backend will carry the format here.
#[luajit_ffi_gen::luajit_ffi(repr = "u32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VertexLayout {
    /// Indexed `Mesh` draws (`pass:drawMesh`).
    Mesh,
    /// The built-in unit quad of `pass:drawFullscreen`.
    Fullscreen,
    /// UI vertices of the immediate batcher (`Imm2DVertex`).
    Imm2D,
    /// Position, uv and color vertices of the immediate batcher
    /// (`Imm3DVertex`): debug geometry and the backdrop box.
    Imm3D,
}

#[luajit_ffi_gen::luajit_ffi(repr = "u32")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PolygonMode {
    Fill,
    /// Wireframe (GL; wgpu needs `POLYGON_MODE_LINE`).
    Line,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DepthState {
    pub test: bool,
    pub write: bool,
    pub compare: CompareFn,
}

impl Default for DepthState {
    /// What a pass starts from: no test, writes
    /// on, `LEQUAL`.
    fn default() -> Self {
        Self {
            test: false,
            write: true,
            compare: CompareFn::LessEqual,
        }
    }
}

/// Everything that defines a pipeline. The cache key is the whole value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PipelineDesc {
    /// The shader's resource id.
    pub shader: ResourceId,
    /// The shader's generation (bumped by every hot reload). Part of the
    /// key, so a reloaded shader never reuses a pipeline made with its old
    /// program, whether or not the reload kept the resource id.
    pub shader_generation: u32,
    pub vertex: VertexLayout,
    pub topology: Topology,
    pub blend: BlendMode,
    pub cull: CullFace,
    pub depth: DepthState,
    pub color_formats: [Option<TexFormat>; MAX_COLOR_ATTACHMENTS],
    pub depth_format: Option<TexFormat>,
    pub polygon: PolygonMode,
}

impl PipelineDesc {
    pub fn new(shader: ResourceId) -> Self {
        Self {
            shader,
            shader_generation: 0,
            vertex: VertexLayout::Mesh,
            topology: Topology::Triangles,
            blend: BlendMode::Disabled,
            cull: CullFace::None,
            depth: DepthState::default(),
            color_formats: [None; MAX_COLOR_ATTACHMENTS],
            depth_format: None,
            polygon: PolygonMode::Fill,
        }
    }
}

impl PipelineDesc {
    /// A description for the current program of `shader`: its resource and
    /// generation, so a hot reload makes a different (new) pipeline.
    pub fn for_shader(shader: &Shader) -> Self {
        Self {
            shader_generation: shader.generation(),
            ..Self::new(shader.resource())
        }
    }
}

#[luajit_ffi_gen::luajit_ffi]
impl PipelineDesc {
    /// A description with the legacy defaults: opaque, no culling, no depth
    /// test, writes on, filled triangles from a `Mesh`.
    #[bind(name = "Create")]
    pub fn create(shader: &Shader) -> PipelineDesc {
        Self::for_shader(shader)
    }

    pub fn blend(&mut self, blend: BlendMode) {
        self.blend = blend;
    }

    pub fn cull(&mut self, cull: CullFace) {
        self.cull = cull;
    }

    pub fn depth(&mut self, test: bool, write: bool, compare: CompareFn) {
        self.depth = DepthState {
            test,
            write,
            compare,
        };
    }

    pub fn topology(&mut self, topology: Topology) {
        self.topology = topology;
    }

    pub fn vertex(&mut self, vertex: VertexLayout) {
        self.vertex = vertex;
    }

    pub fn polygon(&mut self, polygon: PolygonMode) {
        self.polygon = polygon;
    }

    /// Format of color attachment `index` of the passes this pipeline draws
    /// in (unused on GL, required by wgpu).
    pub fn color_format(&mut self, index: i32, format: TexFormat) {
        assert!(
            (0..MAX_COLOR_ATTACHMENTS as i32).contains(&index),
            "PipelineDesc:colorFormat: index {index} out of range"
        );
        self.color_formats[index as usize] = Some(format);
    }

    pub fn depth_format(&mut self, format: TexFormat) {
        self.depth_format = Some(format);
    }
}

/// Main-thread hash cache of pipelines. Pipelines are immortal (a bounded
/// set), so there is no eviction.
#[derive(Default)]
pub struct PipelineCache {
    map: HashMap<PipelineDesc, PipelineId>,
    next: u32,
}

impl PipelineCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// The id of `desc`, and `true` if it was just allocated (the caller
    /// must then send `CreatePipeline`).
    pub fn get_or_insert(&mut self, desc: &PipelineDesc) -> (PipelineId, bool) {
        if let Some(&id) = self.map.get(desc) {
            return (id, false);
        }
        let id = PipelineId(self.next);
        self.next += 1;
        self.map.insert(desc.clone(), id);
        (id, true)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

/// Namespace for `Pipeline.Get`.
pub struct Pipeline;

#[luajit_ffi_gen::luajit_ffi]
impl Pipeline {
    /// The `PipelineId` for `desc`, creating the pipeline on first use.
    pub fn get(r: &mut Renderer, desc: &PipelineDesc) -> u32 {
        r.get_pipeline(desc).0
    }
}

impl Renderer {
    pub fn get_pipeline(&mut self, desc: &PipelineDesc) -> PipelineId {
        let (id, created) = self.data.pipelines.get_or_insert(desc);
        if created {
            self.create_pipeline(id, Box::new(desc.clone()));
        }
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_descs_share_an_id() {
        let mut cache = PipelineCache::new();
        let a = PipelineDesc::new(ResourceId(1));
        let mut b = PipelineDesc::new(ResourceId(1));
        let (ida, created_a) = cache.get_or_insert(&a);
        let (idb, created_b) = cache.get_or_insert(&b);
        assert!(created_a && !created_b);
        assert_eq!(ida, idb);

        b.blend = BlendMode::Alpha;
        let (idc, created_c) = cache.get_or_insert(&b);
        assert!(created_c);
        assert_ne!(ida, idc);

        let d = PipelineDesc::new(ResourceId(2));
        assert_ne!(cache.get_or_insert(&d).0, ida);
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn a_new_shader_generation_is_a_new_pipeline() {
        let mut cache = PipelineCache::new();
        let before = PipelineDesc::new(ResourceId(1));
        let mut after = before.clone();
        after.shader_generation = 1;
        let (a, _) = cache.get_or_insert(&before);
        let (b, created) = cache.get_or_insert(&after);
        assert!(created);
        assert_ne!(a, b);
    }
}
