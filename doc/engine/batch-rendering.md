# Batch Rendering & GPU Instancing

A CPU-side layer on top of the [render thread](render-thread.md) for
submitting many entities as one accumulate-cull-draw pass: `RenderBatch`
(frustum-culled, one draw call per entity) and `InstanceBatch` (one
`glDrawElementsInstanced` call for many copies of the same mesh).

`RenderBatch` is used, but not as a draw path: `RenderCoreSystem.lua`'s
`cullPassLists` uses it purely as a **cull + sort service** —
`beginBatch`/`addCullEntity`/`cullBatch` — then still issues its own
`sh:start()` + per-entity material uniforms + `mesh:draw()` for the
survivors, in `renderInOrder`. The other half of `RenderBatch`, the
draw-emitting `flushBatch`/`process_batch_intern`, has no Lua call site (see
"Why `flushBatch` doesn't drive the scene" below). `InstanceBatch` has no
Lua call site at all (see its section).

## RenderBatch

`render/thread/render_batch.rs`. Accumulates entities plus the camera used
to cull them, for one `begin`/`add`.../`flush` (or `cull`) cycle:

```rust
pub struct RenderBatch {
    pub entities: Vec<EntityRenderData>,
    visible: Vec<u32>,        // indices into `entities`, filled by cull_and_sort
    pub camera: CameraRenderData,
    pub stats: BatchStats,
}
```

`EntityRenderData` (`entity_render_data.rs`) — everything needed to cull and
draw one entity, keyed on `ResourceId` (not raw GL handles):

```rust
pub struct EntityRenderData {
    pub transform: Mat4,
    pub bounds_center: Vec3,   // world-space, for culling
    pub bounds_radius: f32,
    pub mesh_id: ResourceId,
    pub index_count: i32,
    pub shader_id: ResourceId,
    pub sort_key: u32,        // lower = rendered first
    pub user_id: u32,         // opaque caller tag, echoed back by cull_batch
}
```

`CameraRenderData` (`camera_render_data.rs`) — view/projection/eye plus the
6 frustum planes extracted from the view-projection matrix, normalized by
the plane's normal length only (`plane.truncate().length()`, not
`Vec4::normalize()`, which would incorrectly fold `d` into the divisor);
`sphere_in_frustum(center, radius)` is the cull test.

### Culling and sorting (`RenderBatch::cull_and_sort`)

Shared by both the cull-only path (`cull_batch`) and the draw-emitting path
(`process_batch_intern`):

1. For each entity in `entities`: frustum-cull via `camera.sphere_in_frustum`,
   **unless `bounds_radius < 0.0`** — a sentinel meaning "never cull" (used by
   callers with no bounds source for an entity). Survivors' indices go into
   `visible`; `stats.entities_culled`/`_visible` are updated.
2. `visible.sort_by_key(|&i| entities[i].sort_key)` — sorts the 4-byte index
   array, not the ~100-byte `EntityRenderData` records, and only the
   survivors. `sort_by_key` is stable: entities with equal `sort_key` keep
   their relative (insertion) order — callers that want to disable
   shader-sorting for a subset (e.g. alpha-blended geometry, where
   reordering changes which draw wins at equal depth) submit a constant
   `sort_key` for that subset instead of relying on frustum order.

`reset(view, projection, eye)` reinitializes a batch in place (clearing
`entities`/`visible`, rebuilding `camera`, zeroing `stats`) without
reallocating its backing `Vec`s — `begin_batch` calls this when a batch
already exists, only calling `RenderBatch::new` on first use.

### Flow (`process_batch_intern`, in each backend)

1. `batch.cull_and_sort()`.
2. For each surviving entity, in `batch.visible()` order:
   - On a shader change: `RenderCommand::BindShaderByResource { id:
     entity.shader_id, shader_key: None }`.
   - Per-draw transform: `RenderCommand::SetUniformMat4ByGenericName` for
     `mWorld` and `mWorldIT`. View/projection come from the Camera UBO (see
     `doc/engine/shader-system.md`), not a per-draw uniform, so only the
     world matrix and its inverse-transpose are needed here.
   - `RenderCommand::DrawMeshByResource { id: entity.mesh_id, index_count,
     primitive: Triangles }`.
3. `batch.entities.clear()` — matches the old `drain(..)` semantics: the
   batch is empty once processed.

`GenericUniformName` (`render_command.rs`) is a `Copy` enum with variants
`MWorld`/`MWorldIT` — used instead of an `Arc<str>` name or a per-thread
cache because it costs nothing to construct or send across the channel, and
the batch path only ever needs these two fixed names.

### Why `flushBatch` doesn't drive the scene

`process_batch_intern` emits only `BindShaderByResource` + `mWorld`/
`mWorldIT` + `DrawMeshByResource`. That's not enough to reproduce what
`RenderCoreSystem.lua::renderInOrder` actually does per mesh:

- **Textures.** `ShaderState::start` (`shader_state.rs`) replays every
  `(uniform index, texture)` pair recorded in `ShaderState.elems` — that's
  where a material's textures get bound. The batch path only issues
  `BindShaderByResource`, which never replays this.
