//! wgpu backend for the render-thread command executor.
//!
//! Parallel to [`crate::render::thread::command_executor_gl`]: implements the
//! same `cmd_*` surface against a wgpu 30 device instead of an OpenGL 3.3
//! context, and dispatches every [`RenderCommand`] variant exhaustively.
//! Adding a variant to `RenderCommand` therefore fails to compile until this
//! backend handles it too.
//!
//! Migration stage notes (feat/wgpu):
//! - Surface/device/queue creation: next stage (wires the winit 0.30 window).
//! - Shader compilation via naga's GLSL frontend: shader stage.
//! - Draw calls / swap chain / present: parity stage.
//!
//! Device-bound operations are `unimplemented!` markers until their stage
//! lands; state tracking, resource bookkeeping, uniform/UBO staging and the
//! format mappings below are real and unit-tested. Nothing routes commands
//! here yet — the GL executor remains the active backend.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tracing::warn;

use crate::render::thread::{ExecutorStats, GpuHandle};
use crate::render::{
    BlendMode, CmdPrimitiveType, CommandReply, CullFace, GenericUniformName, ImmVertex,
    InstanceData, InstanceUniformsCmd, RenderCommand, RenderStats, ResourceId, ShaderReloadResult,
    TexFilter, TexFormat, TexWrapMode, VertexFormat,
};
use crate::window::PresentMode;

/// GPU resource stored on the backend owner — wgpu flavor.
///
/// Mirrors `GpuResource` (GL) but holds wgpu objects instead of raw handles.
/// Reflected plain uniform: name -> (block binding, member offset, member
/// size). Built from the naga IR of the ADAPTED source at shader creation;
/// the engine's uniform-location protocol (GetUniformLocationByResource ->
/// SetUniform* by location) is served from this table.
#[derive(Debug, Clone)]
struct UniformRefl {
    name: Arc<str>,
    binding: u32,
    offset: u32,
    size: u32,
    block_size: u32,
}

/// Reflected sampler: name -> (texture binding, sampler binding). Pairing is
/// by the "_tex"/"_samp" suffix the adaptation emits.
#[derive(Debug, Clone)]
struct SamplerRefl {
    name: Arc<str>,
    tex_binding: u32,
    samp_binding: u32,
    /// Texture view dimension of the sampled texture (D2 for sampler2D,
    /// Cube for samplerCube — the layout must match the shader's type).
    view_dimension: wgpu::TextureViewDimension,
}

/// Per-shader reflection (location protocol + bind-group layout).
#[derive(Debug, Clone, Default)]
struct ShaderReflection {
    uniforms: Vec<UniformRefl>,
    samplers: Vec<SamplerRefl>,
    /// Highest bind-group binding the shader touches (layout sizing).
    max_binding: u32,
}

impl ShaderReflection {
    /// Location-space offsets: uniforms 0..n, samplers 1000+ (disjoint so a
    /// sampler slot-set can never alias a uniform write).
    fn sampler_location(&self, index: usize) -> i32 {
        1000 + index as i32
    }
}

/// Cache key for a render pipeline: the shader + every state input that
/// changes the compiled pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PipelineKey {
    shader_id: u64,
    vertex_format: (bool, bool, bool, bool, u32),
    blend: BlendMode,
    cull: CullFace,
    /// `CmdPrimitiveType` discriminant (the enum itself isn't `Hash`).
    primitive: u8,
    depth_test: bool,
    depth_writable: bool,
    wireframe: bool,
    instanced: bool,
    /// Color attachment formats of the target the pipeline renders to
    /// (surface vs offscreen textures differ; deferred targets use up to
    /// 4 attachments). Unused slots default to Rgba8Unorm.
    color_formats: [wgpu::TextureFormat; 4],
    /// Number of ACTIVE color attachments (pipeline targets must match the
    /// pass attachments exactly).
    color_attachment_count: u8,
    /// Depth attachment format of the target (the engine's depth textures
    /// can be Depth24Plus or Depth32Float; the pipeline must match).
    depth_format: Option<wgpu::TextureFormat>,
}

/// Offscreen render target. Color slots mirror GL color attachments 0-3;
/// the depth slot is the depth attachment. Auto-created color/depth textures
/// stand in when the producer never attaches one (matches GL's fresh FBO
/// being incomplete-but-unused in that case).
#[derive(Debug, Clone)]
struct WgpuFramebuffer {
    color: [Option<wgpu::TextureView>; 4],
    color_textures: [Option<wgpu::Texture>; 4],
    /// Color formats of the attachments (validated at draw).
    color_formats: [wgpu::TextureFormat; 4],
    depth: Option<wgpu::TextureView>,
    depth_texture: Option<wgpu::Texture>,
    /// Format of the depth attachment (validated at draw).
    depth_format: Option<wgpu::TextureFormat>,
    width: u32,
    height: u32,
    /// Pending clear from `Clear`; consumed by the next render pass.
    pending_clear_color: Option<[f32; 4]>,
    pending_clear_depth: Option<f32>,
}

impl Default for WgpuFramebuffer {
    fn default() -> Self {
        Self {
            color: [None, None, None, None],
            color_textures: std::array::from_fn(|_| None),
            color_formats: [wgpu::TextureFormat::Rgba8Unorm; 4],
            depth: None,
            depth_texture: None,
            depth_format: None,
            width: 1,
            height: 1,
            pending_clear_color: None,
            pending_clear_depth: None,
        }
    }
}

#[derive(Debug)]
#[expect(dead_code)]
enum WgpuGpuResource {
    /// Compiled shader pair; the sources are kept so hot reload can recompile
    /// without the producer resending them.
    Shader {
        vertex_module: wgpu::ShaderModule,
        fragment_module: wgpu::ShaderModule,
        vertex_src: Arc<str>,
        fragment_src: Arc<str>,
        reflection: ShaderReflection,
    },
    Texture1D {
        texture: wgpu::Texture,
        view: wgpu::TextureView,
        sampler: wgpu::Sampler,
    },
    Texture2D {
        texture: wgpu::Texture,
        view: wgpu::TextureView,
        sampler: wgpu::Sampler,
    },
    Texture3D {
        texture: wgpu::Texture,
        view: wgpu::TextureView,
        sampler: wgpu::Sampler,
    },
    TextureCube {
        texture: wgpu::Texture,
        view: wgpu::TextureView,
        sampler: wgpu::Sampler,
    },
    Mesh {
        vertex_buffer: wgpu::Buffer,
        index_buffer: wgpu::Buffer,
        index_format: wgpu::IndexFormat,
        index_count: u32,
        vertex_count: u32,
        vertex_format: VertexFormat,
    },
    /// Render target (offscreen framebuffer)
    Framebuffer {
        view: wgpu::TextureView,
        format: wgpu::TextureFormat,
        size: (u32, u32),
    },
}

/// A uniform value staged by name.
///
/// OpenGL's uniform *locations* do not exist in wgpu; by-name uniforms are
/// staged here and uploaded as push constants / a uniform buffer at draw time
/// (shader stage). Location-based `SetUniform*` commands carry values for the
/// currently bound GL program — the wgpu backend stages them under fixed
/// keys that the naga-rewritten shaders will consume via UBO instead.
#[derive(Debug, Clone)]
#[expect(dead_code)]
enum UniformValue {
    Int(i32),
    Int2([i32; 2]),
    Int3([i32; 3]),
    Int4([i32; 4]),
    Float(f32),
    Float2([f32; 2]),
    Float3([f32; 3]),
    Float4([f32; 4]),
    Mat4([f32; 16]),
}

/// wgpu backend for [`RenderCommand`] execution.
///
/// Owns the wgpu instance/adapter/device/queue/surface (created in the
/// surface/device stage) and every wgpu GPU object, and executes
/// `RenderCommand`s against them. Same contract as [`CommandExecutor`]:
/// it has no idea whether it runs on the backend owner or inline.
#[derive(Debug)]
pub struct WgpuCommandExecutor {
    // === wgpu handles (created in the surface/device/queue stage) ===
    instance: Option<wgpu::Instance>,
    adapter: Option<wgpu::Adapter>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,

    /// Resources by id (mirrors `CommandExecutor::resources`)
    resources: HashMap<ResourceId, WgpuGpuResource>,
    /// Requested `(mag_filter, min_filter)` per texture. Wgpu samplers are
    /// immutable, while the GL API updates one texture parameter at a time;
    /// retaining both axes prevents a sequential setter from resetting the
    /// other axis to its default.
    texture_filter_modes: HashMap<ResourceId, (wgpu::FilterMode, wgpu::FilterMode)>,

    // === Cached render state (mirrors the GL executor's cache) ===
    viewport: Option<(i32, i32, i32, i32)>,
    scissor: Option<(i32, i32, i32, i32)>,
    scissor_enabled: bool,
    blend_mode: BlendMode,
    cull_face: CullFace,
    depth_test: bool,
    depth_writable: bool,
    wireframe: bool,
    line_width: f32,
    point_size: f32,

    /// Currently bound shader (by `GpuHandle` value)
    bound_shader: Option<GpuHandle>,
    /// Hot-reloaded shader pairs by shader_key (mirrors the GL executor's
    /// `hot_reloaded_shaders`): `ReloadShader` compiles a fresh pair under the
    /// key; `BindShaderByResource` prefers it over the resource's original.
    hot_reloaded_shaders:
        HashMap<String, (wgpu::ShaderModule, wgpu::ShaderModule, ShaderReflection)>,
    /// Shader key of the currently bound hot-reloaded pair, if any.
    bound_hot_shader: Option<String>,
    /// Currently bound mesh (by `GpuHandle` value)
    bound_mesh: Option<GpuHandle>,
    /// Texture unit bindings (unit -> handle). wgpu has no texture units;
    /// kept for parity bookkeeping and mapped to bind-group entries at draw.
    bound_textures: Vec<Option<GpuHandle>>,
    /// By-name uniform cache: name -> latest value
    named_uniforms: HashMap<Arc<str>, UniformValue>,

    // === UBO staging (raw bytes; uploaded at draw time) ===
    camera_ubo: Option<Vec<u8>>,
    material_ubo: Option<Vec<u8>>,
    light_ubo: Option<Vec<u8>>,

    // === Draw path (stage 5) ===
    /// Per-binding uniform buffers, created lazily on first draw.
    ubo_buffers: HashMap<u32, wgpu::Buffer>,
    /// Plain-uniform staging bytes per binding (from reflection; written by
    /// SetUniform* and uploaded when dirty).
    plain_uniform_staging: HashMap<u32, Vec<u8>>,
    plain_uniform_dirty: std::collections::HashSet<u32>,
    /// Sampler name -> texture slot (set via SetUniformInt on a sampler
    /// location; slot indexes `bound_textures`).
    sampler_slots: HashMap<Arc<str>, u32>,
    /// Pipeline cache keyed by (shader, vertex format, draw state).
    pipeline_cache: HashMap<PipelineKey, wgpu::RenderPipeline>,
    /// Bind-group cache keyed by (shader id, sampler-slot configuration).
    bind_group_cache: HashMap<(u64, u64), wgpu::BindGroup>,
    /// Offscreen render targets by framebuffer id (persist across pushes).
    framebuffers: HashMap<u64, WgpuFramebuffer>,
    /// Current render-target stack; empty stack = the surface.
    framebuffer_stack: Vec<WgpuFramebuffer>,
    /// Acquired surface frame (presented on SwapBuffers).
    surface_frame: Option<wgpu::SurfaceTexture>,
    surface_size: (u32, u32),
    /// Scratch buffer for DrawImmediate vertex uploads.
    immediate_buffer: Option<wgpu::Buffer>,
    immediate_capacity: u64,
    /// Scratch buffer for DrawInstancedWithData per-instance attributes.
    instance_buffer: Option<wgpu::Buffer>,
    instance_capacity: u64,
    /// Screen-sized depth texture backing the surface target (surfaces have
    /// no depth; the engine's default framebuffer does).
    surface_depth: Option<(wgpu::Texture, u32, u32)>,
    /// Pending clears for the surface target.
    surface_pending_clear_color: Option<[f32; 4]>,
    surface_pending_clear_depth: Option<f32>,
    /// One encoder per frame: draws accumulate into it, the swap submits it.
    current_encoder: Option<wgpu::CommandEncoder>,
    /// 1x1 white texture + linear sampler for unbound sampler slots.
    default_texture: Option<wgpu::Texture>,
    default_sampler: Option<wgpu::Sampler>,
    /// 1x1 white CUBE texture for unbound Cube-dimension sampler slots
    /// (layout validation rejects a D2 view for a Cube binding).
    default_cube_texture: Option<wgpu::Texture>,
    /// 1x1 white D1 texture for unbound 1D sampler slots.
    default_1d_texture: Option<wgpu::Texture>,
    /// 1x1x1 white D3 texture for unbound 3D sampler slots.
    default_3d_texture: Option<wgpu::Texture>,

    // === Stats (mirrors ExecutorStats + RenderStats plumbing) ===
    stats: ExecutorStats,
    last_stats: RenderStats,
    draw_mesh_calls_this_frame: u64,
    draw_instanced_calls_this_frame: u64,
    draw_immediate_calls_this_frame: u64,
    instanced_data_items_this_frame: u64,
    immediate_vertices_this_frame: u64,
    vertices_drawn_this_frame: u64,
    commands_this_frame: u64,
    state_changes_this_frame: u64,
}

#[expect(dead_code)]
impl WgpuCommandExecutor {
    pub fn new() -> Self {
        Self::with_device(None, None)
    }

    /// Attach a wgpu device/queue (created by [`crate::window::WgpuRenderer`]).
    /// The render-thread wiring and the shader/parity tests use this; until
    /// then the executor is a state-only backend.
    pub fn with_device(device: Option<wgpu::Device>, queue: Option<wgpu::Queue>) -> Self {
        Self {
            instance: None,
            adapter: None,
            surface: None,
            surface_config: None,
            device,
            queue,
            resources: HashMap::new(),
            texture_filter_modes: HashMap::new(),
            viewport: None,
            scissor: None,
            scissor_enabled: false,
            blend_mode: BlendMode::Disabled,
            cull_face: CullFace::None,
            depth_test: false,
            depth_writable: true,
            wireframe: false,
            line_width: 1.0,
            point_size: 1.0,
            bound_shader: None,
            hot_reloaded_shaders: HashMap::new(),
            bound_hot_shader: None,
            bound_mesh: None,
            bound_textures: vec![None; 16],
            named_uniforms: HashMap::new(),
            camera_ubo: None,
            material_ubo: None,
            light_ubo: None,
            ubo_buffers: HashMap::new(),
            plain_uniform_staging: HashMap::new(),
            plain_uniform_dirty: std::collections::HashSet::new(),
            sampler_slots: HashMap::new(),
            pipeline_cache: HashMap::new(),
            bind_group_cache: HashMap::new(),
            framebuffers: HashMap::new(),
            framebuffer_stack: Vec::new(),
            surface_frame: None,
            surface_size: (320, 240),
            immediate_buffer: None,
            immediate_capacity: 0,
            instance_buffer: None,
            instance_capacity: 0,
            surface_depth: None,
            surface_pending_clear_color: None,
            surface_pending_clear_depth: None,
            current_encoder: None,
            default_texture: None,
            default_sampler: None,
            default_cube_texture: None,
            default_1d_texture: None,
            default_3d_texture: None,
            stats: ExecutorStats::default(),
            last_stats: RenderStats::default(),
            draw_mesh_calls_this_frame: 0,
            draw_instanced_calls_this_frame: 0,
            draw_immediate_calls_this_frame: 0,
            instanced_data_items_this_frame: 0,
            immediate_vertices_this_frame: 0,
            vertices_drawn_this_frame: 0,
            commands_this_frame: 0,
            state_changes_this_frame: 0,
        }
    }

    /// Attach the surface bundle (render-thread startup path). The surface
    /// drives present via `SwapBuffers`; the config is reconfigured on
    /// `Resize`.
    pub fn with_surface(
        mut self,
        surface: wgpu::Surface<'static>,
        surface_config: wgpu::SurfaceConfiguration,
        width: u32,
        height: u32,
    ) -> Self {
        self.surface = Some(surface);
        self.surface_config = Some(surface_config);
        self.surface_size = (width.max(1), height.max(1));
        self
    }

    /// Release GPU objects in an explicit owner-controlled order. No device
    /// poll or queue wait is performed here: those are foreign-driver waits
    /// with no safe cancellation boundary. Dropping the surface frame and
    /// encoders first prevents their RAII cleanup from outliving the surface;
    /// clearing resources and caches before the device keeps the ownership
    /// graph finite and inspectable.
    pub(super) fn cleanup(&mut self) {
        self.current_encoder.take();
        self.surface_frame.take();
        self.framebuffer_stack.clear();
        self.framebuffers.clear();
        self.bind_group_cache.clear();
        self.pipeline_cache.clear();
        self.ubo_buffers.clear();
        self.texture_filter_modes.clear();
        self.resources.clear();
        self.hot_reloaded_shaders.clear();
        self.immediate_buffer.take();
        self.instance_buffer.take();
        self.surface_depth.take();
        self.default_texture.take();
        self.default_sampler.take();
        self.default_cube_texture.take();
        self.default_1d_texture.take();
        self.default_3d_texture.take();
        self.surface.take();
        self.surface_config.take();
        self.queue.take();
        self.device.take();
        self.adapter.take();
        self.instance.take();
    }

    /// Mirrors `CommandExecutor::set_category_timing` (stats dashboard
    /// mode). The wgpu executor reports aggregate stats only; the flag is
    /// accepted for interface parity.
    pub fn set_category_timing(&self, _timing: Arc<std::sync::atomic::AtomicBool>) {}

    /// Mirrors `CommandExecutor::stats()`.
    pub fn stats(&self) -> &ExecutorStats {
        &self.stats
    }
    // =====================================================================
    // Shader compilation (naga GLSL frontend — stage 4)
    // =====================================================================

    /// Compile one GLSL stage into a wgpu ShaderModule via naga's GLSL
    /// frontend. The source must already be engine-preprocessed (includes
    /// inlined, `#autovar` stripped) — exactly what the producer sends in
    /// `CreateShader`/`ReloadShader`.
    /// Adapt engine GLSL for naga's frontend WITHOUT touching the on-disk
    /// sources (those stay GL 3.3 / #version 330):
    /// - naga 30 accepts only 440/450/460 -> bump `#version 330` to 440.
    /// - naga requires explicit `layout(binding=N)` on uniform blocks ->
    ///   inject the engine's UBO bindings (CAMERA=0, MATERIAL=1, LIGHT=2,
    ///   see `render/thread/ubo.rs`).
    pub(crate) fn adapt_glsl_for_naga(code: &str) -> String {
        Self::adapt_glsl_for_naga_with_offset(code, 0)
    }

    /// Like [`Self::adapt_glsl_for_naga`], but the plain-uniform/sampler
    /// binding counter starts at `10 + offset`.
    ///
    /// The GLSL frontend parses each stage separately, so a naive per-stage
    /// adaptation would give the vertex and fragment stages the SAME binding
    /// numbers (both start at 10). Shader pairs merge both stages into one
    /// pipeline layout, so the fragment stage must start past the vertex
    /// stage's highest binding — otherwise wgpu fails pipeline creation with
    /// "Conflicting binding at index N".
    pub(crate) fn adapt_glsl_for_naga_with_offset(code: &str, offset: u32) -> String {
        use crate::render::thread::ubo::{
            CAMERA_UBO_BINDING, LIGHT_UBO_BINDING, MATERIAL_UBO_BINDING,
        };
        let mut out = code.replace("#version 330", "#version 440");
        let injections = [
            ("CameraUBO", CAMERA_UBO_BINDING),
            ("MaterialUBO", MATERIAL_UBO_BINDING),
            ("LightUBO", LIGHT_UBO_BINDING),
        ];
        for (name, binding) in injections {
            let pattern = format!("layout(std140) uniform {name}");
            let replacement = format!("layout(std140, binding={binding}) uniform {name}");
            out = out.replace(&pattern, &replacement);
        }
        Self::split_mat4_attributes(&mut out);
        Self::inject_varying_locations(&mut out);
        Self::inject_plain_uniform_bindings_with_offset(&mut out, offset);
        Self::rewrite_sampler_params(&mut out);
        Self::ensure_missing_fn_shims(&mut out);
        Self::rename_sample_identifier(&mut out);
        out
    }

    /// naga 30 cannot type `sampler*D` function parameters, and helper
    /// functions taking samplers are widespread (sampleTriplanar & friends in
    /// include/texturing.glsl, FXAA, moon/planet detail helpers). For every
    /// function with sampler params: split each param into the texture+sampler
    /// pair, rewrite the body's texture() calls into the sampler<D>(tex, samp)
    /// constructor form, and expand the matching argument at every call site
    /// (the argument is a sampler uniform, renamed to <name>_tex/_samp by the
    /// uniform split). Must run AFTER inject_plain_uniform_bindings.
    fn rewrite_sampler_params(code: &mut String) {
        // Pass 1: collect (fn_name, param_name, dim, param_index) for every
        // sampler-typed function parameter.
        let text: Vec<char> = code.chars().collect();
        let mut sigs: Vec<(String, String, String, usize)> = Vec::new();
        let mut i = 0usize;
        while i < text.len() {
            if text[i..].starts_with(&['s', 'a', 'm', 'p', 'l', 'e', 'r'][..]) && i > 0 {
                // whitespace-tolerant: "pos, sampler2D tex" has a space
                // between the comma and the type
                let mut prev_pos = i - 1;
                while prev_pos > 0 && text[prev_pos] == ' ' {
                    prev_pos -= 1;
                }
                let prev_ok = prev_pos == 0 || text[prev_pos] == '(' || text[prev_pos] == ',';
                if !prev_ok {
                    i += 1;
                    continue;
                }
                let mut j = i + 7;
                while j < text.len() && text[j].is_ascii_alphanumeric() {
                    j += 1;
                }
                let dim: String = text[i + 7..j].iter().collect();
                let mut k = j;
                while k < text.len() && text[k] == ' ' {
                    k += 1;
                }
                let mut m = k;
                while m < text.len() && (text[m].is_ascii_alphanumeric() || text[m] == '_') {
                    m += 1;
                }
                let pname: String = text[k..m].iter().collect();
                if !dim.is_empty() && !pname.is_empty() {
                    // Enclosing fn '(' via back-scan with paren depth.
                    let mut depth = 0usize;
                    let mut b = i - 1;
                    while b > 0 {
                        match text[b] {
                            ')' => depth += 1,
                            '(' => {
                                if depth == 0 {
                                    break;
                                }
                                depth -= 1;
                            }
                            _ => {}
                        }
                        b -= 1;
                    }
                    // fn name = identifier before b
                    let mut f = b;
                    while f > 0 && (text[f - 1].is_ascii_alphanumeric() || text[f - 1] == '_') {
                        f -= 1;
                    }
                    let fname: String = text[f..b].iter().collect();
                    // param index = top-level commas between b and i-1
                    // (empty range = the sampler is the first parameter)
                    let mut idx = 0usize;
                    let mut d2 = 0usize;
                    if b + 1 < i {
                        for c in &text[b + 1..i - 1] {
                            match c {
                                '(' => d2 += 1,
                                ')' => d2 -= 1,
                                ',' if d2 == 0 => idx += 1,
                                _ => {}
                            }
                        }
                    }
                    if !fname.is_empty() && !fname.starts_with("sampler") {
                        sigs.push((fname, pname, dim, idx));
                    }
                }
            }
            i += 1;
        }
        if sigs.is_empty() {
            return;
        }
        sigs.sort();
        sigs.dedup();

        // Pass 2: signature + body use-site rewrites.
        for (_, pname, dim, _) in &sigs {
            // A param literally named `sampler`/`texture` would collide with
            // naga's type names: rename the texture side to <P>_tex then.
            let tex_param = if pname == "sampler" || pname == "texture" {
                format!("{pname}_tex")
            } else {
                pname.clone()
            };
            // signature: sampler<D> <P> -> texture<D> <P>, sampler <P>_s
            let sig_old = format!("sampler{dim} {pname}");
            let sig_new = format!("texture{dim} {tex_param}, sampler {pname}_s");
            *code = code.replace(&sig_old, &sig_new);
            // body: texture(<P>, -> texture(sampler<D>(<P>[, <P>_tex], <P>_s),
            let use_old = format!("texture({pname},");
            let use_new = format!("texture(sampler{dim}({tex_param}, {pname}_s),");
            *code = code.replace(&use_old, &use_new);
        }

        // FXAA texture macros take the sampler as a macro argument; rewrite
        // the definitions to build the combined sampler from the function's
        // split params (the macro is only ever expanded inside FxaaPixelShader,
        // where tex_s is in scope).
        if code.contains("FxaaTexTop") {
            *code = code.replace(
                "#define FxaaTexTop(t, p) texture(t, p)",
                "#define FxaaTexTop(t, p) texture(sampler2D(t, tex_s), p)",
            );
            *code = code.replace(
                "#define FxaaTexOff(t, p, o) texture(t, p + o)",
                "#define FxaaTexOff(t, p, o) texture(sampler2D(t, tex_s), p + o)",
            );
        }

        // Pass 3: call sites — expand the sampler argument at its index.
        for (fname, _pname, _dim, idx) in &sigs {
            let needle = format!("{fname}(");
            let mut out = String::with_capacity(code.len());
            let mut cursor = 0usize;
            while let Some(rel) = code[cursor..].find(&needle) {
                let start = cursor + rel;
                out.push_str(&code[cursor..start + needle.len()]);
                // walk to the matching ')'
                let mut depth = 1usize;
                let mut p = start + needle.len();
                while p < code.len() && depth > 0 {
                    match code.as_bytes()[p] {
                        b'(' => depth += 1,
                        b')' => depth -= 1,
                        _ => {}
                    }
                    p += 1;
                }
                let args_text = &code[start + needle.len()..p - 1];
                let mut args: Vec<String> = Vec::new();
                let mut cur = String::new();
                let mut d = 0usize;
                for ch in args_text.chars() {
                    match ch {
                        '(' => {
                            d += 1;
                            cur.push(ch);
                        }
                        ')' => {
                            d -= 1;
                            cur.push(ch);
                        }
                        ',' if d == 0 => {
                            args.push(cur.trim().to_string());
                            cur.clear();
                        }
                        _ => cur.push(ch),
                    }
                }
                if !cur.trim().is_empty() {
                    args.push(cur.trim().to_string());
                }
                // the signature itself? its arg at the sampler index is a
                // TYPE + NAME pair ("texture2D s" — contains a space), while
                // real call args are plain identifiers (a uniform may even be
                // literally named "sampler").
                let is_sig = args
                    .get(*idx)
                    .map(|a| a.contains(' ') || a.starts_with("texture"))
                    .unwrap_or(false);
                if !is_sig && *idx < args.len() {
                    let arg = args[*idx].clone();
                    // Plain identifier? (the signature's own args contain a
                    // space after the type name and are excluded by this)
                    if !arg.is_empty() && arg.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    {
                        // The arg is either a sampler UNIFORM (split by the
                        // uniform pass into <X>_tex/<X>_samp) or a sampler
                        // FUNCTION PARAM (rewritten by pass 2 into
                        // <X>_tex/<X>_s, visible in a rewritten signature as
                        // "sampler <X>_s," or "sampler <X>_s)"). The trailing
                        // boundary keeps "_s" from matching inside "_samp".
                        let param_style = code.contains(&format!("{arg}_s,"))
                            || code.contains(&format!("{arg}_s)"));
                        if param_style {
                            args[*idx] = format!("{arg}_tex, {arg}_s");
                        } else {
                            args[*idx] = format!("{arg}_tex, {arg}_samp");
                        }
                    }
                }
                out.push_str(&args.join(", "));
                out.push(')');
                cursor = p;
            }
            out.push_str(&code[cursor..]);
            *code = out;
        }
    }

