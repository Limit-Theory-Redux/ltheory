# Batch Rendering & GPU Instancing

A CPU-side layer on top of the [render thread](render-thread.md) for
submitting many entities as one accumulate-cull-draw pass: `RenderBatch`
(frustum-culled, one draw call per entity) and `InstanceBatch` (one
`glDrawElementsInstanced` call for many copies of the same mesh). Both are
implemented and FFI-exposed, but **no current Lua call site uses either** —
`RenderCoreSystem.lua::renderInOrder` still issues one shader start + one
`mesh:draw()` per entity directly. They're available capability, not wired
into the live render path.

## RenderBatch

`render/thread/render_batch.rs`. Accumulates entities plus the camera used
to cull them, for one `begin`/`add`.../`flush` cycle:

```rust
pub struct RenderBatch {
    pub entities: Vec<EntityRenderData>,
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
}
```

`CameraRenderData` (`camera_render_data.rs`) — view/projection/eye plus the
6 frustum planes extracted from the view-projection matrix, normalized by
the plane's normal length only (`plane.truncate().length()`, not
`Vec4::normalize()`, which would incorrectly fold `d` into the divisor);
`sphere_in_frustum(center, radius)` is the cull test.

### Flow (`process_batch_intern`, in each backend)

1. Sort `batch.entities` by `sort_key`.
2. For each entity, `drain`ed in order:
   - Frustum-cull via `camera.sphere_in_frustum`; culled entities only
     increment `stats.entities_culled`.
   - On a shader change: `RenderCommand::BindShaderByResource { id:
     entity.shader_id, shader_key: None }`.
   - Per-draw transform: `RenderCommand::SetUniformMat4ByGenericName` for
     `mWorld` and `mWorldIT`. View/projection come from the Camera UBO (see
     `doc/engine/shader-system.md`), not a per-draw uniform, so only the
     world matrix and its inverse-transpose are needed here.
   - `RenderCommand::DrawMeshByResource { id: entity.mesh_id, index_count,
     primitive: Triangles }`.
3. Stats (`entities_submitted`/`_visible`/`_culled`/`total_entities`) land in
   `batch.stats`, readable via `getBatchStats()`.

`GenericUniformName` (`render_command.rs`) is a `Copy` enum with variants
`MWorld`/`MWorldIT` — used instead of an `Arc<str>` name or a per-thread
cache because it costs nothing to construct or send across the channel, and
the batch path only ever needs these two fixed names.

### `BatchStats`

`render/thread/batch_stats.rs`, FFI-exposed getters for all six fields:
`entitiesSubmitted`, `entitiesVisible`, `entitiesCulled`, `totalEntities`,
`commandsGenerated`, `batchesProcessed`.

### Renderer FFI

`render/thread/renderer_ffi.rs`:

```lua
Renderer:beginBatch(view, projection, eyeX, eyeY, eyeZ)  -- starts accumulation, sets the culling camera
Renderer:addEntity(transform, boundsCenterX, boundsCenterY, boundsCenterZ,
                    boundsRadius, meshId, indexCount, shaderId, sortKey)
Renderer:flushBatch()                                    -- runs process_batch_intern, emits commands
Renderer:getBatchStats()                                 -- BatchStats for the active batch, or nil
```

`meshId`/`shaderId` are `ResourceId`s passed as plain `u64` (LuaJIT FFI has
no `ResourceId` type of its own). Obtain them from:

```lua
mesh:resourceId()      -- Mesh_ResourceId(self, Renderer); lazily creates the GPU resource if needed
shader:resourceId()    -- Shader_ResourceId(self); the shader's resource is already created eagerly
```

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
| Entity accumulator + camera | `render/thread/render_batch.rs` |
| Per-entity data | `render/thread/entity_render_data.rs` |
| Culling camera + frustum test | `render/thread/camera_render_data.rs` |
| Stats | `render/thread/batch_stats.rs` |
| Batch flow (`process_batch_intern`) + FFI | `render/thread/renderer_threaded.rs` / `renderer_immediate.rs`, `renderer_ffi.rs` |
| GPU instancing | `render/thread/instance_batch.rs`, `instance_data.rs` |
| Instanced GLSL | `res/shader/include/instanced.glsl`, `res/shader/vertex/wvp_instanced.glsl` |
| Obtaining `ResourceId`s from Lua | `Mesh::resource_id` (`render/mesh.rs`), `Shader::resource_id` (`render/shader.rs`) |
