# Render API v2 — wgpu-shaped interface on the GL renderer

Status: design. S2 (render passes and attachment views) and S3 (binding model, pipelines, samplers, views, frame group) are implemented; later steps are not. Companion to `wgpu-migration-gaps.md`,
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
format-agnostic in S9), `DestroyResources`, `ReloadShader`, `Resize`,
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
  frame. No fences yet (S4).
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