    /// Split `in mat4` vertex attributes into four vec4 attributes.
    ///
    /// naga 30 rejects matrix varyings (NotIOShareableType), but the engine
    /// feeds per-instance model matrices as a mat4 attribute (wvp_instanced*:
    /// locations 4-7). Emit four vec4 columns at the same locations and a
    /// #define that reconstructs the matrix, so shader bodies keep using
    /// `modelMatrix` unchanged. GLSL mat4(vec4, vec4, vec4, vec4) is
    /// column-major, matching GL's layout.
    fn split_mat4_attributes(code: &mut String) {
        let mut out_lines: Vec<String> = Vec::with_capacity(code.lines().count() + 8);
        for line in code.lines() {
            let trimmed = line.trim_start();
            let mut explicit_loc: Option<u32> = None;
            let rest = if let Some(l) = trimmed.strip_prefix("layout(location = ") {
                let mut parts = l.splitn(2, ')');
                explicit_loc = parts.next().and_then(|v| v.trim().parse().ok());
                parts.next().unwrap_or("").trim_start()
            } else {
                trimmed
            };
            if rest.starts_with("in mat4 ") {
                let name = rest["in mat4 ".len()..]
                    .trim_end_matches(';')
                    .trim()
                    .to_string();
                if !name.is_empty() {
                    let base = explicit_loc.unwrap_or(106);
                    let indent = &line[..line.len() - line.trim_start().len()];
                    out_lines.push(format!(
                        "{indent}layout(location = {base}) in vec4 {name}_c0;"
                    ));
                    out_lines.push(format!(
                        "{indent}layout(location = {base}+1) in vec4 {name}_c1;"
                    ));
                    out_lines.push(format!(
                        "{indent}layout(location = {base}+2) in vec4 {name}_c2;"
                    ));
                    out_lines.push(format!(
                        "{indent}layout(location = {base}+3) in vec4 {name}_c3;"
                    ));
                    out_lines.push(format!(
                        "{indent}#define {name} mat4({name}_c0, {name}_c1, {name}_c2, {name}_c3)"
                    ));
                    continue;
                }
            }
            out_lines.push(line.to_string());
        }
        *code = out_lines.join("\n");
    }

    /// Rewrite plain uniforms into naga-parseable forms.
    ///
    /// naga 30's GLSL frontend does NOT support plain `uniform TYPE NAME;`
    /// declarations at all (probe-verified: NotImplemented("variable
    /// qualifier") even WITH layout(binding=...)), and it dropped the
    /// combined sampler types (`sampler2D`, `samplerCube`, ...) in favor of
    /// separate `texture*` + `sampler` declarations combined at the use site
    /// via the `sampler2D(tex, samp)` constructor. Two translations that
    /// keep shader bodies semantically identical:
    /// - samplers: `uniform sampler2D src;` becomes
    ///   `layout(binding=N) uniform texture2D src_tex;` +
    ///   `layout(binding=N+1) uniform sampler src_samp;`, and every use site
    ///   `texture(src, ...)` becomes `texture(sampler2D(src_tex, src_samp), ...)`
    /// - everything else -> an ANONYMOUS std140 block
    ///   `layout(std140, binding=N) uniform <Name>_UBO { <type> <name>; };`
    ///   (anonymous block members are hoisted to global scope, so the
    ///   shader's existing references to `name` keep working).
    /// Bindings start at 10 (0/1/2 = camera/material/light UBOs).
    fn inject_plain_uniform_bindings(code: &mut String) {
        Self::inject_plain_uniform_bindings_with_offset(code, 0);
    }

