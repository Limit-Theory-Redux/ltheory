# Render API v2 — wgpu-shaped interface on the GL renderer

Status: design. S2 (render passes and attachment views), S3 (binding model, pipelines, samplers, views, frame group), S4 (materials, scene list, uniform ring), S5 (fullscreen, post-processing, offscreen generation), S6 (immediate batching, UI, glyph atlas, and the removal of the legacy API), S7 (sampler/view completion), S8 (async readback), S9 (mip chains and texture kinds) and S10 (hot reload) are implemented. Companion to `wgpu-migration-gaps.md`,
`render-thread.md`, `batch-rendering.md` and `shader-system.md`.

**Strategy, already decided.** First the GL renderer and the Lua render
interface move, step by step, to wgpu-style objects (pipelines, bind groups,
passes, a uniform ring, samplers and views, async readback). Each step is
checked on GL. Only then does the backend change. No layer emulates the old
abstractions. `RenderState.Push*/Pop*`, `ShaderVar.Push*`,
`ShaderState`/`shader:setFloat(name)`, texture-unit `setTex*`,
`RenderTarget.Push`/`tex:push` and `Draw.Clear` are deleted once their call
sites have moved. `script/Legacy/**` and `*_outdated*` are not ported (see
§6 for the Legacy modules that active code still requires). Both
`Renderer` backends, threaded (default) and `immediate` (cargo feature),
keep working after every step.

Counts below are for active Lua only, which means excluding `script/Legacy/**`
and `*_outdated*`. They come from a grep of the current tree. Two files,
`Render/RenderPipeline.lua` and `Render/ImageFilter.lua`, have no active
requirers: the only users are `Examples/HmGui/ScrollArea.lua` and `_outdated` tests. They
make up a large share of some counts (marked **RP/IF**) and are proposed for
deletion, not porting.

---

## 1. Core objects and lifetimes

### 1.1 Object overview

| Object | Created | Lifetime / owner | Lua face | Replaces |
|---|---|---|---|---|
| `Shader` | `Shader.Load(vs, fs)` (as today, `Cache.Shader`) | `ResourceHandle` in `Rf<ShaderShared>`; reload swaps the program in place | `Shader` | unchanged, but loses `start/stop/set*/iSet*` |
| `ShaderLayout` | at link, from directives + reflection | owned by the shader and bumped on reload (`generation`) | `shader:blockType(name)` returns an FFI ctype | `#autovar`, uniform-location caches |
| `Pipeline` | `Pipeline.Get(desc)`, hashed and cached | immortal (bounded set), `PipelineId(u32)` | `PipelineId` value | `RenderState.Push*`, `shader:start()` |
| `Sampler` | `Sampler.Get(desc)`, hashed and cached | immortal, `SamplerId(u16)` | `SamplerId` value + `Samplers.*` presets | `tex:setMinFilter/setWrapMode/setAnisotropy` |
| `TexView` | value type, no GPU object on GL | `Copy` struct `{tex, dim, base_mip, mip_count, extent}` | S2 (attachment views): `tex:view()` / `tex:mipView(l)` / `vol:layerView(z)` / `vol:layerMipView(z, l)` / `cube:faceView(f)` / `cube:faceMipView(f, l)`. S3 adds sampling views (`tex:view{...}`). | `setMipRange`, `pushLevel`, `PushTex3D(layer)`, `bind_tex_cube(face)` |
| `BindGroup` (group 1) | `Material` creation | `ResourceHandle` held by `Material` | inside `Material` | `ShaderState` elems, material `setTex` replay |
| Transient bind groups (0, 2, 3) | per pass/draw from ring offsets + inline views | one frame | `pass:setInputs{...}`, `pass:alloc(T)` | `ShaderVar` stack, per-draw `setFloat`, `setTex2D('src')` |
| `RenderPassDesc` | once per pass kind, updated on resize | Lua-owned cdata | `RenderPassDesc` | `RenderTarget.Push`+`BindTex2D`+`Draw.Clear` |
| `RenderPass` (encoder) | `Renderer:beginPass(desc)` → `pass:finish()` | one at a time, main thread | `RenderPass` | FBO stack, viewport stack |
| `UniformRing` | at startup | `MAX_FRAMES_IN_FLIGHT` slots, main-thread allocator + render-thread buffers | implicit via `pass:alloc` | camera/light/material UBO rewrites, `SetUniform*` |
| `VertexRing` | at startup | same slotting | implicit via `Imm` | `DrawImmediate { vertices: Vec }` |
| `ReadbackTicket` | `Renderer:readAsync(view, rect, fmt)` | until `:free()` / GC | `ReadbackTicket` | `tex:sample()`, sync `Read*Data` |

### 1.2 Rust structs (new module `render/gpu/`, shared by both backends)

```rust
// ---- identifiers (main thread allocates, executor maps to GL/wgpu objects)
#[derive(Copy, Clone, PartialEq, Eq, Hash)] pub struct PipelineId(pub u32);
#[derive(Copy, Clone, PartialEq, Eq, Hash)] pub struct SamplerId(pub u16);
#[derive(Copy, Clone, PartialEq, Eq, Hash)] pub struct BindGroupId(pub u32);
#[derive(Copy, Clone, PartialEq, Eq, Hash)] pub struct BufferId(pub u32);

pub const MAX_FRAMES_IN_FLIGHT: usize = 3;   // moves to config.rs, used by pacing too
pub const UNIFORM_ALIGN: u32 = 256;          // max(GL UNIFORM_BUFFER_OFFSET_ALIGNMENT, wgpu min)
pub const MAX_COLOR_ATTACHMENTS: usize = 4;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PipelineDesc {
    pub shader: ResourceId,               // + shader generation in the cache key (S10)
    pub vertex: VertexLayout,             // Mesh(VertexFormat) | Imm3D | Imm2D | Fullscreen
                                          // | MeshInstanced(VertexFormat, InstanceLayout)
    pub topology: Topology,               // CmdPrimitiveType minus Quads (batcher triangulates)
    pub blend: BlendMode,                 // existing enum
    pub cull: CullFace,                   // existing enum
    pub depth: DepthState,                // { test: bool, write: bool, compare: CompareFn }
    pub color_formats: ArrayVec<TexFormat, MAX_COLOR_ATTACHMENTS>,
    pub depth_format: Option<TexFormat>,
    pub polygon: PolygonMode,             // Fill | Line (GL; wgpu needs POLYGON_MODE_LINE)
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SamplerDesc {
    pub min: TexFilter, pub mag: TexFilter, pub mip: MipFilter,
    pub wrap: [TexWrapMode; 3],
    pub anisotropy: u8,                   // 1 = off
    pub lod_min: u8, pub lod_max: u8,     // quantized clamps
    pub compare: Option<CompareFn>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TexView {
    pub tex: ResourceId,
    pub dim: ViewDim,                     // D1 | D2 | D3 | Cube | D2Layer(u16) | CubeFace(CubeFace)
    pub base_mip: u8,
    pub mip_count: u8,                    // 0 = all remaining
    pub extent: [u32; 2],                 // width, height of base_mip (added in S2 so a pass can size its viewport)
}

pub enum BindEntry {
    Uniform { buffer: BufferId, offset: u32, size: u32 },  // material params arena slice
    Texture { view: TexView, sampler: SamplerId },
}

// As built in S2: `LoadOp` is a plain C-like enum so it can be a Lua enum (`LoadOp.Clear`);
// the clear value lives in the attachment next to it.
pub struct ColorAttachment { pub view: TexView, pub load: LoadOp, pub clear: [f32; 4], pub store: StoreOp }
pub struct DepthAttachment { pub view: TexView, pub load: LoadOp, pub clear: f32, pub store: StoreOp }
pub enum LoadOp { Load, Clear, DontCare }
pub enum StoreOp { Store, Discard }

pub struct RenderPassDesc {
    pub label: Arc<str>,                  // profiler + stats category
    pub color: [Option<ColorAttachment>; MAX_COLOR_ATTACHMENTS],  // contiguous from 0
    pub depth: Option<DepthAttachment>,
    pub backbuffer: bool,                 // S2: explicit (`desc:backbuffer(w, h, load, r, g, b, a)`), no attachments
    pub back_color: (LoadOp, [f32; 4]),   // backbuffer load ops (`desc:backbufferDepth` sets back_depth)
    pub back_depth: (LoadOp, f32),
    pub extent: [u32; 2],                 // pass size, taken from the attachments (or given for the backbuffer)
}

/// Fixed per-draw block (group 2) for scene meshes. Other shaders declare their own group-2 block.
#[repr(C, align(16))]
pub struct DrawBlock {                    // 256 bytes = one UNIFORM_ALIGN stride
    pub m_world: [f32; 16],
    pub m_world_it: [f32; 16],
    pub draw_scale: [f32; 4],             // x = uniform scale (was `scale` auto-var)
    pub draw_user: [[f32; 4]; 7],         // material-defined per-draw values
}

/// Group 0, one per pass (ring-allocated), superset of today's CameraUboData.
#[repr(C, align(16))]
pub struct ViewBlock {
    pub m_view: [f32; 16], pub m_proj: [f32; 16],
    pub m_view_inv: [f32; 16], pub m_proj_inv: [f32; 16],
    pub eye: [f32; 4], pub star_dir: [f32; 4],
    pub m_proj_ui: [f32; 16],             // ortho(attachment size), was ShaderVar "mProjUI"
    pub m_world_view_ui: [f32; 16],       // was ShaderVar "mWorldViewUI" (UI/Graph.lua)
    pub viewport: [f32; 4],               // x, y, w, h in pixels
    pub time: [f32; 4],
}
```

Uniform ring. The main thread allocates and writes into staging, and the executor uploads:

```rust
pub struct RingOffset { pub buffer: u16, pub offset: u32 }   // buffer index within the frame slot

pub struct UniformRing {                  // lives in RendererData → identical for both backends
    slot: usize,                          // frame_index % MAX_FRAMES_IN_FLIGHT
    chunk_size: u32,                      // 256 KiB staging chunks, never reallocated
    open: Vec<u8>,                        // current chunk, capacity fixed → stable write pointers
    pending: Vec<RingChunk>,              // filled chunks not yet sent
    pool: Vec<Vec<u8>>,                   // recycled chunks (threaded: returned via channel)
    cursor: RingOffset,                   // per-slot head; slot grows by appending 4 MiB buffers
}
pub struct RingChunk { pub at: RingOffset, pub bytes: Vec<u8> }

impl UniformRing {
    /// Never flushes. A full chunk moves to `pending`, so earlier pointers stay valid
    /// until the next flush point (pass call / pass end / non-pass command).
    pub fn alloc(&mut self, size: u32) -> (RingOffset, *mut u8);
}
```

`VertexRing` (immediate/UI vertices, instance data, instanced index lists)
works the same way with stride alignment rather than 256 alignment.

### 1.3 Binding model

| Group | Contents | Set by | Changes | GL binding points / units |
|---|---|---|---|---|
| 0 frame/view | `ViewBlock` (ring, dynamic offset) + `envMap`, `irMap` (+1 spare) | pass begin (`Renderer:setCamera/setEnvironment` state) | per pass | UBO 0, units 0–2 |
| 1 material | `MaterialParams` block (material arena slice) + material textures | `Material` | on material change (sort key) | UBO 4, units 3–9 |
| 2 draw | `DrawBlock` or shader-declared draw/filter block (ring, dynamic offset); instance data texture | scene list / `pass:alloc` | per draw | UBO 8, units 10–11 |
| 3 pass inputs | textures produced earlier in the frame (`texDepth`, `texNormalMat`, `src`, ...) | `pass:setInputs` | per fullscreen draw | units 12–15 |