- **Sampler auto-vars.** `Shader::start` (`shader.rs`) resets `tex_index` to
  0 and unconditionally re-applies resolved sampler auto-vars (`envMap`,
  `irMap`, ...) because their texture units are reallocated on every bind.
  Skipping this (as the batch path does) leaves samplers pointing at
  whatever unit a previous draw left bound.
- **Per-entity material uniforms.** `EntityRenderData` has no field for the
  ~15 non-transform uniforms `RenderCoreSystem.lua`'s materials set per
  entity or per material (`scale`, `origin`, `rPlanet`, `rAtmo`, `time`,
  `color1..4`, `planetQuat`, ...; see `script/Shared/Definitions/MaterialDefs.lua`).

Also, unlike every other Renderer command, `flush_batch`'s commands land in
`RendererData::command_buffer`, which only `flush_intern` drains — and
nothing calls `Renderer:flush()`. So today `flushBatch` is additionally
inert: its commands accumulate and are never sent.

Turning `flushBatch` into a real draw path would need, in order: (1) route
`command_buffer` through `submit` at the point of the call instead of the
never-invoked `flush_intern`; (2) a `RenderCommand` that reproduces
`ShaderState::start` (replay `elems`, reset `tex_index`, re-apply sampler
auto-vars); (3) a way for `EntityRenderData` to carry a per-entity uniform
payload. Note (3) would have to be recorded before culling, so a Rust-side
draw path would lose the CPU-side saving the cull-only design gets for
free (uniforms are never computed for entities that get culled).

### `BatchStats`

`render/thread/batch_stats.rs`, FFI-exposed getters for all six fields:
`entitiesSubmitted`, `entitiesVisible`, `entitiesCulled`, `totalEntities`,
`commandsGenerated`, `batchesProcessed`.

`Renderer:getBatchStats()` returns a pointer into the *active* batch, so it
only reflects whichever batch was processed last. `RenderCoreSystem.lua`
does not use it for this reason — `cullPassLists` accumulates its own
`self.cullStats` across all three blend-mode buckets instead (surfaced via
`RenderCoreSystem:getCullStats()`, shown in several test states' HUDs as
`Culled: n / m`).

### Renderer FFI

`render/thread/renderer_ffi.rs`:

```lua
Renderer:beginBatch(view, projection, eye)               -- starts/resets accumulation; eye: Vec3f
Renderer:addEntity(transform, boundsCenter, boundsRadius,
                    meshId, indexCount, shaderId, sortKey, userId)
Renderer:addCullEntity(boundsCenter, boundsRadius, sortKey, userId)
                                                          -- cull-only entry: no mesh/shader to draw
Renderer:cullBatch(outIndices, outIndices_size)          -- cull + sort; writes survivors' userIds
                                                          -- into outIndices (sort-key order), returns
                                                          -- the count written. Does not clear the batch.
Renderer:flushBatch()                                    -- runs process_batch_intern, emits commands
                                                          -- (see "Why flushBatch doesn't drive the scene")
Renderer:getBatchStats()                                 -- BatchStats for the active batch, or nil
```

`meshId`/`shaderId` are `ResourceId`s passed as plain `u64` (LuaJIT FFI has
no `ResourceId` type of its own). Obtain them from:

```lua
mesh:resourceId()      -- Mesh_ResourceId(self, Renderer); lazily creates the GPU resource if needed
shader:resourceId()    -- Shader_ResourceId(self); the shader's resource is already created eagerly
```

`boundsCenter`/`eye` are `&Vec3` (`Vec3f` in Lua), not separate x/y/z
scalars, matching the convention used elsewhere in the FFI (e.g.
`RigidBody:applyForce`). `cullBatch`'s generated prelude asserts the output
pointer is non-null and its length is `> 0` — callers must not invoke it
with a zero-length buffer.

### `RenderCoreSystem.lua::cullPassLists`

The actual Lua call site. Once per frame, per non-empty blend-mode bucket
built by `buildPassLists`:

1. `Renderer:beginBatch(view, proj, ZERO_EYE)` — camera-relative, so `eye`
   is always the zero vector (matches `CameraManager:beginDraw`'s
   camera-relative view).