    fn inject_plain_uniform_bindings_with_offset(code: &mut String, offset: u32) {
        const TEXTURE_FNS: [&str; 11] = [
            "textureProjOffset",
            "textureGradOffset",
            "textureLodOffset",
            "textureGatherOffset",
            "textureProjGrad",
            "textureQueryLod",
            "textureGather",
            "textureProj",
            "textureGrad",
            "textureLod",
            "textureSize",
            // Plain "texture(NAME," is handled separately AFTER the qualified
            // forms: its pattern cannot match inside "textureLod(NAME," etc.
            // (the qualifier sits between "texture" and "("), and handling it
            // last keeps it from re-touching the inserted constructors.
        ];
        // These take the TEXTURE directly (no sampler): texelFetch,
        // textureSize, textureQueryLevels. The rewrite is NAME -> NAME_tex
        // WITHOUT the sampler<D>(...) constructor.
        const TEXTURE_ONLY_FNS: [&str; 3] = ["texelFetch", "textureSize", "textureQueryLevels"];

        let mut binding = 10u32 + offset;
        let mut samplers: Vec<(String, String)> = Vec::new(); // (name, dim)
        let mut bools: Vec<String> = Vec::new(); // bool plain-uniform names
        let mut plains: Vec<(String, String)> = Vec::new(); // (ty, name)
        let mut out_lines: Vec<String> = Vec::with_capacity(code.lines().count() + 8);
        for line in code.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("uniform ")
                && !trimmed.contains('{')
                && !trimmed.contains("layout")
            {
                let mut tokens = trimmed["uniform ".len()..].split_whitespace();
                let ty = tokens.next().unwrap_or("");
                let name = tokens.next().unwrap_or("").trim_end_matches(';');
                if !ty.is_empty() && !name.is_empty() {
                    let indent = &line[..line.len() - line.trim_start().len()];
                    let is_sampler = ty.starts_with("sampler")
                        || ty.starts_with("isampler")
                        || ty.starts_with("usampler");
                    if is_sampler {
                        // Separate texture + sampler declarations; the
                        // combined name (e.g. "sampler2D") lives on in the
                        // constructor at the use sites below.
                        let dim = ty
                            .strip_prefix("sampler")
                            .or_else(|| ty.strip_prefix("isampler"))
                            .or_else(|| ty.strip_prefix("usampler"))
                            .unwrap_or("")
                            .to_string();
                        let shadow = ty.ends_with("Shadow");
                        out_lines.push(format!(
                            "{indent}layout(binding={binding}) uniform texture{dim} {name}_tex;"
                        ));
                        binding += 1;
                        out_lines.push(format!(
                            "{indent}layout(binding={binding}) uniform sampler{} {name}_samp;",
                            if shadow { "Shadow" } else { "" }
                        ));
                        binding += 1;
                        samplers.push((name.to_string(), dim));
                        continue;
                    }
                    if ty == "bool" {
                        // naga validation rejects bool members in uniform
                        // blocks; store as float and rewrite if() uses below.
                        bools.push(name.to_string());
                        continue;
                    }
                    plains.push((ty.to_string(), name.to_string()));
                    continue;
                }
            }
            out_lines.push(line.to_string());
        }
        // Pack ALL plain uniforms (and float-ified bools) into ONE shared
        // std140 block: wgpu limits uniform buffers per stage to
        // max_uniform_buffers_per_shader_stage (12 on the default limits) and
        // the engine's shaders can declare 4+ plains (e.g. gen/planet: seed,
        // freq, power, coef + the ui vertex's mWorldViewUI/mProjUI).
        //
        // The block MUST be declared BEFORE the first use: GLSL requires
        // declaration-before-use and the naga frontend rejects a bare member
        // reference before its block (probe-verified: appending the block at
        // the end fails every shader with UnknownVariable("<member>")).
        if !plains.is_empty() || !bools.is_empty() {
            let mut block = format!("layout(std140, binding={binding}) uniform _phx_plain_UBO {{");
            for (ty, name) in &plains {
                block.push_str(&format!("\n    {ty} {name};"));
            }
            for name in &bools {
                block.push_str(&format!("\n    float {name};"));
            }
            block.push_str("};");
            // Insert AFTER the first #version line (it can sit after a
            // leading comment block in the preprocessed source) so the
            // declarations precede every use site.
            let insert_at = out_lines
                .iter()
                .position(|l| l.starts_with("#version"))
                .map(|i| i + 1)
                .unwrap_or(0);
            out_lines.insert(insert_at, block);
        }
        // Rewrite use sites: texture(S, ...) -> texture(sampler2D(S_tex, S_samp), ...)
        let mut joined = out_lines.join("\n");
        for (name, dim) in &samplers {
            let mut rewritten = joined.clone();
            for prefix in TEXTURE_FNS {
                let pattern = format!("{prefix}({name},");
                let replacement = format!("{prefix}(sampler{dim}({name}_tex, {name}_samp),");
                rewritten = rewritten.replace(&pattern, &replacement);
            }
            // Plain texture(...) last (see comment above).
            let pattern = format!("texture({name},");
            let replacement = format!("texture(sampler{dim}({name}_tex, {name}_samp),");
            rewritten = rewritten.replace(&pattern, &replacement);
            // Texture-only functions: texelFetch(NAME, -> texelFetch(NAME_tex,
            for prefix in TEXTURE_ONLY_FNS {
                let pattern = format!("{prefix}({name},");
                let replacement = format!("{prefix}({name}_tex,");
                rewritten = rewritten.replace(&pattern, &replacement);
            }
            joined = rewritten;
        }
        // bool uniforms: `if (x)` / `if (!x)` -> float comparisons
        for name in &bools {
            joined = joined.replace(&format!("if ({name})"), &format!("if ({name} != 0.0)"));
            joined = joined.replace(&format!("if (!{name})"), &format!("if ({name} == 0.0)"));
        }
        *code = joined;
    }

    /// naga 30 has no `saturate` builtin and the engine uses it in float form
    /// (radialminmax defines its own vec2 overload, which coexists). Also
    /// patch missing helper definitions: invGamma is USED by
    /// tonemap_limittheory.glsl but never defined anywhere in the shader set.
    /// Pure convenience shims, inserted after the #version line, only when
    /// used-but-missing.
    fn ensure_missing_fn_shims(code: &mut String) {
        let mut shims = String::new();
        if code.contains("saturate(") {
            // naga 30 has no saturate builtin; emit every overload the shader
            // does not already define (radialminmax defines its own vec2 form).
            for (ty, zero, one) in [
                ("float", "0.0", "1.0"),
                ("vec2", "vec2(0.0)", "vec2(1.0)"),
                ("vec3", "vec3(0.0)", "vec3(1.0)"),
                ("vec4", "vec4(0.0)", "vec4(1.0)"),
            ] {
                let def = format!("{ty} saturate({ty} x)");
                if !code.contains(&def) {
                    shims.push_str(&format!("{def} {{ return clamp(x, {zero}, {one}); }}\n"));
                }
            }
        }
        if code.contains("invGamma(")
            && !(code.contains("vec3 invGamma(") || code.contains("float invGamma("))
        {
            shims.push_str("vec3 invGamma(vec3 c) { return pow(c, vec3(0.45454545)); }\n");
        }
        if shims.is_empty() {
            return;
        }
        let nl = if code.contains("\r\n") { "\r\n" } else { "\n" };
        let mut lines: Vec<&str> = code.split(nl).collect();
        let mut insert_at = 0usize;
        for (idx, line) in lines.iter().enumerate() {
            if line.trim_start().starts_with("#version") {
                insert_at = idx + 1;
                break;
            }
        }
        let mut shim_lines: Vec<&str> = shims.split('\n').filter(|s| !s.is_empty()).collect();
        let mut tail: Vec<&str> = lines.split_off(insert_at);
        lines.push("");
        lines.append(&mut shim_lines);
        lines.append(&mut tail);
        *code = lines.join(nl);
    }

    /// naga 30 reserves `sample` (sampling qualifier); the engine uses it as
    /// a plain local variable name in a few shaders. Rename whole-word
    /// `sample` -> `sample_` (identifier-boundary aware: `sampleBuffer`,
    /// `sampler2D`, `_samp` etc. are untouched).
    fn rename_sample_identifier(code: &mut String) {
        let mut out = String::with_capacity(code.len());
        let text: Vec<char> = code.chars().collect();
        let mut i = 0usize;
        while i < text.len() {
            if text[i].is_ascii_alphanumeric() || text[i] == '_' {
                let mut j = i;
                while j < text.len() && (text[j].is_ascii_alphanumeric() || text[j] == '_') {
                    j += 1;
                }
                let word: String = text[i..j].iter().collect();
                if word == "sample" {
                    out.push_str("sample_");
                } else {
                    out.push_str(&word);
                }
                i = j;
            } else {
                out.push(text[i]);
                i += 1;
            }
        }
        *code = out;
    }

    /// Inject explicit `layout(location=N)` on varyings and vertex attributes.    /// Inject explicit `layout(location=N)` on varyings and vertex attributes.
    ///
    /// naga 30's GLSL frontend auto-assigns location 0 to EVERY varying
    /// (probe-verified: two inputs both got location 0 -> validation
    /// collision), and wgpu requires fragment inputs to match vertex outputs
    /// BY LOCATION, not by name. GL links by name; this map reproduces the
    /// name-based link in location form:
    /// - vertex attributes: 0..3 (position/normal/uv/color — matches the
    ///   engine's attribute binding convention used by the GL path)
    /// - shared varyings (vertex.outs == fragment.ins): 100.. — offset clear
    ///   of fragment color outputs (the header declares outColor at 0).
    fn inject_varying_locations(code: &mut String) {
        const ATTRIBUTES: [(&str, u32); 4] = [
            ("vertex_position", 0),
            ("vertex_normal", 1),
            ("vertex_uv", 2),
            ("vertex_color", 3),
        ];
        // Fragment INPUT locations are validated against
        // max_color_attachments (8 on the default limits), so the whole
        // inter-stage interface must live in 0..7 — the fragment OUTPUT
        // (outColor) sits at 0 in its own namespace, which is fine.
        const VARYINGS: [(&str, u32); 6] = [
            ("uv", 0),
            ("pos", 1),
            ("normal", 2),
            ("vertNormal", 3),
            ("vertPos", 4),
            ("flogz", 5),
        ];

        let mut out_lines: Vec<String> = Vec::with_capacity(code.lines().count() + 8);
        // Per-shader varyings not in the fixed maps get sequential fallback
        // locations. GL matches varyings by NAME; this reproduces that link
        // in location form.
        let mut next_location = 6u32;
        for line in code.lines() {
            let trimmed = line.trim_start();
            // `layout (...)`-prefixed declarations already carry a location.
            if trimmed.starts_with("layout(") {
                out_lines.push(line.to_string());
                continue;
            }
            // Strip interpolation qualifiers: "flat out", "noperspective in"...
            let mut qualifier = "";
            let mut rest = trimmed;
            for q in ["flat ", "noperspective ", "smooth ", "centroid "] {
                if let Some(r) = rest.strip_prefix(q) {
                    qualifier = q;
                    rest = r;
                    break;
                }
            }
            let mut injected = false;
            if let Some(decl) = rest
                .strip_prefix("in ")
                .or_else(|| rest.strip_prefix("out "))
            {
                // tokens: [in|out] <type> <name> [;]
                let mut tokens = decl.split_whitespace();
                let ty = tokens.next().unwrap_or("");
                let name = tokens.next().unwrap_or("").trim_end_matches(';');
                let direction = if rest.starts_with("in ") {
                    "in "
                } else {
                    "out "
                };
                let loc = ATTRIBUTES
                    .iter()
                    .chain(VARYINGS.iter())
                    .find(|(n, _)| *n == name)
                    .map(|(_, l)| *l)
                    .unwrap_or_else(|| {
                        let l = next_location;
                        next_location += 1;
                        l
                    });
                if !ty.is_empty() && !name.is_empty() {
                    let indent = &line[..line.len() - line.trim_start().len()];
                    out_lines.push(format!(
                        "{indent}layout(location={loc}) {qualifier}{direction}{ty} {name};"
                    ));
                    injected = true;
                }
            }
            if !injected {
                out_lines.push(line.to_string());
            }
        }
        *code = out_lines.join(
            "
",
        );
    }

    pub(crate) fn compile_glsl_stage(
        &self,
        stage: wgpu::naga::ShaderStage,
        code: &str,
    ) -> Result<wgpu::ShaderModule, String> {
        self.compile_glsl_stage_adapted(stage, code).map(|(m, _)| m)
    }

    /// Like [`Self::compile_glsl_stage`] but also returns the ADAPTED source
    /// (the reflection is built from it at shader creation).
    pub(crate) fn compile_glsl_stage_adapted(
        &self,
        stage: wgpu::naga::ShaderStage,
        code: &str,
    ) -> Result<(wgpu::ShaderModule, String), String> {
        // wgpu 30 defers shader errors to pipeline creation, so parse with
        // naga's GLSL frontend FIRST: that is what makes CreateShader/
        // ReloadShader report failures synchronously (GL parity).
        // naga 30: parse is a Frontend method (the free fn was removed).
        let adapted = Self::adapt_glsl_for_naga(code);
        let mut frontend = wgpu::naga::front::glsl::Frontend::default();
        let parse_result = frontend.parse(&wgpu::naga::front::glsl::Options::from(stage), &adapted);
        let module = match parse_result {
            Ok(m) => m,
            Err(errors) => {
                // Include the offending source line(s) in the error for debugging.
                let mut detail = String::new();
                for e in errors.errors.iter().take(4) {
                    let line_no = e
                        .location(&adapted)
                        .map(|l| l.line_number as usize)
                        .unwrap_or(0);
                    let line = adapted
                        .lines()
                        .nth(line_no.saturating_sub(1))
                        .unwrap_or("<eof>")
                        .trim();
                    detail.push_str(&format!(" (line {line_no}: {line})"));
                }
                return Err(format!(
                    "naga GLSL {stage:?} parse failed: {errors}{detail}"
                ));
            }
        };
        // Validate NOW, not at pipeline creation: wgpu 30 treats validation
        // errors as fatal panics, while the GL path surfaces shader errors
        // as a compile failure the engine tolerates. Autovar samplers (the
        // preprocessor drops `#autovar` lines, so the source references
        // undeclared identifiers) must fail here, exactly like GL.
        let mut validator = wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::all(),
        );
        if let Err(e) = validator.validate(&module) {
            return Err(format!("naga GLSL {stage:?} validation failed: {e}"));
        }
        let device = self
            .device
            .as_ref()
            .ok_or_else(|| "wgpu device not attached (shader stage)".to_string())?;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(match stage {
                wgpu::naga::ShaderStage::Vertex => "phx-glsl-vertex",
                wgpu::naga::ShaderStage::Fragment => "phx-glsl-fragment",
                _ => "phx-glsl",
            }),
            source: wgpu::ShaderSource::Glsl {
                shader: std::borrow::Cow::Borrowed(&adapted),
                stage,
                defines: &[],
            },
        });
        Ok((module, adapted))
    }

    // =====================================================================
    // Format/state mappings (pure — mirrored 1:1 from the GL executor)
    // =====================================================================

    /// `cmd_set_blend_mode` semantics (command_executor_gl.rs:1736).
    pub(crate) fn blend_mode_to_wgpu(mode: BlendMode) -> Option<wgpu::BlendState> {
        const fn component(src: wgpu::BlendFactor, dst: wgpu::BlendFactor) -> wgpu::BlendComponent {
            wgpu::BlendComponent {
                src_factor: src,
                dst_factor: dst,
                operation: wgpu::BlendOperation::Add,
            }
        }
        match mode {
            BlendMode::Disabled => None,
            BlendMode::Alpha => Some(wgpu::BlendState {
                color: component(
                    wgpu::BlendFactor::SrcAlpha,
                    wgpu::BlendFactor::OneMinusSrcAlpha,
                ),
                alpha: component(wgpu::BlendFactor::One, wgpu::BlendFactor::OneMinusSrcAlpha),
            }),
            BlendMode::Additive => Some(wgpu::BlendState {
                color: component(wgpu::BlendFactor::One, wgpu::BlendFactor::One),
                alpha: component(wgpu::BlendFactor::One, wgpu::BlendFactor::One),
            }),
            BlendMode::PreMultAlpha => Some(wgpu::BlendState {
                color: component(wgpu::BlendFactor::One, wgpu::BlendFactor::OneMinusSrcAlpha),
                alpha: component(wgpu::BlendFactor::One, wgpu::BlendFactor::OneMinusSrcAlpha),
            }),
        }
    }

    /// `cmd_set_cull_face` semantics (command_executor_gl.rs:1764).
    pub(crate) fn cull_face_to_wgpu(face: CullFace) -> Option<wgpu::Face> {
        match face {
            CullFace::None => None,
            CullFace::Back => Some(wgpu::Face::Back),
            CullFace::Front => Some(wgpu::Face::Front),
        }
    }

    /// `CmdPrimitiveType::to_gl` semantics (render_command.rs:44). Quads are
    /// already drawn as triangles in the GL path; TriangleFan has no wgpu
    /// equivalent and must be expanded on the CPU — reported as `None`.
    pub(crate) fn primitive_to_wgpu(
        primitive: CmdPrimitiveType,
    ) -> Option<wgpu::PrimitiveTopology> {
        match primitive {
            CmdPrimitiveType::Points => Some(wgpu::PrimitiveTopology::PointList),
            CmdPrimitiveType::Lines => Some(wgpu::PrimitiveTopology::LineList),
            CmdPrimitiveType::LineStrip => Some(wgpu::PrimitiveTopology::LineStrip),
            CmdPrimitiveType::Triangles => Some(wgpu::PrimitiveTopology::TriangleList),
            CmdPrimitiveType::TriangleStrip => Some(wgpu::PrimitiveTopology::TriangleStrip),
            // wgpu has no triangle-fan topology; needs CPU expansion to a list.
            CmdPrimitiveType::TriangleFan => None,
            CmdPrimitiveType::Quads => Some(wgpu::PrimitiveTopology::TriangleList),
        }
    }

    /// Sampler filter mapping: `TexFilter` -> (mag, min, mipmap).
    /// Mip-less filters pick `Nearest` mipmap mode; `*Mip*` variants map to
    /// the matching `FilterMode`.
    pub(crate) fn filter_to_wgpu(
        filter: TexFilter,
    ) -> (wgpu::FilterMode, wgpu::FilterMode, wgpu::FilterMode) {
        use wgpu::FilterMode::{Linear, Nearest};
        match filter {
            TexFilter::Point => (Nearest, Nearest, Nearest),
            TexFilter::PointMipPoint => (Nearest, Nearest, Nearest),
            TexFilter::PointMipLinear => (Nearest, Nearest, Linear),
            TexFilter::Linear => (Linear, Linear, Nearest),
            TexFilter::LinearMipPoint => (Linear, Linear, Nearest),
            TexFilter::LinearMipLinear => (Linear, Linear, Linear),
        }
    }

    /// `TexWrapMode` -> `wgpu::AddressMode`. MirrorClamp has no wgpu
    /// equivalent (closest is ClampToEdge); the caller is warned.
    pub(crate) fn wrap_mode_to_wgpu(mode: TexWrapMode) -> wgpu::AddressMode {
        match mode {
            TexWrapMode::Clamp => wgpu::AddressMode::ClampToEdge,
            TexWrapMode::MirrorClamp => {
                warn!("TexWrapMode::MirrorClamp has no wgpu equivalent; using ClampToEdge");
                wgpu::AddressMode::ClampToEdge
            }
            TexWrapMode::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
            TexWrapMode::Repeat => wgpu::AddressMode::Repeat,
        }
    }

    /// `TexFormat` -> `wgpu::TextureFormat`. RGB8 has no wgpu equivalent
    /// (closest is RGBA8); reported as `None` so callers decide.
    pub(crate) fn tex_format_to_wgpu(format: TexFormat) -> Option<wgpu::TextureFormat> {
        use wgpu::TextureFormat as F;
        Some(match format {
            TexFormat::R8 => F::R8Unorm,
            TexFormat::R16 => F::R16Unorm,
            TexFormat::R16F => F::R16Float,
            TexFormat::R32F => F::R32Float,
            TexFormat::RG8 => F::Rg8Unorm,
            TexFormat::RG16 => F::Rg16Unorm,
            TexFormat::RG16F => F::Rg16Float,
            TexFormat::RG32F => F::Rg32Float,
            TexFormat::RGB8 => {
                warn!("TexFormat::RGB8 has no wgpu equivalent; callers should use RGBA8");
                return None;
            }
            TexFormat::RGBA8 => F::Rgba8Unorm,
            TexFormat::RGBA16 => F::Rgba16Unorm,
            TexFormat::RGBA16F => F::Rgba16Float,
            TexFormat::RGBA32F => F::Rgba32Float,
            TexFormat::Depth16 => F::Depth16Unorm,
            TexFormat::Depth24 => F::Depth24Plus,
            TexFormat::Depth32F => F::Depth32Float,
        })
    }

    /// Vertex attribute layout for a [`VertexFormat`] — offsets match the
    /// interleaved `pos/normal/uv/color` layout the GL path uploads
    /// (stride = 32 default; color adds 16 bytes).
    pub(crate) fn vertex_attributes(format: &VertexFormat) -> Vec<wgpu::VertexAttribute> {
        let mut attrs = Vec::with_capacity(4);
        let mut offset = 0u64;
        if format.has_position {
            attrs.push(wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset,
                shader_location: 0,
            });
            offset += 12;
        }
        if format.has_normal {
            attrs.push(wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset,
                shader_location: 1,
            });
            offset += 12;
        }
        if format.has_uv {
            attrs.push(wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset,
                shader_location: 2,
            });
            offset += 8;
        }
        if format.has_color {
            attrs.push(wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset,
                shader_location: 3,
            });
        }
        attrs
    }

    // =====================================================================
    // cmd_* surface — same signatures as command_executor_gl.rs
    // =====================================================================

    // --- State management (real: cached state; applied at draw/pass time) ---

    pub(super) fn cmd_set_viewport(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.viewport = Some((x, y, width, height));
    }

    pub(super) fn cmd_set_scissor(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.scissor = Some((x, y, width, height));
    }

    pub(super) fn cmd_enable_scissor(&mut self, enable: bool) {
        self.scissor_enabled = enable;
    }

    pub(super) fn cmd_set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }

    pub(super) fn cmd_set_cull_face(&mut self, face: CullFace) {
        self.cull_face = face;
    }

    pub(super) fn cmd_set_depth_test(&mut self, enable: bool) {
        self.depth_test = enable;
    }

    pub(super) fn cmd_set_depth_writable(&mut self, enable: bool) {
        self.depth_writable = enable;
    }

    pub(super) fn cmd_set_wireframe(&mut self, enable: bool) {
        self.wireframe = enable;
    }

    pub(super) fn cmd_set_line_width(&mut self, width: f32) {
        // wgpu has no line-width state; kept for parity, applied at draw time
        // as a polyline workaround where possible.
        self.line_width = width;
    }

    pub(super) fn cmd_set_point_size(&mut self, size: f32) {
        self.point_size = size;
    }

    // --- Shader operations (bind bookkeeping real; compile = shader stage) ---

    pub(super) fn cmd_bind_shader(&mut self, handle: GpuHandle) {
        self.bound_shader = Some(handle);
    }

    pub(super) fn cmd_bind_shader_by_resource(
        &mut self,
        id: ResourceId,
        shader_key: Option<String>,
    ) {
        // Mirror the GL executor: if this shader key has a hot-reloaded pair,
        // the reload wins over the resource's original modules.
        self.bound_hot_shader =
            shader_key.filter(|key| self.hot_reloaded_shaders.contains_key(key));
        if !self.resources.contains_key(&id) {
            warn!("wgpu: BindShaderByResource for unknown resource {id:?}");
        }
        self.bound_shader = Some(GpuHandle(id.0 as u32));
    }

    pub(super) fn cmd_unbind_shader(&mut self) {
        self.bound_shader = None;
        self.bound_hot_shader = None;
    }

    pub(super) fn cmd_set_uniform_int(&mut self, location: i32, value: i32) {
        self.named_uniforms
            .insert("__uniform_int".into(), UniformValue::Int(value));
        self.apply_uniform(location, &value.to_le_bytes());
    }

    pub(super) fn cmd_set_uniform_int2(&mut self, location: i32, value: [i32; 2]) {
        self.named_uniforms
            .insert("__uniform_int2".into(), UniformValue::Int2(value));
        let mut b = [0u8; 8];
        b[..4].copy_from_slice(&value[0].to_le_bytes());
        b[4..].copy_from_slice(&value[1].to_le_bytes());
        self.apply_uniform(location, &b);
    }

    pub(super) fn cmd_set_uniform_int3(&mut self, location: i32, value: [i32; 3]) {
        self.named_uniforms
            .insert("__uniform_int3".into(), UniformValue::Int3(value));
        let mut b = [0u8; 12];
        for (i, v) in value.iter().enumerate() {
            b[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        self.apply_uniform(location, &b);
    }

    pub(super) fn cmd_set_uniform_int4(&mut self, location: i32, value: [i32; 4]) {
        self.named_uniforms
            .insert("__uniform_int4".into(), UniformValue::Int4(value));
        let mut b = [0u8; 16];
        for (i, v) in value.iter().enumerate() {
            b[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        self.apply_uniform(location, &b);
    }

    pub(super) fn cmd_set_uniform_float(&mut self, location: i32, value: f32) {
        self.named_uniforms
            .insert("__uniform_float".into(), UniformValue::Float(value));
        self.apply_uniform(location, &value.to_le_bytes());
    }

    pub(super) fn cmd_set_uniform_float2(&mut self, location: i32, value: [f32; 2]) {
        self.named_uniforms
            .insert("__uniform_float2".into(), UniformValue::Float2(value));
        let mut b = [0u8; 8];
        b[..4].copy_from_slice(&value[0].to_le_bytes());
        b[4..].copy_from_slice(&value[1].to_le_bytes());
        self.apply_uniform(location, &b);
    }

    pub(super) fn cmd_set_uniform_float3(&mut self, location: i32, value: [f32; 3]) {
        self.named_uniforms
            .insert("__uniform_float3".into(), UniformValue::Float3(value));
        let mut b = [0u8; 12];
        for (i, v) in value.iter().enumerate() {
            b[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        self.apply_uniform(location, &b);
    }

    pub(super) fn cmd_set_uniform_float4(&mut self, location: i32, value: [f32; 4]) {
        self.named_uniforms
            .insert("__uniform_float4".into(), UniformValue::Float4(value));
        let mut b = [0u8; 16];
        for (i, v) in value.iter().enumerate() {
            b[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        self.apply_uniform(location, &b);
    }

    pub(super) fn cmd_set_uniform_mat4(&mut self, location: i32, value: [f32; 16]) {
        self.named_uniforms
            .insert("__uniform_mat4".into(), UniformValue::Mat4(value));
        let mut b = [0u8; 64];
        for (i, v) in value.iter().enumerate() {
            b[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        self.apply_uniform(location, &b);
    }

    pub(super) fn cmd_set_uniform_int_by_name(&mut self, name: Arc<str>, value: i32) {
        self.named_uniforms
            .insert(name.clone(), UniformValue::Int(value));
        self.apply_uniform_by_name(&name, &value.to_le_bytes());
    }

    pub(super) fn cmd_set_uniform_int2_by_name(&mut self, name: Arc<str>, value: [i32; 2]) {
        self.named_uniforms
            .insert(name.clone(), UniformValue::Int2(value));
        let mut b = [0u8; 8];
        b[..4].copy_from_slice(&value[0].to_le_bytes());
        b[4..].copy_from_slice(&value[1].to_le_bytes());
        self.apply_uniform_by_name(&name, &b);
    }

    pub(super) fn cmd_set_uniform_int3_by_name(&mut self, name: Arc<str>, value: [i32; 3]) {
        self.named_uniforms
            .insert(name.clone(), UniformValue::Int3(value));
        let mut b = [0u8; 12];
        for (i, v) in value.iter().enumerate() {
            b[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        self.apply_uniform_by_name(&name, &b);
    }

    pub(super) fn cmd_set_uniform_int4_by_name(&mut self, name: Arc<str>, value: [i32; 4]) {
        self.named_uniforms
            .insert(name.clone(), UniformValue::Int4(value));
        let mut b = [0u8; 16];
        for (i, v) in value.iter().enumerate() {
            b[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        self.apply_uniform_by_name(&name, &b);
    }

    pub(super) fn cmd_set_uniform_float_by_name(&mut self, name: Arc<str>, value: f32) {
        self.named_uniforms
            .insert(name.clone(), UniformValue::Float(value));
        self.apply_uniform_by_name(&name, &value.to_le_bytes());
    }

    pub(super) fn cmd_set_uniform_float2_by_name(&mut self, name: Arc<str>, value: [f32; 2]) {
        self.named_uniforms
            .insert(name.clone(), UniformValue::Float2(value));
        let mut b = [0u8; 8];
        b[..4].copy_from_slice(&value[0].to_le_bytes());
        b[4..].copy_from_slice(&value[1].to_le_bytes());
        self.apply_uniform_by_name(&name, &b);
    }

    pub(super) fn cmd_set_uniform_float3_by_name(&mut self, name: Arc<str>, value: [f32; 3]) {
        self.named_uniforms
            .insert(name.clone(), UniformValue::Float3(value));
        let mut b = [0u8; 12];
        for (i, v) in value.iter().enumerate() {
            b[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        self.apply_uniform_by_name(&name, &b);
    }

    pub(super) fn cmd_set_uniform_float4_by_name(&mut self, name: Arc<str>, value: [f32; 4]) {
        self.named_uniforms
            .insert(name.clone(), UniformValue::Float4(value));
        let mut b = [0u8; 16];
        for (i, v) in value.iter().enumerate() {
            b[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        self.apply_uniform_by_name(&name, &b);
    }

    pub(super) fn cmd_set_uniform_mat4_by_name(&mut self, name: Arc<str>, value: [f32; 16]) {
        self.named_uniforms
            .insert(name.clone(), UniformValue::Mat4(value));
        let mut b = [0u8; 64];
        for (i, v) in value.iter().enumerate() {
            b[i * 4..i * 4 + 4].copy_from_slice(&v.to_le_bytes());
        }
        self.apply_uniform_by_name(&name, &b);
    }

    /// Fixed-name uniform (`mWorld`/`mWorldIT`): same as the by-name path.
    pub(super) fn cmd_set_uniform_mat4_by_generic_name(
        &mut self,
        name: GenericUniformName,
        value: [f32; 16],
    ) {
        self.cmd_set_uniform_mat4_by_name(Arc::from(name.as_str()), value);
    }

    // --- Texture operations (bind bookkeeping real; upload = texture stage) ---

    pub(super) fn cmd_bind_texture_2d(&mut self, slot: u32, handle: GpuHandle) {
        self.bind_texture_slot(slot, handle);
    }

    pub(super) fn cmd_bind_texture_2d_by_resource(&mut self, slot: u32, id: ResourceId) {
        self.bind_texture_slot_by_resource(slot, id);
    }

    pub(super) fn cmd_bind_texture_1d_by_resource(&mut self, slot: u32, id: ResourceId) {
        self.bind_texture_slot_by_resource(slot, id);
    }

    pub(super) fn cmd_bind_texture_3d(&mut self, slot: u32, handle: GpuHandle) {
        self.bind_texture_slot(slot, handle);
    }

    pub(super) fn cmd_bind_texture_3d_by_resource(&mut self, slot: u32, id: ResourceId) {
        self.bind_texture_slot_by_resource(slot, id);
    }

    pub(super) fn cmd_bind_texture_cube(&mut self, slot: u32, handle: GpuHandle) {
        self.bind_texture_slot(slot, handle);
    }

    pub(super) fn cmd_bind_texture_cube_by_resource(&mut self, slot: u32, id: ResourceId) {
        self.bind_texture_slot_by_resource(slot, id);
    }

    pub(super) fn cmd_unbind_texture(&mut self, slot: u32) {
        if let Some(entry) = self.bound_textures.get_mut(slot as usize) {
            *entry = None;
        }
    }

    fn bind_texture_slot(&mut self, slot: u32, handle: GpuHandle) {
        if let Some(entry) = self.bound_textures.get_mut(slot as usize) {
            *entry = Some(handle);
        } else {
            warn!("wgpu: BindTexture* on slot {slot} beyond bound_textures");
        }
    }

    fn bind_texture_slot_by_resource(&mut self, slot: u32, id: ResourceId) {
        if !self.resources.contains_key(&id) {
            warn!("wgpu: BindTexture*ByResource for unknown resource {id:?}");
        }
        self.bind_texture_slot(slot, GpuHandle(id.0 as u32));
    }

    pub(super) fn cmd_set_texture_2d_mag_filter(&mut self, handle: GpuHandle, filter: TexFilter) {
        self.recreate_sampler(handle, Some(filter), None);
    }

    pub(super) fn cmd_set_texture_2d_min_filter(&mut self, handle: GpuHandle, filter: TexFilter) {
        self.recreate_sampler(handle, None, Some(filter));
    }

    pub(super) fn cmd_set_texture_2d_wrap_mode(&mut self, handle: GpuHandle, mode: TexWrapMode) {
        self.recreate_sampler(handle, None, None);
        let _ = mode;
    }

    pub(super) fn cmd_set_texture_2d_mip_range(
        &mut self,
        _handle: GpuHandle,
        _min_level: i32,
        _max_level: i32,
    ) {
    }

    pub(super) fn cmd_generate_mipmap_2d(&mut self, _handle: GpuHandle) {
        // No mipmap generation yet; textures are created with mip_level_count
        // 1 and linear filtering degrades gracefully.
    }

    pub(super) fn cmd_update_texture_2d_data(
        &mut self,
        handle: GpuHandle,
        width: i32,
        height: i32,
        _internal_format: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    ) {
        self.update_texture_2d_impl(handle, width, height, pixel_format, data_format, &data);
    }

    pub(super) fn cmd_update_texture_2d_data_by_resource(
        &mut self,
        id: ResourceId,
        width: i32,
        height: i32,
        _internal_format: i32,
        pixel_format: u32,
        data_format: u32,
        data: Vec<u8>,
    ) {
        self.update_texture_2d_impl(
            GpuHandle(id.0 as u32),
            width,
            height,
            pixel_format,
            data_format,
            &data,
        );
    }

    /// GL pixel-format enum -> component count; GL data-format enum -> bytes
    /// per component. The engine's update carries the DATA layout in these
    /// (GL_RED/RGB/RGBA + GL_UNSIGNED_BYTE/FLOAT/HALF_FLOAT); the resource
    /// format alone is not enough (an RGB8 texture's wgpu format is
    /// Rgba8Unorm after expansion; an RGBA16F texture's rows are 8 bytes).
    fn pixel_format_components(pixel_format: u32) -> u32 {
        const GL_RED: u32 = 0x1903;
        const GL_RG: u32 = 0x8227;
        const GL_RGB: u32 = 0x1907;
        const GL_RGBA: u32 = 0x1908;
        const GL_LUMINANCE: u32 = 0x1909;
        const GL_DEPTH_COMPONENT: u32 = 0x1902;
        match pixel_format {
            GL_RED | GL_LUMINANCE | GL_DEPTH_COMPONENT => 1,
            GL_RG => 2,
            GL_RGB => 3,
            GL_RGBA => 4,
            _ => 0,
        }
    }

    fn data_format_bytes(data_format: u32) -> u32 {
        const GL_UNSIGNED_BYTE: u32 = 0x1401;
        const GL_BYTE: u32 = 0x1400;
        const GL_UNSIGNED_SHORT: u32 = 0x1403;
        const GL_SHORT: u32 = 0x1402;
        const GL_UNSIGNED_INT: u32 = 0x1405;
        const GL_INT: u32 = 0x1404;
        const GL_FLOAT: u32 = 0x1406;
        const GL_HALF_FLOAT: u32 = 0x140B;
        match data_format {
            GL_BYTE | GL_UNSIGNED_BYTE => 1,
            GL_SHORT | GL_UNSIGNED_SHORT | GL_HALF_FLOAT => 2,
            GL_INT | GL_UNSIGNED_INT | GL_FLOAT => 4,
            _ => 0,
        }
    }

    fn update_texture_2d_impl(
        &mut self,
        handle: GpuHandle,
        width: i32,
        height: i32,
        pixel_format: u32,
        data_format: u32,
        data: &[u8],
    ) {
        let Some(queue) = self.queue.clone() else {
            return;
        };
        let Some(resource) = self.resources.get(&ResourceId(handle.0 as u64)) else {
            warn!("wgpu: update texture for unknown resource {handle:?}");
            return;
        };
        let (texture, tex_width, tex_height, resource_bpp) = match resource {
            WgpuGpuResource::Texture2D { texture, .. } => {
                let tex = texture;
                let info = tex.size();
                (
                    tex.clone(),
                    info.width,
                    info.height,
                    Self::format_bpp(tex.format()),
                )
            }
            _ => {
                warn!("wgpu: update texture for non-2D resource {handle:?}");
                return;
            }
        };
        let w = width.max(1) as u32;
        let h = height.max(1) as u32;
        if w > tex_width || h > tex_height {
            warn!("wgpu: texture update larger than texture ({w}x{h} vs {tex_width}x{tex_height})");
            return;
        }
        // Row stride comes from the DATA's pixel format x data format (the
        // engine sends tightly packed rows: RED/RGB/RGBA components at
        // 1/2/4 bytes each — RGBA16F rows are 8 bytes).
        let comps = Self::pixel_format_components(pixel_format);
        let comp_bytes = Self::data_format_bytes(data_format);
        let src_bpp = if comps > 0 && comp_bytes > 0 {
            comps * comp_bytes
        } else {
            0
        };
        let bpp = if src_bpp > 0 { src_bpp } else { resource_bpp };
        let need_expand = bpp == 3 && resource_bpp == 4;
        // f32 payload into an R16Float texture (created from TexFormat::R32F):
        // convert CPU-side, the WebGPU model forbids filtering on 32F formats.
        let need_half = src_bpp == 4 && texture.format() == wgpu::TextureFormat::R16Float;
        let bpr = (w * bpp).max(1);
        // wgpu needs 256-aligned bytes_per_row; RGB rows expand to RGBA, so
        // the padded stride must be aligned on the EXPANDED row size (w*4),
        // not the source bpr (w*3 can under-align).
        let padded_bpr = if need_expand {
            (w * 4 + 255) & !255
        } else {
            (bpr + 255) & !255
        };
        let mut padded = Vec::with_capacity(padded_bpr as usize * h as usize);
        for row in data.chunks(bpr as usize).take(h as usize) {
            if need_half {
                let half = Self::f32_to_f16_bytes(row);
                padded.extend_from_slice(&half);
                padded.resize(padded.len() + (padded_bpr as usize - half.len()), 0);
            } else if need_expand {
                let mut rgba = Vec::with_capacity(w as usize * 4);
                for px in row.chunks(3).take(w as usize) {
                    rgba.extend_from_slice(&[px[0], px[1], px[2], 255]);
                }
                padded.extend_from_slice(&rgba);
                padded.resize(padded.len() + (padded_bpr as usize - rgba.len()), 0);
            } else {
                padded.extend_from_slice(row);
                padded.resize(padded.len() + (padded_bpr as usize - row.len()), 0);
            }
        }
        if padded.is_empty() || w == 0 || h == 0 {
            // GL tolerates zero-sized uploads; wgpu rejects them.
            return;
        }
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &padded,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Re-create a texture's sampler from the stored filter state. Filter
    /// state is tracked per texture (GL texture params); wgpu samplers are
    /// immutable, so filter changes rebuild the sampler.
    fn recreate_sampler(
        &mut self,
        handle: GpuHandle,
        mag: Option<TexFilter>,
        min: Option<TexFilter>,
    ) {
        let Some(device) = self.device.clone() else {
            return;
        };
        let resource_id = ResourceId(handle.0 as u64);
        let filters = self
            .texture_filter_modes
            .entry(resource_id)
            .or_insert((wgpu::FilterMode::Linear, wgpu::FilterMode::Linear));
        if let Some(filter) = mag {
            filters.0 = Self::filter_mode(filter);
        }
        if let Some(filter) = min {
            filters.1 = Self::filter_mode(filter);
        }
        let (requested_mag, requested_min) = *filters;
        let Some(resource) = self.resources.get_mut(&ResourceId(handle.0 as u64)) else {
            return;
        };
        // 32F formats are NOT filterable in WebGPU: clamp the requested
        // filter to Nearest for them (the engine's setMagFilter(Linear) on
        // the Rgba32Float instance-data texture would otherwise fail
        // bind-group validation).
        let non_filterable = match resource {
            WgpuGpuResource::Texture2D { texture, .. }
            | WgpuGpuResource::Texture1D { texture, .. }
            | WgpuGpuResource::Texture3D { texture, .. }
            | WgpuGpuResource::TextureCube { texture, .. } => matches!(
                texture.format(),
                wgpu::TextureFormat::Rgba32Float | wgpu::TextureFormat::R32Float
            ),
            _ => false,
        };
        let sampler = match resource {
            WgpuGpuResource::Texture2D { sampler, .. }
            | WgpuGpuResource::Texture1D { sampler, .. }
            | WgpuGpuResource::Texture3D { sampler, .. }
            | WgpuGpuResource::TextureCube { sampler, .. } => sampler,
            _ => return,
        };
        let new_filter = if non_filterable {
            wgpu::FilterMode::Nearest
        } else {
            requested_mag
        };
        let min_filter = if non_filterable {
            wgpu::FilterMode::Nearest
        } else {
            requested_min
        };
        *sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("phx-sampler-updated"),
            mag_filter: new_filter,
            min_filter,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });
        // Bind groups own the sampler handle; rebuild them before the next
        // draw so a filter mutation cannot reuse a stale sampler.
        self.bind_group_cache.clear();
    }

    pub(crate) fn filter_mode(filter: TexFilter) -> wgpu::FilterMode {
        match filter {
            TexFilter::Point | TexFilter::PointMipLinear | TexFilter::PointMipPoint => {
                wgpu::FilterMode::Nearest
            }
            _ => wgpu::FilterMode::Linear,
        }
    }

    pub(super) fn cmd_set_texture_2d_anisotropy(&mut self, _handle: GpuHandle, _factor: f32) {}

    pub(super) fn cmd_set_texture_2d_anisotropy_by_resource(
        &mut self,
        _id: ResourceId,
        _factor: f32,
    ) {
    }

    pub(super) fn cmd_set_texture_2d_mip_range_by_resource(
        &mut self,
        _id: ResourceId,
        _min_level: i32,
        _max_level: i32,
    ) {
    }

    pub(super) fn cmd_set_texel_1d_by_resource(
        &mut self,
        _id: ResourceId,
        _x: i32,
        _color: [f32; 4],
    ) {
        // Graceful no-op: texel writes are not implemented yet (see
        // ai/wgpu-modernization-list.md); the producer must not crash.
        warn!("wgpu: texel write not implemented (no-op)");
    }

    fn texel_bytes(format: wgpu::TextureFormat, color: [f32; 4]) -> Option<Vec<u8>> {
        let clamp = |v: f32| v.clamp(0.0, 1.0);
        let u8_component = |v: f32| (clamp(v) * 255.0).round() as u8;
        let u16_component = |v: f32| (clamp(v) * 65535.0).round() as u16;
        let floats = [color[0], color[1], color[2], color[3]];
        let bytes_u8 = || {
            [
                u8_component(color[0]),
                u8_component(color[1]),
                u8_component(color[2]),
                u8_component(color[3]),
            ]
        };
        let bytes_u16 = || {
            let mut bytes = Vec::with_capacity(8);
            for value in [
                u16_component(color[0]),
                u16_component(color[1]),
                u16_component(color[2]),
                u16_component(color[3]),
            ] {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            bytes
        };
        match format {
            wgpu::TextureFormat::R8Unorm => Some(vec![u8_component(color[0])]),
            wgpu::TextureFormat::Rg8Unorm => {
                Some(vec![u8_component(color[0]), u8_component(color[1])])
            }
            wgpu::TextureFormat::Rgba8Unorm
            | wgpu::TextureFormat::Rgba8UnormSrgb
            | wgpu::TextureFormat::Bgra8Unorm
            | wgpu::TextureFormat::Bgra8UnormSrgb => {
                let mut bytes = bytes_u8().to_vec();
                if matches!(
                    format,
                    wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
                ) {
                    bytes.swap(0, 2);
                }
                Some(bytes)
            }
            wgpu::TextureFormat::R16Unorm => Some(u16_component(color[0]).to_le_bytes().to_vec()),
            wgpu::TextureFormat::Rg16Unorm => {
                let mut bytes = Vec::with_capacity(4);
                for value in [u16_component(color[0]), u16_component(color[1])] {
                    bytes.extend_from_slice(&value.to_le_bytes());
                }
                Some(bytes)
            }
            wgpu::TextureFormat::Rgba16Unorm => Some(bytes_u16()),
            wgpu::TextureFormat::R16Float => Some(Self::f32_to_f16_bytes(&floats[0].to_le_bytes())),
            wgpu::TextureFormat::Rg16Float => {
                let mut raw = Vec::with_capacity(8);
                raw.extend_from_slice(&floats[0].to_le_bytes());
                raw.extend_from_slice(&floats[1].to_le_bytes());
                Some(Self::f32_to_f16_bytes(&raw))
            }
            wgpu::TextureFormat::Rgba16Float => Some(Self::f32_to_f16_bytes(
                &floats
                    .iter()
                    .flat_map(|value| value.to_le_bytes())
                    .collect::<Vec<_>>(),
            )),
            wgpu::TextureFormat::R32Float => Some(floats[0].to_le_bytes().to_vec()),
            wgpu::TextureFormat::Rg32Float => {
                let mut bytes = Vec::with_capacity(8);
                bytes.extend_from_slice(&floats[0].to_le_bytes());
                bytes.extend_from_slice(&floats[1].to_le_bytes());
                Some(bytes)
            }
            wgpu::TextureFormat::Rgba32Float => Some(
                floats
                    .iter()
                    .flat_map(|value| value.to_le_bytes())
                    .collect(),
            ),
            _ => None,
        }
    }

    pub(super) fn cmd_set_texel_2d_by_resource(
        &mut self,
        id: ResourceId,
        x: i32,
        y: i32,
        color: [f32; 4],
    ) {
        let Some(queue) = self.queue.clone() else {
            return;
        };
        let Some((texture, format, size)) = (match self.resources.get(&id) {
            Some(WgpuGpuResource::Texture2D { texture, .. }) => {
                Some((texture.clone(), texture.format(), texture.size()))
            }
            Some(_) => {
                warn!("wgpu: texel write for non-2D resource {id:?}");
                None
            }
            None => {
                warn!("wgpu: texel write for unknown resource {id:?}");
                None
            }
        }) else {
            return;
        };
        if x < 0 || y < 0 || x as u32 >= size.width || y as u32 >= size.height {
            warn!("wgpu: texel write out of bounds for {id:?}: ({x}, {y})");
            return;
        }
        let Some(bytes) = Self::texel_bytes(format, color) else {
            warn!("wgpu: texel write unsupported for format {format:?}");
            return;
        };
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: x as u32,
                    y: y as u32,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &bytes,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: None,
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
    }

    pub(super) fn cmd_set_texture_mag_filter_by_resource(
        &mut self,
        id: ResourceId,
        filter: TexFilter,
    ) {
        self.recreate_sampler(GpuHandle(id.0 as u32), Some(filter), None);
    }

    pub(super) fn cmd_set_texture_min_filter_by_resource(
        &mut self,
        id: ResourceId,
        filter: TexFilter,
    ) {
        self.recreate_sampler(GpuHandle(id.0 as u32), None, Some(filter));
    }

    pub(super) fn cmd_set_texture_wrap_mode_by_resource(
        &mut self,
        _id: ResourceId,
        _mode: TexWrapMode,
    ) {
    }

    pub(super) fn cmd_generate_mipmap_by_resource(&mut self, _id: ResourceId) {
        // No mipmap generation yet (see ai/wgpu-modernization-list.md #4);
        // textures are created with mip_level_count 1.
    }

    pub(super) fn cmd_update_texture_1d_data_by_resource(
        &mut self,
        _id: ResourceId,
        _width: i32,
        _internal_format: i32,
        _pixel_format: u32,
        _data_format: u32,
        _data: Vec<u8>,
    ) {
        warn!("wgpu: texture upload variant not implemented (no-op)");
    }

    pub(super) fn cmd_update_texture_3d_data_by_resource(
        &mut self,
        _id: ResourceId,
        _width: i32,
        _height: i32,
        _depth: i32,
        _internal_format: i32,
        _pixel_format: u32,
        _data_format: u32,
        _data: Vec<u8>,
    ) {
        warn!("wgpu: texture upload variant not implemented (no-op)");
    }

    pub(super) fn cmd_update_texture_cube_face_data_by_resource(
        &mut self,
        _id: ResourceId,
        _face: u32,
        _level: i32,
        _size: i32,
        _internal_format: i32,
        _pixel_format: u32,
        _data_format: u32,
        _data: Vec<u8>,
    ) {
        warn!("wgpu: cube-face upload not implemented (no-op)");
    }

    pub(super) fn cmd_copy_texture_2d_from_framebuffer_by_resource(
        &mut self,
        _id: ResourceId,
        _internal_format: i32,
        _width: i32,
        _height: i32,
    ) {
        warn!("wgpu: framebuffer copy not implemented (no-op)");
    }

    // --- Readbacks (all device-bound) ---

    fn read_texture_region(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        origin: wgpu::Origin3d,
        extent: wgpu::Extent3d,
    ) -> Option<Vec<u8>> {
        let bytes_per_pixel = match texture.format() {
            wgpu::TextureFormat::R8Unorm
            | wgpu::TextureFormat::R8Snorm
            | wgpu::TextureFormat::R8Uint
            | wgpu::TextureFormat::R8Sint => 1,
            wgpu::TextureFormat::R32Float
            | wgpu::TextureFormat::R32Uint
            | wgpu::TextureFormat::R32Sint
            | wgpu::TextureFormat::Depth32Float => 4,
            wgpu::TextureFormat::Rgba8Unorm
            | wgpu::TextureFormat::Rgba8UnormSrgb
            | wgpu::TextureFormat::Bgra8Unorm
            | wgpu::TextureFormat::Bgra8UnormSrgb => 4,
            wgpu::TextureFormat::Rgba16Float => 8,
            wgpu::TextureFormat::Rgba32Float => 16,
            _ => return None,
        };
        let row_bytes = extent.width as u64 * bytes_per_pixel;
        let padded_row_bytes = row_bytes.div_ceil(256) * 256;
        let buffer_size = padded_row_bytes
            .checked_mul(extent.height as u64)
            .and_then(|size| size.checked_mul(extent.depth_or_array_layers as u64))?;
        let staging = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("phx-readback"),
            size: buffer_size.max(4),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("phx-readback-copy"),
        });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row_bytes as u32),
                    rows_per_image: Some(extent.height),
                },
            },
            extent,
        );
        queue.submit([encoder.finish()]);

        let completion = Arc::new(Mutex::new(None));
        let callback_completion = completion.clone();
        staging
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                if let Ok(mut state) = callback_completion.lock() {
                    *state = Some(result);
                }
            });

        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Ok(mut state) = completion.lock() {
                if let Some(result) = state.take() {
                    if result.is_err() {
                        staging.unmap();
                        return None;
                    }
                    let Ok(mapped) = staging.slice(..).get_mapped_range() else {
                        staging.unmap();
                        return None;
                    };
                    let mut output = Vec::with_capacity(
                        row_bytes as usize
                            * extent.height as usize
                            * extent.depth_or_array_layers as usize,
                    );
                    for layer in 0..extent.depth_or_array_layers as usize {
                        let layer_start =
                            layer * padded_row_bytes as usize * extent.height as usize;
                        for row in 0..extent.height as usize {
                            let start = layer_start + row * padded_row_bytes as usize;
                            output.extend_from_slice(&mapped[start..start + row_bytes as usize]);
                        }
                    }
                    drop(mapped);
                    staging.unmap();
                    return Some(output);
                }
            }
            if Instant::now() >= deadline || device.poll(wgpu::PollType::Poll).is_err() {
                staging.unmap();
                return None;
            }
            std::thread::yield_now();
        }
    }

    pub(super) fn cmd_read_texture_1d_data(
        &mut self,
        id: ResourceId,
        _pixel_format: u32,
        _data_format: u32,
    ) -> Vec<u8> {
        let Some((device, queue)) = self.device.as_ref().zip(self.queue.as_ref()) else {
            return Vec::new();
        };
        let Some(WgpuGpuResource::Texture1D { texture, .. }) = self.resources.get(&id) else {
            return Vec::new();
        };
        let size = texture.size();
        Self::read_texture_region(
            device,
            queue,
            texture,
            wgpu::Origin3d::ZERO,
            wgpu::Extent3d {
                width: size.width,
                height: 1,
                depth_or_array_layers: 1,
            },
        )
        .unwrap_or_default()
    }

    pub(super) fn cmd_read_texture_2d_data(
        &mut self,
        id: ResourceId,
        _pixel_format: u32,
        _data_format: u32,
    ) -> Vec<u8> {
        let Some((device, queue)) = self.device.as_ref().zip(self.queue.as_ref()) else {
            return Vec::new();
        };
        let Some(WgpuGpuResource::Texture2D { texture, .. }) = self.resources.get(&id) else {
            return Vec::new();
        };
        let size = texture.size();
        Self::read_texture_region(
            device,
            queue,
            texture,
            wgpu::Origin3d::ZERO,
            wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
        )
        .unwrap_or_default()
    }

    pub(super) fn cmd_read_texture_3d_data(
        &mut self,
        id: ResourceId,
        _pixel_format: u32,
        _data_format: u32,
    ) -> Vec<u8> {
        let Some((device, queue)) = self.device.as_ref().zip(self.queue.as_ref()) else {
            return Vec::new();
        };
        let Some(WgpuGpuResource::Texture3D { texture, .. }) = self.resources.get(&id) else {
            return Vec::new();
        };
        let size = texture.size();
        Self::read_texture_region(device, queue, texture, wgpu::Origin3d::ZERO, size)
            .unwrap_or_default()
    }

    pub(super) fn cmd_read_texture_cube_face_data(
        &mut self,
        id: ResourceId,
        face: u32,
        _level: i32,
        _pixel_format: u32,
        _data_format: u32,
    ) -> Vec<u8> {
        let Some((device, queue)) = self.device.as_ref().zip(self.queue.as_ref()) else {
            return Vec::new();
        };
        let Some(WgpuGpuResource::TextureCube { texture, .. }) = self.resources.get(&id) else {
            return Vec::new();
        };
        let size = texture.size();
        Self::read_texture_region(
            device,
            queue,
            texture,
            wgpu::Origin3d {
                x: 0,
                y: 0,
                z: face.min(5),
            },
            wgpu::Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
        )
        .unwrap_or_default()
    }

    pub(super) fn cmd_sample_pixel_2d_by_resource(
        &mut self,
        id: ResourceId,
        x: i32,
        y: i32,
    ) -> [u8; 4] {
        let Some((device, queue)) = self.device.as_ref().zip(self.queue.as_ref()) else {
            return [0; 4];
        };
        let Some(WgpuGpuResource::Texture2D { texture, .. }) = self.resources.get(&id) else {
            return [0; 4];
        };
        let size = texture.size();
        if size.width == 0 || size.height == 0 {
            return [0; 4];
        }
        let px = x.clamp(0, size.width.saturating_sub(1) as i32) as u32;
        let py = y.clamp(0, size.height.saturating_sub(1) as i32) as u32;
        let Some(bytes) = Self::read_texture_region(
            device,
            queue,
            texture,
            wgpu::Origin3d { x: px, y: py, z: 0 },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        ) else {
            return [0; 4];
        };
        Self::decode_sample_pixel(texture.format(), &bytes)
    }

    pub(super) fn cmd_read_framebuffer_pixels(
        &mut self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Vec<u8> {
        let Some((device, queue)) = self.device.as_ref().zip(self.queue.as_ref()) else {
            return Vec::new();
        };
        let target = if let Some(fb) = self.framebuffer_stack.last() {
            fb.color_textures[0]
                .clone()
                .map(|texture| (texture, fb.width, fb.height))
        } else {
            self.surface_frame.as_ref().map(|frame| {
                let size = frame.texture.size();
                (frame.texture.clone(), size.width, size.height)
            })
        };
        let Some((texture, target_width, target_height)) = target else {
            return Vec::new();
        };
        let x = x.clamp(0, target_width.saturating_sub(1) as i32) as u32;
        let y = y.clamp(0, target_height.saturating_sub(1) as i32) as u32;
        let width = width.max(0) as u32;
        let height = height.max(0) as u32;
        if width == 0 || height == 0 || x >= target_width || y >= target_height {
            return Vec::new();
        }
        let extent = wgpu::Extent3d {
            width: width.min(target_width - x),
            height: height.min(target_height - y),
            depth_or_array_layers: 1,
        };
        Self::read_texture_region(
            device,
            queue,
            &texture,
            wgpu::Origin3d { x, y, z: 0 },
            extent,
        )
        .unwrap_or_default()
    }

    // --- Framebuffer operations ---

    pub(super) fn cmd_push_framebuffer(&mut self, id: u64, width: i32, height: i32) {
        // GL allocates a fresh FBO for every RenderTarget.Push. The producer
        // uses the legacy id 0 for this stack operation, so cloning a cached
        // entry would retain color/depth attachments from a prior pass and
        // make a later color-only WGPU pass incompatible with its pipeline.
        let fb = WgpuFramebuffer {
            width: width.max(1) as u32,
            height: height.max(1) as u32,
            ..WgpuFramebuffer::default()
        };
        self.framebuffers.insert(id, fb.clone());
        self.framebuffer_stack.push(fb);
    }

    pub(super) fn cmd_pop_framebuffer(&mut self) {
        if self.framebuffer_stack.pop().is_none() {
            warn!("wgpu: pop_framebuffer with empty stack");
        }
    }

    pub(super) fn cmd_framebuffer_attach_texture_2d(
        &mut self,
        attachment: u32,
        texture: GpuHandle,
        _level: i32,
    ) {
        self.framebuffer_attach_handle(attachment, texture, None);
    }

    pub(super) fn cmd_framebuffer_attach_texture_2d_by_resource(
        &mut self,
        attachment: u32,
        id: ResourceId,
        _level: i32,
    ) {
        self.framebuffer_attach_handle(attachment, GpuHandle(id.0 as u32), None);
    }

    pub(super) fn cmd_framebuffer_attach_texture_3d(
        &mut self,
        attachment: u32,
        texture: GpuHandle,
        _layer: i32,
        _level: i32,
    ) {
        self.framebuffer_attach_handle(attachment, texture, None);
    }

    pub(super) fn cmd_framebuffer_attach_texture_3d_by_resource(
        &mut self,
        attachment: u32,
        id: ResourceId,
        _layer: i32,
        _level: i32,
    ) {
        self.framebuffer_attach_handle(attachment, GpuHandle(id.0 as u32), None);
    }

    pub(super) fn cmd_framebuffer_attach_texture_cube(
        &mut self,
        attachment: u32,
        texture: GpuHandle,
        face: u32,
        _level: i32,
    ) {
        self.framebuffer_attach_handle(attachment, texture, Some(face));
    }

    pub(super) fn cmd_framebuffer_attach_texture_cube_by_resource(
        &mut self,
        attachment: u32,
        id: ResourceId,
        face: u32,
        _level: i32,
    ) {
        self.framebuffer_attach_handle(attachment, GpuHandle(id.0 as u32), Some(face));
    }

    /// Attach a texture's view to the current framebuffer's color/depth slot.
    /// `cube_face` selects a single layer of a cube texture (render targets
    /// must be single-layer D2 views; the whole 6-layer cube view is invalid).
    fn framebuffer_attach_handle(
        &mut self,
        attachment: u32,
        texture: GpuHandle,
        cube_face: Option<u32>,
    ) {
        const GL_COLOR_ATTACHMENT0: u32 = 0x8CE0;
        const GL_DEPTH_ATTACHMENT: u32 = 0x8D00;
        let Some(fb) = self.framebuffer_stack.last_mut() else {
            warn!("wgpu: framebuffer attach without a pushed framebuffer");
            return;
        };
        let Some(resource) = self.resources.get(&ResourceId(texture.0 as u64)) else {
            warn!("wgpu: framebuffer attach for unknown texture {texture:?}");
            return;
        };
        let (attached_texture, view) = match resource {
            WgpuGpuResource::Texture2D { texture, view, .. }
            | WgpuGpuResource::Texture1D { texture, view, .. }
            | WgpuGpuResource::Texture3D { texture, view, .. } => {
                (Some(texture.clone()), view.clone())
            }
            WgpuGpuResource::TextureCube { texture, .. } => {
                // Per-face D2 view for render attachment use.
                let view = texture.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    base_array_layer: cube_face.unwrap_or(0).min(5),
                    array_layer_count: Some(1),
                    ..Default::default()
                });
                (Some(texture.clone()), view)
            }
            _ => {
                warn!("wgpu: framebuffer attach for non-texture {texture:?}");
                return;
            }
        };
        // The ATTACHMENT is authoritative for the framebuffer's size: the
        // producer's PushFramebuffer width/height is nominal, and auto-created
        // depth must match the attached texture or wgpu rejects the pass
        // ("Attachments have differing sizes").
        let (tex_w, tex_h) = match resource {
            WgpuGpuResource::Texture2D { texture, .. }
            | WgpuGpuResource::TextureCube { texture, .. } => {
                let s = texture.size();
                (s.width, s.height)
            }
            _ => (fb.width, fb.height),
        };
        if tex_w > 0 && tex_h > 0 {
            fb.width = tex_w;
            fb.height = tex_h;
        }
        if attachment == GL_DEPTH_ATTACHMENT {
            fb.depth = Some(view);
            fb.depth_texture = attached_texture;
            let format = match resource {
                WgpuGpuResource::Texture2D { texture, .. }
                | WgpuGpuResource::TextureCube { texture, .. } => Some(texture.format()),
                _ => fb.depth_format,
            };
            fb.depth_format = format;
        } else if (GL_COLOR_ATTACHMENT0..=GL_COLOR_ATTACHMENT0 + 3).contains(&attachment) {
            let idx = (attachment - GL_COLOR_ATTACHMENT0) as usize;
            let format = match resource {
                WgpuGpuResource::Texture2D { texture, .. }
                | WgpuGpuResource::TextureCube { texture, .. } => texture.format(),
                _ => fb.color_formats[idx],
            };
            fb.color_formats[idx] = format;
            fb.color[idx] = Some(view);
            fb.color_textures[idx] = attached_texture;
        } else {
            warn!("wgpu: framebuffer attach for unknown attachment {attachment:#x}");
        }
    }

    pub(super) fn cmd_set_draw_buffers(&mut self, _count: i32) {
        // wgpu render passes declare their color targets at pass start.
    }

    pub(super) fn cmd_bind_framebuffer(&mut self, _handle: GpuHandle) {
        // The producer's bind_framebuffer maps ids on its side; the stack is
        // authoritative for wgpu (push/pop carry the target state).
    }

    pub(super) fn cmd_bind_default_framebuffer(&mut self) {
        // The default target IS the swap chain surface in wgpu.
    }

    pub(super) fn cmd_clear(&mut self, color: Option<[f32; 4]>, depth: Option<f32>) {
        if let Some(fb) = self.framebuffer_stack.last_mut() {
            if color.is_some() {
                fb.pending_clear_color = color;
            }
            if depth.is_some() {
                fb.pending_clear_depth = depth;
            }
        } else {
            if color.is_some() {
                self.surface_pending_clear_color = color;
            }
            if depth.is_some() {
                self.surface_pending_clear_depth = depth;
            }
        }
    }

    // --- Mesh operations (bookkeeping real; buffer creation = mesh stage) ---

    pub(super) fn cmd_bind_mesh(&mut self, vao: GpuHandle) {
        self.bound_mesh = Some(vao);
    }

    pub(super) fn cmd_bind_mesh_by_resource(&mut self, id: ResourceId) {
        if !self.resources.contains_key(&id) {
            warn!("wgpu: BindMeshByResource for unknown resource {id:?}");
        }
        self.bound_mesh = Some(GpuHandle(id.0 as u32));
    }

    pub(super) fn cmd_unbind_mesh(&mut self) {
        self.bound_mesh = None;
    }

    // --- Drawing (all device-bound; parity stage) ---

    pub(super) fn cmd_draw_mesh(
        &mut self,
        vao: GpuHandle,
        index_count: i32,
        primitive: CmdPrimitiveType,
    ) {
        self.vertices_drawn_this_frame += index_count.max(0) as u64;
        self.draw_indexed_mesh(vao, index_count, primitive, 1, false);
    }

    pub(super) fn cmd_draw_mesh_instanced(
        &mut self,
        vao: GpuHandle,
        index_count: i32,
        instance_count: i32,
        primitive: CmdPrimitiveType,
    ) {
        self.vertices_drawn_this_frame +=
            (index_count.max(0) as u64) * instance_count.max(0) as u64;
        self.draw_indexed_mesh(vao, index_count, primitive, instance_count.max(0), true);
    }

    pub(super) fn cmd_draw_mesh_by_resource(
        &mut self,
        id: ResourceId,
        index_count: i32,
        primitive: CmdPrimitiveType,
    ) {
        self.vertices_drawn_this_frame += index_count.max(0) as u64;
        self.draw_indexed_mesh(GpuHandle(id.0 as u32), index_count, primitive, 1, false);
    }

    pub(super) fn cmd_draw_mesh_instanced_by_resource(
        &mut self,
        id: ResourceId,
        index_count: i32,
        instance_count: i32,
        primitive: CmdPrimitiveType,
    ) {
        self.vertices_drawn_this_frame +=
            (index_count.max(0) as u64) * instance_count.max(0) as u64;
        self.draw_indexed_mesh(
            GpuHandle(id.0 as u32),
            index_count,
            primitive,
            instance_count.max(0),
            true,
        );
    }

    pub(super) fn cmd_draw_instanced_with_data(
        &mut self,
        _mesh_id: ResourceId,
        index_count: i32,
        instances: Vec<InstanceData>,
        _primitive: CmdPrimitiveType,
    ) {
        if instances.is_empty() {
            return;
        }
        self.instanced_data_items_this_frame += instances.len() as u64;
        self.vertices_drawn_this_frame += (index_count.max(0) as u64) * instances.len() as u64;
        self.draw_instanced_with_data(_mesh_id, index_count, &instances, _primitive);
    }

    fn draw_instanced_with_data(
        &mut self,
        mesh_id: ResourceId,
        index_count: i32,
        instances: &[InstanceData],
        primitive: CmdPrimitiveType,
    ) {
        if index_count <= 0 {
            return;
        }
        let Some(device) = self.device.clone() else {
            return;
        };
        let Some(queue) = self.queue.clone() else {
            return;
        };
        // serialize instances into the per-instance attribute layout
        // (mat4 @ 0, color @ 64, scale @ 80; stride 84) plus a trailing
        // uint32 (instanceIndex @ 84, stride 88): wvp_instanced_tex reads
        // `layout(location=10) in uint instanceIndex;` which the GL path
        // leaves disabled (constant 0) - the zero pad reproduces that.
        let mut bytes = Vec::with_capacity(instances.len() * 88);
        for inst in instances {
            for v in inst.model_matrix {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            for v in inst.color {
                bytes.extend_from_slice(&v.to_le_bytes());
            }
            bytes.extend_from_slice(&inst.scale.to_le_bytes());
            bytes.extend_from_slice(&0u32.to_le_bytes());
        }
        let size = bytes.len() as u64;
        if self.instance_capacity < size {
            let buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("phx-instances"),
                size,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            self.instance_buffer = Some(buf);
            self.instance_capacity = size;
        }
        let Some(buf) = self.instance_buffer.clone() else {
            return;
        };
        queue.write_buffer(&buf, 0, &bytes);
        self.draw_with_instance_buffer(
            mesh_id,
            index_count,
            instances.len() as u32,
            primitive,
            buf,
        );
    }

    /// Draw with an explicitly uploaded per-instance buffer (instanced
    /// pipelines: vertex buffer 0 = mesh, buffer 1 = instance attributes).
    fn draw_with_instance_buffer(
        &mut self,
        mesh_id: ResourceId,
        index_count: i32,
        instance_count: u32,
        primitive: CmdPrimitiveType,
        instance_buffer: wgpu::Buffer,
    ) {
        let (vertex_buffer, index_buffer, index_format, vertex_format) =
            match self.resources.get(&mesh_id) {
                Some(WgpuGpuResource::Mesh {
                    vertex_buffer,
                    index_buffer,
                    index_format,
                    vertex_format,
                    ..
                }) => (
                    vertex_buffer.clone(),
                    index_buffer.clone(),
                    *index_format,
                    vertex_format.clone(),
                ),
                Some(_) => {
                    warn!("wgpu: draw target {mesh_id:?} is not a mesh");
                    return;
                }
                None => {
                    warn!("wgpu: draw for unknown mesh {mesh_id:?}");
                    return;
                }
            };
        let shader_id = self.bound_shader.map(|h| h.0 as u64).unwrap_or(u64::MAX);
        let Some(pipeline) =
            self.get_or_create_pipeline(shader_id, &vertex_format, primitive, true)
        else {
            return;
        };
        let Some(reflection) = self.bound_shader_reflection().cloned() else {
            return;
        };
        let Some(bind_group) = self.get_or_create_bind_group(shader_id, &reflection) else {
            return;
        };
        // upload instance buffer in the same encoder as the pass
        let Some((attachments, depth_view, _depth_format, _w, h, clear_color, clear_depth)) =
            self.current_target_views(true)
        else {
            return;
        };
        let Some(device) = self.device.clone() else {
            return;
        };
        let Some(queue) = self.queue.clone() else {
            return;
        };
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("phx-encoder"),
        });
        {
            let clear = clear_color;
            let mut color_attachments: Vec<Option<wgpu::RenderPassColorAttachment>> = Vec::new();
            for (view, _fmt) in &attachments {
                color_attachments.push(Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match clear {
                            Some(c) => wgpu::LoadOp::Clear(wgpu::Color {
                                r: c[0] as f64,
                                g: c[1] as f64,
                                b: c[2] as f64,
                                a: c[3] as f64,
                            }),
                            None => wgpu::LoadOp::Load,
                        },
                        store: wgpu::StoreOp::Store,
                    },
                }));
            }
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("phx-pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: depth_view.as_ref().map(|dv| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view: dv,
                        depth_ops: Some(wgpu::Operations {
                            load: match clear_depth {
                                Some(d) => wgpu::LoadOp::Clear(d),
                                None => wgpu::LoadOp::Load,
                            },
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }
                }),
                ..Default::default()
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, instance_buffer.slice(..));
            pass.set_index_buffer(index_buffer.slice(..), index_format);
            if let Some((x, y, width, height)) = self.viewport {
                let vp_y = (h as i32 - y - height).max(0) as f32;
                pass.set_viewport(
                    x as f32,
                    vp_y,
                    width.max(0) as f32,
                    height.max(0) as f32,
                    0.0,
                    1.0,
                );
            }
            if self.scissor_enabled {
                if let Some((x, y, width, height)) = self.scissor {
                    let sc_y = (h as i32 - y - height).max(0) as u32;
                    pass.set_scissor_rect(
                        x.max(0) as u32,
                        sc_y,
                        width.max(0) as u32,
                        height.max(0) as u32,
                    );
                }
            }
            pass.draw_indexed(0..index_count.max(0) as u32, 0, 0..instance_count.max(1));
        }
        queue.submit([encoder.finish()]);
    }

    pub(super) fn cmd_draw_instanced_indices(
        &mut self,
        mesh_id: ResourceId,
        index_count: i32,
        indices: Vec<u32>,
        primitive: CmdPrimitiveType,
    ) {
        if indices.is_empty() {
            return;
        }
        self.instanced_data_items_this_frame += indices.len() as u64;
        self.vertices_drawn_this_frame += (index_count.max(0) as u64) * indices.len() as u64;

        // The GL executor binds these IDs as a divisor-1 uint attribute at
        // location 10. Keep the same 88-byte instance layout used by
        // draw_instanced_with_data (mat4/color/scale plus instanceIndex at
        // byte offset 84), then reuse the normal WGPU instanced pass. Uploading
        // the IDs as an index buffer would replace the mesh indices and draw
        // only one instance, which is not equivalent to GL.
        let mut bytes = Vec::with_capacity(indices.len() * 88);
        for index in &indices {
            bytes.extend_from_slice(&[0u8; 84]);
            bytes.extend_from_slice(&index.to_le_bytes());
        }
        let size = bytes.len() as u64;
        let Some(device) = self.device.clone() else {
            return;
        };
        let Some(queue) = self.queue.clone() else {
            return;
        };
        if self.instance_capacity < size {
            let buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("phx-instances"),
                size,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            self.instance_buffer = Some(buf);
            self.instance_capacity = size;
        }
        let Some(instance_buffer) = self.instance_buffer.clone() else {
            return;
        };
        queue.write_buffer(&instance_buffer, 0, &bytes);
        self.draw_with_instance_buffer(
            mesh_id,
            index_count,
            indices.len() as u32,
            primitive,
            instance_buffer,
        );
    }

    pub(super) fn cmd_draw_immediate(
        &mut self,
        primitive: CmdPrimitiveType,
        vertices: &[ImmVertex],
    ) {
        self.immediate_vertices_this_frame += vertices.len() as u64;
        self.vertices_drawn_this_frame += vertices.len() as u64;
        if vertices.is_empty() {
            return;
        }
        let Some(device) = self.device.clone() else {
            return;
        };
        let Some(queue) = self.queue.clone() else {
            return;
        };
        // OpenGL's GL_QUADS consumes four vertices as one primitive, while
        // wgpu has no quad topology. Preserve the GL winding by expanding
        // every complete quad into two triangle-list primitives before upload.
        let draw_vertices: Vec<ImmVertex> = if matches!(primitive, CmdPrimitiveType::Quads) {
            let mut expanded = Vec::with_capacity((vertices.len() / 4) * 6);
            for quad in vertices.chunks_exact(4) {
                expanded.extend_from_slice(&[quad[0], quad[1], quad[2], quad[0], quad[2], quad[3]]);
            }
            expanded
        } else {
            vertices.to_vec()
        };
        if draw_vertices.is_empty() {
            return;
        }
        let mut bytes = Vec::with_capacity(draw_vertices.len() * 52);
        for v in &draw_vertices {
            for f in v.pos {
                bytes.extend_from_slice(&f.to_le_bytes());
            }
            for f in v.normal {
                bytes.extend_from_slice(&f.to_le_bytes());
            }
            for f in v.uv {
                bytes.extend_from_slice(&f.to_le_bytes());
            }
            for f in v.color {
                bytes.extend_from_slice(&f.to_le_bytes());
            }
            bytes.extend_from_slice(&0u32.to_le_bytes());
        }
        let size = bytes.len() as u64;
        if self.immediate_capacity < size {
            let buf = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("phx-immediate"),
                size,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            });
            self.immediate_buffer = Some(buf);
            self.immediate_capacity = size;
        }
        let Some(ibuf) = self.immediate_buffer.clone() else {
            return;
        };
        queue.write_buffer(&ibuf, 0, &bytes);
        // immediate vertex format: pos+normal+uv+color, stride 48 (+4 for the
        // trailing instanceIndex uint -> 52, matching the padded mesh stride)
        let vf = VertexFormat {
            has_position: true,
            has_normal: true,
            has_uv: true,
            has_color: true,
            stride: 52,
        };
        let shader_id = self.bound_shader.map(|h| h.0 as u64).unwrap_or(u64::MAX);
        let Some(pipeline) = self.get_or_create_pipeline(shader_id, &vf, primitive, false) else {
            return;
        };
        let Some(reflection) = self.bound_shader_reflection().cloned() else {
            return;
        };
        let Some(bind_group) = self.get_or_create_bind_group(shader_id, &reflection) else {
            return;
        };
        self.upload_staged_uniforms();
        let Some((attachments, depth_view, _depth_format, _w, h, clear_color, clear_depth)) =
            self.current_target_views(true)
        else {
            return;
        };
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("phx-encoder"),
        });
        {
            let clear = clear_color;
            let mut color_attachments: Vec<Option<wgpu::RenderPassColorAttachment>> = Vec::new();
            for (view, _fmt) in &attachments {
                color_attachments.push(Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match clear {
                            Some(c) => wgpu::LoadOp::Clear(wgpu::Color {
                                r: c[0] as f64,
                                g: c[1] as f64,
                                b: c[2] as f64,
                                a: c[3] as f64,
                            }),
                            None => wgpu::LoadOp::Load,
                        },
                        store: wgpu::StoreOp::Store,
                    },
                }));
            }
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("phx-pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: depth_view.as_ref().map(|dv| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view: dv,
                        depth_ops: Some(wgpu::Operations {
                            load: match clear_depth {
                                Some(d) => wgpu::LoadOp::Clear(d),
                                None => wgpu::LoadOp::Load,
                            },
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }
                }),
                ..Default::default()
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_vertex_buffer(0, ibuf.slice(..));
            if let Some((x, y, width, height)) = self.viewport {
                let vp_y = (h as i32 - y - height).max(0) as f32;
                pass.set_viewport(
                    x as f32,
                    vp_y,
                    width.max(0) as f32,
                    height.max(0) as f32,
                    0.0,
                    1.0,
                );
            }
            // Immediate draws share the GL ClipRect state with indexed draws;
            // applying the scissor to this pass preserves clipping for UI
            // quads instead of silently rendering them across the whole target.
            if self.scissor_enabled {
                if let Some((x, y, width, height)) = self.scissor {
                    let sc_y = (h as i32 - y - height).max(0) as u32;
                    pass.set_scissor_rect(
                        x.max(0) as u32,
                        sc_y,
                        width.max(0) as u32,
                        height.max(0) as u32,
                    );
                }
            }
            pass.draw(0..draw_vertices.len() as u32, 0..1);
        }
        queue.submit([encoder.finish()]);
    }

    /// Shared non-instanced draw entry: resolves the mesh, pipeline, bind
    /// group and issues the indexed draw.
    fn draw_indexed_mesh(
        &mut self,
        vao: GpuHandle,
        index_count: i32,
        primitive: CmdPrimitiveType,
        instance_count: i32,
        instanced: bool,
    ) {
        if index_count <= 0 {
            return;
        }
        let Some(mesh) = self
            .resources
            .get(&ResourceId(vao.0 as u64))
            .and_then(|r| match r {
                WgpuGpuResource::Mesh {
                    vertex_buffer,
                    index_buffer,
                    index_format,
                    vertex_format,
                    index_count,
                    ..
                } => Some((
                    vertex_buffer.clone(),
                    index_buffer.clone(),
                    *index_format,
                    vertex_format.clone(),
                    *index_count,
                )),
                _ => None,
            })
        else {
            warn!("wgpu: draw for unknown mesh handle {vao:?}");
            return;
        };
        let shader_id = self.bound_shader.map(|h| h.0 as u64).unwrap_or(u64::MAX);
        let Some(pipeline) = self.get_or_create_pipeline(shader_id, &mesh.3, primitive, instanced)
        else {
            return;
        };
        let Some(reflection) = self.bound_shader_reflection().cloned() else {
            return;
        };
        let Some(bind_group) = self.get_or_create_bind_group(shader_id, &reflection) else {
            return;
        };
        // Clamp to the buffer's real index count: GL tolerates out-of-bounds
        // index reads (undefined pixels), wgpu rejects them.
        let count = (index_count.max(0) as u32).min(mesh.4);
        self.draw_indexed(
            (mesh.0, mesh.1, mesh.2),
            &pipeline,
            &bind_group,
            count,
            instance_count.max(1) as u32,
        );
    }

    // --- Resource creation (all device-bound) ---

    pub(super) fn cmd_create_shader(
        &mut self,
        id: ResourceId,
        vertex_src: String,
        fragment_src: String,
    ) -> Option<String> {
        let error = match self.compile_shader_pair(&vertex_src, &fragment_src) {
            Ok((vertex_module, fragment_module, adapted_vs, adapted_fs, reflection)) => {
                let _ = (&adapted_vs, &adapted_fs);
                self.resources.insert(
                    id,
                    WgpuGpuResource::Shader {
                        vertex_module,
                        fragment_module,
                        vertex_src: Arc::from(vertex_src.as_str()),
                        fragment_src: Arc::from(fragment_src.as_str()),
                        reflection,
                    },
                );
                None
            }
            Err(e) => {
                tracing::error!("wgpu: failed to create shader {id:?}: {e}");
                Some(e)
            }
        };
        error
    }

    /// Highest injected binding (>= 10) in an adapted source, or 9 when the
    /// stage injects nothing (fragment offset base).
    fn max_injected_binding(adapted: &str) -> u32 {
        let mut max = 9u32;
        for line in adapted.lines() {
            if let Some(rest) = line.trim().strip_prefix("layout(") {
                if let Some(b) = rest
                    .split(',')
                    .find_map(|p| p.trim().strip_prefix("binding="))
                    // the value runs "N) uniform ..." — cut at the first ')'
                    .and_then(|v| v.split(')').next().unwrap_or("").trim().parse::<u32>().ok())
                {
                    max = max.max(b);
                }
            }
        }
        max
    }

    /// Fragment-stage compile with the injected-binding offset applied.
    fn compile_glsl_stage_adapted_fs(
        &self,
        code: &str,
        vs_max_binding: u32,
    ) -> Result<(wgpu::ShaderModule, String), String> {
        let stage = wgpu::naga::ShaderStage::Fragment;
        let adapted = Self::adapt_glsl_for_naga_with_offset(code, vs_max_binding.saturating_sub(9));
        let mut frontend = wgpu::naga::front::glsl::Frontend::default();
        let parse_result = frontend.parse(&wgpu::naga::front::glsl::Options::from(stage), &adapted);
        let module = match parse_result {
            Ok(m) => m,
            Err(errors) => {
                let mut detail = String::new();
                for e in errors.errors.iter().take(4) {
                    let line_no = e
                        .location(&adapted)
                        .map(|l| l.line_number as usize)
                        .unwrap_or(0);
                    let line = adapted
                        .lines()
                        .nth(line_no.saturating_sub(1))
                        .unwrap_or("<eof>")
                        .trim();
                    detail.push_str(&format!(" (line {line_no}: {line})"));
                }
                return Err(format!(
                    "naga GLSL {stage:?} parse failed: {errors}{detail}"
                ));
            }
        };
        // Same validation-at-creation rationale as the vertex path: the GL
        // backend reports shader failures synchronously and the engine
        // tolerates them; wgpu's deferred validation would panic.
        let mut validator = wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::all(),
        );
        if let Err(e) = validator.validate(&module) {
            return Err(format!("naga GLSL {stage:?} validation failed: {e}"));
        }
        let Some(device) = self.device.clone() else {
            return Err("wgpu device not attached (shader stage)".to_string());
        };
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("phx-glsl-fragment"),
            source: wgpu::ShaderSource::Glsl {
                shader: std::borrow::Cow::Borrowed(&adapted),
                stage,
                defines: &[],
            },
        });
        Ok((module, adapted))
    }

    /// Compile both stages of a preprocessed shader pair via naga; returns
    /// the modules plus the ADAPTED sources (reflection input).
    fn compile_shader_pair(
        &self,
        vertex_src: &str,
        fragment_src: &str,
    ) -> Result<
        (
            wgpu::ShaderModule,
            wgpu::ShaderModule,
            String,
            String,
            ShaderReflection,
        ),
        String,
    > {
        let (vs, adapted_vs) =
            self.compile_glsl_stage_adapted(wgpu::naga::ShaderStage::Vertex, vertex_src)?;
        // Fragment bindings must not collide with the vertex stage's
        // injected bindings (both start at 10 otherwise).
        let vs_max = Self::max_injected_binding(&adapted_vs);
        let (fs, adapted_fs) = self.compile_glsl_stage_adapted_fs(fragment_src, vs_max)?;
        let reflection = Self::build_reflection(&adapted_vs, &adapted_fs);
        Ok((vs, fs, adapted_vs, adapted_fs, reflection))
    }

    /// Reflect the plain uniforms (name -> binding/offset/size) and samplers
    /// (name -> texture/sampler bindings) from the ADAPTED source pair.
    fn build_reflection(vertex_src: &str, fragment_src: &str) -> ShaderReflection {
        let mut refl = ShaderReflection::default();
        let mut tex_bindings: Vec<(String, u32, wgpu::TextureViewDimension)> = Vec::new();
        let mut samp_bindings: Vec<(String, u32)> = Vec::new();
        // Parse each stage with ITS OWN stage options: parsing the fragment
        // source with the vertex options changes how the frontend resolves
        // the fragment's sampler globals (probe-verified: the fragment's
        // samplers vanish from the module).
        for (source, stage) in [
            (vertex_src, wgpu::naga::ShaderStage::Vertex),
            (fragment_src, wgpu::naga::ShaderStage::Fragment),
        ] {
            let mut frontend = wgpu::naga::front::glsl::Frontend::default();
            let Ok(module) = frontend.parse(&wgpu::naga::front::glsl::Options::from(stage), source)
            else {
                continue;
            };
            for (_, var) in module.global_variables.iter() {
                let Some(wgpu::naga::ResourceBinding { group: 0, binding }) = var.binding else {
                    continue;
                };
                match var.space {
                    wgpu::naga::AddressSpace::Uniform => {
                        // The packed plain-uniform block (and any other block
                        // at binding >= 10): reflect every member with its
                        // std140 offset. Member size = gap to the next member
                        // (last member = span - offset).
                        if binding >= 10 {
                            if let wgpu::naga::TypeInner::Struct { members, span } =
                                &module.types[var.ty].inner
                            {
                                for (i, m) in members.iter().enumerate() {
                                    if let Some(name) = &m.name {
                                        let next_offset =
                                            members.get(i + 1).map(|n| n.offset).unwrap_or(*span);
                                        refl.uniforms.push(UniformRefl {
                                            name: name.clone().into(),
                                            binding,
                                            offset: m.offset,
                                            size: next_offset.saturating_sub(m.offset),
                                            block_size: *span,
                                        });
                                    }
                                }
                                refl.max_binding = refl.max_binding.max(binding);
                            }
                        }
                    }
                    wgpu::naga::AddressSpace::Handle => match &module.types[var.ty].inner {
                        wgpu::naga::TypeInner::Image { dim, .. } => {
                            if let Some(name) = &var.name {
                                let view_dimension = match dim {
                                    wgpu::naga::ImageDimension::D1 => {
                                        wgpu::TextureViewDimension::D1
                                    }
                                    wgpu::naga::ImageDimension::D2 => {
                                        wgpu::TextureViewDimension::D2
                                    }
                                    wgpu::naga::ImageDimension::D3 => {
                                        wgpu::TextureViewDimension::D3
                                    }
                                    wgpu::naga::ImageDimension::Cube => {
                                        wgpu::TextureViewDimension::Cube
                                    }
                                };
                                tex_bindings.push((name.to_string(), binding, view_dimension));
                            }
                        }
                        wgpu::naga::TypeInner::Sampler { .. } => {
                            if let Some(name) = &var.name {
                                samp_bindings.push((name.to_string(), binding));
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
        // Pair texture+sampler globals by CONSECUTIVE BINDING: the
        // adaptation emits each pair as tex@N, samp@N+1 within a stage. Name
        // matching would be ambiguous across stages — the vertex and fragment
        // stages can declare the SAME sampler name (e.g. envMap) at DIFFERENT
        // bindings (the fragment stage is offset past the vertex stage's
        // injected bindings).
        let mut tex_bindings_sorted = tex_bindings.clone();
        tex_bindings_sorted.sort_by_key(|(_, b, _)| *b);
        tex_bindings_sorted.dedup_by_key(|(_, b, _)| *b);
        for (tex_name, tex_binding, view_dimension) in &tex_bindings_sorted {
            let base = tex_name
                .strip_suffix("_tex")
                .unwrap_or(tex_name)
                .to_string();
            let samp_binding = tex_binding + 1;
            if samp_bindings.iter().any(|(_, b)| *b == samp_binding) {
                refl.samplers.push(SamplerRefl {
                    name: Arc::from(base.as_str()),
                    tex_binding: *tex_binding,
                    samp_binding,
                    view_dimension: *view_dimension,
                });
                refl.max_binding = refl.max_binding.max(*tex_binding).max(samp_binding);
            }
        }
        refl
    }

    pub(super) fn cmd_get_uniform_location_by_resource(
        &mut self,
        id: ResourceId,
        name: Arc<str>,
    ) -> i32 {
        // Serve the engine's location protocol from the shader reflection:
        // plain uniforms get 0..n, samplers get 1000+ (disjoint spaces).
        let loc = self
            .shader_reflection_for_resource(id)
            .and_then(|refl| {
                refl.uniforms
                    .iter()
                    .position(|u| u.name == name)
                    .map(|i| i as i32)
                    .or_else(|| {
                        refl.samplers
                            .iter()
                            .position(|s| s.name == name)
                            .map(|i| refl.sampler_location(i))
                    })
            })
            .unwrap_or(-1);
        loc
    }

    /// Reflection of a shader resource (or its hot-reloaded pair).
    fn shader_reflection_for_resource(&self, id: ResourceId) -> Option<&ShaderReflection> {
        match self.resources.get(&id) {
            Some(WgpuGpuResource::Shader { reflection, .. }) => Some(reflection),
            _ => None,
        }
    }

    /// Reflection of the CURRENTLY bound shader (hot pair wins).
    fn bound_shader_reflection(&self) -> Option<&ShaderReflection> {
        if let Some(key) = &self.bound_hot_shader {
            if let Some((_, _, reflection)) = self.hot_reloaded_shaders.get(key) {
                return Some(reflection);
            }
        }
        let handle = self.bound_shader?;
        self.shader_reflection_for_resource(ResourceId(handle.0 as u64))
    }

    /// Write a plain-uniform value into the per-binding staging buffer, or
    /// record a sampler slot assignment for sampler locations (1000+).
    fn apply_uniform(&mut self, location: i32, bytes: &[u8]) {
        let Some(refl) = self.bound_shader_reflection().cloned() else {
            return;
        };
        if (0..refl.uniforms.len() as i32).contains(&location) {
            let u = &refl.uniforms[location as usize];
            self.ensure_plain_staging(u.binding, u.block_size);
            self.stage_plain_uniform(u.binding, u.offset, u.size, bytes);
        } else if location >= 1000 {
            let idx = (location - 1000) as usize;
            if let Some(s) = refl.samplers.get(idx) {
                if bytes.len() == 4 {
                    let slot = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                    self.sampler_slots
                        .insert(s.name.clone(), slot.max(0) as u32);
                }
            }
        }
        // Unknown location: GL ignores writes to missing uniforms too.
    }

    /// By-name variant: resolve the name in the bound shader's reflection.
    fn apply_uniform_by_name(&mut self, name: &str, bytes: &[u8]) {
        let Some(refl) = self.bound_shader_reflection().cloned() else {
            return;
        };
        if let Some(u) = refl.uniforms.iter().find(|u| u.name.as_ref() == name) {
            self.ensure_plain_staging(u.binding, u.block_size);
            self.stage_plain_uniform(u.binding, u.offset, u.size, bytes);
        } else if let Some(s) = refl.samplers.iter().find(|s| s.name.as_ref() == name) {
            if bytes.len() == 4 {
                let slot = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                self.sampler_slots
                    .insert(s.name.clone(), slot.max(0) as u32);
            }
        }
    }

    /// Grow the per-binding plain-uniform staging to the block's full span
    /// (it starts at 64 bytes; larger blocks are resized on first use).
    fn ensure_plain_staging(&mut self, binding: u32, block_size: u32) {
        let staging = self
            .plain_uniform_staging
            .entry(binding)
            .or_insert_with(|| vec![0u8; block_size.max(64) as usize]);
        if staging.len() < block_size as usize {
            staging.resize(block_size as usize, 0);
        }
    }

    fn stage_plain_uniform(&mut self, binding: u32, offset: u32, _size: u32, bytes: &[u8]) {
        // The staging buffer covers the whole shared block (`block_size`);
        // the member length is already represented by `bytes`, while the
        // caller-provided layout size remains part of the reflection contract.
        let staging = self
            .plain_uniform_staging
            .entry(binding)
            .or_insert_with(|| vec![0u8; 64]);
        let start = offset as usize;
        let end = start + bytes.len();
        if end <= staging.len() {
            staging[start..end].copy_from_slice(bytes);
            self.plain_uniform_dirty.insert(binding);
        }
    }

    // =====================================================================
    // Draw path
    // =====================================================================

    fn ensure_ubo_buffer(&mut self, binding: u32, size: u64) -> Option<wgpu::Buffer> {
        if let Some(buf) = self.ubo_buffers.get(&binding) {
            return Some(buf.clone());
        }
        let Some(device) = self.device.clone() else {
            return None;
        };
        // Round UP to 256: the naga struct span can under-report the block's
        // true size (std140 trailing padding; probe-verified: a 160-byte
        // buffer rejected a shader block expecting 192).
        let size = size.max(16).div_ceil(256) * 256;
        let buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("phx-ubo"),
            size,
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        self.ubo_buffers.insert(binding, buf.clone());
        Some(buf)
    }

    fn ensure_default_texture(&mut self) -> Option<(wgpu::Texture, wgpu::Sampler)> {
        if let (Some(t), Some(s)) = (self.default_texture.as_ref(), self.default_sampler.as_ref()) {
            return Some((t.clone(), s.clone()));
        }
        let Some(device) = self.device.clone() else {
            return None;
        };
        let tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("phx-default-tex"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.as_ref().map(|q| {
            q.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &[255u8, 255, 255, 255],
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4),
                    rows_per_image: Some(1),
                },
                wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            )
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("phx-default-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });
        self.default_texture = Some(tex.clone());
        self.default_sampler = Some(sampler.clone());
        let cube = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("phx-default-cube"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        // Fill ALL 6 faces with white - an unfilled default samples BLACK on
        // Vulkan, which was the black-screen culprit for every unbound-cube
        // fallback (irMap in the lighting/composite passes).
        let Some(queue) = self.queue.clone() else {
            return None;
        };
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &cube,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &[
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
                0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            ],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 6,
            },
        );
        self.default_cube_texture = Some(cube);
        let tex1d = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("phx-default-1d"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D1,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &tex1d,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[0xFF, 0xFF, 0xFF, 0xFF],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        self.default_1d_texture = Some(tex1d);
        let tex3d = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("phx-default-3d"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &tex3d,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[0xFF, 0xFF, 0xFF, 0xFF],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        self.default_3d_texture = Some(tex3d);
        Some((tex, sampler))
    }

    /// Current render target views (cloned, so the pass can run while self is
    /// mutably borrowed for cache lookups). Returns (color attachments with
    /// formats, depth, w, h, pending clear color, pending clear depth).
    /// Acquires the surface frame when the framebuffer stack is empty.
    fn current_target_views(
        &mut self,
        consume_clear: bool,
    ) -> Option<(
        Vec<(wgpu::TextureView, wgpu::TextureFormat)>,
        Option<wgpu::TextureView>,
        Option<wgpu::TextureFormat>,
        u32,
        u32,
        Option<[f32; 4]>,
        Option<f32>,
    )> {
        let Some(device) = self.device.clone() else {
            return None;
        };
        if let Some(fb) = self.framebuffer_stack.last_mut() {
            let w = fb.width.max(1);
            let h = fb.height.max(1);
            let color = match fb.color[0].clone() {
                Some(v) => v,
                None => {
                    let tex = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("phx-fbo-color"),
                        size: wgpu::Extent3d {
                            width: w,
                            height: h,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: fb.color_formats[0],
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::TEXTURE_BINDING
                            | wgpu::TextureUsages::COPY_SRC,
                        view_formats: &[],
                    });
                    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
                    fb.color_textures[0] = Some(tex);
                    fb.color[0] = Some(view.clone());
                    view
                }
            };
            // A color-only target must remain color-only. Creating an implicit
            // Depth24Plus view here makes the render pass incompatible with a
            // pipeline whose depth state is intentionally omitted. Depth is
            // created only after an explicit depth attachment sets
            // `fb.depth_format`.
            let depth = match fb.depth.clone() {
                Some(v) => Some(v),
                None => fb.depth_format.map(|depth_format| {
                    let tex = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("phx-fbo-depth"),
                        size: wgpu::Extent3d {
                            width: w,
                            height: h,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: depth_format,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                        view_formats: &[],
                    });
                    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
                    fb.depth_texture = Some(tex);
                    fb.depth = Some(view.clone());
                    view
                }),
            };
            let clear_color = if consume_clear {
                fb.pending_clear_color.take()
            } else {
                fb.pending_clear_color
            };
            let clear_depth = if consume_clear {
                fb.pending_clear_depth.take()
            } else {
                fb.pending_clear_depth
            };
            let mut attachments: Vec<(wgpu::TextureView, wgpu::TextureFormat)> = Vec::new();
            for i in 0..4 {
                if let Some(view) = fb.color[i].clone() {
                    attachments.push((view, fb.color_formats[i]));
                }
            }
            if attachments.is_empty() {
                attachments.push((color, fb.color_formats[0]));
            }
            let depth_format = fb.depth_format;
            return Some((
                attachments,
                depth,
                depth_format,
                w,
                h,
                clear_color,
                clear_depth,
            ));
        }
        // Surface target
        let (w, h) = self.surface_size;
        if self.surface_frame.is_none() {
            let surface = self.surface.as_ref()?;
            match surface.get_current_texture() {
                wgpu::CurrentSurfaceTexture::Success(frame)
                | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                    self.surface_frame = Some(frame);
                }
                wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                    self.reconfigure_surface();
                    return None;
                }
                wgpu::CurrentSurfaceTexture::Timeout
                | wgpu::CurrentSurfaceTexture::Occluded
                | wgpu::CurrentSurfaceTexture::Validation => {
                    return None;
                }
            }
        }
        let frame = self.surface_frame.as_ref()?;
        let color = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let depth = self.ensure_surface_depth();
        let clear_color = if consume_clear {
            self.surface_pending_clear_color.take()
        } else {
            self.surface_pending_clear_color
        };
        let clear_depth = if consume_clear {
            self.surface_pending_clear_depth.take()
        } else {
            self.surface_pending_clear_depth
        };
        let color_format = self
            .surface_config
            .as_ref()
            .map(|c| c.format)
            .unwrap_or(wgpu::TextureFormat::Bgra8UnormSrgb);
        Some((
            vec![(color, color_format)],
            depth,
            Some(wgpu::TextureFormat::Depth24Plus),
            w,
            h,
            clear_color,
            clear_depth,
        ))
    }

    fn ensure_surface_depth(&mut self) -> Option<wgpu::TextureView> {
        let Some(device) = self.device.clone() else {
            return None;
        };
        let (w, h) = self.surface_size;
        let need_recreate = match &self.surface_depth {
            Some((_tex, tw, th)) => *tw != w || *th != h,
            None => true,
        };
        if need_recreate {
            let tex = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("phx-surface-depth"),
                size: wgpu::Extent3d {
                    width: w.max(1),
                    height: h.max(1),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24Plus,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
            self.surface_depth = Some((tex, w, h));
            return Some(view);
        }
        self.surface_depth
            .as_ref()
            .map(|(tex, _, _)| tex.create_view(&wgpu::TextureViewDescriptor::default()))
    }

    fn reconfigure_surface(&mut self) {
        let Some(surface) = self.surface.as_ref() else {
            return;
        };
        let Some(device) = self.device.clone() else {
            return;
        };
        let (w, h) = self.surface_size;
        if let Some(cfg) = &mut self.surface_config {
            cfg.width = w.max(1);
            cfg.height = h.max(1);
            surface.configure(&device, cfg);
        }
    }

    fn create_bind_group_layout(
        refl: &ShaderReflection,
        device: &wgpu::Device,
    ) -> wgpu::BindGroupLayout {
        let mut entries = Vec::new();
        for binding in 0..=2u32 {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
        }
        // One entry per DISTINCT plain binding (all members share the block).
        let mut seen: Vec<u32> = Vec::new();
        for u in &refl.uniforms {
            if seen.contains(&u.binding) {
                continue;
            }
            seen.push(u.binding);
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: u.binding,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
        }
        for s in &refl.samplers {
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: s.tex_binding,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: s.view_dimension,
                    multisampled: false,
                },
                count: None,
            });
            entries.push(wgpu::BindGroupLayoutEntry {
                binding: s.samp_binding,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }
        entries.sort_by_key(|e| e.binding);
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("phx-bgl"),
            entries: &entries,
        })
    }

    /// Pipeline for (bound shader, mesh vertex format, current state).
    /// `instanced` adds the per-instance attribute buffer (locations 4-9).
    fn get_or_create_pipeline(
        &mut self,
        shader_id: u64,
        vertex_format: &VertexFormat,
        primitive: CmdPrimitiveType,
        instanced: bool,
    ) -> Option<wgpu::RenderPipeline> {
        let (color_formats, color_attachment_count, depth_format) =
            match self.current_target_views(false) {
                Some((attachments, _, depth_format, _, _, _, _)) => {
                    let mut formats = [wgpu::TextureFormat::Rgba8Unorm; 4];
                    for (i, (_, f)) in attachments.iter().take(4).enumerate() {
                        formats[i] = *f;
                    }
                    (formats, attachments.len().min(4) as u8, depth_format)
                }
                None => return None,
            };
        let key = PipelineKey {
            shader_id,
            vertex_format: (
                vertex_format.has_position,
                vertex_format.has_normal,
                vertex_format.has_uv,
                vertex_format.has_color,
                vertex_format.stride,
            ),
            blend: self.blend_mode,
            cull: self.cull_face,
            primitive: primitive as u8,
            depth_test: self.depth_test,
            depth_writable: self.depth_writable,
            wireframe: self.wireframe,
            instanced,
            color_formats,
            color_attachment_count,
            depth_format,
        };
        if let Some(p) = self.pipeline_cache.get(&key) {
            return Some(p.clone());
        }
        let Some(device) = self.device.clone() else {
            return None;
        };
        let queue = self.queue.as_ref()?;
        let reflection = self.bound_shader_reflection()?.clone();
        let (vs, fs) = if let Some(hot_key) = &self.bound_hot_shader {
            let (vs, fs, _) = self.hot_reloaded_shaders.get(hot_key)?;
            (vs.clone(), fs.clone())
        } else {
            let handle = self.bound_shader?;
            match self.resources.get(&ResourceId(handle.0 as u64)) {
                Some(WgpuGpuResource::Shader {
                    vertex_module,
                    fragment_module,
                    ..
                }) => (vertex_module.clone(), fragment_module.clone()),
                _ => return None,
            }
        };

        let mut attributes = Vec::new();
        let mut offset = 0u32;
        let mut shader_location = 0u32;
        if vertex_format.has_position {
            attributes.push(wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: offset as u64,
                shader_location,
            });
            offset += 12;
            shader_location += 1;
        }
        if vertex_format.has_normal {
            attributes.push(wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: offset as u64,
                shader_location,
            });
            offset += 12;
            shader_location += 1;
        }
        if vertex_format.has_uv {
            attributes.push(wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: offset as u64,
                shader_location,
            });
            offset += 8;
            shader_location += 1;
        }
        // ALWAYS emit the color attribute: the vertex include declares
        // `in vec4 vertex_color;` at location 3 unconditionally, and wgpu
        // rejects a pipeline whose vertex inputs lack a buffer attribute
        // (GL tolerates the disabled array -> constant (0,0,0,0)). For
        // meshes without color data the vertex buffer is zero-padded to
        // carry the attribute (see cmd_create_mesh).
        attributes.push(wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: offset as u64,
            shader_location,
        });
        // wvp_instanced_tex reads `layout(location=10) in uint instanceIndex;`
        // on non-instanced draws (the GL path leaves it disabled -> constant
        // 0). For an instanced pipeline location 10 belongs to the divisor-1
        // buffer below; advertising the mesh's zero-padded fallback there as
        // well makes wgpu reject the pipeline for duplicate locations.
        if !instanced {
            attributes.push(wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint32,
                offset: (offset + 16) as u64,
                shader_location: 10,
            });
        }
        let mut buffers: Vec<Option<wgpu::VertexBufferLayout>> =
            vec![Some(wgpu::VertexBufferLayout {
                array_stride: vertex_format.stride as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &attributes,
            })];
        let instance_attributes = [
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 4,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 16,
                shader_location: 5,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 32,
                shader_location: 6,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 48,
                shader_location: 7,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 64,
                shader_location: 8,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32,
                offset: 80,
                shader_location: 9,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint32,
                offset: 84,
                shader_location: 10,
            },
        ];
        if instanced {
            buffers.push(Some(wgpu::VertexBufferLayout {
                array_stride: 88,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &instance_attributes,
            }));
        }

        let bgl = Self::create_bind_group_layout(&reflection, &device);
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("phx-pl"),
            bind_group_layouts: &[Some(&bgl)],
            immediate_size: 0,
        });
        let blend = Self::blend_mode_to_wgpu(self.blend_mode);
        let (topology, strip_index_format) = Self::primitive_topology_with_strip(primitive);
        let cull_mode = Self::cull_face_to_wgpu(self.cull_face);
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("phx-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &vs,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                buffers: &buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: color_formats[0],
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: color_formats[1],
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: color_formats[2],
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: color_formats[3],
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ][..color_attachment_count.max(1) as usize],
            }),
            primitive: wgpu::PrimitiveState {
                topology,
                strip_index_format,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode,
                unclipped_depth: false,
                polygon_mode: if self.wireframe {
                    wgpu::PolygonMode::Line
                } else {
                    wgpu::PolygonMode::Fill
                },
                conservative: false,
            },
            depth_stencil: if self.depth_test || self.depth_writable {
                Some(wgpu::DepthStencilState {
                    format: depth_format.unwrap_or(wgpu::TextureFormat::Depth24Plus),
                    depth_write_enabled: Some(self.depth_writable),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                })
            } else {
                None
            },
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        self.pipeline_cache.insert(key, pipeline.clone());
        let _ = queue; // keep queue borrow symmetrical (uploads happen at draw)
        Some(pipeline)
    }

    /// Bind group for (bound shader, current sampler slots + UBO staging).
    fn get_or_create_bind_group(
        &mut self,
        shader_id: u64,
        reflection: &ShaderReflection,
    ) -> Option<wgpu::BindGroup> {
        let Some(device) = self.device.clone() else {
            return None;
        };
        // cache key: shader + the slot assignment AND the bound texture
        // handle for every sampler. The engine's texture-unit counter is
        // PER-SHADER (each shader starts at unit 1), so different shaders
        // reuse the same slots and overwrite each other's bound textures;
        // the bind group must therefore track the actual texture handles,
        // not just the slot numbers (probe-verified: nebula LUT slots 1-3
        // held the composite's 2D textures at first creation -> D1/D2
        // validation error).
        let mut slot_hash: u64 = 0;
        for s in &reflection.samplers {
            let slot = self.sampler_slots.get(&s.name).copied().unwrap_or(u32::MAX);
            let handle = self
                .bound_textures
                .get(slot as usize)
                .and_then(|h| h.as_ref())
                .map(|h| h.0 as u64)
                .unwrap_or(u64::MAX);
            slot_hash = slot_hash
                .wrapping_mul(31)
                .wrapping_add(slot as u64)
                .wrapping_mul(31)
                .wrapping_add(handle);
        }
        let cache_key = (shader_id, slot_hash);
        if let Some(bg) = self.bind_group_cache.get(&cache_key) {
            return Some(bg.clone());
        }

        // Owned holders so the BindingResource references live as long as
        // the entries (wgpu 30's BufferBinding borrows the Buffer).
        let mut buffers: Vec<(u32, wgpu::Buffer)> = Vec::new();
        for binding in 0..=2u32 {
            let size = match binding {
                0 => 288u64,
                1 => 32,
                // LightUBO: size to the engine's actual payload (192 bytes;
                // the old hardcoded 32 overran on write_buffer).
                _ => self
                    .light_ubo
                    .as_ref()
                    .map(|b| b.len() as u64)
                    .unwrap_or(192),
            };
            let Some(buf) = self.ensure_ubo_buffer(binding, size) else {
                return None;
            };
            buffers.push((binding, buf));
        }
        let mut seen: Vec<u32> = Vec::new();
        for u in &reflection.uniforms {
            if seen.contains(&u.binding) {
                continue;
            }
            seen.push(u.binding);
            let Some(buf) = self.ensure_ubo_buffer(u.binding, u.block_size as u64) else {
                return None;
            };
            buffers.push((u.binding, buf));
        }
        let (default_tex, default_sampler) = self.ensure_default_texture()?;
        let default_view = default_tex.create_view(&wgpu::TextureViewDescriptor::default());
        // Cube-dimension bindings need a Cube fallback view (a D2 view fails
        // layout validation for a Cube binding).
        let default_cube_view = self.default_cube_texture.as_ref().map(|t| {
            t.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::Cube),
                base_array_layer: 0,
                array_layer_count: Some(6),
                ..Default::default()
            })
        });
        let default_1d_view = self
            .default_1d_texture
            .as_ref()
            .map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));
        let default_3d_view = self
            .default_3d_texture
            .as_ref()
            .map(|t| t.create_view(&wgpu::TextureViewDescriptor::default()));
        let mut views: Vec<(u32, wgpu::TextureView)> = Vec::new();
        let mut sampler_entries: Vec<(u32, wgpu::Sampler)> = Vec::new();
        for s in &reflection.samplers {
            let slot = self.sampler_slots.get(&s.name).copied().unwrap_or(u32::MAX);
            match self.bound_textures.get(slot as usize).and_then(|h| {
                h.as_ref().and_then(|handle| {
                    match self.resources.get(&ResourceId(handle.0 as u64)) {
                        Some(WgpuGpuResource::Texture2D { view, sampler, .. }) => {
                            // The engine's texture-unit counters are per-shader,
                            // so another shader's 2D texture can sit in a slot
                            // the current shader binds a Cube/1D/3D sampler to.
                            // GL tolerates the wrong texture (garbage sample);
                            // wgpu rejects a mismatched view dimension, so fall
                            // back to the per-dimension default instead.
                            if s.view_dimension == wgpu::TextureViewDimension::D2 {
                                Some((view.clone(), sampler.clone()))
                            } else {
                                None
                            }
                        }
                        Some(WgpuGpuResource::TextureCube { view, sampler, .. }) => {
                            if s.view_dimension == wgpu::TextureViewDimension::Cube {
                                Some((view.clone(), sampler.clone()))
                            } else {
                                None
                            }
                        }
                        Some(WgpuGpuResource::Texture1D { view, sampler, .. }) => {
                            if s.view_dimension == wgpu::TextureViewDimension::D1 {
                                Some((view.clone(), sampler.clone()))
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                })
            }) {
                Some((view, sampler)) => {
                    views.push((s.tex_binding, view));
                    sampler_entries.push((s.samp_binding, sampler));
                }
                None => {
                    let fallback = match s.view_dimension {
                        wgpu::TextureViewDimension::Cube => default_cube_view.clone(),
                        wgpu::TextureViewDimension::D1 => default_1d_view.clone(),
                        wgpu::TextureViewDimension::D3 => default_3d_view.clone(),
                        _ => Some(default_view.clone()),
                    };
                    if let Some(v) = fallback {
                        views.push((s.tex_binding, v));
                        sampler_entries.push((s.samp_binding, default_sampler.clone()));
                    }
                }
            }
        }
        let mut entries = Vec::new();
        for (binding, buf) in &buffers {
            entries.push(wgpu::BindGroupEntry {
                binding: *binding,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: buf,
                    offset: 0,
                    size: None,
                }),
            });
        }
        for (binding, view) in &views {
            entries.push(wgpu::BindGroupEntry {
                binding: *binding,
                resource: wgpu::BindingResource::TextureView(view),
            });
        }
        for (binding, sampler) in &sampler_entries {
            entries.push(wgpu::BindGroupEntry {
                binding: *binding,
                resource: wgpu::BindingResource::Sampler(sampler),
            });
        }
        entries.sort_by_key(|e| e.binding);
        let layout = Self::create_bind_group_layout(reflection, &device);
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("phx-bg"),
            layout: &layout,
            entries: &entries,
        });
        self.bind_group_cache.insert(cache_key, bg.clone());
        Some(bg)
    }

    /// Upload the fixed and shader-declared uniform buffers staged by the
    /// command stream. Immediate and indexed draws share this step; omitting
    /// it from either path leaves matrix-based vertices at zero and silently
    /// clips the whole primitive in wgpu.
    fn upload_staged_uniforms(&mut self) {
        let Some(queue) = self.queue.clone() else {
            return;
        };
        for (binding, data) in [
            (0u32, self.camera_ubo.as_deref()),
            (1u32, self.material_ubo.as_deref()),
            (2u32, self.light_ubo.as_deref()),
        ] {
            if let Some(bytes) = data {
                if !bytes.is_empty() {
                    if let Some(buf) = self.ubo_buffers.get(&binding) {
                        let n = bytes.len().min(buf.size() as usize);
                        queue.write_buffer(buf, 0, &bytes[..n]);
                    }
                }
            }
        }
        let dirty: Vec<u32> = self.plain_uniform_dirty.iter().copied().collect();
        for binding in dirty {
            if let (Some(bytes), Some(buf)) = (
                self.plain_uniform_staging.get(&binding),
                self.ubo_buffers.get(&binding),
            ) {
                let n = bytes.len().min(buf.size() as usize);
                queue.write_buffer(buf, 0, &bytes[..n]);
            }
            self.plain_uniform_dirty.remove(&binding);
        }
    }

    /// Upload dirty UBO staging + plain uniforms, then run one indexed draw
    /// against the current render target.
    fn draw_indexed(
        &mut self,
        mesh_buffers: (wgpu::Buffer, wgpu::Buffer, wgpu::IndexFormat),
        pipeline: &wgpu::RenderPipeline,
        bind_group: &wgpu::BindGroup,
        index_count: u32,
        instance_count: u32,
    ) {
        let Some(queue) = self.queue.clone() else {
            return;
        };
        self.upload_staged_uniforms();

        let Some((attachments, depth_view, _depth_format, _w, h, clear_color, clear_depth)) =
            self.current_target_views(true)
        else {
            return;
        };
        let Some(device) = self.device.clone() else {
            return;
        };
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("phx-encoder"),
        });
        {
            let clear = clear_color;
            let mut color_attachments: Vec<Option<wgpu::RenderPassColorAttachment>> = Vec::new();
            for (view, _fmt) in &attachments {
                color_attachments.push(Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match clear {
                            Some(c) => wgpu::LoadOp::Clear(wgpu::Color {
                                r: c[0] as f64,
                                g: c[1] as f64,
                                b: c[2] as f64,
                                a: c[3] as f64,
                            }),
                            None => wgpu::LoadOp::Load,
                        },
                        store: wgpu::StoreOp::Store,
                    },
                }));
            }
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("phx-pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: depth_view.as_ref().map(|dv| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view: dv,
                        depth_ops: Some(wgpu::Operations {
                            load: match clear_depth {
                                Some(d) => wgpu::LoadOp::Clear(d),
                                None => wgpu::LoadOp::Load,
                            },
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }
                }),
                ..Default::default()
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, bind_group, &[]);
            pass.set_vertex_buffer(0, mesh_buffers.0.slice(..));
            pass.set_index_buffer(mesh_buffers.1.slice(..), mesh_buffers.2);
            if let Some((x, y, width, height)) = self.viewport {
                // GL viewport origin is bottom-left; wgpu top-left.
                let vp_y = (h as i32 - y - height).max(0) as f32;
                pass.set_viewport(
                    x as f32,
                    vp_y,
                    width.max(0) as f32,
                    height.max(0) as f32,
                    0.0,
                    1.0,
                );
            }
            if self.scissor_enabled {
                if let Some((x, y, width, height)) = self.scissor {
                    let sc_y = (h as i32 - y - height).max(0) as u32;
                    pass.set_scissor_rect(
                        x.max(0) as u32,
                        sc_y,
                        width.max(0) as u32,
                        height.max(0) as u32,
                    );
                }
            }
            pass.draw_indexed(0..index_count, 0, 0..instance_count.max(1));
        }
        queue.submit([encoder.finish()]);
    }

    fn primitive_topology_with_strip(
        primitive: CmdPrimitiveType,
    ) -> (wgpu::PrimitiveTopology, Option<wgpu::IndexFormat>) {
        match primitive {
            CmdPrimitiveType::Points => (wgpu::PrimitiveTopology::PointList, None),
            CmdPrimitiveType::Lines => (wgpu::PrimitiveTopology::LineList, None),
            CmdPrimitiveType::LineStrip => (wgpu::PrimitiveTopology::LineStrip, None),
            CmdPrimitiveType::Triangles => (wgpu::PrimitiveTopology::TriangleList, None),
            CmdPrimitiveType::TriangleStrip => (
                wgpu::PrimitiveTopology::TriangleStrip,
                Some(wgpu::IndexFormat::Uint32),
            ),
            // GL's own to_gl maps Quads/TriangleFan to Triangles.
            CmdPrimitiveType::TriangleFan | CmdPrimitiveType::Quads => {
                (wgpu::PrimitiveTopology::TriangleList, None)
            }
        }
    }

    pub(super) fn cmd_reload_shader(
        &mut self,
        shader_key: &str,
        vertex_src: &str,
        fragment_src: &str,
    ) -> CommandReply {
        // Same semantics as the GL path: compile a FRESH pair, store it under
        // shader_key, and let BindShaderByResource prefer it. On failure the
        // old pair stays in effect (reload never destroys the working shader).
        let result = match self.compile_shader_pair(vertex_src, fragment_src) {
            Ok((vs, fs, _avs, _afs, reflection)) => {
                self.hot_reloaded_shaders
                    .insert(shader_key.to_string(), (vs, fs, reflection));
                ShaderReloadResult {
                    shader_key: shader_key.to_string(),
                    error: None,
                    program: 0,
                }
            }
            Err(e) => {
                tracing::error!("wgpu: shader reload '{shader_key}' failed: {e}");
                ShaderReloadResult {
                    shader_key: shader_key.to_string(),
                    error: Some(e),
                    program: 0,
                }
            }
        };
        CommandReply::ShaderReload(result)
    }

    pub(super) fn cmd_create_texture_1d(
        &mut self,
        id: ResourceId,
        width: u32,
        format: TexFormat,
        data: Option<Vec<u8>>,
    ) {
        // REAL D1 texture: backing 1D textures with a 1xN 2D texture makes
        // bind-group validation fail for sampler1D bindings (the layout
        // expects dimension D1, the 2D view is D2 -> probe-verified crash).
        let Some(device) = self.device.clone() else {
            return;
        };
        let Some(queue) = self.queue.clone() else {
            return;
        };
        let (wgpu_format, bpp, converted) = Self::tex_format_to_wgpu_with_bpp(format);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("phx-tex1d"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D1,
            format: wgpu_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        if let Some(raw) = data {
            let expected = width as usize * bpp;
            if raw.len() >= expected {
                let bytes = if converted {
                    match format {
                        TexFormat::R32F => Self::f32_to_rgba16f_bytes(&raw),
                        TexFormat::RGBA32F => Self::f32_to_f16_bytes(&raw),
                        _ => Self::rgb_to_rgba(&raw, width, 1),
                    }
                } else {
                    raw
                };
                let dest_bpp = if converted {
                    match format {
                        TexFormat::R32F | TexFormat::RGBA32F => 8,
                        _ => 4,
                    }
                } else {
                    bpp
                } as u32;
                Self::upload_texture_2d(&queue, &texture, width, 1, &bytes, dest_bpp);
            }
        }
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D1),
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("phx-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });
        self.resources.insert(
            id,
            WgpuGpuResource::Texture1D {
                texture,
                view,
                sampler,
            },
        );
    }

    pub(super) fn cmd_create_texture_2d(
        &mut self,
        id: ResourceId,
        width: u32,
        height: u32,
        format: TexFormat,
        data: Option<Vec<u8>>,
    ) {
        let Some(device) = self.device.clone() else {
            return;
        };
        let Some(queue) = self.queue.clone() else {
            return;
        };
        let (wgpu_format, bpp, converted) = Self::tex_format_to_wgpu_with_bpp(format);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("phx-tex2d"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        if let Some(raw) = data {
            let expected = (width * height) as usize * bpp;
            if raw.len() >= expected {
                let (bytes, dest_bpp) = if converted {
                    match format {
                        TexFormat::R32F => (Self::f32_to_rgba16f_bytes(&raw), 8),
                        TexFormat::RGBA32F => (Self::f32_to_f16_bytes(&raw), 8),
                        _ => (Self::rgb_to_rgba(&raw, width, height), 4),
                    }
                } else {
                    (raw, bpp)
                };
                Self::upload_texture_2d(&queue, &texture, width, height, &bytes, dest_bpp as u32);
            }
        }
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        // 32F formats are NOT filterable in WebGPU: a Linear sampler for an
        // Rgba32Float texture fails bind-group validation. Use Nearest for
        // those (the shaders texelFetch them anyway).
        let filterable = !matches!(
            wgpu_format,
            wgpu::TextureFormat::Rgba32Float | wgpu::TextureFormat::R32Float
        );
        let filter = if filterable {
            wgpu::FilterMode::Linear
        } else {
            wgpu::FilterMode::Nearest
        };
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("phx-sampler"),
            mag_filter: filter,
            min_filter: filter,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });
        self.resources.insert(
            id,
            WgpuGpuResource::Texture2D {
                texture,
                view,
                sampler,
            },
        );
    }

    /// Rgba8UnormSrgb -> Rgba8Unorm distinction for shader-visible textures:
    /// keep the raw format (the engine's data is raw RGBA); srgb correction
    /// is applied at the view level if the format demands it. We use the
    /// non-srgb format to match GL's unmanaged sampling.
    fn f16_to_f32(bits: u16) -> f32 {
        let sign = if bits & 0x8000 != 0 { -1.0 } else { 1.0 };
        let exponent = ((bits >> 10) & 0x1f) as i32;
        let mantissa = (bits & 0x03ff) as u32;
        match exponent {
            0 => sign * (mantissa as f32 / 1024.0) * 2.0f32.powi(-14),
            0x1f => {
                if mantissa == 0 {
                    sign * f32::INFINITY
                } else {
                    f32::NAN
                }
            }
            _ => sign * (1.0 + mantissa as f32 / 1024.0) * 2.0f32.powi(exponent - 15),
        }
    }

    fn float_to_u8(value: f32) -> u8 {
        if !value.is_finite() {
            return 0;
        }
        (value.clamp(0.0, 1.0) * 255.0).round() as u8
    }

    fn decode_sample_pixel(format: wgpu::TextureFormat, bytes: &[u8]) -> [u8; 4] {
        let half = |offset: usize| -> f32 {
            bytes
                .get(offset..offset + 2)
                .and_then(|raw| raw.try_into().ok())
                .map(u16::from_le_bytes)
                .map(Self::f16_to_f32)
                .unwrap_or(0.0)
        };
        let float = |offset: usize| -> f32 {
            bytes
                .get(offset..offset + 4)
                .and_then(|raw| raw.try_into().ok())
                .map(f32::from_le_bytes)
                .unwrap_or(0.0)
        };
        match format {
            wgpu::TextureFormat::Rgba16Float => [
                Self::float_to_u8(half(0)),
                Self::float_to_u8(half(2)),
                Self::float_to_u8(half(4)),
                Self::float_to_u8(half(6)),
            ],
            wgpu::TextureFormat::R16Float => [Self::float_to_u8(half(0)), 0, 0, 255],
            wgpu::TextureFormat::Rg16Float => [
                Self::float_to_u8(half(0)),
                Self::float_to_u8(half(2)),
                0,
                255,
            ],
            wgpu::TextureFormat::R32Float => [Self::float_to_u8(float(0)), 0, 0, 255],
            wgpu::TextureFormat::Rgba32Float => [
                Self::float_to_u8(float(0)),
                Self::float_to_u8(float(4)),
                Self::float_to_u8(float(8)),
                Self::float_to_u8(float(12)),
            ],
            wgpu::TextureFormat::Rgba8Unorm
            | wgpu::TextureFormat::Rgba8UnormSrgb
            | wgpu::TextureFormat::Rgba8Snorm => [
                *bytes.first().unwrap_or(&0),
                *bytes.get(1).unwrap_or(&0),
                *bytes.get(2).unwrap_or(&0),
                *bytes.get(3).unwrap_or(&255),
            ],
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => [
                *bytes.get(2).unwrap_or(&0),
                *bytes.get(1).unwrap_or(&0),
                *bytes.first().unwrap_or(&0),
                *bytes.get(3).unwrap_or(&255),
            ],
            wgpu::TextureFormat::R8Unorm
            | wgpu::TextureFormat::R8Snorm
            | wgpu::TextureFormat::R8Uint
            | wgpu::TextureFormat::R8Sint => [*bytes.first().unwrap_or(&0), 0, 0, 255],
            _ => [
                *bytes.first().unwrap_or(&0),
                *bytes.get(1).unwrap_or(&0),
                *bytes.get(2).unwrap_or(&0),
                *bytes.get(3).unwrap_or(&255),
            ],
        }
    }

    /// Convert f32 (4 bytes LE) to f16 (2 bytes LE), round-to-nearest-even.
    fn f32_to_f16_bytes(data: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(data.len() / 2);
        for chunk in data.chunks_exact(4) {
            let f = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let bits = f.to_bits();
            let sign = ((bits >> 16) & 0x8000) as u16;
            let exp = (((bits >> 23) & 0xff) as i32) - 127 + 15;
            let mant = bits & 0x7fffff;
            let h: u16 = if exp <= 0 {
                if exp < -10 {
                    sign
                } else {
                    let m = mant | 0x800000;
                    let shift = (14 - exp) as u32;
                    let mut v = m >> shift;
                    let rem = m & ((1u32 << shift) - 1);
                    let halfway = 1u32 << (shift - 1);
                    if rem > halfway || (rem == halfway && (v & 1) == 1) {
                        v += 1;
                    }
                    sign | v as u16
                }
            } else if exp >= 31 {
                sign | 0x7c00
            } else {
                let mut v = ((exp as u32) << 10) | (mant >> 13);
                let rem = mant & 0x1fff;
                if rem > 0x1000 || (rem == 0x1000 && (v & 1) == 1) {
                    v += 1;
                }
                sign | v as u16
            };
            out.extend_from_slice(&h.to_le_bytes());
        }
        out
    }

    /// Expand an engine R32F upload into the filterable/renderable RGBA16F
    /// representation used by WGPU. Deferred shaders consume only `.x`; the
    /// fixed zero/one channels preserve the scalar texture's public meaning.
    fn f32_to_rgba16f_bytes(data: &[u8]) -> Vec<u8> {
        let mut out = Vec::with_capacity(data.len() * 2);
        for chunk in data.chunks_exact(4) {
            out.extend_from_slice(&Self::f32_to_f16_bytes(chunk));
            out.extend_from_slice(&[0, 0]);
            out.extend_from_slice(&[0, 0]);
            out.extend_from_slice(&[0, 0x3c]);
        }
        out
    }

    pub(crate) fn tex_format_to_wgpu_with_bpp(
        format: TexFormat,
    ) -> (wgpu::TextureFormat, usize, bool) {
        // RGB8: payload is 3 bpp, texture is Rgba8Unorm (expanded).
        // R32F/RGBA32F: the WebGPU portability model forbids filtering on
        // 32F formats, but the engine's shaders sample them with filtering
        // samplers (lighting depth + the instanced asteroid transforms) —
        // so create them as 16F (filterable) and convert the f32 payload
        // CPU-side. A 32F texture can never satisfy a filterable sampler
        // binding regardless of the sampler's filter mode.
        let converted =
            format == TexFormat::RGB8 || format == TexFormat::R32F || format == TexFormat::RGBA32F;
        let wf = match format {
            TexFormat::RGB8 => wgpu::TextureFormat::Rgba8Unorm,
            TexFormat::RGBA8 => wgpu::TextureFormat::Rgba8Unorm,
            TexFormat::RGBA16F => wgpu::TextureFormat::Rgba16Float,
            TexFormat::R16F => wgpu::TextureFormat::R16Float,
            TexFormat::RG16F => wgpu::TextureFormat::Rg16Float,
            TexFormat::R8 => wgpu::TextureFormat::R8Unorm,
            TexFormat::RG8 => wgpu::TextureFormat::Rg8Unorm,
            TexFormat::R16 => wgpu::TextureFormat::R16Unorm,
            TexFormat::RG16 => wgpu::TextureFormat::Rg16Unorm,
            TexFormat::RGBA32F => wgpu::TextureFormat::Rgba16Float,
            TexFormat::R32F => wgpu::TextureFormat::Rgba16Float,
            TexFormat::RG32F => wgpu::TextureFormat::Rg32Float,
            TexFormat::RGBA16 => wgpu::TextureFormat::Rgba16Unorm,
            TexFormat::Depth24 => wgpu::TextureFormat::Depth24Plus,
            TexFormat::Depth16 => wgpu::TextureFormat::Depth16Unorm,
            TexFormat::Depth32F => wgpu::TextureFormat::Depth32Float,
        };
        let bpp = if converted {
            match format {
                TexFormat::R32F => 4,     // f32 payload, expanded to RGBA16F
                TexFormat::RGBA32F => 16, // 4 x f32, converted to f16
                _ => 3,
            }
        } else {
            match format {
                TexFormat::RGB8 => 3,
                TexFormat::RGBA8 => 4,
                TexFormat::RGBA16F => 8,
                TexFormat::R16F => 2,
                TexFormat::RG16F => 4,
                TexFormat::R8 => 1,
                TexFormat::RG8 => 2,
                TexFormat::R16 => 2,
                TexFormat::RG16 => 4,
                TexFormat::RGBA32F => 16,
                TexFormat::R32F => 4,
                TexFormat::RG32F => 8,
                TexFormat::RGBA16 => 8,
                TexFormat::Depth24 | TexFormat::Depth16 | TexFormat::Depth32F => 4,
            }
        };
        (wf, bpp, converted)
    }

    fn rgb_to_rgba(raw: &[u8], width: u32, height: u32) -> Vec<u8> {
        let count = (width * height) as usize;
        let mut out = Vec::with_capacity(count * 4);
        for px in raw.chunks(3).take(count) {
            out.extend_from_slice(&[px[0], px[1], px[2], 255]);
        }
        out
    }

    /// Bytes per pixel for the formats the engine actually uploads.
    pub(crate) fn format_bpp(format: wgpu::TextureFormat) -> u32 {
        match format {
            wgpu::TextureFormat::R8Unorm => 1,
            wgpu::TextureFormat::Rg8Unorm => 2,
            wgpu::TextureFormat::R16Unorm | wgpu::TextureFormat::R16Float => 2,
            // RG = 2 components: RG16 is 4 B/texel, RG32F is 8.
            wgpu::TextureFormat::Rg16Unorm | wgpu::TextureFormat::Rg16Float => 4,
            wgpu::TextureFormat::Rg32Float => 8,
            wgpu::TextureFormat::Rgba8Unorm
            | wgpu::TextureFormat::Rgba8UnormSrgb
            | wgpu::TextureFormat::Bgra8Unorm
            | wgpu::TextureFormat::Bgra8UnormSrgb => 4,
            wgpu::TextureFormat::Rgba16Float => 8,
            wgpu::TextureFormat::Rgba32Float => 16,
            _ => 4,
        }
    }

    fn upload_texture_2d(
        queue: &wgpu::Queue,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
        bytes: &[u8],
        bpp: u32,
    ) {
        // wgpu write_texture requires bytes_per_row aligned to 256.
        let bpr = (width * bpp).max(1);
        let padded_bpr = (bpr + 255) & !255;
        let mut padded = Vec::with_capacity(padded_bpr as usize * height as usize);
        for row in bytes.chunks(bpr as usize).take(height as usize) {
            padded.extend_from_slice(row);
            padded.resize(padded.len() + (padded_bpr as usize - row.len()), 0);
        }
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &padded,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bpr),
                rows_per_image: Some(height.max(1)),
            },
            wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
        );
    }

    pub(super) fn cmd_create_texture_3d(
        &mut self,
        _id: ResourceId,
        _width: u32,
        _height: u32,
        _depth: u32,
        _format: TexFormat,
        _data: Option<Vec<u8>>,
    ) {
        // 3D textures are not sampled by the benchmark scene; create a
        // placeholder 2D texture so bind-group lookups never panic.
        self.cmd_create_texture_2d(_id, _width.max(1), _height.max(1), _format, None);
    }

    pub(super) fn cmd_create_texture_cube(&mut self, id: ResourceId, size: u32, format: TexFormat) {
        let Some(device) = self.device.clone() else {
            return;
        };
        let Some(queue) = self.queue.clone() else {
            return;
        };
        let (wgpu_format, bpp, _converted) = Self::tex_format_to_wgpu_with_bpp(format);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("phx-texcube"),
            size: wgpu::Extent3d {
                width: size.max(1),
                height: size.max(1),
                depth_or_array_layers: 6,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        // fill each face with magenta so unloaded faces are visibly wrong
        let face_bytes = (size * size) as usize * bpp;
        let mut face = vec![0u8; face_bytes.max(4)];
        if bpp >= 4 {
            for px in face.chunks_mut(4) {
                px[0] = 255;
                px[2] = 255;
                px[3] = 255;
            }
        }
        for layer in 0..6u32 {
            let bpr = (size * bpp as u32).max(1);
            let padded_bpr = (bpr + 255) & !255;
            let mut padded = Vec::with_capacity(padded_bpr as usize * size.max(1) as usize);
            for row in face.chunks(bpr as usize).take(size.max(1) as usize) {
                padded.extend_from_slice(row);
                padded.resize(padded.len() + (padded_bpr as usize - row.len()), 0);
            }
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: 0,
                        y: 0,
                        z: layer,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                &padded,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bpr),
                    rows_per_image: Some(size.max(1)),
                },
                wgpu::Extent3d {
                    width: size.max(1),
                    height: size.max(1),
                    depth_or_array_layers: 1,
                },
            );
        }
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("phx-sampler-cube"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });
        self.resources.insert(
            id,
            WgpuGpuResource::TextureCube {
                texture,
                view,
                sampler,
            },
        );
    }

    pub(super) fn cmd_create_mesh(
        &mut self,
        id: ResourceId,
        vertices: Vec<u8>,
        indices: Vec<u32>,
        vertex_format: VertexFormat,
    ) {
        let Some(device) = self.device.clone() else {
            return;
        };
        let Some(queue) = self.queue.clone() else {
            return;
        };
        // Meshes without color data get zero-padded vertices: the vertex
        // include declares `in vec4 vertex_color;` unconditionally and the
        // pipeline now ALWAYS provides the color attribute (location 3), so
        // the buffer must carry 12 zero bytes per vertex (GL's disabled
        // attribute array reads as constant (0,0,0,0) - this reproduces it).
        let (vertices, vertex_format) = if vertex_format.has_color {
            // Color meshes also need the trailing uint instanceIndex slot:
            // append 4 zero bytes per vertex (stride 48 -> 52).
            let stride = vertex_format.stride as usize;
            let mut padded =
                Vec::with_capacity(vertices.len() + (vertices.len() / stride.max(1)) * 4);
            for chunk in vertices.chunks(stride.max(1)) {
                padded.extend_from_slice(chunk);
                padded.extend_from_slice(&[0u8; 4]);
            }
            let mut fmt = vertex_format;
            fmt.stride += 4;
            (padded, fmt)
        } else {
            let stride = vertex_format.stride as usize;
            // Float32x4 color = 16 bytes + 4 bytes for the instanceIndex
            // uint at the end of every vertex.
            let mut padded =
                Vec::with_capacity(vertices.len() + (vertices.len() / stride.max(1)) * 20);
            for chunk in vertices.chunks(stride.max(1)) {
                padded.extend_from_slice(chunk);
                padded.extend_from_slice(&[0u8; 20]);
            }
            let mut fmt = vertex_format;
            fmt.stride += 20;
            (padded, fmt)
        };
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("phx-mesh-vb"),
            size: vertices.len().max(1) as u64,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        if !vertices.is_empty() {
            queue.write_buffer(&vertex_buffer, 0, &vertices);
        }
        let mut index_bytes = Vec::with_capacity(indices.len() * 4);
        for i in &indices {
            index_bytes.extend_from_slice(&i.to_le_bytes());
        }
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("phx-mesh-ib"),
            size: index_bytes.len().max(1) as u64,
            usage: wgpu::BufferUsages::INDEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        if !index_bytes.is_empty() {
            queue.write_buffer(&index_buffer, 0, &index_bytes);
        }
        self.resources.insert(
            id,
            WgpuGpuResource::Mesh {
                vertex_buffer,
                index_buffer,
                index_format: wgpu::IndexFormat::Uint32,
                index_count: indices.len() as u32,
                vertex_count: if vertex_format.stride > 0 {
                    (vertices.len() / vertex_format.stride as usize) as u32
                } else {
                    0
                },
                vertex_format,
            },
        );
    }

    pub(super) fn cmd_destroy_resource(&mut self, ids: &[ResourceId]) {
        for id in ids {
            self.resources.remove(id);
            self.texture_filter_modes.remove(id);
        }
    }

    // --- Uniform buffer objects (byte staging real; upload = draw stage) ---

    pub(super) fn cmd_create_camera_ubo(&mut self) {
        self.camera_ubo = Some(Vec::new());
    }

    pub(super) fn cmd_update_camera_ubo(&mut self, data: &[u8]) {
        if let Some(ubo) = self.camera_ubo.as_mut() {
            ubo.clear();
            ubo.extend_from_slice(data);
        } else {
            warn!("wgpu: UpdateCameraUBO before CreateCameraUBO");
        }
    }

    pub(super) fn cmd_create_material_ubo(&mut self) {
        self.material_ubo = Some(Vec::new());
    }

    pub(super) fn cmd_update_material_ubo(&mut self, data: &[u8]) {
        if let Some(ubo) = self.material_ubo.as_mut() {
            ubo.clear();
            ubo.extend_from_slice(data);
        } else {
            warn!("wgpu: UpdateMaterialUBO before CreateMaterialUBO");
        }
    }

    pub(super) fn cmd_create_light_ubo(&mut self) {
        self.light_ubo = Some(Vec::new());
    }

    pub(super) fn cmd_update_light_ubo(&mut self, data: &[u8]) {
        if let Some(ubo) = self.light_ubo.as_mut() {
            ubo.clear();
            ubo.extend_from_slice(data);
        } else {
            warn!("wgpu: UpdateLightUBO before CreateLightUBO");
        }
    }

    // --- Window operations / synchronization (device-bound) ---

    pub(super) fn cmd_resize(&mut self, width: u32, height: u32) {
        let new_size = (width.max(1), height.max(1));
        if new_size == self.surface_size {
            return;
        }
        self.surface_size = new_size;
        self.surface_frame = None;
        self.reconfigure_surface();
    }

    fn flush_pending_surface_clear(&mut self) {
        // Draw passes normally consume pending clears while they open their
        // attachments. A clear-only frame has no draw pass, so it must open a
        // no-draw pass here or presentation would expose the surface's
        // previous/undefined contents instead of the requested clear color.
        // The owner keeps the pending values until the pass is submitted so a
        // missing device/queue cannot silently discard the application state.
        let Some((attachments, depth_view, _depth_format, _w, _h, clear_color, clear_depth)) =
            self.current_target_views(false)
        else {
            return;
        };
        if clear_color.is_none() && clear_depth.is_none() {
            return;
        }
        let Some(device) = self.device.clone() else {
            return;
        };
        let Some(queue) = self.queue.clone() else {
            return;
        };
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("phx-surface-clear-encoder"),
        });
        let color_attachments = attachments
            .iter()
            .map(|(view, _format)| {
                Some(wgpu::RenderPassColorAttachment {
                    view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match clear_color {
                            Some([r, g, b, a]) => wgpu::LoadOp::Clear(wgpu::Color {
                                r: r as f64,
                                g: g as f64,
                                b: b as f64,
                                a: a as f64,
                            }),
                            None => wgpu::LoadOp::Load,
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })
            })
            .collect::<Vec<_>>();
        let depth_stencil_attachment = clear_depth.and_then(|depth| {
            depth_view
                .as_ref()
                .map(|view| wgpu::RenderPassDepthStencilAttachment {
                    view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(depth),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                })
        });
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("phx-surface-clear-pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment,
                ..Default::default()
            });
        }
        queue.submit([encoder.finish()]);
        self.surface_pending_clear_color = None;
        self.surface_pending_clear_depth = None;
    }

    pub(super) fn cmd_swap_buffers(&mut self) -> CommandReply {
        if self.framebuffer_stack.is_empty()
            && (self.surface_pending_clear_color.is_some()
                || self.surface_pending_clear_depth.is_some())
        {
            self.flush_pending_surface_clear();
        }
        if let Some(frame) = self.surface_frame.take() {
            if let Some(queue) = self.queue.as_ref() {
                queue.present(frame);
            }
        }
        // End-frame stats bookkeeping: snapshot + hand back as the reply so
        // the threaded backend forwards it to the stats dashboard (the GL
        // executor does the same).
        self.stats.frame_count += 1;
        self.last_stats = RenderStats {
            commands_processed: self.stats.commands_processed,
            draw_calls_cumulative: self.stats.draw_calls,
            state_changes_cumulative: self.stats.state_changes,
            frame_count: self.stats.frame_count,
            last_frame_time_us: 0,
            commands: self.commands_this_frame,
            draw_calls: self.draw_mesh_calls_this_frame
                + self.draw_immediate_calls_this_frame
                + self.draw_instanced_calls_this_frame,
            state_changes: self.state_changes_this_frame,
            present_wait_us: 0,
            draw_mesh_calls: self.draw_mesh_calls_this_frame,
            draw_immediate_calls: self.draw_immediate_calls_this_frame,
            draw_instanced_calls: self.draw_instanced_calls_this_frame,
            immediate_vertices: self.immediate_vertices_this_frame,
            instanced_data_items: self.instanced_data_items_this_frame,
            vertices_drawn: self.vertices_drawn_this_frame,
            ..self.last_stats
        };
        CommandReply::Stats(Box::new(self.last_stats.clone()))
    }

    /// Mirrors GL's `set_swap_interval`: reconfigure the swapchain with the
    /// new present mode (Vsync = Fifo, NoVsync = Immediate).
    pub(super) fn cmd_set_present_mode(&mut self, mode: PresentMode) {
        let Some(cfg) = self.surface_config.as_mut() else {
            return;
        };
        let present_mode: wgpu::PresentMode = mode.into();
        if cfg.present_mode == present_mode {
            return;
        }
        cfg.present_mode = present_mode;
        self.surface_frame = None;
        self.reconfigure_surface();
        tracing::info!("wgpu: present mode set to {mode:?}");
    }

    pub(super) fn cmd_flush(&mut self) {
        // This is the explicit public GPU-finish operation (`Draw.Flush`), not
        // teardown. Keep its foreign-driver boundary finite and report a
        // timeout instead of pretending the GPU is idle.
        let Some(device) = self.device.as_ref() else {
            return;
        };
        if let Err(error) = device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_secs(5)),
        }) {
            warn!("wgpu GPU finish did not complete: {error:?}");
        }
    }

    // =====================================================================
    // Dispatch — exhaustive over every RenderCommand variant
    // =====================================================================

    /// Execute one command. Mirrors `CommandExecutor::execute` — counts the
    /// same stats, then routes to the wgpu `cmd_*` implementation.
    pub fn execute(&mut self, cmd: RenderCommand) -> CommandReply {
        let mut reply = CommandReply::None;

        self.stats.commands_processed += 1;
        self.commands_this_frame += 1;
        if cmd.is_draw_call() {
            self.stats.draw_calls += 1;
        }
        if cmd.is_state_change() {
            self.stats.state_changes += 1;
            self.state_changes_this_frame += 1;
        }

        match cmd {
            // === State Management ===
            RenderCommand::SetViewport {
                x,
                y,
                width,
                height,
            } => {
                self.cmd_set_viewport(x, y, width, height);
            }
            RenderCommand::SetScissor {
                x,
                y,
                width,
                height,
            } => {
                self.cmd_set_scissor(x, y, width, height);
            }
            RenderCommand::EnableScissor(enable) => self.cmd_enable_scissor(enable),
            RenderCommand::SetBlendMode(mode) => self.cmd_set_blend_mode(mode),
            RenderCommand::SetCullFace(face) => self.cmd_set_cull_face(face),
            RenderCommand::SetDepthTest(enable) => self.cmd_set_depth_test(enable),
            RenderCommand::SetDepthWritable(enable) => self.cmd_set_depth_writable(enable),
            RenderCommand::SetWireframe(enable) => self.cmd_set_wireframe(enable),
            RenderCommand::SetLineWidth(width) => self.cmd_set_line_width(width),
            RenderCommand::SetPointSize(size) => self.cmd_set_point_size(size),

            // === Shader Operations ===
            RenderCommand::BindShader { handle } => self.cmd_bind_shader(handle),
            RenderCommand::BindShaderByResource { id, shader_key } => {
                self.cmd_bind_shader_by_resource(id, shader_key);
            }
            RenderCommand::UnbindShader => self.cmd_unbind_shader(),
            RenderCommand::SetUniformInt { location, value } => {
                self.cmd_set_uniform_int(location, value);
            }
            RenderCommand::SetUniformInt2 { location, value } => {
                self.cmd_set_uniform_int2(location, value);
            }
            RenderCommand::SetUniformInt3 { location, value } => {
                self.cmd_set_uniform_int3(location, value);
            }
            RenderCommand::SetUniformInt4 { location, value } => {
                self.cmd_set_uniform_int4(location, value);
            }
            RenderCommand::SetUniformFloat { location, value } => {
                self.cmd_set_uniform_float(location, value);
            }
            RenderCommand::SetUniformFloat2 { location, value } => {
                self.cmd_set_uniform_float2(location, value);
            }
            RenderCommand::SetUniformFloat3 { location, value } => {
                self.cmd_set_uniform_float3(location, value);
            }
            RenderCommand::SetUniformFloat4 { location, value } => {
                self.cmd_set_uniform_float4(location, value);
            }
            RenderCommand::SetUniformMat4 { location, value } => {
                self.cmd_set_uniform_mat4(location, value);
            }
            RenderCommand::SetInstanceUniforms(cmd) => {
                let InstanceUniformsCmd {
                    world_loc,
                    world_it_loc,
                    scale_loc,
                    world,
                    world_it,
                    scale,
                } = *cmd;
                // GL writes three uniforms at cached locations; wgpu stages
                // them under fixed names for the draw path.
                let _ = (world_loc, world_it_loc, scale_loc);
                self.named_uniforms
                    .insert("__inst_world".into(), UniformValue::Mat4(world));
                self.named_uniforms
                    .insert("__inst_world_it".into(), UniformValue::Mat4(world_it));
                self.named_uniforms
                    .insert("__inst_scale".into(), UniformValue::Float(scale));
            }

            // === Name-based Uniform Operations ===
            RenderCommand::SetUniformIntByName { name, value } => {
                self.cmd_set_uniform_int_by_name(name, value);
            }
            RenderCommand::SetUniformInt2ByName { name, value } => {
                self.cmd_set_uniform_int2_by_name(name, value);
            }
            RenderCommand::SetUniformInt3ByName { name, value } => {
                self.cmd_set_uniform_int3_by_name(name, value);
            }
            RenderCommand::SetUniformInt4ByName { name, value } => {
                self.cmd_set_uniform_int4_by_name(name, value);
            }
            RenderCommand::SetUniformFloatByName { name, value } => {
                self.cmd_set_uniform_float_by_name(name, value);
            }
            RenderCommand::SetUniformFloat2ByName { name, value } => {
                self.cmd_set_uniform_float2_by_name(name, value);
            }
            RenderCommand::SetUniformFloat3ByName { name, value } => {
                self.cmd_set_uniform_float3_by_name(name, value);
            }
            RenderCommand::SetUniformFloat4ByName { name, value } => {
                self.cmd_set_uniform_float4_by_name(name, value);
            }
            RenderCommand::SetUniformMat4ByGenericName { name, value } => {
                self.cmd_set_uniform_mat4_by_generic_name(name, value);
            }
            RenderCommand::SetUniformMat4ByName { name, value } => {
                self.cmd_set_uniform_mat4_by_name(name, value);
            }

            // === Texture Operations ===
            RenderCommand::BindTexture2D { slot, handle } => self.cmd_bind_texture_2d(slot, handle),
            RenderCommand::BindTexture2DByResource { slot, id } => {
                self.cmd_bind_texture_2d_by_resource(slot, id);
            }
            RenderCommand::BindTexture1DByResource { slot, id } => {
                self.cmd_bind_texture_1d_by_resource(slot, id);
            }
            RenderCommand::BindTexture3D { slot, handle } => self.cmd_bind_texture_3d(slot, handle),
            RenderCommand::BindTexture3DByResource { slot, id } => {
                self.cmd_bind_texture_3d_by_resource(slot, id);
            }
            RenderCommand::BindTextureCube { slot, handle } => {
                self.cmd_bind_texture_cube(slot, handle)
            }
            RenderCommand::BindTextureCubeByResource { slot, id } => {
                self.cmd_bind_texture_cube_by_resource(slot, id);
            }
            RenderCommand::UnbindTexture { slot } => self.cmd_unbind_texture(slot),

            // === Texture State Commands ===
            RenderCommand::SetTexture2DMagFilter { handle, filter } => {
                self.cmd_set_texture_2d_mag_filter(handle, filter);
            }
            RenderCommand::SetTexture2DMinFilter { handle, filter } => {
                self.cmd_set_texture_2d_min_filter(handle, filter);
            }
            RenderCommand::SetTexture2DWrapMode { handle, mode } => {
                self.cmd_set_texture_2d_wrap_mode(handle, mode);
            }
            RenderCommand::SetTexture2DMipRange {
                handle,
                min_level,
                max_level,
            } => {
                self.cmd_set_texture_2d_mip_range(handle, min_level, max_level);
            }
            RenderCommand::GenerateMipmap2D { handle } => self.cmd_generate_mipmap_2d(handle),
            RenderCommand::UpdateTexture2DData {
                handle,
                width,
                height,
                internal_format,
                pixel_format,
                data_format,
                data,
            } => self.cmd_update_texture_2d_data(
                handle,
                width,
                height,
                internal_format,
                pixel_format,
                data_format,
                data,
            ),
            RenderCommand::UpdateTexture2DDataByResource {
                id,
                width,
                height,
                internal_format,
                pixel_format,
                data_format,
                data,
            } => self.cmd_update_texture_2d_data_by_resource(
                id,
                width,
                height,
                internal_format,
                pixel_format,
                data_format,
                data,
            ),
            RenderCommand::SetTexture2DAnisotropy { handle, factor } => {
                self.cmd_set_texture_2d_anisotropy(handle, factor);
            }
            RenderCommand::SetTexture2DAnisotropyByResource { id, factor } => {
                self.cmd_set_texture_2d_anisotropy_by_resource(id, factor);
            }
            RenderCommand::SetTexture2DMipRangeByResource {
                id,
                min_level,
                max_level,
            } => {
                self.cmd_set_texture_2d_mip_range_by_resource(id, min_level, max_level);
            }
            RenderCommand::SetTexel1DByResource { id, x, color } => {
                self.cmd_set_texel_1d_by_resource(id, x, color);
            }
            RenderCommand::SetTexel2DByResource { id, x, y, color } => {
                self.cmd_set_texel_2d_by_resource(id, x, y, color);
            }
            RenderCommand::SetTextureMagFilterByResource { id, filter } => {
                self.cmd_set_texture_mag_filter_by_resource(id, filter);
            }
            RenderCommand::SetTextureMinFilterByResource { id, filter } => {
                self.cmd_set_texture_min_filter_by_resource(id, filter);
            }
            RenderCommand::SetTextureWrapModeByResource { id, mode } => {
                self.cmd_set_texture_wrap_mode_by_resource(id, mode);
            }
            RenderCommand::GenerateMipmapByResource { id } => {
                self.cmd_generate_mipmap_by_resource(id)
            }
            RenderCommand::UpdateTexture1DDataByResource {
                id,
                width,
                internal_format,
                pixel_format,
                data_format,
                data,
            } => self.cmd_update_texture_1d_data_by_resource(
                id,
                width,
                internal_format,
                pixel_format,
                data_format,
                data,
            ),
            RenderCommand::UpdateTexture3DDataByResource {
                id,
                width,
                height,
                depth,
                internal_format,
                pixel_format,
                data_format,
                data,
            } => self.cmd_update_texture_3d_data_by_resource(
                id,
                width,
                height,
                depth,
                internal_format,
                pixel_format,
                data_format,
                data,
            ),
            RenderCommand::UpdateTextureCubeFaceDataByResource {
                id,
                face,
                level,
                size,
                internal_format,
                pixel_format,
                data_format,
                data,
            } => self.cmd_update_texture_cube_face_data_by_resource(
                id,
                face,
                level,
                size,
                internal_format,
                pixel_format,
                data_format,
                data,
            ),
            RenderCommand::CopyTexture2DFromFramebufferByResource {
                id,
                internal_format,
                width,
                height,
            } => self.cmd_copy_texture_2d_from_framebuffer_by_resource(
                id,
                internal_format,
                width,
                height,
            ),
            RenderCommand::ReadTexture1DData {
                id,
                pixel_format,
                data_format,
                reply_tx,
            } => {
                let data = self.cmd_read_texture_1d_data(id, pixel_format, data_format);
                let _ = reply_tx.send(data);
            }
            RenderCommand::ReadTexture2DData {
                id,
                pixel_format,
                data_format,
                reply_tx,
            } => {
                let data = self.cmd_read_texture_2d_data(id, pixel_format, data_format);
                let _ = reply_tx.send(data);
            }
            RenderCommand::ReadTexture3DData {
                id,
                pixel_format,
                data_format,
                reply_tx,
            } => {
                let data = self.cmd_read_texture_3d_data(id, pixel_format, data_format);
                let _ = reply_tx.send(data);
            }
            RenderCommand::ReadTextureCubeFaceData {
                id,
                face,
                level,
                pixel_format,
                data_format,
                reply_tx,
            } => {
                let data = self.cmd_read_texture_cube_face_data(
                    id,
                    face,
                    level,
                    pixel_format,
                    data_format,
                );
                let _ = reply_tx.send(data);
            }
            RenderCommand::SamplePixel2DByResource { id, x, y, reply_tx } => {
                let data = self.cmd_sample_pixel_2d_by_resource(id, x, y);
                let _ = reply_tx.send(data);
            }
            RenderCommand::ReadFramebufferPixels {
                x,
                y,
                width,
                height,
                reply_tx,
            } => {
                let data = self.cmd_read_framebuffer_pixels(x, y, width, height);
                let _ = reply_tx.send(data);
            }

            // === Framebuffer Operations ===
            RenderCommand::PushFramebuffer { id, width, height } => {
                self.cmd_push_framebuffer(id, width, height);
            }
            RenderCommand::PopFramebuffer => self.cmd_pop_framebuffer(),
            RenderCommand::FramebufferAttachTexture2D {
                attachment,
                texture,
                level,
            } => {
                self.cmd_framebuffer_attach_texture_2d(attachment, texture, level);
            }
            RenderCommand::FramebufferAttachTexture2DByResource {
                attachment,
                id,
                level,
            } => {
                self.cmd_framebuffer_attach_texture_2d_by_resource(attachment, id, level);
            }
            RenderCommand::FramebufferAttachTexture3D {
                attachment,
                texture,
                layer,
                level,
            } => self.cmd_framebuffer_attach_texture_3d(attachment, texture, layer, level),
            RenderCommand::FramebufferAttachTexture3DByResource {
                attachment,
                id,
                layer,
                level,
            } => self.cmd_framebuffer_attach_texture_3d_by_resource(attachment, id, layer, level),
            RenderCommand::FramebufferAttachTextureCube {
                attachment,
                texture,
                face,
                level,
            } => self.cmd_framebuffer_attach_texture_cube(attachment, texture, face, level),
            RenderCommand::FramebufferAttachTextureCubeByResource {
                attachment,
                id,
                face,
                level,
            } => self.cmd_framebuffer_attach_texture_cube_by_resource(attachment, id, face, level),
            RenderCommand::SetDrawBuffers { count } => self.cmd_set_draw_buffers(count),
            RenderCommand::BindFramebuffer { handle } => self.cmd_bind_framebuffer(handle),
            RenderCommand::BindDefaultFramebuffer => self.cmd_bind_default_framebuffer(),
            RenderCommand::Clear { color, depth } => self.cmd_clear(color, depth),

            // === Mesh Operations ===
            RenderCommand::BindMesh { vao } => self.cmd_bind_mesh(vao),
            RenderCommand::BindMeshByResource { id } => self.cmd_bind_mesh_by_resource(id),
            RenderCommand::UnbindMesh => self.cmd_unbind_mesh(),

            // === Drawing Operations ===
            RenderCommand::DrawMesh {
                vao,
                index_count,
                primitive,
            } => {
                self.draw_mesh_calls_this_frame += 1;
                self.cmd_draw_mesh(vao, index_count, primitive);
            }
            RenderCommand::DrawMeshInstanced {
                vao,
                index_count,
                instance_count,
                primitive,
            } => {
                self.draw_instanced_calls_this_frame += 1;
                self.cmd_draw_mesh_instanced(vao, index_count, instance_count, primitive);
            }
            RenderCommand::DrawMeshByResource {
                id,
                index_count,
                primitive,
            } => {
                self.draw_mesh_calls_this_frame += 1;
                self.cmd_draw_mesh_by_resource(id, index_count, primitive);
            }
            RenderCommand::DrawMeshInstancedByResource {
                id,
                index_count,
                instance_count,
                primitive,
            } => {
                self.draw_instanced_calls_this_frame += 1;
                self.cmd_draw_mesh_instanced_by_resource(
                    id,
                    index_count,
                    instance_count,
                    primitive,
                );
            }
            RenderCommand::DrawInstancedWithData {
                mesh_id,
                index_count,
                instances,
                primitive,
            } => {
                self.draw_instanced_calls_this_frame += 1;
                self.instanced_data_items_this_frame += instances.len() as u64;
                self.cmd_draw_instanced_with_data(mesh_id, index_count, instances, primitive);
            }
            RenderCommand::DrawInstancedIndices {
                mesh_id,
                index_count,
                indices,
                primitive,
            } => {
                self.draw_instanced_calls_this_frame += 1;
                self.instanced_data_items_this_frame += indices.len() as u64;
                self.cmd_draw_instanced_indices(mesh_id, index_count, indices, primitive);
            }
            RenderCommand::DrawImmediate {
                primitive,
                vertices,
            } => {
                self.draw_immediate_calls_this_frame += 1;
                self.immediate_vertices_this_frame += vertices.len() as u64;
                self.cmd_draw_immediate(primitive, &vertices);
            }

            // === Resource Creation ===
            RenderCommand::CreateShader {
                id,
                vertex_src,
                fragment_src,
                reply_tx,
            } => {
                let data = self.cmd_create_shader(id, vertex_src, fragment_src);
                let _ = reply_tx.send(data);
            }
            RenderCommand::GetUniformLocationByResource { id, name, reply_tx } => {
                let data = self.cmd_get_uniform_location_by_resource(id, name);
                let _ = reply_tx.send(data);
            }
            RenderCommand::ReloadShader {
                shader_key,
                vertex_src,
                fragment_src,
            } => {
                reply = self.cmd_reload_shader(&shader_key, &vertex_src, &fragment_src);
            }
            RenderCommand::CreateTexture1D {
                id,
                width,
                format,
                data,
            } => {
                self.cmd_create_texture_1d(id, width, format, data);
            }
            RenderCommand::CreateTexture2D {
                id,
                width,
                height,
                format,
                data,
            } => {
                self.cmd_create_texture_2d(id, width, height, format, data);
            }
            RenderCommand::CreateTexture3D {
                id,
                width,
                height,
                depth,
                format,
                data,
            } => {
                self.cmd_create_texture_3d(id, width, height, depth, format, data);
            }
            RenderCommand::CreateTextureCube { id, size, format } => {
                self.cmd_create_texture_cube(id, size, format);
            }
            RenderCommand::CreateMesh {
                id,
                vertices,
                indices,
                vertex_format,
            } => self.cmd_create_mesh(id, vertices, indices, vertex_format),
            RenderCommand::DestroyResources { ids } => self.cmd_destroy_resource(&ids),

            // === Uniform Buffer Objects ===
            RenderCommand::CreateCameraUBO => self.cmd_create_camera_ubo(),
            RenderCommand::UpdateCameraUBO { data } => self.cmd_update_camera_ubo(&data[..]),
            RenderCommand::CreateMaterialUBO => self.cmd_create_material_ubo(),
            RenderCommand::UpdateMaterialUBO { data } => self.cmd_update_material_ubo(&data),
            RenderCommand::CreateLightUBO => self.cmd_create_light_ubo(),
            RenderCommand::UpdateLightUBO { data } => self.cmd_update_light_ubo(&data),

            // === Window Operations ===
            RenderCommand::Resize { width, height } => self.cmd_resize(width, height),

            RenderCommand::SetPresentMode { mode } => self.cmd_set_present_mode(mode),

            // === Window operations ===
            RenderCommand::SwapBuffers => {
                reply = self.cmd_swap_buffers();
            }

            // === Synchronization ===
            RenderCommand::Flush => self.cmd_flush(),

            RenderCommand::Fence { fence_id } => {
                reply = CommandReply::Fence(fence_id);
            }

            RenderCommand::PacingFence { fence_id } => {
                reply = CommandReply::PacingFence(fence_id);
            }

            RenderCommand::Shutdown => {
                // Handled by the caller's loop
            }
        }

        reply
    }
}