All sixteen units fit in the GL 3.3 minimum `MAX_TEXTURE_IMAGE_UNITS` (16,
which is also macOS's limit). Block bindings are `group*4 + k`, which fits the
GL 3.3 minimum of 36. wgpu uses `set = group`, with bindings numbered in
declaration order (§2).

### 1.4 Frame and pass lifecycle

```
Renderer:beginFrame()                 -> BeginFrame { slot }  (GL: client-wait slot fence)
Renderer:setCamera(view, proj, starDir) / setEnvironment(envMap, irMap)   (state, no command)
pass = Renderer:beginPass(desc)       -> BeginRenderPass(Box<RenderPassDesc>) + ViewBlock alloc
  pass:setPipeline / setMaterial / setInputs / draw* / submitScene / imm()...
                                         recorded into PassEncoder (main thread, Vec<PassCmd>)
                                         flushed as PassCommands{uploads, cmds} at 512 cmds,
                                         at finish, or before any non-pass command
pass:finish()                         -> PassCommands + EndRenderPass
end_frame_triple_buffered()           -> SwapBuffers + PacingFence (existing); GL inserts slot fence
```

Passes stream as Begin / chunks / End. They are not buffered whole. That
keeps the render thread busy while Lua is still recording, and on wgpu one
Begin/End pair becomes one `wgpu::RenderPass` (gap 13). Resource writes
(texture uploads, material param writes) are not allowed inside an open pass.
Debug builds assert this, because wgpu orders `queue.write_*` before the
whole submission.

### 1.5 RenderCommand delta

Added:

```rust
CreatePipeline { id: PipelineId, desc: Box<PipelineDesc> },
CreateSampler  { id: SamplerId, desc: SamplerDesc },
CreateBuffer   { id: BufferId, size: u32 },                         // material param arenas
WriteBuffer    { id: BufferId, offset: u32, data: Vec<u8> },
CreateBindGroup{ id: BindGroupId, shader: ResourceId, group: u8, entries: Box<[BindEntry]> },
BeginFrame     { slot: u8 },
BeginRenderPass(Box<RenderPassDesc>),
PassCommands   { uniforms: Vec<RingChunk>, vertices: Vec<RingChunk>, cmds: Vec<PassCmd> },
EndRenderPass,
CopyTexture    { src: TexView, dst: TexView, size: [u32; 3] },
GenerateMips   { tex: ResourceId },                                 // renamed GenerateMipmapByResource
ReadbackAsync  { src: TexView, rect: [u32; 4], format: TexFormat, slot: Arc<ReadbackSlot> },
ReadTextureSync{ src: TexView, rect: [u32; 4], format: TexFormat, reply_tx: Sender<Vec<u8>> },

pub enum PassCmd {
    SetPipeline(PipelineId),
    SetBindGroup { group: u8, id: BindGroupId },                    // group 1
    SetView(RingOffset),                                            // group 0 (re-alloc on setUiTransform)
    SetDraw(RingOffset),                                            // group 2 dynamic offset
    SetInputs(ArrayVec<(TexView, SamplerId), 4>),                   // group 3
    SetViewport([i32; 4]), SetScissor(Option<[i32; 4]>),
    DrawMesh { mesh: ResourceId, first_index: u32, index_count: u32 },
    DrawMeshInstanced { mesh: ResourceId, index_count: u32, instances: RingOffset, count: u32 },
    DrawInstancedIndices { mesh: ResourceId, index_count: u32, indices: RingOffset, count: u32 },
    DrawImm { first_vertex: u32, vertex_count: u32 },               // from VertexRing
    DrawFullscreen,
}
```

Deleted, roughly 75 of today's variants: `SetViewport`, `SetScissor`,
`EnableScissor`, `SetBlendMode`, `SetCullFace`, `SetDepthTest`,
`SetDepthWritable`, `SetWireframe`, `SetLineWidth`, `SetPointSize`, `BindShader*`,
`UnbindShader`, all 9 `SetUniform*` by location, all 9 `*ByName`,
`SetUniformMat4ByGenericName` (+ `GenericUniformName`), `SetInstanceUniforms`
(+ `InstanceUniformsCmd`), all 7 `BindTexture*`, `UnbindTexture`, every
`SetTexture*Filter/WrapMode/MipRange/Anisotropy*`, `GenerateMipmap2D`,
`UpdateTexture2DData` (GpuHandle form), `CopyTexture2DFromFramebufferByResource`,
`SamplePixel2DByResource`, `ReadFramebufferPixels`, `Read*Data` (collapsed into
`ReadTextureSync`), `PushFramebuffer`, `PopFramebuffer`, all 6
`FramebufferAttach*`, `SetDrawBuffers`, `BindFramebuffer`,
`BindDefaultFramebuffer`, `Clear`, `BindMesh*`, `UnbindMesh`, `DrawMesh*` (4),
`DrawInstancedWithData`, `DrawInstancedIndices` (moved into `PassCmd`),
`DrawImmediate`, `GetUniformLocationByResource`, `Create/UpdateCameraUBO`,
`Create/UpdateMaterialUBO` (dead today), `Create/UpdateLightUBO`. Every
`GpuHandle` variant goes, and `GpuHandle` goes with them.

Kept: `Create{Shader,Texture*,Mesh}`, `Update*DataByResource` (made
format-agnostic in S9), `DestroyResources`, `Resize`,
`SetPresentMode`, `SwapBuffers`, `Flush`, `Fence`, `PacingFence`, `Shutdown`.

### 1.6 Threaded and immediate backends

All new main-thread logic lives in `RendererData` or the new
`render/gpu/{pass_encoder,uniform_ring,vertex_ring,scene_list,imm_batcher,pipeline_cache}.rs`.
That covers id allocation, the pipeline and sampler hash caches, ring
allocation, the pass encoder, scene cull/sort and imm batching. Both
`renderer_threaded.rs` and `renderer_immediate.rs` reach it through
`r.data`. The only backend-specific parts are transport (`submit`) and
chunk recycling. Threaded mode returns used `Vec<u8>` chunks over a small
return channel; immediate mode hands them back inline after `execute`. All
GL work stays in `CommandExecutor`, which both backends share. A step that
deletes a `Renderer` method deletes it from both files in the same commit.
Every step's gate includes `cargo check --features immediate` and the
validation run on an immediate build (§5).

---

## 2. Uniform layout source of truth

**Decision: the shader source declares the layout. Reflection validates it and
generates the CPU-side types.** GLSL 330 has no `layout(binding=)`, so the
preprocessor in `GLSLCode::preprocess` (`shader.rs`) owns group and binding
assignment.

```glsl
// res/shader/fragment/material/planet.glsl (after S4)
#include fragment            // pulls view_block (group 0) + envMap/irMap
#include draw_block          // group 2: DrawBlock { mWorld, mWorldIT, drawScale, drawUser[7] }
#define origin  (mWorld[3].xyz)   // camera-relative translation: was a per-instance auto-var
#define rPlanet (drawScale.x)     // was a per-instance auto-var == rigid-body scale
#define time    (drawUser[0].x)   // CloudMotion time, per entity

#group 1
layout(std140) uniform MaterialParams {
    vec3 color1;   float heightMult;
    vec3 color2;   float oceanLevel;
    vec3 color3;   float rAtmo;
    vec3 color4;   float _pad0;
    vec3 starTint; float _pad1;
};
uniform samplerCube surface;
uniform samplerCube cloudCube;   // required by include scattering2
uniform sampler3D  cloudNoise;
```

Rules:

1. A `#group N` directive applies to the declarations that follow it until
   the next `#group`. Includes inherit the current group. Blocks and samplers
   are numbered in declaration order within their group. The preprocessor
   strips the directive for GL and records
   `ShaderLayout { groups: [ { blocks: [(name, binding)], textures: [(name, unit, dim)] } ] }`.
   For the wgpu/naga path it rewrites each declaration to
   `layout(std140, set=N, binding=B)`, plus the texture/sampler pair that the
   wgpu executor already splits combined samplers into.
2. Includes never declare loose uniforms. `scattering2.glsl` references
   `rPlanet`/`rAtmo` by name, and the including shader supplies them (block
   member or `#define`). From S6 on, `create_shader` rejects any active
   uniform that is neither a sampler nor in a block, with an error naming it.
   During S3–S5 it only warns.
3. At link, the GL executor calls `glGetActiveUniformBlockiv` and
   `glGetActiveUniformsiv(UNIFORM_OFFSET/TYPE/ARRAY_STRIDE/MATRIX_STRIDE)` to
   build `BlockLayout { name, size, members: [(name, ty, offset, count, stride)] }`.
   The wgpu executor builds the same thing from naga's module. A unit test
   compiles every shader in `res/shader` on both paths and asserts that the
   layouts are identical.
4. `ViewBlock` and `DrawBlock` are fixed `#[repr(C)]` Rust structs, and a
   startup assert checks their size and offsets against reflection.
   Material, filter and other shader-specific blocks are dynamic:
   `shader:blockType('MaterialParams')` returns a LuaJIT ctype that Rust
   generates from `BlockLayout`, as an anonymous
   `ffi.typeof("struct { Vec3f color1; float heightMult; ... float _pad0; ... }")`.
   Being anonymous lets it be regenerated on hot reload without colliding
   with `ffi.cdef`. Field types map to the existing FFI types: `float`,
   `int32_t`, `Vec2f`/`Vec3f`/`Vec4f`, `float[16]` for `mat4` (column-major,
   with `Matrix` copy helpers), and explicit `_padN` fields for std140 holes.

**Material params** go in a persistent block. Lua writes the typed cdata and
calls `mat:commit()`, which copies the struct bytes into a
`WriteBuffer{arena, offset}`. That happens once per change, never per draw.
Material params must stay constant for the frame. Anything that changes
between draws of the same material goes in the draw block.

**Per-draw data** is written in bulk. For scene meshes, Lua fills an FFI
array of transforms and items (§3b). Rust computes `mWorldIT` once per
transform rather than once per mesh, culls, sorts, and writes one `DrawBlock`
per surviving item straight into the ring. For other shaders
`pass:alloc(T)` returns a `T*` pointing into ring staging, Lua writes the
fields in place, and the next draw picks up the offset. That is zero copies
and one FFI call.

---

## 3. Lua interface

### 3a. Materials

`MaterialDefinition` (with `autoShaderVars`, `constShaderVars` and
`staticShaderVars`), `DynamicShaderVar`, `Shared/Rendering/Texture.lua`,
`UniformFuncs`, `UniformFuncDefs` and `ShaderVarFuncs` are replaced by
`MaterialType` and `Material`. A thin Rust `Material` holds the bind group,
its arena slice and its `PipelineDesc` template. The Lua wrapper holds the
params cdata and the per-draw callback.

```lua
-- Shared/Definitions/MaterialDefs.lua (after S4)
MaterialType {
    name     = "PlanetSurface",
    shader   = { "wvp", "material/planet" },
    state    = { blend = BlendMode.Disabled, cull = CullFace.Back, depthTest = true, depthWrite = true },
    defaults = { heightMult = 1.0, starTint = Vec3f(1.0, 0.5, 0.1) },  -- MaterialParams fields
    textures = {
        surface    = { sampler = Samplers.LinearMipClamp },            -- set per planet
        cloudCube  = { sampler = Samplers.LinearMipClamp },
        cloudNoise = { sampler = Samplers.LinearRepeat3D },
    },
    -- optional: fills DrawBlock.drawUser for this entity (called only for culled-in items)
    perDraw  = function(entity, user) user[0].x = entity:get(CloudMotion):getTime() end,
}

-- SolarSystemVisualizer:_materializePlanet (after S4)
local mat = Materials.PlanetSurface:instance()        -- own MaterialParams + bind group
local p = mat:params()                                -- typed cdata
p.color1, p.color2, p.color3, p.color4 = gen.color1, gen.color2, gen.color3, gen.color4
p.oceanLevel = gen.oceanLevel
p.rAtmo      = rb:getScale() * gen.atmoScale
mat:setTexture("surface", texSurface)                 -- default view, type-checked vs layout
mat:commit()                                          -- one WriteBuffer
```

The blend mode chooses the scene bucket (opaque/additive/alpha), as it
does today. The pipeline comes from the material's `state` plus the
target pass's attachment formats, through a lookup cached on the material
per pass signature. `PlanetTest`'s
`matRing:addStaticShaderVar(...)` ×7 become fields of PlanetRing's
`MaterialParams` written once, plus `mat:commit()` when the debug toggles
change.

### 3b. RenderCoreSystem scene passes

Before (`renderInOrder`, once per mesh): `sh:start()`, then
`applyMaterialVars` (Lua closures ending in `setFloat*`), then
`applyInstanceVars` (more closures, plus `UniformFuncs`), then
`mesh:draw()`. On top of that, `RenderingPass:start` pushes four
`RenderState`s and a `RenderTarget`.

After:

```lua
-- registerPasses / on resize: descriptors built once
self.opaqueDesc = RenderPassDesc.Create("Opaque")
self.opaqueDesc:color(0, b[buffer0]:view(), LoadOp.Clear, 0, 0, 0, 0)
self.opaqueDesc:color(1, b[buffer1]:view(), LoadOp.Clear, 0, 0, 0, 0)
self.opaqueDesc:color(2, b[zBufferL]:view(), LoadOp.Clear, 0, 0, 0, 0)
self.opaqueDesc:depth(b[zBuffer]:view(), LoadOp.Clear, 1.0)

-- buildPassLists: one SceneList per frame, entries reused in place (as today)
local list = self.scene                      -- Rust SceneList, FFI arrays inside
list:reset()
for entity in Registry:view(RenderComp) do
    local t = list:addTransform()            -- SceneTransform* { world[16], center, scale }
    rb:getToWorldMatrixInto(eye, t.world)    -- writes straight into the FFI array
    t.cx, t.cy, t.cz, t.scale = cx, cy, cz, scale
    for _, mm in ipairs(meshes) do
        list:addItem(t.index, mm.mesh, mm.material, meshOriginRadius(mm.mesh) * scale)
    end
end

-- render
local pass = Renderer:beginPass(self.opaqueDesc)
list:submit(pass, BlendMode.Disabled)        -- Rust: cull, sort, mWorldIT, perDraw, ring, cmds
self:renderFns(pass, BlendMode.Disabled)     -- skybox etc. draw through `pass`
pass:finish()
```

`SceneList::submit` merges `RenderBatch::cull_and_sort` with draw emission.
It frustum-culls, keeps the `radius < 0` "never cull" sentinel, and sorts by
`(pipeline, material, mesh)` except in the alpha bucket, which keeps insertion
order. It then calls the material's `perDraw` callback for survivors only:
one Lua callback per surviving item, and only for materials that define one.
That means culled entities still never have per-draw values computed, which
keeps the property `batch-rendering.md` cites for the cull-only design.
Finally it writes `DrawBlock`s and emits `SetPipeline`/`SetBindGroup`/`SetDraw`/`DrawMesh`.
This retires `beginBatch/addCullEntity/cullBatch`, the inert
`flushBatch`/`command_buffer` path, and the uncalled `InstanceBatch`.

Mid-pass state changes become separate pipelines. Deferred lighting's
additive directional and point lights (`RenderCoreSystem.lua:816/835`) use a
pipeline with `blend = Additive` inside the same pass as the global term,
whose pipeline has blend disabled.

### 3c. Fullscreen and post passes

```lua
-- Before: RenderCoreSystem:blur
dst:push()
shader:start()
shader:setFloat('variance', variance); shader:setFloat2('dir', dx, dy)
shader:setFloat2('size', size.x, size.y); shader:setInt('radius', radius)
shader:setTex2D('src', src)
Draw.Rect(0, 0, size.x, size.y)
shader:stop()
dst:pop()

-- After
local BlurParams = Cache.Shader('fullscreen', 'filter/blur'):blockType('Params')   -- once
local pass = Renderer:beginPass(self.blurDesc[dst])      -- color(0) = dst:view(), LoadOp.DontCare
pass:setPipeline(self.pipes.blur)                        -- Pipeline.Fullscreen(shader, RGBA16F)
pass:setInputs(src:view(), Samplers.LinearClamp)
local p = pass:alloc(BlurParams)
p.variance, p.dir.x, p.dir.y = variance, dx, dy
p.size.x, p.size.y, p.radius = size.x, size.y, radius
pass:drawFullscreen()
pass:finish()
```

`applyFilter` becomes a small helper with the same shape. Its target is
`buffer1:mipView(self.level)` and its input is `buffer0:mipView(self.level)`.
That replaces `pushLevel` and the `setMipRange` loop in `downsampleForPost`.
`present`/`presentAll` become one backbuffer pass with `pass:setViewport`
per quadrant. The y-flip that `Draw.Rect(x, y+sy, sx, -sy)` used to do moves
into `p.flipY`.

Deferred point lights. Before, this was `Renderer:updateLightUbo(...)` +
`setTex2D` ×2 + `Draw.Rect` per light, which meant one UBO rewrite per light.
After:

```lua
pass:setPipeline(self.pipes.pointLight)                -- blend = Additive
pass:setInputs(zBufferL:view(), Samplers.Point, buffer1:view(), Samplers.Point)  -- once
for _, light in ipairs(pointLights) do
    local p = pass:alloc(PointLightParams)             -- group 2, was LightUBO
    local rp = light.pos:relativeTo(eye)
    p.positionRadius.x, p.positionRadius.y, p.positionRadius.z, p.positionRadius.w = rp.x, rp.y, rp.z, light.radius or 0
    p.colorIntensity.x, p.colorIntensity.y, p.colorIntensity.z, p.colorIntensity.w = light.color.x, light.color.y, light.color.z, light.intensity or 1
    pass:drawFullscreen()
end
```

`light_ubo.glsl` keeps its member names, moved into a `#group 2` block. The
`#define`s are unchanged, so `fragment/light/point.glsl` only changes its
include.

### 3d. Immediate, debug and UI drawing

`Draw.*` (global, one `DrawImmediate` command per primitive) and the
per-primitive `shader:start()/setFloat/Draw.Rect` in `UI/DrawEx.lua` are
replaced by `Imm`. `Imm` is a batcher on the current pass that appends to
`VertexRing` and merges consecutive primitives with the same
`(pipeline, texture binding, scissor)` into a single `DrawImm`.

```lua
local imm = Render.imm()                  -- current pass's batcher (one open pass at a time)
imm:rect(x, y, w, h, color)               -- solid
imm:image(view, sampler, x, y, w, h, u0, v0, u1, v1, color)
imm:shape(Shape.Circle, x, y, w, h, color, radius)   -- SDF params ride in vertex attrs
imm:line(x1, y1, x2, y2, color, width)    -- quad, width honoured on wgpu too (gap 11)
imm:line3(p1, p2, color)                  -- Imm3D layout, depth-tested debug pipeline
imm:text(font, str, x, y, color)          -- glyph-atlas quads
imm:pushClip(x, y, w, h) / imm:popClip()  -- ClipRect stack now feeds batcher scissor
imm:setTransform(m)                       -- replaces ShaderVar "mWorldViewUI" (UI/Graph.lua)
```

```lua
-- Before: DrawEx.Circle
RenderState.PushBlendMode(BlendMode.Additive)
shader:start(); shader:setFloat('radius', r); shader:setFloat2('size', sx, sy)
shader:setFloat4('color', color.r, color.g, color.b, color.a * alpha)
Draw.Rect(x, y, sx, sy); shader:stop()
RenderState.PopBlendMode()
-- After
Render.imm():shape(Shape.Circle, x, y, sx, sy, color:withAlpha(color.a * alpha), r)
```

The `Imm2D` vertex is `{pos: vec2, uv: vec2, color: vec4, params: vec4}`, 48 B.
Each `Shape` maps to its existing `ui/*` fragment shader. Values that were
uniforms (`radius`, `size`, `color`) come from varyings instead, and
`Additive` is part of the shape's pipeline. Text goes from one draw per glyph
to one draw per atlas page per batch. Each `Font` gets a 1024² R8 atlas page
list with a shelf packer. Glyphs are rasterized on demand with FreeType and
uploaded with a sub-rect `UpdateTexture2DDataByResource`. The Rust
`UIRenderer` (`ui_renderer/*`, used by HmGui) and `font.rs` draw through the
same batcher. `Draw.Color`/`PushAlpha` state becomes per-vertex color.

### 3e. Offscreen generation

These are Rust helpers in `render/gpu/gen.rs`, built on passes:

```lua
-- Before (Legacy GenUtil.ShaderToTexCube / TexCube:generate(ShaderState))
local tex = GenUtil.ShaderToTexCube(res, TexFormat.RGBA16F, 'gen/planet', { seed = s, freq = f, ... })
-- After
local P = Cache.Shader('fullscreen', 'gen/planet'):blockType('Params')
local p = P(); p.seed, p.freq, p.power, p.coef = s, f, pw, coef
local tex = Gen.Cube { size = res, format = TexFormat.RGBA16F, shader = 'gen/planet', params = p, mips = true }
local vol = Gen.Volume { size = res, format = TexFormat.R32F, shader = 'gen/asteroid_density', params = p }
```

`Gen.Cube` runs six passes on `CubeFace` views. `cubeLook`, `cubeUp` and
`cubeSize` move into a fixed `GenFaceBlock` on group 2. It keeps today's
adaptive job splitting, which bounds time per submit to avoid driver TDR, as
scissored `DrawFullscreen` slices inside each face pass. `Gen.Volume` runs one
pass per `D2Layer` view. `origin`/`du`/`dv` from `GenUtil.ShaderToTex3D`
become per-layer draw-block values. `gen_ir_map` replaces its six synchronous
`get_data`/`set_data` round trips per cube with `CopyTexture` (GL:
`glBlitFramebuffer` per face/level, because `glCopyImageSubData` needs 4.3),
followed by mip passes over `CubeFace` views at mip *n*. Nebula's 1D LUTs are
material-style textures in group 1 of the gen shader.

### 3f. Readbacks

```lua
local t = Renderer:readAsync(view, x, y, w, h, TexFormat.RGBA16F)   -- ticket, no stall
if t:ready() then local px = t:data() ... t:free() end              -- typically 2–3 frames later
Renderer:readSync(view, x, y, w, h, TexFormat.RGBA8)                -- screenshots, tests, tools only
```

Auto-exposure (`RenderCoreSystem:tonemap`) currently does `genMipmap` and
then 128 synchronous `src:sample()` calls, each one a full render-thread
round trip. It changes to `GenerateMips` plus one `readAsync` of the ≤512²
mip, resolved a few frames later. The percentile and log-average maths stay
in Lua. Adaptation is already slow (`speedUp/speedDown`), so the extra
latency can't be seen. Validation probes and `Tex2D.ScreenCapture` use
`readSync` explicitly.

---

## 4. GL 3.3 implementation notes and the wgpu mapping

| Object | GL 3.3 implementation | wgpu |
|---|---|---|
| Pipeline | `GlPipeline { program, blend, cull, depth, polygon }`. `SetPipeline` skips work if the id is unchanged, otherwise diffs each field against a tracked `GlStateCache` (`glEnable/BlendFunc/CullFace/DepthMask/DepthFunc/PolygonMode`). The VAO comes from the mesh or ring layout, not the pipeline. | `RenderPipeline` compiled at `CreatePipeline`, keyed by desc + shader generation |
| Shader link | After linking: `glUniformBlockBinding(block, group*4+k)` and, once, `glUseProgram` + `glUniform1i(sampler, fixed unit)` from `ShaderLayout`. Draws never touch sampler uniforms again. | naga module with injected `set/binding`; `BindGroupLayout`s from reflection |
| Group 0/2 dynamic block | `glBindBufferRange(UNIFORM_BUFFER, binding, ring_buf[slot][i], offset, size)`, cached per binding point | `set_bind_group(g, bg, &[offset])` with `has_dynamic_offset` |
| Group 1 bind group | Entry list: `glBindBufferRange` for the params slice, then `glActiveTexture` + `glBindTexture` + `glBindSampler` per unit, with per-unit `(tex, sampler)` cache | `BindGroup` created once and invalidated by texture generation (gap 3) |
| Group 3 inputs | Same as group 1 on units 12–15, no object | Transient bind group cached by content hash, evicted per frame |
| Sampler | `glGenSamplers` + `glSamplerParameter*` (core in 3.3), including `TEXTURE_MAX_ANISOTROPY` when the extension exists, and LOD clamp | `Sampler` (gaps 5/6/18) |
| TexView (sampling) | No views in 3.3. On bind, `GL_TEXTURE_BASE_LEVEL/MAX_LEVEL` is set if it differs from that texture's cached range. This is the backend's own implementation of the view, not a Lua-visible shim. | `TextureView { base_mip_level, mip_level_count, dimension }` (gaps 7/8) |
| TexView (attachment) | `glFramebufferTexture2D(level)`, `glFramebufferTextureLayer` for 3D layers, `TEXTURE_CUBE_MAP_POSITIVE_X+face` for cube faces | Per-face/layer view (gap 9) |
| RenderPass | FBO cache keyed by `(color views, depth view)`. `glDrawBuffers` and completeness are checked once at creation. Entries are evicted on `DestroyResources` through a texture→FBO index. Begin: bind FBO, set viewport to the attachment's mip size, then `LoadOp::Clear` via `glClearBufferfv/fi` per attachment with color/depth mask forced on and scissor off, then restore. `DontCare`/`Discard`: no-op (no `glInvalidateFramebuffer` in 3.3). Backbuffer = FBO 0. | `begin_render_pass` with `LoadOp`/`StoreOp`; one pass per Begin/End |
| UniformRing | One or more 4 MiB `GL_UNIFORM_BUFFER`s per slot, `glBufferSubData` per `RingChunk`. `glFenceSync` after the frame's last pass, `glClientWaitSync` at `BeginFrame{slot}` before overwriting. That fixes the pacing protocol, which today only proves the render thread *executed* the frame. | `queue.write_buffer` per chunk (or `StagingBelt`); fence via `on_submitted_work_done` (gap 10) |
| VertexRing | Same buffers with `GL_ARRAY_BUFFER`; one VAO per `Imm2D`/`Imm3D` layout; `glDrawArrays(first_vertex)` with stride-aligned chunks | `set_vertex_buffer(0, ring.slice(..))` + `draw(first..)` |
| Instanced | Per-draw `glVertexAttribPointer` re-point of divisor attributes at the ring offset (no `BaseInstance` before 4.2); the texture-fetch path (`DrawInstancedIndices`) keeps the static data texture in group 2 | Vertex buffer with `step_mode: Instance`, or a storage buffer |
| Readback | `GL_PIXEL_PACK_BUFFER` per ticket: `glReadPixels` into the PBO from a read FBO on the view, `glFenceSync`, poll with timeout 0 each `BeginFrame`, map and copy into `ReadbackSlot` (`Arc<{state: AtomicU8, data: Mutex<Vec<u8>>}>`) | `copy_texture_to_buffer` + `map_async` (gap 1) |
| Mips | `glGenerateMipmap` | Blit chain, one pass per level (gap 4) |

**Coexistence between S3 and S6.** This is not a shim. Old commands stay
exactly as they are until the step that deletes their last caller. The
executor keeps one `GlStateCache`. Any old state, shader or bind command
writes through it and clears `current_pipeline`, so the next `SetPipeline`
applies a full diff. The old `tex_index` unit allocator starts at the
program's first unit above its fixed units. Any non-pass `submit` flushes the
open `PassEncoder` first, so mixed old and new draws in one pass execute in
order. These hooks are about 20 lines, and they are deleted in S6 together
with the old commands.

---

## 5. Migration steps

The order is revised from the original plan. Passes come first because
every later API hangs off `pass`, so each call site moves once into its
final shape. The uniform ring merges into materials, because a
single-buffer interim would be throwaway work. Samplers and views arrive
with bind groups in S3, and S7 only deletes the dead texture-level state.

| Step | Was | Removes (old API → gone) |
|---|---|---|
| S2 Passes | S4 | `RenderTarget.*`, `tex:push/pushLevel/pop`, `Draw.Clear/ClearDepth`, FBO stack cmds |
| S3 Binding model, pipelines, samplers, views | S2a+S5+S7 | `ShaderVar.*`, `#autovar`, `ShaderVarMap`, `Viewport.push` vars, `GetUniformLocationByResource` protocol for new shaders |
| S4 Materials, scene list, uniform ring | S2b+S3 | `MaterialDefinition`/`Material`/`DynamicShaderVar`/`UniformFuncs*`/`ShaderVarFuncs`, `SetInstanceUniforms`, batch draw path, Material UBO |
| S5 Fullscreen, post, generation | (S5/S9 parts) | `TexCube:generate(ShaderState)`, `GenUtil.*`, Light UBO, `Renderer:updateLightUbo` |
| S6 Immediate batching + glyph atlas | S6 | `Draw.*`, `Shader:start/stop/set*/iSet*`, `ShaderState`, `RenderState`, all uniform/texture-bind/state/draw-immediate commands, `GpuHandle` |
| S7 Sampler/view completion | S7 | texture-level filter/wrap/anisotropy/mip-range API + commands |
| S8 Async readback | S8 | `tex:sample()`, `SamplePixel2D`, `ReadFramebufferPixels`, sync reads in engine paths |
| S9 Mips and texture kinds | S9 | GL-enum upload payloads, placeholder 1D/3D on wgpu |
| S10 Hot reload | S10 | stale-pipeline and stale-bind-group behaviour |

**Verification gate for every step.** Run `tools/render_validation/run_all.py gl` with
both the default and `--features immediate` builds. The supervised scenes
are Clear, Gradient, Downsample, Upscale, ViewportScissor, UiComposite and
the Indexed* scenes. Then `compare.py gl` against the baseline captures of
PlanetTest, Benchmark, SolarSystemPlayable and MoonTest at frame 120, plus
`cargo check --features immediate` and `cargo test` (layout parity test from
S3 on). A step is done when baseline RMSE is unchanged within tolerance and
`Benchmark` draw-call/fps stats are equal or better. A baseline may only be
re-blessed with a written reason, as S8's auto-exposure step will need.

### S2. Render passes and attachment views

**Status: implemented.** Differences from the plan below:

- `LoadOp` is a C-like enum (see §1.2); `TexView` carries the mip `extent`; `RenderPassDesc` has an explicit
  `backbuffer(w, h, load, r, g, b, a)` / `backbufferDepth(load, d)` instead of "no attachments means backbuffer".
- Lua view names: `view()`, `mipView(l)` on `Tex2D`, `layerView(z)`/`layerMipView(z, l)` on `Tex3D`,
  `faceView(f)`/`faceMipView(f, l)` on `TexCube`. `Tex3D` has no `clear()`, so none was ported.
- One pass at a time is enforced: `beginPass` panics with the open pass's label if another is open. The old FBO
  stack allowed nesting, which exposed two patterns that now live outside passes: (1) the skybox closures in
  eight states generated the nebula cube lazily inside the opaque pass, so each state now runs the closure once
  at init with `blendMode = nil` (draws nothing); (2) `RenderCoreSystem:render` no longer wraps the frame in
  `Window:beginDraw/endDraw`. It pushes a base `Viewport` (ClipRect needs one), and only the final present is a
  backbuffer pass. `Window:beginDraw()` is a `LoadOp.Load` backbuffer pass (used by `Application:immediateUI`).
- `RenderPass:finish()` ends the pass. Ending a pass rebinds the default framebuffer, and the pass viewport
  push/pop still goes through the internal `Viewport` code (removed in S3).
- The wgpu executor builds a framebuffer-stack entry from the pass desc at `BeginRenderPass` and pops it at
  `EndRenderPass`. 3D layer attachments still use the whole volume view there (TODO: `depth_slice`).

Original plan:
- **Engine.** Add `RenderPassDesc`, `TexView` (attachment dims only),
  `BeginRenderPass/EndRenderPass` and the GL FBO cache with load ops.
  Pass begin sets the viewport and the 2D projection; until S3 this is done by
  the existing internal `Viewport` code, which is not visible to Lua. Port the
  Rust callers: `TexCube::generate`, `gen_ir_map`, and the `clear()` methods
  of `Tex2D`/`Tex3D`/`TexCube`. `Window:beginDraw/endDraw` becomes the
  backbuffer pass. Old draws still run inside passes.
- **Lua.** 75 target pushes/binds (`RenderTarget.Push/BindTex*` +
  `tex:push/pushLevel`) and 36 `Draw.Clear*` across 20 files. Main sites:
  RenderCoreSystem (11 + 5), Shared/Rendering/RenderingPass.lua (2, which
  becomes a `RenderPassDesc` builder), Core/ECS/Mesh/Util/GenUtil.lua (1),
  States/App/Tests/GenTex2D.lua (1 + 2), and the validation scenes IndexedMrt
  5, IndexedPostProcess 4, Downsample 4, IndexedDepth 3, UiComposite 2,
  IndexedCull 2, IndexedClip 2, and Upscale, IndexedTexture, IndexedMaterial,
  IndexedGeometry, IndexedBlend, IndexedBatch, Gradient, Clear and
  ViewportScissor at 1 each. **RP/IF:** 31 + 4 more (delete those files).
- **Removal.** Delete `RenderTarget`, `RenderTargetStack`, `Tex*:push/pop/pushLevel`,
  `Draw::clear/clear_depth`, the commands `PushFramebuffer`, `PopFramebuffer`,
  `FramebufferAttach*` ×6, `SetDrawBuffers`, `BindFramebuffer`,
  `BindDefaultFramebuffer` and `Clear`, plus the matching `renderer_ffi.rs`
  entries, in both renderer files.
- **Immediate renderer.** It gets only new commands plus deletions. FBO-cache
  state lives in the shared executor.

### S3. Binding model, pipelines, samplers, views, frame group
- **Engine.** Add the `#group` preprocessor and `ShaderLayout` reflection,
  plus the layout parity test. Add `Pipeline`/`PipelineDesc` with the GL
  state cache, `Sampler` and `TexView` sampling dims. Add `CreateBindGroup`,
  `PassCmd`, `PassCommands`, the `PassEncoder`, and `ViewBlock` in group 0,
  ring-allocated per pass. In this step the ring is a single frame-slotted
  allocator; S4 extends it. Add `Renderer:setCamera/setEnvironment`. Shaders:
  `camera_ubo.glsl` becomes `view_block.glsl` with the same member names, and
  `envMap`/`irMap` move to group 0 in `include/fragment.glsl`. `#autovar` is
  removed from 12 shader files.
- **Lua.** 22 `ShaderVar.Push*` in 10 files:
  Modules/Cameras/Managers/CameraManager.lua (5, along with
  `Renderer:updateCameraUbo`, becoming `Renderer:setCamera`); `envMap`/`irMap`
  in LTheoryRedux, CameraTest, PlanetTest, MoonTest, ShipTest, StationTest,
  SolarSystemPlayable and Testbeds/WeaponSystem (2 each, becoming
  `Renderer:setEnvironment`); UI/Graph.lua (1, becoming
  `pass:setUiTransform`). The validation scenes are also ported fully to
  `setPipeline`/`alloc`/`setInputs` as the first end-to-end users. That is
  about 13 small files, and each probe value must match before and after.
- **Removal.** Delete `ShaderVar`, `ShaderVarMap`, the auto-var code in
  `Shader::start`, `Viewport::push/pop` (folded into the pass),
  `CreateCameraUBO`/`UpdateCameraUBO`, `Renderer:createCameraUbo/updateCameraUbo`,
  and `ShaderVarCache.lua`.
- **Immediate renderer.** Shared code only. Pipelines and bind groups are
  executor state.

#### S3 notes

**Status: implemented.** Differences from the plan above, and what shipped:

*Shader layout.*
- `#group N` applies to the uniform blocks and samplers that follow it. An include inherits the includer's group,
  and a `#group` inside an include does not leak out of it. Declarations outside any `#group` are legacy loose
  declarations and are not recorded. Vertex and fragment layouts are merged per program, numbered by first
  appearance (vertex first); a layout error (more than 4 blocks or the unit count of a group, one name in two
  groups or with two dimensions, `#group` outside 0..3) fails the compile and falls back to the error shader.
- `CreateShader` carries the merged `ShaderLayout` and replies with the reflected `Vec<BlockLayout>` (GL:
  `glGetActiveUniformBlockiv/glGetActiveUniformsiv`; wgpu: naga). At link the GL executor applies
  `glUniformBlockBinding` and, with the program current, `glUniform1i` per declared sampler. `ViewBlock` is asserted
  against its reflected block at link (size, offsets, types; a mismatch panics, hot reload included).
- `shader:blockType(name)` is in already (it was only implied for S4): `BlockLayout::lua_struct` builds the
  anonymous `ffi.typeof` struct with `_padN` holes; arrays become raw 4-byte lanes (`float name[count*stride/4]`),
  `mat3` is `float[12]`. `pass:alloc(T)` is the hand-written export `RenderPass_Alloc` (the FFI generator cannot
  return pointers) plus a Lua cast.
- Parity test (`render/thread/layout_parity.rs`): every vertex and fragment shader in `res/shader` goes through the
  preprocessor and through naga with the recorded block bindings injected; blocks (name, binding, std140 member
  offsets and types, `ViewBlock` against the Rust struct) and samplers (name, dimension) must agree. It cannot
  compare against GL reflection without a context; that side is asserted at link time and exercised by the
  validation scenes. Two documented dead shaders (`uv_metal`, `ptracer`) are excluded, as in the wgpu compile test.

*Pipelines, samplers, views, bind groups.*
- `PipelineDesc` uses `[Option<TexFormat>; 4]` and `Option<TexFormat>` instead of `ArrayVec`, and
  `VertexLayout` is `Mesh | Fullscreen` (no `VertexFormat` payload until wgpu needs it). Lua:
  `PipelineDesc.Create(shader)` with `:blend/:cull/:depth(test, write, compare)/:topology/:vertex/:polygon/
  :colorFormat/:depthFormat`, then `Pipeline.Get(desc)` returns the `PipelineId` as a plain integer. The GL executor
  diffs each state field against a `GlStateCache`; there is no main-thread `SetPipeline` dedupe (the executor skips an
  unchanged pipeline, and the main thread cannot know when legacy commands invalidate it).
- Samplers: `Sampler.Get(SamplerDesc)` returns a plain integer id. `Samplers.*` is a Rust enum whose value is the
  id: `Point, PointRepeat, LinearClamp, LinearRepeat, LinearMipClamp, LinearMipRepeat`, created at renderer start.
  The environment maps use `LinearMipClamp` (what the nebula cubes had as texture state). Texture-level filter and
  wrap state is untouched until S7.
- `TexView`: `ViewDim::D1` added. `tex:view()` now spans every mip (for sampling; as an attachment it is level 0),
  `mipView(l)` is one level, `tex:view{ baseMip = , mipCount = }` narrows it (`TexView:mips`). On bind the GL executor
  sets `TEXTURE_BASE_LEVEL/MAX_LEVEL` only when they differ from a per-texture cache; `D2Layer`/`CubeFace` views sample
  the whole texture on GL 3.3.
- `CreateBindGroup` holds textures with samplers only. Uniform entries arrive with the material arenas (S4). Bind
  groups are never freed until S4 gives `Material` ownership. `BindGroupDesc.Create(shader, group):texture(name, view,
  sampler)` + `Renderer:createBindGroup(desc)` + `pass:setBindGroup(group, id)`; `IndexedTexture` uses it for group 1.

*Passes.*
- `PassCmd` as built: `SetPipeline, SetBindGroup, SetView, SetEnvironment (added), SetDraw, SetInputs, SetViewport,
  SetScissor, DrawMesh, DrawFullscreen`. `DrawMeshInstanced`, `DrawInstancedIndices` and `DrawImm` come with S4/S6;
  until then those draws are legacy commands issued inside the pass (`IndexedBatch` does this and draws with the
  pass's pipeline). `PassCommands { slot, uniforms, cmds }` has no vertex chunks yet, and there is no `BeginFrame`
  command: the frame slot rides in `PassCommands`.
- Lua: `pass:setPipeline`, `setInputs(view, sampler, ...)` (staged, sent as one `SetInputs` before the next draw),
  `setBindGroup`, `alloc(T)`, `drawMesh`, `drawFullscreen`, `setViewport` (also resizes the UI projection, like the
  old `Viewport.Push`), `setScissor`/`clearScissor`, `setUiTransform`. `Renderer:currentPass()` gives non-owning
  access to the open pass (UI widgets; it cannot `finish`).
- Flush points: 512 recorded commands (checked after a draw and at `alloc` entry), `finish`, and any non-pass command
  (threaded: `Renderer::submit`; immediate: `Renderer::ex()`). A pointer from `alloc` is valid until the next draw is
  recorded or the next flush; write the block, then draw.
- `ViewBlock` is 448 bytes (`ViewBlock::SIZE`): the 288 bytes of the old camera UBO plus `mProjUI`, `mWorldViewUI`,
  `viewport` and a reserved `time`. It is ring-allocated per pass, and passes of a frame with an identical block share
  one upload. `Renderer:setCamera(view, proj, starDir)` and `setEnvironment(env, ir)` are state; an open pass
  re-emits. The environment is bound at every `beginPass` (the executor's unit cache makes that cheap).
- Uniform ring: 256 KiB chunks, one GL uniform buffer per chunk and frame slot, orphaned at its first use in a
  frame. No fences yet (S4 adds them).
- `vertex/fullscreen.glsl` draws a unit quad scaled by `ubo_viewport.zw` through `mProjUI`, so it reproduces
  `Draw.Rect(0, 0, w, h)` bit for bit (`fullscreen_flip.glsl` reproduces `Draw.Rect(0, h, w, -h)`). The NDC quad
  of S5 replaces it. `fragment/blit.glsl`, `color_block.glsl` and the `indexed_*` shaders declare their
  samplers and `Params` blocks under `#group`.

*Viewport and ClipRect.* The `Viewport` Lua type and `VpStack` are gone. `Renderer::target_size()` is the viewport of
the open pass (or the last pass's extent). `ClipRect` keeps its stack and re-syncs the scissor from it at `beginPass`
and after every operation, sending commands only when the wanted scissor differs from the last one sent. Outside a
pass it only edits the stack, so `RenderCoreSystem` no longer needs a base viewport.

*Coexistence hooks* (all marked `S6: remove`): the pass-encoder flush before any non-pass command; the executor's
write-through of the state commands into `GlStateCache` and `invalidate_pipeline` in every shader-bind or state
command; `drop_unit_sampler` in the legacy texture bind (a sampler object a pass left on the unit would override the
texture's own parameters); `note_mip_range` in the legacy mip-range command; `last_shader_bind` reset on
`PassCommands`. One deviation from the §4 text: the legacy unit allocator hands out units from 3 up (not "above
the program's fixed units"), skipping the units of the program's layout. Units 0..2 belong to group 0, and every
pass rebinds the environment maps there, so a legacy bind on unit 0 to 2 would be clobbered by the next
`beginPass`. (Found the hard way: `gen_ir_map` binds its source cube on unit 1.)

*Lua call sites.* 22 `ShaderVar.Push*` migrated (CameraManager 5 plus `updateCameraUbo` to `Renderer:setCamera`,
16 `envMap`/`irMap` pushes in 8 files to `Renderer:setEnvironment`, `UI/Graph.lua` to `pass:setUiTransform`). All
16 files in `States/App/Rendering` (the 13 supervised scenes plus Clear, Upscale, ViewportScissor) are ported;
`Application:immediateUI` and `RenderCoreSystem` lost their `Viewport` calls. `ShaderVar`, `ShaderVarMap`, the
auto-var code in `Shader::start`, `Viewport`, `CreateCameraUBO/UpdateCameraUBO`, `Renderer:createCameraUbo/
updateCameraUbo`, `CameraUbo*` and `script/Render/ShaderVarCache.lua` are deleted in both renderer files. The
legacy effect objects (`Pulse`, `Explosion`, `Bay`, `Drone`, `Turret`) used `ShaderVarCache` for uniform
locations; they now use `script/Legacy/Util/ShaderLocations.lua` (the same eight lines, Legacy-owned). Legacy files that
still mention removed APIs but are not loaded by any active state: `GameObjects/Entities/StarSystem.lua`,
`Systems/Camera/Camera.lua` (and `Overlay/GameView` through it).

*wgpu executor.* It compiles and implements the new commands 1:1 on its existing GL-shaped state (pipelines set
shader, blend, cull, depth and wireframe; the view block and draw block are staged as plain-uniform bytes; samplers
are matched to units through the shader layout). It does not render these paths correctly yet; its block
reflection is built from naga, with grouped block bindings injected by `adapt_glsl_for_naga_with_layout`.


### S4. Materials, scene list, uniform ring
- **Engine.** Full `UniformRing`/`VertexRing` with chunk recycling in both
  backends, GL slot fences (gap 10), `DrawBlock`, material param arenas
  (`CreateBuffer/WriteBuffer`), the Rust `Material`, and
  `SceneList::submit` (cull, sort, `mWorldIT`, `perDraw`, emit). The
  instance ring replaces `DrawInstancedWithData`'s per-call `Vec`, which
  addresses gap 24's 8.4 MB/frame upload.
- **Lua.** MaterialDefs.lua (9 materials). Rewritten or deleted:
  Shared/Rendering/{Material,DynamicShaderVar,Texture,UniformFuncs}.lua,
  Shared/Definitions/{UniformFuncDefs (10 set + 4 setTex),ShaderVarFuncs}.lua,
  Shared/Types/MaterialDefinition.lua, Shared/Registries/Materials.lua
  (`MaterialType` registry), and 21 `Materials.X()` clone sites (become
  `:instance()`). RenderCoreSystem: `buildPassLists/cullPassLists/renderInOrder/apply*Vars`
  plus its 4 per-pass `RenderState` pushes in RenderingPass.lua. Render-fn
  draws that run inside scene passes: the skybox and starbg closures in 9
  states (2 RS + 1 set + 2 setTex each), AsteroidBeltRenderer (1 + 2),
  BeamEntity (5), ImpactEffectEntity (4), PointLightSystem diagnostics (6),
  PlanetTest ring (7 `addStaticShaderVar`), and ad-hoc `Material(...)` in
  LTheoryRedux.lua:747 and SolarSystemPlayable.lua:489.
- **Removal.** Delete `SetInstanceUniforms`, `InstanceUniformsCmd`,
  `GenericUniformName`, `SetUniformMat4ByGenericName`,
  `RenderBatch::process_batch_intern`/`flush_batch`/`add_entity`,
  `command_buffer`/`flush_intern`, `InstanceBatch`, `begin_batch`/`add_cull_entity`/`cull_batch`
  (absorbed into `SceneList`), `DrawInstancedWithData`, `MaterialUboData` and
  its commands, and `index_set_instance_uniforms`.
- **Immediate renderer.** Ring chunks are returned inline. The GL fence wait
  is in the executor, so it is the same code path.

#### S4 notes

**Status: implemented.** Differences from the plan above, and what shipped:

*Rings and frames.*
- `UniformRing` and `VertexRing` are two wrappers of one allocator (`StagingRing`, `render/gpu/uniform_ring.rs`).
  Chunks: 256 KiB (uniform), 1 MiB (vertex; one instanced draw must fit in a chunk). A `RingChunk` is
  `{at, bytes, skip}`: `bytes[skip..]` goes to `at`. A chunk that filled up moves to the executor as one `Vec` (no
  copy; `skip` is what an earlier flush already sent); only the filled part of the chunk still being written is
  copied, into a small pooled `Vec`. The executor does not own a channel: it queues the uploaded memory
  (`take_returned_chunks`), and `RenderThread` forwards it over an unbounded return channel (threaded) or
  `Renderer::send_pass_commands` recycles it inline (immediate). The wgpu executor drops the chunks (the main
  thread then allocates fresh ones).
- GL fences: `SwapBuffers` inserts `glFenceSync` for the executor's current slot just before the swap, and
  `BeginFrame { slot }` (a new command, sent right after the frame's `SwapBuffers` and `PacingFence`) waits for the
  fence of the slot it enters with `glClientWaitSync` (flush flag, 1 ms slices, 5 s cap) before that slot's buffers
  are overwritten. The S3 buffer orphaning is gone: buffers are created once per (slot, chunk) and reused.
- `PassCommands` gained `vertices`. `PassCmd::DrawMeshInstanced { mesh, index_count, instances, count }` and
  `DrawInstancedIndices { mesh, index_count, indices, count }` carry `RingOffset`s into the vertex ring; Lua:
  `pass:drawMeshInstanced(mesh, instances, count)`, `pass:drawInstancedIndices(mesh, indices, count)`.
  Removed with them: the `GpuHandle` and `ByResource` forms of the `DrawMeshInstanced` command, the
  `DrawInstancedIndices` and `DrawInstancedWithData` commands, `InstanceBatch`, and
  `Mesh:drawInstancedWithData/drawInstancedIndices`.
- `BeginRenderPass` resets the GL state cache to the legacy defaults (no blend, no culling, no depth test, depth
  writes on, `LEQUAL`, filled polygons). The scene passes no longer push `RenderState`, so without this the last
  pipeline of one pass would leak into the legacy draws of the next (post-processing, UI composite). wgpu passes
  start from scratch as well.

*DrawBlock and shaders.*
- `DrawBlock` members are named `mWorld`, `mWorldIT`, `drawScale`, `drawUser[7]` (no `ubo_` prefix: the shaders use
  them directly). Size 256 bytes, binding 8, checked at link like `ViewBlock`
  (`draw_block_matches_the_rust_struct` covers a vertex and a fragment user in the parity test).
- `include/vertex.glsl` is split: `vertex_base.glsl` (attributes, varyings, `VS_BEGIN/VS_END`, `logDepth`),
  `vertex.glsl` (base plus the loose `mWorld`/`mWorldIT`, for the shaders that are not scene materials until S6) and
  `vertex_scene.glsl` (base plus the draw block). `wvp.glsl` and `traveldrive.glsl` use `vertex_scene`.
  `fragment.glsl` no longer declares the loose `mWorldIT`, which no fragment shader read.
- `scattering2.glsl` declared `rPlanet`/`rAtmo` (and the unused cloud samplers) as loose uniforms. They are now
  parameters of `atmosphereDefault(rd, ro, rPlanet, rAtmo)`, so the including shaders pass them from `drawScale.x`
  and their `MaterialParams`; a `#define rPlanet` would have rewritten the identifiers inside the include. The cloud
  functions are behind `#if CLOUDS_ENABLED` (it is 0), and two dead `getClouds` calls in `atmosphere()` are gone.
- Define macros over `drawUser` after the includes (see `planet.glsl`, `star.glsl`).
- Effects that draw in a scene pass with their own pipeline use the same block: the new `billboard/quad_draw`,
  `billboard/axis_draw`, `effect/pulsehead_draw` and `effect/beam_draw` read color, alpha, size and seed from
  `drawUser` (the Legacy effect objects keep the loose-uniform originals). `starbg.glsl` has a group-2
  `StarBackgroundParams { brightnessScale }`. `wvp_instanced_tex.glsl` has a group-2 `InstanceParams` block and the
  `instanceDataTex` sampler in group 2.

*Materials.*
- `CreateBuffer { id, size }`, `WriteBuffer { id, offset, data }` and `DestroyBindGroups { ids }` are new commands;
  `BindEntry::Uniform { index, buffer, offset, size }` binds a buffer range to the group's `index`-th block when the
  group is set. Arenas are 256 KiB buffers carved into `UNIFORM_ALIGN` strides; a freed slice goes on a free list
  keyed by its reserved size. `Material` has no renderer in its `Drop`, so it queues its slice and bind group on a
  channel that `end_frame_triple_buffered` drains (`drain_releases`).
- `Material.Create(shader, blend, cull, depthTest, depthWrite)` takes the state directly instead of a
  `PipelineDesc` template. Pipelines have no attachment formats yet (`TexView` carries none); GL ignores them.
  `Material:commit()` is one `WriteBuffer` plus a new bind group if a texture changed. `setTexture(name, view,
  sampler)` panics on an unknown name or a dimension mismatch. A material that is drawn with an uncommitted change is
  committed by `addItem`, which warns once about samplers without a texture; a resource write in an open pass trips
  a debug assertion.
- The Lua side: `Shared/Types/MaterialType.lua` (`MaterialType { name, shader = { vs, fs }, state, defaults,
  textures, perDraw }`), `Shared/Rendering/Material.lua` (`mat:params()`, `setTexture`, `commit`), the registry
  `Shared/Registries/Materials.lua` (`Materials.X:instance()`) and `Shared/Definitions/MaterialDefs.lua`. Cull and
  depth state default to those of the scene pass of the material's blend mode (`Render/Pipelines.lua`:
  `Pipelines.Opaque/Additive/Alpha`), which is what `RenderingPass` used to push. Textures a type declares with a
  `tex` are shared by its instances and get their mip chain once; samplers default by texture kind
  (`Samplers.LinearMipRepeatAniso`, a new preset with 16x anisotropy, for 2D: the state the old `Texture` class gave
  every material texture). `Material.ReloadAll()` recommits the live materials (pipelines follow the shader's new
  program by themselves).
- Parameters moved from per-frame closures into data. Planet (`color1..4`, `oceanLevel`, `rAtmo`), atmosphere
  (`rAtmo`), moon (`highlandColor`, `mariaColor`, `heightMult`, `enableAtmosphere`) and ring (`rMin`, `rMax`, `seed`, ...)
  are written once through `Shared/Rendering/PlanetMaterials.lua`. The cloud and ring time, the star's time,
  temperature and tint, and the travel drive's time, intensity and speed are `perDraw` values. `rAtmo` is the body's
  scale at creation times `atmoScale`; the old shader var re-read it every frame. Parameters the shaders never
  declared are gone (`starColor`, the planet's `starTint`, `craterDepth`, the moon's base textures, `ringTex`, the
  ring's `planetPos`/`planetQuat`/`planetRadius`/`ringQuat`), and with them their "no such uniform" warnings;
  `enableDebug`/`debugMode` of the ring are real block fields now.
- `DebugColor` (turret bodies) used to share one color variable between all clones (the clones copied the list of
  variable objects, not the objects), so the last turret loadout colored every box. Each instance has its own color
  now. That is the only intended pixel difference: the WeaponSystem testbed at frame 40 differs from the pre-change
  capture in 91 pixels (max 6/255) around dark turret boxes; the four baseline scenes are bit-identical.

*Scene list.*
- `SceneList` is described in section 3b and in `batch-rendering.md`. `submit` is a Lua wrapper
  (`ffi_ext/SceneList.lua`) around the Rust `prepare` (cull, sort, survivor flags) and `emit`. The `perDraw`
  callbacks run in Lua between the two, for the survivors, so no C-to-Lua call exists and the callbacks stay
  JIT-friendly. `addTransform()` returns a `SceneTransform*` (`world`, `cx/cy/cz`, `scale`, `index`);
  `addItem(t.index, mesh, material, entity)` computes the cull radius in Rust (the mesh bound about the origin
  times the scale). Culling can be turned off per call (`Config.render.general.frustumCulling`); then nothing is
  sorted either, as before.
- The sort key `(pipeline, material uid, mesh id)` replaces the old shader-pair key; equal keys keep insertion
  order, and the alpha bucket does not sort.
- Removed: `RenderBatch`, `EntityRenderData`, `BatchStats`, `Renderer:beginBatch/addEntity/addCullEntity/cullBatch/
  flushBatch/getBatchStats`, `command_buffer` and `Renderer:beginFrame/flush`, `SetInstanceUniforms` and
  `InstanceUniformsCmd`, `GenericUniformName` and `SetUniformMat4ByGenericName`, `MaterialUboData` and its commands,
  `Shader:iSetInstanceUniforms`, `Enums.UniformType`, and the Lua files listed in the plan (`MaterialDefinition`,
  `UniformFunc`, `DynamicShaderVar`, `Texture`, `UniformFuncs`, `UniformFuncDefs`, `ShaderVarFuncs`). The generated
  FFI of removed types (`InstanceBatch`, `BatchStats`) was deleted by hand.

*Render-fn draws in scene passes.* They set their pipeline with `Pipelines.get(shader, state)` (cached per shader
resource and state table) through `Renderer:currentPass()`: the skybox and star field (shared by eight states in
`Render/Backdrop.lua`), `AsteroidBeltRenderer` (bind groups created once, `pass:alloc` for `originRelEye`),
`BeamEntity`, `ImpactEffectEntity` and the point-light markers of `PointLightSystem`. The skybox box and
`Starfield:draw()` are still legacy immediate draws (S6); they run on the program the pipeline bound. Nothing in
`script/Legacy` was touched. The one Legacy dependency an active state has in a scene pass is the WeaponSystem
testbed's `Pulse.Render` (still on `shader:start()`); the testbed wraps the call in the additive pass's
`RenderState` pushes, marked `S6`.

*wgpu executor.* It handles `BeginFrame` (nothing to wait for), the buffer commands (CPU copies, staged as plain
uniform blocks when a bind group with a uniform entry is set) and both instanced pass commands (reading the vertex
ring bytes back into the old instanced paths). Like in S3 it does not render these paths correctly yet.

### S5. Fullscreen, post-processing and offscreen generation
- **Engine.** Add `DrawFullscreen` and `vertex/fullscreen.glsl` (NDC quad,
  replacing the `ui` vertex for filters; `worldray` already works in NDC),
  `render/gpu/gen.rs` (`Gen.Cube`, `Gen.Volume`), and `CopyTexture`.
  Rewrite `gen_ir_map`.
- **Lua.** RenderCoreSystem post chain, deferred lighting, UI composite and
  present (34 set + 19 setTex + remaining RS);
  Modules/Rendering/Systems/LensFlareSystem.lua (5 + 1 RS);
  Core/ECS/Mesh/Util/GenUtil.lua `ShaderToTex3D` (8 set + 4 setTex, called by
  AsteroidMesh); GenTex2D (3). The Legacy generation modules used by active
  code (§6) are promoted and ported: GenUtil.ShaderToTexCube
  (SolarSystemVisualizer ×2, CameraTest) and Nebula1.
- **Removal.** Delete `TexCube::generate(ShaderState)`, `LightUboData` and its
  commands, `Renderer:createLightUbo/updateLightUbo`, `light_ubo.glsl` as a
  UBO (now a group 2 block), `RenderingPass.lua`, and `ImageFilter.lua`/`RenderPipeline.lua`
  (pending §6).
- **Immediate renderer.** No backend-specific work.

#### S5 notes

**Status: implemented.** Differences from the plan above, and what shipped:

*Fullscreen draws.*
- `vertex/fullscreen_ndc.glsl` is the NDC path: the unit quad of `pass:drawFullscreen` goes to `2 * pos - 1`, so
  uv.y = 0 is the bottom row of whatever the pass renders to. In a texture pass that is what `Draw.Rect(0, 0, w, h)`
  gave; in the (y-up) window it is what the flipped `Draw.Rect(0, h, w, -h)` gave, so present needs **no `flipY`
  parameter**. `vertex/fullscreen_ray.glsl` adds the camera-relative world ray of the deferred-lighting shaders and
  replaces `worldray.glsl`. The quad is the same four-vertex fan as before (not a triangle: its diagonal would change
  the interpolation), so every post and lighting pass is bit-identical to the `ui`-vertex rectangles.
- `fullscreen.glsl` and `fullscreen_flip.glsl` (S3, the UI-projection path) stay: the validation scenes use them to
  keep that path covered, and `fullscreen` in a window pass is y-down, which the NDC shader deliberately is not.
- Filter, lighting, `ui/composite` and generation shaders declare their `Params` block under `#group 2` and their
  samplers under `#group 3`. **The group-3 slots are the declaration order** (`texNormalMat` = slot 0, `texDepth` =
  slot 1, ...): `pass:setInputs` callers follow the order in the shader. `include/filter.glsl` only declares `src`
  now (a loose `size` there collided with the filters that declare it in their block).
- `light_ubo.glsl` is `light_block.glsl`: a group-2 `PointLight` block (`positionRadius`, `colorIntensity`) written
  with `pass:alloc` per light. The directional light has its own `Params` block.

*Deferred lighting and post.*
- The global term and the additive directional and point lights draw in **one** pass (the lights use a pipeline with
  `blend = Additive` and share the `texNormalMat`/`texDepth` inputs); the composite is a second pass. Four passes
  became two.
- `applyFilter(name, fill, extra)` is the helper of section 3c: the sampled input is `buffer0:mipView(level)` with
  `Samplers.LinearClamp` (what the old per-buffer `setMipRange(level, level)` plus Linear filters meant) and the target
  is `buffer1:mipView(level)`. The per-frame mip-range and min-filter reset loop of `handleResize` is gone, and with
  it its commands. Auto-exposure still used `sample()` until S8.
- Present is one backbuffer pass with `LoadOp.Load`. `presentAll` (the `showBuffers` debug view) uses one viewport per
  quadrant and now really shows four quadrants: the old y-down code drew the bottom two off the top of the window.
- `radialblur` binds the linear depth it always declared (`depthBuffer`, slot 1; it used to read whatever unit 0
  held). The Legacy tonemapper runs `filter/tonemap_limittheory` (`filter/tonemap_legacy` never existed). The
  `downsample` chain draws the whole target level; for levels of 2 and up the old rectangle was twice the viewport (a
  quarter of the image), which only mattered for super-sampling of 4 or more.
- `LensFlareSystem` draws an additive fullscreen quad into the open scene pass (`Renderer:currentPass()`).
- `RenderingPass.lua` is gone: `RenderCoreSystem:scenePassDesc` caches the scene descriptors itself.

*Generation (`render/gpu/gen.rs`).*
- The Lua namespace is `TexGen`, not `Gen`: `Gen` is the global the Legacy generator namespace injects (`Gen.Primitive`
  in three active files, `Gen.ShapeLib` 101 times in the ship generators), and the loader never overrides a global.
  `TexGen.Cube { shader, size, format, params, inputs, mips }`, `TexGen.CubeInto(cube, { ... })` (ping-pong) and
  `TexGen.Volume { ... }`; `GenDesc` (shader, `Params` bytes, up to four `{ view, sampler }` inputs) is the Rust object
  behind them. `GenUtil.ShaderToTexCube/ShaderToTex3D` are thin wrappers that fill the typed `Params` from a table
  (undeclared names are ignored, as the old `ShaderState` only warned).
- **No separate `GenFaceBlock`.** One draw has one group-2 block, so the face (`genLook`, `genUp`, `genSize`) or the
  slice (`genOrigin`, `genDu`, `genDv`) are the first three `vec4` members of the shader's own `Params` block.
  `texcube.glsl` defines `cubeLook`/`cubeUp`/`cubeSize` over them and is included after the block. The engine checks
  the member names and offsets against the reflected block when a generation starts.
- `generate_cube` keeps the adaptive slicing (scissored rows through `ClipRect`, a quarter of a second per slice) and
  clears each face to (0, 0, 0, 1); `generate_volume` clears nothing (`LoadOp.DontCare`). A planet cube (2048^2
  RGBA16F, six passes, with a face read back) takes about 365 ms on the dev machine against about 395 ms before: the
  GPU time dominates, and the main thread only records for 0.3 ms either way.
- `CopyTexture { src, dst, size }` (both backends): GL blits through two scratch framebuffers with `NEAREST`, restoring
  the pass framebuffer and scissor; wgpu uses `copy_texture_to_texture` for 2D and cube-face views. `gen_ir_map` copies
  level 0 with it and filters the other levels in passes over `CubeFace` views: no CPU round trip any more.
- **Found while porting `gen_ir_map`:** its `sampleBuffer` sampler was never bound (the code set `sample_buffer`), so
  every GGX sample read (pitch, yaw) = (0, 0) and each irradiance level was a plain resample of the source. The port
  reproduces that exactly with a one-texel zero texture; `CONVOLVE_IRMAP` in `texcube.rs` switches to the real, time
  seeded samples. Doing so changes the ambient light of every scene by a little (and makes captures differ from run
  to run), so it needs a decision and a re-bless.
- Also kept bit-exact on purpose: the nebula colour LUTs sample with `Samplers.Point` (uploading a `Tex1D` resets its
  filters to nearest, so ColorLUT's linear filters never applied), and `gen/moon` still declares `baseMoonTex` and
  gets a black texture (`GenUtil` defaults): with the sampler folded away into a constant the compiler rounded twelve
  moon pixels differently.

*Legacy modules.* `Generator`, `Starfield`, `ColorLUT`, `Nebula1` and `Nebula2` moved to `script/Shared/Generation/`
(Nebula1/2 are ported to `TexGen`; the requirers in the states, the Legacy nebula object and `SystemBasic-unused` are
updated). Nebula2 registered itself through the `Namespace.LoadInline('Legacy')` of LTheoryRedux, so that state
requires it explicitly now. `Legacy.Systems.Gen.GenUtil` keeps `FindMountPoint` and forwards `ShaderToTexCube`; its
`ShaderToTex3D` is gone (the unreferenced Legacy `Asteroid.lua` called it). `Primitive` and `MathUtil` stay in Legacy
(pure Lua, no GL). `GenTex2D` starts again (it needed the `Gen` global; it requires `MathUtil` now, and its inline
shader repeated `#version`), but its `Draw.*` based output looks wrong (a black texture with one white quadrant);
that is the immediate-draw path of S6 and was not triaged.

*Removed.* `TexCube::generate`, `LightUboData`/`LightUbo`/`UniformBuffer` (`ubo.rs`), the `CreateLightUBO` and
`UpdateLightUBO` commands (both renderer backends, GL and wgpu executors), `Renderer:createLightUbo/updateLightUbo`,
the by-name `LightUBO` block binding, the `ubo` command category, `worldray.glsl`, `light_ubo.glsl`,
`RenderingPass.lua` and the `RenderState` pushes of `RenderCoreSystem`. (`ImageFilter.lua` and `RenderPipeline.lua`
went in S2.) The first-pass GL error log at startup lost its `GL_INVALID_ENUM` (the by-name light block lookup); the
`GL_INVALID_OPERATION` next to it is older and was left alone.

*Cost.* Commands sent to the render thread per frame (median of three captures at frame 120): PlanetTest, Benchmark,
MoonTest and PlanetTestRing 220 -> 90, SolarSystemPlayable 893 -> 754, WeaponSystem 4258 -> 3784 (the point-light
passes and the per-frame mip-range resets are gone). Frame times, draw calls and render idle time are unchanged within
noise (the post chain is the same draws through passes).

*Capture baseline.* `PlanetTestRing` (PlanetTest with `LTHEORY_CAPTURE_SEED=27`, a seed that rolls a ring; the
variable is read only under `LTHEORY_CAPTURE`) and `WeaponSystem` (the testbed, with deferred point lights) joined the
four scenes. All six are bit-identical to their baselines after S5 on both builds.

### S6. Immediate batching, UI and glyph atlas (the removal gate)
- **Engine.** `ImmBatcher` (Imm2D/Imm3D), the `Shape` pipelines, `ClipRect`
  feeding the scissor, and the per-`Font` glyph atlas. `ui_renderer/*` and
  `font.rs` draw through `Imm`. The `ui/*` fragment shaders read their params
  from varyings.
- **Lua.** UI/DrawEx.lua (18 RS, 53 set, 1 setTex, 15 Draw). The UI widgets
  are Canvas, Graph, Slider, ScrollView, OptionSlider, Container, Widget,
  Stretch, Rect, Image, Hidden, Grid, Collapsible, Window, Checkbox and
  Button. Also SystemMap (4 RS, 13 set, 28 DrawEx), SystemMap3D (3 RS, 4 set),
  InputTest (3 RS, 37 DrawEx), AudioTest (1 RS, 3 set, 5 Draw),
  Shared/Tools/ShaderErrorOverlay.lua, WorldLabelRenderSystem,
  GameplayHUDSystem and States/Application.lua. With RP/IF gone, about 110
  `Draw.*` calls remain in total.
- **Removal.** Delete `Draw` (Lua global and `draw.rs` public API),
  `Shader::start/stop/set_*/index_set_*/get_uniform_index*/has_variable/reset_tex_index`,
  `ShaderState`, `RenderState` + `RenderStateIntern`, and `Viewport`. Delete
  the commands `BindShader*`, `UnbindShader`, `SetUniform*` (all 18),
  `BindTexture*`, `UnbindTexture`, `BindMesh*`, `UnbindMesh`, `DrawMesh*`,
  `DrawImmediate`, `SetBlendMode`, `SetCullFace`, `SetDepth*`, `SetWireframe`,
  `SetLineWidth`, `SetPointSize`, `SetViewport`, `SetScissor`, `EnableScissor`
  and `GetUniformLocationByResource`, then `GpuHandle` and the coexistence
  hooks from §4. The loose-uniform check becomes an error.
- **Immediate renderer.** About 400 lines of mirrored methods disappear from
  both renderer files.

#### S6 notes

**Status: implemented.** The legacy immediate API is gone: the `Draw` global, `Shader:start/stop/set*/iSet*/getVariable/
hasVariable/resetTexIndex`, `ShaderState`, `RenderState` (and its stacks), `Mesh:draw/drawBind/drawBound/drawUnbind`,
`LodMesh:draw`, `PrimitiveBuilder`, `GpuHandle`, `ShaderVarData`, and in `RenderCommand` (both backends, GL and wgpu
executors) `SetViewport/SetScissor/EnableScissor/SetBlendMode/SetCullFace/SetDepthTest/SetDepthWritable/SetWireframe/
SetLineWidth/SetPointSize`, `BindShader*/UnbindShader`, all 18 `SetUniform*`, `BindTexture*/UnbindTexture`, the
`GpuHandle` forms of the texture state and update commands, `BindMesh*/UnbindMesh`, `DrawMesh*`, `DrawImmediate`,
`GetUniformLocationByResource`. The coexistence hooks of section 4 are deleted (the pass-encoder flush before a non-pass
command stays: it is what orders a texture upload in the middle of a pass), as are the legacy texture-unit allocator
(`fixed_unit_mask`, `tex_index`), the state-cache write-through, `invalidate_pipeline` and `drop_unit_sampler`. Stats
that only the legacy path produced (`uniform_dedup_skips`) are gone with it.

*Lua API (deviation from section 3d).* `Imm` is a namespace of statics with the renderer injected (like `ClipRect` and
`Draw` were), not `Render.imm()` with methods: `Imm.Rect`, `Border`, `Image(tex, sampler, x, y, w, h, u0, v0, u1, v1,
color)`, `Icon`, `Shape(Shape.X, x, y, w, h, color, a, b, c, d)` (the shape parameters, see the `Shape` enum in `imm.rs`),
`Tri`, `TriGlow`, `Line(x1, y1, x2, y2, color, width)`, `LineGlow`, `Point`, `Box3`, `Line3`, `Point3`. There is no
`pushClip/popClip/setTransform`: `ClipRect.Push/Pop` already feed the scissor (below) and `pass:setUiTransform` is the
transform. `Font:draw(text, x, y, color)` is unchanged. Color is a `Color`, per-vertex; `Draw.Color/PushAlpha` state is
gone (`DrawEx` keeps its own alpha stack in Lua).

*Batching.* `ImmBatcher` (`render/gpu/imm.rs`) appends vertices of the current run to a CPU buffer. A run is keyed by
`(layout, pipeline, texture view + sampler, scissor)`; a different key, or anything else recorded into the pass (a
pipeline, a draw, an `alloc`, an input, a viewport or transform change, a flush), ends the run: its bytes are copied into
the vertex ring and one `PassCmd::DrawImm { layout, vertices: RingOffset, count }` is recorded (at most 8190 vertices
each). A quad is two triangles in the order of the old triangle fan, so the geometry is bit-identical. The batcher binds
its own pipelines and writes input slot 0 for textured runs, so the pass tracks `user_pipeline` (what Lua set) and
`bound_pipeline` (what the executor will have): a mesh draw re-emits the user's pipeline if a batch run changed it, and a
run marks the staged inputs dirty so the next draw sends them again. `DrawImm` is the one new `PassCmd`.

*Vertex layouts.* `Imm2D` is 64 bytes (the doc said 48): `pos: vec2, uv: vec2, color: vec4, params: vec4, params2:
vec4`. Triangle, wedge and the glow line need six values, so there are two parameter vectors. Attribute locations 0, 2, 3
(as for every mesh) and 11, 12 (`imm_params`, `imm_params2`). `vertex/imm2d.glsl` passes color and parameters as `flat`
varyings; `include/imm.glsl` declares them for the fragment side and every `ui/*` shader names its parameters with
`#define`s over them (`#define radius (imm_p.x)`, `#define color imm_color`). `Imm3D` is 36 bytes (position, uv, color):
the backdrop box (drawn with the pass's own pipeline, which must use `VertexLayout.Imm3D`) and the debug geometry. Both
layouts have a GL VAO whose attribute pointers are set per draw at the ring offset.

*Shapes.* `Shape` has one pipeline per value (shader + baked blend mode): `Solid`, `Image`, `Text` (alpha); `Box`,
`Circle`, `Grid`, `Hex`, `Icon`, `PanelGlow`, `PointGlow`, `RingGlow`, `RingDim`, `Triangle`, `Wedge`, `LineGlow`
(additive); `Panel`, `Point`, `Ring`, `Annulus` (alpha). The padding the shaders need around a shape is part of the
rectangle the caller passes (as before); the panel shader's `padding` is the constant 64. `fragment/ui/line.glsl` reads
the fragment position from the `pos` varying instead of rebuilding it from `origin` and `size`, which is the only change
in shape output (up to 12/255 on a few edge pixels of glow lines). `DrawEx.Hologram`, `ui/hologram`, `vertex/ui3D`,
`ui/logo`, `ui/shadow`, `ui/circle-old` and the loose-uniform `simple_color`/`simple_image` are deleted (no callers).
Mesh-based UI draws (the asteroid dots of `SystemMap`, the trails of `SystemMap3D`) are plain pipelines with a `Params`
block (`vertex/mappoints`, `vertex/hologram3d` declare it under `#group 2` and hand the color to the fragment shader as
the same `imm_color` varying).

*Lazy scissor.* `ClipRect` only edits its stack now. `Renderer::pass_apply_scissor` emits `PassCmd::SetScissor` right
before a draw or a batch run when the wanted scissor (computed against the pass's current viewport) differs from the one
last sent, so clip pushes and pops without a draw in between cost nothing, and the run key carries the scissor. A clipped
away rectangle has a negative extent, which GL rejects (and then keeps the previous scissor, which the old code did
silently): it is clamped to empty. `pass:setScissor` is unchanged.

*Glyph atlas (`font.rs`).* Each `Font` owns 1024x1024 `R8` pages. A shelf packer places a glyph (1 texel of padding;
a shelf is reused for glyphs up to 1.5x its height) and a new page opens when none fits. Glyphs are rasterized on demand
into a CPU copy of the page (coverage with the same gamma 1/1.8 as before, rounded to 8 bits as the driver did), and the
rows touched are uploaded once, before the draw, by the new `UpdateTexture2DRect` command (the existing update command
replaces the whole image; GL sets `UNPACK_ALIGNMENT` to 1 for it). A string is one run per page it touches, sampled with
`Samplers.Point`; glyph quads sit on whole pixels, so the sampled texels are exactly the ones the per-glyph textures
gave and the text pixels did not change. `ui/text` samples `.r`. `Font:draw` always blended with alpha whatever state
surrounded it, so `DrawEx.TextAdditive` was never additive; it still draws with alpha (there is no additive text).
`UIRenderer` (HmGui) draws panels, images, rects and text through the batcher; its images use `Samplers.Point` (what
`Tex2D.Load` textures sampled with).

*Lines and points.* 2D lines are quads (`Imm.Line`, width honoured; the glow line is one quad over its bounding box). 2D
points are small squares. 3D debug lines and points (`imm_debug_line3/point3`, used by the Rust debug draws of physics,
BSP, octree, box tree and `Mesh:drawNormals`) are camera-facing quads expanded on the CPU from the pass's camera and
viewport, so their pixel width is honoured too; `glLineWidth`, `glPointSize` and `Draw.SmoothPoints` are gone. The star
field was already a quad mesh (`pass:drawMesh`), so no point sprites were needed. `Physics:drawWireframes(eye)` lost its
`shader` argument and `BSP.Create(mesh)` its (hidden) renderer; the BSP debug draws, which set `wireframe` through the old
state stack, use `ImmDebugState { blend, depth_test, wireframe }` pipelines.

*Fullscreen compute.* `Mesh:computeAO`/`computeOcclusion` (the asteroid meshes) draw their fullscreen quad in a pass with
a `Params` block (`sDim`, `radius`) and `Samplers.Point` inputs (`fragment/compute/occlusion*.glsl` declare them under
`#group 2/3`).

*Shaders.* `create_shader` rejects a program with an active uniform that is neither in a block nor a sampler, with the
names in the error (the shader falls back to the error shader and the overlay shows it). The wgpu executor has no such
check. The loose-uniform effect shaders that active code used move to draw-block variants (`quad_draw`, `axis_draw`,
`quadpos_draw`, `*head_draw`, `*tail_draw`, `explosion_draw`; the originals that no active or ported code loads are
deleted).

*Legacy code touched.* `Legacy/.../Effects/Pulse.lua` (`Pulse.Render`, used by the WeaponSystem testbed) draws its heads
and tails through the additive pass with draw blocks, and `Effects/Explosion.lua` (loaded by `LoadInline('Legacy')` at
start-up) likewise; the testbed's `RenderState` wrap is gone. Not ported, and broken if reached: `Effects/Dust`,
`Entities/Ship/{Bay,Drone,Turret,Thruster}`, `Entities/Objects/{Nebula,Planet}`, `GameObjects/Material`,
`Systems/Gen/{Asteroid,DiffuseMap}`, `Systems/Overlay/GameView`, `Systems/CommandView/SystemMap` and
`Util/ShaderLocations` (all use the removed `Shader` methods or loose-uniform shaders; the modules still load).
`Profiler.TimeGPU` uses the new `Renderer:gpuFinish()`.

*Capture and validation additions.* `UiShapes` (every `DrawEx` shape, both text paths, clipping, alpha, an icon) and
`UiMaps` (the dots, the trail ribbon, an annulus and the 3D debug primitives) in `States/App/Tests` cover the UI paths.
Under `LTHEORY_CAPTURE`, `LTHEORY_CAPTURE_SEED` fixes the `LTheoryRedux` menu scene and `LTHEORY_CAPTURE_VIEW` picks the
menu view. `GenTex2D` and `AudioTest` define `onDraw`, which `Application` stopped calling long ago; they define
`onRender` now. GenTex2D's "black texture with one white quadrant" was the immediate draws running without a shader; with
`Imm` it generates its worn plate pattern.

*Cost* (frame 120, median of three runs on the dev machine; commands sent to the render thread per frame and draw calls):
PlanetTest 90 -> 82 commands, 26 -> 26 draws; Benchmark 90 -> 82, 33 -> 33; MoonTest 90 -> 82, 24 -> 24; PlanetTestRing 90
-> 82, 27 -> 27; SolarSystemPlayable 754 -> 85, 233 -> 26 (fps 318 -> 332, render idle 25% -> 31%); WeaponSystem 3784 ->
85, 1318 -> 212 (frame 31.9 -> 30.1 ms, render thread 3.3 -> 2.4 ms per frame). Main menu views at frame 400: Title 206 ->
85 commands and 65 -> 37 draws, Main 370 -> 85 and 105 -> 51, Newgame 1327 -> 85 and 381 -> 140, Loadgame 1971 -> 85 and
668 -> 113, Settings 429 -> 85 and 120 -> 54; `UiShapes` 863 -> 10 commands and 245 -> 29 draws.

*Pixels.* All six capture scenes are bit-identical to the baseline on both builds. UI output differs only by the glow line
shape above (`UiShapes`: RMSE 0.010, max 12, 0.0001% of pixels over 8) and the moving clock and pulse of the menu; the menu
views match their pre-S6 captures (RMSE <= 0.03). The 3D debug lines and points have no previous output to match (they
were 1 or 2 pixel GL lines).

*wgpu executor.* `DrawImm` reads the vertices back from the vertex-ring bytes and draws them through the old immediate
path of that executor, and the rect upload goes through its texture write; like the other pass commands since S3 these do
not render correctly there yet.

### S7. Samplers and views, finishing up
- After S6 every sample goes through a bind group with an explicit sampler,
  so texture-level sampler state is dead. Fold any remaining intent into
  sampler descs (in Material `textures`, `Samplers.*` presets), then delete
  the 70 `setMagFilter/setMinFilter/setWrapMode/setAnisotropy` and 10
  `setMipRange` call sites, the `SetTexture*Filter/WrapMode/MipRange/Anisotropy*`
  commands, and their GL and wgpu handlers.

### S8. Async readback
- **Engine.** Tickets via PBO + fence, `ReadTextureSync`, readback polling in
  `BeginFrame` (both backends).
- **Lua.** RenderCoreSystem auto-exposure (1 site, which replaces 128 reads
  per frame); 37 probe `:sample()` calls in 13 validation scenes (become
  `Renderer:readSync`); `Tex2D.ScreenCapture` (Application.lua:313); the
  `LTHEORY_CAPTURE` path in Rust (explicit sync).
- **Removal.** Delete `Tex2D:sample`, `SamplePixel2DByResource`,
  `ReadFramebufferPixels` and the per-kind `Read*Data` commands.
- **Verification.** Auto-exposure lags by 2–3 frames, so the frame-120
  baselines are re-captured after confirming visually that only exposure
  differs.

### S9. Mip chains and texture kinds
- `TexDesc { dim, format, size, mips, usage: SAMPLED|ATTACHMENT|COPY_SRC|COPY_DST }`
  at creation. Add real 1D/3D/cube on wgpu (gaps 7/8), `GenerateMips` as a
  blit chain on wgpu (gap 4), and a format-agnostic `Update*Data` payload
  (gap 20). Also fix `TexFormat::RG8 = gl::RGB` and replace `RGB8`, which has
  no wgpu equivalent, with RGBA8. Lua: the 16 `genMipmap` sites keep their
  name, and `Tex*.Create` gains an optional desc.

### S10. Hot reload
- The pipeline cache key gains the shader generation (gap 2). Bind groups
  rebuild on texture or shader generation change (gap 3). If a block's
  `BlockLayout` hash changes, `ShaderHotReload` asks each `MaterialType` to
  regenerate its ctype, copy fields by name from the old cdata, reallocate its
  arena slice and `commit()`. GL relinks through the same `create_shader`
  path, so block bindings and units are reapplied.

#### S7/S9/S10 notes

##### S7 (implemented)

Texture-level sampler state is gone. Removed: `setMagFilter`, `setMinFilter`, `setWrapMode`, `setAnisotropy` on
`Tex1D/2D/3D/Cube` (Lua and Rust), `Tex2D:setMipRange`, the commands `SetTexture2DAnisotropyByResource`,
`SetTexture2DMipRangeByResource`, `SetTexture{Mag,Min}FilterByResource` and `SetTextureWrapModeByResource` with their
GL and wgpu handlers, the `Renderer` methods in both renderer files, the executor's `note_mip_range` and the wgpu
executor's per-texture filter state (`recreate_sampler`, `texture_filter_modes`). The view binding keeps its own
`TEXTURE_BASE_LEVEL/MAX_LEVEL` cache (`mip_ranges`), which is now the only writer of those parameters.

*Nothing had to be folded.* Every sampled texture already went through an explicit sampler (S3 to S6: `Samplers.*`
presets, `MaterialType.textures`, `pass:setInputs`, `Imm.Image`), and a GL sampler object overrides the texture's own
parameters, so all 41 Lua call sites (15 files, 12 of them active) were dead. They were deleted, not translated, and
the effective filters are the ones the samplers already name: that keeps the accidents of S5 (nebula LUTs sample
`Samplers.Point`; `gen/moon`'s `baseMoonTex` is black) as they were. `Cache.Texture(name, true)` now only generates the
mip chain. The environment, moon, planet and generated cube textures are sampled with `LinearMipClamp`, 2D material
textures with `LinearMipRepeatAniso`, as before.

*Auto-exposure.* `tonemap` set the mip range of `buffer0` before its 128 `sample()` calls, but `sample()` reads level 0
through a framebuffer, so the range never reached it; the call is deleted and the mip chain is still generated for S8.
Auto-exposure is off in the default config, which is also why the captures do not depend on it.

*Verification.* All six capture scenes RMSE 0 and 13/13 supervisors, on both builds.

##### S8 (implemented)

*Commands.* `ReadTextureSync { src, region, format, reply_tx }` and `ReadbackAsync { src, region, format, slot }` replace
`ReadTexture1DData/2DData/3DData/CubeFaceData`, `SamplePixel2DByResource` and `ReadFramebufferPixels` in both renderer
files and both executors. Deviation from section 1.5: the source is a `ReadSource` (`Texture(ResourceId)` or `Backbuffer`)
and the part read is a `TexRegion { level, origin, size }` (the type `UpdateTexture` uses), not a `TexView` plus rectangle:
a region also says "all layers of a volume" and "the 4th face of a cube", which `get_data` needs. `format` is the layout
of the **result** (`TexFormat`: tightly packed rows, row 0 of the texture first, no GL enum anywhere); the executor
converts from the texture's own format (GL: `glReadPixels` does it, as `sample()` always relied on; wgpu: `convert_format`
in `tex_desc.rs` after the rows are unpadded). Depth formats cannot be read back (nothing active does).

*Lua.* `Renderer:readSync(view, x, y, w, h, fmt) -> Bytes` (empty on failure; stalls, tools and tests only) and
`Renderer:readAsync(view, x, y, w, h, fmt) -> ReadbackTicket`. The view picks the mip level (`tex:mipView(l)`), cube face
(`cube:faceView(f)`) or volume layer (`vol:layerView(z)`); `x, y, w, h` are texels of that level, clamped to it. A
ticket has `:ready()` (the read is over, successfully or not; it never blocks), `:failed()`, `:data()` (a `Bytes` copy,
empty until ready or after a failure), `:getWidth()/getHeight()` and `:free()` (`release`; the garbage collector frees
the ticket anyway, and a read still in flight just drops its pixels when it arrives). The Rust side is
`Renderer::read_texture_sync/read_texture_async(ReadSource, TexRegion, TexFormat)`; `Tex1D/2D/3D/Cube::get_data` (and
`getDataBytes`, `Tex2D:save`, `TexCube:save`, `Mesh::compute_ao/occlusion`, `SDF.FromTex3D`) are one `read_layout` call over
it (`(PixelFormat, DataFormat)` mapped to a `TexFormat` by `format_for_layout`; a three-component layout has none and
reads as zeros with a warning). `Tex2D.ScreenCapture` (and so `LTHEORY_CAPTURE`, which is Lua in `Application:captureTick`,
not Rust) reads `ReadSource::Backbuffer` synchronously.

*GL.* Sync: the texture goes onto the scratch read framebuffer of `CopyTexture` (`FramebufferTexture1D/2D/Layer` or a cube
face target, at the region's level; one `glReadPixels` per layer of a volume) and is read with the format and type of
`TexFormat::to_gl_formats`. Async: the same read into a fresh `GL_PIXEL_PACK_BUFFER` (so the driver queues the copy and
returns), then `glFenceSync` and `glFlush`. `BeginFrame` polls the pending list with `glClientWaitSync(fence, 0, 0)` and
maps and copies the buffers that are done into the slot (`poll_readbacks`); it never waits. The poll is part of the
executor's `BeginFrame` command, so both backends do the same (the render thread in the threaded one, inline in the
immediate one). Measured latency (`TexKinds`, GL threaded, main-thread frames between request and `ready`): 6 (the main
thread runs up to three frames ahead of the render thread, which polls once per frame).

*wgpu* (`command_executor_wgpu/readback.rs`). `copy_texture_to_buffer` into a `MAP_READ` staging buffer with
`bytes_per_row` padded to 256, `map_async`; sync polls the device (5 s cap), async polls once per frame at `BeginFrame` and
finishes the jobs whose callback has fired (2 frames in `TexKinds`). Rows are unpadded, then brought from the storage
format to the `TexFormat` layout (`R32F` and `RGBA32F` live in RGBA16F textures: halves widen to f32; the surface is
BGRA: red and blue swap), then to the requested format. Any texture kind, level, cube face and volume layer works; the
surface is configured with `COPY_SRC` where the adapter allows it, so `ReadSource::Backbuffer` can read the frame that is
being drawn. `TexKinds` passes all 23 checks on wgpu, including the RG8, R32F and cube-face readbacks that failed before
(gap 1 is closed).

*Auto-exposure.* `RenderCoreSystem:tonemap` no longer reads 128 texels, one synchronous round trip each. Each time no read
is in flight it generates the mip chain of `buffer0` and issues one `readAsync` of the small mip (at most 512 px, at least
mip 2, `RGBA8`); when the ticket is ready (2 to 6 frames later) the same Lua maths (128 random samples, cap, lowest 65%,
log-average) runs over its bytes and sets `autoExposure.target`; the adaptation toward it runs every frame as before.
**Two behaviours that were already wrong:** `sample()` read level 0 at coordinates meant for the small mip, so it measured
only the top-left `w x h` texels of the full image (a 320 x 180 corner of 1280 x 720), while the new read is the mean of
the whole frame; and `glGenerateMipmap` had been generating nothing for any texture a sampling view had narrowed to one
level (`TEXTURE_MAX_LEVEL` is the view's level), which is every post-processing buffer after its first frame, so
`GenerateMips` on GL now resets the range to the full chain first (the next view binding sets its own range again). The
`RGBA8` read keeps the old quantization: a uniform frame is either exactly 0 (target = `maxTarget`) or at least 1/255
(target = `minTarget`), so the dark end of the range is coarse. Reading `RGBA16F` or `RGBA32F` is the obvious follow-up
and would change exposures.

*Validation scenes.* The 40 `:sample()` calls in 14 files of `States/App/Rendering` (the 13 supervised scenes and
`Upscale`) are `ProbeRead.sample(tex, x, y)` (`Rendering/ProbeRead.lua`): one `readSync` of one texel as `RGBA8`, with
the y flip `sample()` had. `TexKinds` gained 13 checks: `readSync` of a whole texture, a sub-rectangle and a mip level,
`RGBA16F` as floats and as bytes, `R32F` as floats and as clamped bytes, cube `faceView`s, a volume `layerView`, a part of
a 1D texture, and three `readAsync` tickets polled over the following frames.

*Removed.* `Tex2D:sample` (Rust `sample_pixel`, `Tex2D_Sample`, its `ffi_ext` wrapper), `SamplePixel2DByResource`,
`ReadFramebufferPixels`, `ReadTexture1DData/2DData/3DData/CubeFaceData`, their `Renderer` methods in both backends, the
GL and wgpu `cmd_*` handlers, `texel_buffer_size`, and the wgpu pixel decoding (`decode_sample_pixel`, `float_to_u8`; the
half-to-float conversion lives in `tex_desc.rs` now). No readback payload carries a GL enum any more.
`CopyTexture2DFromFramebufferByResource` (`deep_clone`) is not a readback and stays.

*Test.* The `AutoExposure` scene (`States/App/Tests`) runs `tonemap` with auto-exposure on over a black and then a
mid-gray frame (150 frames each, fixed dt) and logs target and adapted exposure per frame.
`tools/render_validation/auto_exposure_test.py` compares the result with `baseline/auto_exposure.json`, recorded with the
synchronous code at `197d5bf6`: targets 2.5 and 0.4 exactly; adapted exposure at the end of the phases 2.0725 and 1.1885
before, 2.054 and 1.209 now (tolerance 0.05, the lag of the asynchronous version). A uniform frame is the one input whose
measurement does not depend on the part of the frame that is sampled, so it is the one on which old and new must agree.
The same scene times `tonemap` on the main thread: 12.6 ms per frame before, 0.07 ms after (threaded; 1.1 ms on the
immediate backend, where the GL calls run inline).

*Cost* (frame 400, three runs each, threaded GL, auto-exposure switched on through a temporary `LTHEORY_AUTOEXPOSE` check
in `PostFxConfig` that is not committed): PlanetTest 15.6 to 17.0 ms per frame before, 3.3 ms after (2.8 ms with
auto-exposure off); WeaponSystem 39.4 to 44.1 ms before, 22.8 to 25.3 ms after (23.0 ms off).

*Pixels.* Auto-exposure is off in all capture scenes, so all six are bit-identical to the baseline on both builds (RMSE 0)
and 13/13 supervisors pass on both builds. The `GL_INVALID_OPERATION` that `LTHEORY_GL_CHECK=1` logs once per run
("commands before this PassCommands") comes from the first `SwapBuffers` and is in `197d5bf6` too (checked with a worktree
at that commit); the 12-scene frame-40 smoke shows no new error line on the default and immediate builds.

##### S9 (implemented)

*Descriptions.* `TexDesc { dim, format, size: [u32; 3], mips, usage }` (`render/gpu/tex_desc.rs`) is what the executors
create a texture from: one command, `CreateTexture { id, desc, data }`, replaces `CreateTexture1D/2D/3D/Cube`. `dim`
is the existing `TexDim` (D1, D2, D3, Cube), `usage` a `TexUsages` bit set (`TexUsage.Sampled/Attachment/CopySrc/
CopyDst` in Lua; the default is everything, minus attachment for 1D). `mips` is resolved when the desc is built:
0 asks for the full chain, a larger number than the chain is clamped, depth formats always get 1. GL has either one
level or the whole chain (a request for more than one allocates every level, empty, so `GenerateMips` and per-level
render targets have storage; `glGenerateMipmap` still fills them). wgpu allocates exactly `mips` levels, except for 1D
textures, which WebGPU limits to one.

*Updates.* One command, `UpdateTexture { id, region, data }`, replaces `UpdateTexture2DDataByResource`,
`UpdateTexture2DRect` and the 1D, 3D and cube-face variants. `TexRegion { level, origin, size }` addresses a box of one
mip level (a cube face is the z layer, `face_layer(face)`), and **`data` is already in the texture's own `TexFormat`
layout, tightly packed**: no GL enum is left in the payload. `convert_texels` (`tex_desc.rs`) turns the engine's
`(PixelFormat, DataFormat)` source layouts into that layout on the main thread: components the source lacks are 0 and
alpha 1 (what GL did for RGB data into RGBA8), integer and float sources are normalized and rounded as GL does (round
to nearest), f32 to half is round to nearest even, and a source that already is native (`RGBA`/`U8` into RGBA8, `RG`/
`Float` into RG32F, ...) is passed on without a copy. `Tex*::set_data` and `Tex2D::load` use it; the Lua call sites did
not change. `CopyTexture2DFromFramebufferByResource` takes a `TexFormat` instead of a GL internal format. The
readback commands still spoke `(pixel_format, data_format)`; S8 replaced them. (`TexCube::get_data` passed its
`TexFormat` argument as a GL pixel format, which is `GL_INVALID_ENUM` and a zero-filled result; it derives the pixel
format from the component count now, like `set_data`.) GL uploads set `UNPACK_ALIGNMENT` 1 instead of leaving 4 behind
after a glyph upload.

*Formats.* `TexFormat::RG8` was `gl::RGB` (the value used as the internal format of cube textures and of
`CopyTexImage2D`); it is `gl::RG8` now. `RGB8` is removed: it has no wgpu equivalent and the only users were
`ColorLUT` (a 1D LUT, now `RGBA8`, still uploaded as `RGB`/`Float` and converted, alpha 1) and `TexCube::load`
(JPEG faces, now RGBA8; `Tex2D::load` already stored RGBA8). The LUT pixels are bit-identical (captures with a nebula
sky: RMSE 0 on both builds).

*Lua.* `Tex1D/2D/3D/Cube.Create(..., desc)` takes an optional `{ mips = true | <levels>, usage = <bits> }` (no `mips` is
one level), through new `Tex*_CreateDesc(r, ..., mips, usage)` functions. `genMipmap` keeps its name on all four.
Sites that sample a mip chain now say so at creation: the post-processing buffers of `RenderCoreSystem`, GenTex2D's
texture, both cubes of Nebula2's ping-pong, `TexGen.Cube { mips = true }` and the irradiance cube of `GenIRMap`.
Loaded textures stay single level until `genMipmap` (GL), because allocating a chain would also make a texture complete
that was sampled with a mip sampler before it had one (it read black).

*wgpu.* The texture code is in `command_executor_wgpu/tex.rs`: real `D1`, `D2`, `D3` and cube (six-layer 2D with a cube
view) textures (gaps 7 and 8; the 3D placeholder is gone), updates by region with `write_texture` (rows are repacked by
wgpu, so no 256-byte padding), the 16F backing of `R32F`/`RGBA32F` converted on update, and `GenerateMips` as a blit chain
(gap 4): one pass per level and face (or 3D slice, with `depth_slice`), sampling the level above with a bilinear
filter (nearest for 32F formats), pipelines cached per target format. A texture created with one level has no chain to
fill: `GenerateMips` warns and does nothing, so the sites above ask for mips at creation. Samplers
(`CreateSampler`) are real `wgpu::Sampler`s now, bound with the texture of the unit they came with (`SetInputs`,
`SetBindGroup`) unless the format is 32F: filters, wrap, anisotropy and LOD clamps reach the GPU (gaps 5, 6 and 18).
These paths run (`TexKinds` under `LTHEORY_WGPU=1` creates 2D, 3D, 1D and cube textures, uploads, and generates mips
without a validation error) but, like everything the wgpu executor does since S3, they do not render scenes correctly.

*Probe.* `States/App/Tests/TexKinds` creates one texture of each kind with mips, uploads in layouts that differ from
the texture's format (RGB float into RGBA8, RG8, R32F, per cube face), reads everything back and logs a PASS/FAIL line.
It passes on GL; on wgpu the readbacks of RG8, R32F and cube faces failed until S8 (gap 1), and pass now.

##### S10 (implemented)

*What reload already did.* `Shader::reload` has made a new GPU resource (a new `ResourceId`, a new link through the
same `CreateShader` path, so block bindings and sampler units are applied again) and dropped the old one since S3. That
refreshed everything keyed by the id (`Pipelines.get`, `shader:blockType`, the pipeline cache), but not what held on to
the shader's layout: a `Material` kept the `MaterialParams` size, its member offsets, the index of its block and its
texture slots from the day it was made, and `ShaderHotReload` called a `material.reloadShader` that no longer existed. A
reload that moved a member, or added a sampler, wrote the old bytes into the new block.

*Generations.* `PipelineDesc` has `shader_generation` (`Shader::generation()`, bumped by every successful reload), so
the generation is part of the pipeline cache key whether or not a reload keeps the resource id (`PipelineDesc::
for_shader`; `Material::pipeline()` follows both). The wgpu executor counts how often each shader or texture resource
was created (`resource_generations`) and puts the count into its pipeline key and into its bind group key, for the
shader and for every bound texture; a creation under an existing id drops the cached groups (gaps 2 and 3). The
vestigial `ReloadShader` command (a second reload path that nothing called, with its `hot_reloaded_shaders` pairs in both
executors, the `ShaderReloadResult` channel to the render thread and `Renderer::reload_shader`) is deleted: it was the
stale-pipeline behaviour gap 2 describes.

*Layout hash.* `BlockLayout::layout_hash()` (FNV-1a over the block's name and size and every member's name, type,
offset, array length and strides) changes exactly when a ctype made from the block would. `shader:blockHash(name)` and
`shader:blockNames()` expose it; `Cache.ReloadShader(key)` (used by `ShaderHotReload` and by `Cache.ReloadShaders`)
compares the hashes of all blocks before and after the reload and hands the set of changed blocks to the reload hooks
(`Cache.OnShaderReload`). A changed block nobody claims is logged: LuaJIT types made from it (a `Params` ctype held in a
module local) are stale until restart. Only `MaterialParams` of material types is claimed.

*Materials.* `Material.OnShaderReloaded` (the hook) asks every `MaterialType` that draws with the shader to
`regenerate()` its ctype (`paramsType`, `paramsHash`), then refreshes each live instance: `Material::refresh_shader`
(Rust) reads the new layout, copies the parameters into a block of the new size **by member name** (same name, type,
array length and strides; `BlockLayout::copy_members`; new members are zero, and Lua then applies the type's `defaults`
to them), keeps the textures by sampler name (one the shader dropped, or now declares with another dimension, is
dropped and reported), gives back the old arena slice and bind group and re-points the pipeline template. Lua recasts
`params()` to the new ctype, and `commit()` writes one new slice and makes the new bind group. The report line logs
`changed`, `copied`, `added`, `dropped` and `textures_dropped`. Hot reload runs in `onPreRender`, outside any pass.

*Test.* `tools/render_validation/hot_reload_probe.py` runs `PlanetTest` four times to frame 9000 and edits
`material/planet.glsl` 8 s after the start (while the scene runs, `LTHEORY_CAPTURE` set only for the picture): untouched
(reference); albedo tinted red (differs from the reference only inside the planet's bounding box, RMSE 5.07); tinted
and reverted (identical); and a new *first* member `vec4 hotProbe` in `MaterialParams`, which moves every other member
by 16 bytes (identical to the reference, so the copy by name worked; the log shows `changed=1 ... added=hotProbe`). A
plain run without `LTHEORY_CAPTURE` for 20 s with four edits in a row (a comment, the tint, the new member, the revert)
reloaded four times without an `ERROR` or panic line, `added=hotProbe` after the third and `dropped=hotProbe` after the
fourth.

---

## 6. Open questions for a human

1. **Legacy modules on active paths.** Active states require
   `Legacy.Systems.Gen.{Generator,Starfield,Primitive,GenUtil}` (7/7/5/4
   requirers), `Legacy.Systems.Gen.Nebula.Nebula1`,
   `Legacy.GameObjects.Material` (PlanetTest) and
   `Legacy.Systems.Overlay.GameView` (PhysicsTest), and LTheoryRedux uses
   `LoadInline('Legacy.Systems')`. Proposal: promote and port Gen (S5), drop
   PhysicsTest's GameView path, and switch PlanetTest to MaterialType. Is
   that OK?
2. **Delete instead of port** `Render/RenderPipeline.lua`,
   `Render/ImageFilter.lua` and `States/App/Examples/HmGui/ScrollArea.lua`'s
   use of RenderPipeline?
3. **Coexistence between S3 and S6.** Old commands keep working, unchanged,
   while call sites migrate (§4 hooks, deleted in S6). The alternative is one
   much larger step. Is that acceptable under the no-shim rule?
4. **wgpu executor during the GL refactor.** Recommendation: keep
   `command_executor_wgpu.rs` compiling and implement each new command as it
   lands, since they map 1:1, deleting old handlers in the same step. wgpu
   captures may stay red until S6. The alternative is to freeze it behind a
   feature until the swap.
5. **Auto-exposure re-baseline in S8**, and whether `compare.py` thresholds
   should be loosened for that one step.
6. **Shape and wide-line output on GL.** Lines become quads in `Imm`, so
   GL's `glLineWidth` is no longer used. Starfield and point sprites
   (`SetPointSize`) need a quad expansion as well. Are pixel diffs in those
   scenes acceptable?
7. **Coordination.** Other agents are editing the validation scenes and
   shaders now. S2 and S3 rewrite those scenes and S3 rewrites shader
   includes. We need an owner and a freeze window.