2. `Renderer:addCullEntity(...)` per entry, with:
   - bounds centre = the entity's rigid-body position, camera-relative;
   - bounds radius = the *mesh's* origin-centred bounding radius (`|mesh
     centre| + mesh:getRadius()`, computed per mesh, not per entity — an
     entity's meshes can differ wildly in extent, e.g. a planet's atmosphere
     shell vs. its surface) scaled by the rigid body's scale factor (the
     same factor `RigidBody::get_to_world_matrix` bakes into `mWorld`);
   - `radius = -1` (never-cull) when the entity has no rigid body to source
     bounds from.

     The bound intentionally comes from the mesh, not the physics collider:
     the two are unrelated in general (a collider is sized for collision
     response, not visual extent) and PlanetTest's ring is a concrete case
     where they diverge by orders of magnitude.
   - sort key = a dense per-(vertex-shader, fragment-shader) id, except for
     the alpha blend bucket, which is pinned to a constant key to preserve
     draw order (shader-sorting blended geometry would change which draw
     wins at equal depth — a pixel change, not just a reorder).
3. `Renderer:cullBatch(buf, n)` into a per-blend-mode scratch
   `uint32_t[?]` (grown by doubling, never shrunk, reused across frames).

`renderInOrder` then walks `buf[0..vis-1]` (falling back to the full,
unculled list when culling is off or `Renderer.cullBatch` doesn't exist)
and runs its normal per-entry `sh:start()` / material-uniform / `mesh:draw()`
sequence only for survivors.

## InstanceBatch

`render/thread/instance_batch.rs` — true GPU instancing via
`RenderCommand::DrawInstancedWithData` (persistent instance VBO + attribute
divisors in `command_executor_gl.rs`), for drawing many copies of one mesh
with per-instance transform + color in a single draw call:

```lua
local batch = InstanceBatch.Create(mesh, Renderer, CmdPrimitiveType.Triangles)
batch:addInstance(transform, r, g, b, a)   -- queue one instance; no GL effect yet
batch:draw(Renderer)                        -- one glDrawElementsInstanced for all queued instances
batch:clear()                               -- drop queued instances without drawing
batch:flush(Renderer)                       -- draw() + clear()
batch:instanceCount()
```

One `InstanceBatch` is bound to one mesh's `ResourceId` and index count at
creation (`Create` lazily creates the mesh's GPU resource, like
`Mesh:drawBind()`); switching meshes requires a new `InstanceBatch`.

**Deliberately unused — zero call sites tree-wide, in either Lua or Rust.**
The two existing bulk-asteroid renderers each need something `InstanceBatch`
doesn't provide, and adopting it would regress both:

- `AsteroidInstancedRenderer.lua` writes a real per-instance `scale` into
  each `InstanceData` slot (the instanced shader reads it from attribute 9,
  for size diversity). `InstanceBatch::add_instance` hardcodes
  `scale: 1.0` (`InstanceData::from_transform_color`) — every instance would
  render at the same size. It also fills one `ffi.new('InstanceData[?]')`
  array and makes a single FFI call per group; `InstanceBatch::add_instance`
  is one FFI call per instance.
- `AsteroidBeltRenderer.lua` doesn't use `InstanceData` at all: it
  precomputes each asteroid's transform into a static `RGBA32F` data
  texture and streams only a 4-byte `u32` index per visible asteroid per
  frame, vs. `InstanceData`'s 84 bytes — a ~21x bandwidth difference that is
  the reason 100k+-asteroid belts are viable on the main thread at all.

Adopting `InstanceBatch` for either would need a per-instance `scale`
field plus a bulk `add_instances(&[InstanceData])` entry point at minimum,
and wouldn't help the belt renderer regardless (different data shape
entirely). Until then, treat `InstanceBatch` as available capability, not
a migration target for either renderer.

### Instanced vertex attributes

GLSL 330 has no `layout(location=N)`, so attribute locations are bound by
name via `gl::BindAttribLocation` before linking
(`command_executor_gl.rs::create_shader`). Locations 0-3 are the standard
per-vertex attributes (`vertex_position`/`vertex_normal`/`vertex_uv`/
`vertex_color`); locations 4-8 are the per-instance attributes
(`instance_matrix_col0..3`/`instance_color`) an instanced shader must
declare under those exact names to receive instance data. Binding
unconditionally is a no-op for shaders that don't declare those names.

`res/shader/include/instanced.glsl` declares `mWorld`/`mWorldIT` as local
`mat4` variables built from the per-instance attributes inside a
`VS_INSTANCED_BEGIN` macro (a uniform can't vary per-instance within one
instanced draw call, unlike `include/vertex.glsl`'s plain
`uniform mat4 mWorld`). `res/shader/vertex/wvp_instanced.glsl` mirrors
`wvp.glsl`, swapping `#include vertex`/`VS_BEGIN`/`VS_END` for
`#include instanced`/`VS_INSTANCED_BEGIN`/`VS_INSTANCED_END`.

## Quick Reference

| Concern | File |
|---|---|
| Entity accumulator + camera + cull/sort | `render/thread/render_batch.rs` |
| Per-entity data | `render/thread/entity_render_data.rs` |
| Culling camera + frustum test | `render/thread/camera_render_data.rs` |
| Stats | `render/thread/batch_stats.rs` |
| Batch flow (`process_batch_intern`) + FFI | `render/thread/renderer_threaded.rs` / `renderer_immediate.rs`, `renderer_ffi.rs` |
| Lua cull+sort call site | `script/Modules/Rendering/Systems/RenderCoreSystem.lua` (`cullPassLists`, `buildPassLists`) |
| GPU instancing | `render/thread/instance_batch.rs`, `instance_data.rs` |
| Instanced GLSL | `res/shader/include/instanced.glsl`, `res/shader/vertex/wvp_instanced.glsl` |
| Obtaining `ResourceId`s from Lua | `Mesh::resource_id` (`render/mesh.rs`), `Shader::resource_id` (`render/shader.rs`) |