impl Default for WgpuCommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test-local stand-in for the engine's (now private) `GLSLCode`
    /// preprocessor: expands `#include` through `loader`, drops `#autovar`.
    struct GLSLCode {
        code: String,
    }

    impl GLSLCode {
        fn preprocess_with_loader(code: &str, loader: &mut dyn FnMut(&str) -> String) -> Self {
            let mut out = String::new();
            for line in code.lines() {
                if let Some(include_val) = line.strip_prefix("#include ") {
                    let inc = Self::preprocess_with_loader(
                        &loader(&format!("include/{include_val}")),
                        loader,
                    );
                    out += &inc.code;
                    out += "
";
                } else if line.starts_with("#autovar ") {
                } else {
                    out += line;
                    out += "
";
                }
            }
            Self { code: out }
        }
    }
    use crate::render::TexFormat;

    #[test]
    fn blend_mode_matches_gl_semantics() {
        // Disabled = no blend state
        assert!(WgpuCommandExecutor::blend_mode_to_wgpu(BlendMode::Disabled).is_none());
        // Additive: (ONE, ONE) for both color and alpha
        let add = WgpuCommandExecutor::blend_mode_to_wgpu(BlendMode::Additive).unwrap();
        assert_eq!(add.color.src_factor, wgpu::BlendFactor::One);
        assert_eq!(add.color.dst_factor, wgpu::BlendFactor::One);
        assert_eq!(add.alpha.src_factor, wgpu::BlendFactor::One);
        // Alpha: color (SRC_ALPHA, ONE_MINUS_SRC_ALPHA), alpha (ONE, ONE_MINUS_SRC_ALPHA)
        let alpha = WgpuCommandExecutor::blend_mode_to_wgpu(BlendMode::Alpha).unwrap();
        assert_eq!(alpha.color.src_factor, wgpu::BlendFactor::SrcAlpha);
        assert_eq!(alpha.color.dst_factor, wgpu::BlendFactor::OneMinusSrcAlpha);
        assert_eq!(alpha.alpha.src_factor, wgpu::BlendFactor::One);
        // PreMultAlpha: (ONE, ONE_MINUS_SRC_ALPHA)
        let premult = WgpuCommandExecutor::blend_mode_to_wgpu(BlendMode::PreMultAlpha).unwrap();
        assert_eq!(premult.color.src_factor, wgpu::BlendFactor::One);
        assert_eq!(
            premult.color.dst_factor,
            wgpu::BlendFactor::OneMinusSrcAlpha
        );
    }

    #[test]
    fn cull_face_maps_directly() {
        assert_eq!(WgpuCommandExecutor::cull_face_to_wgpu(CullFace::None), None);
        assert_eq!(
            WgpuCommandExecutor::cull_face_to_wgpu(CullFace::Back),
            Some(wgpu::Face::Back)
        );
        assert_eq!(
            WgpuCommandExecutor::cull_face_to_wgpu(CullFace::Front),
            Some(wgpu::Face::Front)
        );
    }

    #[test]
    fn primitive_mapping_covers_all_variants() {
        assert_eq!(
            WgpuCommandExecutor::primitive_to_wgpu(CmdPrimitiveType::Points),
            Some(wgpu::PrimitiveTopology::PointList)
        );
        assert_eq!(
            WgpuCommandExecutor::primitive_to_wgpu(CmdPrimitiveType::Lines),
            Some(wgpu::PrimitiveTopology::LineList)
        );
        assert_eq!(
            WgpuCommandExecutor::primitive_to_wgpu(CmdPrimitiveType::LineStrip),
            Some(wgpu::PrimitiveTopology::LineStrip)
        );
        assert_eq!(
            WgpuCommandExecutor::primitive_to_wgpu(CmdPrimitiveType::Triangles),
            Some(wgpu::PrimitiveTopology::TriangleList)
        );
        assert_eq!(
            WgpuCommandExecutor::primitive_to_wgpu(CmdPrimitiveType::TriangleStrip),
            Some(wgpu::PrimitiveTopology::TriangleStrip)
        );
        // Quads are drawn as triangles in GL; keep the same mapping.
        assert_eq!(
            WgpuCommandExecutor::primitive_to_wgpu(CmdPrimitiveType::Quads),
            Some(wgpu::PrimitiveTopology::TriangleList)
        );
        // TriangleFan has no wgpu topology.
        assert_eq!(
            WgpuCommandExecutor::primitive_to_wgpu(CmdPrimitiveType::TriangleFan),
            None
        );
    }

    #[test]
    fn filter_mapping_covers_all_variants() {
        use crate::render::tex_filter::TexFilter;
        let (mag, min, mip) = WgpuCommandExecutor::filter_to_wgpu(TexFilter::LinearMipLinear);
        assert_eq!(
            (mag, min, mip),
            (
                wgpu::FilterMode::Linear,
                wgpu::FilterMode::Linear,
                wgpu::FilterMode::Linear,
            )
        );
        let (mag, min, mip) = WgpuCommandExecutor::filter_to_wgpu(TexFilter::PointMipLinear);
        assert_eq!(
            (mag, min, mip),
            (
                wgpu::FilterMode::Nearest,
                wgpu::FilterMode::Nearest,
                wgpu::FilterMode::Linear,
            )
        );
        let (mag, min, mip) = WgpuCommandExecutor::filter_to_wgpu(TexFilter::Point);
        assert_eq!(
            (mag, min, mip),
            (
                wgpu::FilterMode::Nearest,
                wgpu::FilterMode::Nearest,
                wgpu::FilterMode::Nearest,
            )
        );
    }

    #[test]
    fn wrap_mode_mapping_covers_all_variants() {
        use crate::render::tex_wrap_mode::TexWrapMode;
        assert_eq!(
            WgpuCommandExecutor::wrap_mode_to_wgpu(TexWrapMode::Clamp),
            wgpu::AddressMode::ClampToEdge
        );
        assert_eq!(
            WgpuCommandExecutor::wrap_mode_to_wgpu(TexWrapMode::MirrorRepeat),
            wgpu::AddressMode::MirrorRepeat
        );
        assert_eq!(
            WgpuCommandExecutor::wrap_mode_to_wgpu(TexWrapMode::Repeat),
            wgpu::AddressMode::Repeat
        );
        // MirrorClamp has no wgpu equivalent; falls back to ClampToEdge.
        assert_eq!(
            WgpuCommandExecutor::wrap_mode_to_wgpu(TexWrapMode::MirrorClamp),
            wgpu::AddressMode::ClampToEdge
        );
    }

    #[test]
    fn tex_format_mapping_round_trips() {
        assert_eq!(
            WgpuCommandExecutor::tex_format_to_wgpu(TexFormat::RGBA8),
            Some(wgpu::TextureFormat::Rgba8Unorm)
        );
        assert_eq!(
            WgpuCommandExecutor::tex_format_to_wgpu(TexFormat::RGBA32F),
            Some(wgpu::TextureFormat::Rgba32Float)
        );
        assert_eq!(
            WgpuCommandExecutor::tex_format_to_wgpu(TexFormat::R32F),
            Some(wgpu::TextureFormat::R32Float)
        );
        assert_eq!(
            WgpuCommandExecutor::tex_format_to_wgpu(TexFormat::Depth24),
            Some(wgpu::TextureFormat::Depth24Plus)
        );
        // RGB8 has no wgpu equivalent
        assert_eq!(
            WgpuCommandExecutor::tex_format_to_wgpu(TexFormat::RGB8),
            None
        );
    }

    #[test]
    fn r32f_uses_filterable_rgba16f_backing_and_decodes_samples() {
        let (format, bpp, converted) =
            WgpuCommandExecutor::tex_format_to_wgpu_with_bpp(TexFormat::R32F);
        assert_eq!(format, wgpu::TextureFormat::Rgba16Float);
        assert_eq!(bpp, 4);
        assert!(converted);

        let source = 0.75f32.to_le_bytes();
        let encoded = WgpuCommandExecutor::f32_to_rgba16f_bytes(&source);
        assert_eq!(
            encoded,
            vec![0x00, 0x3a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x3c]
        );
        assert_eq!(
            WgpuCommandExecutor::decode_sample_pixel(wgpu::TextureFormat::Rgba16Float, &encoded),
            [191, 0, 0, 255]
        );
    }

    #[test]
    fn vertex_attributes_follow_gl_offsets() {
        let fmt = VertexFormat::default(); // pos + normal + uv, stride 32
        let attrs = WgpuCommandExecutor::vertex_attributes(&fmt);
        assert_eq!(attrs.len(), 3);
        assert_eq!(attrs[0].offset, 0); // pos
        assert_eq!(attrs[1].offset, 12); // normal
        assert_eq!(attrs[2].offset, 24); // uv
        assert_eq!(attrs[2].format, wgpu::VertexFormat::Float32x2);

        let with_color = VertexFormat {
            has_position: true,
            has_normal: true,
            has_uv: true,
            has_color: true,
            stride: 48,
        };
        let attrs = WgpuCommandExecutor::vertex_attributes(&with_color);
        assert_eq!(attrs.len(), 4);
        assert_eq!(attrs[3].offset, 32); // color
        assert_eq!(attrs[3].format, wgpu::VertexFormat::Float32x4);
    }

    #[test]
    fn state_commands_track_state() {
        let mut ex = WgpuCommandExecutor::new();
        ex.execute(RenderCommand::SetViewport {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
        });
        ex.execute(RenderCommand::SetBlendMode(BlendMode::Additive));
        ex.execute(RenderCommand::SetCullFace(CullFace::Back));
        ex.execute(RenderCommand::SetDepthTest(true));
        ex.execute(RenderCommand::SetWireframe(true));
        assert_eq!(ex.viewport, Some((0, 0, 800, 600)));
        assert_eq!(ex.blend_mode, BlendMode::Additive);
        assert_eq!(ex.cull_face, CullFace::Back);
        assert!(ex.depth_test);
        assert!(ex.wireframe);
    }

    #[test]
    fn named_uniforms_stage_by_name() {
        let mut ex = WgpuCommandExecutor::new();
        let name: Arc<str> = Arc::from("u_time");
        ex.execute(RenderCommand::SetUniformFloatByName {
            name: name.clone(),
            value: 1.5,
        });
        assert!(matches!(
            ex.named_uniforms.get(&name),
            Some(UniformValue::Float(v)) if *v == 1.5
        ));
    }

    #[test]
    fn ubo_staging_keeps_latest_bytes() {
        let mut ex = WgpuCommandExecutor::new();
        ex.execute(RenderCommand::CreateCameraUBO);
        ex.execute(RenderCommand::UpdateCameraUBO {
            data: Box::new([7u8; 288]),
        });
        assert_eq!(ex.camera_ubo.as_deref(), Some(&[7u8; 288][..]));
        // update before create warns but does not panic
        ex.execute(RenderCommand::UpdateLightUBO { data: [1u8; 32] });
        assert!(ex.light_ubo.is_none());
    }

    #[test]
    fn texture_slot_bookkeeping() {
        let mut ex = WgpuCommandExecutor::new();
        ex.execute(RenderCommand::BindTexture2D {
            slot: 2,
            handle: GpuHandle(42),
        });
        assert_eq!(ex.bound_textures[2], Some(GpuHandle(42)));
        ex.execute(RenderCommand::UnbindTexture { slot: 2 });
        assert_eq!(ex.bound_textures[2], None);
    }

    #[test]
    fn stats_count_draws_and_state_changes() {
        let mut ex = WgpuCommandExecutor::new();
        ex.execute(RenderCommand::SetDepthTest(true));
        ex.execute(RenderCommand::SetDepthTest(false));
        ex.execute(RenderCommand::SetViewport {
            x: 0,
            y: 0,
            width: 1,
            height: 1,
        });
        assert_eq!(ex.stats.commands_processed, 3);
        assert_eq!(ex.stats.state_changes, 3); // SetViewport counts too
        assert_eq!(ex.stats.draw_calls, 0);
    }

    /// Compile EVERY shader stage in res/shader through naga's GLSL frontend
    /// with a real wgpu device, plus an end-to-end CreateShader/ReloadShader
    /// round trip through the command surface. Ignored by default (opens a
    /// real window + needs GPU); run with:
    /// `cargo test -p phx --lib all_shaders_compile_through_naga -- --ignored --nocapture`
    #[test]
    fn probe_skybox_fragment_module() {
        // Load + preprocess the skybox fragment exactly like the engine.
        let res_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../res")
            .canonicalize()
            .expect("resolve res root");
        eprintln!("DBG res_root = {}", res_root.display());
        let fs = GLSLCode::preprocess_with_loader(
            &std::fs::read_to_string(res_root.join("shader/fragment/skybox.glsl")).unwrap(),
            &mut |name| {
                // the preprocess passes "include/<name>"; the engine's loader
                // appends ".glsl"
                std::fs::read_to_string(res_root.join(format!("shader/{name}.glsl")))
                    .unwrap_or_default()
            },
        );
        // Replicate compile_shader_pair: vs_max -> fs offset.
        let vs = GLSLCode::preprocess_with_loader(
            &std::fs::read_to_string(res_root.join("shader/vertex/farplane.glsl")).unwrap(),
            &mut |name| {
                std::fs::read_to_string(res_root.join(format!("shader/{name}.glsl")))
                    .unwrap_or_default()
            },
        );
        let vs_adapted = WgpuCommandExecutor::adapt_glsl_for_naga(&vs.code);
        let vs_max = WgpuCommandExecutor::max_injected_binding(&vs_adapted);
        eprintln!("DBG vs_max = {vs_max}");
        eprintln!("--- ADAPTED VS binding lines ---");
        for line in vs_adapted.lines().filter(|l| l.contains("layout(binding")) {
            eprintln!("VS: {line}");
        }
        let adapted = WgpuCommandExecutor::adapt_glsl_for_naga_with_offset(
            &fs.code,
            vs_max.saturating_sub(9),
        );
        eprintln!("--- ADAPTED FS binding lines ---");
        for line in adapted.lines().filter(|l| l.contains("layout(binding")) {
            eprintln!("FS: {line}");
        }
        let refl = WgpuCommandExecutor::build_reflection(&vs_adapted, &adapted);
        eprintln!(
            "DBG refl uniforms=[{}] samplers=[{}]",
            refl.uniforms
                .iter()
                .map(|u| format!("{}@{}", u.name, u.binding))
                .collect::<Vec<_>>()
                .join(","),
            refl.samplers
                .iter()
                .map(|s| format!("{}@{}/{}", s.name, s.tex_binding, s.samp_binding))
                .collect::<Vec<_>>()
                .join(",")
        );
        let mut frontend = wgpu::naga::front::glsl::Frontend::default();
        match frontend.parse(
            &wgpu::naga::front::glsl::Options::from(wgpu::naga::ShaderStage::Fragment),
            &adapted,
        ) {
            Ok(module) => {
                for (h, var) in module.global_variables.iter() {
                    eprintln!(
                        "GLOBAL {} name={:?} space={:?} binding={:?}",
                        h.index(),
                        var.name,
                        var.space,
                        var.binding
                    );
                }
                let mut validator = wgpu::naga::valid::Validator::new(
                    wgpu::naga::valid::ValidationFlags::all(),
                    wgpu::naga::valid::Capabilities::all(),
                );
                match validator.validate(&module) {
                    Ok(_) => eprintln!("VALIDATOR: OK"),
                    Err(e) => eprintln!("VALIDATOR: FAIL {e}"),
                }
            }
            Err(e) => eprintln!("PARSE FAIL: {e}"),
        }
    }

    #[test]
    #[ignore = "opens a real window + needs GPU; run explicitly"]
    #[allow(deprecated)] // EventLoop::create_window in the smoke pattern
    fn probe_ui_vertex_adapt() {
        let res_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../res")
            .canonicalize()
            .expect("resolve res root");
        let code = std::fs::read_to_string(res_root.join("shader/vertex/ui.glsl")).unwrap();
        let adapted = WgpuCommandExecutor::adapt_glsl_for_naga(&code);
        eprintln!("--- ADAPTED UI ---");
        eprintln!("{adapted}");
        eprintln!("--- END ---");
        let mut frontend = wgpu::naga::front::glsl::Frontend::default();
        match frontend.parse(
            &wgpu::naga::front::glsl::Options::from(wgpu::naga::ShaderStage::Vertex),
            &adapted,
        ) {
            Ok(_) => eprintln!("PARSE OK"),
            Err(e) => eprintln!("PARSE FAIL: {e}"),
        }
    }

    #[test]
    fn probe_farplane_varyings() {
        let res_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../res")
            .canonicalize()
            .expect("resolve res root");
        // Preprocess farplane.glsl the same way the engine does (includes).
        let loader = |name: &str| -> String {
            let path = res_root.join(format!("shader/include/{name}.glsl"));
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("include {name}: {e}"))
        };
        let mut code =
            std::fs::read_to_string(res_root.join("shader/vertex/farplane.glsl")).unwrap();
        let mut guard = 0;
        while code.contains("#include") && guard < 20 {
            guard += 1;
            let lines: Vec<String> = code.lines().map(|l| l.to_string()).collect();
            for line in &lines {
                if let Some(name) = line.trim().strip_prefix("#include ") {
                    let rep = loader(name);
                    code = code.replace(line, &rep);
                }
            }
        }
        let adapted = WgpuCommandExecutor::adapt_glsl_for_naga(&code);
        for l in adapted.lines() {
            if l.contains("out ") && l.contains("layout(location") {
                eprintln!("VS OUT: {l}");
            }
        }
        // starbg fragment
        let mut fcode =
            std::fs::read_to_string(res_root.join("shader/fragment/starbg.glsl")).unwrap();
        let mut guard = 0;
        while fcode.contains("#include") && guard < 30 {
            guard += 1;
            let flines: Vec<String> = fcode.lines().map(|l| l.to_string()).collect();
            for line in &flines {
                if let Some(name) = line.trim().strip_prefix("#include ") {
                    let path = if name.contains('/') {
                        res_root.join(format!("shader/{name}.glsl"))
                    } else {
                        res_root.join(format!("shader/include/{name}.glsl"))
                    };
                    let rep = std::fs::read_to_string(&path).unwrap();
                    fcode = fcode.replace(line, &rep);
                }
            }
        }
        let fadapted = WgpuCommandExecutor::adapt_glsl_for_naga_with_offset(&fcode, 1);
        for l in fadapted.lines() {
            if l.contains("in ") && l.contains("layout(location") {
                eprintln!("FS IN: {l}");
            }
        }
    }

    #[test]
    #[ignore = "opens a real window + needs GPU; run explicitly"]
    #[allow(deprecated)]
    fn all_shaders_compile_through_naga() {
        use crate::window::{PresentMode, WgpuRenderer};
        use std::path::{Path, PathBuf};

        // Window + device (same pattern as the smoke test).
        #[cfg(target_os = "windows")]
        let event_loop = {
            use winit::platform::windows::EventLoopBuilderExtWindows;
            winit::event_loop::EventLoopBuilder::new()
                .with_any_thread(true)
                .build()
                .expect("create winit event loop")
        };
        #[cfg(not(target_os = "windows"))]
        let event_loop = {
            use winit::event_loop::EventLoop;
            EventLoop::new().expect("create winit event loop")
        };
        let window = event_loop
            .create_window(
                winit::window::WindowAttributes::default()
                    .with_title("phx wgpu shader compile test")
                    .with_inner_size(winit::dpi::LogicalSize::new(64.0, 64.0)),
            )
            .expect("create winit window");
        let renderer =
            WgpuRenderer::new(&window, PresentMode::Vsync, 64, 64).expect("create wgpu renderer");
        let mut ex = WgpuCommandExecutor::with_device(
            Some(renderer.device.clone()),
            Some(renderer.queue.clone()),
        );

        // Repo root: engine/lib/phx -> up three levels.
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .expect("repo root");
        let res_shader = root.join("res").join("shader");
        assert!(res_shader.exists(), "res/shader missing at {res_shader:?}");

        // Same include semantics as GLSLCode::load: "include/vertex" ->
        // <res_shader>/include/vertex.glsl.
        let mut loader = |name: &str| -> String {
            let path = res_shader.join(format!("{name}.glsl"));
            std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("cannot load include {path:?}: {e}"))
        };

        let mut failures: Vec<String> = Vec::new();
        let mut total = 0usize;

        // The Z: share serves PARTIAL directory listings (25/135 files
        // observed live), so enumerate via git: the canonical, deterministic
        // file set even on the broken share.
        let ls = std::process::Command::new("git")
            .args(["ls-files", "res/shader/vertex", "res/shader/fragment"])
            .current_dir(root)
            .output()
            .expect("git ls-files");
        assert!(ls.status.success(), "git ls-files failed");
        let mut files: Vec<PathBuf> = String::from_utf8_lossy(&ls.stdout)
            .lines()
            .filter(|l| l.ends_with(".glsl"))
            .map(|l| root.join(l))
            .collect();
        // Documented exclusions (dead shaders, referenced by NO script; both
        // fail in the GL path too):
        //  - fragment/material/uv_metal.glsl: pre-existing source bug
        //    (unbalanced parens at line 30, `exp(-sqrt(...)` never closed).
        //  - fragment/ptracer.glsl: needs an engine-provided `SCENE_DESC`
        //    define the engine never supplies (the GLSL is a template).
        let excluded = [
            "res/shader/fragment/material/uv_metal.glsl",
            "res/shader/fragment/ptracer.glsl",
        ];
        files.retain(|f| {
            let rel = f
                .strip_prefix(root)
                .unwrap_or(f)
                .to_string_lossy()
                .replace('\\', "/");
            !excluded.contains(&rel.as_ref())
        });
        files.sort();
        assert!(
            files.len() >= 130,
            "expected 130+ tracked shaders, got {}",
            files.len()
        );
        println!("excluded (documented): {excluded:?}");

        for file in files {
            let rel = file
                .strip_prefix(&res_shader)
                .expect("shader under res/shader")
                .to_string_lossy()
                .replace('\\', "/");
            let stage = if rel.starts_with("vertex/") {
                wgpu::naga::ShaderStage::Vertex
            } else {
                wgpu::naga::ShaderStage::Fragment
            };
            let src = std::fs::read_to_string(&file).expect("read shader");
            let pre = GLSLCode::preprocess_with_loader(&src, &mut loader);
            total += 1;
            if let Err(e) = ex.compile_glsl_stage(stage, &pre.code) {
                failures.push(format!("{rel}: {e}"));
            }
        }

        // End-to-end through the command surface: create + reload one pair.
        let vs_src = std::fs::read_to_string(res_shader.join("vertex").join("wvp.glsl"))
            .expect("read wvp vertex");
        let fs_src = std::fs::read_to_string(res_shader.join("fragment").join("simple_color.glsl"))
            .expect("read simple_color fragment");
        let vs = GLSLCode::preprocess_with_loader(&vs_src, &mut loader);
        let fs = GLSLCode::preprocess_with_loader(&fs_src, &mut loader);

        let (create_tx, create_rx) = crossbeam::channel::unbounded();
        let create_err = match ex.execute(RenderCommand::CreateShader {
            id: ResourceId(1),
            vertex_src: vs.code.clone(),
            fragment_src: fs.code.clone(),
            reply_tx: create_tx,
        }) {
            CommandReply::None => create_rx.recv().unwrap_or_else(|e| Some(e.to_string())),
            other => Some(format!("unexpected create reply: {other:?}")),
        };
        let reload = ex.execute(RenderCommand::ReloadShader {
            shader_key: "[vs: vertex/wvp, fs: fragment/simple_color]".to_string(),
            vertex_src: vs.code.clone(),
            fragment_src: fs.code.clone(),
        });
        let hot_pairs = ex.hot_reloaded_shaders.len();

        println!(
            "naga compile: {total} shader stages, {} failures",
            failures.len()
        );
        for f in failures.iter().take(25) {
            println!("  FAIL {f}");
        }
        println!(
            "e2e: create_err={create_err:?} reload_success={} hot_pairs={hot_pairs}",
            matches!(&reload, CommandReply::ShaderReload(r) if r.error.is_none())
        );

        assert!(
            failures.is_empty(),
            "{} shader stages failed naga: {:?}",
            failures.len(),
            failures
        );
        assert!(create_err.is_none(), "create shader failed: {create_err:?}");
        assert!(
            matches!(&reload, CommandReply::ShaderReload(r) if r.error.is_none()),
            "reload failed: {reload:?}"
        );
        assert_eq!(hot_pairs, 1, "reload must store one hot pair");
    }

    #[test]
    #[ignore = "naga behavior probe"]
    fn probe_full_validate_sweep() {
        use std::path::Path;
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let res_shader = root.join("res").join("shader");
        let mut loader = |name: &str| -> String {
            let path = res_shader.join(format!("{name}.glsl"));
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("include {path:?}: {e}"))
        };
        let ls = std::process::Command::new("git")
            .args(["ls-files", "res/shader/vertex", "res/shader/fragment"])
            .current_dir(root)
            .output()
            .expect("git ls-files");
        let files: Vec<String> = String::from_utf8_lossy(&ls.stdout)
            .lines()
            .filter(|l| l.ends_with(".glsl"))
            .map(|l| l.to_string())
            .collect();
        println!("total files: {}", files.len());
        let mut ok = 0usize;
        let mut fail = 0usize;
        for f in &files {
            let stage = if f.starts_with("res/shader/vertex") {
                wgpu::naga::ShaderStage::Vertex
            } else {
                wgpu::naga::ShaderStage::Fragment
            };
            let src = std::fs::read_to_string(root.join(f)).expect("read");
            let pre = GLSLCode::preprocess_with_loader(&src, &mut loader);
            let adapted = WgpuCommandExecutor::adapt_glsl_for_naga(&pre.code);
            let mut front = wgpu::naga::front::glsl::Frontend::default();
            let module = match front.parse(&wgpu::naga::front::glsl::Options::from(stage), &adapted)
            {
                Ok(m) => m,
                Err(e) => {
                    fail += 1;
                    let mut detail = String::new();
                    for err in e.errors.iter().take(2) {
                        let ln = err
                            .location(&adapted)
                            .map(|l| l.line_number as usize)
                            .unwrap_or(0);
                        let line = adapted
                            .lines()
                            .nth(ln.saturating_sub(1))
                            .unwrap_or("<eof>")
                            .trim();
                        detail.push_str(&format!(" @L{ln}: {line}"));
                    }
                    println!("PARSE FAIL {f}: {e}{detail}");
                    continue;
                }
            };
            let mut validator = wgpu::naga::valid::Validator::new(
                wgpu::naga::valid::ValidationFlags::all(),
                wgpu::naga::valid::Capabilities::all(),
            );
            match validator.validate(&module) {
                Ok(_) => ok += 1,
                Err(e) => {
                    fail += 1;
                    let msg = e.to_string();
                    let first = msg.lines().next().unwrap_or("").to_string();
                    println!("VALIDATE FAIL {f}: {first}");
                    if first.contains("invalid") || first.contains("Entry point") {
                        let mut n = 0;
                        for l in msg.lines() {
                            println!("    V: {l}");
                            n += 1;
                            if n > 6 {
                                break;
                            }
                        }
                    }
                }
            }
        }
        println!("RESULT: {ok} ok, {fail} fail of {}", files.len());
    }
}
